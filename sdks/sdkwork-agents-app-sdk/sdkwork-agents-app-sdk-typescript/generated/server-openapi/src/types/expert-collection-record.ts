import type { ExpertCollectionMember } from './expert-collection-member';

export interface ExpertCollectionRecord {
  collectionCode: string;
  name: string;
  nameEn: string;
  description: string;
  descriptionEn: string;
  /** Stable key the client maps to its own illustration. */
  coverKey: string;
  sortOrder: number;
  experts: ExpertCollectionMember[];
}
