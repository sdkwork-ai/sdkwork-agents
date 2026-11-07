import type { AgentCategoryRecord } from './agent-category-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single managed agent category response following SdkWorkApiResponse envelope. */
export interface AgentCategoryResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentCategoryRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
