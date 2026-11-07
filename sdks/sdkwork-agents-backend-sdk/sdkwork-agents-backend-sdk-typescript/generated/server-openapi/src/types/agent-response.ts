import type { AgentRecord } from './agent-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single managed agent response following SdkWorkApiResponse envelope. */
export interface AgentResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
