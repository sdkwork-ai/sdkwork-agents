import type { AgentSessionItemRecord } from './agent-session-item-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated agent session item response following the SDKWork page envelope. */
export interface AgentSessionItemListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentSessionItemRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
