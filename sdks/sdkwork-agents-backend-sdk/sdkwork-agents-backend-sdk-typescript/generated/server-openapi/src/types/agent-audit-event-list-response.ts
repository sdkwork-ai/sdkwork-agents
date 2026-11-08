import type { AgentAuditEvent } from './agent-audit-event';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated agent audit event list response following SdkWorkApiResponse envelope. */
export interface AgentAuditEventListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentAuditEvent[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
