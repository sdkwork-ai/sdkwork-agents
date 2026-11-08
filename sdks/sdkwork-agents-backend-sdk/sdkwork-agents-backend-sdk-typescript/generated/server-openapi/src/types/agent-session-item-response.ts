import type { AgentSessionItemRecord } from './agent-session-item-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single agent session item response following the SDKWork resource envelope. */
export interface AgentSessionItemResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentSessionItemRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
