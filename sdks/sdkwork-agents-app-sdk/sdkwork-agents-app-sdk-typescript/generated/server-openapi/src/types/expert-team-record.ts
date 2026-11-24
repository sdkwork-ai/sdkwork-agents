import type { ExpertTeamMember } from './expert-team-member';

export interface ExpertTeamRecord {
  teamId: string;
  name: string;
  description: string;
  tags: string[];
  goal: string;
  deliveryRequirements: string;
  examples: string[];
  coordinatorMode: 'template' | 'custom';
  coordinatorTemplateId: 'general' | 'product' | 'technical' | 'content' | 'data' | 'research';
  locale: 'zh-CN' | 'en-US';
  members: ExpertTeamMember[];
}
