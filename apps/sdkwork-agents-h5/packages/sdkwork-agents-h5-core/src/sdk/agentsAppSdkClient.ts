import {
  createClient,
  completeAgentTurn,
  type CompleteAgentTurnResult,
  type SdkworkAppClient as GeneratedSdkworkAgentsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/agents-app-sdk";
import type { Interceptors } from "@sdkwork/sdk-common";

import {
  createSdkworkChatRequestContextInterceptors,
  getSdkworkChatGlobalTokenManager,
  readAppSdkSessionTokens,
  resolveAppSdkAccessToken,
  resolveAppSdkAuthToken,
  type SdkworkChatSession,
} from "../session/session";
import { readRuntimeEnv } from "./runtimeEnv";

export type SdkworkAgentsAppClient = GeneratedSdkworkAgentsAppClient;
export type SdkworkAgentsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

let agentsAppSdkClient: SdkworkAgentsAppClient | null = null;

export function resolveAgentsAppSdkBaseUrl(): string {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_H5_APP_API_BASE_URL");
  if (fromEnv) return fromEnv;
  const publicUrl =
    readRuntimeEnv("VITE_SDKWORK_AGENTS_H5_APPLICATION_PUBLIC_HTTP_URL") ?? "http://127.0.0.1:8095";
  return `${String(publicUrl).replace(/\/+$/u, "")}/app/v3/api`;
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
