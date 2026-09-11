import React from 'react';
import { Copy, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
import { MarkdownRenderer } from '@sdkwork/agents-pc-commons';

interface UserMessageItemProps {
  message: ChatMessage;
  copiedId: string | null;
  handleCopy: (text: string, id: string) => void;
}

export const UserMessageItem: React.FC<UserMessageItemProps> = ({
  message,
  copiedId,
  handleCopy,
}) => {
  const { t: tCommon } = useTranslation('common');

  return (
    <div className="group relative flex w-fit max-w-full flex-col items-end gap-1">
      <div className="chat-markdown-user rounded-[20px] rounded-tr-[4px] border border-[#e8e8e8] bg-[#f4f4f4] px-4 py-3 text-[15px] leading-relaxed text-gray-900 shadow-sm dark:border-[#3a3a3a] dark:bg-[#2a2a2a] dark:text-gray-100 sm:px-5 sm:py-3.5">
        <MarkdownRenderer
          content={message.text}
          streaming={false}
        />
      </div>
      <div className="mt-0.5 flex h-6 items-center opacity-0 transition-opacity group-hover:opacity-100">
        <button
          type="button"
          onClick={() => handleCopy(message.text, message.id)}
          className="rounded-md p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:text-gray-500 dark:hover:bg-[#2f2f2f] dark:hover:text-gray-300"
          title={tCommon('copy')}
        >
          {copiedId === message.id ? (
            <Check size={14} className="text-emerald-500 dark:text-emerald-400" />
          ) : (
            <Copy size={14} />
          )}
        </button>
      </div>
    </div>
  );
};
