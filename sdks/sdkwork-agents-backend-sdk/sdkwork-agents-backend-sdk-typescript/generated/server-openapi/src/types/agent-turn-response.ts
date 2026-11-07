import type { AgentTurnRecord } from './agent-turn-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single durable agent turn response. */
export interface AgentTurnResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentTurnRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
