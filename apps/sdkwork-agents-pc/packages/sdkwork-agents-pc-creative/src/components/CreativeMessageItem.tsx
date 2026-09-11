import React from 'react';
import { Sparkles, RotateCcw, LayoutGrid, Trash2 } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { CreativeMessage } from '../types';
import { GridLayout } from './GridLayout';
import { MasonryLayout } from './MasonryLayout';
import { CarouselLayout } from './CarouselLayout';

interface CreativeMessageItemProps {
  message: CreativeMessage;
  layout: 'grid' | 'masonry' | 'carousel';
  carouselIndex: number;
  onPreviewImage: (message: CreativeMessage, index: number) => void;
  onSetCarouselIndex: (index: number) => void;
  onToggleLayout: () => void;
  onSend: (text: string, mode: string) => void;
  onDelete: () => void;
}

export const CreativeMessageItem: React.FC<CreativeMessageItemProps> = ({
  message: m,
  layout,
  carouselIndex: activeIdx,
  onPreviewImage,
  onSetCarouselIndex,
  onToggleLayout,
  onSend,
  onDelete
}) => {
  return (
    <div className="w-full flex flex-col gap-3 group border-b border-black/[0.03] pb-8 last:border-b-0 last:pb-0 animate-in fade-in duration-300 dark:border-white/[0.03]">
      
      {/* Prompt Title & Model Info */}
      <div className="flex items-center justify-between gap-4 w-full select-none">
        <div className="text-[14px] font-medium text-zinc-800 leading-relaxed dark:text-zinc-200">
          {m.text}
        </div>
        <div className="flex items-center gap-1.5 text-zinc-500 text-[11px] font-medium shrink-0">
          <span>{m.modelInfo || '图片 5.0 Lite | 1:1 | 2K'}</span>
          <span className="cursor-pointer hover:text-zinc-300 transition-colors flex items-center gap-0.5">
            详细信息 ⓘ
          </span>
        </div>
      </div>

      {/* Rendering stage styles */}
      {m.stage === 'thinking' || m.stage === 'loading' ? (
        /* Progress thinking / loading card */
        <div className="w-full aspect-[2.39/1] min-h-[220px] bg-zinc-100 border border-black/5 rounded-2xl flex flex-col items-center justify-center relative overflow-hidden dark:bg-white/[0.01] dark:border-white/5">
          {m.stage === 'thinking' ? (
            <div className="flex flex-col items-center gap-2">
              <div className="w-6 h-6 rounded-full border-2 border-cyan-400 border-t-transparent animate-spin shrink-0" />
              <span className="text-zinc-400 text-xs font-medium animate-pulse">认真思考中...</span>
            </div>
          ) : (
            <div className="w-full h-full flex flex-col items-center justify-center p-4">
              {/* Progress Badge */}
              <div className="absolute top-4 left-4 bg-black/60 backdrop-blur-md text-white text-[10px] font-bold px-2 py-0.5 rounded-md flex items-center gap-1 z-10">
                <div className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse" />
                <span>{m.progress}%</span>
              </div>
              
              {/* Scanning bar animation */}
              <div className="absolute inset-x-0 h-0.5 bg-cyan-500/50 shadow-[0_0_10px_#06b6d4] scanner-bar" />
              
              <Sparkles size={20} className="text-zinc-600 animate-pulse mb-2" />
              <span className="text-zinc-500 text-[11px] font-medium">生成画面中...</span>
            </div>
          )}
        </div>
      ) : (
        /* Completed Stage: Dynamic Layout Modes */
        <div className="w-full">
          {layout === 'masonry' ? (
            <MasonryLayout message={m} onPreviewImage={onPreviewImage} />
          ) : layout === 'carousel' ? (
            <CarouselLayout message={m} activeIdx={activeIdx} onSetCarouselIndex={onSetCarouselIndex} onPreviewImage={onPreviewImage} />
          ) : (
            <GridLayout message={m} onPreviewImage={onPreviewImage} />
          )}

          {/* Action buttons row below images */}
          <div className="flex items-center gap-2 mt-4 select-none relative">
            <button 
              onClick={() => onSend(m.text || '', m.mode || 'agent')}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-black/10 text-zinc-600 hover:text-zinc-900 hover:bg-black/5 transition-colors text-xs font-medium cursor-pointer dark:border-white/10 dark:text-zinc-300 dark:hover:text-white dark:hover:bg-white/5"
            >
              <RotateCcw size={13} />
              重新生成
            </button>
            <button 
              onClick={onToggleLayout}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-black/10 text-zinc-600 hover:text-zinc-900 hover:bg-black/5 transition-colors text-xs font-medium cursor-pointer dark:border-white/10 dark:text-zinc-300 dark:hover:text-white dark:hover:bg-white/5"
            >
              <LayoutGrid size={13} />
              切换排版
            </button>
            <button
              onClick={onDelete}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-black/10 text-rose-600 hover:text-rose-700 hover:bg-rose-50 transition-colors text-xs font-medium cursor-pointer dark:border-white/10 dark:text-rose-400 dark:hover:text-rose-300 dark:hover:bg-rose-500/5"
            >
              <Trash2 size={13} />
              删除记录
            </button>
          </div>

          {/* Creative Suggestions (Inspired by Octo / MJ style prompt hints) */}
          {m.suggestions && m.suggestions.length > 0 && (
            <div className="mt-4 flex flex-wrap gap-2">
              <span className="text-zinc-500 text-[11px] font-medium py-1.5 mr-1 select-none">试试：</span>
              {m.suggestions.map((sug, idx) => (
                <button
                  key={idx}
                  onClick={() => onSend(sug, 'agent')}
                  className="bg-zinc-100 hover:bg-zinc-200 border border-black/5 rounded-full px-3 py-1.5 text-[11px] text-zinc-600 hover:text-zinc-900 transition-colors flex items-center gap-1.5 group cursor-pointer dark:bg-white/[0.03] dark:hover:bg-white/[0.08] dark:border-white/5 dark:text-zinc-300 dark:hover:text-white"
                >
                  <Sparkles size={10} className="text-cyan-500 group-hover:text-cyan-400" />
                  {sug}
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
