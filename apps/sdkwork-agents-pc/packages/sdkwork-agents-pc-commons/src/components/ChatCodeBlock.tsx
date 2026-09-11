import { useState } from 'react';
import type { ReactNode } from 'react';
import { Check, Code, Copy, Play } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import type { MarkdownRendererProps } from './MarkdownRenderer.types';

const CODE_COPY_RESET_MS = 2000;

interface ChatCodeBlockProps {
  code: string;
  language?: string;
  onOpenArtifact?: MarkdownRendererProps['onOpenArtifact'];
}

export function ChatCodeBlock({
  code,
  language,
  onOpenArtifact,
}: ChatCodeBlockProps) {
  const { t } = useTranslation('common');
  const [copied, setCopied] = useState(false);
  const displayCode = normalizeCodeBlockLineSeparators(code);
  const languageLabel = normalizeLanguageLabel(language);
  const canPreview = ['html', 'svg', 'xml', 'md', 'markdown'].includes(languageLabel.toLowerCase());

  async function handleCopy(): Promise<void> {
    if (!displayCode || !globalThis.navigator?.clipboard?.writeText) {
      return;
    }
    try {
      await globalThis.navigator.clipboard.writeText(displayCode);
      setCopied(true);
      globalThis.setTimeout(() => setCopied(false), CODE_COPY_RESET_MS);
    } catch {
      setCopied(false);
    }
  }

  return (
    <figure className="chat-md-code-block my-4 min-w-0 overflow-hidden rounded-xl border border-[#d9d9d9] bg-[#f7f7f7] text-left shadow-sm dark:border-[#333] dark:bg-[#1e1e1e]">
      <figcaption className="flex items-center justify-between border-b border-[#d9d9d9] bg-[#ebebeb] px-4 py-2.5 text-xs text-gray-500 dark:border-[#333] dark:bg-[#171717] dark:text-gray-300">
        <div className="flex items-center gap-4">
          <span className="font-mono text-[#1890ff] dark:text-cyan-400">{languageLabel}</span>
          {onOpenArtifact && (
            <div className="flex rounded-lg border border-[#ccc] bg-[#d9d9d9] p-0.5 dark:border-gray-800 dark:bg-[#0f0f0f]">
              <button
                type="button"
                onClick={() => onOpenArtifact(languageLabel, displayCode, 'code')}
                className="flex items-center gap-1.5 rounded-md px-3 py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-[#e5e5e5] hover:text-gray-900 dark:text-gray-400 dark:hover:bg-[#2f2f2f] dark:hover:text-gray-200"
              >
                <Code size={14} />
                {t('code')}
              </button>
              {canPreview && (
                <button
                  type="button"
                  onClick={() => onOpenArtifact(languageLabel, displayCode, 'preview')}
                  className="flex items-center gap-1.5 rounded-md px-3 py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-[#e5e5e5] hover:text-gray-900 dark:text-gray-400 dark:hover:bg-[#2f2f2f] dark:hover:text-gray-200"
                >
                  <Play size={14} />
                  {t('preview')}
                </button>
              )}
            </div>
          )}
        </div>
        <button
          type="button"
          onClick={() => {
            void handleCopy();
          }}
          className="flex items-center gap-1.5 text-gray-500 transition-colors hover:text-gray-900 dark:text-gray-400 dark:hover:text-white"
        >
          {copied ? (
            <Check size={14} className="text-emerald-500 dark:text-emerald-400" />
          ) : (
            <Copy size={14} />
          )}
          {copied ? t('copied') : t('copy')}
        </button>
      </figcaption>
      <pre
        tabIndex={0}
        className="max-w-full overflow-x-auto bg-[#1e1e1e] px-0 py-3 text-[13.5px] leading-[1.625rem] text-gray-100 [tab-size:2] dark:bg-[#0f0f0f]"
      >
        <code className="block min-w-max whitespace-pre font-mono">
          {renderHighlightedCodeLines(displayCode || '\u00a0', languageLabel)}
        </code>
      </pre>
    </figure>
  );
}

function normalizeLanguageLabel(language: string | undefined): string {
  const normalized = language?.trim().replace(/^language-/, '') || '';
  return normalized || 'text';
}

