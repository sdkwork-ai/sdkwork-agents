import { useState, useRef, useEffect } from 'react';

import { useCreativeModelCatalog } from '../creative-model-catalog';

export interface CreativeInputSettings {
  model?: string;
  ratio?: string;
  resolution?: string;
  duration?: number;
  videoMode?: 'all_around' | 'first_last' | 'smart_multi';
  count?: number;
}

export function useCreativeInputBox(
  defaultValue: string = '', 
  initialMode?: string, 
  initialSettings?: CreativeInputSettings, 
  onSettingsChange?: (settings: any) => void
) {
  const [input, setInput] = useState(() => {
    if (defaultValue) return defaultValue;
    try {
      return localStorage.getItem('creative_input_draft') || '';
    } catch {
      return '';
    }
  });

  useEffect(() => {
    try {
      localStorage.setItem('creative_input_draft', input);
    } catch {
      // ignore
    }
  }, [input]);
  const [creationType, setCreationType] = useState(initialMode || 'video'); // Default to video based on prompt
  // Unified model catalog: every modality resolves its selection through the
  // shared service (static sdkwork-models baseline + remote sync layer).
  const imageCatalog = useCreativeModelCatalog('image', { initialModelId: initialSettings?.model });
  const videoCatalog = useCreativeModelCatalog('video', { initialModelId: initialSettings?.model });
  const musicCatalog = useCreativeModelCatalog('music', { initialModelId: initialSettings?.model });
  const voiceCatalog = useCreativeModelCatalog('voice', { initialModelId: initialSettings?.model });
  const digitalHumanCatalog = useCreativeModelCatalog('digital_human', { initialModelId: initialSettings?.model });
  const actionCatalog = useCreativeModelCatalog('action', { initialModelId: initialSettings?.model });
  const selectedImageModel = imageCatalog.selectedModelId;
  const setSelectedImageModel = imageCatalog.selectModel;
  const selectedVideoModel = videoCatalog.selectedModelId;
  const setSelectedVideoModel = videoCatalog.selectModel;
  const selectedMusicModel = musicCatalog.selectedModelId;
  const setSelectedMusicModel = musicCatalog.selectModel;
  const selectedVoiceModel = voiceCatalog.selectedModelId;
  const setSelectedVoiceModel = voiceCatalog.selectModel;
  const selectedAvatarModel = digitalHumanCatalog.selectedModelId;
  const setSelectedAvatarModel = digitalHumanCatalog.selectModel;
  const selectedActionModel = actionCatalog.selectedModelId;
  const setSelectedActionModel = actionCatalog.selectModel;
  
  const [isCreationMenuOpen, setIsCreationMenuOpen] = useState(false);
  const [isModelMenuOpen, setIsModelMenuOpen] = useState(false);
  const [isVideoSettingsOpen, setIsVideoSettingsOpen] = useState(false);
  const [isMusicSettingsOpen, setIsMusicSettingsOpen] = useState(false);
  const [isVoiceSettingsOpen, setIsVoiceSettingsOpen] = useState(false);
  
  const [creationMenuPlacement, setCreationMenuPlacement] = useState<'top' | 'bottom'>('top');
  const [modelMenuPlacement, setModelMenuPlacement] = useState<'top' | 'bottom'>('top');
  const [videoSettingsPlacement, setVideoSettingsPlacement] = useState<'top' | 'bottom'>('top');
  const [musicSettingsPlacement, setMusicSettingsPlacement] = useState<'top' | 'bottom'>('top');
  const [voiceSettingsPlacement, setVoiceSettingsPlacement] = useState<'top' | 'bottom'>('top');
  const [imageSettingsPlacement, setImageSettingsPlacement] = useState<'top' | 'bottom'>('top');

  const [videoSettingsMode, setVideoSettingsMode] = useState<'all_around' | 'first_last' | 'smart_multi'>(initialSettings?.videoMode || 'all_around');
  const [videoRatio, setVideoRatio] = useState(initialSettings?.ratio || '16:9');
  const [videoResolution, setVideoResolution] = useState(initialSettings?.resolution || '720P');
  const [videoCount, setVideoCount] = useState(initialSettings?.count ? String(initialSettings.count) : '1');
  const [videoDuration, setVideoDuration] = useState(initialSettings?.duration || 4);

  const [imageRatio, setImageRatio] = useState(initialSettings?.ratio || '1:1');
  const [imageResolution, setImageResolution] = useState(initialSettings?.resolution || '2K');
  const [imageWidth, setImageWidth] = useState(1328);
  const [imageHeight, setImageHeight] = useState(1328);
  const [imageAspectRatioLocked, setImageAspectRatioLocked] = useState(true);
  const [isImageSettingsOpen, setIsImageSettingsOpen] = useState(false);
  const [uploadedImages, setUploadedImages] = useState<string[]>([]);

  const [musicSmartDuration, setMusicSmartDuration] = useState(true);
  const [musicDuration, setMusicDuration] = useState(120);
  const [selectedVoice, setSelectedVoice] = useState('zh_male_1');
  const [activeVoiceCategory, setActiveVoiceCategory] = useState('all');
  const [playingVoiceId, setPlayingVoiceId] = useState<string | null>(null);

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const creationWrapperRef = useRef<HTMLDivElement>(null);
  const modelWrapperRef = useRef<HTMLDivElement>(null);
  const videoSettingsRef = useRef<HTMLDivElement>(null);
  const musicSettingsRef = useRef<HTMLDivElement>(null);
  const voiceSettingsRef = useRef<HTMLDivElement>(null);
  const imageSettingsRef = useRef<HTMLDivElement>(null);
  
  const creationDropdownRef = useRef<HTMLDivElement>(null);
  const modelDropdownRef = useRef<HTMLDivElement>(null);
  const videoSettingsDropdownRef = useRef<HTMLDivElement>(null);
  const musicSettingsDropdownRef = useRef<HTMLDivElement>(null);
  const voiceSettingsDropdownRef = useRef<HTMLDivElement>(null);
  const imageSettingsDropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Node;
      
      const clickInCreationBtn = creationWrapperRef.current?.contains(target);
      const clickInCreationDropdown = creationDropdownRef.current?.contains(target);
      if (!clickInCreationBtn && !clickInCreationDropdown) {
        setIsCreationMenuOpen(false);
      }
      
      const clickInModelBtn = modelWrapperRef.current?.contains(target);
      const clickInModelDropdown = modelDropdownRef.current?.contains(target);
      if (!clickInModelBtn && !clickInModelDropdown) {
        setIsModelMenuOpen(false);
      }
      
      const clickInVideoBtn = videoSettingsRef.current?.contains(target);
      const clickInVideoDropdown = videoSettingsDropdownRef.current?.contains(target);
      if (!clickInVideoBtn && !clickInVideoDropdown) {
        setIsVideoSettingsOpen(false);
      }
      
      const clickInMusicBtn = musicSettingsRef.current?.contains(target);
      const clickInMusicDropdown = musicSettingsDropdownRef.current?.contains(target);
      if (!clickInMusicBtn && !clickInMusicDropdown) {
        setIsMusicSettingsOpen(false);
      }
      
      const clickInVoiceBtn = voiceSettingsRef.current?.contains(target);
      const clickInVoiceDropdown = voiceSettingsDropdownRef.current?.contains(target);
      if (!clickInVoiceBtn && !clickInVoiceDropdown) {
        setIsVoiceSettingsOpen(false);
      }

      const clickInImageBtn = imageSettingsRef.current?.contains(target);
      const clickInImageDropdown = imageSettingsDropdownRef.current?.contains(target);
      if (!clickInImageBtn && !clickInImageDropdown) {
        setIsImageSettingsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const [agentModelTab, setAgentModelTab] = useState<'image' | 'video'>('image');
  const [agentAutoMatch, setAgentAutoMatch] = useState(true);
  const [agentSelectedModels, setAgentSelectedModels] = useState<string[]>(['img_5_lite', 'img_4_7']);

  const getDropdownPlacement = (btnRef: React.RefObject<HTMLDivElement | null>, dropdownHeight: number) => {
    if (!btnRef.current) return 'top';
    const rect = btnRef.current.getBoundingClientRect();
    if (window.innerHeight - rect.bottom > dropdownHeight + 20) {
      return 'bottom';
    }
    return 'top';
  };

  const toggleCreationMenu = () => {
    if (!isCreationMenuOpen) {
      setCreationMenuPlacement(getDropdownPlacement(creationWrapperRef, 300));
    }
    setIsCreationMenuOpen(!isCreationMenuOpen);
    setIsModelMenuOpen(false);
    setIsVideoSettingsOpen(false);
    setIsMusicSettingsOpen(false);
    setIsVoiceSettingsOpen(false);
  };

  const toggleModelMenu = () => {
    if (!isModelMenuOpen) {
      setModelMenuPlacement(getDropdownPlacement(modelWrapperRef, 350));
    }
    setIsModelMenuOpen(!isModelMenuOpen);
    setIsCreationMenuOpen(false);
    setIsVideoSettingsOpen(false);
    setIsMusicSettingsOpen(false);
    setIsVoiceSettingsOpen(false);
  };
  
  const toggleVideoSettings = () => {
    if (!isVideoSettingsOpen) {
      setVideoSettingsPlacement(getDropdownPlacement(videoSettingsRef, 400));
    }
    setIsVideoSettingsOpen(!isVideoSettingsOpen);
    setIsCreationMenuOpen(false);
    setIsModelMenuOpen(false);
    setIsMusicSettingsOpen(false);
    setIsVoiceSettingsOpen(false);
  };
  
  const toggleMusicSettings = () => {
    if (!isMusicSettingsOpen) {
      setMusicSettingsPlacement(getDropdownPlacement(musicSettingsRef, 150));
    }
    setIsMusicSettingsOpen(!isMusicSettingsOpen);
    setIsCreationMenuOpen(false);
    setIsModelMenuOpen(false);
    setIsVideoSettingsOpen(false);
    setIsVoiceSettingsOpen(false);
  };
  
  const toggleVoiceSettings = () => {
    if (!isVoiceSettingsOpen) {
      setVoiceSettingsPlacement(getDropdownPlacement(voiceSettingsRef, 250));
    }
    setIsVoiceSettingsOpen(!isVoiceSettingsOpen);
    setIsCreationMenuOpen(false);
    setIsModelMenuOpen(false);
    setIsVideoSettingsOpen(false);
    setIsMusicSettingsOpen(false);
  };

  const toggleImageSettings = () => {
    if (!isImageSettingsOpen) {
      setImageSettingsPlacement(getDropdownPlacement(imageSettingsRef, 400));
    }
    setIsImageSettingsOpen(!isImageSettingsOpen);
    setIsCreationMenuOpen(false);
    setIsModelMenuOpen(false);
    setIsVideoSettingsOpen(false);
    setIsMusicSettingsOpen(false);
    setIsVoiceSettingsOpen(false);
  };

  const isVideo = creationType === 'video';
  const isImage = creationType === 'image';
  const isAgent = creationType === 'agent';
  const isMusic = creationType === 'music';
  const isVoice = creationType === 'voice';
  const isDigitalHuman = creationType === 'digital_human';
  const isAction = creationType === 'action';

  const selectedModelId = isDigitalHuman ? selectedAvatarModel : isAction ? selectedActionModel : isVoice ? selectedVoiceModel : isMusic ? selectedMusicModel : isVideo ? selectedVideoModel : selectedImageModel;

  const onSettingsChangeRef = useRef(onSettingsChange);
  useEffect(() => {
    onSettingsChangeRef.current = onSettingsChange;
  }, [onSettingsChange]);

  const lastSettingsRef = useRef<string>('');

  // Real-time synchronization of state settings with the parent component
  useEffect(() => {
    const nextSettings = {
      model: selectedModelId,
      ratio: isVideo ? videoRatio : (isImage ? imageRatio : '1:1'),
      resolution: isVideo ? videoResolution : (isImage ? imageResolution : undefined),
      imageWidth: isImage ? imageWidth : undefined,
      imageHeight: isImage ? imageHeight : undefined,
      duration: isVideo ? videoDuration : (isMusic ? musicDuration : undefined),
      videoMode: isVideo ? videoSettingsMode : undefined,
      count: isVideo ? Number(videoCount) : undefined
    };

    const nextSettingsStr = JSON.stringify(nextSettings);
    if (lastSettingsRef.current !== nextSettingsStr) {
      lastSettingsRef.current = nextSettingsStr;
      if (onSettingsChangeRef.current) {
        onSettingsChangeRef.current(nextSettings);
      }
    }
  }, [
    selectedModelId,
    videoRatio,
    videoResolution,
    videoDuration,
    musicDuration,
    videoSettingsMode,
    videoCount,
    imageRatio,
    imageResolution,
    imageWidth,
    imageHeight,
    isVideo,
    isImage,
    isMusic,
    isVoice,
    isDigitalHuman,
    isAction
  ]);

  // Sync value changes from parent if any
  useEffect(() => {
    if (defaultValue !== undefined) {
      setInput(defaultValue);
    }
  }, [defaultValue]);

  useEffect(() => {
    if (initialMode !== undefined) {
      setCreationType(initialMode);
    }
  }, [initialMode]);

  return {
    input, setInput,
    creationType, setCreationType,
    // Unified model catalog lists (static baseline + remote sync layer)
    imageModels: imageCatalog.models,
    videoModels: videoCatalog.models,
    musicModels: musicCatalog.models,
    voiceModels: voiceCatalog.models,
    avatarModels: digitalHumanCatalog.models,
    actionModels: actionCatalog.models,
    selectedImageModel, setSelectedImageModel,
    selectedVideoModel, setSelectedVideoModel,
    selectedMusicModel, setSelectedMusicModel,
    selectedVoiceModel, setSelectedVoiceModel,
    selectedAvatarModel, setSelectedAvatarModel,
    selectedActionModel, setSelectedActionModel,
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
    selectedModelId
  };
}
