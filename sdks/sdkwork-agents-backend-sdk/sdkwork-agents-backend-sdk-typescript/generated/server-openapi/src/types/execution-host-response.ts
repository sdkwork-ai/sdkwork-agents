import type { ExecutionHostRecord } from './execution-host-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single execution host response following SdkWorkApiResponse envelope. */
export interface ExecutionHostResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: ExecutionHostRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
