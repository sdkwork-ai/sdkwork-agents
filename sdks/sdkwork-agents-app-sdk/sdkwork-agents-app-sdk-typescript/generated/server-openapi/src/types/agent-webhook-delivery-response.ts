import type { AgentWebhookDelivery } from './agent-webhook-delivery';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Webhook delivery response following SdkWorkApiResponse envelope. */
export interface AgentWebhookDeliveryResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentWebhookDelivery; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
