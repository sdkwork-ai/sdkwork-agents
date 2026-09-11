import React, { useState, useMemo } from 'react';
import { Sparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
import { ChatTurnRow } from './ChatTurnRow';
import { groupMessagesIntoTurns } from '../utils/chatTurnGrouping';
import './chat-turn-list.css';

interface MessageListProps {
  messages: ChatMessage[];
  messagesEndRef: React.RefObject<HTMLDivElement | null>;
  onOpenArtifact: (lang: string, code: string, mode?: 'preview' | 'code') => void;
  onFeedback: (messageId: string, rating: 'up' | 'down' | undefined) => Promise<boolean>;
  streamingMessageId?: string | null;
  welcomeTitle?: string;
  welcomeDescription?: string;
}

export const MessageList: React.FC<MessageListProps> = ({
  messages,
  messagesEndRef,
  onOpenArtifact,
  onFeedback,
  streamingMessageId = null,
  welcomeTitle,
  welcomeDescription,
}) => {
  const { t } = useTranslation('chat');
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const feedback = useMemo(
    () => Object.fromEntries(
      messages
        .filter((message) => message.feedback)
        .map((message) => [message.id, message.feedback]),
    ) as Record<string, 'up' | 'down'>,
    [messages],
  );

  const turns = useMemo(() => groupMessagesIntoTurns(messages), [messages]);

  const handleCopy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const handleFeedback = (id: string, type: 'up' | 'down') => {
    const rating = feedback[id] === type ? undefined : type;
    void onFeedback(id, rating).catch(() => undefined);
  };

  if (messages.length === 0) {
    return (
      <div className="h-[calc(100cqh-200px)] flex flex-col items-center justify-center p-8 text-center animate-in fade-in zoom-in duration-500 ease-out">
        <div className="w-16 h-16 bg-gradient-to-tr from-[#1890ff] to-[#096dd9] text-white rounded-2xl flex items-center justify-center mb-6 shadow-[#1890ff]/20 shadow-xl border border-[#1890ff]/30 ring-4 ring-[#1890ff]/10">
          <Sparkles size={32} className="text-white drop-shadow-md" />
        </div>
        <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-3 tracking-tight">
          {welcomeTitle ?? t('howCanIHelp')}
        </h2>
        <p className="text-[15px] leading-relaxed text-gray-500 dark:text-gray-400 max-w-sm">
          {welcomeDescription ?? t('welcomeDescription')}
        </p>
      </div>
    );
  }

  return (
    <div className="chat-turn-list mx-auto w-full px-4 pb-48 pt-8 sm:px-6">
      {turns.map((turn) => (
        <ChatTurnRow
          key={turn.id}
          turn={turn}
          copiedId={copiedId}
          feedback={feedback}
          handleCopy={handleCopy}
          handleFeedback={handleFeedback}
          onOpenArtifact={onOpenArtifact}
          streamingMessageId={streamingMessageId}
        />
      ))}
      <div ref={messagesEndRef} />
    </div>
  );
};
