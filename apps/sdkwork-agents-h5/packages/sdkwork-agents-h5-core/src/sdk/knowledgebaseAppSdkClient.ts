import {
  createClient,
  type SdkworkKnowledgebaseAppClient as GeneratedKnowledgebaseAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/knowledgebase-app-sdk";
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

export type SdkworkKnowledgebaseAppClient = GeneratedKnowledgebaseAppClient;
export type SdkworkKnowledgebaseAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

let knowledgebaseAppSdkClient: SdkworkKnowledgebaseAppClient | null = null;

export function resolveKnowledgebaseAppSdkBaseUrl(): string {
  // Single shared base-url key; the matching API host is chosen from the
  // current page's environment+brand. This SDK client expects a bare origin.
  return resolveBaseUrl({ envKey: "SDKWORK_API_BASE_URL" }).url;
}

export function isKnowledgebaseAppSdkConfigured(): boolean {
  // The shared base-url key drives availability: configured (non-empty) means
  // the knowledgebase app SDK surface is enabled.
  return resolveBaseUrl({ envKey: "SDKWORK_API_BASE_URL" }).reason !== "empty";
}

export function createKnowledgebaseAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkKnowledgebaseAppClientConfig {
  const baseUrl = resolveKnowledgebaseAppSdkBaseUrl();

  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");

  return {
    baseUrl,
    accessToken: resolveAppSdkAccessToken(currentSession) ?? envAccessToken,
    authToken: resolveAppSdkAuthToken(currentSession),
    interceptors: createSdkworkChatRequestContextInterceptors(
      () => readAppSdkSessionTokens() ?? currentSession,
    ),
    platform: "h5",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initKnowledgebaseAppSdkClient(
  config: SdkworkKnowledgebaseAppClientConfig = createKnowledgebaseAppSdkClientConfig(),
): SdkworkKnowledgebaseAppClient {
  knowledgebaseAppSdkClient = createClient(config);
  return knowledgebaseAppSdkClient;
}

export function getKnowledgebaseAppSdkClient(): SdkworkKnowledgebaseAppClient {
  return knowledgebaseAppSdkClient ?? initKnowledgebaseAppSdkClient();
}

export function resetKnowledgebaseAppSdkClient(): void {
  knowledgebaseAppSdkClient = null;
}

export type { KnowledgeMarketCatalogItem } from "@sdkwork/knowledgebase-app-sdk";
