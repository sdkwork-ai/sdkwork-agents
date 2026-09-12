import type { AgentCallOutputSpec } from './agent-call-output-spec';
import type { AgentCallPolicySpec } from './agent-call-policy-spec';

export interface CreateAgentCallRequest {
  executionId: string;
  mode: 'prompt' | 'params';
  prompt?: string;
  params?: Record<string, unknown>;
  paramSchema?: Record<string, unknown>;
  output?: AgentCallOutputSpec;
  policy?: AgentCallPolicySpec;
  /** "sync" executes inside the request and returns the terminal record (201). "async" persists a queued record and returns 202. */
  executionMode?: 'sync' | 'async';
  requestedAt: string;
}
