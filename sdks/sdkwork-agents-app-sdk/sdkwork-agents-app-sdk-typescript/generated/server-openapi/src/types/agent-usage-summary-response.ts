import type { AgentUsageSummary } from './agent-usage-summary';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Usage aggregation response following SdkWorkApiResponse envelope. */
export interface AgentUsageSummaryResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentUsageSummary; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
