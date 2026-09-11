import React from 'react';
import { Copy, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
import { cn } from '@sdkwork/agents-pc-commons';

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
    <div className="flex flex-col items-end gap-1.5 group w-full relative">
      <div className="bg-[#f4f4f4] dark:bg-[#2a2a2a] text-gray-900 dark:text-gray-100 rounded-[20px] rounded-tr-[4px] px-5 py-3.5 max-w-[80%] inline-block border border-[#ebebeb] dark:border-[#333] shadow-sm">
        <div className="text-[15px] leading-relaxed whitespace-pre-wrap break-words">
          {message.text}
        </div>
      </div>
      <div className="flex items-center opacity-0 group-hover:opacity-100 transition-opacity h-6 mt-0.5">
        <button 
          onClick={() => handleCopy(message.text, message.id)}
          className="p-1 text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors rounded-md hover:bg-gray-100 dark:hover:bg-[#2f2f2f]"
          title={tCommon('copy')}
        >
          {copiedId === message.id ? <Check size={14} className="text-emerald-500 dark:text-emerald-400" /> : <Copy size={14} />}
        </button>
      </div>
    </div>
  );
};
