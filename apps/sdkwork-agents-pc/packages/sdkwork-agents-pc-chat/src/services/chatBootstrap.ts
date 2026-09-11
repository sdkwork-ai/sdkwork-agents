import { ChatService, DEFAULT_CHAT_AGENT_SCOPE, type ChatAgentScope } from '../services/ChatService';

export interface ChatBootstrapResult {
  sessions: Awaited<ReturnType<typeof ChatService.loadSessions>>;
  currentSessionId: string;
}

const bootstrapPromises = new Map<string, Promise<ChatBootstrapResult>>();

function bootstrapCacheKey(scope: ChatAgentScope): string {
  return scope.agentId;
}

/**
 * Ensures session bootstrap runs once per page load (React Strict Mode safe).
 * Concurrent callers share the same in-flight promise.
 */
export function bootstrapChatSessions(
  model: string,
  newChatTitle: string,
  scope?: ChatAgentScope,
): Promise<ChatBootstrapResult> {
  const resolvedScope = scope ?? DEFAULT_CHAT_AGENT_SCOPE;
  const cacheKey = bootstrapCacheKey(resolvedScope);
  const existing = bootstrapPromises.get(cacheKey);
  if (existing) {
    return existing;
  }

  const promise = (async () => {
    const remoteSessions = await ChatService.loadSessions(model, resolvedScope);
    if (remoteSessions.length === 0) {
      const created = await ChatService.createSession(model, newChatTitle, resolvedScope);
      return {
        sessions: [created],
        currentSessionId: created.id,
      };
    }
    return {
      sessions: remoteSessions,
      currentSessionId: remoteSessions[0].id,
    };
  })().catch((error) => {
    bootstrapPromises.delete(cacheKey);
    throw error;
  });

  bootstrapPromises.set(cacheKey, promise);
  return promise;
}

/** Test-only reset for isolated unit tests. */
export function resetChatBootstrapForTests(): void {
  bootstrapPromises.clear();
}
