import {
  agentChatService,
  agentProjectService,
  agentService,
} from '@sdkwork/agents-pc-agents/services';
import { agentsDriveUploadService } from '@sdkwork/agents-pc-core/sdk/driveUploadService';
import { configureChatAgentPort, configureProjectPort } from '@sdkwork/agents-pc-chat';

export function configureAgentsWorkbenchPorts(): void {
  configureChatAgentPort({
    getAgent: (agentId) => agentService.getAgent(agentId),
    createAgent: (agent) => agentService.createAgent(agent),
    updateAgent: (agentId, patch) => agentService.updateAgent(agentId, patch),
    resolveOrCreateSession: (agentId, sessionId, title) =>
      agentChatService.resolveOrCreateNamedSession(agentId, sessionId, title),
    createSession: (agentId, title) => agentChatService.createSessionSummary(agentId, title),
    listSessions: (agentId) => agentChatService.listSessionSummaries(agentId),
    updateSession: (agentId, sessionId, patch) =>
      agentChatService.updateSession(agentId, sessionId, patch),
    deleteSession: (agentId, sessionId) => agentChatService.deleteSession(agentId, sessionId),
    listSessionUserStates: (agentId, pinnedOnly) =>
      agentChatService.listSessionUserStates(agentId, pinnedOnly),
    updateSessionUserState: (agentId, sessionId, patch) =>
      agentChatService.updateSessionUserState(agentId, sessionId, patch),
    listMessageFeedback: (agentId, sessionId) =>
      agentChatService.listMessageFeedback(agentId, sessionId),
    updateMessageFeedback: (agentId, sessionId, messageId, patch) =>
      agentChatService.updateMessageFeedback(agentId, sessionId, messageId, patch),
    listMessages: (agentId, sessionId) => agentChatService.listMessages(agentId, sessionId),
    resolveMediaPreviewUrl: (driveUri) => agentsDriveUploadService.resolvePreviewUrl(driveUri),
    sendMessage: (agentId, sessionId, content, model, media, systemPrompt, wireProtocol) =>
      agentChatService.sendMessage(agentId, sessionId, content, model, media, systemPrompt, wireProtocol),
    sendMessageStream: (agentId, sessionId, content, model, media, onDelta, systemPrompt, onReasoning, onToolEvent, wireProtocol) =>
      agentChatService.sendMessageStream(
        agentId, sessionId, content, model, media, onDelta, systemPrompt, onReasoning, onToolEvent, wireProtocol,
      ),
  });
  configureProjectPort({
    list: () => agentProjectService.list(),
    retrieve: (projectId) => agentProjectService.retrieve(projectId),
    create: (input) => agentProjectService.create(input),
    update: (projectId, patch) => agentProjectService.update(projectId, patch),
    delete: (projectId) => agentProjectService.delete(projectId),
    listCompositionSlots: (projectId) => agentProjectService.listCompositionSlots(projectId),
    getInstructions: (slots) => agentProjectService.getInstructions(slots),
    saveInstructions: (project, slots, content) =>
      agentProjectService.saveInstructions(project, slots, content),
    listMemorySpaces: () => agentProjectService.listMemorySpaces(),
    saveMemorySpace: (projectId, slots, memorySpaceId) =>
      agentProjectService.saveMemorySpace(projectId, slots, memorySpaceId),
  });
}
