import React, { memo } from 'react';
import { FileText } from 'lucide-react';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
import type { ChatTurn } from '../utils/chatTurnGrouping';
import { UserMessageItem } from './UserMessageItem';
import { BotMessageItem } from './BotMessageItem';
import { ImagePreviewList } from './ImagePreviewList';

interface ChatTurnRowProps {
  turn: ChatTurn;
  copiedId: string | null;
  feedback: Record<string, 'up' | 'down'>;
  handleCopy: (text: string, id: string) => void;
  handleFeedback: (id: string, type: 'up' | 'down') => void;
  onOpenArtifact: (lang: string, code: string, mode?: 'preview' | 'code') => void;
  streamingMessageId?: string | null;
}

function renderAttachments(message: ChatMessage, isUser: boolean) {
  const nonImageResources = message.mediaResources?.filter((resource) => resource.kind !== 'image') ?? [];
  if (nonImageResources.length === 0) {
    return null;
  }

  return (
    <div className={`mb-2 flex max-w-full flex-wrap gap-2 ${isUser ? 'justify-end' : 'justify-start'}`}>
      {nonImageResources.map((resource) => {
        const content = (
          <>
            <FileText size={16} aria-hidden="true" />
            <span className="max-w-72 truncate">{resource.fileName ?? resource.id}</span>
            {resource.sizeBytes && (
              <span className="text-xs opacity-60">{resource.sizeBytes} B</span>
            )}
          </>
        );
        return resource.url ? (
          <a
            key={`${resource.id}-${resource.kind}`}
            href={resource.url}
            target="_blank"
            rel="noreferrer"
            aria-label={resource.fileName ?? resource.id}
            className="flex min-h-9 items-center gap-2 rounded-md border border-gray-200 bg-white px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-200 dark:hover:bg-gray-800"
          >
            {content}
          </a>
        ) : (
          <div
            key={`${resource.id}-${resource.kind}`}
            className="flex min-h-9 items-center gap-2 rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-400"
          >
            {content}
          </div>
        );
      })}
    </div>
  );
}

export const ChatTurnRow: React.FC<ChatTurnRowProps> = memo(({
  turn,
  copiedId,
  feedback,
  handleCopy,
  handleFeedback,
  onOpenArtifact,
  streamingMessageId = null,
}) => {
  const userMessage = turn.user;
  const assistantMessage = turn.assistant;
  const isStreaming = assistantMessage?.id === streamingMessageId;

  return (
    <article className="chat-turn">
      {userMessage && (
        <div className="chat-turn__user">
          <div className="chat-turn__user-inner">
            <ImagePreviewList images={userMessage.images || []} isUser />
            {renderAttachments(userMessage, true)}
            <UserMessageItem
              message={userMessage}
              copiedId={copiedId}
              handleCopy={handleCopy}
            />
          </div>
        </div>
      )}

      {assistantMessage && (
        <div className="chat-turn__assistant">
          <ImagePreviewList images={assistantMessage.images || []} isUser={false} />
          {renderAttachments(assistantMessage, false)}
          <BotMessageItem
            message={assistantMessage}
            copiedId={copiedId}
            feedback={feedback}
            handleCopy={handleCopy}
            handleFeedback={handleFeedback}
            onOpenArtifact={onOpenArtifact}
            isStreaming={isStreaming}
          />
        </div>
      )}
    </article>
  );
});
ChatTurnRow.displayName = 'ChatTurnRow';