function normalizeCodeBlockLineSeparators(value: string): string {
  if (!/\\[rnt]/.test(value)) {
    return value;
  }

  let result = '';
  let quote: '"' | "'" | '`' | null = null;
  let escaped = false;

  for (let index = 0; index < value.length; index += 1) {
    const current = value[index];
    const next = value[index + 1];

    if (quote) {
      result += current;
      if (escaped) {
        escaped = false;
      } else if (current === '\\') {
        escaped = true;
      } else if (current === quote) {
        quote = null;
      }
      continue;
    }

    if (current === '"' || current === "'" || current === '`') {
      quote = current;
      result += current;
      continue;
    }

    if (current === '\\' && next === 'r') {
      if (value[index + 2] === '\\' && value[index + 3] === 'n') {
        result += '\n';
        index += 3;
      } else {
        result += '\n';
        index += 1;
      }
      continue;
    }

    if (current === '\\' && next === 'n') {
      result += '\n';
      index += 1;
      continue;
    }

    if (current === '\\' && next === 't') {
      result += '\t';
      index += 1;
      continue;
    }

    result += current;
  }

  return result;
}

function renderHighlightedCodeLines(code: string, language: string | undefined): ReactNode[] {
  return code.split('\n').map((line, lineIndex) => (
    <span
      key={`code-line-${lineIndex}`}
      data-chat-code-line={lineIndex + 1}
      className="block min-h-[1.625rem] whitespace-pre px-4"
    >
      {highlightCodeLine(line, language, lineIndex)}
    </span>
  ));
}

function highlightCodeLine(line: string, language: string | undefined, lineIndex: number): ReactNode[] {
  const result: ReactNode[] = [];
  const tokenPattern = /("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\/\/.*$|#.*$|\b\d+(?:\.\d+)?\b|\b[A-Za-z_$][\w$]*\b|[{}()[\].,;:<>/=+\-*%!?&|]+)/g;
  let cursor = 0;
  let tokenIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = tokenPattern.exec(line)) !== null) {
    if (match.index > cursor) {
      result.push(line.slice(cursor, match.index));
    }
    const token = match[0];
    const className = codeTokenClassName(token, language);
    result.push(className ? (
      <span key={`${lineIndex}-${tokenIndex}`} className={className}>{token}</span>
    ) : token);
    cursor = match.index + token.length;
    tokenIndex += 1;
  }

  if (cursor < line.length) {
    result.push(line.slice(cursor));
  }
  return result.length > 0 ? result : [''];
}

function codeTokenClassName(token: string, language: string | undefined): string | undefined {
  if (/^(\/\/|#)/.test(token)) {
    return 'chat-md-code-token-comment';
  }
  if (/^(['"`])/.test(token)) {
    return 'chat-md-code-token-string';
  }
  if (/^\d/.test(token)) {
    return 'chat-md-code-token-number';
  }
  if (isCodeKeyword(token, language)) {
    return 'chat-md-code-token-keyword';
  }
  if (isCodeBuiltin(token)) {
    return 'chat-md-code-token-builtin';
  }
  if (/^[{}()[\].,;:<>/=+\-*%!?&|]+$/.test(token)) {
    return 'chat-md-code-token-punct';
  }
  return undefined;
}

function isCodeKeyword(token: string, language: string | undefined): boolean {
  const normalizedLanguage = language?.toLowerCase() || '';
  const sharedKeywords = new Set([
    'as', 'async', 'await', 'break', 'case', 'catch', 'class', 'const', 'continue',
    'default', 'do', 'else', 'export', 'extends', 'false', 'finally', 'for', 'from',
    'function', 'if', 'import', 'in', 'interface', 'let', 'new', 'null', 'return',
    'switch', 'throw', 'true', 'try', 'type', 'undefined', 'while',
  ]);
  if (sharedKeywords.has(token)) {
    return true;
  }
  if (['py', 'python'].includes(normalizedLanguage)) {
    return ['def', 'elif', 'except', 'global', 'lambda', 'nonlocal', 'pass', 'with', 'yield'].includes(token);
  }
  if (['rs', 'rust'].includes(normalizedLanguage)) {
    return ['fn', 'impl', 'let', 'match', 'mod', 'mut', 'pub', 'self', 'struct', 'trait', 'use', 'where'].includes(token);
  }
  if (normalizedLanguage === 'sql') {
    return [
      'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'JOIN', 'LEFT', 'RIGHT',
      'GROUP', 'ORDER', 'BY', 'LIMIT', 'VALUES',
    ].includes(token.toUpperCase());
  }
  return false;
}

function isCodeBuiltin(token: string): boolean {
  return [
    'Array', 'Boolean', 'Date', 'Error', 'Map', 'Number', 'Object', 'Promise',
    'Record', 'Set', 'String', 'console', 'fetch',
  ].includes(token);
}

/** Test hook for chat markdown token styling without rendering React. */
export function getChatCodeTokenClassName(token: string, language?: string): string | undefined {
  return codeTokenClassName(token, language);
}
