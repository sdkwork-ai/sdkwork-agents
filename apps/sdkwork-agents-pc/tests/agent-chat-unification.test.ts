import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  ChatService,
  configureChatAgentPort,
  createChatAgentScope,
  type ChatAgentPort,
} from '../packages/sdkwork-agents-pc-chat/src/services/ChatService';
import { resetChatBootstrapForTests } from '../packages/sdkwork-agents-pc-chat/src/services/chatBootstrap';

function createChatAgentPort(overrides: Partial<ChatAgentPort>): ChatAgentPort {
  return {
    getAgent: async () => ({ model: 'openai/gpt-4o-mini', systemPrompt: 'You are a test agent.' }),
    createAgent: async () => ({ model: 'openai/gpt-4o-mini' }),
    updateAgent: async () => ({ model: 'openai/gpt-4o-mini' }),
    resolveOrCreateSession: async (_agentId, _sessionId, _title) => 'session.scoped',
    createSession: async (agentId, title) => ({
      id: `session.${agentId}`,
      title,
      updatedAt: '2026-08-28T00:00:00.000Z',
      version: '1',
    }),
    listSessions: async (agentId) => [{
      id: `session.${agentId}`,
      title: 'Scoped session',
      updatedAt: '2026-08-28T00:00:00.000Z',
      version: '1',
    }],
    updateSession: async (_agentId, sessionId) => ({
      id: sessionId,
      title: 'Scoped session',
      updatedAt: '2026-08-28T00:00:00.000Z',
      version: '1',
    }),
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
      version: '1',
    }),
    listMessages: async () => [],
    resolveMediaPreviewUrl: async (driveUri) => driveUri,
    sendMessageStream: async (agentId, _sessionId, _content, _model, _media, onDelta) => {
      onDelta(`reply:${agentId}`);
      return { id: 'assistant-1', content: `reply:${agentId}` };
    },
    ...overrides,
  };
}

test('agent chat reuses the shared ChatService scope instead of a bespoke turn client', () => {
  const agentChatViewSource = readFileSync(
    new URL('../packages/sdkwork-agents-pc-agents/src/pages/AgentChatView.tsx', import.meta.url),
    'utf8',
  );
  const chatViewSource = readFileSync(
    new URL('../packages/sdkwork-agents-pc-chat/src/ChatView.tsx', import.meta.url),
    'utf8',
  );
  const homePageSource = readFileSync(
    new URL('../packages/sdkwork-agents-pc-agents/src/pages/AgentsHomePage.tsx', import.meta.url),
    'utf8',
  );

  assert.match(agentChatViewSource, /@sdkwork\/agents-pc-chat\/ChatView/);
  assert.doesNotMatch(agentChatViewSource, /agentChatService/);
  assert.doesNotMatch(agentChatViewSource, /LazyMessageInput/);
  assert.match(homePageSource, /<AgentChatView/);
  assert.doesNotMatch(homePageSource, /HomeAgentConversation/);
  assert.doesNotMatch(chatViewSource, /showModelPicker=\{!isAgentScopedChat\}/);
  assert.match(chatViewSource, /resolveAgentChatSelectedModelId/);
  assert.match(chatViewSource, /persistAgentChatSelectedModelId/);
});

test('scoped chat sessions and turns route through the configured agent id', async () => {
  resetChatBootstrapForTests();
  const scopedAgentId = 'agent.product.research';
  const scope = createChatAgentScope(scopedAgentId, {
    title: 'Product Research',
    systemPrompt: 'You are a product research assistant.',
  });
  let streamedAgentId = '';
  configureChatAgentPort(createChatAgentPort({
    sendMessageStream: async (agentId, _sessionId, _content, _model, _media, onDelta) => {
      streamedAgentId = agentId;
      onDelta('hello');
      return { id: 'assistant-1', content: 'hello' };
    },
  }));

  const sessions = await ChatService.loadSessions('openai/gpt-4o-mini', scope);
  assert.equal(sessions[0]?.id, `session.${scopedAgentId}`);

  let completed = false;
  await ChatService.streamChat({
    sessionId: sessions[0].id,
    model: 'openai/gpt-4o-mini',
    scope,
    messages: [{ id: 'user-1', role: 'user', text: 'hi' }],
    onMessageUpdate: () => undefined,
    onComplete: () => {
      completed = true;
    },
  });

  assert.equal(streamedAgentId, scopedAgentId);
  assert.equal(completed, true);
});
