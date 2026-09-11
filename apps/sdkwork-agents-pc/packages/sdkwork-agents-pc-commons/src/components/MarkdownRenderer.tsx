import { Component, type FC, type ReactNode } from 'react';

import { cn } from './classNames';
import { MarkdownRenderer as MarkdownRendererCore } from './MarkdownRendererImpl';
import type { MarkdownRendererProps } from './MarkdownRenderer.types';
import './chat-markdown.css';

export { cn };
export type { MarkdownRendererProps } from './MarkdownRenderer.types';

function plainTextMarkdown(props: MarkdownRendererProps): ReactNode {
  return (
    <div
      className={cn(
        'whitespace-pre-wrap',
        props.muted
          ? 'text-xs text-gray-500 dark:text-gray-400'
          : 'text-gray-800 dark:text-gray-200',
      )}
    >
      {props.content}
    </div>
  );
}

/**
 * Contains render-time failures inside the markdown renderer so a single
 * message can never take down the whole chat: on error the raw content is
 * shown as plain text instead of propagating to the app error boundary.
 */
class MarkdownRendererErrorBoundary extends Component<
  { fallback: ReactNode; children: ReactNode },
  { failed: boolean }
> {
  state = { failed: false };

  static getDerivedStateFromError(): { failed: boolean } {
    return { failed: true };
  }

  componentDidCatch(error: unknown): void {
    console.error('[MarkdownRenderer] render failed, falling back to plain text', error);
  }

  render(): ReactNode {
    if (this.state.failed) {
      return this.props.fallback;
    }
    return this.props.children;
  }
}

/** Eager markdown renderer — lazy loading previously hid chunk failures as plain text. */
export const MarkdownRenderer: FC<MarkdownRendererProps> = (props) => (
  <MarkdownRendererErrorBoundary fallback={plainTextMarkdown(props)}>
    <MarkdownRendererCore {...props} />
  </MarkdownRendererErrorBoundary>
);
