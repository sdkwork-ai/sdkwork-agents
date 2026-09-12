export interface AgentUsageRecord {
  turnId: string;
  sessionId: string;
  agentId: string;
  ownerUserId: string;
  status: 'requested' | 'running' | 'completed' | 'failed' | 'cancelled';
  modelId?: string;
  providerId?: string;
  inputTokens: string;
  outputTokens: string;
  cachedTokens: string;
  createdAt: string;
  completedAt?: string;
}
