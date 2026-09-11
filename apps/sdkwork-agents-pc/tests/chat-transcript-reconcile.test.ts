import assert from 'node:assert/strict';
import test from 'node:test';

import { reconcileTranscriptWithServer } from '../packages/sdkwork-agents-pc-chat/src/utils/transcriptReconcile';

test('reconcileTranscriptWithServer copies server ids onto optimistic rows', () => {
  const merged = reconcileTranscriptWithServer(
    [
      { id: 'local-user', role: 'user', text: 'Hi' },
      { id: 'local-model', role: 'model', text: 'Hello' },
    ],
    [
      { id: 'server-user', role: 'user', text: 'Hi' },
      { id: 'server-model', role: 'model', text: 'Hello' },
    ],
  );

  assert.deepEqual(merged.map((message) => message.id), ['server-user', 'server-model']);
});

test('reconcileTranscriptWithServer keeps optimistic tail when user already sent another turn', () => {
  const local = [
    { id: 'local-user-1', role: 'user' as const, text: 'One' },
    { id: 'local-model-1', role: 'model' as const, text: 'A' },
    { id: 'local-user-2', role: 'user' as const, text: 'Two' },
    { id: 'local-model-2', role: 'model' as const, text: '' },
  ];
  const server = [
    { id: 'server-user-1', role: 'user' as const, text: 'One' },
    { id: 'server-model-1', role: 'model' as const, text: 'A' },
  ];

  const merged = reconcileTranscriptWithServer(local, server);
  assert.deepEqual(
    merged.map((message) => message.id),
    ['server-user-1', 'server-model-1', 'local-user-2', 'local-model-2'],
  );
});

test('reconcileTranscriptWithServer does not merge when roles diverge at the same index', () => {
  const merged = reconcileTranscriptWithServer(
    [{ id: 'local-user', role: 'user', text: 'Hi' }],
    [{ id: 'server-model', role: 'model', text: 'Hello' }],
  );

  assert.equal(merged[0]?.id, 'local-user');
});
