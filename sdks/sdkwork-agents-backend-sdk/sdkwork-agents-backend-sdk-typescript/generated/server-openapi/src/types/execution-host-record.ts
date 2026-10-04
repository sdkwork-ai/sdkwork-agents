import type { ExecutionHostKind } from './execution-host-kind';
import type { ExecutionHostStatus } from './execution-host-status';
import type { Int64String } from './int64-string';

export interface ExecutionHostRecord {
  id: Int64String;
  hostId: string;
  displayName?: string | null;
  hostKind: ExecutionHostKind;
  /** Opaque server-owned dispatch reference; never carries credentials. */
  endpoint: string;
  region?: string | null;
  maxConcurrentSessions: number;
  /** Bounded JSON capability document. */
  capabilitiesJson?: string | null;
  status: ExecutionHostStatus;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
}
