import React from 'react';
import Editor from '@monaco-editor/react';
import { useTranslation } from 'react-i18next';
import { cn } from '@sdkwork/agents-pc-commons';
import { useTheme } from '@sdkwork/agents-pc-commons';

interface PPTEditorProps {
  content: string;
  setContent: (val: string) => void;
  layout: 'split' | 'edit' | 'preview';
}

export const PPTEditor: React.FC<PPTEditorProps> = ({ content, setContent, layout }) => {
  const { t } = useTranslation(['ppt']);
  const { resolvedTheme } = useTheme();
  const isDark = resolvedTheme === 'dark';

  if (layout !== 'split' && layout !== 'edit') return null;

  return (
    <div className={cn("h-full border-gray-200 dark:border-gray-800 flex flex-col", layout === 'split' ? "w-1/2 border-r" : "w-full")}>
      <div className="bg-gray-50 dark:bg-[#1a1a1a] px-4 py-2 text-xs font-mono text-gray-500 dark:text-gray-400 border-b border-gray-200 dark:border-gray-800">
        {t('ppt:markdownPrefix')}
      </div>
      <div className="flex-1 relative">
        <Editor
          height="100%"
          language="markdown"
          theme={isDark ? "vs-dark" : "vs"}
          value={content}
          onChange={(val) => setContent(val || '')}
          options={{
            minimap: { enabled: false },
            wordWrap: 'on',
            fontSize: 14,
            padding: { top: 16 },
          }}
        />
      </div>
    </div>
  );
};
