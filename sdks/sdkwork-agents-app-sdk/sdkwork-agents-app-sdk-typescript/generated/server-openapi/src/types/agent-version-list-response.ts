import type { AgentVersionRecord } from './agent-version-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Agent version list response following SdkWorkApiResponse envelope. */
export interface AgentVersionListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentVersionRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
