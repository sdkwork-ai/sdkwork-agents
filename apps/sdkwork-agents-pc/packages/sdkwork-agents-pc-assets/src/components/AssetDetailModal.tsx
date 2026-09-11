import React, { useEffect, useState } from 'react';
import { X, Star, MoreHorizontal, Info, ChevronUp, ChevronDown, Download, Play, Edit3, Image, Sparkles, Move, Compass, Crop, Wand2, Scissors, Video, HelpCircle } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

export interface AssetItem {
  id: string;
  imageUrl: string;
  mediaUrl?: string;
  type: 'image' | 'video' | 'audio' | 'document';
  prompt: string;
  model: string;
  aspectRatio: string;
  resolution: string;
  thumbnails: string[];
}

interface AssetDetailModalProps {
  isOpen: boolean;
  onClose: () => void;
  currentItem: AssetItem | null;
  onPrev: () => void;
  onNext: () => void;
  hasPrev: boolean;
  hasNext: boolean;
}

export const AssetDetailModal: React.FC<AssetDetailModalProps> = ({
  isOpen,
  onClose,
  currentItem,
  onPrev,
  onNext,
  hasPrev,
  hasNext
}) => {
  const [imageAspect, setImageAspect] = useState<number | null>(null);
  const [activeThumbnailIndex, setActiveThumbnailIndex] = useState<number>(0);
  const [isFavorite, setIsFavorite] = useState<boolean>(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      if (e.key === 'ArrowUp') onPrev();
      if (e.key === 'ArrowDown') onNext();
    };

    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'hidden';
      setImageAspect(null);
      setActiveThumbnailIndex(0);
    }

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'unset';
    };
  }, [isOpen, onClose, currentItem, onPrev, onNext]);

  if (!isOpen || !currentItem) return null;

  const handleImageLoad = (e: React.SyntheticEvent<HTMLImageElement>) => {
    const img = e.currentTarget;
    if (img.naturalWidth && img.naturalHeight) {
      setImageAspect(img.naturalWidth / img.naturalHeight);
    }
  };

  const currentDisplayUrl = currentItem.thumbnails[activeThumbnailIndex]
    || currentItem.mediaUrl
    || currentItem.imageUrl;

  return (
    <div id="asset-detail-modal-root" className="fixed inset-0 z-50 flex bg-zinc-100 dark:bg-[#0a0a0c] animate-in fade-in duration-200">
      {/* Top Close Button on the left side */}
      <button 
        id="close-preview-btn"
        onClick={onClose}
        className="absolute top-6 left-6 p-2.5 bg-white/90 hover:bg-white dark:bg-zinc-900/60 dark:hover:bg-zinc-800/80 rounded-lg text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white transition-colors z-[60] border border-black/10 dark:border-white/5"
      >
        <X size={18} />
      </button>

      {/* Main Container split into Left (Image) and Right (Sidebar) */}
      <div className="flex w-full h-full relative overflow-hidden">
        
        {/* Left Side - Responsive Visual Viewport */}
        <div className="flex-1 bg-black/50 p-6 md:p-12 flex items-center justify-center relative group h-full select-none">
          
          {/* Outer adaptive frame */}
          <div 
            id="adaptive-preview-frame"
            className="relative rounded-2xl overflow-hidden shadow-2xl bg-zinc-200 border border-black/5 dark:bg-zinc-950 dark:border-white/5 flex items-center justify-center transition-all duration-300"
            style={imageAspect ? {
              aspectRatio: imageAspect,
              maxWidth: '90%',
              maxHeight: '85vh',
              width: imageAspect > 1 ? 'min(900px, 100%)' : 'auto',
              height: imageAspect > 1 ? 'auto' : '85vh'
            } : {
              maxWidth: '90%',
              maxHeight: '85vh'
            }}
          >
            {/* AI Generated tag overlay top-left */}
            <div className="absolute top-4 left-4 z-10 px-3 py-1.5 rounded-lg bg-zinc-900/60 backdrop-blur-md border border-white/10 text-white/90 text-xs font-semibold tracking-wide flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
              AI 生成
            </div>

            {/* Main Visual Asset (supports Image / Video preview) */}
            {currentItem.type === 'video' ? (
              <video
                src={currentDisplayUrl}
                className="w-full h-full object-contain rounded-2xl"
                controls
                autoPlay
                loop
                muted
              />
            ) : currentItem.type === 'audio' ? (
              <audio src={currentDisplayUrl} className="w-[80%]" controls autoPlay />
            ) : currentItem.type === 'document' ? (
              <iframe
                src={currentDisplayUrl}
                title={currentItem.prompt}
                className="h-[80vh] w-[70vw] max-w-full rounded-2xl bg-white"
              />
            ) : (
              <img 
                src={currentDisplayUrl} 
                alt={currentItem.prompt} 
                onLoad={handleImageLoad}
                className="w-full h-full object-contain rounded-2xl animate-in fade-in duration-300"
                referrerPolicy="no-referrer"
              />
            )}
          </div>

          {/* Vertical Navigation Controls on the right edge of the viewport */}
          <div className="absolute right-8 top-1/2 -translate-y-1/2 flex flex-col gap-2 z-50">
            <button
              id="prev-asset-btn"
              disabled={!hasPrev}
              onClick={onPrev}
              className={cn(
                "p-2.5 rounded-lg border transition-all duration-200",
                hasPrev 
                  ? "bg-zinc-100 hover:bg-zinc-200 border-black/10 text-zinc-700 hover:text-zinc-900 dark:bg-zinc-900/80 dark:hover:bg-zinc-800 dark:border-white/5 dark:text-zinc-300 dark:hover:text-white" 
                  : "bg-zinc-950/40 border-transparent text-zinc-600 cursor-not-allowed"
              )}
              title="上一个"
            >
              <ChevronUp size={20} />
            </button>
            <button
              id="next-asset-btn"
              disabled={!hasNext}
              onClick={onNext}
              className={cn(
                "p-2.5 rounded-lg border transition-all duration-200",
                hasNext 
                  ? "bg-zinc-100 hover:bg-zinc-200 border-black/10 text-zinc-700 hover:text-zinc-900 dark:bg-zinc-900/80 dark:hover:bg-zinc-800 dark:border-white/5 dark:text-zinc-300 dark:hover:text-white" 
                  : "bg-zinc-950/40 border-transparent text-zinc-600 cursor-not-allowed"
              )}
              title="下一个"
            >
              <ChevronDown size={20} />
            </button>
          </div>
        </div>

        {/* Right Side - Premium Details Sidebar */}
        <div className="w-[380px] shrink-0 flex flex-col h-full bg-white border-l border-black/10 dark:bg-[#121214] dark:border-white/5 z-50 select-none">
          {/* Scrollable controls panel */}
          <div className="flex-1 overflow-y-auto px-5 py-6 space-y-6 [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:bg-white/10 [&::-webkit-scrollbar-track]:bg-transparent">
            
            {/* Row 1: Primary Action & Tools */}
            <div className="flex items-center justify-between gap-2.5">
              <button className="flex-1 h-10 px-4 bg-zinc-100 hover:bg-zinc-200 text-zinc-800 dark:bg-white/10 dark:hover:bg-white/15 dark:text-zinc-100 rounded-xl text-sm font-medium transition-all flex items-center justify-center gap-2 border border-black/10 dark:border-white/5">
                <Download size={15} />
                下载
              </button>
              
              <button 
                onClick={() => setIsFavorite(!isFavorite)}
                className={cn(
                  "w-10 h-10 rounded-xl border flex items-center justify-center transition-all",
                  isFavorite 
                    ? "bg-amber-500/10 border-amber-500/30 text-amber-400" 
                    : "bg-zinc-100 border-black/10 text-zinc-500 hover:text-zinc-800 hover:bg-zinc-200 dark:bg-white/5 dark:border-white/5 dark:text-zinc-400 dark:hover:text-white dark:hover:bg-white/10"
                )}
              >
                <Star size={16} fill={isFavorite ? "currentColor" : "none"} />
              </button>

              <button className="w-10 h-10 rounded-xl bg-zinc-100 hover:bg-zinc-200 border border-black/10 text-zinc-500 hover:text-zinc-800 dark:bg-white/5 dark:hover:bg-white/10 dark:border-white/5 dark:text-zinc-400 dark:hover:text-white flex items-center justify-center transition-all">
                <MoreHorizontal size={16} />
              </button>
            </div>

            {/* Row 2: Visual Variations Carousel (4 thumbnails) */}
            <div className="space-y-2">
              <div className="grid grid-cols-4 gap-2">
                {currentItem.thumbnails.map((thumb, index) => (
                  <div
                    key={index}
                    onClick={() => setActiveThumbnailIndex(index)}
                    className={cn(
                      "aspect-square rounded-xl overflow-hidden cursor-pointer bg-zinc-900 border-2 transition-all relative group",
                      activeThumbnailIndex === index 
                        ? "border-blue-500 shadow-md shadow-blue-500/10 scale-102" 
                        : "border-transparent opacity-70 hover:opacity-100 hover:scale-102"
                    )}
                  >
                    <img 
                      src={thumb} 
                      alt={`Variation ${index + 1}`} 
                      className="w-full h-full object-cover rounded-[10px]"
                      referrerPolicy="no-referrer"
                    />
                  </div>
                ))}
              </div>
            </div>

            {/* Row 3: Prompts Details Text */}
            <div className="space-y-2.5">
              <h4 className="text-zinc-400 text-xs font-semibold tracking-wider">图片提示词</h4>
              <div className="p-3.5 bg-zinc-50 rounded-xl border border-black/5 dark:bg-zinc-900/50 dark:border-white/5">
                <p className="text-zinc-200 text-sm leading-relaxed break-words whitespace-pre-wrap select-text">
                  {currentItem.prompt}
                </p>
              </div>
            </div>

            {/* Row 4: Tag indicators */}
            <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1.5 text-xs text-zinc-500 font-medium bg-zinc-50 p-2.5 rounded-lg border border-black/5 dark:bg-zinc-900/20 dark:border-white/5">
              <span className="text-zinc-400">图片 {currentItem.model}</span>
              <span className="w-1 h-1 rounded-full bg-zinc-700"></span>
              <span>{currentItem.aspectRatio}</span>
              <span className="w-1 h-1 rounded-full bg-zinc-700"></span>
              <span>{currentItem.resolution}</span>
              <span className="w-1 h-1 rounded-full bg-zinc-700"></span>
              <button className="flex items-center gap-0.5 text-blue-400/90 hover:text-blue-400 transition-colors">
                详细信息 <Info size={11} className="inline" />
              </button>
            </div>

            {/* Row 5: Action Group 1 */}
            <div className="space-y-1.5">
              <button className="w-full h-10 px-4 bg-zinc-100 hover:bg-zinc-200 text-zinc-700 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-200 rounded-xl text-xs font-medium transition-all flex items-center justify-between border border-black/10 dark:border-white/5">
                <span className="flex items-center gap-2">
                  <Play size={14} className="text-blue-400" />
                  生成视频
                </span>
                <span className="text-[10px] text-zinc-500 bg-white/5 px-1.5 py-0.5 rounded">快速</span>
              </button>
              
              <button className="w-full h-10 px-4 bg-zinc-100 hover:bg-zinc-200 text-zinc-700 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-200 rounded-xl text-xs font-medium transition-all flex items-center justify-between border border-black/10 dark:border-white/5">
                <span className="flex items-center gap-2">
                  <Edit3 size={14} className="text-indigo-400" />
                  去画布编辑
                </span>
                <span className="text-[10px] text-zinc-400 font-bold">&gt;</span>
              </button>

              <button className="w-full h-10 px-4 bg-zinc-100 hover:bg-zinc-200 text-zinc-700 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-200 rounded-xl text-xs font-medium transition-all flex items-center justify-between border border-black/10 dark:border-white/5">
                <span className="flex items-center gap-2">
                  <Image size={14} className="text-amber-400" />
                  用作参考图
                </span>
              </button>
            </div>

            {/* Row 6: Quick Enhancement Tools Grid */}
            <div className="space-y-2">
              <div className="grid grid-cols-2 gap-1.5">
                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-600 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5 text-left">
                  <Sparkles size={13} className="text-emerald-400 shrink-0" />
                  <span className="truncate">智能超清</span>
                </button>

                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-700 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center justify-between border border-black/10 dark:border-white/5 text-left relative overflow-hidden">
                  <span className="flex items-center gap-2 truncate">
                    <Move size={13} className="text-blue-400 shrink-0" />
                    <span className="truncate">多角度</span>
                  </span>
                  <span className="text-[8px] bg-sky-500 text-white px-1 rounded-full scale-90 origin-right">New</span>
                </button>

                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-600 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5 text-left">
                  <Compass size={13} className="text-amber-400 shrink-0" />
                  <span className="truncate">超清</span>
                </button>

                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-600 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5 text-left">
                  <Wand2 size={13} className="text-purple-400 shrink-0" />
                  <span className="truncate">智能改图</span>
                </button>

                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-600 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5 text-left">
                  <Sparkles size={13} className="text-rose-400 shrink-0" />
                  <span className="truncate">细节修复</span>
                </button>

                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-600 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5 text-left">
                  <Crop size={13} className="text-teal-400 shrink-0" />
                  <span className="truncate">局部重绘</span>
                </button>

                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-600 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5 text-left">
                  <Move size={13} className="text-orange-400 shrink-0" />
                  <span className="truncate">扩图</span>
                </button>

                <button className="h-9 px-3 bg-zinc-100 hover:bg-zinc-200 text-zinc-600 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5 text-left">
                  <Scissors size={13} className="text-pink-400 shrink-0" />
                  <span className="truncate">消除笔</span>
                </button>

                <button className="h-9 px-3 col-span-2 bg-zinc-100 hover:bg-zinc-200 text-zinc-700 dark:bg-zinc-900 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs transition-all flex items-center gap-2 border border-black/10 dark:border-white/5">
                  <Video size={13} className="text-sky-400 shrink-0" />
                  <span>对口型</span>
                </button>
              </div>
            </div>

            {/* Row 7: System Workflow items */}
            <div className="space-y-1.5 pt-2 border-t border-black/5 dark:border-white/5">
              <button className="w-full h-10 px-4 bg-zinc-50 hover:bg-zinc-100 text-zinc-600 dark:bg-zinc-900/50 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs font-medium transition-all flex items-center justify-between border border-black/10 dark:border-white/5">
                <span>重新编辑</span>
                <span className="text-[10px] text-zinc-500">&gt;</span>
              </button>

              <button className="w-full h-10 px-4 bg-zinc-50 hover:bg-zinc-100 text-zinc-600 dark:bg-zinc-900/50 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs font-medium transition-all flex items-center justify-between border border-black/10 dark:border-white/5">
                <span>再次生成</span>
                <span className="text-[10px] text-zinc-500">&gt;</span>
              </button>

              <button className="w-full h-10 px-4 bg-zinc-50 hover:bg-zinc-100 text-zinc-600 dark:bg-zinc-900/50 dark:hover:bg-zinc-800/80 dark:text-zinc-300 rounded-xl text-xs font-medium transition-all flex items-center justify-between border border-black/10 dark:border-white/5">
                <span>在生成页面定位</span>
                <span className="text-[10px] text-zinc-500">&gt;</span>
              </button>
            </div>

          </div>
        </div>

      </div>
    </div>
  );
};
