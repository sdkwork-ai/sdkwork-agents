import React, { useEffect, useState, useRef } from 'react';
import { Paperclip, Send, Loader2, X, Square, Plus, Mic, AudioLines, ArrowUp, ImageIcon, Globe, Telescope, Box } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { PlusMenuPopup } from './PlusMenuPopup';

interface ChatInputProps {
  input: string;
  setInput: (value: string) => void;
  selectedImages: string[];
  isGenerating: boolean;
  inputDisabled?: boolean;
  textareaRef: React.RefObject<HTMLTextAreaElement | null>;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  handleFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  handleRemoveImage: (index: number) => void;
  handleSend: () => void;
  handleStop?: () => void;
  handleKeyDown: (e: React.KeyboardEvent) => void;
}

export const ChatInput: React.FC<ChatInputProps> = ({
  input,
  setInput,
  selectedImages,
  isGenerating,
  inputDisabled = false,
  textareaRef,
  fileInputRef,
  handleFileChange,
  handleRemoveImage,
  handleSend,
  handleStop,
  handleKeyDown
}) => {
  const { t } = useTranslation('chat');
  type InputMode = 'image' | 'search' | 'research' | null;
  const [showMenu, setShowMenu] = useState(false);
  const [inputMode, setInputMode] = useState<InputMode>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const handleKeyDownWithMode = (e: React.KeyboardEvent) => {
    if (e.key === 'Backspace' && input === '') {
      setInputMode(null);
    }
    handleKeyDown(e);
  };

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);


  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      const scrollHeight = textareaRef.current.scrollHeight;
      textareaRef.current.style.height = Math.min(scrollHeight, 200) + 'px';
      textareaRef.current.style.overflowY = scrollHeight > 200 ? 'auto' : 'hidden';
    }
  }, [input, textareaRef]);

  return (
    <div className="absolute bottom-0 left-0 right-0 p-6 bg-gradient-to-t from-[#f5f5f5] dark:from-[#191919] via-[#f5f5f5]/90 dark:via-[#191919]/90 to-transparent pt-20 z-10 pointer-events-none">
      <div className="max-w-4xl mx-auto w-full bg-[#f4f4f4] dark:bg-[#2f2f2f] rounded-[24px] relative transition-all pointer-events-auto">
        <div className="relative flex flex-col focus-within:ring-0 transition-shadow">
          
          {/* Plus Menu Popup */}
          {showMenu && (
            <PlusMenuPopup
              menuRef={menuRef}
              setShowMenu={setShowMenu}
              setInputMode={setInputMode}
              fileInputRef={fileInputRef}
            />
          )}

          {selectedImages.length > 0 && (
            <div className="flex flex-wrap gap-2 p-3 border-b border-[#f0f0f0] dark:border-[#333]">
              {selectedImages.map((img, i) => (
                <div key={i} className="relative w-16 h-16 rounded-xl overflow-hidden group shadow-sm">
                  <img src={img} alt="preview" className="w-full h-full object-cover" />
                  <button
                    onClick={() => handleRemoveImage(i)}
                    className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity text-white backdrop-blur-sm"
                  >
                    <X size={16} />
                  </button>
                </div>
              ))}
            </div>
          )}

                    <div className="flex items-end gap-1 p-2 relative">
            <div className="flex items-center justify-center flex-shrink-0 w-10 h-10 mb-1 ml-1">
              <input
                type="file"
                ref={fileInputRef}
                onChange={handleFileChange}
                accept="image/*,video/*,audio/*,application/*,text/*"
                multiple
                className="hidden"
              />
              <button
                onClick={() => setShowMenu(!showMenu)}
                className="p-2 rounded-full text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-[#3d3d3d] transition-colors"
                title={t('attachFile')}
              >
                <Plus size={22} strokeWidth={2} />
              </button>
            </div>

            <div className="flex-1 min-h-[44px] flex items-center px-1 py-3 mb-0.5 gap-2 overflow-x-auto [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]">
              {inputMode === 'image' && (
                <div onClick={() => setInputMode(null)} className="cursor-pointer flex items-center gap-1.5 px-3 py-1.5 bg-blue-500/10 hover:bg-blue-500/20 transition-colors text-blue-500 dark:text-[#5e9bfa] rounded-[14px] text-[14px] font-medium shrink-0">
                  <ImageIcon size={16} /> 创建图片
                </div>
              )}
              {inputMode === 'search' && (
                <div onClick={() => setInputMode(null)} className="cursor-pointer flex items-center gap-1.5 px-3 py-1.5 bg-blue-500/10 hover:bg-blue-500/20 transition-colors text-blue-500 dark:text-[#5e9bfa] rounded-[14px] text-[14px] font-medium shrink-0">
                  <Globe size={16} /> 网页搜索
                </div>
              )}
              {inputMode === 'research' && (
                <div onClick={() => setInputMode(null)} className="cursor-pointer flex items-center gap-1.5 px-3 py-1.5 bg-blue-500/10 hover:bg-blue-500/20 transition-colors text-blue-500 dark:text-[#5e9bfa] rounded-[14px] text-[14px] font-medium shrink-0">
                  <Telescope size={16} /> 深度研究
                </div>
              )}
              <textarea
                ref={textareaRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDownWithMode}
                placeholder="有问题，尽管问"
                disabled={inputDisabled}
                className="w-full bg-transparent border-none focus:ring-0 text-[16px] resize-none text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 outline-none p-0 m-0 leading-relaxed max-h-[200px] disabled:cursor-not-allowed disabled:opacity-60"
                rows={1}
              />
            </div>
            
            <div className="flex items-center flex-shrink-0 gap-1.5 mb-1.5 mr-1.5">
              <button
                className="p-2 rounded-full text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-[#3d3d3d] transition-colors mr-1"
                title="Voice Input"
              >
                <Mic size={22} strokeWidth={2} />
              </button>

              {isGenerating ? (
                <button
                  onClick={handleStop}
                  className="w-[34px] h-[34px] flex items-center justify-center bg-black dark:bg-white text-white dark:text-black rounded-full shadow-sm hover:opacity-80 transition-all transform active:scale-95"
                  title={t('stopGenerating')}
                >
                  <Square size={14} fill="currentColor" />
                </button>
              ) : (
                <button
                  onClick={handleSend}
                  disabled={inputDisabled || (!input.trim() && selectedImages.length === 0)}
                  className={`w-[34px] h-[34px] flex items-center justify-center rounded-full shadow-sm transition-all transform active:scale-95 ${!inputDisabled && (input.trim() || selectedImages.length > 0) ? 'bg-black dark:bg-white text-white dark:text-black hover:opacity-80' : 'bg-transparent text-gray-400 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#3d3d3d] disabled:cursor-not-allowed disabled:opacity-60'}`}
                  title={t('sendMessage')}
                >
                  {input.trim() || selectedImages.length > 0 ? (
                    <ArrowUp size={20} strokeWidth={2.5} />
                  ) : (
                    <AudioLines size={18} strokeWidth={2} />
                  )}
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
      <div className="text-center mt-4 text-[11px] text-gray-400 dark:text-gray-500 font-medium pointer-events-auto">
        {t('disclaimer')}
      </div>
    </div>
  );
};
