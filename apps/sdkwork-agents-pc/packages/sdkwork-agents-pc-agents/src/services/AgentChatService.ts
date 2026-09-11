import {
  completeAgentTurn,
  completeAgentTurnStream,
  type TurnRichToolEvent,
} from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";
import {
  getAgentsAppSdkClientWithSession,
  type AgentItemFeedbackRecord,
  type AgentResourceUserStateRecord,
  type AgentSessionItemRecord,
  type AgentSessionRecord,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";
import type { AgentsDriveMediaResource } from "@sdkwork/agents-pc-core/sdk/driveUploadService";
import type { CreateAgentTurnRequest } from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";
import { sha256Hash, uuid } from "@sdkwork/utils";

import { resolveChatRuntimeModel } from "./RuntimeCatalogService";
import { sortSessionItems } from "./sessionMessageOrdering";

/**
 * LLM wire protocol used for the cloudrouter gateway invocation. Values mirror
 * the backend `wireProtocol` enum on `CreateAgentTurnRequest`; omitting the
 * field keeps the gateway default (`chat_completions`).
 */
export type AgentTurnWireProtocol = NonNullable<CreateAgentTurnRequest["wireProtocol"]>;

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  /** Reasoning/thinking text for this assistant turn (collapsible block). */
  reasoning?: string;
  createdAt: string;
  mediaResources?: AgentsDriveMediaResource[];
}

export interface ChatMessageListPage {
  items: ChatMessage[];
  pageInfo: {
    hasMore: boolean;
    nextCursor?: string;
  };
}

export interface ChatSessionSummary {
  id: string;
  title: string;
  updatedAt: string;
  version: string;
  projectId?: string;
}

export interface ChatSessionUserState {
  sessionId: string;
  pinned: boolean;
  version: string;
}

export interface ChatMessageFeedback {
  messageId: string;
  rating?: "up" | "down";
  version: string;
}

/** Matches the bounded server context window for one interactive session page. */
const SESSION_ITEM_PAGE_SIZE = 50;

function toChatMessage(record: AgentSessionItemRecord): ChatMessage {
  if (!record.itemId) {
    throw new Error("Agent session item did not include itemId.");
  }

  const role: ChatMessage["role"] = record.kind === "user_input"
    ? "user"
    : record.kind === "system_instruction"
        || record.kind === "status_notice"
        || record.kind === "error_notice"
      ? "system"
      : record.kind === "tool_call" || record.kind === "tool_result"
        ? "tool"
        : "assistant";

  return {
    id: record.itemId,
    role,
    content: record.content ?? "",
    createdAt: record.createdAt,
    mediaResources: record.driveRefs
      .filter((resource) => resource.status === "active")
      .map((resource): AgentsDriveMediaResource => ({
        id: resource.mediaResourceId ?? `${resource.driveSpaceId}:${resource.driveNodeId}`,
        kind: resource.resourceRole === "image"
          ? "image"
          : resource.resourceRole === "audio"
            ? "audio"
            : "other",
        source: "drive",
        uri: `drive://spaces/${resource.driveSpaceId}/nodes/${resource.driveNodeId}`,
        metadata: {
          driveSpaceId: resource.driveSpaceId,
          driveNodeId: resource.driveNodeId,
          drive: {
            spaceId: resource.driveSpaceId,
            nodeId: resource.driveNodeId,
          },
        },
      })),
  };
}

function toSessionId(record: AgentSessionRecord): string {
  return record.sessionId;
}

function findAssistantOutput(
  items: AgentSessionItemRecord[],
): AgentSessionItemRecord | undefined {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    const item = items[index];
    if (item.kind === "assistant_output") {
      return item;
    }
  }
  return undefined;
}

/**
 * Merges persisted `reasoning` session items into the adjacent assistant
 * message's `reasoning` field so chat UIs render them as a collapsible
 * thinking block instead of a standalone assistant bubble. This keeps the
 * message list aligned with the user/assistant pairs the optimistic
 * transcript produces (index-based server reconciliation depends on it).
 */
