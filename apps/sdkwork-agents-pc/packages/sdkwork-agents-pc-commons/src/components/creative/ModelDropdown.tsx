import React from 'react';
import { Box, Sparkles, Check } from 'lucide-react';
import { cn } from '../MarkdownRenderer';

export const IMAGE_MODELS = [
  { id: '5.0-lite', label: '图片 5.0 Lite', desc: '指令响应更精准，生成效果更智能', spark: false },
  { id: '4.7', label: '图片 4.7', desc: '画质全面优化，指令响应能力再次提升', spark: false },
  { id: '4.6', label: '图片 4.6', desc: '人像一致性保持更好，性价比更高', spark: false },
  { id: '4.5', label: '图片 4.5', desc: '强化一致性、风格与图文响应', spark: false },
  { id: '4.1', label: '图片 4.1', desc: '更专业的创意、美学和一致性保持', spark: false },
];

export const VIDEO_MODELS = [
  { id: '2.0-mini', label: '即梦 Seedance 2.0 mini', desc: '极具性价比，相近的体验，比Fast更快的推理速度', spark: true },
  { id: '2.0-fast-vip', label: '即梦 Seedance 2.0 Fast VIP', desc: '极速推理，会员专属通道，音视文图均可参考（暂不支持...', spark: true },
  { id: '2.0-vip', label: '即梦 Seedance 2.0 VIP', desc: '全模态能力，会员专属通道，音视文图均可参考（暂不支持...', spark: true },
  { id: '2.0-fast', label: '即梦 Seedance 2.0 Fast', desc: '高性价比，音视文图均可参考（暂不支持真人人脸）', spark: false },
  { id: '2.0', label: '即梦 Seedance 2.0', desc: '全能王者，音视文图均可参考（暂不支持真人人脸）', spark: false },
];

export const MUSIC_MODELS = [
  { id: 'music_pro', label: '即梦音乐 Pro', desc: '根据文本提示词和参考音频生成高质量音乐', spark: true },
  { id: 'music_1.0', label: '即梦音乐 1.0', desc: '强大的音乐生成能力，支持多种曲风', spark: false },
  { id: 'suno_v3.5', label: 'Suno v3.5', desc: '生成流派融合的音乐及人声演唱', spark: false },
  { id: 'udio', label: 'Udio', desc: '惊艳的高保真人声与复杂的音乐结构', spark: false },
  { id: 'stable_audio', label: 'Stable Audio', desc: '高质量的纯音乐与环境音效生成', spark: false },
];

export const VOICE_MODELS = [
  { id: 'voice_pro', label: '即梦配音 Pro', desc: '根据文本生成高质量语音', spark: true },
  { id: 'voice_1.0', label: '即梦配音 1.0', desc: '支持多种音色与情绪控制', spark: false },
];

export const AVATAR_MODELS = [
  { 
    id: 'master_mode', 
    label: '大师模式', 
    desc: '电影级的表演效果', 
    spark: true, 
    image: 'https://images.unsplash.com/photo-1544005313-94ddf0286df2?auto=format&fit=crop&w=120&h=120&q=80' 
  },
  { 
    id: 'fast_mode', 
    label: '快速模式', 
    desc: '更低成本，快速生成', 
    spark: false, 
    image: 'https://images.unsplash.com/photo-1517841905240-472988babdf9?auto=format&fit=crop&w=120&h=120&q=80' 
  },
  { 
    id: 'basic_mode', 
    label: '基础模式', 
    desc: '仅仅修改人物口型。适合演讲、对白', 
    spark: false, 
    image: 'https://images.unsplash.com/photo-1539571696357-5a69c17a67c6?auto=format&fit=crop&w=120&h=120&q=80' 
  }
];

export const ACTION_MODELS = [
  { 
    id: 'master', 
    label: '大师', 
    desc: '效果最佳，画质超清', 
    spark: true, 
    badge: '1.5', 
    subBadge: 'PRO', 
    isNew: true 
  },
  { 
    id: 'vivid', 
    label: '生动', 
    desc: '不限画幅，动效更真', 
    spark: false, 
    badge: '2.0', 
    isNew: true 
  },
  { 
    id: 'fast', 
    label: '快速', 
    desc: '更快生成，成本更低', 
    spark: false, 
    badge: '2.0', 
    isNew: true 
  }
];

const ModelIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
    <path d="M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22ZM12 10.5L8.5 7L12 15L15.5 7L12 10.5Z" />
  </svg>
);

interface ModelDropdownProps {
  isAgent: boolean;
  agentAutoMatch: boolean;
  setAgentAutoMatch: (val: boolean) => void;
  agentModelTab: 'image' | 'video';
  setAgentModelTab: (tab: 'image' | 'video') => void;
  agentSelectedModels: string[];
  setAgentSelectedModels: (models: string[]) => void;
  selectedModelId: string;
  onSelectModel: (id: string) => void;
  currentModels: any[];
  currentModel: any;
  modelMenuPlacement: 'top' | 'bottom';
  dropdownRef: React.RefObject<HTMLDivElement | null>;
  isVoice: boolean;
  isDigitalHuman?: boolean;
  isAction?: boolean;
}

export const ModelDropdown: React.FC<ModelDropdownProps> = ({
  isAgent,
  agentAutoMatch,
  setAgentAutoMatch,
  agentModelTab,
  setAgentModelTab,
  agentSelectedModels,
  setAgentSelectedModels,
  selectedModelId,
  onSelectModel,
  currentModels,
  currentModel,
  modelMenuPlacement,
  dropdownRef,
  isVoice,
  isDigitalHuman = false,
  isAction = false
}) => {
  return (
    <div 
      ref={dropdownRef}
      className={cn(
        "absolute right-0 bg-white border border-black/10 rounded-xl shadow-xl py-2 z-50 animate-in fade-in zoom-in-95 duration-100 flex flex-col w-[calc(100vw-32px)] max-w-[380px] sm:w-[380px] dark:bg-[#1e1e1e] dark:border-white/10 dark:shadow-2xl",
        modelMenuPlacement === 'top' ? "bottom-full mb-2" : "top-full mt-2",
        isAgent ? "max-h-[70vh] sm:w-[420px] max-w-[420px]" : "max-h-[60vh]"
      )}
    >
      {isAgent ? (
        <div className="flex flex-col h-full">
          <div className="flex items-center justify-between px-4 py-3 shrink-0">
            <span className="text-[14px] font-medium text-zinc-200">选择模型</span>
            <div className="flex items-center gap-2 cursor-pointer" onClick={() => {
              setAgentAutoMatch(!agentAutoMatch);
              if (!agentAutoMatch) {
                setAgentSelectedModels([...IMAGE_MODELS.map(m => m.id), ...VIDEO_MODELS.map(m => m.id)]);
              }
            }}>
              <span className="text-[12px] text-zinc-400">自动匹配模型</span>
              <div className={cn("w-8 h-4 rounded-full flex items-center px-[2px] transition-colors", agentAutoMatch ? "bg-cyan-500 justify-end" : "bg-zinc-600 justify-start")}>
                <div className="w-3.5 h-3.5 bg-white rounded-full shadow-sm"></div>
              </div>
            </div>
          </div>
          <div className="px-4 pb-2 shrink-0">
            <div className="flex bg-zinc-100 rounded-xl p-1 dark:bg-[#2a2a2a]">
              <button 
                onClick={() => setAgentModelTab('image')}
                className={cn("flex-1 py-1.5 text-[13px] font-medium rounded-lg transition-colors", agentModelTab === 'image' ? "bg-white/10 text-white shadow-sm" : "text-zinc-400 hover:text-zinc-200")}
              >
                图片模型
              </button>
              <button 
                onClick={() => setAgentModelTab('video')}
                className={cn("flex-1 py-1.5 text-[13px] font-medium rounded-lg transition-colors", agentModelTab === 'video' ? "bg-white/10 text-white shadow-sm" : "text-zinc-400 hover:text-zinc-200")}
              >
                视频模型
              </button>
            </div>
          </div>
          <div className="px-4 py-2 text-[12px] text-zinc-500 shrink-0">
            {agentModelTab === 'image' ? '图片生成' : '视频生成'}
          </div>
          <div className="flex-1 overflow-y-auto custom-scrollbar">
            {(agentModelTab === 'image' ? IMAGE_MODELS : VIDEO_MODELS).map(model => {
              const isSelected = agentSelectedModels.includes(model.id);
              return (
                <button
                  key={model.id}
                  onClick={() => {
                    setAgentAutoMatch(false);
                    if (isSelected) {
                      setAgentSelectedModels(agentSelectedModels.filter(id => id !== model.id));
                    } else {
                      setAgentSelectedModels([...agentSelectedModels, model.id]);
                    }
                  }}
                  className={cn(
                    "w-full flex items-center justify-between px-4 py-3 hover:bg-white/5 transition-colors text-left",
                    isSelected ? "bg-white/5" : ""
                  )}
                >
                  <span className={cn("text-[14px]", isSelected ? "text-white font-medium" : "text-zinc-300")}>
                    {model.label}
                  </span>
                  <div className={cn("w-4 h-4 rounded-[4px] border flex items-center justify-center transition-colors", isSelected ? "bg-cyan-500 border-cyan-500 text-[#1e1e1e]" : "border-zinc-500")}>
                    {isSelected && <Check size={12} strokeWidth={3} />}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      ) : (
        <>
          <div className="px-4 py-2 text-zinc-400 text-[12px] mb-1 shrink-0">
            选择模型: {currentModel?.label || ''} {isDigitalHuman ? 'by OmniHuman 1.5' : isAction ? 'by DreamActor M2.0' : 'by seed'}
          </div>
          <div className="flex-1 overflow-y-auto custom-scrollbar">
            {currentModels.map(model => (
              <button
                key={model.id}
                onClick={() => { 
                  onSelectModel(model.id);
                }}
                className={cn(
                  "w-full flex gap-4 px-4 py-3 hover:bg-white/5 transition-colors text-left",
                  (model.image || model.badge) ? "items-center" : "items-start",
                  selectedModelId === model.id ? "bg-white/5" : ""
                )}
              >
                {model.image ? (
                  <div className="shrink-0 w-11 h-11 rounded-lg overflow-hidden border border-white/10 shadow-md">
                    <img src={model.image} className="w-full h-full object-cover" alt="" referrerPolicy="no-referrer" />
                  </div>
                ) : model.badge ? (
                  <div className="shrink-0 w-11 h-11 flex flex-col items-center justify-center rounded-xl bg-gradient-to-b from-[#1b4383] to-[#0c1f44] border border-blue-400/30 text-white shadow-lg relative select-none">
                    <div className="text-[14px] font-black tracking-tight leading-none">{model.badge}</div>
                    {model.subBadge && (
                      <div className="text-[7px] font-extrabold text-cyan-300 tracking-wider mt-0.5 scale-90">{model.subBadge}</div>
                    )}
                  </div>
                ) : (
                  <div className={cn("mt-0.5 shrink-0 w-7 h-7 rounded-md flex items-center justify-center", selectedModelId === model.id ? "bg-white/10 text-white" : "text-zinc-400")}>
                    <ModelIcon />
                  </div>
                )}

                <div className="flex flex-col flex-1 min-w-0">
                  <div className="flex items-center gap-1.5 mb-1 flex-wrap">
                    <span className={cn("text-[14px] font-semibold", selectedModelId === model.id ? "text-white" : "text-zinc-200")}>
                      {model.label}
                    </span>
                    {model.spark && <Sparkles size={12} className="text-cyan-400 fill-cyan-400 shrink-0" />}
                    {model.isNew && (
                      <span className="bg-cyan-500/10 border border-cyan-400/20 text-cyan-400 text-[9px] font-bold px-1 py-0.5 rounded scale-90 origin-left shrink-0">
                        New
                      </span>
                    )}
                  </div>
                  <span className="text-[12px] text-zinc-400 leading-normal line-clamp-2">{model.desc}</span>
                </div>
                {selectedModelId === model.id && <Check size={16} className="text-white shrink-0 self-center" />}
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
};
