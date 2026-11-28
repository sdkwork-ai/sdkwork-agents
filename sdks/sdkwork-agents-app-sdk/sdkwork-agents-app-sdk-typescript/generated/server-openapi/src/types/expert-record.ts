export interface ExpertRecord {
  slug: string;
  agentId: string;
  name: string;
  description: string;
  emoji: string;
  divisionCode: string;
  tags: string[];
  /** Curated editorial usage figure for the expert market; never telemetry. */
  usageCount: number;
  /** Optional short editorial badge code such as `leader`, `official` or `enterprise`. */
  badgeCode: string | null;
  /** Catalog order of the expert inside its division. */
  sortOrder: number;
  publishedAt: string | null;
}
