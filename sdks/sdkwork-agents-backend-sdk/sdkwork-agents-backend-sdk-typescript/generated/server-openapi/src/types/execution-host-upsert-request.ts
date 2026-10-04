import type { ExecutionHostKind } from './execution-host-kind';
import type { ExecutionHostStatus } from './execution-host-status';
import type { Int64String } from './int64-string';

export interface ExecutionHostUpsertRequest {
  displayName?: string | null;
  hostKind: ExecutionHostKind;
  endpoint: string;
  region?: string | null;
  maxConcurrentSessions: number;
  capabilitiesJson?: string | null;
  status?: ExecutionHostStatus;
  /** Optimistic version of the existing row; omit or null to register a new host. */
  expectedVersion?: Int64String | null;
}
