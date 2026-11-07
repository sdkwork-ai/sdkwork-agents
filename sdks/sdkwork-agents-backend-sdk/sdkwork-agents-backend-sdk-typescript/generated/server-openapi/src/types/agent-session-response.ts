import type { AgentSessionRecord } from './agent-session-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single agent session response following the SDKWork resource envelope. */
export interface AgentSessionResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentSessionRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
