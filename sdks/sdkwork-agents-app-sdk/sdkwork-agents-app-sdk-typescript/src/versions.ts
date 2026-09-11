import type { SdkworkAppClient } from '../generated/server-openapi/src/index.ts';
import { appApiPath } from '../generated/server-openapi/src/api/paths.ts';
import type { AgentVersionRecord } from '../generated/server-openapi/src/types/index.ts';
import { NetworkError, TimeoutError } from '@sdkwork/sdk-common';

export type { AgentVersionRecord };

function pathSegment(value: string, name: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw new Error(`${name} is required.`);
  }
  return encodeURIComponent(normalized);
}

/** Body for {@link createAgentVersion}. */
export interface CreateAgentVersionRequest {
  version_id: string;
  description?: string;
}

/**
 * Snapshot the current agent definition as a new immutable version through
 * `agents.versions.create`. The version number increases monotonically;
 * replaying a versionId is a conflict.
 */
export async function createAgentVersion(
  client: SdkworkAppClient,
  agentId: string,
  body: CreateAgentVersionRequest,
): Promise<AgentVersionRecord> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/versions`,
  );
  return client.http.request<AgentVersionRecord>(path, {
    method: 'POST',
    body,
    contentType: 'application/json',
    sdkworkUnwrapKind: 'item',
    retry: {
      retryCondition: (error) =>
        error instanceof NetworkError || error instanceof TimeoutError,
    },
  });
}

/** Query parameters for {@link listAgentVersions}. */
export interface ListAgentVersionsParams {
  cursor?: string;
  page_size?: number;
}

/**
 * List the agent version history through `agents.versions.list`.
 *
 * Keyset-paginated, ordered by versionNumber descending.
 */
export async function listAgentVersions(
  client: SdkworkAppClient,
  agentId: string,
  params: ListAgentVersionsParams = {},
): Promise<{ items: AgentVersionRecord[]; nextCursor?: string }> {
  const search = new URLSearchParams();
  if (params.cursor) search.set('cursor', params.cursor);
  if (params.page_size !== undefined) {
    search.set('page_size', String(params.page_size));
  }
  const suffix = search.size > 0 ? `?${search.toString()}` : '';
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/versions${suffix}`,
  );
  const page = await client.http.request<{
    items: AgentVersionRecord[];
    pageInfo?: { nextCursor?: string };
  }>(path, {
    method: 'GET',
    sdkworkUnwrapKind: 'data',
  });
  return {
    items: page.items,
    nextCursor: page.pageInfo?.nextCursor,
  };
}

/** Retrieve one immutable agent version through `agents.versions.retrieve`. */
export async function getAgentVersion(
  client: SdkworkAppClient,
  agentId: string,
  versionId: string,
): Promise<AgentVersionRecord> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/versions/${pathSegment(versionId, 'versionId')}`,
  );
  return client.http.request<AgentVersionRecord>(path, {
    method: 'GET',
    sdkworkUnwrapKind: 'item',
  });
}

/**
 * Activate one agent version through `agents.versions.activate`.
 *
 * Activation is the publish and rollback path: the version's immutable
 * manifest is written back onto the live agent definition.
 */
export async function activateAgentVersion(
  client: SdkworkAppClient,
  agentId: string,
  versionId: string,
): Promise<AgentVersionRecord> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/versions/${pathSegment(versionId, 'versionId')}/activate`,
  );
  return client.http.request<AgentVersionRecord>(path, {
    method: 'POST',
    sdkworkUnwrapKind: 'item',
    retry: {
      retryCondition: (error) =>
        error instanceof NetworkError || error instanceof TimeoutError,
    },
  });
}
