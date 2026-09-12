import type { AgentToolkitOverview } from './agent-toolkit-overview';

/** Effective agent toolkit following the SdkWorkApiResponse envelope; the toolkit overview is returned under data. */
export interface AgentToolkitOverviewResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & AgentToolkitOverview;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
