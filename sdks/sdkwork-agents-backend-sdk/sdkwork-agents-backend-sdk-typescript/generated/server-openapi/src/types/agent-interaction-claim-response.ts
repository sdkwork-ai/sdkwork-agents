import type { AgentInteractionRecord } from './agent-interaction-record';
import type { Int64String } from './int64-string';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Claimed interaction with a short-lived credential that must never be logged. */
export interface AgentInteractionClaimResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: { interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
