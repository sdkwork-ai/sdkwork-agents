import React, { useState } from 'react';
import { 
  Play, 
  CornerDownLeft, 
  Trash2, 
  Search, 
  Wand2, 
  Image as ImageIcon, 
  PlaySquare, 
  Music, 
  AudioLines, 
  Smile, 
  Accessibility, 
  History,
  Info
} from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

export interface PromptHistoryItem {
  id: string;
  prompt: string;
  mode: string; // agent, image, video, music, voice, digital_human, action
  timestamp: number;
  settings?: {
    model?: string;
    ratio?: string;
    resolution?: string;
    duration?: number;
    videoMode?: string;
    count?: number;
    style?: string;
    [key: string]: any;
  };
}

interface CreativeHistoryLogProps {
  historyLogs: PromptHistoryItem[];
  onReRun: (item: PromptHistoryItem) => void;
  onLoadConfig: (item: PromptHistoryItem) => void;
  onDeleteLog: (id: string) => void;
  onClearLogs: () => void;
}

const MODE_META: Record<string, { label: string; icon: any; color: string; bg: string }> = {
  agent: { label: 'Agent 模式', icon: Wand2, color: 'text-cyan-400', bg: 'bg-cyan-500/10 border-cyan-500/20' },
  image: { label: '图片生成', icon: ImageIcon, color: 'text-yellow-400', bg: 'bg-yellow-500/10 border-yellow-500/20' },
  video: { label: '视频生成', icon: PlaySquare, color: 'text-indigo-400', bg: 'bg-indigo-500/10 border-indigo-500/20' },
  music: { label: '音乐生成', icon: Music, color: 'text-emerald-400', bg: 'bg-emerald-500/10 border-emerald-500/20' },
  voice: { label: '配音生成', icon: AudioLines, color: 'text-fuchsia-400', bg: 'bg-fuchsia-500/10 border-fuchsia-500/20' },
  digital_human: { label: '数字人', icon: Smile, color: 'text-sky-400', bg: 'bg-sky-500/10 border-sky-500/20' },
  action: { label: '动作模仿', icon: Accessibility, color: 'text-orange-400', bg: 'bg-orange-500/10 border-orange-500/20' }
};

