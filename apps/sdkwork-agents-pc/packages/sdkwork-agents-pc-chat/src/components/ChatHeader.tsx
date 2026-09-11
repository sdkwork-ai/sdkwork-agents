import React from 'react';
import { ArrowLeft, Cable, PanelLeftOpen, Settings } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ModelPicker } from '@sdkwork/models-pc-picker/model-picker';
import type {
  ModelsPickerGroup,
  ModelsPickerOption,
} from '@sdkwork/models-pc-picker/model-picker-types';
import './modelPickerTheme.css';

interface ChatHeaderProps {
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
  modelGroups: ModelsPickerGroup[];
  selectedModelId: string;
  onSelectModel: (modelId: string) => void;
  fallbackModel: ModelsPickerOption;
  isModelSelectorOpen: boolean;
  setIsModelSelectorOpen: (v: boolean) => void;
  onOpenSettings: () => void;
  onManageCustomProvider: () => void;
  agentTitle?: string;
  onBack?: () => void;
  showModelPicker?: boolean;
}

export const ChatHeader: React.FC<ChatHeaderProps> = ({
  isSidebarOpen,
  toggleSidebar,
  modelGroups,
  selectedModelId,
  onSelectModel,
  fallbackModel,
  isModelSelectorOpen,
  setIsModelSelectorOpen,
  onOpenSettings,
  onManageCustomProvider,
  agentTitle,
  onBack,
  showModelPicker = true,
}) => {
  const { t } = useTranslation('common');

  return (
    <header className="absolute top-0 left-0 right-0 h-14 flex items-center justify-between px-4 shrink-0 z-20 bg-transparent pointer-events-none">
      <div className="flex items-center gap-1 pointer-events-auto mt-2">
        {onBack ? (
          <button
            type="button"
            onClick={onBack}
            className="p-1.5 mr-1 text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-md transition-colors hover:bg-gray-200/50 dark:hover:bg-[#2f2f2f]/50"
            title="Back"
            aria-label="Back"
          >
            <ArrowLeft size={18} />
          </button>
        ) : null}
        {!isSidebarOpen && !onBack ? (
          <button
            onClick={toggleSidebar}
            className="p-1.5 mr-1 text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-md transition-colors hover:bg-gray-200/50 dark:hover:bg-[#2f2f2f]/50"
            title={t('openSidebar')}
          >
            <PanelLeftOpen size={18} />
          </button>
        ) : null}
        {agentTitle ? (
          <div className="max-w-[min(40vw,20rem)] truncate px-1 text-sm font-semibold text-gray-800 dark:text-gray-100">
            {agentTitle}
          </div>
        ) : null}
        {showModelPicker ? (
          <>
            <ModelPicker
              bucket="llms"
              modelGroups={modelGroups}
              selectedModelId={selectedModelId}
              onSelectModel={onSelectModel}
              showModelMenu={isModelSelectorOpen}
              setShowModelMenu={setIsModelSelectorOpen}
              fallback={fallbackModel}
              variant="flat"
              compact
              showModelDescription
            />
            <button
              onClick={onManageCustomProvider}
              className="p-1.5 text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-md transition-colors hover:bg-gray-200/50 dark:hover:bg-[#2f2f2f]/50"
              title={t('customProvider')}
            >
              <Cable size={16} />
            </button>
          </>
        ) : null}
      </div>
      <div className="flex items-center gap-2 pointer-events-auto mt-2">
        <button 
          onClick={onOpenSettings}
          className="text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors p-1.5 rounded-lg hover:bg-gray-200/50 dark:hover:bg-[#2f2f2f]/50"
          title={t('settings')}
        >
          <Settings size={18} />
        </button>
      </div>
    </header>
  );
}
