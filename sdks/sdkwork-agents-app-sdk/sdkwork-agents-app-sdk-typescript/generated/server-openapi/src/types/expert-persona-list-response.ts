import type { ExpertPersonaRecord } from './expert-persona-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated expert persona list response following SdkWorkApiResponse envelope. */
export interface ExpertPersonaListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: ExpertPersonaRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
