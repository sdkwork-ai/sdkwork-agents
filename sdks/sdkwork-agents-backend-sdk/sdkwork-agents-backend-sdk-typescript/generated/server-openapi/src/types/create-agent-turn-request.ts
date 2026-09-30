import type { AgentTurnMode } from './agent-turn-mode';

export interface CreateAgentTurnRequest {
  turnId?: string;
  content: string;
  contentType?: string;
  turnMode: AgentTurnMode;
  runtimeBindingId?: string;
  requestedModelId?: string;
  /** Optional execution placement override for this turn. `in_process` executes on the receiving replica's agent engines; `sandbox` requires the deployment to assemble the sandbox session port and fails closed otherwise. Omitted resolves through the deployment default (SDKWORK_AGENTS_TURN_EXECUTION_ROUTE). */
  executionRoute?: 'in_process' | 'sandbox';
  idempotencyKey: string;
  payloadHash: string;
  clientRequestId?: string;
  driveRefs?: ({ resourceRole: 'attachment' | 'image' | 'audio' | 'artifact'; driveSpaceId: string; driveNodeId: string; })[];
  requestedAt: string;
}
