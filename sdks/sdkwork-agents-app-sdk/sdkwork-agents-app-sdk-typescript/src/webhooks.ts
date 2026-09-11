import type { SdkworkAppClient } from '../generated/server-openapi/src/index.ts';
import { appApiPath } from '../generated/server-openapi/src/api/paths.ts';
import type {
  AgentWebhookDelivery,
  AgentWebhookSubscription,
  AgentWebhookSubscriptionCreated,
  CreateWebhookSubscriptionRequest,
} from '../generated/server-openapi/src/types/index.ts';
import { NetworkError, TimeoutError } from '@sdkwork/sdk-common';

export type {
  AgentWebhookDelivery,
  AgentWebhookEventType,
  AgentWebhookSubscription,
  AgentWebhookSubscriptionCreated,
  AgentWebhookSubscriptionCreatedResponse,
  AgentWebhookSubscriptionListResponse,
  AgentWebhookSubscriptionResponse,
  CreateWebhookSubscriptionRequest,
} from '../generated/server-openapi/src/types/index.ts';

function pathSegment(value: string, name: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw new Error(`${name} is required.`);
  }
  return encodeURIComponent(normalized);
}

/**
 * Register a webhook subscription through `agents.webhooks.create`. An HMAC
 * signing secret is generated server-side and echoed exactly once in the
 * created subscription; it is never returned again.
 */
export async function createWebhookSubscription(
  client: SdkworkAppClient,
  body: CreateWebhookSubscriptionRequest,
): Promise<AgentWebhookSubscriptionCreated> {
  const path = appApiPath('/ai/webhooks');
  return client.http.request<AgentWebhookSubscriptionCreated>(path, {
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

/** Query parameters for {@link listWebhookSubscriptions}. */
export interface ListWebhooksParams {
  page?: number;
  page_size?: number;
}

/**
 * List the tenant's webhook subscriptions through `agents.webhooks.list`.
 *
 * Offset-paginated (a low-volume configuration set); signing secrets are
 * never included.
 */
export async function listWebhookSubscriptions(
  client: SdkworkAppClient,
  params: ListWebhooksParams = {},
): Promise<{ items: AgentWebhookSubscription[] }> {
  const search = new URLSearchParams();
  if (params.page !== undefined) search.set('page', String(params.page));
  if (params.page_size !== undefined) {
    search.set('page_size', String(params.page_size));
  }
  const suffix = search.size > 0 ? `?${search.toString()}` : '';
  const path = appApiPath(`/ai/webhooks${suffix}`);
  const page = await client.http.request<{
    items: AgentWebhookSubscription[];
  }>(path, {
    method: 'GET',
    sdkworkUnwrapKind: 'data',
  });
  return { items: page.items };
}

/** Retrieve one webhook subscription (without its signing secret). */
export async function getWebhookSubscription(
  client: SdkworkAppClient,
  webhookId: string,
): Promise<AgentWebhookSubscription> {
  const path = appApiPath(
    `/ai/webhooks/${pathSegment(webhookId, 'webhookId')}`,
  );
  return client.http.request<AgentWebhookSubscription>(path, {
    method: 'GET',
    sdkworkUnwrapKind: 'item',
  });
}

/** Delete a webhook subscription; past deliveries stay in the ledger. */
export async function deleteWebhookSubscription(
  client: SdkworkAppClient,
  webhookId: string,
): Promise<void> {
  const path = appApiPath(
    `/ai/webhooks/${pathSegment(webhookId, 'webhookId')}`,
  );
  await client.http.request<void>(path, {
    method: 'DELETE',
    sdkworkUnwrapKind: 'void',
  });
}

/**
 * Send a signed `agent_call.completed` test delivery through
 * `agents.webhooks.test` and return the recorded delivery outcome.
 */
export async function testWebhookDelivery(
  client: SdkworkAppClient,
  webhookId: string,
): Promise<AgentWebhookDelivery> {
  const path = appApiPath(
    `/ai/webhooks/${pathSegment(webhookId, 'webhookId')}/test`,
  );
  return client.http.request<AgentWebhookDelivery>(path, {
    method: 'POST',
    sdkworkUnwrapKind: 'item',
    retry: {
      retryCondition: (error) =>
        error instanceof NetworkError || error instanceof TimeoutError,
    },
  });
}
