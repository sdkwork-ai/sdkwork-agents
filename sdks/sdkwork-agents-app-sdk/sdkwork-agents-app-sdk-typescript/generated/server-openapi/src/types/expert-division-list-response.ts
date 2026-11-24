import type { ExpertDivisionRecord } from './expert-division-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated expert division list response following SdkWorkApiResponse envelope. */
export interface ExpertDivisionListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: ExpertDivisionRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
