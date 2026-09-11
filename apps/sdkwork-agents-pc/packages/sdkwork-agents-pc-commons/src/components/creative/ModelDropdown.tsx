import React from 'react';
import { Box, Sparkles, Check, ShieldAlert } from 'lucide-react';
import { cn } from '../MarkdownRenderer';
import type { CreativeModelDefinition } from '../../creative-model-catalog';

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
  /** Agent multi-select catalogs, sourced from the unified model catalog service. */
  agentImageModels: CreativeModelDefinition[];
  agentVideoModels: CreativeModelDefinition[];
  selectedModelId: string;
  onSelectModel: (id: string) => void;
  currentModels: CreativeModelDefinition[];
  currentModel?: CreativeModelDefinition;
  modelMenuPlacement: 'top' | 'bottom';
  dropdownRef: React.RefObject<HTMLDivElement | null>;
  isVoice: boolean;
  isDigitalHuman?: boolean;
  isAction?: boolean;
}

const isSelectable = (model: CreativeModelDefinition) => model.lifecycle !== 'deprecated' && model.lifecycle !== 'retired';

export const ModelDropdown: React.FC<ModelDropdownProps> = ({
  isAgent,
  agentAutoMatch,
  setAgentAutoMatch,
  agentModelTab,
  setAgentModelTab,
  agentSelectedModels,
  setAgentSelectedModels,
  agentImageModels,
  agentVideoModels,
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
  const agentTabModels = agentModelTab === 'image' ? agentImageModels : agentVideoModels;
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
            <span className="text-[14px] font-medium text-zinc-800 dark:text-zinc-200">选择模型</span>
            <div className="flex items-center gap-2 cursor-pointer" onClick={() => {
              setAgentAutoMatch(!agentAutoMatch);
              if (!agentAutoMatch) {
                setAgentSelectedModels([
                  ...agentImageModels.filter(isSelectable).map(m => m.id),
                  ...agentVideoModels.filter(isSelectable).map(m => m.id),
                ]);
              }
            }}>
              <span className="text-[12px] text-zinc-500 dark:text-zinc-400">自动匹配模型</span>
              <div className={cn("w-8 h-4 rounded-full flex items-center px-[2px] transition-colors", agentAutoMatch ? "bg-cyan-500 justify-end" : "bg-zinc-600 justify-start")}>
                <div className="w-3.5 h-3.5 bg-white rounded-full shadow-sm"></div>
              </div>
            </div>
          </div>
          <div className="px-4 pb-2 shrink-0">
            <div className="flex bg-zinc-100 rounded-xl p-1 dark:bg-[#2a2a2a]">
              <button 
                onClick={() => setAgentModelTab('image')}
                className={cn("flex-1 py-1.5 text-[13px] font-medium rounded-lg transition-colors", agentModelTab === 'image' ? "bg-black/10 text-zinc-900 shadow-sm dark:bg-[#333333] dark:text-white" : "text-zinc-500 dark:text-zinc-400 hover:text-zinc-800 dark:hover:text-zinc-200")}
              >
                图片模型
              </button>
              <button 
                onClick={() => setAgentModelTab('video')}
                className={cn("flex-1 py-1.5 text-[13px] font-medium rounded-lg transition-colors", agentModelTab === 'video' ? "bg-black/10 text-zinc-900 shadow-sm dark:bg-[#333333] dark:text-white" : "text-zinc-500 dark:text-zinc-400 hover:text-zinc-800 dark:hover:text-zinc-200")}
              >
                视频模型
              </button>
            </div>
          </div>
          <div className="px-4 py-2 text-[12px] text-zinc-500 shrink-0">
            {agentModelTab === 'image' ? '图片生成' : '视频生成'}
          </div>
          <div className="flex-1 overflow-y-auto custom-scrollbar">
            {agentTabModels.map(model => {
              const isSelected = agentSelectedModels.includes(model.id);
              const selectable = isSelectable(model);
              return (
                <button
                  key={model.id}
                  disabled={!selectable}
                  onClick={() => {
                    if (!selectable) return;
                    setAgentAutoMatch(false);
                    if (isSelected) {
                      setAgentSelectedModels(agentSelectedModels.filter(id => id !== model.id));
                    } else {
                      setAgentSelectedModels([...agentSelectedModels, model.id]);
                    }
                  }}
                  className={cn(
                    "w-full flex items-center justify-between px-4 py-3 hover:bg-black/5 dark:hover:bg-[#2a2a2a] transition-colors text-left",
                    isSelected ? "bg-black/5 dark:bg-[#2f2f2f]" : "",
                    !selectable && "opacity-50 cursor-not-allowed hover:bg-transparent dark:hover:bg-transparent"
                  )}
                >
                  <span className={cn("text-[14px] flex items-center gap-1.5", isSelected ? "text-zinc-900 font-medium dark:text-white" : "text-zinc-700 dark:text-zinc-300")}>
                    {model.label}
                    {model.lifecycle === 'deprecated' && (
                      <span className="bg-amber-500/10 border border-amber-500/30 text-amber-500 text-[9px] font-bold px-1 py-0.5 rounded shrink-0">
                        已弃用
                      </span>
                    )}
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
          <div className="px-4 py-2 text-zinc-500 dark:text-zinc-400 text-[12px] mb-1 shrink-0">
            选择模型: {currentModel?.label || ''} {isDigitalHuman ? 'by OmniHuman 1.5' : isAction ? 'by DreamActor M2.0' : 'by seed'}
          </div>
          <div className="flex-1 overflow-y-auto custom-scrollbar">
            {currentModels.map(model => {
              const selectable = isSelectable(model);
              return (
                <button
                  key={model.id}
                  disabled={!selectable}
                  onClick={() => { 
                    if (!selectable) return;
                    onSelectModel(model.id);
                  }}
                  className={cn(
                    "w-full flex gap-4 px-4 py-3 hover:bg-black/5 dark:hover:bg-[#2a2a2a] transition-colors text-left",
                    (model.image || model.badge) ? "items-center" : "items-start",
                    selectedModelId === model.id ? "bg-black/5 dark:bg-[#2f2f2f]" : "",
                    !selectable && "opacity-50 cursor-not-allowed hover:bg-transparent dark:hover:bg-transparent"
                  )}
                >
                  {model.image ? (
                    <div className="shrink-0 w-11 h-11 rounded-lg overflow-hidden border border-black/10 dark:border-white/10 shadow-md">
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
                    <div className={cn("mt-0.5 shrink-0 w-7 h-7 rounded-md flex items-center justify-center", selectedModelId === model.id ? "bg-black/10 text-zinc-900 dark:bg-[#333333] dark:text-white" : "text-zinc-500 dark:text-zinc-400")}>
                      <ModelIcon />
                    </div>
                  )}

                  <div className="flex flex-col flex-1 min-w-0">
                    <div className="flex items-center gap-1.5 mb-1 flex-wrap">
                      <span className={cn("text-[14px] font-semibold", selectedModelId === model.id ? "text-zinc-900 dark:text-white" : "text-zinc-800 dark:text-zinc-200")}>
                        {model.label}
                      </span>
                      {model.spark && <Sparkles size={12} className="text-cyan-400 fill-cyan-400 shrink-0" />}
                      {model.isNew && (
                        <span className="bg-cyan-500/10 border border-cyan-400/20 text-cyan-400 text-[9px] font-bold px-1 py-0.5 rounded scale-90 origin-left shrink-0">
                          New
                        </span>
                      )}
                      {model.lifecycle === 'deprecated' && (
                        <span className="bg-amber-500/10 border border-amber-500/30 text-amber-500 text-[9px] font-bold px-1 py-0.5 rounded scale-90 origin-left shrink-0 flex items-center gap-0.5">
                          <ShieldAlert size={9} strokeWidth={2.5} />
                          已弃用
                        </span>
                      )}
                    </div>
                    <span className="text-[12px] text-zinc-500 dark:text-zinc-400 leading-normal line-clamp-2">
                      {model.lifecycle === 'deprecated' && model.replacementModelId
                        ? `已弃用，推荐改用 ${model.replacementModelId}。${model.desc}`
                        : model.desc}
                    </span>
                  </div>
                  {selectedModelId === model.id && <Check size={16} className="text-zinc-900 dark:text-white shrink-0 self-center" />}
                </button>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
};
