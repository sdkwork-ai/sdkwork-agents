import React from 'react';
import { AlertTriangle, ArrowUpRight, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import {
  requestAgentsTokenPlan,
  type ChatBalanceSnapshot,
} from '../services/chatBalancePort';

export interface ChatBalanceAlertProps {
  /** Balance read from the host; `null` renders the copy without an amount. */
  snapshot: ChatBalanceSnapshot | null;
  onDismiss: () => void;
}

function formatBalance(snapshot: ChatBalanceSnapshot): string {
  const amount = Number.isFinite(snapshot.available)
    ? snapshot.available.toLocaleString()
    : '0';
  return snapshot.currency ? `${amount} ${snapshot.currency}` : amount;
}

/**
 * Insufficient-balance warning rendered directly above the chat composer.
 * It is advisory only: sending stays enabled so the host backend remains the
 * single source of truth for billing decisions.
 */
export const ChatBalanceAlert: React.FC<ChatBalanceAlertProps> = ({
  snapshot,
  onDismiss,
}) => {
  const { t } = useTranslation('chat');

  return (
    <div className="w-full px-4 pb-1 pt-2" data-chat-balance-alert>
      <div
        className="mx-auto flex w-full max-w-3xl items-start gap-2 rounded-xl border border-amber-300 bg-amber-50 px-3 py-2.5 text-[13px] text-amber-900 shadow-sm dark:border-amber-500/40 dark:bg-amber-500/10 dark:text-amber-200"
        role="alert"
      >
        <AlertTriangle aria-hidden className="mt-[2px] shrink-0" size={16} />
        <p className="min-w-0 flex-1 leading-5">
          {snapshot
            ? t('balance.insufficientWithAmount', { amount: formatBalance(snapshot) })
            : t('balance.insufficient')}
        </p>
        <button
          className="inline-flex shrink-0 items-center gap-1 rounded-md px-1.5 py-0.5 font-semibold underline decoration-amber-500/60 underline-offset-2 transition-colors hover:bg-amber-100 hover:decoration-amber-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-amber-600 dark:hover:bg-amber-500/20"
          onClick={() => requestAgentsTokenPlan()}
          type="button"
        >
          {t('balance.buyCredits')}
          <ArrowUpRight aria-hidden size={13} />
        </button>
        <button
          aria-label={t('balance.dismiss')}
          className="shrink-0 rounded-md p-0.5 text-amber-700/70 transition-colors hover:bg-amber-100 hover:text-amber-900 dark:text-amber-200/70 dark:hover:bg-amber-500/20 dark:hover:text-amber-100"
          onClick={onDismiss}
          title={t('balance.dismiss')}
          type="button"
        >
          <X aria-hidden size={14} />
        </button>
      </div>
    </div>
  );
};
