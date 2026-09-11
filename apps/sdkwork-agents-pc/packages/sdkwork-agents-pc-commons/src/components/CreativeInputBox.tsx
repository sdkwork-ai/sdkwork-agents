import React from 'react';
import { useCreativeInputBox } from '../hooks/useCreativeInputBox';
import { 
  Image as ImageIcon, Crop, Type, AtSign, ArrowUp, ChevronDown, ChevronUp, Plus, Sparkles,
  Wand2, PlaySquare, Music, AudioLines, Smile, Accessibility, Check, Settings2, Wrench, Mic, Box,
  PenTool, RectangleHorizontal, LayoutTemplate, Scan, Minus, Play, Pause, Search, X
} from 'lucide-react';
import { cn } from './MarkdownRenderer';
import { VideoSettingsDropdown } from './creative/VideoSettingsDropdown';
import { MusicSettingsDropdown } from './creative/MusicSettingsDropdown';
import { VoiceSettingsDropdown, VOICE_OPTIONS } from './creative/VoiceSettingsDropdown';
import { ImageSettingsDropdown } from './creative/ImageSettingsDropdown';
import { ModelDropdown } from './creative/ModelDropdown';
import {
  agentsDriveUploadService,
  type AgentsDriveMediaResource,
} from '@sdkwork/agents-pc-core/sdk/driveUploadService';
import { uuid } from '@sdkwork/utils';

interface CreativeInputBoxProps {
  className?: string;
  defaultValue?: string;
  initialMode?: string;
  initialSettings?: {
    model?: string;
    ratio?: string;
    resolution?: string;
    duration?: number;
    videoMode?: 'all_around' | 'first_last' | 'smart_multi';
    count?: number;
  };
  onSubmit?: (value: string, mode: string, settings?: any) => void;
  onChange?: (value: string) => void;
  onModeChange?: (mode: string) => void;
  onSettingsChange?: (settings: any) => void;
}

const CREATION_TYPES = [
  { id: 'agent', label: 'Agent 模式', icon: Wand2 },
  { id: 'image', label: '图片生成', icon: ImageIcon },
  { id: 'video', label: '视频生成', icon: PlaySquare },
  { id: 'music', label: '音乐生成', icon: Music },
  { id: 'voice', label: '配音生成', icon: AudioLines },
  { id: 'digital_human', label: '数字人', icon: Smile },
  { id: 'action', label: '动作模仿', icon: Accessibility },
];

// Reusable custom Model Icon (similar to the image)
const ModelIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
    <path d="M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22ZM12 10.5L8.5 7L12 15L15.5 7L12 10.5Z" />
  </svg>
);

const MOTION_TEMPLATES = [
  {
    id: 'lcx',
    author: '来自@lcx迷妹团',
    duration: '00:08',
    image: 'https://images.unsplash.com/photo-1539109136881-3be0616acf4b?auto=format&fit=crop&w=300&h=300&q=80'
  },
  {
    id: 'xiangchi',
    author: '来自@想吃烧仙草',
    duration: '00:06',
    image: 'https://images.unsplash.com/photo-1509631179647-0177331693ae?auto=format&fit=crop&w=300&h=300&q=80',
    grayscale: true
  },
  {
    id: 'kkgg',
    author: '来自@kkgg',
    duration: '00:02',
    image: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?auto=format&fit=crop&w=300&h=300&q=80'
  },
  {
    id: 'qingmo',
    author: '来自@青墨qm',
    duration: '00:09',
    image: 'https://images.unsplash.com/photo-1508214751196-bcfd4ca60f91?auto=format&fit=crop&w=300&h=300&q=80'
  }
];


