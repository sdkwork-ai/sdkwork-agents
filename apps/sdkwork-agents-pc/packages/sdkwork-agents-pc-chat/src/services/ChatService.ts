import type { ChatMessage, ChatToolStreamEvent } from '../types';
import { trimSessionTitle } from '../utils/sessionTitleUtils';
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

export interface ChatAgentScope {
  agentId: string;
  title?: string;
  systemPrompt?: string;
  welcomeMessage?: string;
}

export const DEFAULT_CHAT_AGENT_ID = 'agent.chat.default';

export const DEFAULT_CHAT_AGENT_SCOPE: ChatAgentScope = {
  agentId: DEFAULT_CHAT_AGENT_ID,
};

export function createChatAgentScope(
  agentId: string,
  overrides: Omit<ChatAgentScope, 'agentId'> = {},
): ChatAgentScope {
  return { agentId, ...overrides };
}

export function isDefaultChatAgentScope(scope: ChatAgentScope): boolean {
  return scope.agentId === DEFAULT_CHAT_AGENT_ID;
}

function resolveChatAgentScope(scope?: ChatAgentScope): ChatAgentScope {
  return scope ?? DEFAULT_CHAT_AGENT_SCOPE;
}

export interface ChatServiceOptions {
  sessionId: string;
  model: string;
  messages: ChatMessage[];
  signal?: AbortSignal;
  scope?: ChatAgentScope;
  /** Optional LLM wire protocol for the cloudrouter gateway (defaults to chat_completions). */
  wireProtocol?: string;
  onMessageUpdate: (text: string) => void;
  /** Reasoning/thinking delta streamed for the assistant message. */
  onReasoning?: (reasoning: string) => void;
  /** Tool/skill/MCP invocation lifecycle event for the assistant message. */
  onToolEvent?: (event: ChatToolStreamEvent) => void;
  onComplete?: (message?: { id: string }) => void;
  onError?: (failure: ChatSendFailure) => void;
}

let chatAgentPort: ChatAgentPort | null = null;

// Cached agent records are session-stable, so loading one on every send is wasteful.
const CHAT_AGENT_CACHE_TTL_MS = 5 * 60 * 1000;
const chatAgentCacheById = new Map<string, { agent: ChatAgentRecord | null; expiresAt: number }>();

export interface ChatAgentConfig {
  id: string;
  name: string;
  description: string;
  type: 'normal';
  model: string;
  systemPrompt: string;
  welcomeMessage: string;
}

interface ChatAgentRecord {
  model?: string;
  systemPrompt?: string;
  welcomeMessage?: string;
  name?: string;
}

export interface ChatAgentPort {
  getAgent(agentId: string): Promise<ChatAgentRecord | null>;
  createAgent(agent: ChatAgentConfig): Promise<unknown>;
  updateAgent(agentId: string, patch: { model: string }): Promise<unknown>;
  resolveOrCreateSession(agentId: string, sessionId: string, title: string): Promise<string>;
  createSession(agentId: string, title: string): Promise<{
    id: string;
    title: string;
    updatedAt: string;
    version: string;
  }>;
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
    /** Reasoning/thinking text for this assistant turn (collapsible block). */
    reasoning?: string;
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
    /** Optional LLM wire protocol for the cloudrouter gateway (defaults to chat_completions). */
    wireProtocol?: string,
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
    onReasoning?: (reasoning: string) => void,
    onToolEvent?: (event: ChatToolStreamEvent) => void,
    /** Optional LLM wire protocol for the cloudrouter gateway (defaults to chat_completions). */
    wireProtocol?: string,
  ): Promise<{ id: string; content: string }>;
}

