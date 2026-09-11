import assert from 'node:assert/strict';
import test from 'node:test';

import type { AgentSessionItemRecord } from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';
import { sortSessionItems } from '../packages/sdkwork-agents-pc-agents/src/services/sessionMessageOrdering';

function item(
  kind: AgentSessionItemRecord['kind'],
  itemId: string,
  createdAt = '2026-08-28T04:00:00.000Z',
  sequence = '0',
): AgentSessionItemRecord {
  return {
    tenantId: '1',
    organizationId: '1',
    sessionId: 'session-1',
    itemId,
    kind,
    status: 'completed',
    sequence,
    inputTokens: '0',
    outputTokens: '0',
    driveRefs: [],
    createdBy: '1',
    version: '1',
    createdAt,
    updatedAt: createdAt,
  };
}

test('sortSessionItems keeps user_input before assistant_output for same timestamp', () => {
  const sorted = sortSessionItems([
    item('assistant_output', 'assistant-1'),
    item('user_input', 'user-1'),
  ]);
  assert.equal(sorted[0]?.kind, 'user_input');
  assert.equal(sorted[1]?.kind, 'assistant_output');
});

test('sortSessionItems orders by sequence when timestamps match', () => {
  const sorted = sortSessionItems([
    item('assistant_output', 'assistant-2', '2026-08-28T04:00:00.000Z', '20'),
    item('user_input', 'user-2', '2026-08-28T04:00:00.000Z', '10'),
  ]);
  assert.equal(sorted[0]?.itemId, 'user-2');
  assert.equal(sorted[1]?.itemId, 'assistant-2');
});
