import React from 'react';
import { ChevronDown, SquarePen } from 'lucide-react';

interface CreativeToolbarProps {
  title: string;
}

export const CreativeToolbar: React.FC<CreativeToolbarProps> = ({ title }) => {
  return (
    <div className="h-14 border-b border-black/5 flex items-center px-6 shrink-0 justify-between dark:border-white/5">
      <div className="text-[14px] font-medium text-zinc-700 truncate max-w-[300px] dark:text-zinc-300">
        {title}
      </div>
      
      {/* Filter controls matching screenshot */}
      <div className="flex items-center gap-4 text-[12px] text-zinc-500 font-medium select-none dark:text-zinc-400">
        <div className="flex items-center gap-1 hover:text-zinc-800 dark:hover:text-zinc-200 cursor-pointer py-1 px-2 rounded hover:bg-black/5 dark:hover:bg-[#2a2a2a] transition-colors">
          <span>时间</span>
          <ChevronDown size={13} className="text-zinc-400 dark:text-zinc-500" />
        </div>
        <div className="flex items-center gap-1 hover:text-zinc-800 dark:hover:text-zinc-200 cursor-pointer py-1 px-2 rounded hover:bg-black/5 dark:hover:bg-[#2a2a2a] transition-colors">
          <span>生成模式</span>
          <ChevronDown size={13} className="text-zinc-400 dark:text-zinc-500" />
        </div>
        <div className="flex items-center gap-1 hover:text-zinc-800 dark:hover:text-zinc-200 cursor-pointer py-1 px-2 rounded hover:bg-black/5 dark:hover:bg-[#2a2a2a] transition-colors">
          <span>操作类型</span>
          <ChevronDown size={13} className="text-zinc-400 dark:text-zinc-500" />
        </div>
        <div className="w-[1px] h-3.5 bg-black/10 dark:bg-[#333333]" />
        <div className="flex items-center gap-1.5 hover:text-cyan-600 text-cyan-500 cursor-pointer py-1 px-2 rounded hover:bg-black/5 dark:hover:text-cyan-300 dark:text-cyan-400 dark:hover:bg-[#2a2a2a] transition-colors">
          <SquarePen size={12} />
          <span>自资库</span>
        </div>
      </div>
    </div>
  );
};
