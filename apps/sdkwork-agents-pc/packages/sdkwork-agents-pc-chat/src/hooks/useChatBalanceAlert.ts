import { useCallback, useEffect, useState } from 'react';
import {
  AGENTS_TOKEN_PLAN_CLOSED_EVENT,
  getChatBalancePort,
  isChatBalanceInsufficient,
  type ChatBalanceSnapshot,
} from '../services/chatBalancePort';

export interface ChatBalanceAlertState {
  /** `true` only when a signed-in account balance was read and is insufficient. */
  insufficient: boolean;
  snapshot: ChatBalanceSnapshot | null;
  /** Forces an immediate re-read of the balance. */
  refresh: () => void;
}

const NO_ALERT: ChatBalanceAlertState = {
  insufficient: false,
  snapshot: null,
  refresh: () => undefined,
};

/**
 * Reads the host-injected account balance and keeps the insufficient-balance
 * warning in sync with purchases, window focus, and host polling.
 *
 * Without a configured balance port the hook is inert, so standalone chat
 * embeds keep their previous behaviour.
 */
export function useChatBalanceAlert(): ChatBalanceAlertState {
  const [snapshot, setSnapshot] = useState<ChatBalanceSnapshot | null>(null);
  const [refreshToken, setRefreshToken] = useState(0);
  const [hasPort, setHasPort] = useState(() => getChatBalancePort() !== null);

  const refresh = useCallback(() => {
    // The port is configured by the host during module bootstrap, which can
    // land after the first render; re-checking here keeps the hook reactive.
    setHasPort(getChatBalancePort() !== null);
    setRefreshToken((token) => token + 1);
  }, []);

  useEffect(() => {
    const port = getChatBalancePort();
    if (!port) {
      return;
    }
    let active = true;
    const load = (): void => {
      void Promise.resolve(port.fetchBalance())
        .then((next) => {
          if (active) {
            setSnapshot(next);
          }
        })
        .catch(() => {
          // A failed balance lookup must never block the chat surface.
        });
    };

    load();

    const interval = port.refreshIntervalMs && port.refreshIntervalMs > 0
      ? window.setInterval(load, port.refreshIntervalMs)
      : undefined;
    const handleFocus = (): void => load();
    const handleTokenPlanClosed = (): void => load();

    window.addEventListener('focus', handleFocus);
    window.addEventListener(AGENTS_TOKEN_PLAN_CLOSED_EVENT, handleTokenPlanClosed);

    return () => {
      active = false;
      if (interval !== undefined) {
        window.clearInterval(interval);
      }
      window.removeEventListener('focus', handleFocus);
      window.removeEventListener(AGENTS_TOKEN_PLAN_CLOSED_EVENT, handleTokenPlanClosed);
    };
  }, [hasPort, refreshToken]);

  if (!hasPort) {
    return NO_ALERT;
  }

  return {
    insufficient: isChatBalanceInsufficient(snapshot),
    snapshot,
    refresh,
  };
}
