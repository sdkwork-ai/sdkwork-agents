import type { AgentTurnRecord } from './agent-turn-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated durable agent turn response. */
export interface AgentTurnListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentTurnRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
