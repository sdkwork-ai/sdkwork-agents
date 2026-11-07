import type { AgentSessionRecord } from './agent-session-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated agent session response following the SDKWork page envelope. */
export interface AgentSessionListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentSessionRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
