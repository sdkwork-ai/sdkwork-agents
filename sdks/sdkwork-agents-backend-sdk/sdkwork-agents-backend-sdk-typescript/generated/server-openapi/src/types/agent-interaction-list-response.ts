import type { AgentInteractionRecord } from './agent-interaction-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated durable agent interaction response. */
export interface AgentInteractionListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentInteractionRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
