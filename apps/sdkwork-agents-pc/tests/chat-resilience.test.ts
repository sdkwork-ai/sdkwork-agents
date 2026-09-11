import assert from 'node:assert/strict';
import test from 'node:test';

import type { SdkworkAgentsDriveAppClient } from '@sdkwork/agents-pc-core/sdk';
import {
  ChatFileLibraryService,
} from '../packages/sdkwork-agents-pc-chat/src/services/chatFileLibraryService';
import {
  ChatService,
  callerScopeGrantsAgentManage,
  configureChatAgentPermissionScopeReader,
  configureChatAgentPort,
  type ChatAgentPort,
} from '../packages/sdkwork-agents-pc-chat/src/services/ChatService';

function forbiddenError(): Error & { httpStatus: number; code: string } {
  const error = new Error('iam.permission.missing:ai.agents.manage') as Error & {
    httpStatus: number;
    code: string;
  };
  error.httpStatus = 403;
  error.code = 'FORBIDDEN';
  return error;
}

function notFoundError(): Error & {
  httpStatus: number;
  code: string;
  problem: { status: number; code: number };
} {
  const error = new Error('Not found') as Error & {
    httpStatus: number;
    code: string;
    problem: { status: number; code: number };
  };
  error.httpStatus = 404;
  error.code = 'NOT_FOUND';
  error.problem = { status: 404, code: 40401 };
  return error;
}

function createChatAgentPort(overrides: Partial<ChatAgentPort>): ChatAgentPort {
  const session = {
    id: 'session.1',
    title: 'Session 1',
    updatedAt: new Date().toISOString(),
    version: '1',
  };
  return {
    getAgent: async () => ({
      id: 'agent.chat.default',
      name: 'SDKWork Agents',
      description: '',
      type: 'normal' as const,
      model: 'model-a',
      systemPrompt: '',
      welcomeMessage: '',
    }),
    createAgent: async (agent) => agent,
    updateAgent: async () => session,
    resolveOrCreateSession: async (_agentId, sessionId) => sessionId,
    createSession: async (_agentId, title) => ({
      id: 'session.new',
      title,
      updatedAt: new Date().toISOString(),
      version: '1',
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
    sendMessage: async (_agentId, _sessionId, content) => ({ id: 'm1', content }),
    ...overrides,
  };
}

test('does not attempt the default agent model sync without ai.agents.manage', async () => {
  let updateAttempted = false;
  configureChatAgentPermissionScopeReader(() => ['ai.agents.read', 'ai.agents.use']);
  configureChatAgentPort(createChatAgentPort({
    updateAgent: async () => {
      updateAttempted = true;
      throw forbiddenError();
    },
  }));

  const sessions = await ChatService.loadSessions('model-b');
  assert.equal(updateAttempted, false);
  assert.deepEqual(
    sessions.map((session) => session.id),
    ['session.1'],
  );
});

test('keeps syncing the default agent model when the caller is allowed', async () => {
  let updateAttempted = false;
  configureChatAgentPermissionScopeReader(() => ['ai.agents.manage']);
  configureChatAgentPort(createChatAgentPort({
    updateAgent: async () => {
      updateAttempted = true;
      return {
        id: 'agent.chat.default',
        name: 'SDKWork Agents',
        description: '',
        type: 'normal' as const,
        model: 'model-b',
        systemPrompt: '',
        welcomeMessage: '',
      };
    },
  }));

  await ChatService.loadSessions('model-b');
  assert.equal(updateAttempted, true);
});

test('syncs the default agent model for ai.* wildcard callers', async () => {
  let updateAttempted = false;
  configureChatAgentPermissionScopeReader(() => ['ai.*']);
  configureChatAgentPort(createChatAgentPort({
    updateAgent: async () => {
      updateAttempted = true;
      return {
        id: 'agent.chat.default',
        name: 'SDKWork Agents',
        description: '',
        type: 'normal' as const,
        model: 'model-b',
        systemPrompt: '',
        welcomeMessage: '',
      };
    },
  }));

  await ChatService.loadSessions('model-b');
  assert.equal(updateAttempted, true);
});

test('callerScopeGrantsAgentManage only admits manage and wildcard scopes', () => {
  assert.equal(callerScopeGrantsAgentManage([]), false);
  assert.equal(callerScopeGrantsAgentManage(['ai.agents.read', 'ai.agents.use']), false);
  assert.equal(callerScopeGrantsAgentManage(['ai.agents.manage']), true);
  assert.equal(callerScopeGrantsAgentManage(['ai.*']), true);
  assert.equal(callerScopeGrantsAgentManage(['*']), true);
});

test('treats a missing chat file library property as an empty library', async () => {
  const service = new ChatFileLibraryService(() => ({
    drive: {
      propertyNodes: {
        list: async () => {
          throw notFoundError();
        },
      },
    },
  } as unknown as SdkworkAgentsDriveAppClient));

  assert.deepEqual(await service.listFiles(), { items: [], nextCursor: null });
});

test('normalizes Drive property nodes into chat library files', async () => {
  const service = new ChatFileLibraryService(() => ({
    drive: {
      propertyNodes: {
        list: async () => ({
          items: [{
            id: 'node-1',
            nodeName: 'guide.md',
            contentType: 'text/markdown',
            contentLength: '42',
            updatedAt: '2026-08-07T00:00:00.000Z',
            spaceId: 'space-1',
          }],
          pageInfo: { nextCursor: 'page-2' },
        }),
      },
    },
  } as unknown as SdkworkAgentsDriveAppClient));

  const page = await service.listFiles();
  assert.equal(page.nextCursor, 'page-2');
  assert.deepEqual(page.items, [{
    id: 'node-1',
    name: 'guide.md',
    mimeType: 'text/markdown',
    sizeBytes: '42',
    updatedAt: '2026-08-07T00:00:00.000Z',
    spaceId: 'space-1',
  }]);
});

test('rethrows non-404 file library failures', async () => {
  const service = new ChatFileLibraryService(() => ({
    drive: {
      propertyNodes: {
        list: async () => {
          throw new Error('boom');
        },
      },
    },
  } as unknown as SdkworkAgentsDriveAppClient));

  await assert.rejects(service.listFiles(), /boom/u);
});
