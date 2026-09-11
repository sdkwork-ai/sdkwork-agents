import React from 'react';
import { PenTool, LayoutTemplate, Scan, Sparkles } from 'lucide-react';
import { cn } from '../MarkdownRenderer';

// Ratio Icon
const RatioIcon = ({ ratio, active }: { ratio: string, active: boolean }) => {
  let width = 16;
  let height = 16;
  if (ratio === '21:9') { width = 18; height = 8; }
  else if (ratio === '16:9') { width = 16; height = 9; }
  else if (ratio === '4:3') { width = 16; height = 12; }
  else if (ratio === '1:1') { width = 14; height = 14; }
  else if (ratio === '3:4') { width = 12; height = 16; }
  else if (ratio === '9:16') { width = 9; height = 16; }

  return (
    <div className="h-5 flex items-center justify-center mb-1">
      <div 
        style={{ width: `${width}px`, height: `${height}px` }} 
        className={cn("border-[1.5px] rounded-[3px] transition-colors", active ? "border-zinc-900 dark:border-white" : "border-zinc-400 dark:border-zinc-500")}
      />
    </div>
  );
};

interface VideoSettingsDropdownProps {
  videoSettingsMode: 'all_around' | 'first_last' | 'smart_multi';
  setVideoSettingsMode: (mode: 'all_around' | 'first_last' | 'smart_multi') => void;
  videoRatio: string;
  setVideoRatio: (ratio: string) => void;
  videoResolution: string;
  setVideoResolution: (resolution: string) => void;
  videoCount: string;
  setVideoCount: (count: string) => void;
  videoDuration: number;
  setVideoDuration: (duration: number) => void;
  videoSettingsPlacement: 'top' | 'bottom';
  dropdownRef: React.RefObject<HTMLDivElement | null>;
}

export const VideoSettingsDropdown: React.FC<VideoSettingsDropdownProps> = ({
  videoSettingsMode,
  setVideoSettingsMode,
  videoRatio,
  setVideoRatio,
  videoResolution,
  setVideoResolution,
  videoCount,
  setVideoCount,
  videoDuration,
  setVideoDuration,
  videoSettingsPlacement,
  dropdownRef
}) => {
  return (
    <div 
      ref={dropdownRef} 
      className={cn(
        "absolute left-0 sm:left-[80px] w-[calc(100vw-32px)] sm:w-[420px] max-w-[420px] bg-white border border-black/10 rounded-2xl shadow-xl p-4 z-50 animate-in fade-in zoom-in-95 duration-100 flex flex-col gap-5 max-h-[60vh] overflow-y-auto custom-scrollbar dark:bg-[#222222] dark:border-white/10 dark:shadow-2xl",
        videoSettingsPlacement === 'top' ? "bottom-full mb-2" : "top-full mt-2"
      )}
    >
      <div className="flex bg-zinc-100 rounded-xl p-1 dark:bg-[#2a2a2a]">
        <button 
          onClick={() => setVideoSettingsMode('all_around')}
          className={cn("flex-1 py-2 text-[13px] font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5", videoSettingsMode === 'all_around' ? "bg-white text-zinc-900 shadow-sm dark:bg-white/10 dark:text-white" : "text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200")}
        >
          <PenTool size={14} /> 全能参考
        </button>
        <button 
          onClick={() => setVideoSettingsMode('first_last')}
          className={cn("flex-1 py-2 text-[13px] font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5", videoSettingsMode === 'first_last' ? "bg-white text-zinc-900 shadow-sm dark:bg-white/10 dark:text-white" : "text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200")}
        >
          <LayoutTemplate size={14} /> 首尾帧
        </button>
        <button 
          onClick={() => setVideoSettingsMode('smart_multi')}
          className={cn("flex-1 py-2 text-[13px] font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5", videoSettingsMode === 'smart_multi' ? "bg-white text-zinc-900 shadow-sm dark:bg-white/10 dark:text-white" : "text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200")}
        >
          <Scan size={14} /> 智能多帧
        </button>
      </div>

      <div className="flex flex-col gap-3">
        <div className="text-[12px] text-zinc-500 font-medium">选择比例</div>
        <div className="flex justify-between gap-1.5">
          {['21:9', '16:9', '4:3', '1:1', '3:4', '9:16'].map(ratio => (
            <button 
              key={ratio}
              onClick={() => setVideoRatio(ratio)}
              className={cn("flex-1 flex flex-col items-center justify-center py-2.5 rounded-xl transition-colors border", videoRatio === ratio ? "bg-white/10 border-transparent text-white" : "border-transparent text-zinc-400 hover:bg-white/5")}
            >
              <RatioIcon ratio={ratio} active={videoRatio === ratio} />
              <span className="text-[11px] font-medium">{ratio}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="flex flex-col gap-3">
        <div className="text-[12px] text-zinc-500 font-medium">选择分辨率</div>
        <div className="flex gap-2.5">
          <button 
            onClick={() => setVideoResolution('720P')}
            className={cn("flex-1 py-2.5 rounded-xl transition-colors text-[13px] font-medium border", videoResolution === '720P' ? "bg-white/10 border-transparent text-white" : "border-transparent bg-white/5 text-zinc-400 hover:bg-white/10")}
          >
            720P
          </button>
          <button 
            onClick={() => setVideoResolution('1080P')}
            className={cn("flex-1 py-2.5 rounded-xl transition-colors text-[13px] font-medium border", videoResolution === '1080P' ? "bg-white/10 border-transparent text-white" : "border-transparent bg-white/5 text-zinc-400 hover:bg-white/10")}
          >
            1080P <Sparkles size={10} className="inline text-cyan-400 fill-cyan-400 -mt-0.5" />
          </button>
        </div>
      </div>

      <div className="flex flex-col gap-3">
        <div className="text-[12px] text-zinc-500 font-medium">选择生成数量</div>
        <div className="flex gap-2.5">
          {['1', '2', '3', '4'].map(count => (
            <button 
              key={count}
              onClick={() => setVideoCount(count)}
              className={cn("flex-1 py-2.5 rounded-xl transition-colors text-[13px] font-medium border", videoCount === count ? "bg-white/10 border-transparent text-white" : "border-transparent bg-white/5 text-zinc-400 hover:bg-white/10")}
            >
              {count}
            </button>
          ))}
        </div>
      </div>

      <div className="flex flex-col gap-3">
        <div className="text-[12px] text-zinc-500 font-medium">时长</div>
        <div className="flex items-center gap-4">
          <input 
            type="range" 
            min="1" 
            max="10" 
            value={videoDuration}
            onChange={(e) => setVideoDuration(Number(e.target.value))}
            className="flex-1 h-[3px] bg-white/10 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3.5 [&::-webkit-slider-thumb]:h-3.5 [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full hover:[&::-webkit-slider-thumb]:scale-110 transition-all focus:outline-none"
            style={{ 
              backgroundImage: 'linear-gradient(white, white)', 
              backgroundRepeat: 'no-repeat',
              backgroundSize: `${((videoDuration - 1) / 9) * 100}% 100%`
            }}
          />
          <div className="w-16 h-8 bg-white/5 rounded-lg flex items-center justify-center text-sm shrink-0">
            <span className="font-medium text-white">{videoDuration}</span>
            <span className="text-zinc-500 ml-1">s</span>
          </div>
        </div>
      </div>
    </div>
  );
};
