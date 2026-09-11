/**
 * Window event dispatched by the chat surface when the user asks for the
 * Token Plan purchase page. The embedding workbench listens for it and opens
 * the purchase surface as a full-screen overlay.
 */
export const AGENTS_OPEN_TOKEN_PLAN_EVENT = 'agents:open-token-plan';

/**
 * Window event broadcast when the Token Plan overlay is closed so balance
 * consumers can re-read the account balance (a purchase may have completed).
 */
export const AGENTS_TOKEN_PLAN_CLOSED_EVENT = 'agents:token-plan-closed';

export interface ChatBalanceSnapshot {
  /** Remaining spendable balance expressed in the host credit/currency unit. */
  available: number;
  /** Optional display unit, e.g. `CNY` or `credits`. */
  currency?: string;
  /**
   * Host-side verdict. When omitted the chat surface falls back to
   * `available <= 0`.
   */
  insufficient?: boolean;
}

export interface ChatBalancePort {
  /**
   * Resolves `null` when no signed-in account balance is available (anonymous
   * visitor, host without billing, or a failed lookup). `null` never renders a
   * warning so the chat surface stays usable in unauthenticated embeds.
   */
  fetchBalance: () => Promise<ChatBalanceSnapshot | null>;
  /** Optional polling cadence in milliseconds; polling is off when omitted. */
  refreshIntervalMs?: number;
  /** Optional host override for the purchase entry point. */
  onPurchase?: () => void;
}

let chatBalancePort: ChatBalancePort | null = null;

export function configureChatBalancePort(port: ChatBalancePort | null): void {
  chatBalancePort = port;
}

export function getChatBalancePort(): ChatBalancePort | null {
  return chatBalancePort;
}

export function isChatBalanceInsufficient(snapshot: ChatBalanceSnapshot | null): boolean {
  if (!snapshot) {
    return false;
  }
  return snapshot.insufficient ?? snapshot.available <= 0;
}

/**
 * Asks the host to open the Token Plan purchase surface. Prefers an explicit
 * host hook so non-workbench embeds can route to their own billing page, and
 * falls back to the workbench overlay event.
 */
export function requestAgentsTokenPlan(): void {
  const port = getChatBalancePort();
  if (port?.onPurchase) {
    port.onPurchase();
    return;
  }
  window.dispatchEvent(new CustomEvent(AGENTS_OPEN_TOKEN_PLAN_EVENT));
}
