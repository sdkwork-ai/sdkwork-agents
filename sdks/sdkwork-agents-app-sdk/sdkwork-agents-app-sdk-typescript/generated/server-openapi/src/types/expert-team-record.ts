import type { ExpertTeamMember } from './expert-team-member';

export interface ExpertTeamRecord {
  teamId: string;
  name: string;
  description: string;
  tags: string[];
  goal: string;
  /** The team's shared constraints. The catalog authors them once, so every locale serves the same text. */
  constraints: string;
  deliveryRequirements: string;
  examples: string[];
  coordinatorMode: 'template' | 'custom';
  coordinatorTemplateId: 'general' | 'product' | 'technical' | 'content' | 'data' | 'research';
  /** A custom coordinator's own prompt; empty for a template coordinator, whose text the app resolves from coordinatorTemplateId. */
  coordinatorPrompt: string;
  locale: 'zh-CN' | 'en-US';
  members: ExpertTeamMember[];
}
