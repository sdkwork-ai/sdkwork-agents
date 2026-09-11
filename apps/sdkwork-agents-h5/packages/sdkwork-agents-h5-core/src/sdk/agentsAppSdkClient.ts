import {
  createClient,
  completeAgentTurn,
  type CompleteAgentTurnResult,
  type SdkworkAppClient as GeneratedSdkworkAgentsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/agents-app-sdk";
import { resolveBaseUrl, readRuntimeEnv } from "@sdkwork/sdk-common";
import type { Interceptors } from "@sdkwork/sdk-common";

import {
  createSdkworkChatRequestContextInterceptors,
  getSdkworkChatGlobalTokenManager,
  readAppSdkSessionTokens,
  resolveAppSdkAccessToken,
  resolveAppSdkAuthToken,
  type SdkworkChatSession,
} from "../session/session";

export type SdkworkAgentsAppClient = GeneratedSdkworkAgentsAppClient;
export type SdkworkAgentsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

let agentsAppSdkClient: SdkworkAgentsAppClient | null = null;

export function resolveAgentsAppSdkBaseUrl(): string {
  // Single shared base-url key; candidates may be comma/semicolon separated and
  // the matching API host is chosen from the current page's environment+brand
  // (https page -> https://api-*, http page -> http://api-*). preservePath keeps
  // the /app/v3/api suffix this SDK client expects.
  return resolveBaseUrl({
    envKey: "SDKWORK_API_BASE_URL",
    preservePath: true,
  }).url;
}

export function createAgentsAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkAgentsAppClientConfig {
  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");

  return {
    baseUrl: resolveAgentsAppSdkBaseUrl(),
    accessToken: resolveAppSdkAccessToken(currentSession) ?? envAccessToken,
    authToken: resolveAppSdkAuthToken(currentSession),
    interceptors: createSdkworkChatRequestContextInterceptors(
      () => readAppSdkSessionTokens() ?? currentSession,
    ),
    platform: "h5",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initAgentsAppSdkClient(
  config: SdkworkAgentsAppClientConfig = createAgentsAppSdkClientConfig(),
): SdkworkAgentsAppClient {
  agentsAppSdkClient = createClient(config);
  return agentsAppSdkClient;
}

export function getAgentsAppSdkClient(): SdkworkAgentsAppClient {
  return agentsAppSdkClient ?? initAgentsAppSdkClient();
}

export function getAgentsAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkAgentsAppClient {
  return initAgentsAppSdkClient(createAgentsAppSdkClientConfig(session));
}

export function resetAgentsAppSdkClient(): void {
  agentsAppSdkClient = null;
}

export function useAgentsAppSdkClient(): SdkworkAgentsAppClient {
  return getAgentsAppSdkClientWithSession();
}

export type {
  AgentCompositionSlotRecord,
  AgentImplementationKind,
  AgentItemFeedbackRecord,
  AgentManagementProfile,
  AgentProjectCompositionSlotRecord,
  AgentProjectRecord,
  AgentProviderBindingRecord,
  AgentRecord,
  AgentResourceUserStateRecord,
  AgentRuntimeExecutionRecord,
  AgentSessionItemRecord,
  AgentSessionRecord,
  AgentSessionRuntimeBindingRecord,
  AgentSessionRuntimeBindingStatus,
  AgentEngineCatalog,
  AgentEngineCatalogEngine,
  AgentEngineModelCatalogEntry,
  CreateAgentProviderBindingRequest,
  CreateAgentRequest,
  CreateAgentSessionRuntimeBindingRequest,
  McpServerMarketplaceRecord,
  PageInfo,
  UpdateAgentRequest,
  UpdateAgentSessionRuntimeBindingRequest,
} from "@sdkwork/agents-app-sdk";

export { completeAgentTurn };
export type { CompleteAgentTurnResult };
