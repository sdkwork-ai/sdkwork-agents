import type { AgentConfig } from '../services/AgentService';

export const DEFAULT_AGENT_CONFIG = {
  debugMode: true,
  jsonMode: false,
  knowledgeBaseIds: [],
  memoryEnabled: true,
  model: 'gpt-4',
  skillIds: [],
  suggestedPrompts: ['你能帮我做什么？', '给我讲个笑话'],
  temperature: 0.7,
  toolIds: [],
  mcpServerKeys: [],
  voiceIds: ['voice-1'],
  welcomeMessage: '你好！我是你的智能体，我们可以开始测试了。',
} satisfies Pick<
  AgentConfig,
  | 'debugMode'
  | 'jsonMode'
  | 'knowledgeBaseIds'
  | 'memoryEnabled'
  | 'model'
  | 'skillIds'
  | 'suggestedPrompts'
  | 'temperature'
  | 'toolIds'
  | 'mcpServerKeys'
  | 'voiceIds'
  | 'welcomeMessage'
>;
