import type { AgentSessionCheckpointRecord } from './agent-session-checkpoint-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated agent session checkpoint response. */
export interface AgentSessionCheckpointListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentSessionCheckpointRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
