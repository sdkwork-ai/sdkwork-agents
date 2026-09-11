import React from 'react';
import { Copy, Check, ThumbsUp, ThumbsDown } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
import { MarkdownRenderer, cn } from '@sdkwork/agents-pc-commons';
import { TypingIndicator } from './TypingIndicator';

interface BotMessageItemProps {
  message: ChatMessage;
  copiedId: string | null;
  feedback: Record<string, 'up' | 'down'>;
  handleCopy: (text: string, id: string) => void;
  handleFeedback: (id: string, type: 'up' | 'down') => void;
  onOpenArtifact: (lang: string, code: string, mode?: 'preview' | 'code') => void;
}

export const BotMessageItem: React.FC<BotMessageItemProps> = ({
  message,
  copiedId,
  feedback,
  handleCopy,
  handleFeedback,
  onOpenArtifact
}) => {
  const { t: tCommon } = useTranslation('common');

  const isGenerating = !message.text;

  return (
    <div className="flex items-start w-full">
      <div className="text-[15px] leading-7 text-gray-800 dark:text-gray-200 w-full flex flex-col gap-2 group relative">
        {isGenerating ? (
          <TypingIndicator />
        ) : (
          <MarkdownRenderer content={message.text} onOpenArtifact={onOpenArtifact} />
        )}
        
        {!isGenerating && (
          <div className="flex items-center space-x-1 mt-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button 
              onClick={() => handleCopy(message.text, message.id)}
              className="p-1.5 text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-colors rounded-md hover:bg-[#e5e5e5] dark:hover:bg-[#2f2f2f]"
              title={tCommon('copy')}
            >
              {copiedId === message.id ? <Check size={14} className="text-emerald-500 dark:text-emerald-400" /> : <Copy size={14} />}
            </button>
            <button 
              onClick={() => handleFeedback(message.id, 'up')}
              className={cn(
                "p-1.5 transition-colors rounded-md hover:bg-[#e5e5e5] dark:hover:bg-[#2f2f2f]",
                feedback[message.id] === 'up' ? "text-[#1890ff]" : "text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
              )}
              title={tCommon('goodResponse')}
            >
              <ThumbsUp size={14} />
            </button>
            <button 
              onClick={() => handleFeedback(message.id, 'down')}
              className={cn(
                "p-1.5 transition-colors rounded-md hover:bg-[#e5e5e5] dark:hover:bg-[#2f2f2f]",
                feedback[message.id] === 'down' ? "text-red-500 dark:text-red-400" : "text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
              )}
              title={tCommon('badResponse')}
            >
              <ThumbsDown size={14} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
