import assert from 'node:assert/strict';
import test from 'node:test';

import { groupMessagesIntoTurns } from '../packages/sdkwork-agents-pc-chat/src/utils/chatTurnGrouping';

test('groupMessagesIntoTurns pairs user messages with the following assistant reply', () => {
  const turns = groupMessagesIntoTurns([
    { id: 'u1', role: 'user', text: 'Hello' },
    { id: 'a1', role: 'model', text: 'Hi there' },
    { id: 'u2', role: 'user', text: 'Next question' },
    { id: 'a2', role: 'model', text: 'Sure' },
  ]);

  assert.equal(turns.length, 2);
  assert.equal(turns[0].user?.id, 'u1');
  assert.equal(turns[0].assistant?.id, 'a1');
  assert.equal(turns[1].user?.id, 'u2');
  assert.equal(turns[1].assistant?.id, 'a2');
});

test('groupMessagesIntoTurns keeps orphan assistant rows when no user precedes them', () => {
  const turns = groupMessagesIntoTurns([
    { id: 'a1', role: 'model', text: 'Welcome back' },
  ]);

  assert.equal(turns.length, 1);
  assert.equal(turns[0].user, undefined);
  assert.equal(turns[0].assistant?.id, 'a1');
});
