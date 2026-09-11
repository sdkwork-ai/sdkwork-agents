export {
  ChatService,
  callerScopeGrantsAgentManage,
  configureChatAgentPermissionScopeReader,
  configureChatAgentPort,
} from './services/ChatService';
export type {
  ChatAgentConfig,
  ChatAgentPermissionScopeReader,
  ChatAgentPort,
  ChatServiceOptions,
} from './services/ChatService';
export { ProjectService, configureProjectPort } from './services/ProjectService';
export type {
  ChatMemorySpaceOption,
  ChatProject,
  ChatProjectCompositionSlot,
  ProjectDetails,
  ProjectPort,
  ProjectSettingsData,
} from './services/ProjectService';
export type { ChatMessage, ChatSession, MessageRole } from './types';
export type { ChatPcSession } from './session';
