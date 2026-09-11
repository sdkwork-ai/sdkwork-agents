import type { SdkworkAppClient } from '../generated/server-openapi/src/index.ts';
import { appApiPath } from '../generated/server-openapi/src/api/paths.ts';
import type { AgentCallRecord } from '../generated/server-openapi/src/types/index.ts';
import type { CreateAgentCallRequest } from '../generated/server-openapi/src/types/index.ts';
import { NetworkError, TimeoutError } from '@sdkwork/sdk-common';

export type { CreateAgentCallRequest };

/** Structured-call result carried inside `SdkWorkResourceData.item`. */
export type AgentCallResult = AgentCallRecord;
export type { AgentCallRecord };
export type {
  AgentCallCorrelation,
  AgentCallListResponse,
  AgentCallOutputSpec,
  AgentCallPolicySpec,
  AgentCallUsage,
  AgentCallValidation,
} from '../generated/server-openapi/src/types/index.ts';

function pathSegment(value: string, name: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw new Error(`${name} is required.`);
  }
  return encodeURIComponent(normalized);
}

/**
 * Execute one structured agent call through `agents.calls.create`.
 *
 * Business outcomes (`validation_failed`, `agent_failed`, `timeout`) resolve
 * normally with `status` on the record; only transport/authorization
 * failures reject. Pass `executionMode: "async"` to queue the call and
 * observe the terminal state through {@link getAgentCall}.
 */
export async function createAgentCall(
  client: SdkworkAppClient,
  agentId: string,
  body: CreateAgentCallRequest,
): Promise<AgentCallResult> {
  const path = appApiPath(`/ai/agents/${pathSegment(agentId, 'agentId')}/calls`);
  return client.http.request<AgentCallResult>(path, {
    method: 'POST',
    body,
    contentType: 'application/json',
    sdkworkUnwrapKind: 'item',
    // The call is idempotent by executionId: replaying the same key cannot
    // re-execute it. Only transport-level failures are worth retrying.
    retry: {
      retryCondition: (error) =>
        error instanceof NetworkError || error instanceof TimeoutError,
    },
  });
}

/** Query parameters for {@link listAgentCalls}. */
export interface ListAgentCallsParams {
  status?: 'queued' | 'running' | 'completed' | 'failed';
  cursor?: string;
  page_size?: number;
}

/**
 * List durable structured agent calls through `agents.calls.list`.
 *
 * Keyset-paginated, ordered by (requestedAt, executionId) descending.
 */
export async function listAgentCalls(
  client: SdkworkAppClient,
  agentId: string,
  params: ListAgentCallsParams = {},
): Promise<{ items: AgentCallResult[]; nextCursor?: string }> {
  const search = new URLSearchParams();
  if (params.status) search.set('status', params.status);
  if (params.cursor) search.set('cursor', params.cursor);
  if (params.page_size !== undefined) {
    search.set('page_size', String(params.page_size));
  }
  const suffix = search.size > 0 ? `?${search.toString()}` : '';
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/calls${suffix}`,
  );
  const page = await client.http.request<{
    items: AgentCallResult[];
    pageInfo?: { nextCursor?: string };
  }>(path, {
    method: 'GET',
    sdkworkUnwrapKind: 'data',
    retry: {
      retryCondition: (error) =>
        error instanceof NetworkError || error instanceof TimeoutError,
    },
  });
  return {
    items: page.items,
    nextCursor: page.pageInfo?.nextCursor,
  };
}

/**
 * Retrieve one durable structured agent call through
 * `agents.calls.retrieve`.
 *
 * Poll this after {@link createAgentCall} with `executionMode: "async"`:
 * `status` starts as `queued`, then becomes one of the terminal outcomes
 * (`succeeded`, `validation_failed`, `agent_failed`, `timeout`, or the
 * recovery outcome `failed`).
 */
export async function getAgentCall(
  client: SdkworkAppClient,
  agentId: string,
  executionId: string,
): Promise<AgentCallResult> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/calls/${pathSegment(executionId, 'executionId')}`,
  );
  return client.http.request<AgentCallResult>(path, {
    method: 'GET',
    sdkworkUnwrapKind: 'item',
    retry: {
      retryCondition: (error) =>
        error instanceof NetworkError || error instanceof TimeoutError,
    },
  });
}
