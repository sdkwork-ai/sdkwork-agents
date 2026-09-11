import type { AgentInteractionQuestion } from './agent-interaction-question';
import type { AgentInteractionQuestionOption } from './agent-interaction-question-option';

export interface AgentInteractionRequestData {
  itemId?: string | null;
  command?: string | null;
  cwd?: string | null;
  reason?: string | null;
  changes?: Record<string, unknown>;
  proposedExecPolicyAmendment?: Record<string, unknown>;
  proposedNetworkPolicyAmendment?: Record<string, unknown>;
  questions?: AgentInteractionQuestion[];
  autoResolutionMs?: string | null;
  isBlocking?: boolean;
  question?: string;
  options?: AgentInteractionQuestionOption[];
  allowMultiple?: boolean;
  submitLabel?: string | null;
  skipLabel?: string | null;
  step?: 'role' | 'task' | 'context';
  mode?: 'form' | 'openai/form' | 'url';
  serverName?: string;
  message?: string;
  elicitationId?: string;
  url?: string;
  requestedSchema?: unknown;
  requestedPermissions?: Record<string, unknown>;
}
