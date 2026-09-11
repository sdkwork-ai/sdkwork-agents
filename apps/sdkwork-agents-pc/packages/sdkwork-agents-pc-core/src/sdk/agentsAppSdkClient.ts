import {
  createClient,
  completeAgentTurn,
  completeAgentTurnStream,
  TURN_EVENT_PROTOCOL_KERNEL_V1,
  type CompleteAgentTurnResult,
  type SdkworkAppClient as GeneratedSdkworkAgentsAppClient,
  type SdkworkAppConfig,
  type TurnRichToolEvent,
  type TurnStreamHandlers,
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
let agentsAppSdkClientProvider: (() => SdkworkAgentsAppClient) | null = null;

export function configureAgentsAppSdkClientProvider(
  provider: () => SdkworkAgentsAppClient,
): void {
  agentsAppSdkClientProvider = provider;
  agentsAppSdkClient = null;
}

function resolveDefaultPublicHttpUrl(): string {
  return typeof window === "undefined" ? "http://127.0.0.1:8095" : window.location.origin;
}

export function resolveAgentsAppSdkBaseUrl(): string {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL");
  if (fromEnv) return fromEnv;
  const publicUrl =
    readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_APPLICATION_PUBLIC_HTTP_URL")
    ?? resolveDefaultPublicHttpUrl();
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
    platform: "pc",
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
  if (agentsAppSdkClientProvider) {
    return agentsAppSdkClientProvider();
  }
  return agentsAppSdkClient ?? initAgentsAppSdkClient();
}

export function getAgentsAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkAgentsAppClient {
  if (agentsAppSdkClientProvider) {
    return agentsAppSdkClientProvider();
  }
  return initAgentsAppSdkClient(createAgentsAppSdkClientConfig(session));
}

export function resetAgentsAppSdkClient(): void {
  agentsAppSdkClient = null;
  agentsAppSdkClientProvider = null;
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
  CreateAgentTurnRequest,
  McpServerMarketplaceRecord,
  PageInfo,
  UpdateAgentRequest,
  UpdateAgentSessionRuntimeBindingRequest,
} from "@sdkwork/agents-app-sdk";

export { completeAgentTurn, completeAgentTurnStream, TURN_EVENT_PROTOCOL_KERNEL_V1 };
export type { CompleteAgentTurnResult, TurnRichToolEvent, TurnStreamHandlers };
