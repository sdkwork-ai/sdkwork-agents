import React, { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import { PrismLight as SyntaxHighlighter } from 'react-syntax-highlighter';
import bash from 'react-syntax-highlighter/dist/esm/languages/prism/bash';
import css from 'react-syntax-highlighter/dist/esm/languages/prism/css';
import javascript from 'react-syntax-highlighter/dist/esm/languages/prism/javascript';
import json from 'react-syntax-highlighter/dist/esm/languages/prism/json';
import jsx from 'react-syntax-highlighter/dist/esm/languages/prism/jsx';
import markdown from 'react-syntax-highlighter/dist/esm/languages/prism/markdown';
import markup from 'react-syntax-highlighter/dist/esm/languages/prism/markup';
import python from 'react-syntax-highlighter/dist/esm/languages/prism/python';
import rust from 'react-syntax-highlighter/dist/esm/languages/prism/rust';
import typescript from 'react-syntax-highlighter/dist/esm/languages/prism/typescript';
import tsx from 'react-syntax-highlighter/dist/esm/languages/prism/tsx';
import vscDarkPlus from 'react-syntax-highlighter/dist/esm/styles/prism/vsc-dark-plus';
import { Play, Code, Copy, Check, ExternalLink, PanelRightOpen } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { cn } from './classNames';

SyntaxHighlighter.registerLanguage('bash', bash);
SyntaxHighlighter.registerLanguage('css', css);
SyntaxHighlighter.registerLanguage('html', markup);
SyntaxHighlighter.registerLanguage('javascript', javascript);
SyntaxHighlighter.registerLanguage('js', javascript);
SyntaxHighlighter.registerLanguage('json', json);
SyntaxHighlighter.registerLanguage('jsx', jsx);
SyntaxHighlighter.registerLanguage('markdown', markdown);
SyntaxHighlighter.registerLanguage('md', markdown);
SyntaxHighlighter.registerLanguage('python', python);
SyntaxHighlighter.registerLanguage('rust', rust);
SyntaxHighlighter.registerLanguage('typescript', typescript);
SyntaxHighlighter.registerLanguage('ts', typescript);
SyntaxHighlighter.registerLanguage('tsx', tsx);

interface CodeBlockProps {
  language: string;
  value: string;
  onOpenArtifact?: (language: string, code: string, mode?: 'preview' | 'code') => void;
}

const CodeBlock: React.FC<CodeBlockProps> = ({ language, value, onOpenArtifact }) => {
  const [copied, setCopied] = useState(false);
  const { t } = useTranslation('common');

  // If the language is html or contains xml/svg/md, we can offer a preview tab
  const canPreview = ['html', 'svg', 'xml', 'md', 'markdown'].includes(language);

  const handleCopy = () => {
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="my-4 rounded-xl border border-[#d9d9d9] dark:border-[#333] bg-[#f7f7f7] dark:bg-[#1e1e1e] overflow-hidden shadow-sm">
      <div className="flex items-center justify-between px-4 py-2.5 bg-[#ebebeb] dark:bg-[#171717] border-b border-[#d9d9d9] dark:border-[#333] text-gray-500 dark:text-gray-300 text-xs">
        <div className="flex items-center gap-4">
          <span className="font-mono text-[#1890ff]">{language}</span>
          {onOpenArtifact && (
            <div className="flex bg-[#d9d9d9] dark:bg-[#0f0f0f] rounded-lg p-0.5 border border-[#ccc] dark:border-gray-800">
              <button
                onClick={() => onOpenArtifact(language, value, 'code')}
                className={cn(
                  "flex items-center gap-1.5 px-3 py-1 rounded-md transition-colors font-medium text-xs text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-[#e5e5e5] dark:hover:bg-[#2f2f2f]"
                )}
                title={t('splitView')}
              >
                <Code size={14} />
                {t('code')}
              </button>
              {canPreview && (
                <button
                  onClick={() => onOpenArtifact(language, value, 'preview')}
                  className={cn(
                    "flex items-center gap-1.5 px-3 py-1 rounded-md transition-colors font-medium text-xs text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-[#e5e5e5] dark:hover:bg-[#2f2f2f]"
                  )}
                  title={t('splitView')}
                >
                  <Play size={14} />
                  {t('preview')}
                </button>
              )}
            </div>
          )}
        </div>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1.5 text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors"
        >
          {copied ? <Check size={14} className="text-emerald-500 dark:text-emerald-400" /> : <Copy size={14} />}
          {copied ? t('copied') : t('copy')}
        </button>
      </div>
      
      <SyntaxHighlighter
        style={vscDarkPlus as any}
        language={language}
        PreTag="div"
        customStyle={{
          margin: 0,
          padding: '1rem',
          background: 'transparent',
          fontSize: '0.875rem',
        }}
      >
        {value}
      </SyntaxHighlighter>
    </div>
  );
};

export const MarkdownRenderer: React.FC<{ content: string; onOpenArtifact?: (language: string, code: string, mode?: 'preview' | 'code') => void }> = ({ content, onOpenArtifact }) => {
  return (
    <div className="markdown-body text-gray-800 dark:text-gray-200 space-y-4">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw as any]}
        components={{
          code: (props: any) => {
            const { children, className, node, ...rest } = props;
            const match = /language-(\w+)/.exec(className || '');
            const isMatch = match;
            
            if (!className) {
                return <code className="bg-[#f0f0f0] dark:bg-[#333] text-gray-800 dark:text-gray-300 border border-[#e5e5e5] dark:border-[#444] px-1.5 py-0.5 rounded font-mono text-sm" {...rest}>{children}</code>;
            }

            return isMatch ? (
              <CodeBlock language={match[1]} value={String(children).replace(/\n$/, '')} onOpenArtifact={onOpenArtifact} />
            ) : (
              <code className="bg-[#f0f0f0] dark:bg-[#333] text-gray-800 dark:text-gray-300 border border-[#e5e5e5] dark:border-[#444] px-1.5 py-0.5 rounded font-mono text-sm" {...rest}>
                {children}
              </code>
            );
          },
          p: ({ children }) => <p>{children}</p>,
          ul: ({ children }) => <ul className="list-disc pl-6 space-y-1.5">{children}</ul>,
          ol: ({ children }) => <ol className="list-decimal pl-6 space-y-1.5">{children}</ol>,
          h1: ({ children }) => <h1 className="text-2xl font-bold mt-8 mb-4 text-gray-900 dark:text-gray-50">{children}</h1>,
          h2: ({ children }) => <h2 className="text-xl font-bold mt-7 mb-3 text-gray-900 dark:text-gray-50">{children}</h2>,
          h3: ({ children }) => <h3 className="text-lg font-bold mt-6 mb-2 text-gray-900 dark:text-gray-50">{children}</h3>,
          a: ({ children, href }) => <a href={href} className="text-[#1890ff] dark:text-[#1890ff] hover:text-[#40a9ff] dark:hover:text-[#40a9ff] underline underline-offset-2">{children}</a>,
          blockquote: ({ children }) => <blockquote className="border-l-4 border-[#d9d9d9] dark:border-[#333] pl-4 italic text-gray-500 dark:text-gray-400">{children}</blockquote>,
          table: ({ children }) => (
            <div className="overflow-x-auto my-4">
               <table className="min-w-full divide-y divide-[#d9d9d9] dark:divide-[#333] border border-[#d9d9d9] dark:border-[#333]">
                {children}
              </table>
            </div>
          ),
          th: ({ children }) => <th className="px-4 py-2 bg-[#f7f7f7] dark:bg-[#202020] text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider border-b border-[#d9d9d9] dark:border-[#333]">{children}</th>,
          td: ({ children }) => <td className="px-4 py-2 whitespace-nowrap text-sm border-t border-[#d9d9d9] dark:border-[#333] text-gray-700 dark:text-gray-300">{children}</td>,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};
