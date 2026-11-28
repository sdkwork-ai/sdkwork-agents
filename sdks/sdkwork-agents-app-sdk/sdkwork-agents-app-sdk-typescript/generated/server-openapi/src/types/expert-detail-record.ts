/** One expert summary plus the persona served for the resolved locale. */
export interface ExpertDetailRecord {
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
  persona: string;
  personaLocale: 'zh-CN' | 'en-US';
  personaLocales: ('zh-CN' | 'en-US')[];
  contentSha256: string;
}
