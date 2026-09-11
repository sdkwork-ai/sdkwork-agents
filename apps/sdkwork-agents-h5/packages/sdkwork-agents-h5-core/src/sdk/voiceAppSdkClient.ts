import {
  createClient,
  type SdkworkAppClient as GeneratedVoiceAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/voice-app-sdk";
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

export type SdkworkVoiceAppClient = GeneratedVoiceAppClient;
export type SdkworkVoiceAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";

let voiceAppSdkClient: SdkworkVoiceAppClient | null = null;

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function resolveVoiceAppSdkBaseUrl(): string | null {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_H5_VOICE_APP_API_BASE_URL");
  if (fromEnv) return fromEnv;
  return null;
}

export function isVoiceAppSdkConfigured(): boolean {
  return resolveVoiceAppSdkBaseUrl() !== null;
}

export function createVoiceAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkVoiceAppClientConfig {
  const baseUrl = resolveVoiceAppSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("voice app SDK base URL is not configured");
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
    platform: "h5",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initVoiceAppSdkClient(
  config: SdkworkVoiceAppClientConfig = createVoiceAppSdkClientConfig(),
): SdkworkVoiceAppClient {
  voiceAppSdkClient = createClient(config);
  return voiceAppSdkClient;
}

export function getVoiceAppSdkClient(): SdkworkVoiceAppClient {
  return voiceAppSdkClient ?? initVoiceAppSdkClient();
}

export function resetVoiceAppSdkClient(): void {
  voiceAppSdkClient = null;
}
