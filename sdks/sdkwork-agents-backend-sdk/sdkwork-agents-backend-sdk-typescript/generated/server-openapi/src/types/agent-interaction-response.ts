import type { AgentInteractionRecord } from './agent-interaction-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single durable agent interaction response. */
export interface AgentInteractionResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentInteractionRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
