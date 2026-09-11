import React, { createContext, memo, useContext, useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeSanitize from 'rehype-sanitize';

import { ChatCodeBlock } from './ChatCodeBlock';
import { cn } from './classNames';
import {
  isSafeMarkdownHref,
  prepareChatMarkdownSource,
} from './chatMarkdownUtils';
import { chatMarkdownSanitizeSchema } from './chatMarkdownSanitizeSchema';
import type { MarkdownRendererProps } from './MarkdownRenderer.types';

const BlockCodeContext = createContext(false);

function ChatMarkdownPre({ children }: { children?: React.ReactNode }) {
  return (
    <BlockCodeContext.Provider value={true}>
      {children}
    </BlockCodeContext.Provider>
  );
}

function ChatMarkdownCode({
  children,
  className,
  onOpenArtifact,
  ...rest
}: {
  children?: React.ReactNode;
  className?: string;
  onOpenArtifact?: MarkdownRendererProps['onOpenArtifact'];
}) {
  const inBlock = useContext(BlockCodeContext);
  const match = /language-([\w-]+)/.exec(className || '');
  const text = String(children ?? '').replace(/\n$/, '');
  if (inBlock || match) {
    return (
      <ChatCodeBlock
        language={match?.[1] || 'text'}
        code={text}
        onOpenArtifact={onOpenArtifact}
      />
    );
  }
  return (
    <code className="chat-md-inline-code" {...rest}>
      {children}
    </code>
  );
}

const MarkdownRendererImplComponent: React.FC<MarkdownRendererProps> = ({
  content,
  onOpenArtifact,
  streaming = false,
  muted = false,
}) => {
  const markdown = useMemo(
    () => prepareChatMarkdownSource(content, streaming),
    [content, streaming],
  );

  const components = useMemo(
    () => ({
      pre: ChatMarkdownPre,
      code(props: {
        children?: React.ReactNode;
        className?: string;
        node?: unknown;
      }) {
        const { node: _node, ...rest } = props;
        return (
          <ChatMarkdownCode
            {...rest}
            onOpenArtifact={onOpenArtifact}
          />
        );
      },
      p({ children }: { children?: React.ReactNode }) {
        return <p className="chat-md-paragraph">{children}</p>;
      },
      ul({ children }: { children?: React.ReactNode }) {
        return <ul className="chat-md-list chat-md-list-disc">{children}</ul>;
      },
      ol({ children }: { children?: React.ReactNode }) {
        return <ol className="chat-md-list chat-md-list-decimal">{children}</ol>;
      },
      li({
        children,
        className,
      }: {
        children?: React.ReactNode;
        className?: string;
      }) {
        const isTaskItem = className?.includes('task-list-item');
        return (
          <li
            className={cn(
              'chat-md-list-item',
              isTaskItem && 'chat-md-task-item',
              className,
            )}
          >
            {children}
          </li>
        );
      },
      h1({ children }: { children?: React.ReactNode }) {
        return <h1 className="chat-md-heading chat-md-h1">{children}</h1>;
      },
      h2({ children }: { children?: React.ReactNode }) {
        return <h2 className="chat-md-heading chat-md-h2">{children}</h2>;
      },
      h3({ children }: { children?: React.ReactNode }) {
        return <h3 className="chat-md-heading chat-md-h3">{children}</h3>;
      },
      h4({ children }: { children?: React.ReactNode }) {
        return <h4 className="chat-md-heading chat-md-h4">{children}</h4>;
      },
      a({ children, href }: { children?: React.ReactNode; href?: string }) {
        if (!href || !isSafeMarkdownHref(href)) {
          return <span className="chat-md-link-fallback">{children}</span>;
        }
        const external = /^https?:\/\//i.test(href);
        return (
          <a
            href={href}
            className="chat-md-link"
            target={external ? '_blank' : undefined}
            rel={external ? 'noopener noreferrer' : undefined}
          >
            {children}
          </a>
        );
      },
      blockquote({ children }: { children?: React.ReactNode }) {
        return <blockquote className="chat-md-blockquote">{children}</blockquote>;
      },
      hr() {
        return <hr className="chat-md-divider" />;
      },
      table({ children }: { children?: React.ReactNode }) {
        return (
          <div className="chat-md-table-wrap">
            <table className="chat-md-table">{children}</table>
          </div>
        );
      },
      thead({ children }: { children?: React.ReactNode }) {
        return <thead className="chat-md-table-head">{children}</thead>;
      },
      tbody({ children }: { children?: React.ReactNode }) {
        return <tbody>{children}</tbody>;
      },
      tr({ children }: { children?: React.ReactNode }) {
        return <tr className="chat-md-table-row">{children}</tr>;
      },
      th({ children }: { children?: React.ReactNode }) {
        return <th className="chat-md-table-header">{children}</th>;
      },
      td({ children }: { children?: React.ReactNode }) {
        return <td className="chat-md-table-cell">{children}</td>;
      },
      strong({ children }: { children?: React.ReactNode }) {
        return <strong className="chat-md-strong">{children}</strong>;
      },
      em({ children }: { children?: React.ReactNode }) {
        return <em className="chat-md-emphasis">{children}</em>;
      },
      del({ children }: { children?: React.ReactNode }) {
        return <del className="chat-md-deleted">{children}</del>;
      },
      img({ src, alt }: { src?: string; alt?: string }) {
        if (!src || !/^https?:\/\//i.test(src)) {
          return null;
        }
        return (
          <img
            src={src}
            alt={alt ?? ''}
            loading="lazy"
            className="chat-md-image"
          />
        );
      },
      input(props: { type?: string; checked?: boolean; disabled?: boolean }) {
        if (props.type !== 'checkbox') {
          return null;
        }
        return (
          <input
            type="checkbox"
            checked={props.checked}
            disabled
            readOnly
            className="chat-md-checkbox"
          />
        );
      },
    }),
    [onOpenArtifact],
  );

  if (!markdown.trim()) {
    return null;
  }

  return (
    <div
      className={cn(
        'markdown-body chat-markdown',
        muted && 'chat-markdown--muted',
        muted
          ? 'text-gray-500 dark:text-gray-400'
          : 'text-gray-800 dark:text-gray-200',
        streaming && 'chat-markdown-streaming',
      )}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[[rehypeSanitize, chatMarkdownSanitizeSchema]]}
        components={components}
      >
        {markdown}
      </ReactMarkdown>
      {streaming && (
        <span className="chat-md-stream-cursor" aria-hidden="true" />
      )}
    </div>
  );
};

export const MarkdownRenderer = memo(MarkdownRendererImplComponent);
