import React, { useEffect, useRef, useState } from 'react';
import { X, Heart, MoreHorizontal, Maximize2, Play, Pause, Volume2, VolumeX } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { VideoLightbox } from '@sdkwork/agents-pc-commons';

interface VideoDetailModalProps {
  isOpen: boolean;
  onClose: () => void;
  video: {
    id: string;
    title: string;
    author: string;
    avatar: string;
    likes: number;
    duration: string;
    desc: string;
    cover: string;
    videoUrl: string;
  } | null;
}

export const VideoDetailModal: React.FC<VideoDetailModalProps> = ({ isOpen, onClose, video }) => {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [isPlaying, setIsPlaying] = useState(true);
  const [isMuted, setIsMuted] = useState(false);
  const [progress, setProgress] = useState(0);
  const [videoAspect, setVideoAspect] = useState<number>(16 / 9);
  const [isFollowed, setIsFollowed] = useState(false);
  const [isLiked, setIsLiked] = useState(false);
  const [likesCount, setLikesCount] = useState(0);
  const [isVideoLightboxOpen, setIsVideoLightboxOpen] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'hidden';
      setIsPlaying(true);
      if (video) {
        setLikesCount(video.likes);
        setIsLiked(false);
        setVideoAspect(16 / 9); // Reset to default 16:9 until metadata loads
      }
    }
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'unset';
    };
  }, [isOpen, onClose, video]);

  useEffect(() => {
    if (videoRef.current) {
      if (isPlaying) {
        videoRef.current.play().catch(() => {
          // Auto-play might be blocked by browser policies if not muted, handle it gracefully
          setIsPlaying(false);
        });
      } else {
        videoRef.current.pause();
      }
    }
  }, [isPlaying, video]);

  if (!isOpen || !video) return null;

  const handleLoadedMetadata = (e: React.SyntheticEvent<HTMLVideoElement>) => {
    const videoEl = e.currentTarget;
    if (videoEl.videoWidth && videoEl.videoHeight) {
      setVideoAspect(videoEl.videoWidth / videoEl.videoHeight);
    }
  };

  const togglePlay = () => {
    setIsPlaying(!isPlaying);
  };

  const toggleMute = () => {
    if (videoRef.current) {
      videoRef.current.muted = !isMuted;
      setIsMuted(!isMuted);
    }
  };

  const handleTimeUpdate = () => {
    if (videoRef.current) {
      const current = videoRef.current.currentTime;
      const duration = videoRef.current.duration;
      if (duration) {
        setProgress((current / duration) * 100);
      }
    }
  };

  const handleProgressChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (videoRef.current) {
      const newTime = (parseFloat(e.target.value) / 100) * videoRef.current.duration;
      videoRef.current.currentTime = newTime;
      setProgress(parseFloat(e.target.value));
    }
  };

  const handleFullscreen = () => {
    if (videoRef.current) {
      if (videoRef.current.requestFullscreen) {
        videoRef.current.requestFullscreen();
      }
    }
  };

  const handleLike = () => {
    if (isLiked) {
      setLikesCount(prev => prev - 1);
    } else {
      setLikesCount(prev => prev + 1);
    }
    setIsLiked(!isLiked);
  };

  return (
    <div className="fixed inset-0 z-50 flex bg-[#0a0a0a]/95 animate-in fade-in duration-200">
      {/* Close Button */}
      <button 
        onClick={onClose}
        className="absolute top-6 left-6 p-2.5 bg-white/10 hover:bg-white/20 rounded-full text-zinc-400 hover:text-white transition-colors z-[60]"
      >
        <X size={20} />
      </button>

      <div className="flex w-full h-full relative overflow-hidden">
        {/* Left Side - Custom Video Player */}
        <div className="flex-1 bg-black/60 p-6 md:p-12 flex flex-col items-center justify-center relative group h-full">
          <div 
            className="relative rounded-xl overflow-hidden shadow-2xl bg-black border border-white/5 flex items-center justify-center transition-all duration-300"
            style={{
              aspectRatio: videoAspect,
              maxWidth: '100%',
              maxHeight: '80vh',
              width: videoAspect > 1 ? 'min(850px, 100%)' : 'auto',
              height: videoAspect > 1 ? 'auto' : '80vh'
            }}
          >
            <video
              ref={videoRef}
              src={video.videoUrl}
              poster={video.cover}
              className="w-full h-full object-contain"
              loop
              muted={isMuted}
              onTimeUpdate={handleTimeUpdate}
              onLoadedMetadata={handleLoadedMetadata}
              onClick={togglePlay}
            />

            {/* Custom Control Overlay */}
            <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4">
              {/* Progress bar */}
              <div className="w-full flex items-center gap-2 mb-3">
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={progress}
                  onChange={handleProgressChange}
                  className="w-full h-1 bg-white/20 accent-cyan-500 rounded-lg cursor-pointer hover:h-1.5 transition-all"
                />
              </div>

              {/* Controls footer */}
              <div className="flex items-center justify-between text-white">
                <div className="flex items-center gap-4">
                  <button onClick={togglePlay} className="p-1.5 hover:bg-white/10 rounded-full transition-colors">
                    {isPlaying ? <Pause size={18} /> : <Play size={18} />}
                  </button>
                  <button onClick={toggleMute} className="p-1.5 hover:bg-white/10 rounded-full transition-colors">
                    {isMuted ? <VolumeX size={18} /> : <Volume2 size={18} />}
                  </button>
                  <span className="text-[12px] text-zinc-300 font-mono">
                    {video.duration}
                  </span>
                </div>

                <div className="flex items-center gap-4">
                  <button onClick={() => setIsVideoLightboxOpen(true)} className="p-1.5 hover:bg-white/10 rounded-full transition-colors">
                    <Maximize2 size={18} />
                  </button>
                </div>
              </div>
            </div>

            {/* Big Centered Play/Pause Button on Pause State */}
            {!isPlaying && (
              <button 
                onClick={togglePlay}
                className="absolute inset-0 m-auto w-16 h-16 flex items-center justify-center rounded-full bg-black/50 text-white backdrop-blur-sm border border-white/10 hover:scale-105 transition-transform"
              >
                <Play size={28} className="ml-1" />
              </button>
            )}
          </div>
        </div>

        {/* Right Side - Details Sidebar */}
        <div className="w-[400px] shrink-0 flex flex-col h-full bg-[#141414] border-l border-white/5 relative z-50">
          {/* Author info */}
          <div className="p-6 pb-5 border-b border-white/5 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <img src={video.avatar} alt={video.author} className="w-10 h-10 rounded-full object-cover" />
              <div className="flex flex-col">
                <div className="flex items-center gap-2">
                  <span className="text-zinc-200 font-medium text-[14px]">{video.author}</span>
                  <button 
                    onClick={() => setIsFollowed(!isFollowed)}
                    className={cn(
                      "px-2.5 py-0.5 rounded-full text-[11px] transition-colors font-medium",
                      isFollowed 
                        ? "bg-zinc-800 text-zinc-400 hover:bg-zinc-700" 
                        : "bg-cyan-500 text-black hover:bg-cyan-400"
                    )}
                  >
                    {isFollowed ? '已关注' : '+ 关注'}
                  </button>
                </div>
                <div className="flex items-center gap-2 text-[11px] text-zinc-500 mt-0.5">
                  <span>刚刚</span>
                  <span className="w-1 h-1 rounded-full bg-zinc-600"></span>
                  <span>AI 智能视频</span>
                </div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button 
                onClick={handleLike}
                className={cn(
                  "flex items-center gap-1.5 p-2 rounded-lg transition-colors",
                  isLiked ? "text-red-500 bg-red-500/10" : "text-zinc-400 hover:text-white"
                )}
              >
                <Heart size={18} fill={isLiked ? "currentColor" : "none"} />
                <span className="text-[13px] font-mono">{likesCount}</span>
              </button>
              <button className="text-zinc-400 hover:text-white p-2 rounded-lg transition-colors">
                <MoreHorizontal size={18} />
              </button>
            </div>
          </div>

          {/* Video Description */}
          <div className="p-6 flex-1 overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:bg-white/10 [&::-webkit-scrollbar-track]:bg-transparent hover:[&::-webkit-scrollbar-thumb]:bg-white/20">
            <div className="mb-6">
              <h3 className="text-zinc-200 text-[15px] font-semibold mb-2">{video.title}</h3>
              <p className="text-zinc-400 text-[13px] leading-relaxed break-words whitespace-pre-wrap">
                {video.desc || "这件精美的AI视频作品向大家展现了人工智能时代创作的无限可能性，每一个画面的生成、光影的流转都融入了艺术家对未来的深度思考。"}
              </p>
            </div>

            <div className="p-4 bg-[#1e1e1e] rounded-xl border border-white/5 mb-6">
              <div className="text-[11px] text-zinc-500 uppercase tracking-wider mb-1.5 font-semibold">生成模型</div>
              <div className="text-[13px] text-zinc-300">Seedance 2.0 (High Definition Video Model)</div>
            </div>

            <div className="p-4 bg-[#1e1e1e] rounded-xl border border-white/5">
              <div className="text-[11px] text-zinc-500 uppercase tracking-wider mb-1.5 font-semibold">推荐提示词</div>
              <p className="text-[12px] text-zinc-400 leading-normal italic">
                "stunning AI video production, high fidelity details, unreal engine 5 style, dramatic lighting, volumetric effects, cinematic atmosphere, masterpiece..."
              </p>
            </div>
          </div>

          {/* Action Footer */}
          <div className="p-6 pt-4 border-t border-white/5 flex items-center gap-3 bg-[#141414]">
            <button className="flex-1 bg-white/10 hover:bg-white/15 text-zinc-200 py-2.5 rounded-xl text-[13px] font-medium transition-colors flex items-center justify-center gap-1.5">
              <Play size={14} />
              做同款视频
            </button>
            <button className="flex-1 bg-cyan-500 text-black hover:bg-cyan-400 py-2.5 rounded-xl text-[13px] font-semibold transition-colors flex items-center justify-center gap-1.5">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M15 3h6v6"/><path d="M9 21H3v-6"/><path d="M21 3l-7 7"/><path d="M3 21l7-7"/></svg>
              参考此视频
            </button>
          </div>
        </div>
      </div>

      {isVideoLightboxOpen && (
        <VideoLightbox
          videoUrls={[video.videoUrl]}
          onClose={() => setIsVideoLightboxOpen(false)}
        />
      )}
    </div>
  );
};
