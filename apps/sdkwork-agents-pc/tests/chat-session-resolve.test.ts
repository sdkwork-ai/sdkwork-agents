import assert from 'node:assert/strict';
import test from 'node:test';

import {
  ChatService,
  configureChatAgentPort,
  type ChatAgentPort,
} from '../packages/sdkwork-agents-pc-chat/src/services/ChatService';
import { trimSessionTitle } from '../packages/sdkwork-agents-pc-chat/src/utils/sessionTitleUtils';

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
    resolveOrCreateSession: async () => {
      throw new Error('resolveOrCreateSession should not be called for persisted ids');
    },
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
    sendMessageStream: async (_agentId, _sessionId, _content, _model, _media, onDelta) => {
      onDelta('Hello');
      return { id: 'm1', content: 'Hello' };
    },
    ...overrides,
  };
}

test('trimSessionTitle keeps short titles unchanged', () => {
  assert.equal(trimSessionTitle('Hello world', 30), 'Hello world');
});

test('trimSessionTitle truncates long titles with an ellipsis', () => {
  const longTitle = 'a'.repeat(40);
  assert.equal(trimSessionTitle(longTitle, 30), `${'a'.repeat(27)}...`);
});

test('trimSessionTitle strips control characters and collapses whitespace', () => {
  assert.equal(trimSessionTitle('  hello\r\nworld  '), 'hello world');
});

test('streamChat rejects empty session ids', async () => {
  let sendCalled = false;
  configureChatAgentPort(createChatAgentPort({
    resolveOrCreateSession: async (_agentId, sessionId) => sessionId,
    sendMessageStream: async () => {
      sendCalled = true;
      return { id: 'm1', content: 'ok' };
    },
  }));

  let failureMessage = '';
  await ChatService.streamChat({
    sessionId: '   ',
    model: 'model-a',
    messages: [{ id: 'u1', role: 'user', text: 'Hi' }],
    onMessageUpdate: () => undefined,
    onError: (failure) => {
      failureMessage = failure.message;
    },
  });

  assert.equal(sendCalled, false);
  assert.equal(failureMessage, 'A chat session is required.');
});

test('streamChat reuses persisted server session ids without resolveOrCreate', async () => {
  let resolveCalled = false;
  configureChatAgentPort(createChatAgentPort({
    resolveOrCreateSession: async () => {
      resolveCalled = true;
      return 'session.dup';
    },
  }));

  await ChatService.createSession('model-a', 'New chat');
  await ChatService.streamChat({
    sessionId: 'session.created',
    model: 'model-a',
    messages: [{ id: 'u1', role: 'user', text: 'Hi' }],
    onMessageUpdate: () => undefined,
  });

  assert.equal(resolveCalled, false);
});

test('loadSessions warms the resolve cache for listed sessions', async () => {
  let resolveCalled = false;
  configureChatAgentPort(createChatAgentPort({
    resolveOrCreateSession: async () => {
      resolveCalled = true;
      return 'session.dup';
    },
  }));

  await ChatService.loadSessions('model-a');
  await ChatService.streamChat({
    sessionId: 'session.1',
    model: 'model-a',
    messages: [{ id: 'u1', role: 'user', text: 'Hi' }],
    onMessageUpdate: () => undefined,
  });

  assert.equal(resolveCalled, false);
});
