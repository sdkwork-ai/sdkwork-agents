import {
  createClient,
  type SdkworkAppClient as GeneratedModelsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/models-app-sdk";
import type { Interceptors } from "@sdkwork/sdk-common";

import {
  createSdkworkChatRequestContextInterceptors,
  getSdkworkChatGlobalTokenManager,
  readAppSdkSessionTokens,
  resolveAppSdkAccessToken,
  resolveAppSdkAuthToken,
  type SdkworkChatSession,
} from "../session/session";
import { resolveAgentsAppSdkBaseUrl } from "./agentsAppSdkClient";
import { readRuntimeEnv } from "./runtimeEnv";

export type SdkworkModelsAppClient = GeneratedModelsAppClient;
export type SdkworkModelsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";

let modelsAppSdkClient: SdkworkModelsAppClient | null = null;
let modelsAppSdkClientProvider: (() => SdkworkModelsAppClient) | null = null;

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function resolveModelsAppSdkBaseUrl(): string | null {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_MODELS_APP_API_BASE_URL")
    ?? readRuntimeEnv("VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL");
  if (fromEnv) return fromEnv;
  // Gateway-routed deployments (cloud profiles and local dev ingress) serve
  // every app API under the same origin as the Agents API. Reuse the Agents
  // base URL fallback chain (public HTTP URL -> window origin) so the models
  // catalog SDK works without its own explicit VITE_ override.
  return resolveAgentsAppSdkBaseUrl();
}

export function isModelsAppSdkConfigured(): boolean {
  return resolveModelsAppSdkBaseUrl() !== null;
}

export function createModelsAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkModelsAppClientConfig {
  const baseUrl = resolveModelsAppSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("models app SDK base URL is not configured");
  }

  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");
  return {
    baseUrl: normalizeGeneratedSdkBaseUrl(baseUrl),
    accessToken: resolveAppSdkAccessToken(currentSession) ?? envAccessToken,
    authToken: resolveAppSdkAuthToken(currentSession),
    interceptors: createSdkworkChatRequestContextInterceptors(
      () => readAppSdkSessionTokens() ?? currentSession,
    ),
    platform: "pc",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initModelsAppSdkClient(
  config: SdkworkModelsAppClientConfig = createModelsAppSdkClientConfig(),
): SdkworkModelsAppClient {
  modelsAppSdkClient = createClient(config);
  return modelsAppSdkClient;
}

export function getModelsAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkModelsAppClient {
  if (modelsAppSdkClientProvider) {
    return modelsAppSdkClientProvider();
  }
  return initModelsAppSdkClient(createModelsAppSdkClientConfig(session));
}

export function getModelsAppSdkClient(): SdkworkModelsAppClient {
  if (modelsAppSdkClientProvider) {
    return modelsAppSdkClientProvider();
  }
  return modelsAppSdkClient ?? initModelsAppSdkClient();
}

export function resetModelsAppSdkClient(): void {
  modelsAppSdkClient = null;
  modelsAppSdkClientProvider = null;
}

export type {
  AiModelsListParams,
  AppModelCatalogGroup,
  AppModelCatalogItem,
  AppModelCatalogPage,
} from "@sdkwork/models-app-sdk";
