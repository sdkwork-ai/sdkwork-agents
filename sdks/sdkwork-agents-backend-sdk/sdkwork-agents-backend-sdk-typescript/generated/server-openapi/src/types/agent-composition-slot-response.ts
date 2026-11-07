import type { AgentCompositionSlotRecord } from './agent-composition-slot-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single agent composition slot response following SdkWorkApiResponse envelope. */
export interface AgentCompositionSlotResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentCompositionSlotRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
