import {
  createClient,
  type SdkworkAppClient as GeneratedSkillsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/skills-app-sdk";
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

export type SdkworkSkillsAppClient = GeneratedSkillsAppClient;
export type SdkworkSkillsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

let skillsAppSdkClient: SdkworkSkillsAppClient | null = null;

export function resolveSkillsAppSdkBaseUrl(): string {
  // Single shared base-url key; the matching API host is chosen from the
  // current page's environment+brand. This SDK client expects a bare origin.
  return resolveBaseUrl({ envKey: "SDKWORK_API_BASE_URL" }).url;
}

export function isSkillsAppSdkConfigured(): boolean {
  // The shared base-url key drives availability: configured (non-empty) means
  // the skills app SDK surface is enabled.
  return resolveBaseUrl({ envKey: "SDKWORK_API_BASE_URL" }).reason !== "empty";
}

export function createSkillsAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkSkillsAppClientConfig {
  const baseUrl = resolveSkillsAppSdkBaseUrl();

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

export function initSkillsAppSdkClient(
  config: SdkworkSkillsAppClientConfig = createSkillsAppSdkClientConfig(),
): SdkworkSkillsAppClient {
  skillsAppSdkClient = createClient(config);
  return skillsAppSdkClient;
}

export function getSkillsAppSdkClient(): SdkworkSkillsAppClient {
  return skillsAppSdkClient ?? initSkillsAppSdkClient();
}

export function resetSkillsAppSdkClient(): void {
  skillsAppSdkClient = null;
}

export type { SkillPackageRecord, SkillRecord } from "@sdkwork/skills-app-sdk";
