import type { AgentWebhookSubscriptionCreated } from './agent-webhook-subscription-created';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Webhook subscription creation response following SdkWorkApiResponse envelope. */
export interface AgentWebhookSubscriptionCreatedResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentWebhookSubscriptionCreated; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