function mergeReasoningIntoAssistantMessages(
  items: AgentSessionItemRecord[],
): ChatMessage[] {
  const messages: ChatMessage[] = [];
  let pendingReasoning = "";
  for (const item of items) {
    if (item.kind === "reasoning") {
      pendingReasoning += item.content ?? "";
      continue;
    }
    const message = toChatMessage(item);
    if (message.role === "assistant" && pendingReasoning.length > 0) {
      message.reasoning = pendingReasoning;
      pendingReasoning = "";
    }
    messages.push(message);
  }
  return messages;
}

export class AgentChatService {
  constructor(
    private readonly getClient: () => SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession,
  ) {}

  async createSessionSummary(agentId: string, title?: string): Promise<ChatSessionSummary> {
    const idempotencyKey = uuid();
    const normalizedTitle = title?.trim() || "Agent session";
    const session = await this.getClient().ai.agents.sessions.create(agentId, {
      sessionKind: "assistant",
      entrySurface: "pc",
      title: normalizedTitle,
      idempotencyKey,
      payloadHash: `sha256:${sha256Hash(JSON.stringify({
        sessionKind: "assistant",
        entrySurface: "pc",
        title: normalizedTitle,
      }))}`,
      requestedAt: new Date().toISOString(),
    });
    const createdSessionId = toSessionId(session);
    if (!createdSessionId) {
      throw new Error("Chat session create did not return sessionId.");
    }
    return {
      id: createdSessionId,
      title: session.title ?? normalizedTitle,
      updatedAt: session.updatedAt,
      version: session.version,
      projectId: session.projectId ?? undefined,
    };
  }

  async createSession(agentId: string, title?: string): Promise<string> {
    const summary = await this.createSessionSummary(agentId, title);
    return summary.id;
  }

