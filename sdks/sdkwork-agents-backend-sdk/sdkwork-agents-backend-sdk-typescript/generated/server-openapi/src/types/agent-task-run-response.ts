import type { AgentTaskRunRecord } from './agent-task-run-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

export interface AgentTaskRunResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentTaskRunRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
