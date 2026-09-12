import type { AgentCallCorrelation } from './agent-call-correlation';
import type { AgentCallUsage } from './agent-call-usage';
import type { AgentCallValidation } from './agent-call-validation';

export interface AgentCallRecord {
  executionId: string;
  agentId: string;
  tenantId: string;
  status: 'queued' | 'succeeded' | 'validation_failed' | 'agent_failed' | 'timeout' | 'failed';
  output: unknown;
  rawText?: string;
  agentError?: string;
  validation: AgentCallValidation;
  usage: AgentCallUsage;
  correlation: AgentCallCorrelation;
}
