import type { ResolvedSkill } from './resolved-skill';
import type { TurnToolDescriptor } from './turn-tool-descriptor';

/** Effective toolkit of one agent (model-visible MCP tools, attached skills, and the assembled system prompt). */
export interface AgentToolkitOverview {
  agentId: string;
  tools: TurnToolDescriptor[];
  skills: ResolvedSkill[];
  assembledSystemPrompt?: string | null;
}
