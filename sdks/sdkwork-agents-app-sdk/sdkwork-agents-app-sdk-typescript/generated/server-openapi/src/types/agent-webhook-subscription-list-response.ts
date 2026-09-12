import type { AgentWebhookSubscription } from './agent-webhook-subscription';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Webhook subscription list response following SdkWorkApiResponse envelope. */
export interface AgentWebhookSubscriptionListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentWebhookSubscription[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