export const CreativeHistoryLog: React.FC<CreativeHistoryLogProps> = ({
  historyLogs,
  onReRun,
  onLoadConfig,
  onDeleteLog,
  onClearLogs
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeFilter, setActiveFilter] = useState<string>('all');

  const filteredLogs = historyLogs.filter(log => {
    const matchesSearch = log.prompt.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesType = activeFilter === 'all' || log.mode === activeFilter;
    return matchesSearch && matchesType;
  });

  const getFormatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return `${date.getMonth() + 1}-${date.getDate()} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  };

  return (
    <div className="flex-1 flex flex-col h-full overflow-hidden">
      {/* Search Bar */}
      <div className="px-3 pt-2 pb-1.5 shrink-0">
        <div className="relative flex items-center bg-zinc-100 border border-zinc-200 rounded-xl px-2.5 py-1.5 focus-within:border-cyan-500/50 focus-within:bg-white transition-all dark:bg-white/5 dark:border-white/10 dark:focus-within:bg-white/10">
          <Search size={14} className="text-zinc-400 shrink-0 dark:text-zinc-500" />
          <input
            type="text"
            placeholder="搜索历史 prompt..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-transparent border-0 outline-none ring-0 text-xs text-zinc-800 placeholder-zinc-400 focus:outline-none focus:ring-0 ml-1.5 dark:text-zinc-200 dark:placeholder-zinc-500"
          />
        </div>
      </div>

      {/* Filter Tabs Scrollable Row */}
      <div className="px-3 py-1 flex gap-1 overflow-x-auto shrink-0 custom-scrollbar-hide select-none whitespace-nowrap">
        <button
          onClick={() => setActiveFilter('all')}
          className={cn(
            "px-2.5 py-1 rounded-lg text-[11px] font-medium transition-all cursor-pointer",
            activeFilter === 'all' 
              ? "bg-cyan-500/10 text-cyan-600 border border-cyan-500/25 dark:text-cyan-400" 
              : "text-zinc-500 hover:text-zinc-800 bg-zinc-100 border border-transparent dark:text-zinc-400 dark:hover:text-zinc-200 dark:bg-white/5"
          )}
        >
          全部
        </button>
        {Object.entries(MODE_META).map(([key, meta]) => (
          <button
            key={key}
            onClick={() => setActiveFilter(key)}
            className={cn(
              "px-2.5 py-1 rounded-lg text-[11px] font-medium transition-all cursor-pointer flex items-center gap-1 border border-transparent",
              activeFilter === key 
                ? "bg-cyan-500/10 text-cyan-600 border-cyan-500/25 dark:text-cyan-400" 
                : "text-zinc-500 hover:text-zinc-800 bg-zinc-100 dark:text-zinc-400 dark:hover:text-zinc-200 dark:bg-white/5"
            )}
          >
            <meta.icon size={10} className={meta.color} />
            <span>{meta.label.replace('生成', '')}</span>
          </button>
        ))}
      </div>

      {/* Header Panel Actions */}
      <div className="px-3 py-2 flex items-center justify-between text-[11px] font-semibold text-zinc-400 uppercase tracking-wider select-none shrink-0 border-b border-black/5 mb-1 dark:text-zinc-500 dark:border-white/5">
        <span>生成记录 ({filteredLogs.length})</span>
        {historyLogs.length > 0 && (
          <button
            onClick={() => {
              onClearLogs();
            }}
            className="text-rose-400/80 hover:text-rose-400 transition-colors lowercase font-normal cursor-pointer text-[10px]"
          >
            清空历史
          </button>
        )}
      </div>

      {/* History List */}
      <div className="flex-1 overflow-y-auto px-3 py-1 space-y-2.5 custom-scrollbar">
        {filteredLogs.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-10 px-4 text-center text-zinc-400 dark:text-zinc-500">
            <History size={24} className="text-zinc-300 mb-2 dark:text-zinc-600" />
            <p className="text-[12px]">暂无历史记录</p>
            <p className="text-[10px] text-zinc-400 mt-1 dark:text-zinc-600">开始创作并在输入框发送生成 Prompt 时，配置将自动同步记录于此</p>
          </div>
        ) : (
          filteredLogs.map((log) => {
            const meta = MODE_META[log.mode] || { label: '智能生成', icon: Wand2, color: 'text-zinc-500', bg: 'bg-zinc-100 border-zinc-200 dark:text-zinc-400 dark:bg-white/5 dark:border-white/10' };
            const IconComp = meta.icon;

            return (
              <div
                key={log.id}
                className="group relative flex flex-col p-3 rounded-xl bg-zinc-50 border border-black/5 hover:border-black/10 hover:bg-zinc-100 transition-all dark:bg-white/[0.02] dark:border-white/5 dark:hover:border-white/10 dark:hover:bg-white/[0.04]"
              >
                {/* Mode and Time Header */}
                <div className="flex items-center justify-between mb-1.5">
                  <div className={cn("flex items-center gap-1.5 px-2 py-0.5 rounded text-[9px] font-semibold tracking-wider border", meta.bg)}>
                    <IconComp size={10} className={meta.color} />
                    <span className="text-zinc-600 uppercase dark:text-zinc-300">{meta.label}</span>
                  </div>
                  <span className="text-[10px] text-zinc-400 font-mono dark:text-zinc-600">{getFormatTime(log.timestamp)}</span>
                </div>

                {/* Prompt Text */}
                <div className="text-xs text-zinc-700 font-medium break-all line-clamp-3 leading-relaxed mb-2 dark:text-zinc-300" title={log.prompt}>
                  {log.prompt}
                </div>

                {/* Associated Output Parameters */}
                {log.settings && Object.keys(log.settings).length > 0 && (
                  <div className="flex flex-wrap gap-1 mt-1">
                    {log.settings.model && (
                      <span className="px-1.5 py-0.5 rounded bg-zinc-100 text-[9px] text-zinc-500 border border-black/[0.03] font-mono dark:bg-white/5 dark:text-zinc-400 dark:border-white/[0.03]">
                        {log.settings.model}
                      </span>
                    )}
                    {log.settings.ratio && (
                      <span className="px-1.5 py-0.5 rounded bg-zinc-100 text-[9px] text-zinc-500 border border-black/[0.03] font-mono dark:bg-white/5 dark:text-zinc-400 dark:border-white/[0.03]">
                        比例: {log.settings.ratio}
                      </span>
                    )}
                    {log.settings.style && (
                      <span className="px-1.5 py-0.5 rounded bg-zinc-100 text-[9px] text-zinc-500 border border-black/[0.03] font-mono dark:bg-white/5 dark:text-zinc-400 dark:border-white/[0.03]">
                        风格: {log.settings.style}
                      </span>
                    )}
                    {log.settings.duration && (
                      <span className="px-1.5 py-0.5 rounded bg-zinc-100 text-[9px] text-zinc-500 border border-black/[0.03] font-mono dark:bg-white/5 dark:text-zinc-400 dark:border-white/[0.03]">
                        时长: {log.settings.duration}s
                      </span>
                    )}
                    {log.settings.count && (
                      <span className="px-1.5 py-0.5 rounded bg-zinc-100 text-[9px] text-zinc-500 border border-black/[0.03] font-mono dark:bg-white/5 dark:text-zinc-400 dark:border-white/[0.03]">
                        数量: {log.settings.count}
                      </span>
                    )}
                    {log.settings.videoMode && (
                      <span className="px-1.5 py-0.5 rounded bg-zinc-100 text-[9px] text-zinc-500 border border-black/[0.03] font-mono dark:bg-white/5 dark:text-zinc-400 dark:border-white/[0.03]">
                        模式: {log.settings.videoMode}
                      </span>
                    )}
                  </div>
                )}

                {/* Hover Control Overlay */}
                <div className="absolute top-2 right-2 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity bg-white/95 backdrop-blur-md p-1 rounded-lg border border-black/10 shadow-lg dark:bg-[#141414]/90 dark:border-white/10">
                  <button
                    onClick={() => onLoadConfig(log)}
                    title="载入配置并填入输入框"
                    className="p-1 rounded hover:bg-white/10 text-cyan-400 hover:text-cyan-300 transition-all cursor-pointer"
                  >
                    <CornerDownLeft size={13} />
                  </button>
                  <button
                    onClick={() => onReRun(log)}
                    title="立即重新运行配置"
                    className="p-1 rounded hover:bg-emerald-500/20 text-emerald-400 hover:text-emerald-300 transition-all cursor-pointer"
                  >
                    <Play size={13} fill="currentColor" />
                  </button>
                  <button
                    onClick={() => onDeleteLog(log.id)}
                    title="删除此条记录"
                    className="p-1 rounded hover:bg-rose-500/20 text-rose-400 hover:text-rose-300 transition-all cursor-pointer"
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
