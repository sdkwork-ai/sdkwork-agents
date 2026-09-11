import React, { useState } from 'react';
import { X, Clock, Calendar, ChevronDown, Flame, Award, ThumbsUp, Play } from 'lucide-react';
import type { Activity, ActivityWork } from '../types';
import { cn } from '@sdkwork/agents-pc-commons';

interface ActivityDetailViewProps {
  activity: Activity;
  onClose: () => void;
  onPlayVideo: (work: ActivityWork) => void;
}

export const ActivityDetailView: React.FC<ActivityDetailViewProps> = ({ activity, onClose, onPlayVideo }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [activeSubTab, setActiveSubTab] = useState<'全部' | '获奖作品'>('全部');
  const [sortBy, setSortBy] = useState<'time' | 'likes'>('likes');

  // Sort works
  const sortedWorks = [...activity.works].sort((a, b) => {
    if (sortBy === 'likes') return b.likes - a.likes;
    return b.id.localeCompare(a.id); // Dummy time sort
  });

  // Filter works (for dummy purposes, award tab only shows top ones)
  const filteredWorks = activeSubTab === '获奖作品' 
    ? sortedWorks.filter(w => w.likes > 50) 
    : sortedWorks;

  return (
    <div className="w-full min-h-screen bg-[#141414] text-white animate-in fade-in slide-in-from-right-4 duration-300">
      {/* Top Banner and Navigation Bar */}
      <div className="max-w-[1200px] mx-auto px-6 py-8 relative">
        {/* Close button inside page context */}
        <button 
          onClick={onClose}
          className="absolute top-8 right-6 p-2 bg-white/5 hover:bg-white/10 text-zinc-400 hover:text-white rounded-full transition-all border border-white/5"
          title="返回列表"
        >
          <X size={20} />
        </button>

        {/* Header Metadata */}
        <div className="mb-6 max-w-[85%]">
          <h1 className="text-[24px] font-bold tracking-tight text-white mb-4">
            {activity.title}
          </h1>
          <div className="flex flex-wrap items-center gap-4 text-xs text-zinc-400">
            <span className="flex items-center gap-1.5 bg-yellow-500/10 text-yellow-500 px-2.5 py-1 rounded-md font-medium border border-yellow-500/10">
              <Clock size={12} />
              {activity.status}
            </span>
            <span className="flex items-center gap-1.5 text-zinc-300">
              <Calendar size={12} className="text-zinc-500" />
              活动时间: {activity.timeRange}
            </span>
          </div>
        </div>

        {/* Action Buttons Row */}
        <div className="flex items-center gap-3 mb-8">
          <button className="px-5 py-2 rounded-xl bg-cyan-500/10 border border-cyan-500 text-cyan-400 hover:bg-cyan-500/20 text-[13px] font-semibold transition-colors">
            去创作
          </button>
          <button className="px-5 py-2 rounded-xl bg-white text-black hover:bg-zinc-200 dark:bg-zinc-50 dark:text-black dark:hover:bg-zinc-300 text-[13px] font-semibold transition-all">
            立即投稿
          </button>
        </div>

        {/* Big Beautiful Cover Banner */}
        <div className="w-full aspect-[21/9] rounded-2xl overflow-hidden border border-white/5 shadow-2xl mb-8 relative bg-black">
          <img 
            src={activity.banner} 
            alt={activity.title} 
            className="w-full h-full object-cover opacity-80"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-black/60 to-transparent" />
        </div>

        {/* Two-column Layout: Content Left, Side Timeline Right */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start mb-12">
          {/* Main Info */}
          <div className="lg:col-span-2 space-y-8">
            {/* Background Section */}
            <div className="bg-[#1e1e1e] rounded-2xl p-6 border border-white/5">
              <h2 className="text-[15px] font-bold text-zinc-200 mb-4 flex items-center gap-2">
                <span className="w-1.5 h-3.5 bg-cyan-500 rounded-full"></span>
                活动背景
              </h2>
              <div className={cn(
                "text-[13px] text-zinc-400 leading-relaxed whitespace-pre-line transition-all duration-300",
                !isExpanded ? "line-clamp-4" : ""
              )}>
                {activity.background}
              </div>
              <div className="mt-4 flex justify-center border-t border-white/5 pt-3">
                <button 
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="text-zinc-500 hover:text-zinc-300 text-[12px] font-medium transition-colors flex items-center gap-1"
                >
                  {isExpanded ? '收起' : '展开'}
                  <ChevronDown size={14} className={cn("transition-transform duration-300", isExpanded ? "rotate-180" : "")} />
                </button>
              </div>
            </div>

            {/* Submitted Works Header & Grid */}
            <div>
              <div className="flex items-center justify-between border-b border-white/5 pb-4 mb-6">
                <div className="flex items-center gap-1 bg-[#1e1e1e] p-1 rounded-lg">
                  {(['全部', '获奖作品'] as const).map(tab => (
                    <button
                      key={tab}
                      onClick={() => setActiveSubTab(tab)}
                      className={cn(
                        "px-4 py-1.5 rounded-md text-[13px] font-medium transition-colors",
                        activeSubTab === tab ? "bg-white/10 text-white" : "text-zinc-400 hover:text-zinc-200"
                      )}
                    >
                      {tab}
                    </button>
                  ))}
                </div>

                <div className="flex items-center gap-3 text-xs text-zinc-400">
                  <button 
                    onClick={() => setSortBy('time')}
                    className={cn("hover:text-white transition-colors", sortBy === 'time' ? "text-cyan-400 font-semibold" : "")}
                  >
                    按时间
                  </button>
                  <span className="text-zinc-400 dark:text-zinc-700">|</span>
                  <button 
                    onClick={() => setSortBy('likes')}
                    className={cn("hover:text-white transition-colors", sortBy === 'likes' ? "text-cyan-400 font-semibold" : "")}
                  >
                    按热度
                  </button>
                </div>
              </div>

              {/* Works Grid */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {filteredWorks.map(work => (
                  <div 
                    key={work.id}
                    className="bg-[#1e1e1e] border border-white/5 rounded-xl overflow-hidden group cursor-pointer hover:border-white/10 transition-all flex flex-col"
                    onClick={() => onPlayVideo(work)}
                  >
                    {/* Media Cover */}
                    <div className="relative aspect-video w-full bg-[#141414] overflow-hidden">
                      <img 
                        src={work.cover} 
                        alt={work.title} 
                        className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                      />
                      
                      {/* Play Button Icon */}
                      <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center">
                        <div className="w-10 h-10 rounded-full bg-cyan-500 text-black flex items-center justify-center shadow-lg transform scale-90 group-hover:scale-100 transition-transform duration-300">
                          <Play size={18} fill="currentColor" className="ml-0.5" />
                        </div>
                      </div>

                      {/* Header Badge */}
                      <div className="absolute top-2 left-2 bg-black/60 backdrop-blur-md text-[10px] text-zinc-300 px-1.5 py-0.5 rounded font-mono">
                        {work.duration}
                      </div>

                      {/* Footer Badge (Likes) */}
                      <div className="absolute bottom-2 right-2 bg-black/60 backdrop-blur-md text-[10px] text-red-400 px-1.5 py-0.5 rounded flex items-center gap-1">
                        <ThumbsUp size={10} fill="currentColor" />
                        {work.likes}
                      </div>
                    </div>

                    {/* Meta info */}
                    <div className="p-3.5 flex flex-col flex-1 justify-between">
                      <h4 className="text-[13px] font-medium text-zinc-100 line-clamp-1 group-hover:text-cyan-400 transition-colors mb-2">
                        {work.title}
                      </h4>
                      <div className="flex items-center gap-2">
                        <img src={work.avatar} alt={work.author} className="w-4 h-4 rounded-full object-cover" />
                        <span className="text-[11px] text-zinc-400 truncate">{work.author}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Side Timeline */}
          <div className="space-y-6">
            <div className="bg-[#1e1e1e] rounded-2xl p-6 border border-white/5">
              <h3 className="text-[14px] font-bold text-zinc-200 mb-5 flex items-center gap-2">
                <span className="w-1.5 h-3.5 bg-yellow-500 rounded-full"></span>
                征集时间
              </h3>
              
              <div className="relative border-l border-zinc-800 pl-5 ml-2.5 space-y-6 text-xs text-zinc-400">
                <div className="relative">
                  <div className="absolute -left-[29px] top-0.5 w-3 h-3 rounded-full bg-cyan-500 border border-[#1e1e1e]" />
                  <div className="font-semibold text-zinc-200">活动启动</div>
                  <div className="text-[11px] text-zinc-500 mt-1">2026年06月30日</div>
                </div>

                <div className="relative">
                  <div className="absolute -left-[29px] top-0.5 w-3 h-3 rounded-full bg-yellow-500 border border-[#1e1e1e]" />
                  <div className="font-semibold text-zinc-200">截稿投递</div>
                  <div className="text-[11px] text-zinc-500 mt-1">2026年07月20日 23:59:59</div>
                </div>

                <div className="relative">
                  <div className="absolute -left-[29px] top-0.5 w-3 h-3 rounded-full bg-zinc-700 border border-[#1e1e1e]" />
                  <div className="font-semibold text-zinc-500">评审与展映</div>
                  <div className="text-[11px] text-zinc-600 mt-1">2026年07月21日 - 07月31日</div>
                </div>
              </div>
            </div>
            
            <div className="bg-[#1e1e1e] rounded-2xl p-6 border border-white/5">
              <h3 className="text-[14px] font-bold text-zinc-200 mb-4">
                作品要求
              </h3>
              <ul className="text-xs text-zinc-400 space-y-2.5 list-disc list-inside">
                <li>必须使用 AI 生成工具（如即梦 AI 等）辅助制作</li>
                <li>短片总时长须在 30 秒至 15 分钟之间</li>
                <li>内容积极向上，不侵犯第三方肖像权和著作权</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