export const CreativeInputBox: React.FC<CreativeInputBoxProps> = ({ 
  className, 
  defaultValue = '', 
  initialMode, 
  initialSettings,
  onSubmit,
  onChange,
  onModeChange,
  onSettingsChange
}) => {
  const {
    input, setInput,
    creationType, setCreationType,
    selectedImageModel, setSelectedImageModel,
    selectedVideoModel, setSelectedVideoModel,
    selectedMusicModel, setSelectedMusicModel,
    selectedVoiceModel, setSelectedVoiceModel,
    isCreationMenuOpen, setIsCreationMenuOpen,
    isModelMenuOpen, setIsModelMenuOpen,
    isVideoSettingsOpen, setIsVideoSettingsOpen,
    isMusicSettingsOpen, setIsMusicSettingsOpen,
    isVoiceSettingsOpen, setIsVoiceSettingsOpen,
    isImageSettingsOpen, setIsImageSettingsOpen,
    creationMenuPlacement,
    modelMenuPlacement,
    videoSettingsPlacement,
    musicSettingsPlacement,
    voiceSettingsPlacement,
    imageSettingsPlacement,
    videoSettingsMode, setVideoSettingsMode,
    videoRatio, setVideoRatio,
    videoResolution, setVideoResolution,
    videoCount, setVideoCount,
    videoDuration, setVideoDuration,
    imageRatio, setImageRatio,
    imageResolution, setImageResolution,
    imageWidth, setImageWidth,
    imageHeight, setImageHeight,
    imageAspectRatioLocked, setImageAspectRatioLocked,
    musicSmartDuration, setMusicSmartDuration,
    musicDuration, setMusicDuration,
    selectedVoice, setSelectedVoice,
    activeVoiceCategory, setActiveVoiceCategory,
    playingVoiceId, setPlayingVoiceId,
    agentModelTab, setAgentModelTab,
    agentAutoMatch, setAgentAutoMatch,
    agentSelectedModels, setAgentSelectedModels,
    textareaRef,
    creationWrapperRef,
    modelWrapperRef,
    videoSettingsRef,
    musicSettingsRef,
    voiceSettingsRef,
    imageSettingsRef,
    creationDropdownRef,
    modelDropdownRef,
    videoSettingsDropdownRef,
    musicSettingsDropdownRef,
    voiceSettingsDropdownRef,
    imageSettingsDropdownRef,
    toggleCreationMenu,
    toggleModelMenu,
    toggleVideoSettings,
    toggleMusicSettings,
    toggleVoiceSettings,
    toggleImageSettings,
    uploadedImages, setUploadedImages,
    isVideo, isImage, isAgent, isMusic, isVoice, isDigitalHuman, isAction,
    imageModels, videoModels, musicModels, voiceModels, avatarModels, actionModels,
    selectedAvatarModel, setSelectedAvatarModel,
    selectedActionModel, setSelectedActionModel,
    selectedModelId
  } = useCreativeInputBox(defaultValue, initialMode, initialSettings, onSettingsChange);

  const [isImagesHovered, setIsImagesHovered] = React.useState(false);
  const [uploadResourceId] = React.useState(() => `creative:${uuid()}`);
  const [uploadedImageResources, setUploadedImageResources] = React.useState<AgentsDriveMediaResource[]>([]);
  const fileInputRef = React.useRef<HTMLInputElement>(null);
  const audioInputRef = React.useRef<HTMLInputElement>(null);
  const [uploadedAudioName, setUploadedAudioName] = React.useState<string | null>(null);
  const [uploadedAudioResource, setUploadedAudioResource] = React.useState<AgentsDriveMediaResource | null>(null);
  const [uploadError, setUploadError] = React.useState<string | null>(null);

  // Action mimicry (动作模仿) states and refs
  const [characterImage, setCharacterImage] = React.useState<string | null>(null);
  const [characterImageResource, setCharacterImageResource] = React.useState<AgentsDriveMediaResource | null>(null);
  const [selectedTemplate, setSelectedTemplate] = React.useState<{ id: string; author: string; duration: string; image: string; grayscale?: boolean } | null>(null);
  const [uploadedMotionVideo, setUploadedMotionVideo] = React.useState<{
    name: string;
    url: string;
    resource: AgentsDriveMediaResource;
  } | null>(null);
  const [isMotionMenuOpen, setIsMotionMenuOpen] = React.useState(false);
  const [isTemplatePopoverOpen, setIsTemplatePopoverOpen] = React.useState(false);

  const characterInputRef = React.useRef<HTMLInputElement>(null);
  const motionVideoInputRef = React.useRef<HTMLInputElement>(null);
  const motionMenuRef = React.useRef<HTMLDivElement>(null);
  const templatePopoverRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    const handleOutsideClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (isMotionMenuOpen && !motionMenuRef.current?.contains(target)) {
        setIsMotionMenuOpen(false);
      }
      if (isTemplatePopoverOpen && !templatePopoverRef.current?.contains(target)) {
        setIsTemplatePopoverOpen(false);
      }
    };
    document.addEventListener('mousedown', handleOutsideClick);
    return () => {
      document.removeEventListener('mousedown', handleOutsideClick);
    };
  }, [isMotionMenuOpen, isTemplatePopoverOpen]);

  const triggerAudioUpload = (e: React.MouseEvent) => {
    e.stopPropagation();
    audioInputRef.current?.click();
  };

  const handleAudioChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (!file) return;
    setUploadError(null);
    void agentsDriveUploadService.upload({
      file,
      purpose: 'agent-creative-audio',
      resourceId: uploadResourceId,
    }).then((media) => {
      setUploadedAudioName(media.fileName ?? null);
      setUploadedAudioResource(media);
    }).catch((error) => {
      console.error('Creative audio Drive upload failed', error);
      setUploadError('音频上传失败，请检查 Drive 服务后重试');
    });
  };

  const triggerUpload = (e: React.MouseEvent) => {
    e.stopPropagation();
    fileInputRef.current?.click();
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files: File[] = [];
    for (let index = 0; index < (event.target.files?.length ?? 0); index += 1) {
      const file = event.target.files?.item(index);
      if (file) files.push(file);
    }
    event.target.value = '';
    if (files.length === 0) return;
    setUploadError(null);
    void Promise.all(files.map((file) => agentsDriveUploadService.upload({
      file,
      purpose: 'agent-creative-image',
      resourceId: uploadResourceId,
    }))).then((uploaded) => {
      setUploadedImageResources((previous) => [...previous, ...uploaded]);
      setUploadedImages((previous) => [...previous, ...uploaded.map((media) => media.url ?? '')]);
    }).catch((error) => {
      console.error('Creative image Drive upload failed', error);
      setUploadError('图片上传失败，请检查 Drive 服务后重试');
    });
  };

  const handleSubmit = () => {
    if (onSubmit) {
      const hasActionInput = isAction && (characterImage && (selectedTemplate || uploadedMotionVideo));
      if (!input.trim() && !hasActionInput) {
        return;
      }
      onSubmit(input, creationType, {
        model: selectedModelId,
        ratio: isVideo ? videoRatio : (isImage ? imageRatio : '1:1'),
        resolution: isVideo ? videoResolution : (isImage ? imageResolution : undefined),
        imageWidth: isImage ? imageWidth : undefined,
        imageHeight: isImage ? imageHeight : undefined,
        duration: isVideo ? videoDuration : (isMusic ? musicDuration : undefined),
        videoMode: isVideo ? videoSettingsMode : undefined,
        count: isVideo ? Number(videoCount) : undefined,
        refImages: isAction
          ? [characterImageResource?.uri].filter(Boolean)
          : uploadedImageResources.map((media) => media.uri),
        mediaResources: [
          ...uploadedImageResources,
          ...(uploadedAudioResource ? [uploadedAudioResource] : []),
          ...(characterImageResource ? [characterImageResource] : []),
          ...(uploadedMotionVideo ? [uploadedMotionVideo.resource] : []),
        ],
        characterImage: characterImageResource?.uri,
        selectedTemplate,
        uploadedMotionVideo: uploadedMotionVideo ? {
          name: uploadedMotionVideo.name,
          uri: uploadedMotionVideo.resource.uri,
        } : null,
        uploadedAudio: uploadedAudioResource?.uri,
      });
      setInput('');
      setUploadedImages([]);
      setUploadedImageResources([]);
      setUploadedAudioName(null);
      setUploadedAudioResource(null);
      setCharacterImage(null);
      setCharacterImageResource(null);
      setSelectedTemplate(null);
      setUploadedMotionVideo(null);
      if (textareaRef.current) {
        textareaRef.current.style.height = 'auto';
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const currentType = CREATION_TYPES.find(t => t.id === creationType) || CREATION_TYPES[0];
  const TypeIcon = currentType.icon;
  const currentModels = isDigitalHuman ? avatarModels : isAction ? actionModels : isVoice ? voiceModels : isMusic ? musicModels : isVideo ? videoModels : imageModels;
  const currentModel = currentModels.find(m => m.id === selectedModelId) || currentModels[0];

  const placeholderText = isVideo ? "描述您想要生成的视频内容，例如：“一只赛博朋克风格的机器猫在霓虹灯下行走”..." :
    isMusic ? "描述您想要生成的音乐风格，例如：“一首轻快的电子乐，适合作为Vlog背景音乐”..." :
    isVoice ? "输入您想要转换的文本内容，选择一个合适的音色..." :
    isDigitalHuman ? "输入数字人想要说的话，例如：“大家好，欢迎来到我的智能频道，今天我们来聊聊...”" :
    isAction ? "上传图片后，选择模板即可生成" :
    isAgent ? "告诉智能体您想要完成什么任务，例如：“帮我分析一下最近的AI行业趋势”..." :
    "描述您想要生成的画面，例如：“一杯冒着热气的咖啡放在木桌上，清晨的阳光透过窗户洒进来，电影感”...";

  return (
    <div className={cn("w-full bg-white rounded-2xl border border-black/5 p-4 relative shadow-lg focus-within:border-black/10 transition-colors z-10 dark:bg-[#1e1e1e] dark:border-white/5 dark:shadow-2xl dark:focus-within:border-white/20", className)}>
      <div className="flex gap-4 mb-2">
        {/* Hidden File Input for Image Reference Upload */}
        <input 
          type="file" 
          ref={fileInputRef} 
          onChange={handleFileChange} 
          multiple 
          accept="image/*" 
          className="hidden" 
        />

        {/* Hidden Inputs for Action Mimicry */}
        <input 
          type="file"
          ref={characterInputRef}
          onChange={(e) => {
            const file = e.target.files?.[0];
            e.target.value = '';
            if (!file) return;
            setUploadError(null);
            void agentsDriveUploadService.upload({
              file,
              purpose: 'agent-creative-image',
              resourceId: uploadResourceId,
            }).then((media) => {
              setCharacterImage(media.url ?? '');
              setCharacterImageResource(media);
            }).catch((error) => {
              console.error('Character image Drive upload failed', error);
              setUploadError('角色图片上传失败，请重试');
            });
          }}
          accept="image/*"
          className="hidden"
        />
        <input 
          type="file"
          ref={motionVideoInputRef}
          onChange={(e) => {
            const file = e.target.files?.[0];
            e.target.value = '';
            if (!file) return;
            setUploadError(null);
            void agentsDriveUploadService.upload({
              file,
              purpose: 'agent-creative-video',
              resourceId: uploadResourceId,
            }).then((media) => {
              setUploadedMotionVideo({ name: media.fileName ?? '', url: media.url ?? '', resource: media });
              setSelectedTemplate(null);
            }).catch((error) => {
              console.error('Motion video Drive upload failed', error);
              setUploadError('动作视频上传失败，请重试');
            });
          }}
          accept="video/*"
          className="hidden"
        />

        {uploadError ? (
          <div className="absolute left-4 right-4 top-1 z-20 rounded-lg border border-red-500/20 bg-red-500/10 px-3 py-2 text-xs text-red-300">
            {uploadError}
          </div>
        ) : null}

        {/* Reference Image / Action Dual Cards Container */}
        {isAction ? (
          <div className="flex gap-3 shrink-0 relative select-none">
            {/* 1. Character Card (角色) */}
            <div 
              className="relative cursor-pointer group"
              onClick={(e) => {
                e.stopPropagation();
                characterInputRef.current?.click();
              }}
            >
              {characterImage ? (
                <div 
                  className="w-[60px] h-[80px] rounded-xl overflow-hidden border border-white/10 shadow-lg transition-all duration-300 -rotate-3 hover:scale-[1.05] hover:rotate-0 relative bg-zinc-900"
                >
                  <img src={characterImage} className="w-full h-full object-cover" alt="Character" referrerPolicy="no-referrer" />
                  {/* Remove Button */}
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      setCharacterImage(null);
                      setCharacterImageResource(null);
                      if (characterInputRef.current) characterInputRef.current.value = '';
                    }}
                    className="absolute top-1 right-1 w-4 h-4 bg-black/80 hover:bg-black text-white hover:text-red-400 rounded-full flex items-center justify-center border border-white/20 shadow-md transition-all hover:scale-110 z-10 cursor-pointer"
                    title="删除角色图片"
                  >
                    <X size={10} strokeWidth={3} />
                  </button>
                </div>
              ) : (
                <div 
                  className="w-[60px] h-[80px] bg-zinc-100 rounded-xl flex flex-col items-center justify-center border border-dashed border-zinc-300 hover:bg-zinc-200 transition-all duration-300 -rotate-6 hover:scale-[1.05] hover:rotate-0 dark:bg-[#252525] dark:border-white/10 dark:hover:bg-[#2f2f2f] sdk-dark:bg-[#252525] sdk-dark:border-white/10 sdk-dark:hover:bg-[#2f2f2f]"
                >
                  <Plus size={20} className="text-zinc-500 group-hover:text-zinc-600 dark:text-zinc-500 dark:group-hover:text-zinc-300 sdk-dark:text-zinc-500 sdk-dark:group-hover:text-zinc-300" />
                  <span className="text-[11px] text-zinc-400 mt-1 font-medium dark:text-zinc-500 sdk-dark:text-zinc-500">角色</span>
                </div>
              )}
            </div>

            {/* 2. Motion Card (动作) */}
            <div 
              className="relative cursor-pointer group animate-in fade-in duration-300"
              onClick={(e) => {
                e.stopPropagation();
                setIsMotionMenuOpen(!isMotionMenuOpen);
              }}
            >
              {selectedTemplate || uploadedMotionVideo ? (
                <div 
                  className="w-[60px] h-[80px] rounded-xl overflow-hidden border border-white/10 shadow-lg transition-all duration-300 rotate-3 hover:scale-[1.05] hover:rotate-0 relative bg-zinc-900"
                >
                  {selectedTemplate ? (
                    <>
                      <img src={selectedTemplate.image} className="w-full h-full object-cover" alt="Motion template" referrerPolicy="no-referrer" />
                      {/* Duration Tag */}
                      <div className="absolute bottom-1 left-1 bg-black/60 text-white text-[8px] font-semibold px-1 rounded scale-90 origin-bottom-left">
                        {selectedTemplate.duration}
                      </div>
                      {/* Scan/Maximize Icon on Bottom Right */}
                      <div className="absolute bottom-1 right-1 text-white/80">
                        <Scan size={8} />
                      </div>
                    </>
                  ) : (
                    <div className="w-full h-full flex flex-col items-center justify-center bg-zinc-800 text-zinc-300 p-1">
                      <PlaySquare size={18} className="text-cyan-400" />
                      <span className="text-[8px] mt-1 text-center truncate max-w-full leading-tight opacity-80">{uploadedMotionVideo?.name}</span>
                    </div>
                  )}

                  {/* Remove Button */}
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedTemplate(null);
                      setUploadedMotionVideo(null);
                    }}
                    className="absolute top-1 right-1 w-4 h-4 bg-black/80 hover:bg-black text-white hover:text-red-400 rounded-full flex items-center justify-center border border-white/20 shadow-md transition-all hover:scale-110 z-10 cursor-pointer"
                    title="删除动作"
                  >
                    <X size={10} strokeWidth={3} />
                  </button>
                </div>
              ) : (
                <div 
                  className="w-[60px] h-[80px] bg-zinc-100 rounded-xl flex flex-col items-center justify-center border border-dashed border-zinc-300 hover:bg-zinc-200 transition-all duration-300 rotate-6 hover:scale-[1.05] hover:rotate-0 dark:bg-[#252525] dark:border-white/10 dark:hover:bg-[#2f2f2f] sdk-dark:bg-[#252525] sdk-dark:border-white/10 sdk-dark:hover:bg-[#2f2f2f]"
                >
                  <Sparkles size={18} className="text-zinc-500 group-hover:text-zinc-600 dark:text-zinc-500 dark:group-hover:text-zinc-300 sdk-dark:text-zinc-500 sdk-dark:group-hover:text-zinc-300" />
                  <span className="text-[11px] text-zinc-400 mt-1 font-medium dark:text-zinc-500 sdk-dark:text-zinc-500">动作</span>
                </div>
              )}

              {/* Dropdown Menu: "选择模板" or "上传参考视频" */}
              {isMotionMenuOpen && (
                <div 
                  ref={motionMenuRef}
                  className="absolute left-1/2 -translate-x-1/2 top-[84px] w-[140px] bg-white border border-black/10 rounded-xl p-1 shadow-xl z-50 animate-in fade-in zoom-in-95 duration-100 dark:bg-[#222] dark:border-white/10 dark:shadow-2xl"
                >
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      setIsMotionMenuOpen(false);
                      setIsTemplatePopoverOpen(true);
                    }}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-left text-zinc-600 hover:text-zinc-900 hover:bg-black/5 transition-colors text-[13px] font-medium dark:text-zinc-300 dark:hover:text-white dark:hover:bg-[#2a2a2a]"
                  >
                    <LayoutTemplate size={14} className="text-zinc-400" />
                    <span>选择模板</span>
                  </button>
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      setIsMotionMenuOpen(false);
                      motionVideoInputRef.current?.click();
                    }}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-left text-zinc-600 hover:text-zinc-900 hover:bg-black/5 transition-colors text-[13px] font-medium dark:text-zinc-300 dark:hover:text-white dark:hover:bg-[#2a2a2a]"
                  >
                    <ArrowUp size={14} className="text-zinc-400" />
                    <span>上传参考视频</span>
                  </button>
                </div>
              )}
            </div>

            {/* Template Selection Popover (shown above the bar) */}
            {isTemplatePopoverOpen && (
              <div 
                ref={templatePopoverRef}
                className="absolute left-0 bottom-[96px] w-[350px] bg-white border border-black/10 rounded-2xl p-4 shadow-xl z-55 animate-in fade-in slide-in-from-bottom-2 duration-200 dark:bg-[#1e1e1e] dark:border-white/10 dark:shadow-2xl"
              >
                <div className="text-[12px] font-semibold text-zinc-400 mb-3 select-none flex items-center justify-between">
                  <span>选择动作模板</span>
                  <button 
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      setIsTemplatePopoverOpen(false);
                    }}
                    className="text-zinc-500 hover:text-white transition-colors"
                  >
                    <X size={14} />
                  </button>
                </div>
                <div className="grid grid-cols-2 gap-3 max-h-[280px] overflow-y-auto custom-scrollbar pr-1">
                  {MOTION_TEMPLATES.map((tpl) => (
                    <div 
                      key={tpl.id}
                      onClick={(e) => {
                        e.stopPropagation();
                        setSelectedTemplate(tpl);
                        setUploadedMotionVideo(null);
                        setIsTemplatePopoverOpen(false);
                      }}
                      className="group/item cursor-pointer flex flex-col gap-1.5"
                    >
                      {/* Thumbnail Container */}
                      <div className="relative aspect-[4/3] rounded-xl overflow-hidden border border-white/5 group-hover/item:border-cyan-500/40 transition-colors bg-zinc-900 shadow-md">
                        <img 
                          src={tpl.image} 
                          className={cn(
                            "w-full h-full object-cover transition-transform duration-300 group-hover/item:scale-105",
                            tpl.grayscale ? "grayscale" : ""
                          )} 
                          alt={tpl.author}
                          referrerPolicy="no-referrer"
                        />
                        {/* Play overlay */}
                        <div className="absolute inset-0 bg-black/20 opacity-0 group-hover/item:opacity-100 transition-opacity flex items-center justify-center">
                          <Play size={18} className="text-white fill-white" />
                        </div>
                        {/* Duration Overlay */}
                        <div className="absolute bottom-1.5 left-1.5 bg-black/60 text-white text-[9px] font-bold px-1.5 py-0.5 rounded shadow">
                          {tpl.duration}
                        </div>
                        {/* Expand Icon Overlay */}
                        <div className="absolute bottom-1.5 right-1.5 text-white/80 group-hover/item:text-white transition-colors">
                          <Scan size={12} />
                        </div>
                      </div>
                      {/* Description Label below */}
                      <span className="text-[11px] text-zinc-400 font-medium truncate group-hover/item:text-zinc-200 px-1 transition-colors">
                        {tpl.author}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        ) : uploadedImages.length === 0 ? (
          <div 
            onClick={triggerUpload}
            className="w-[60px] h-[80px] bg-zinc-100 rounded-xl flex items-center justify-center border border-dashed border-zinc-300 hover:bg-zinc-200 transition-colors cursor-pointer group shrink-0 dark:bg-[#252525] dark:border-white/10 dark:hover:bg-[#2f2f2f] sdk-dark:bg-[#252525] sdk-dark:border-white/10 sdk-dark:hover:bg-[#2f2f2f]"
          >
            <div className="text-zinc-400 group-hover:text-zinc-600 flex flex-col items-center dark:text-zinc-500 dark:group-hover:text-zinc-300 sdk-dark:text-zinc-500 sdk-dark:group-hover:text-zinc-300">
              <Plus size={22} strokeWidth={2} className="mb-0.5" />
              <span className="text-[10px] scale-90 whitespace-nowrap opacity-80 font-medium">参考内容</span>
            </div>
          </div>
        ) : (
          <div 
            className="relative shrink-0 flex flex-col justify-end"
            onMouseEnter={() => setIsImagesHovered(true)}
            onMouseLeave={() => setIsImagesHovered(false)}
            style={{
              width: isImagesHovered ? `${(uploadedImages.length + 1) * 72 - 12}px` : '60px',
              height: '80px',
              transition: 'width 0.3s cubic-bezier(0.16, 1, 0.3, 1)',
            }}
          >
            {/* If hovered, show the "智能参考" tag floating at the top */}
            {isImagesHovered && (
              <div className="absolute -top-11 left-1/2 -translate-x-1/2 bg-zinc-800 border border-zinc-700 px-2.5 py-1 rounded-lg text-[10px] font-semibold text-zinc-200 shadow-xl z-50 select-none animate-in fade-in slide-in-from-bottom-2 duration-150 whitespace-nowrap dark:bg-[#252525] dark:border-white/10 dark:text-zinc-300">
                智能参考
              </div>
            )}

            {/* When NOT hovered (collapsed state): Stacked Images */}
            {!isImagesHovered ? (
              <div className="relative w-[60px] h-[80px] cursor-pointer" onClick={triggerUpload}>
                {uploadedImages.slice(0, 3).map((url, idx) => {
                  const stackStyles = [
                    { rotate: 'rotate(4deg)', translate: 'translate(2px, 0px)', zIndex: 30 },
                    { rotate: 'rotate(-6deg)', translate: 'translate(-2px, 1px)', zIndex: 20 },
                    { rotate: 'rotate(8deg)', translate: 'translate(4px, -2px)', zIndex: 10 }
                  ];
                  const style = stackStyles[idx] || { rotate: 'rotate(0deg)', translate: 'translate(0px, 0px)', zIndex: 0 };
                  return (
                    <div 
                      key={url + '-' + idx}
                      className="absolute inset-0 w-[60px] h-[80px] rounded-xl overflow-hidden border border-white/15 shadow-lg bg-zinc-900 transition-all duration-300 origin-center"
                      style={{
                        transform: `${style.rotate} ${style.translate}`,
                        zIndex: style.zIndex,
                      }}
                    >
                      <img src={url} className="w-full h-full object-cover" alt="" referrerPolicy="no-referrer" />
                    </div>
                  );
                })}

                {/* Circular Floating Plus Button at the bottom right corner of the stack */}
                <button
                  type="button"
                  onClick={triggerUpload}
                  className="absolute -bottom-1 -right-1 w-6 h-6 bg-zinc-800 hover:bg-zinc-700 text-white rounded-full flex items-center justify-center border border-white/15 shadow-md transition-all hover:scale-110 z-45 flex items-center justify-center cursor-pointer"
                >
                  <Plus size={14} strokeWidth={2.5} />
                </button>
              </div>
            ) : (
              /* When hovered (expanded state): Horizontal Row of Cards */
              <div className="absolute inset-0 flex items-center gap-3 animate-in fade-in zoom-in-95 duration-200">
                {uploadedImages.map((url, idx) => {
                  const tiltDegrees = idx % 2 === 0 ? '-rotate-3' : 'rotate-3';
                  return (
                    <div 
                      key={url + '-' + idx}
                      className={cn(
                        "relative w-[60px] h-[80px] rounded-xl overflow-visible border border-white/15 shadow-lg bg-zinc-900 group/card cursor-default shrink-0 transition-transform duration-200 hover:scale-[1.04]",
                        tiltDegrees
                      )}
                    >
                      <img src={url} className="w-full h-full object-cover rounded-xl" alt="" referrerPolicy="no-referrer" />
                      
                      {/* Close / Remove button at the top-right corner of each card */}
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          setUploadedImages(prev => prev.filter((_, i) => i !== idx));
                          setUploadedImageResources(prev => prev.filter((_, i) => i !== idx));
                        }}
                        className="absolute -top-1.5 -right-1.5 w-4 h-4 bg-black/80 hover:bg-black text-white hover:text-red-400 rounded-full flex items-center justify-center border border-white/20 shadow-md transition-all hover:scale-110 z-50 flex items-center justify-center cursor-pointer"
                        title="删除图片"
                      >
                        <X size={10} strokeWidth={3} />
                      </button>
                    </div>
                  );
                })}

                {/* Plus placeholder card at the end of the expanded row */}
                <div 
                  onClick={triggerUpload}
                  className="w-[60px] h-[80px] bg-zinc-100 rounded-xl flex items-center justify-center border border-dashed border-zinc-300 hover:bg-zinc-200 transition-all cursor-pointer group shrink-0 rotate-1 dark:bg-[#252525] dark:border-white/10 dark:hover:bg-[#2f2f2f] sdk-dark:bg-[#252525] sdk-dark:border-white/10 sdk-dark:hover:bg-[#2f2f2f]"
                >
                  <div className="text-zinc-400 group-hover:text-zinc-600 dark:text-zinc-500 dark:group-hover:text-zinc-300 sdk-dark:text-zinc-500 sdk-dark:group-hover:text-zinc-300">
                    <Plus size={18} strokeWidth={2.5} />
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
        
        <div className="flex-1">
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              if (onChange) {
                onChange(e.target.value);
              }
            }}
            onKeyDown={handleKeyDown}
            placeholder={placeholderText}
            className="w-full bg-transparent border-none text-[15px] resize-none text-zinc-800 placeholder-zinc-400 outline-none leading-relaxed min-h-[80px] pt-1 dark:text-zinc-200 dark:placeholder-zinc-500"
          />
        </div>
      </div>
      
      <div className="flex items-end justify-between gap-3 mt-2 pt-2 border-t border-transparent relative w-full min-w-0">
        <div 
          className="flex items-center gap-1 text-zinc-500 min-w-0 flex-1 overflow-x-auto flex-nowrap [scrollbar-width:none] [&::-webkit-scrollbar]:hidden dark:text-zinc-400"
        >
          {/* Creation Type Dropdown */}
          <div className="relative shrink-0" ref={creationWrapperRef}>
            <button 
              onClick={toggleCreationMenu}
              className={cn(
                "flex items-center gap-1.5 text-[14px] font-medium transition-colors px-2 py-1.5 rounded-md hover:bg-black/5 -ml-2 whitespace-nowrap dark:hover:bg-[#2a2a2a]",
                isAgent || isImage || isVideo ? "text-cyan-600 hover:text-cyan-700 dark:text-cyan-400 dark:hover:text-cyan-300" : "text-zinc-600 hover:text-zinc-900 dark:text-zinc-300 dark:hover:text-white"
              )}
            >
              <TypeIcon size={16} className="shrink-0" />
              {currentType.label}
              {isCreationMenuOpen ? <ChevronUp size={14} className="opacity-70 ml-0.5 shrink-0" /> : <ChevronDown size={14} className="opacity-70 ml-0.5 shrink-0" />}
            </button>
          </div>
          
          <div className="w-px h-3.5 bg-black/10 mx-1 shrink-0 dark:bg-[#333333]"></div>
          
          {isAgent ? (
            <>
              <button className="flex items-center gap-1.5 hover:text-zinc-800 transition-colors text-[14px] px-2 py-1.5 rounded-md hover:bg-black/5 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a] whitespace-nowrap shrink-0">
                <Settings2 size={16} className="shrink-0" />
                画面设置
              </button>
              <button className="flex items-center gap-1.5 hover:text-zinc-800 transition-colors text-[14px] px-2 py-1.5 rounded-md hover:bg-black/5 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a] whitespace-nowrap shrink-0">
                <Wrench size={16} className="shrink-0" />
                使用技能
              </button>
              <button className="hover:text-zinc-800 transition-colors px-2 py-1.5 rounded-md hover:bg-black/5 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a] shrink-0" title="Mention">
                <AtSign size={16} />
              </button>
            </>
          ) : isVideo ? (
            <div className="relative flex items-center flex-nowrap shrink-0" ref={videoSettingsRef}>
              <div 
                className="flex items-center flex-nowrap cursor-pointer group hover:bg-black/5 rounded-md px-1.5 py-1 transition-colors dark:hover:bg-[#2a2a2a]"
                onClick={toggleVideoSettings}
              >
                <div className="flex items-center gap-1.5 text-[14px] text-zinc-600 group-hover:text-zinc-900 whitespace-nowrap shrink-0 dark:text-zinc-300 dark:group-hover:text-zinc-100">
                  {videoSettingsMode === 'all_around' && <PenTool size={16} className="shrink-0" />}
                  {videoSettingsMode === 'first_last' && <LayoutTemplate size={16} className="shrink-0" />}
                  {videoSettingsMode === 'smart_multi' && <Scan size={16} className="shrink-0" />}
                  {videoSettingsMode === 'all_around' ? '全能参考' : videoSettingsMode === 'first_last' ? '首尾帧' : '智能多帧'}
                </div>
                <div className="w-px h-3 bg-black/10 mx-2 shrink-0 dark:bg-[#333333]"></div>
                <div className="flex items-center gap-1 text-[14px] text-zinc-600 group-hover:text-zinc-900 whitespace-nowrap shrink-0 dark:text-zinc-300 dark:group-hover:text-zinc-100">
                  <RectangleHorizontal size={16} className="shrink-0 mr-0.5" />
                  {videoRatio}
                </div>
                <div className="w-px h-3 bg-black/10 mx-2 shrink-0 dark:bg-[#333333]"></div>
                <div className="flex items-center text-[14px] font-medium text-zinc-600 group-hover:text-zinc-900 whitespace-nowrap shrink-0 dark:text-zinc-300 dark:group-hover:text-zinc-100">
                  {videoResolution} {videoResolution === '1080P' && <Sparkles size={10} className="inline text-cyan-400 fill-cyan-400 -mt-2 ml-0.5 shrink-0" />}
                </div>
                <div className="w-px h-3 bg-black/10 mx-2 shrink-0 dark:bg-[#333333]"></div>
                <div className="text-[14px] font-medium text-zinc-600 group-hover:text-zinc-900 whitespace-nowrap shrink-0 dark:text-zinc-300 dark:group-hover:text-zinc-100">
                  {videoCount}
                </div>
                <div className="w-px h-3 bg-black/10 mx-2 shrink-0 dark:bg-[#333333]"></div>
                <div className="text-[14px] font-medium text-zinc-600 group-hover:text-zinc-900 whitespace-nowrap shrink-0 dark:text-zinc-300 dark:group-hover:text-zinc-100">
                  {videoDuration}s
                </div>
              </div>
              <button className="hover:text-zinc-200 transition-colors p-1.5 rounded-md hover:bg-white/5 shrink-0 ml-1" title="Mention">
                <AtSign size={16} />
              </button>
            </div>
          ) : isMusic ? (
            <div className="flex items-center">
              <div className="relative shrink-0 flex items-center" ref={musicSettingsRef}>
                <button 
                  onClick={toggleMusicSettings}
                  className="flex items-center gap-1.5 hover:text-zinc-800 transition-colors text-[14px] px-2 py-1.5 rounded-md hover:bg-black/5 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a] whitespace-nowrap shrink-0"
                >
                  <div className="flex items-center justify-center w-[14px] h-[14px] rounded-full border border-current shrink-0">
                    <div className="w-1.5 h-1.5 rounded-full bg-current"></div>
                  </div>
                  智能时长
                </button>
              </div>
              <button className="hover:text-zinc-800 transition-colors px-2 py-1.5 rounded-md hover:bg-black/5 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a] shrink-0 ml-1" title="Mention">
                <AtSign size={16} />
              </button>
            </div>
          ) : isVoice ? (
            <div className="flex items-center gap-2">
              <div className="relative shrink-0 flex items-center" ref={voiceSettingsRef}>
                <button 
                  onClick={toggleVoiceSettings}
                  className="flex items-center gap-1.5 hover:text-zinc-200 transition-colors text-[14px] px-3 py-1.5 rounded-md hover:bg-white/5 whitespace-nowrap shrink-0"
                >
                  <AudioLines size={16} className="shrink-0 text-cyan-400" />
                  <span className="text-cyan-400 font-medium">{VOICE_OPTIONS.find(v => v.id === selectedVoice)?.name || '选择语音'}</span>
                  {isVoiceSettingsOpen ? <ChevronUp size={14} className="text-cyan-400/70 ml-0.5 shrink-0" /> : <ChevronDown size={14} className="text-cyan-400/70 ml-0.5 shrink-0" />}
                </button>
              </div>
              <button className="flex items-center gap-1.5 hover:text-zinc-800 transition-colors text-[14px] px-3 py-1.5 rounded-md border border-black/10 hover:bg-black/5 whitespace-nowrap shrink-0 dark:hover:text-zinc-200 dark:border-white/10 dark:hover:bg-[#2a2a2a]">
                <Mic size={16} className="shrink-0" />
                克隆声音
              </button>
            </div>
          ) : isDigitalHuman ? (
            <div className="flex items-center gap-2">
              <input 
                type="file" 
                ref={audioInputRef} 
                onChange={handleAudioChange} 
                accept="audio/*" 
                className="hidden" 
              />
              <button 
                onClick={triggerAudioUpload}
                className={cn(
                  "flex items-center gap-1.5 text-[14px] px-3.5 py-1.5 rounded-lg border transition-all whitespace-nowrap shrink-0 select-none cursor-pointer",
                  uploadedAudioName
                    ? "bg-cyan-500/10 border-cyan-400/20 text-cyan-600 font-semibold dark:text-cyan-400"
                    : "bg-zinc-100 border-zinc-200 text-zinc-600 hover:text-zinc-900 hover:bg-zinc-200 dark:bg-[#252525] dark:border-white/10 dark:text-zinc-300 dark:hover:text-white dark:hover:bg-[#2a2a2a]"
                )}
                title="上传音频进行人物对白/配音"
              >
                <ArrowUp size={14} className="shrink-0" />
                <span>{uploadedAudioName ? `已上传音频: ${uploadedAudioName}` : '上传音频'}</span>
                {uploadedAudioName && (
                  <span 
                    onClick={(e) => {
                      e.stopPropagation();
                      setUploadedAudioName(null);
                      setUploadedAudioResource(null);
                      if (audioInputRef.current) audioInputRef.current.value = '';
                    }}
                    className="ml-1.5 hover:text-red-400 text-zinc-400 font-bold px-0.5 text-xs transition-colors"
                  >
                    ×
                  </span>
                )}
              </button>
            </div>
          ) : isAction ? (
            <div className="flex items-center gap-2 select-none">
              <div className="text-[13px] text-zinc-500 flex items-center gap-1.5 font-medium">
                <Sparkles size={13} className="text-cyan-400 fill-cyan-400" />
                <span>智能动作姿态参考</span>
              </div>
            </div>
          ) : (
            <>
              <div className="relative shrink-0 flex items-center" ref={imageSettingsRef}>
                <button 
                  onClick={toggleImageSettings}
                  className={cn(
                    "flex items-center gap-1.5 text-[13px] px-2.5 py-1.5 rounded-lg border transition-all whitespace-nowrap shrink-0 hover:bg-white/10 select-none cursor-pointer animate-in fade-in duration-200",
                    isImageSettingsOpen 
                      ? "bg-zinc-100 border-zinc-300 text-zinc-900 shadow-lg scale-[1.02] dark:bg-[#333333] dark:border-white/20 dark:text-white" 
                      : "bg-zinc-100 border-zinc-200 text-zinc-600 hover:text-zinc-900 dark:bg-[#252525] dark:border-white/10 dark:text-zinc-300 dark:hover:text-white"
                  )}
                  title="图片参数设置"
                >
                  <Crop size={14} className="shrink-0 opacity-80" />
                  <span className="font-semibold">{imageRatio}</span>
                  <div className="w-px h-3.5 bg-white/10 mx-0.5"></div>
                  <span className="font-semibold">{imageResolution}</span>
                  {imageResolution === '1K' ? (
                    <span className="bg-cyan-400 text-black px-1.5 py-0.5 rounded text-[8px] font-extrabold tracking-tight uppercase leading-none scale-90 origin-left">
                      限免1次
                    </span>
                  ) : (
                    <span className="text-cyan-400 text-xs font-semibold">✦</span>
                  )}
                </button>
              </div>
              <button className="flex items-center hover:text-zinc-800 transition-colors text-[14px] px-2 py-1.5 rounded-md hover:bg-black/5 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a] whitespace-nowrap shrink-0" title="Text">
                <span className="font-serif font-bold text-[15px] leading-none">T</span><span className="text-[10px] leading-none mb-2">+</span>
              </button>
              <button className="hover:text-zinc-800 transition-colors px-2 py-1.5 rounded-md hover:bg-black/5 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a] shrink-0" title="Mention">
                <AtSign size={16} />
              </button>
            </>
          )}
        </div>
        
        <div className="flex items-center gap-1 shrink-0 self-end pl-2 relative" ref={modelWrapperRef}>
          {isAgent ? (
            <button 
              onClick={toggleModelMenu}
              className="flex items-center gap-1.5 text-[14px] text-zinc-600 hover:text-zinc-900 transition-colors px-2 py-1.5 rounded-md hover:bg-black/5 whitespace-nowrap shrink-0 dark:text-zinc-300 dark:hover:text-white dark:hover:bg-[#2a2a2a]"
            >
              <Box size={14} className="opacity-70 shrink-0" />
              {agentAutoMatch ? '模型·自动' : '模型·自选'} {isModelMenuOpen ? <ChevronUp size={14} className="text-zinc-500 ml-0.5 shrink-0" /> : <ChevronDown size={14} className="text-zinc-500 ml-0.5 shrink-0" />}
            </button>
          ) : (
            <button 
              onClick={toggleModelMenu}
              className="flex items-center gap-1.5 text-[14px] text-zinc-600 hover:text-zinc-900 transition-colors px-3 py-1.5 rounded-md hover:bg-black/5 bg-zinc-100 whitespace-nowrap dark:text-zinc-300 dark:hover:text-white dark:hover:bg-[#333333] dark:bg-[#2f2f2f]"
            >
              <span className="shrink-0"><ModelIcon /></span>
              <span className="max-w-[150px] truncate">{currentModel.label}</span>
              {currentModel.spark && <Sparkles size={12} className="text-cyan-400 fill-cyan-400 -ml-0.5 shrink-0" />}
              {isModelMenuOpen ? <ChevronUp size={14} className="text-zinc-400 ml-0.5 shrink-0" /> : <ChevronDown size={14} className="text-zinc-400 ml-0.5 shrink-0" />}
            </button>
          )}
          
          {isAgent ? (
            <button className="flex items-center justify-center w-8 h-8 rounded-full hover:bg-white/10 transition-colors mx-1 text-zinc-400 hover:text-zinc-200">
              <Mic size={18} />
            </button>
          ) : isVideo ? (
            <div className="flex items-center gap-1.5 text-[13px] text-zinc-500 mx-2 font-medium">
              <Sparkles size={14} className="fill-zinc-500" /> 36
            </div>
          ) : isMusic ? (
            <div className="flex items-center gap-1.5 text-[13px] text-zinc-500 mx-2 font-medium">
              <Sparkles size={14} className="fill-zinc-500" /> 5/首
            </div>
          ) : (
            <div className="flex items-center gap-1.5 text-[13px] text-zinc-500 mx-2 font-medium">
              <Sparkles size={14} className="fill-zinc-500" /> 3/张
            </div>
          )}
          
          <button 
            onClick={handleSubmit}
            disabled={!input.trim()}
            className={cn(
              "w-8 h-8 rounded-full flex items-center justify-center transition-all ml-1.5",
              input.trim() 
                ? "bg-zinc-900 text-white hover:bg-zinc-700 cursor-pointer shadow-md dark:bg-white dark:text-black dark:hover:bg-zinc-200" 
                : "bg-zinc-200 text-zinc-400 cursor-not-allowed dark:bg-[#333333] dark:text-white/30"
            )}
          >
            <ArrowUp size={18} strokeWidth={2.5} />
          </button>
        </div>

        {/* Render Dropdowns here to prevent clipping */}
        {isCreationMenuOpen && (
          <div 
            ref={creationDropdownRef}
            className={cn(
              "absolute left-0 w-[calc(100vw-32px)] sm:w-[220px] max-w-[220px] bg-white border border-black/10 rounded-xl shadow-xl py-2 z-50 overflow-y-auto custom-scrollbar text-[13px] animate-in fade-in zoom-in-95 duration-100 max-h-[50vh] dark:bg-[#1e1e1e] dark:border-white/10 dark:shadow-2xl",
              creationMenuPlacement === 'top' ? "bottom-full mb-2" : "top-full mt-2"
            )}
          >
            <div className="px-4 py-1.5 text-zinc-400 text-[12px] mb-1 dark:text-zinc-500">创作类型</div>
            {CREATION_TYPES.map(type => {
              const Icon = type.icon;
              return (
                <button
                  key={type.id}
                  onClick={() => { 
                    setCreationType(type.id); 
                    setIsCreationMenuOpen(false); 
                    if (onModeChange) onModeChange(type.id);
                  }}
                  className={cn(
                    "w-full flex items-center justify-between px-4 py-2.5 hover:bg-black/5 transition-colors dark:hover:bg-[#2a2a2a]",
                    creationType === type.id ? "bg-zinc-100 text-zinc-900 dark:bg-[#2f2f2f] dark:text-white" : "text-zinc-600 hover:text-zinc-900 dark:text-zinc-300 dark:hover:text-white"
                  )}
                >
                  <div className="flex items-center gap-3">
                    <Icon size={16} className={creationType === type.id ? "text-zinc-900 shrink-0 dark:text-white" : "text-zinc-400 shrink-0"} />
                    <span className="text-[14px]">{type.label}</span>
                  </div>
                  {creationType === type.id && <Check size={16} className="text-zinc-900 shrink-0 dark:text-white" />}
                </button>
              );
            })}
          </div>
        )}
        
        {isVideoSettingsOpen && (
          <VideoSettingsDropdown
            videoSettingsMode={videoSettingsMode}
            setVideoSettingsMode={setVideoSettingsMode}
            videoRatio={videoRatio}
            setVideoRatio={setVideoRatio}
            videoResolution={videoResolution}
            setVideoResolution={setVideoResolution}
            videoCount={videoCount}
            setVideoCount={setVideoCount}
            videoDuration={videoDuration}
            setVideoDuration={setVideoDuration}
            videoSettingsPlacement={videoSettingsPlacement}
            dropdownRef={videoSettingsDropdownRef}
          />
        )}

        {isMusicSettingsOpen && (
          <MusicSettingsDropdown
            musicSmartDuration={musicSmartDuration}
            setMusicSmartDuration={setMusicSmartDuration}
            musicDuration={musicDuration}
            setMusicDuration={setMusicDuration}
            musicSettingsPlacement={musicSettingsPlacement}
            dropdownRef={musicSettingsDropdownRef}
          />
        )}

        {isVoiceSettingsOpen && (
          <VoiceSettingsDropdown
            selectedVoice={selectedVoice}
            setSelectedVoice={setSelectedVoice}
            voiceSettingsPlacement={voiceSettingsPlacement}
            dropdownRef={voiceSettingsDropdownRef}
            onClose={() => setIsVoiceSettingsOpen(false)}
          />
        )}

        {isImageSettingsOpen && (
          <ImageSettingsDropdown
            imageRatio={imageRatio}
            setImageRatio={setImageRatio}
            imageResolution={imageResolution}
            setImageResolution={setImageResolution}
            imageWidth={imageWidth}
            setImageWidth={setImageWidth}
            imageHeight={imageHeight}
            setImageHeight={setImageHeight}
            imageAspectRatioLocked={imageAspectRatioLocked}
            setImageAspectRatioLocked={setImageAspectRatioLocked}
            imageSettingsPlacement={imageSettingsPlacement}
            dropdownRef={imageSettingsDropdownRef}
          />
        )}

        {isModelMenuOpen && (
          <ModelDropdown
            isAgent={isAgent}
            agentAutoMatch={agentAutoMatch}
            setAgentAutoMatch={setAgentAutoMatch}
            agentModelTab={agentModelTab}
            setAgentModelTab={setAgentModelTab}
            agentSelectedModels={agentSelectedModels}
            setAgentSelectedModels={setAgentSelectedModels}
            agentImageModels={imageModels}
            agentVideoModels={videoModels}
            selectedModelId={selectedModelId}
            onSelectModel={(id) => {
              if (isDigitalHuman) setSelectedAvatarModel(id);
              else if (isAction) setSelectedActionModel(id);
              else if (isVoice) setSelectedVoiceModel(id);
              else if (isMusic) setSelectedMusicModel(id);
              else if (isVideo) setSelectedVideoModel(id); 
              else setSelectedImageModel(id); 
              setIsModelMenuOpen(false);
            }}
            currentModels={currentModels}
            currentModel={currentModel}
            modelMenuPlacement={modelMenuPlacement}
            dropdownRef={modelDropdownRef}
            isVoice={isVoice}
            isDigitalHuman={isDigitalHuman}
            isAction={isAction}
          />
        )}
      </div>
    </div>
  );
};
