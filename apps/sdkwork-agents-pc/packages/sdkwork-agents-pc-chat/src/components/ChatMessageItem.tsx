import React from 'react';
import { FileText } from 'lucide-react';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
import { cn } from '@sdkwork/agents-pc-commons';
import { UserMessageItem } from './UserMessageItem';
import { BotMessageItem } from './BotMessageItem';
import { ImagePreviewList } from './ImagePreviewList';

interface ChatMessageItemProps {
  message: ChatMessage;
  copiedId: string | null;
  feedback: Record<string, 'up' | 'down'>;
  handleCopy: (text: string, id: string) => void;
  handleFeedback: (id: string, type: 'up' | 'down') => void;
  onOpenArtifact: (lang: string, code: string, mode?: 'preview' | 'code') => void;
  isStreaming?: boolean;
}

export const ChatMessageItem: React.FC<ChatMessageItemProps> = ({
  message,
  copiedId,
  feedback,
  handleCopy,
  handleFeedback,
  onOpenArtifact,
  isStreaming = false,
}) => {
  return (
    <div
      className={cn(
        "flex w-full mb-6",
        message.role === 'user' ? "justify-end" : "justify-start"
      )}
    >
      <div className={cn(
        "flex flex-col min-w-0 w-full",
        message.role === 'user' ? "items-end" : "items-start"
      )}>
        <ImagePreviewList images={message.images || []} isUser={message.role === 'user'} />
        {message.mediaResources?.some((resource) => resource.kind !== 'image') && (
          <div className="mb-2 flex max-w-xl flex-wrap justify-end gap-2">
            {message.mediaResources
              .filter((resource) => resource.kind !== 'image')
              .map((resource) => {
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
        )}
        
        {message.role === 'user' ? (
          <UserMessageItem 
            message={message} 
            copiedId={copiedId} 
            handleCopy={handleCopy} 
          />
        ) : (
          <BotMessageItem 
            message={message} 
            copiedId={copiedId} 
            feedback={feedback}
            handleCopy={handleCopy}
            handleFeedback={handleFeedback}
            onOpenArtifact={onOpenArtifact}
            isStreaming={isStreaming}
          />
        )}
      </div>
    </div>
  );
};
