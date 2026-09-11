import {
  SdkworkAppClient as GeneratedSdkworkAppClient,
} from '../generated/server-openapi/src/index.ts';
import { APP_API_PREFIX } from '../generated/server-openapi/src/api/paths.ts';
import type { SdkworkAppConfig } from '../generated/server-openapi/src/types/common.ts';

export type { SdkworkAppConfig };
export * from '../generated/server-openapi/src/types/index.ts';
export * from '../generated/server-openapi/src/api/index.ts';
export * from '../generated/server-openapi/src/http/index.ts';
export * from '../generated/server-openapi/src/auth/index.ts';

function resolveTransportBaseUrl(appApiBaseUrl: string): string {
  const normalized = appApiBaseUrl.trim().replace(/\/+$/u, '');
  if (!normalized) {
    throw new Error('Agents App SDK base URL must not be empty.');
  }

  const prefixIndex = normalized.indexOf(APP_API_PREFIX);
  const hasCanonicalSurfaceSuffix = normalized.endsWith(APP_API_PREFIX);
  if (prefixIndex >= 0 && !hasCanonicalSurfaceSuffix) {
    throw new Error(
      `Agents App SDK base URL must identify a gateway root or end with ${APP_API_PREFIX}.`,
    );
  }

  const transportBaseUrl = hasCanonicalSurfaceSuffix
    ? normalized.slice(0, -APP_API_PREFIX.length)
    : normalized;
  if (transportBaseUrl.includes(APP_API_PREFIX)) {
    throw new Error(`Agents App API base URL must include ${APP_API_PREFIX} exactly once.`);
  }
  return transportBaseUrl;
}

export class SdkworkAppClient extends GeneratedSdkworkAppClient {
  constructor(config: SdkworkAppConfig) {
    super({
      ...config,
      baseUrl: resolveTransportBaseUrl(config.baseUrl),
    });
  }
}

export function createClient(config: SdkworkAppConfig): SdkworkAppClient {
  return new SdkworkAppClient(config);
}

export { completeAgentTurn, completeAgentTurnStream } from './turns.ts';
export type { CompleteAgentTurnResult, CreateAgentTurnRequest, TurnStreamEvent } from './turns.ts';
