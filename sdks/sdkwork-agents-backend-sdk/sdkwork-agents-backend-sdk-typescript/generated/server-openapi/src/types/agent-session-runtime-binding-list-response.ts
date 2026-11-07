import type { AgentSessionRuntimeBindingRecord } from './agent-session-runtime-binding-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated agent session runtime binding response. */
export interface AgentSessionRuntimeBindingListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentSessionRuntimeBindingRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
