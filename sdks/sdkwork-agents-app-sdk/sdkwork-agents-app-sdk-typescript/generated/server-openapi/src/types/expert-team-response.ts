import type { ExpertTeamRecord } from './expert-team-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single expert team response following SdkWorkApiResponse envelope. */
export interface ExpertTeamResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: ExpertTeamRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
