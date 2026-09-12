import type { AgentWebhookSubscription } from './agent-webhook-subscription';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Webhook subscription response following SdkWorkApiResponse envelope. */
export interface AgentWebhookSubscriptionResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentWebhookSubscription; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
