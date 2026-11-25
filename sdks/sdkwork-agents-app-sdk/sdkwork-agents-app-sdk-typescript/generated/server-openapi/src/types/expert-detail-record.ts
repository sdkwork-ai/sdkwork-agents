/** One expert summary plus the persona served for the resolved locale. */
export interface ExpertDetailRecord {
  slug: string;
  agentId: string;
  name: string;
  description: string;
  emoji: string;
  divisionCode: string;
  tags: string[];
  persona: string;
  personaLocale: 'zh-CN' | 'en-US';
  personaLocales: ('zh-CN' | 'en-US')[];
  contentSha256: string;
}
