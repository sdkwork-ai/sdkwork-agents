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
  /** Optional LLM wire protocol for the cloudrouter gateway invocation (chat completions, Anthropic messages, Google content, or OpenAI responses). */
  wireProtocol?: 'chat_completions' | 'anthropic_messages' | 'google_content' | 'openai_responses';
  /** Optional execution placement override for this turn. `in_process` executes on the receiving replica's agent engines; `sandbox` requires the deployment to assemble the sandbox session port and fails closed otherwise. Omitted resolves through the deployment default (SDKWORK_AGENTS_TURN_EXECUTION_ROUTE). */
  executionRoute?: 'in_process' | 'sandbox';
  idempotencyKey: string;
  payloadHash: string;
  clientRequestId?: string;
  driveRefs?: ({ resourceRole: 'attachment' | 'image' | 'audio' | 'artifact'; driveSpaceId: string; driveNodeId: string; })[];
  requestedAt: string;
}
