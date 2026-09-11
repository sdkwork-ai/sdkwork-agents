import type { SdkworkAppClient } from '../generated/server-openapi/src/index.ts';
import { appApiPath } from '../generated/server-openapi/src/api/paths.ts';
import type {
  AgentUsageRecord,
  AgentUsageSummary,
} from '../generated/server-openapi/src/types/index.ts';

export type { AgentUsageRecord, AgentUsageSummary };

/** Optional metering filters shared by the usage feeds. */
export interface UsageFilterParams {
  agent_id?: string;
  session_id?: string;
  model_id?: string;
  /** Inclusive RFC 3339 lower bound on turn creation time. */
  from?: string;
  /** Exclusive RFC 3339 upper bound on turn creation time. */
  to?: string;
}

function usageSearchParams(params: UsageFilterParams): URLSearchParams {
  const search = new URLSearchParams();
  if (params.agent_id) search.set('agentId', params.agent_id);
  if (params.session_id) search.set('sessionId', params.session_id);
  if (params.model_id) search.set('modelId', params.model_id);
  if (params.from) search.set('from', params.from);
  if (params.to) search.set('to', params.to);
  return search;
}

/**
 * Aggregate usage totals through `agents.usage.summary`.
 *
 * Token and turn totals for the current tenant scope computed from the
 * durable turn facts. Billing and quota enforcement stay owned by the
 * platform gateway.
 */
export async function getUsageSummary(
  client: SdkworkAppClient,
  params: UsageFilterParams = {},
): Promise<AgentUsageSummary> {
  const search = usageSearchParams(params);
  const suffix = search.size > 0 ? `?${search.toString()}` : '';
  const path = appApiPath(`/ai/usage/summary${suffix}`);
  return client.http.request<AgentUsageSummary>(path, {
    method: 'GET',
    sdkworkUnwrapKind: 'item',
  });
}

/** Query parameters for {@link listUsageRecords}. */
export interface ListUsageRecordsParams extends UsageFilterParams {
  cursor?: string;
  page_size?: number;
}

/**
 * List turn-level usage records through `agents.usage.records`.
 *
 * Keyset-paginated, ordered by (createdAt, id) descending.
 */
export async function listUsageRecords(
  client: SdkworkAppClient,
  params: ListUsageRecordsParams = {},
): Promise<{ items: AgentUsageRecord[]; nextCursor?: string }> {
  const search = usageSearchParams(params);
  if (params.cursor) search.set('cursor', params.cursor);
  if (params.page_size !== undefined) {
    search.set('page_size', String(params.page_size));
  }
  const suffix = search.size > 0 ? `?${search.toString()}` : '';
  const path = appApiPath(`/ai/usage/records${suffix}`);
  const page = await client.http.request<{
    items: AgentUsageRecord[];
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
