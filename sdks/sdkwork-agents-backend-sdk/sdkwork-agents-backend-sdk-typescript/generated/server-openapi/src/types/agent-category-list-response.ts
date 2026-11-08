import type { AgentCategoryRecord } from './agent-category-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated managed agent category list response following SdkWorkApiResponse envelope. */
export interface AgentCategoryListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentCategoryRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
