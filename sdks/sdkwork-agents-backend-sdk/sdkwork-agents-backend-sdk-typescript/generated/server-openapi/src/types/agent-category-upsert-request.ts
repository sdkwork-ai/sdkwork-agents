/** Admin upsert body for the category identified by the path code. `expectedVersion` enables optimistic concurrency; `status` is admin-managed except for the server-owned `deleted` state. */
export interface AgentCategoryUpsertRequest {
  name: string;
  description?: string;
  parentId?: string | null;
  sortOrder?: number;
  iconRef?: string;
  status?: 'draft' | 'active' | 'disabled' | 'archived';
  /** Version the operator last read; a stale value returns HTTP 409. */
  expectedVersion?: string | null;
}
