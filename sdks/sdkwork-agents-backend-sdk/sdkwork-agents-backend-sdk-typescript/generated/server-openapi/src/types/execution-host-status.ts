/** Registry lifecycle of an execution host. Only `active` hosts are scheduler-eligible. */
export type ExecutionHostStatus = 'active' | 'draining' | 'disabled';
