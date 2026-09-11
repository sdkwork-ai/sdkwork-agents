import type { ChatMessage } from '../types';
import type { AgentsDriveMediaResource } from '@sdkwork/agents-pc-core/sdk/driveUploadService';
import { createSdkworkChatRequestContext } from '@sdkwork/agents-pc-core/session';

export interface ChatSendFailure {
  /** Fallback display message (the SDK error message, already safe). */
  message: string;
  /** Backend problem i18n key (e.g. `errors.result.50301`) when available. */
  i18nKey?: string;
  /** Backend problem numeric code (e.g. `50301`) for `errors.result.<code>`. */
  code?: number | string;
  httpStatus?: number;
  traceId?: string;
}

export interface ChatServiceOptions {
  sessionId: string;
  model: string;
  messages: ChatMessage[];
  signal?: AbortSignal;
  onMessageUpdate: (text: string) => void;
  onComplete?: (message?: { id: string }) => void;
  onError?: (failure: ChatSendFailure) => void;
}

const DEFAULT_CHAT_AGENT_ID = 'agent.chat.default';
let chatAgentPort: ChatAgentPort | null = null;

// Cached default chat agent record: the agent is session-stable, so loading
// it on every send (one GET per message) is wasteful. Creation/model-sync
// update the cache so a changed default is picked up within the session.
const CHAT_AGENT_CACHE_TTL_MS = 5 * 60 * 1000;
let chatAgentCache: { agent: { model?: string } | null; expiresAt: number } | null = null;

export interface ChatAgentConfig {
  id: string;
  name: string;
  description: string;
  type: 'normal';
  model: string;
  systemPrompt: string;
  welcomeMessage: string;
}

export interface ChatAgentPort {
  getAgent(agentId: string): Promise<{ model?: string } | null>;
  createAgent(agent: ChatAgentConfig): Promise<unknown>;
  updateAgent(agentId: string, patch: { model: string }): Promise<unknown>;
  resolveOrCreateSession(agentId: string, sessionId: string, title: string): Promise<string>;
  listSessions(agentId: string): Promise<Array<{
    id: string;
    title: string;
    updatedAt: string;
    version: string;
    projectId?: string;
  }>>;
  updateSession(
    agentId: string,
    sessionId: string,
    patch: { title?: string; projectId?: string; clearProject?: boolean; expectedVersion?: string },
  ): Promise<{ id: string; title: string; updatedAt: string; version: string; projectId?: string }>;
  deleteSession(agentId: string, sessionId: string): Promise<void>;
  listSessionUserStates(agentId: string, pinnedOnly?: boolean): Promise<Array<{
    sessionId: string;
    pinned: boolean;
    version: string;
  }>>;
  updateSessionUserState(
    agentId: string,
    sessionId: string,
    patch: { pinned: boolean; expectedVersion?: string },
  ): Promise<{ sessionId: string; pinned: boolean; version: string }>;
  listMessageFeedback(agentId: string, sessionId: string): Promise<Array<{
    messageId: string;
    rating?: 'up' | 'down';
    version: string;
  }>>;
  updateMessageFeedback(
    agentId: string,
    sessionId: string,
    messageId: string,
    patch: { rating?: 'up' | 'down'; clearFeedback?: boolean; expectedVersion?: string },
  ): Promise<{ messageId: string; rating?: 'up' | 'down'; version: string }>;
  listMessages(
    agentId: string,
    sessionId: string,
  ): Promise<Array<{
    id: string;
    role: 'user' | 'assistant' | 'system' | 'tool';
    content: string;
    mediaResources?: AgentsDriveMediaResource[];
  }>>;
  resolveMediaPreviewUrl(driveUri: string): Promise<string>;
  sendMessage(
    agentId: string,
    sessionId: string,
    content: string,
    model: string,
    media?: AgentsDriveMediaResource[],
    systemPrompt?: string,
  ): Promise<{ id: string; content: string }>;
  /** Optional SSE streaming variant: deltas are delivered via `onDelta`. */
  sendMessageStream?(
    agentId: string,
    sessionId: string,
    content: string,
    model: string,
    media: AgentsDriveMediaResource[] | undefined,
    onDelta: (delta: string) => void,
    systemPrompt?: string,
  ): Promise<{ id: string; content: string }>;
}

export function configureChatAgentPort(port: ChatAgentPort): void {
  chatAgentPort = port;
  // A new port implementation may target a different backend, so the cached
  // agent record and resolved session ids must not leak across reconfigures
  // (also keeps tests isolated).
  chatAgentCache = null;
  resolvedSessionIdByChatId.clear();
}

function requireChatAgentPort(): ChatAgentPort {
  if (!chatAgentPort) {
    throw new Error('Chat agent port is not configured.');
  }
  return chatAgentPort;
}

export type ChatAgentPermissionScopeReader = () => string[];

function readChatAgentPermissionScopeFromSession(): string[] {
  return createSdkworkChatRequestContext()?.permissionScope ?? [];
}

