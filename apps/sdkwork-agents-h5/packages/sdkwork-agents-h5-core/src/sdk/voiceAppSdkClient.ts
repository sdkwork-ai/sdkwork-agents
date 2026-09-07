import {
  createClient,
  type SdkworkAppClient as GeneratedVoiceAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/voice-app-sdk";
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

export type SdkworkVoiceAppClient = GeneratedVoiceAppClient;
export type SdkworkVoiceAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

let voiceAppSdkClient: SdkworkVoiceAppClient | null = null;

export function resolveVoiceAppSdkBaseUrl(): string {
  // Single shared base-url key; the matching API host is chosen from the
  // current page's environment+brand. This SDK client expects a bare origin.
  return resolveBaseUrl({ envKey: "SDKWORK_API_BASE_URL" }).url;
}

export function isVoiceAppSdkConfigured(): boolean {
  // The shared base-url key drives availability: configured (non-empty) means
  // the voice app SDK surface is enabled.
  return resolveBaseUrl({ envKey: "SDKWORK_API_BASE_URL" }).reason !== "empty";
}

export function createVoiceAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkVoiceAppClientConfig {
  const baseUrl = resolveVoiceAppSdkBaseUrl();

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