export function configureChatAgentPort(port: ChatAgentPort): void {
  chatAgentPort = port;
  chatAgentCacheById.clear();
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

async function ensureChatAgent(model: string, scope: ChatAgentScope): Promise<void> {
  const port = requireChatAgentPort();
  const now = Date.now();
  const cacheKey = scope.agentId;
  const cached = chatAgentCacheById.get(cacheKey);
  const current = cached && cached.expiresAt > now
    ? cached.agent
    : await port.getAgent(scope.agentId);
  if (!cached) {
    chatAgentCacheById.set(cacheKey, { agent: current, expiresAt: now + CHAT_AGENT_CACHE_TTL_MS });
  }

  if (!isDefaultChatAgentScope(scope)) {
    if (!current) {
      throw new Error(`Agent ${scope.agentId} is not available.`);
    }
    return;
  }

  if (!current) {
    const created = await port.createAgent(defaultAgent(model));
    chatAgentCacheById.set(cacheKey, {
      agent: created as ChatAgentRecord | null,
      expiresAt: now + CHAT_AGENT_CACHE_TTL_MS,
    });
    return;
  }
  if (model && current.model !== model) {
    if (!callerScopeGrantsAgentManage(chatAgentPermissionScopeReader())) {
      return;
    }
    try {
      await port.updateAgent(DEFAULT_CHAT_AGENT_ID, { model });
      chatAgentCacheById.set(cacheKey, { agent: { model }, expiresAt: now + CHAT_AGENT_CACHE_TTL_MS });
    } catch (error) {
      console.warn('Failed to sync the default chat agent model', error);
    }
  }
}

function canonicalSessionId(sessionId: string): string {
  const normalized = sessionId.trim().toLowerCase().replace(/[^a-z0-9_-]/gu, '-');
  return sessionId.startsWith('session.') ? sessionId : `session.${normalized}`;
}

function isPersistedServerSessionId(sessionId: string): boolean {
  return sessionId.trim().startsWith('session.');
}

function resolvedSessionCacheKey(agentId: string, canonicalSessionIdValue: string): string {
  return `${agentId}:${canonicalSessionIdValue}`;
}

function rememberResolvedSessionId(agentId: string, sessionId: string): string {
  const trimmed = sessionId.trim();
  const canonical = canonicalSessionId(trimmed);
  resolvedSessionIdByChatId.set(resolvedSessionCacheKey(agentId, canonical), trimmed);
  return trimmed;
}

const resolvedSessionIdByChatId = new Map<string, string>();

async function resolveSession(
  model: string,
  localSessionId: string,
  scope: ChatAgentScope,
): Promise<string> {
  await ensureChatAgent(model, scope);
  const canonical = canonicalSessionId(localSessionId);
  const cacheKey = resolvedSessionCacheKey(scope.agentId, canonical);
  const cached = resolvedSessionIdByChatId.get(cacheKey);
  if (cached) {
    return cached;
  }
  if (isPersistedServerSessionId(localSessionId)) {
    return rememberResolvedSessionId(scope.agentId, localSessionId);
  }
  const resolved = await requireChatAgentPort().resolveOrCreateSession(
    scope.agentId,
    canonical,
    scope.title ?? 'SDKWork Agents',
  );
  resolvedSessionIdByChatId.set(cacheKey, resolved);
  return resolved;
}

function resolveSystemPrompt(model: string, scope: ChatAgentScope): string {
  if (scope.systemPrompt?.trim()) {
    return scope.systemPrompt.trim();
  }
  return defaultAgent(model).systemPrompt;
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
  /** Creates a server-backed session immediately (e.g. on "New chat"). */
  static async createSession(
    model: string,
    title?: string,
    scope?: ChatAgentScope,
  ): Promise<{
    id: string;
    title: string;
    updatedAt: number;
    version: string;
    messages: ChatMessage[];
  }> {
    const resolvedScope = resolveChatAgentScope(scope);
    await ensureChatAgent(model, resolvedScope);
    const created = await requireChatAgentPort().createSession(
      resolvedScope.agentId,
      trimSessionTitle(title?.trim() || 'New chat'),
    );
    rememberResolvedSessionId(resolvedScope.agentId, created.id);
    return {
      id: created.id,
      title: created.title,
      updatedAt: Date.parse(created.updatedAt) || Date.now(),
      version: created.version,
      messages: [],
    };
  }

  static async loadSessions(
    model: string,
    scope?: ChatAgentScope,
  ): Promise<Array<{
    id: string;
    title: string;
    updatedAt: number;
    version: string;
    projectId?: string;
    messages: ChatMessage[];
  }>> {
    const resolvedScope = resolveChatAgentScope(scope);
    await ensureChatAgent(model, resolvedScope);
    const port = requireChatAgentPort();
    const [sessions, userStates] = await Promise.all([
      port.listSessions(resolvedScope.agentId),
      port.listSessionUserStates(resolvedScope.agentId, true),
    ]);
    const userStateBySessionId = new Map(
      userStates.map((state) => [state.sessionId, state]),
    );
    for (const session of sessions) {
      rememberResolvedSessionId(resolvedScope.agentId, session.id);
    }
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
    }).sort((left, right) => right.updatedAt - left.updatedAt);
  }

  /** Loads one session transcript (messages + feedback) on demand. */
  static async loadSessionDetail(sessionId: string, scope?: ChatAgentScope): Promise<ChatMessage[]> {
    const resolvedScope = resolveChatAgentScope(scope);
    const port = requireChatAgentPort();
    const [messages, feedbackItems] = await Promise.all([
      port.listMessages(resolvedScope.agentId, canonicalSessionId(sessionId)),
      port.listMessageFeedback(resolvedScope.agentId, canonicalSessionId(sessionId)),
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
        reasoning: message.reasoning,
        images: mediaResources
          .filter((resource) => resource.kind === 'image' && resource.url)
          .map((resource) => resource.url as string),
        mediaResources,
        feedback: feedback?.rating,
        feedbackVersion: feedback?.version,
      };
    }));
  }

  static async setSessionPinned(
    sessionId: string,
    pinned: boolean,
    version?: string,
    scope?: ChatAgentScope,
  ) {
    const resolvedScope = resolveChatAgentScope(scope);
    return requireChatAgentPort().updateSessionUserState(
      resolvedScope.agentId,
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
    scope?: ChatAgentScope,
  ) {
    const resolvedScope = resolveChatAgentScope(scope);
    return requireChatAgentPort().updateMessageFeedback(
      resolvedScope.agentId,
      canonicalSessionId(sessionId),
      messageId,
      rating
        ? { rating, ...(version ? { expectedVersion: version } : {}) }
        : { clearFeedback: true, ...(version ? { expectedVersion: version } : {}) },
    );
  }

  static async renameSession(
    sessionId: string,
    title: string,
    version: string,
    scope?: ChatAgentScope,
  ) {
    const resolvedScope = resolveChatAgentScope(scope);
    return requireChatAgentPort().updateSession(resolvedScope.agentId, canonicalSessionId(sessionId), {
      title: trimSessionTitle(title),
      ...(version ? { expectedVersion: version } : {}),
    });
  }

  static async moveSession(
    sessionId: string,
    projectId: string,
    version: string,
    scope?: ChatAgentScope,
  ) {
    const resolvedScope = resolveChatAgentScope(scope);
    return requireChatAgentPort().updateSession(resolvedScope.agentId, canonicalSessionId(sessionId), {
      projectId,
      ...(version ? { expectedVersion: version } : {}),
    });
  }

  static async deleteSession(sessionId: string, scope?: ChatAgentScope): Promise<void> {
    const resolvedScope = resolveChatAgentScope(scope);
    const canonical = canonicalSessionId(sessionId);
    await requireChatAgentPort().deleteSession(resolvedScope.agentId, canonical);
    resolvedSessionIdByChatId.delete(resolvedSessionCacheKey(resolvedScope.agentId, canonical));
  }

  static async streamChat(options: ChatServiceOptions): Promise<void> {
    const resolvedScope = resolveChatAgentScope(options.scope);
    if (!options.sessionId.trim()) {
      options.onError?.({ message: 'A chat session is required.' });
      return;
    }
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
      const sessionId = await resolveSession(options.model, options.sessionId, resolvedScope);
      if (options.signal?.aborted) {
        options.onError?.({ message: 'AbortError' });
        return;
      }
      const port = requireChatAgentPort();
      const content = latest.text
        || latest.mediaResources?.map((item) => item.fileName ?? item.id).join(', ')
        || 'Attachment';
      const systemPrompt = resolveSystemPrompt(options.model, resolvedScope);
      const response = port.sendMessageStream
        ? await port.sendMessageStream(
            resolvedScope.agentId,
            sessionId,
            content,
            options.model,
            latest.mediaResources,
            (delta) => options.onMessageUpdate(delta),
            systemPrompt,
            (reasoning) => options.onReasoning?.(reasoning),
            (event) => options.onToolEvent?.(event),
            options.wireProtocol,
          )
        : await port.sendMessage(
            resolvedScope.agentId,
            sessionId,
            content,
            options.model,
            latest.mediaResources,
            systemPrompt,
            options.wireProtocol,
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
      console.warn(
        `[agents-chat] turn failed: ${failure.message}`,
        { httpStatus: failure.httpStatus, i18nKey: failure.i18nKey, code: failure.code, traceId: failure.traceId },
      );
      options.onError?.(failure);
    }
  }
}
