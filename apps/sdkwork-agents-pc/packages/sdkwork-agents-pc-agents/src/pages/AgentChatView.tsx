import { useEffect, useState } from 'react';

import { ChatView } from '@sdkwork/agents-pc-chat/ChatView';

import { agentService } from '../services/AgentService';

export interface AgentChatViewProps {
  agentId: string;
  agentName?: string;
  welcomeMessage?: string;
  onBack: () => void;
}

export const AgentChatView = ({
  agentId,
  agentName: initialAgentName,
  welcomeMessage: initialWelcomeMessage,
  onBack,
}: AgentChatViewProps) => {
  const [agentName, setAgentName] = useState(initialAgentName);
  const [welcomeMessage, setWelcomeMessage] = useState(initialWelcomeMessage);
  const [agentSystemPrompt, setAgentSystemPrompt] = useState<string>();
  const [agentModelId, setAgentModelId] = useState<string>();

  useEffect(() => {
    setAgentName(initialAgentName);
    setWelcomeMessage(initialWelcomeMessage);
  }, [initialAgentName, initialWelcomeMessage, agentId]);

  useEffect(() => {
    let cancelled = false;
    void agentService.getAgent(agentId)
      .then((agent) => {
        if (!agent || cancelled) {
          return;
        }
        setAgentName(agent.name);
        if (agent.welcomeMessage) {
          setWelcomeMessage(agent.welcomeMessage);
        }
        if (agent.systemPrompt) {
          setAgentSystemPrompt(agent.systemPrompt);
        }
        if (agent.model) {
          setAgentModelId(agent.model);
        }
      })
      .catch((error) => console.error('Agent chat scope bootstrap failed', error));
    return () => {
      cancelled = true;
    };
  }, [agentId]);

  return (
    <ChatView
      agentId={agentId}
      agentName={agentName}
      welcomeMessage={welcomeMessage}
      agentSystemPrompt={agentSystemPrompt}
      agentModelId={agentModelId}
      onBack={onBack}
    />
  );
};
