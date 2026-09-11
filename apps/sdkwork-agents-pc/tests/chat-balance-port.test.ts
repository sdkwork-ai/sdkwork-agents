import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  AGENTS_OPEN_TOKEN_PLAN_EVENT,
  AGENTS_TOKEN_PLAN_CLOSED_EVENT,
  configureChatBalancePort,
  getChatBalancePort,
  isChatBalanceInsufficient,
  requestAgentsTokenPlan,
} from '../packages/sdkwork-agents-pc-chat/src/services/chatBalancePort';

test('isChatBalanceInsufficient falls back to available <= 0', () => {
  assert.equal(isChatBalanceInsufficient(null), false);
  assert.equal(isChatBalanceInsufficient({ available: 5 }), false);
  assert.equal(isChatBalanceInsufficient({ available: 0 }), true);
  assert.equal(isChatBalanceInsufficient({ available: -1 }), true);
});

test('isChatBalanceInsufficient honours the host verdict', () => {
  assert.equal(isChatBalanceInsufficient({ available: 0, insufficient: false }), false);
  assert.equal(isChatBalanceInsufficient({ available: 10, insufficient: true }), true);
});

test('balance port event names are stable', () => {
  assert.equal(AGENTS_OPEN_TOKEN_PLAN_EVENT, 'agents:open-token-plan');
  assert.equal(AGENTS_TOKEN_PLAN_CLOSED_EVENT, 'agents:token-plan-closed');
});

test('requestAgentsTokenPlan prefers the host purchase hook over the window event', () => {
  const originalWindow = (globalThis as { window?: unknown }).window;
  let hookCalls = 0;
  const dispatched: string[] = [];
  (globalThis as Record<string, unknown>).window = {
    dispatchEvent: (event: { type: string }) => {
      dispatched.push(event.type);
    },
  };

  configureChatBalancePort({
    fetchBalance: async () => ({ available: 0 }),
    onPurchase: () => {
      hookCalls += 1;
    },
  });
  requestAgentsTokenPlan();
  assert.equal(hookCalls, 1);
  assert.deepEqual(dispatched, []);
  assert.equal(getChatBalancePort()?.onPurchase != null, true);

  configureChatBalancePort({
    fetchBalance: async () => ({ available: 0 }),
  });
  requestAgentsTokenPlan();
  assert.equal(hookCalls, 1);
  assert.deepEqual(dispatched, [AGENTS_OPEN_TOKEN_PLAN_EVENT]);

  if (originalWindow === undefined) {
    delete (globalThis as Record<string, unknown>).window;
  } else {
    (globalThis as Record<string, unknown>).window = originalWindow;
  }
  configureChatBalancePort(null);
  assert.equal(getChatBalancePort(), null);
});
