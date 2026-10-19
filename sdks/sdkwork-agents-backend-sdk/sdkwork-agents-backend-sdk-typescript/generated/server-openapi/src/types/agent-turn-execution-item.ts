import type { AgentSessionItemRecord } from './agent-session-item-record';
import type { AgentSessionRecord } from './agent-session-record';
import type { AgentTurnCitation } from './agent-turn-citation';
import type { AgentTurnRecord } from './agent-turn-record';

export interface AgentTurnExecutionItem {
  session: AgentSessionRecord;
  turn: AgentTurnRecord;
  items: AgentSessionItemRecord[];
  /** Knowledge sources that grounded this turn, numbered like the citation markers in the prompt. Empty for ungrounded turns. */
  citations?: AgentTurnCitation[];
}
