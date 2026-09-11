import React, { useEffect, useState } from 'react';
import { X, Heart, MoreHorizontal, Info, Maximize2 } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { ImageLightbox } from '@sdkwork/agents-pc-commons';

interface ImageDetailModalProps {
  isOpen: boolean;
  onClose: () => void;
  image: {
    src: string;
    imageUrls?: string[];
    currentIndex?: number;
    prompt?: string;
    author: string;
    avatar: string;
    date?: string;
    likes: number;
    aspectRatio?: string;
    model?: string;
    title?: string;
  } | null;
}

export const ImageDetailModal: React.FC<ImageDetailModalProps> = ({ isOpen, onClose, image }) => {
  const [activeIdx, setActiveIdx] = useState<number>(0);
  const [imageAspect, setImageAspect] = useState<number | null>(null);
  const [isLightboxOpen, setIsLightboxOpen] = useState(false);

  useEffect(() => {
    if (image) {
      setActiveIdx(image.currentIndex !== undefined ? image.currentIndex : 0);
    }
  }, [image]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'hidden';
      setImageAspect(null); // Reset when modal opens
    }
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'unset';
    };
  }, [isOpen, onClose]);

  useEffect(() => {
    setImageAspect(null); // Reset aspect ratio calculation when active image switches
  }, [activeIdx]);

  if (!isOpen || !image) return null;

  const imageUrls = image.imageUrls && image.imageUrls.length > 0 ? image.imageUrls : [image.src];
  const currentSrc = imageUrls[activeIdx] || image.src;

  const handleImageLoad = (e: React.SyntheticEvent<HTMLImageElement>) => {
    const img = e.currentTarget;
    if (img.naturalWidth && img.naturalHeight) {
      setImageAspect(img.naturalWidth / img.naturalHeight);
    }
  };

  const handleDownload = () => {
    const a = document.createElement('a');
    a.href = currentSrc;
    a.download = `ai-generation-${activeIdx + 1}.png`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  };

  return (
    <div className="fixed inset-0 z-50 flex bg-[#0a0a0a] animate-in fade-in duration-200">
      {/* Close Button on Left */}
      <button 
        onClick={onClose}
        className="absolute top-6 left-6 p-2.5 bg-white/10 hover:bg-white/20 rounded-full text-zinc-400 hover:text-white transition-all duration-200 z-[60] cursor-pointer"
      >
        <X size={20} />
      </button>

      <div className="flex w-full h-full relative overflow-hidden">
        {/* Left Side - Main Image Container */}
        <div 
          className="flex-1 bg-black/40 p-12 flex flex-col items-center justify-center relative group h-full cursor-zoom-in select-none"
          onClick={() => setIsLightboxOpen(true)}
        >
          <div 
            className="relative rounded-xl overflow-hidden shadow-[0_0_50px_rgba(0,0,0,0.8)] bg-black border border-white/5 flex items-center justify-center transition-all duration-300"
            style={imageAspect ? {
              aspectRatio: imageAspect,
              maxWidth: '100%',
              maxHeight: '75vh',
              width: imageAspect > 1 ? 'min(850px, 100%)' : 'auto',
              height: imageAspect > 1 ? 'auto' : '75vh'
            } : {
              maxWidth: '100%',
              maxHeight: '75vh'
            }}
          >
            {/* "AI生成" Label overlay */}
            <div className="absolute top-4 left-4 bg-black/50 backdrop-blur-md text-white/90 border border-white/10 rounded-full px-3 py-1 text-xs font-semibold tracking-wider z-20">
              AI生成
            </div>

            <img 
              src={currentSrc} 
              alt={image.prompt} 
              onLoad={handleImageLoad}
              className="w-full h-full object-contain rounded-xl animate-in fade-in duration-300"
              referrerPolicy="no-referrer"
            />
          </div>

          {/* Navigation Pill `< 1 / 4 >` at bottom center */}
          {imageUrls.length > 1 && (
            <div 
              onClick={(e) => e.stopPropagation()} 
              className="absolute bottom-8 flex items-center gap-4 bg-zinc-900/80 hover:bg-zinc-900 backdrop-blur-md px-4 py-2 rounded-full border border-white/10 select-none text-zinc-300 text-xs font-semibold shadow-2xl transition-all"
            >
              <button 
                onClick={() => setActiveIdx(prev => (prev - 1 + imageUrls.length) % imageUrls.length)}
                className="hover:text-white transition-colors px-2 cursor-pointer font-bold text-sm"
              >
                &lt;
              </button>
              <span className="font-mono tracking-wider text-white">{activeIdx + 1} / {imageUrls.length}</span>
              <button 
                onClick={() => setActiveIdx(prev => (prev + 1) % imageUrls.length)}
                className="hover:text-white transition-colors px-2 cursor-pointer font-bold text-sm"
              >
                &gt;
              </button>
            </div>
          )}
          
          <div className="absolute bottom-8 right-8 flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-all">
            <button 
              onClick={(e) => {
                e.stopPropagation();
                handleDownload();
              }}
              className="p-3 bg-black/60 hover:bg-black/80 border border-white/10 rounded-lg text-white/80 hover:text-white transition-all backdrop-blur-md cursor-pointer flex items-center gap-2 font-medium text-xs shadow-lg"
              title="下载当前图片"
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
              <span>下载</span>
            </button>
            <button 
              onClick={(e) => {
                e.stopPropagation();
                setIsLightboxOpen(true);
              }}
              className="p-3 bg-black/60 hover:bg-black/80 border border-white/10 rounded-lg text-white/80 hover:text-white transition-all backdrop-blur-md cursor-pointer shadow-lg"
              title="全屏查看"
            >
              <Maximize2 size={16} />
            </button>
          </div>
        </div>

        {/* Right Side - Details Sidebar */}
        <div className="w-[420px] shrink-0 flex flex-col h-full bg-[#111112] border-l border-white/5 relative z-50">
          
          {/* Top Actions: Download, Star, Options */}
          <div className="p-4 border-b border-white/5 flex items-center justify-end gap-2 bg-[#111112]">
            <button 
              onClick={handleDownload}
              className="bg-white/10 hover:bg-white/15 text-zinc-200 px-3.5 py-1.5 rounded-lg text-xs font-semibold transition-colors flex items-center gap-1.5 cursor-pointer"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
              下载
            </button>
            <button className="p-2 bg-white/10 hover:bg-white/15 rounded-lg text-zinc-300 hover:text-white transition-all cursor-pointer">
              <Heart size={15} />
            </button>
            <button className="p-2 bg-white/10 hover:bg-white/15 rounded-lg text-zinc-300 hover:text-white transition-all cursor-pointer">
              <MoreHorizontal size={15} />
            </button>
          </div>

          {/* Scrollable details and toolbox section */}
          <div className="p-6 flex-1 overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:bg-white/10 [&::-webkit-scrollbar-track]:bg-transparent hover:[&::-webkit-scrollbar-thumb]:bg-white/20 space-y-6">
            
            {/* Multiple image thumbnails row for click preview switcher */}
            {imageUrls.length > 1 && (
              <div>
                <div className="flex gap-2 overflow-x-auto pb-1 custom-scrollbar">
                  {imageUrls.map((url, idx) => (
                    <button
                      key={idx}
                      onClick={() => setActiveIdx(idx)}
                      className={cn(
                        "relative w-14 h-14 rounded-lg overflow-hidden border transition-all cursor-pointer bg-black/40 shrink-0",
                        activeIdx === idx 
                          ? "border-cyan-400 ring-2 ring-cyan-400/25 scale-95" 
                          : "border-white/10 opacity-60 hover:opacity-100"
                      )}
                    >
                      <img src={url} className="absolute inset-0 w-full h-full object-cover" alt="" referrerPolicy="no-referrer" />
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Prompt text area */}
            <div>
              <h3 className="text-zinc-400 text-xs font-semibold mb-2.5">图片提示词</h3>
              <p className="text-zinc-200 text-sm leading-relaxed break-words whitespace-pre-wrap select-text font-medium bg-white/5 p-3 rounded-lg border border-white/5">
                {image.prompt || '未提供提示词'}
              </p>
            </div>
            
            {/* Metadata row */}
            <div className="flex items-center gap-2.5 text-[11px] text-zinc-500 font-medium">
              <span className="flex items-center gap-1.5">图片 {image.model || '5.0 Lite'}</span>
              <span className="w-px h-2.5 bg-zinc-800"></span>
              <span>{image.aspectRatio || '1:1'}</span>
              <span className="w-px h-2.5 bg-zinc-800"></span>
              <span>2K 分辨率</span>
              <span className="w-px h-2.5 bg-zinc-800"></span>
              <button className="flex items-center gap-0.5 hover:text-zinc-300 transition-colors">
                详细信息 <Info size={11} />
              </button>
            </div>

            {/* Author Info block */}
            <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <img src={image.avatar} alt={image.author} className="w-9 h-9 rounded-full object-cover border border-white/10" />
                <div className="flex flex-col">
                  <span className="text-zinc-200 font-semibold text-xs">{image.author}</span>
                  <span className="text-[10px] text-zinc-500 mt-0.5">{image.date || '刚刚'} · AI生成</span>
                </div>
              </div>
              <button className="px-2.5 py-1 rounded-full bg-white/10 hover:bg-white/15 text-zinc-300 text-[11px] font-bold transition-all cursor-pointer">
                + 关注
              </button>
            </div>

            {/* Creative Actions section */}
            <div className="space-y-4 pt-2">
              {/* Action grid 1 */}
              <div className="grid grid-cols-2 gap-2">
                <button className="flex items-center gap-2 justify-center px-4 py-2.5 bg-white/5 hover:bg-white/10 border border-white/5 rounded-xl text-zinc-300 text-xs font-bold transition-all cursor-pointer">
                  <svg className="w-3.5 h-3.5 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2.5">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  生成视频
                </button>
                <button className="flex items-center gap-2 justify-center px-4 py-2.5 bg-white/5 hover:bg-white/10 border border-white/5 rounded-xl text-zinc-300 text-xs font-bold transition-all cursor-pointer">
                  <svg className="w-3.5 h-3.5 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2.5">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z" />
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9 3v18M15 3v18M3 9h18M3 15h18" />
                  </svg>
                  去画布编辑
                </button>
              </div>

              <button className="w-full flex items-center justify-center gap-2 py-2.5 bg-[#1f2128] hover:bg-[#282b35] text-zinc-200 border border-white/5 rounded-xl text-xs font-bold transition-all cursor-pointer">
                <svg className="w-3.5 h-3.5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2.5">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                用作参考图
              </button>

              {/* Advanced AI Tools matrix */}
              <div className="grid grid-cols-2 gap-2 text-xs font-semibold pt-2">
                <button className="flex items-center gap-2 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 shrink-0"></span>
                  智能超清
                </button>
                <button className="flex items-center gap-1.5 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0"></span>
                  多角度 <span className="text-[8px] bg-cyan-500/10 text-cyan-400 px-1 py-0.5 rounded scale-90">NEW</span>
                </button>
                <button className="flex items-center gap-2 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-indigo-400 shrink-0"></span>
                  超清
                </button>
                <button className="flex items-center gap-1.5 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-amber-400 shrink-0"></span>
                  智能改图 <span className="text-[12px] text-cyan-400">✦</span>
                </button>
                <button className="flex items-center gap-2 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-purple-400 shrink-0"></span>
                  细节修复
                </button>
                <button className="flex items-center gap-2 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-pink-400 shrink-0"></span>
                  局部重绘
                </button>
                <button className="flex items-center gap-2 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-red-400 shrink-0"></span>
                  扩图
                </button>
                <button className="flex items-center gap-2 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0"></span>
                  消除笔
                </button>
                <button className="flex items-center gap-2 px-3.5 py-2.5 bg-zinc-800/40 hover:bg-zinc-800/70 text-zinc-300 rounded-xl transition-all cursor-pointer col-span-2 text-left">
                  <span className="w-1.5 h-1.5 rounded-full bg-violet-400 shrink-0"></span>
                  对口型
                </button>
              </div>
            </div>

          </div>

          {/* Bottom Footer Actions */}
          <div className="p-6 pt-4 border-t border-black/5 dark:border-white/5 flex items-center gap-3 bg-zinc-100 dark:bg-[#111112]">
            <button className="flex-1 bg-black/5 hover:bg-black/10 text-zinc-700 border border-black/5 dark:bg-[#2f2f2f] dark:hover:bg-[#333333] dark:text-zinc-300 dark:border-white/5 py-2.5 rounded-xl text-[13px] font-bold transition-colors flex items-center justify-center gap-2 cursor-pointer">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
              重新编辑
            </button>
            <button className="flex-1 bg-cyan-600 hover:bg-cyan-500 text-black py-2.5 rounded-xl text-[13px] font-bold transition-colors flex items-center justify-center gap-2 cursor-pointer">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
              再次生成
            </button>
          </div>

        </div>
      </div>

      {isLightboxOpen && (
        <ImageLightbox
          imageUrls={imageUrls}
          initialIndex={activeIdx}
          onClose={() => setIsLightboxOpen(false)}
        />
      )}
    </div>
  );
};
