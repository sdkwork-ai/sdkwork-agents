import React, { useEffect, useRef } from 'react';
import { cn } from '../MarkdownRenderer';
import { Link2, Link2Off } from 'lucide-react';

// Ratio Icon specifically designed for all aspect ratios
const ImageRatioIcon = ({ ratio, active }: { ratio: string; active: boolean }) => {
  let width = 16;
  let height = 16;

  if (ratio === '21:9') { width = 18; height = 8; }
  else if (ratio === '16:9') { width = 16; height = 9; }
  else if (ratio === '3:2') { width = 15; height = 10; }
  else if (ratio === '4:3') { width = 15; height = 11; }
  else if (ratio === '1:1') { width = 13; height = 13; }
  else if (ratio === '3:4') { width = 11; height = 15; }
  else if (ratio === '2:3') { width = 10; height = 15; }
  else if (ratio === '9:16') { width = 9; height = 16; }

  if (ratio === 'smart' || ratio === '智能') {
    return (
      <div className="h-5 flex items-center justify-center mb-1">
        <svg 
          width="15" 
          height="15" 
          viewBox="0 0 24 24" 
          fill="none" 
          stroke={active ? "currentColor" : "#71717a"} 
          strokeWidth="2.5" 
          strokeLinecap="round" 
          strokeLinejoin="round" 
          className="transition-colors"
        >
          <path d="M3 7V5a2 2 0 0 1 2-2h2" />
          <path d="M17 3h2a2 2 0 0 1 2 2v2" />
          <path d="M21 17v2a2 2 0 0 1-2 2h-2" />
          <path d="M7 21H5a2 2 0 0 1-2-2v-2" />
          <rect x="8" y="8" width="8" height="8" rx="1" strokeWidth="1.5" />
        </svg>
      </div>
    );
  }

  return (
    <div className="h-5 flex items-center justify-center mb-1">
      <div 
        style={{ width: `${width}px`, height: `${height}px` }} 
        className={cn(
          "border-[1.5px] rounded-[3px] transition-colors", 
          active ? "border-zinc-900 dark:border-white" : "border-zinc-400 dark:border-zinc-500"
        )}
      />
    </div>
  );
};

interface ImageSettingsDropdownProps {
  imageRatio: string;
  setImageRatio: (ratio: string) => void;
  imageResolution: string;
  setImageResolution: (resolution: string) => void;
  imageWidth: number;
  setImageWidth: (w: number) => void;
  imageHeight: number;
  setImageHeight: (h: number) => void;
  imageAspectRatioLocked: boolean;
  setImageAspectRatioLocked: (locked: boolean) => void;
  imageSettingsPlacement: 'top' | 'bottom';
  dropdownRef: React.RefObject<HTMLDivElement | null>;
}

const RATIOS = ['智能', '21:9', '16:9', '3:2', '4:3', '1:1', '3:4', '2:3', '9:16'];

const DIMENSIONS_MAP: Record<string, Record<string, { w: number; h: number }>> = {
  '1K': {
    '智能': { w: 1024, h: 1024 },
    '21:9': { w: 1536, h: 648 },
    '16:9': { w: 1344, h: 756 },
    '3:2': { w: 1200, h: 800 },
    '4:3': { w: 1152, h: 864 },
    '1:1': { w: 1024, h: 1024 },
    '3:4': { w: 864, h: 1152 },
    '2:3': { w: 800, h: 1200 },
    '9:16': { w: 756, h: 1344 },
  },
  '2K': {
    '智能': { w: 1328, h: 1328 },
    '21:9': { w: 1920, h: 816 },
    '16:9': { w: 1728, h: 972 },
    '3:2': { w: 1600, h: 1066 },
    '4:3': { w: 1536, h: 1152 },
    '1:1': { w: 1328, h: 1328 },
    '3:4': { w: 1152, h: 1536 },
    '2:3': { w: 1066, h: 1600 },
    '9:16': { w: 972, h: 1728 },
  },
  '4K': {
    '智能': { w: 2048, h: 2048 },
    '21:9': { w: 3072, h: 1296 },
    '16:9': { w: 2688, h: 1512 },
    '3:2': { w: 2400, h: 1600 },
    '4:3': { w: 2304, h: 1728 },
    '1:1': { w: 2048, h: 2048 },
    '3:4': { w: 1728, h: 2304 },
    '2:3': { w: 1600, h: 2400 },
    '9:16': { w: 1512, h: 2688 },
  }
};

