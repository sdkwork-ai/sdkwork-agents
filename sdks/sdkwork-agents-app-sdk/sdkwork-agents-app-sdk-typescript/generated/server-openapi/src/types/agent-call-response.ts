import type { AgentCallRecord } from './agent-call-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Structured agent call response following SdkWorkApiResponse envelope. */
export interface AgentCallResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentCallRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