let chatAgentPermissionScopeReader: ChatAgentPermissionScopeReader =
  readChatAgentPermissionScopeFromSession;

/** Overrides where the caller's IAM permission scope is read from. */
export function configureChatAgentPermissionScopeReader(
  reader: ChatAgentPermissionScopeReader,
): void {
  chatAgentPermissionScopeReader = reader;
}

/**
 * Mirrors the backend `IamGatedPolicyProvider` grant rules: updating an agent
 * record (`agents.update`) requires `ai.agents.manage`, with `ai.*` and `*`
 * wildcards also granting it. Chat-only callers must not attempt the model
 * sync PATCH at all, since the API contract rejects it with 403.
 */
export function callerScopeGrantsAgentManage(scopes: string[]): boolean {
  return scopes.some(
    (scope) => scope === 'ai.agents.manage' || scope === 'ai.*' || scope === '*',
  );
}

function defaultAgent(model: string): ChatAgentConfig {
  return {
    id: DEFAULT_CHAT_AGENT_ID,
    name: 'SDKWork Agents',
    description: 'SDKWork Agents PC built-in conversational assistant.',
    type: 'normal',
    model,
    systemPrompt: 'You are SDKWork Agents. Provide accurate, concise, secure, and useful answers.',
    welcomeMessage: 'How can I help?',
  };
}

async function ensureChatAgent(model: string): Promise<void> {
  const port = requireChatAgentPort();
  const now = Date.now();
  const cached = chatAgentCache && chatAgentCache.expiresAt > now
    ? chatAgentCache.agent
    : undefined;
  const current = cached ?? await port.getAgent(DEFAULT_CHAT_AGENT_ID);
  if (cached === undefined) {
    chatAgentCache = { agent: current, expiresAt: now + CHAT_AGENT_CACHE_TTL_MS };
  }
  if (!current) {
    const created = await port.createAgent(defaultAgent(model));
    chatAgentCache = { agent: created as { model?: string } | null, expiresAt: now + CHAT_AGENT_CACHE_TTL_MS };
    return;
  }
  if (model && current.model !== model) {
    // The model is passed per message through sendMessage, so syncing the
    // stored default is only worthwhile for callers the backend allows to
    // update the agent. Without manage scope the PATCH would always 403.
    if (!callerScopeGrantsAgentManage(chatAgentPermissionScopeReader())) {
      return;
    }
    try {
      await port.updateAgent(DEFAULT_CHAT_AGENT_ID, { model });
      chatAgentCache = { agent: { model }, expiresAt: now + CHAT_AGENT_CACHE_TTL_MS };
    } catch (error) {
      // Best-effort: a failed model sync (e.g. a stale scope claim) must not
      // block session loading or chat.
      console.warn('Failed to sync the default chat agent model', error);
    }
  }
}

function canonicalSessionId(sessionId: string): string {
  const normalized = sessionId.trim().toLowerCase().replace(/[^a-z0-9_-]/gu, '-');
  return sessionId.startsWith('session.') ? sessionId : `session.${normalized}`;
}

// Maps the canonical local chat id to the server session id resolved for it.
// `sessions.create` does not accept client-chosen ids (B12 context-selector
// guard), so the first turn in a local chat must remember the server-generated
// session id to keep later turns in the same conversation.
const resolvedSessionIdByChatId = new Map<string, string>();

async function resolveSession(model: string, localSessionId: string): Promise<string> {
  await ensureChatAgent(model);
  const canonical = canonicalSessionId(localSessionId);
  const cached = resolvedSessionIdByChatId.get(canonical);
  if (cached) {
    return cached;
  }
  const resolved = await requireChatAgentPort().resolveOrCreateSession(
    DEFAULT_CHAT_AGENT_ID,
    canonical,
    'SDKWork Agents',
  );
  resolvedSessionIdByChatId.set(canonical, resolved);
  return resolved;
}

function toChatSendFailure(error: unknown): ChatSendFailure {
  if (error instanceof Error) {
    const problem = (error as { problem?: { i18nKey?: string; code?: number | string } }).problem;
    const httpStatus = (error as { httpStatus?: number }).httpStatus;
    const traceId = (error as { traceId?: string }).traceId;
    return {
      message: error.message,
      i18nKey: problem?.i18nKey,
      code: problem?.code,
      httpStatus,
      traceId,
    };
  }
  return { message: 'Agents chat request failed.' };
}

export class ChatService {
  static async loadSessions(model: string): Promise<Array<{
    id: string;
    title: string;
    updatedAt: number;
    version: string;
    projectId?: string;
    messages: ChatMessage[];
  }>> {
    await ensureChatAgent(model);
    const port = requireChatAgentPort();
    const [sessions, userStates] = await Promise.all([
      port.listSessions(DEFAULT_CHAT_AGENT_ID),
      port.listSessionUserStates(DEFAULT_CHAT_AGENT_ID, true),
    ]);
    const userStateBySessionId = new Map(
      userStates.map((state) => [state.sessionId, state]),
    );
    // Lazy detail: transcripts and feedback load per selected session
    // (`loadSessionDetail`) instead of fanning out 2N+2 requests here.
    return sessions.map((session) => {
      const userState = userStateBySessionId.get(session.id);
      return {
        id: session.id,
        title: session.title,
        updatedAt: Date.parse(session.updatedAt) || 0,
        version: session.version,
        projectId: session.projectId,
        pinned: userState?.pinned ?? false,
        userStateVersion: userState?.version,
        messages: [],
      };
    });
  }

