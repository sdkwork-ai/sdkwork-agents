import { Component, lazy, Suspense, type FC, type ReactNode } from 'react';

import { cn } from './classNames';

export { cn };

export interface MarkdownRendererProps {
  content: string;
  onOpenArtifact?: (language: string, code: string, mode?: 'preview' | 'code') => void;
}

function plainTextMarkdown(props: MarkdownRendererProps): ReactNode {
  return (
    <div className="whitespace-pre-wrap text-gray-800 dark:text-gray-200">{props.content}</div>
  );
}

const MarkdownRendererFallback: FC<MarkdownRendererProps> = (props) => (
  <>{plainTextMarkdown(props)}</>
);

/**
 * Loads the markdown renderer lazily. Chunk-load failures (broken or
 * unavailable markdown dependencies in dev) resolve to the plain-text
 * fallback at the promise level, so React never sees an error: the message
 * still renders as raw text without crashing the chat or spamming the
 * console with error boundary reports.
 */
const MarkdownRendererImpl = lazy(() =>
  import('./MarkdownRendererImpl')
    .then((module) => ({
      default: module.MarkdownRenderer,
    }))
    .catch(() => ({
      default: MarkdownRendererFallback,
    })),
);

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

  render(): ReactNode {
    if (this.state.failed) {
      return this.props.fallback;
    }
    return this.props.children;
  }
}

export const MarkdownRenderer: FC<MarkdownRendererProps> = (props) => (
  <Suspense fallback={plainTextMarkdown(props)}>
    <MarkdownRendererErrorBoundary fallback={plainTextMarkdown(props)}>
      <MarkdownRendererImpl {...props} />
    </MarkdownRendererErrorBoundary>
  </Suspense>
);