const getRatioFactor = (ratio: string): number => {
  if (ratio === '智能') return 1;
  const parts = ratio.split(':');
  if (parts.length === 2) {
    const w = parseFloat(parts[0]);
    const h = parseFloat(parts[1]);
    if (!isNaN(w) && !isNaN(h) && h !== 0) {
      return w / h;
    }
  }
  return 1;
};

export const ImageSettingsDropdown: React.FC<ImageSettingsDropdownProps> = ({
  imageRatio,
  setImageRatio,
  imageResolution,
  setImageResolution,
  imageWidth,
  setImageWidth,
  imageHeight,
  setImageHeight,
  imageAspectRatioLocked,
  setImageAspectRatioLocked,
  imageSettingsPlacement,
  dropdownRef
}) => {
  // Update dimensions when ratio or resolution changes
  const updateDimensions = (ratio: string, resolution: string) => {
    const map = DIMENSIONS_MAP[resolution] || DIMENSIONS_MAP['2K'];
    const dims = map[ratio] || map['1:1'];
    setImageWidth(dims.w);
    setImageHeight(dims.h);
  };

  const handleRatioSelect = (ratio: string) => {
    setImageRatio(ratio);
    updateDimensions(ratio, imageResolution);
  };

  const handleResolutionSelect = (res: string) => {
    setImageResolution(res);
    updateDimensions(imageRatio, res);
  };

  const handleWidthChange = (val: string) => {
    const num = parseInt(val, 10);
    if (isNaN(num)) {
      setImageWidth(0);
      return;
    }
    setImageWidth(num);
    if (imageAspectRatioLocked) {
      const factor = getRatioFactor(imageRatio);
      setImageHeight(Math.round(num / factor));
    }
  };

  const handleHeightChange = (val: string) => {
    const num = parseInt(val, 10);
    if (isNaN(num)) {
      setImageHeight(0);
      return;
    }
    setImageHeight(num);
    if (imageAspectRatioLocked) {
      const factor = getRatioFactor(imageRatio);
      setImageWidth(Math.round(num * factor));
    }
  };

  return (
    <div 
      ref={dropdownRef} 
      className={cn(
        "absolute left-0 sm:left-[110px] w-[calc(100vw-32px)] sm:w-[410px] max-w-[410px] bg-white border border-black/10 rounded-2xl shadow-xl p-5 z-50 animate-in fade-in zoom-in-95 duration-100 flex flex-col gap-5 max-h-[60vh] overflow-y-auto custom-scrollbar select-none dark:bg-[#1e1e1e] dark:border-white/10 dark:shadow-2xl",
        imageSettingsPlacement === 'top' ? "bottom-full mb-2" : "top-full mt-2"
      )}
    >
      {/* Aspect Ratio Section */}
      <div className="flex flex-col gap-4">
        {/* Recommended Presets Selector */}
        <div className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <div className="text-[12px] text-zinc-400 font-semibold flex items-center gap-1.5">
              <span>常用预设比例</span>
              <span className="bg-cyan-500/10 border border-cyan-400/20 text-cyan-400 text-[8px] font-bold px-1 py-0.5 rounded scale-90 origin-left">
                推荐
              </span>
            </div>
            <span className="text-[10px] text-zinc-500 font-medium">点击快速应用</span>
          </div>
          <div className="grid grid-cols-3 gap-2.5">
            {[
              { ratio: '1:1', label: '1:1 正方形', desc: '社交头像 / 设计插画' },
              { ratio: '16:9', label: '16:9 电脑宽屏', desc: '影视壁纸 / 宣传横幅' },
              { ratio: '9:16', label: '9:16 手机竖屏', desc: '小红书 / 抖音短视频' }
            ].map(preset => (
              <button
                key={preset.ratio}
                onClick={() => handleRatioSelect(preset.ratio)}
                type="button"
                className={cn(
                  "flex flex-col items-center justify-center p-3 rounded-xl transition-all border text-left cursor-pointer select-none",
                  imageRatio === preset.ratio
                    ? "bg-cyan-500/10 border-cyan-400/40 text-cyan-400 shadow-[0_0_12px_rgba(34,211,238,0.05)]"
                    : "border-black/5 bg-zinc-50 text-zinc-500 hover:bg-zinc-100 hover:text-zinc-800 hover:border-black/10 dark:border-white/5 dark:bg-white/[0.02] dark:text-zinc-400 dark:hover:bg-white/5 dark:hover:text-zinc-200 dark:hover:border-white/10"
                )}
              >
                <div className="h-5 flex items-center justify-center mb-1.5">
                  {preset.ratio === '1:1' && (
                    <div className={cn("w-[12px] h-[12px] border-[1.5px] rounded-sm transition-colors", imageRatio === preset.ratio ? "border-cyan-400" : "border-zinc-500")} />
                  )}
                  {preset.ratio === '16:9' && (
                    <div className={cn("w-[18px] h-[10px] border-[1.5px] rounded-sm transition-colors", imageRatio === preset.ratio ? "border-cyan-400" : "border-zinc-500")} />
                  )}
                  {preset.ratio === '9:16' && (
                    <div className={cn("w-[10px] h-[18px] border-[1.5px] rounded-sm transition-colors", imageRatio === preset.ratio ? "border-cyan-400" : "border-zinc-500")} />
                  )}
                </div>
                <span className="font-bold text-[12px] leading-tight text-center">{preset.label}</span>
                <span className="text-[8px] text-zinc-500 mt-1 text-center scale-95 font-medium leading-tight">{preset.desc}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Regular Ratios Grid */}
        <div className="flex flex-col gap-2">
          <div className="text-[12px] text-zinc-500 font-semibold">所有比例</div>
          <div className="grid grid-cols-5 gap-2">
            {RATIOS.map(ratio => (
              <button 
                key={ratio}
                onClick={() => handleRatioSelect(ratio)}
                type="button"
                className={cn(
                  "flex flex-col items-center justify-center py-2.5 rounded-xl transition-all border text-[11px] font-medium cursor-pointer", 
                  imageRatio === ratio 
                    ? "bg-zinc-100 border-zinc-200 text-zinc-900 dark:bg-white/10 dark:border-white/10 dark:text-white" 
                    : "border-transparent text-zinc-500 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-white/5 dark:hover:text-white"
                )}
              >
                <ImageRatioIcon ratio={ratio} active={imageRatio === ratio} />
                <span>{ratio}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Resolution Section */}
      <div className="flex flex-col gap-3">
        <div className="text-[12px] text-zinc-500 font-medium">选择分辨率</div>
        <div className="flex gap-2.5 relative">
          {/* 1K Option with '限免1次' badge */}
          <div className="flex-1 relative flex flex-col pt-2.5">
            <span className="absolute top-0 left-1/2 -translate-x-1/2 bg-cyan-400/20 text-cyan-400 border border-cyan-400/30 px-1.5 py-0.5 rounded-md text-[9px] font-bold scale-90 whitespace-nowrap z-20">
              限免1次
            </span>
            <button 
              onClick={() => handleResolutionSelect('1K')}
              className={cn(
                "w-full py-2.5 rounded-xl transition-all text-[13px] font-medium border cursor-pointer", 
                imageResolution === '1K' 
                  ? "bg-zinc-100 border-zinc-200 text-zinc-900 dark:bg-white/10 dark:border-white/10 dark:text-white" 
                  : "border-transparent bg-zinc-100 text-zinc-500 hover:bg-zinc-200 hover:text-zinc-900 dark:bg-white/5 dark:text-zinc-400 dark:hover:bg-white/10 dark:hover:text-white"
              )}
            >
              标清 1K
            </button>
          </div>

          <div className="flex-1 pt-2.5">
            <button 
              onClick={() => handleResolutionSelect('2K')}
              className={cn(
                "w-full py-2.5 rounded-xl transition-all text-[13px] font-medium border cursor-pointer flex items-center justify-center gap-1", 
                imageResolution === '2K' 
                  ? "bg-zinc-100 border-zinc-200 text-zinc-900 dark:bg-white/10 dark:border-white/10 dark:text-white" 
                  : "border-transparent bg-zinc-100 text-zinc-500 hover:bg-zinc-200 hover:text-zinc-900 dark:bg-white/5 dark:text-zinc-400 dark:hover:bg-white/10 dark:hover:text-white"
              )}
            >
              <span>高清 2K</span>
              <span className="text-cyan-400 text-xs font-semibold">✦</span>
            </button>
          </div>

          <div className="flex-1 pt-2.5">
            <button 
              onClick={() => handleResolutionSelect('4K')}
              className={cn(
                "w-full py-2.5 rounded-xl transition-all text-[13px] font-medium border cursor-pointer flex items-center justify-center gap-1", 
                imageResolution === '4K' 
                  ? "bg-zinc-100 border-zinc-200 text-zinc-900 dark:bg-white/10 dark:border-white/10 dark:text-white" 
                  : "border-transparent bg-zinc-100 text-zinc-500 hover:bg-zinc-200 hover:text-zinc-900 dark:bg-white/5 dark:text-zinc-400 dark:hover:bg-white/10 dark:hover:text-white"
              )}
            >
              <span>超清 4K</span>
              <span className="text-cyan-400 text-xs font-semibold">✦</span>
            </button>
          </div>
        </div>
      </div>

      {/* Manual Dimensions Section */}
      <div className="flex flex-col gap-3">
        <div className="text-[12px] text-zinc-500 font-medium">尺寸</div>
        <div className="flex items-center gap-3">
          {/* Width Input */}
          <div className="flex-1 flex items-center bg-zinc-100 rounded-xl px-3 py-2 border border-zinc-200 focus-within:border-cyan-500/50 transition-all dark:bg-[#252525] dark:border-white/5">
            <span className="text-zinc-400 text-xs font-medium mr-2 select-none dark:text-zinc-500">W</span>
            <input 
              type="text" 
              value={imageWidth || ''}
              onChange={(e) => handleWidthChange(e.target.value)}
              className="w-full bg-transparent border-none outline-none text-right font-mono text-[13px] text-zinc-900 dark:text-white"
            />
          </div>

          {/* Aspect Ratio Lock Button */}
          <button
            onClick={() => setImageAspectRatioLocked(!imageAspectRatioLocked)}
            className={cn(
              "p-2 rounded-lg transition-colors border cursor-pointer",
              imageAspectRatioLocked 
                ? "bg-zinc-100 border-zinc-200 text-cyan-600 hover:text-cyan-700 dark:bg-white/5 dark:border-white/10 dark:text-cyan-400 dark:hover:text-cyan-300" 
                : "bg-transparent border-transparent text-zinc-400 hover:text-zinc-600 dark:text-zinc-500 dark:hover:text-zinc-300"
            )}
            title={imageAspectRatioLocked ? "解除比例锁定" : "锁定比例"}
          >
            {imageAspectRatioLocked ? <Link2 size={16} /> : <Link2Off size={16} />}
          </button>

          {/* Height Input */}
          <div className="flex-1 flex items-center bg-zinc-100 rounded-xl px-3 py-2 border border-zinc-200 focus-within:border-cyan-500/50 transition-all dark:bg-[#252525] dark:border-white/5">
            <span className="text-zinc-400 text-xs font-medium mr-2 select-none dark:text-zinc-500">H</span>
            <input 
              type="text" 
              value={imageHeight || ''}
              onChange={(e) => handleHeightChange(e.target.value)}
              className="w-full bg-transparent border-none outline-none text-right font-mono text-[13px] text-zinc-900 dark:text-white"
            />
          </div>

          <span className="text-zinc-500 text-xs font-semibold font-mono pr-1 select-none">PX</span>
        </div>
      </div>
    </div>
  );
};