  /** Loads one session transcript (messages + feedback) on demand. */
  static async loadSessionDetail(sessionId: string): Promise<ChatMessage[]> {
    const port = requireChatAgentPort();
    const [messages, feedbackItems] = await Promise.all([
      port.listMessages(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId)),
      port.listMessageFeedback(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId)),
    ]);
    const feedbackByMessageId = new Map(
      feedbackItems.map((feedback) => [feedback.messageId, feedback]),
    );
    return Promise.all(messages.map(async (message) => {
      const feedback = feedbackByMessageId.get(message.id);
      const mediaResources = await Promise.all(
        (message.mediaResources ?? []).map(async (resource) => {
          try {
            const url = await port.resolveMediaPreviewUrl(resource.uri);
            return { ...resource, url };
          } catch {
            return resource;
          }
        }),
      );
      return {
        id: message.id,
        role: message.role === 'assistant' ? 'model' : 'user',
        text: message.content,
        images: mediaResources
          .filter((resource) => resource.kind === 'image' && resource.url)
          .map((resource) => resource.url as string),
        mediaResources,
        feedback: feedback?.rating,
        feedbackVersion: feedback?.version,
      };
    }));
  }

  static async setSessionPinned(sessionId: string, pinned: boolean, version?: string) {
    return requireChatAgentPort().updateSessionUserState(
      DEFAULT_CHAT_AGENT_ID,
      canonicalSessionId(sessionId),
      {
        pinned,
        ...(version ? { expectedVersion: version } : {}),
      },
    );
  }

  static async setMessageFeedback(
    sessionId: string,
    messageId: string,
    rating: 'up' | 'down' | undefined,
    version?: string,
  ) {
    return requireChatAgentPort().updateMessageFeedback(
      DEFAULT_CHAT_AGENT_ID,
      canonicalSessionId(sessionId),
      messageId,
      rating
        ? { rating, ...(version ? { expectedVersion: version } : {}) }
        : { clearFeedback: true, ...(version ? { expectedVersion: version } : {}) },
    );
  }

  static async renameSession(sessionId: string, title: string, version: string) {
    return requireChatAgentPort().updateSession(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId), {
      title,
      ...(version ? { expectedVersion: version } : {}),
    });
  }

  static async moveSession(sessionId: string, projectId: string, version: string) {
    return requireChatAgentPort().updateSession(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId), {
      projectId,
      ...(version ? { expectedVersion: version } : {}),
    });
  }

  static async deleteSession(sessionId: string): Promise<void> {
    await requireChatAgentPort().deleteSession(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId));
  }

  static async streamChat(options: ChatServiceOptions): Promise<void> {
    if (options.signal?.aborted) {
      options.onError?.({ message: 'AbortError' });
      return;
    }

    const latest = options.messages.at(-1);
    if (!latest || latest.role !== 'user') {
      options.onError?.({ message: 'A user message is required.' });
      return;
    }

    try {
      const sessionId = await resolveSession(options.model, options.sessionId);
      if (options.signal?.aborted) {
        options.onError?.({ message: 'AbortError' });
        return;
      }
      const port = requireChatAgentPort();
      const content = latest.text
        || latest.mediaResources?.map((item) => item.fileName ?? item.id).join(', ')
        || 'Attachment';
      const systemPrompt = defaultAgent(options.model).systemPrompt;
      const response = port.sendMessageStream
        ? await port.sendMessageStream(
            DEFAULT_CHAT_AGENT_ID,
            sessionId,
            content,
            options.model,
            latest.mediaResources,
            (delta) => options.onMessageUpdate(delta),
            systemPrompt,
          )
        : await port.sendMessage(
            DEFAULT_CHAT_AGENT_ID,
            sessionId,
            content,
            options.model,
            latest.mediaResources,
            systemPrompt,
          );
      if (options.signal?.aborted) {
        options.onError?.({ message: 'AbortError' });
        return;
      }
      if (!port.sendMessageStream) {
        options.onMessageUpdate(response.content);
      }
      options.onComplete?.({ id: response.id });
    } catch (error) {
      const failure = toChatSendFailure(error);
      // Keep the console line compact and parseable: the user-facing message
      // is rendered in the message list (translated via the problem i18n key
      // or `errors.result.<code>`), so only correlation info is logged here.
      console.warn(
        `[agents-chat] turn failed: ${failure.message}`,
        { httpStatus: failure.httpStatus, i18nKey: failure.i18nKey, code: failure.code, traceId: failure.traceId },
      );
      options.onError?.(failure);
    }
  }
}
