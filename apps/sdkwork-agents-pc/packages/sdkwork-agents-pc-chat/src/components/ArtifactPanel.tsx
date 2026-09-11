import React from 'react';
import { Code, Play, Download, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import Editor from '@monaco-editor/react';
import { cn, MarkdownRenderer } from '@sdkwork/agents-pc-commons';
import { useTheme } from '@sdkwork/agents-pc-commons';

interface ArtifactPanelProps {
  artifact: { language: string, code: string, mode: 'preview' | 'code' } | null;
  onClose: () => void;
  onModeChange: (mode: 'preview' | 'code') => void;
  onCodeChange: (code: string) => void;
}

export const ArtifactPanel: React.FC<ArtifactPanelProps> = ({ artifact, onClose, onModeChange, onCodeChange }) => {
  const { t } = useTranslation('chat');
  const { t: tCommon } = useTranslation('common');
  const { resolvedTheme } = useTheme();
  const isDark = resolvedTheme === 'dark';

  if (!artifact) return null;

  return (
    <div className="flex-1 flex flex-col min-w-0 h-full bg-[#f5f5f5] dark:bg-[#191919] animate-in slide-in-from-right-4 duration-300">
      <div className="h-14 border-b border-[#d9d9d9] dark:border-[#1a1a1a] flex items-center justify-between px-4 shrink-0 bg-[#ebebeb] dark:bg-[#202020]">
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300 capitalize">{artifact.language} {tCommon('component')}</span>
        </div>
        <div className="flex items-center gap-3">
          {['html', 'svg', 'xml', 'md', 'markdown'].includes(artifact.language) && (
            <div className="flex bg-[#d9d9d9] dark:bg-[#151515] rounded-lg p-1 border border-[#ccc] dark:border-[#2a2a2a]">
              <button
                onClick={() => onModeChange('code')}
                className={cn(
                  "flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors text-xs font-medium",
                  artifact.mode === 'code' ? "bg-white dark:bg-[#333] text-[#1890ff] shadow-sm" : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                )}
              >
                <Code size={14} />
                {tCommon('code')}
              </button>
              <button
                onClick={() => onModeChange('preview')}
                className={cn(
                  "flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors text-xs font-medium",
                  artifact.mode === 'preview' ? "bg-white dark:bg-[#333] text-[#1890ff] shadow-sm" : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                )}
              >
                <Play size={14} />
                {tCommon('preview')}
              </button>
            </div>
          )}
          <button 
            onClick={() => {
              const blob = new Blob([artifact.code], { type: 'text/plain' });
              const url = URL.createObjectURL(blob);
              const a = document.createElement('a');
              a.href = url;
              a.download = `artifact.${artifact.language}`;
              a.click();
              URL.revokeObjectURL(url);
            }}
            className="text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors p-1.5 rounded-lg hover:bg-gray-200 dark:hover:bg-[#333]"
            title={tCommon('download')}
          >
            <Download size={16} />
          </button>
          <div className="w-px h-4 bg-gray-300 dark:bg-gray-800 mx-1"></div>
          <button 
            onClick={onClose}
            className="text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors p-1.5 rounded-lg hover:bg-gray-200 dark:hover:bg-[#333]"
          >
            <X size={18} />
          </button>
        </div>
      </div>
      
      <div className="flex-1 overflow-hidden relative">
        {artifact.mode === 'preview' ? (
          ['md', 'markdown'].includes(artifact.language) ? (
            <div className="w-full h-full p-6 overflow-y-auto bg-white dark:bg-[#1e1e1e]">
              <MarkdownRenderer content={artifact.code} />
            </div>
          ) : (
            <iframe
              srcDoc={artifact.code}
              className="w-full h-full bg-white border-none"
              sandbox="allow-scripts allow-forms allow-same-origin"
              title={tCommon('artifactPreview')}
            />
          )
        ) : (
          <Editor
            height="100%"
            language={artifact.language === 'html' ? 'html' : artifact.language === 'javascript' ? 'javascript' : artifact.language.includes('json') ? 'json' : artifact.language.includes('ts') ? 'typescript' : artifact.language.includes('md') ? 'markdown' : artifact.language}
            theme={isDark ? 'vs-dark' : 'vs'}
            value={artifact.code}
            options={{
              minimap: { enabled: false },
              fontSize: 13,
              wordWrap: 'on',
              padding: { top: 16 },
              scrollBeyondLastLine: false,
              smoothScrolling: true,
              cursorBlinking: 'smooth',
              cursorSmoothCaretAnimation: 'on',
            }}
            onChange={(val) => {
              if (val !== undefined) onCodeChange(val);
            }}
          />
        )}
      </div>
    </div>
  );
};
