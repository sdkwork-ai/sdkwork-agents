import assert from 'node:assert/strict';
import test from 'node:test';

import {
  ChatService,
  configureChatAgentPort,
  type ChatAgentPort,
} from '../packages/sdkwork-agents-pc-chat/src/services/ChatService';
import {
  bootstrapChatSessions,
  resetChatBootstrapForTests,
} from '../packages/sdkwork-agents-pc-chat/src/services/chatBootstrap';

function createChatAgentPort(overrides: Partial<ChatAgentPort>): ChatAgentPort {
  const session = {
    id: 'session.1',
    title: 'Session 1',
    updatedAt: new Date().toISOString(),
    version: '1',
  };
  return {
    getAgent: async () => ({ model: 'model-a' }),
    createAgent: async () => ({ model: 'model-a' }),
    updateAgent: async () => ({ model: 'model-a' }),
    resolveOrCreateSession: async (_agentId, sessionId) => sessionId,
    createSession: async (_agentId, title) => ({
      id: 'session.created',
      title,
      updatedAt: '2026-08-28T00:00:00.000Z',
      version: '2',
    }),
    listSessions: async () => [],
    updateSession: async (_agentId, sessionId) => ({ ...session, id: sessionId }),
    deleteSession: async () => undefined,
    listSessionUserStates: async () => [],
    updateSessionUserState: async (_agentId, sessionId) => ({
      sessionId,
      pinned: false,
      version: '1',
    }),
    listMessageFeedback: async () => [],
    updateMessageFeedback: async (_agentId, _sessionId, messageId) => ({
      messageId,
      rating: undefined,
      version: '1',
    }),
    listMessages: async () => [],
    resolveMediaPreviewUrl: async (driveUri) => driveUri,
    sendMessage: async () => ({ id: 'm1', content: 'ok' }),
    ...overrides,
  };
}

test('bootstrapChatSessions creates one empty session when history is empty', async () => {
  resetChatBootstrapForTests();
  let createCount = 0;
  configureChatAgentPort(createChatAgentPort({
    createSession: async (_agentId, title) => {
      createCount += 1;
      return {
        id: `session.created-${createCount}`,
        title,
        updatedAt: '2026-08-28T00:00:00.000Z',
        version: '2',
      };
    },
  }));

  const first = await bootstrapChatSessions('model-a', 'New chat');
  const second = await bootstrapChatSessions('model-a', 'New chat');

  assert.equal(createCount, 1);
  assert.equal(first.currentSessionId, second.currentSessionId);
  assert.equal(first.sessions.length, 1);
});

test('bootstrapChatSessions reuses listed sessions without creating a new one', async () => {
  resetChatBootstrapForTests();
  let createCount = 0;
  configureChatAgentPort(createChatAgentPort({
    listSessions: async () => [{
      id: 'session.1',
      title: 'Existing',
      updatedAt: '2026-08-28T00:00:00.000Z',
      version: '1',
    }],
    createSession: async () => {
      createCount += 1;
      return {
        id: 'session.created',
        title: 'New chat',
        updatedAt: '2026-08-28T00:00:00.000Z',
        version: '2',
      };
    },
  }));

  const bootstrapped = await bootstrapChatSessions('model-a', 'New chat');
  assert.equal(createCount, 0);
  assert.equal(bootstrapped.currentSessionId, 'session.1');
});
