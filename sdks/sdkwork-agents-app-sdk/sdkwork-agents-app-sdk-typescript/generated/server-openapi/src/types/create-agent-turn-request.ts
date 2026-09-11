import type { AgentTurnMode } from './agent-turn-mode';

export interface CreateAgentTurnRequest {
  turnId?: string;
  content: string;
  contentType?: string;
  turnMode: AgentTurnMode;
  /** Agent system prompt injected ahead of the turn history. */
  systemPrompt?: string;
  runtimeBindingId?: string;
  requestedModelId?: string;
  accessModeId?: string;
  idempotencyKey: string;
  payloadHash: string;
  clientRequestId?: string;
  driveRefs?: ({ resourceRole: 'attachment' | 'image' | 'audio' | 'artifact'; driveSpaceId: string; driveNodeId: string; })[];
  requestedAt: string;
}