  async listSessions(agentId: string, pageSize = 10): Promise<string[]> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, { pageSize });
    return (response.items as AgentSessionRecord[])
      .map(toSessionId)
      .filter((sessionId) => sessionId.length > 0);
  }

  async listSessionSummaries(
    agentId: string,
    pageSize = 50,
  ): Promise<ChatSessionSummary[]> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, { pageSize });
    return (response.items as AgentSessionRecord[])
      .map((record) => ({
        id: toSessionId(record),
        title: record.title ?? "New chat",
        updatedAt: record.updatedAt,
        version: record.version,
        projectId: record.projectId ?? undefined,
      }))
      .filter((session) => session.id.length > 0);
  }

  async updateSession(
    agentId: string,
    sessionId: string,
    patch: { title?: string; projectId?: string; clearProject?: boolean; expectedVersion?: string },
  ): Promise<ChatSessionSummary> {
    const record = await this.getClient().ai.agents.sessions.update(agentId, sessionId, patch);
    return {
      id: toSessionId(record),
      title: record.title ?? "New chat",
      updatedAt: record.updatedAt,
      version: record.version,
      projectId: record.projectId ?? undefined,
    };
  }

  async deleteSession(agentId: string, sessionId: string): Promise<void> {
    await this.getClient().ai.agents.sessions.delete(agentId, sessionId);
  }

  async listSessionUserStates(
    agentId: string,
    pinnedOnly = false,
  ): Promise<ChatSessionUserState[]> {
    const response = await this.getClient().ai.agents.sessionUserStates.list(agentId, {
      page: 1,
      pageSize: 200,
      pinnedOnly,
    });
    return (response.items as AgentResourceUserStateRecord[])
      .map((record: AgentResourceUserStateRecord) => ({
        sessionId: record.resourceId,
        pinned: Boolean(record.pinnedAt),
        version: record.version,
      }))
      .filter((state) => state.sessionId.length > 0);
  }

  async updateSessionUserState(
    agentId: string,
    sessionId: string,
    patch: { pinned: boolean; expectedVersion?: string },
  ): Promise<ChatSessionUserState> {
    const record = await this.getClient().ai.agents.sessionUserStates.update(
      agentId,
      sessionId,
      patch,
    );
    return {
      sessionId: record.resourceId,
      pinned: Boolean(record.pinnedAt),
      version: record.version,
    };
  }

  async listMessageFeedback(
    agentId: string,
    sessionId: string,
  ): Promise<ChatMessageFeedback[]> {
    const response = await this.getClient().ai.agents.itemFeedback.list(agentId, sessionId, {
      page: 1,
      pageSize: SESSION_ITEM_PAGE_SIZE,
    });
    return (response.items as AgentItemFeedbackRecord[])
      .map((record: AgentItemFeedbackRecord) => ({
        messageId: record.itemId,
        rating: record.deletedAt ? undefined : record.rating,
        version: record.version,
      }))
      .filter((feedback) => feedback.messageId.length > 0);
  }

  async updateMessageFeedback(
    agentId: string,
    sessionId: string,
    messageId: string,
    patch: {
      rating?: "up" | "down";
      clearFeedback?: boolean;
      expectedVersion?: string;
    },
  ): Promise<ChatMessageFeedback> {
    const record = await this.getClient().ai.agents.itemFeedback.update(
      agentId,
      sessionId,
      messageId,
      patch,
    );
    return {
      messageId: record.itemId,
      rating: record.deletedAt ? undefined : record.rating,
      version: record.version,
    };
  }

  /** Reuse the latest active session when present; otherwise create one. */
  async resolveOrCreateSession(agentId: string, title?: string): Promise<string> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, {
      pageSize: 10,
    });
    const sessions = response.items as AgentSessionRecord[];
    const reusable = sessions.find((session) => {
      return session.status === "active";
    });
    if (reusable) {
      const sessionId = toSessionId(reusable);
      if (sessionId) {
        return sessionId;
      }
    }
    return this.createSession(agentId, title);
  }

  /**
   * Reuse an existing session with the requested id, otherwise create a new
   * one. Client-chosen session ids are not accepted by `sessions.create`
   * (B12 context-selector guard on the app surface), so a new session gets a
   * server-generated id; the caller keeps the returned id for the next turn.
   */
  async resolveOrCreateNamedSession(
    agentId: string,
    sessionId: string,
    title?: string,
  ): Promise<string> {
    const sessionIds = await this.listSessions(agentId, 200);
    if (sessionIds.includes(sessionId)) {
      return sessionId;
    }
    return this.createSession(agentId, title);
  }

  /** Load one server page for interactive chat history (`PAGINATION_SPEC.md` §8). */
  async listMessagesPage(
    agentId: string,
    sessionId: string,
    cursor?: string,
  ): Promise<ChatMessageListPage> {
    const response = await this.getClient().ai.agents.sessionItems.list(agentId, sessionId, {
      ...(cursor ? { cursor } : {}),
      pageSize: SESSION_ITEM_PAGE_SIZE,
      sort: "-sequence",
    });
    return {
      items: this.normalizeMessages(response.items as AgentSessionItemRecord[]),
      pageInfo: {
        hasMore: response.pageInfo.hasMore ?? false,
        nextCursor: response.pageInfo.nextCursor ?? undefined,
      },
    };
  }

  /** Load the newest transcript window; the cursor continues toward older items. */
  async loadRecentMessages(agentId: string, sessionId: string): Promise<ChatMessageListPage> {
    return this.listMessagesPage(agentId, sessionId);
  }

  async listMessages(agentId: string, sessionId: string): Promise<ChatMessage[]> {
    const page = await this.loadRecentMessages(agentId, sessionId);
    return page.items;
  }

  async sendMessage(
    agentId: string,
    sessionId: string,
    content: string,
    modelId?: string,
    media?: AgentsDriveMediaResource | AgentsDriveMediaResource[],
    systemPrompt?: string,
    wireProtocol?: AgentTurnWireProtocol,
  ): Promise<ChatMessage> {
    const completion = await completeAgentTurn(
      this.getClient(),
      agentId,
      sessionId,
      await this.buildTurnBody(content, modelId, media, systemPrompt, wireProtocol),
    );
    const assistantRecord = findAssistantOutput(completion.items);
    if (!assistantRecord) {
      throw new Error("Agent turn did not return an assistant_output item.");
    }
    return toChatMessage(assistantRecord);
  }

  /**
   * Streams one turn through the SSE protocol: `onDelta` receives incremental
   * text chunks, and the resolved assistant message is returned at the end.
   */
  async sendMessageStream(
    agentId: string,
    sessionId: string,
    content: string,
    modelId: string | undefined,
    media: AgentsDriveMediaResource | AgentsDriveMediaResource[] | undefined,
    onDelta: (delta: string) => void,
    systemPrompt?: string,
    onReasoning?: (reasoning: string) => void,
    onToolEvent?: (event: TurnRichToolEvent) => void,
    wireProtocol?: AgentTurnWireProtocol,
  ): Promise<ChatMessage> {
    const completion = await completeAgentTurnStream(
      this.getClient(),
      agentId,
      sessionId,
      await this.buildTurnBody(content, modelId, media, systemPrompt, wireProtocol),
      { onDelta, onReasoning, onToolEvent },
    );
    const assistantRecord = findAssistantOutput(completion.items);
    if (!assistantRecord) {
      throw new Error("Agent turn stream did not return an assistant_output item.");
    }
    return toChatMessage(assistantRecord);
  }

  private async buildTurnBody(
    content: string,
    modelId?: string,
    media?: AgentsDriveMediaResource | AgentsDriveMediaResource[],
    systemPrompt?: string,
    wireProtocol?: AgentTurnWireProtocol,
  ): Promise<CreateAgentTurnRequest> {
    const mediaResources = media ? (Array.isArray(media) ? media : [media]) : [];
    const requestId = uuid();
    const driveRefs = mediaResources.map((item) => {
      const driveSpaceId = item.metadata.driveSpaceId ?? item.metadata.drive?.spaceId;
      const driveNodeId = item.metadata.driveNodeId ?? item.metadata.drive?.nodeId;
      if (!driveSpaceId || !driveNodeId) {
        throw new Error("Drive attachment is missing driveSpaceId or driveNodeId.");
      }
      return {
        resourceRole: item.kind === "image" ? "image" as const : item.kind === "audio" ? "audio" as const : "attachment" as const,
        driveSpaceId,
        driveNodeId,
      };
    });
    // Resolve the runtime model id for the turn request. No local provider
    // binding or API key is required: chat turns route through the cloudrouter
    // account-pool gateway using the caller's auth token (sessions that
    // already carry a runtime binding keep the local binding chain).
    const runtimeModel = await resolveChatRuntimeModel(modelId, this.getClient());
    const contentType = mediaResources[0]?.mimeType ?? "text/plain";
    const payloadHash = `sha256:${sha256Hash(JSON.stringify({
      content: content.trim(),
      contentType,
      requestedModelId: runtimeModel.id,
      driveRefs,
    }))}`;
    return {
      content: content.trim(),
      contentType,
      turnMode: "interactive" as const,
      ...(systemPrompt ? { systemPrompt: systemPrompt.trim() } : {}),
      ...(wireProtocol ? { wireProtocol } : {}),
      ...(driveRefs.length > 0 ? { driveRefs } : {}),
      requestedAt: new Date().toISOString(),
      idempotencyKey: requestId,
      payloadHash,
      clientRequestId: requestId,
      requestedModelId: runtimeModel.id,
    };
  }

  private normalizeMessages(items: AgentSessionItemRecord[]): ChatMessage[] {
    return mergeReasoningIntoAssistantMessages(sortSessionItems(items));
  }
}

export const agentChatService = new AgentChatService();
