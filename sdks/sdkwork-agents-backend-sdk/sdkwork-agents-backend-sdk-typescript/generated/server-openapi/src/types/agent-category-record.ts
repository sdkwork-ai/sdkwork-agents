import type { Int64String } from './int64-string';

/** One managed agent category row. Categories form a tenant-scoped tree of at most five levels; `parentId` is the optional same-tenant parent, `iconRef` is an optional presentation reference, and `tags` / `attributes` carry free-form taxonomy metadata. */
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
  /** Free-form taxonomy tags used for filtering and display. */
  tags: string[];
  /** Free-form category attributes keyed by operator-defined names. */
  attributes: Record<string, string>;
  lifecycleStatus: 'draft' | 'active' | 'disabled' | 'archived' | 'deleted';
  version: Int64String;
  createdAt: string;
  updatedAt: string;
}
