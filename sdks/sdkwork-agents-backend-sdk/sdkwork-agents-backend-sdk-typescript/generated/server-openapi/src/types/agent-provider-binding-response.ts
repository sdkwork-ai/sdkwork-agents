import type { AgentProviderBindingRecord } from './agent-provider-binding-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single agent provider binding response following SdkWorkApiResponse envelope. */
export interface AgentProviderBindingResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentProviderBindingRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
