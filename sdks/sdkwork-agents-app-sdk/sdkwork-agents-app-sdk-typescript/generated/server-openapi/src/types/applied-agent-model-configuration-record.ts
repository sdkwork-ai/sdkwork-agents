import type { AgentModelProviderId } from './agent-model-provider-id';

export interface AppliedAgentModelConfigurationRecord {
  configurationId: string;
  profileId: string;
  engineId: AgentModelProviderId;
  agentId: string;
  providerScope: string;
  vendorCode: string;
  baseUrl: string;
  defaultModelId: string;
  supportedModelIds: string[];
  supportedProviderIds: AgentModelProviderId[];
  inputContextTokens?: string;
  outputContextTokens?: string;
  toolCallRounds?: string;
  supportsMultimodal: boolean;
  apiKeyConfigured: boolean;
}
