import type { AgentSessionCheckpointRecord } from './agent-session-checkpoint-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single resumable agent session checkpoint response. */
export interface AgentSessionCheckpointResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentSessionCheckpointRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
