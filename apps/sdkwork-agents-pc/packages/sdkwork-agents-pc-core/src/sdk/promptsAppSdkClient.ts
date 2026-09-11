import {
  createClient,
  type SdkworkAppClient as SdkworkPromptsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/prompts-app-sdk";
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

export type SdkworkAgentsPromptsAppClient = SdkworkPromptsAppClient;
export type SdkworkAgentsPromptsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";
let promptsAppSdkClient: SdkworkAgentsPromptsAppClient | null = null;
let promptsAppSdkClientProvider: (() => SdkworkAgentsPromptsAppClient) | null = null;

export function configurePromptsAppSdkClientProvider(
  provider: () => SdkworkAgentsPromptsAppClient,
): void {
  promptsAppSdkClientProvider = provider;
}

function transportBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  return normalized.endsWith(APP_API_SUFFIX)
    ? normalized.slice(0, -APP_API_SUFFIX.length)
    : normalized;
}

export function resolvePromptsAppSdkBaseUrl(): string {
  return readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_PROMPTS_APP_API_BASE_URL")
    ?? resolveAgentsAppSdkBaseUrl();
}

export function createPromptsAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkAgentsPromptsAppClientConfig {
  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");
  return {
    baseUrl: transportBaseUrl(resolvePromptsAppSdkBaseUrl()),
    accessToken: resolveAppSdkAccessToken(currentSession) ?? envAccessToken,
    authToken: resolveAppSdkAuthToken(currentSession),
    interceptors: createSdkworkChatRequestContextInterceptors(
      () => readAppSdkSessionTokens() ?? currentSession,
    ),
    platform: "pc",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initPromptsAppSdkClient(
  config: SdkworkAgentsPromptsAppClientConfig = createPromptsAppSdkClientConfig(),
): SdkworkAgentsPromptsAppClient {
  promptsAppSdkClient = createClient(config);
  return promptsAppSdkClient;
}

export function getPromptsAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkAgentsPromptsAppClient {
  if (promptsAppSdkClientProvider) {
    return promptsAppSdkClientProvider();
  }
  return initPromptsAppSdkClient(createPromptsAppSdkClientConfig(session));
}

export function resetPromptsAppSdkClient(): void {
  promptsAppSdkClient = null;
  promptsAppSdkClientProvider = null;
}

export type {
  PromptTemplate,
  PromptTemplatePage,
  PromptTemplateVersion,
  PromptTemplateVersionPage,
} from "@sdkwork/prompts-app-sdk";
