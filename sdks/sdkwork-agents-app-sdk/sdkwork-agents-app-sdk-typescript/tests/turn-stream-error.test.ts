import assert from 'node:assert/strict';
import test from 'node:test';

import { turnStreamErrorFromEvent } from '../src/index.ts';

test('turnStreamErrorFromEvent carries the funding problem for chat classification', () => {
  const error = turnStreamErrorFromEvent({
    eventType: 'error',
    problem: {
      type: 'https://docs.sdkwork.com/problems/40201',
      title: 'Payment Required',
      status: 402,
      detail: 'insufficient account balance',
      code: 40201,
      i18nKey: 'errors.result.40201',
      action: { kind: 'recharge' },
    },
  });
  assert.equal(error.problem?.code, 40201);
  assert.equal(error.problem?.action?.kind, 'recharge');
  assert.equal(error.message, 'insufficient account balance');
});

test('turnStreamErrorFromEvent falls back to the title and tolerates a missing body', () => {
  const titled = turnStreamErrorFromEvent({
    eventType: 'error',
    problem: { title: 'Service Unavailable', status: 503 },
  });
  assert.equal(titled.message, 'Service Unavailable');
  assert.equal(titled.problem?.code, undefined);

  const bare = turnStreamErrorFromEvent({ eventType: 'error' });
  assert.equal(bare.message, 'Agent turn failed.');
  assert.equal(bare.problem, undefined);
});
