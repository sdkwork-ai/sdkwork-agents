import assert from 'node:assert/strict';
import test from 'node:test';

import {
  ChatService,
  configureChatAgentPort,
  type ChatAgentPort,
} from '../packages/sdkwork-agents-pc-chat/src/services/ChatService';

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
    listSessions: async () => [session],
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

test('createSession persists immediately and returns server metadata', async () => {
  let createCalled = false;
  configureChatAgentPort(createChatAgentPort({
    createSession: async (_agentId, title) => {
      createCalled = true;
      return {
        id: 'session.created',
        title,
        updatedAt: '2026-08-28T00:00:00.000Z',
        version: '2',
      };
    },
  }));

  const created = await ChatService.createSession('model-a', 'New chat');
  assert.equal(createCalled, true);
  assert.equal(created.id, 'session.created');
  assert.equal(created.title, 'New chat');
  assert.equal(created.version, '2');
  assert.deepEqual(created.messages, []);
});
