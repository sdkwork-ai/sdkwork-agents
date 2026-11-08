import type { Int64String } from './int64-string';

/** One managed agent category row. Categories form a tenant-scoped taxonomy of at most two levels; `parentId` is the optional same-tenant parent and `iconRef` is an optional presentation reference. */
export interface AgentCategoryRecord {
  id: Int64String;
  uuid: string;
  tenantId: Int64String;
  organizationId: Int64String;
  categoryCode: string;
  name: string;
  description?: string | null;
  /** Same-tenant parent category id, or null for a top-level category. */
  parentId: string | null;
  sortOrder: number;
  /** Optional presentation reference (drive URI or URL) for the category icon. */
  iconRef: string | null;
  lifecycleStatus: 'draft' | 'active' | 'disabled' | 'archived' | 'deleted';
  version: Int64String;
  createdAt: string;
  updatedAt: string;
}
