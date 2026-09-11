export interface MarkdownRendererProps {
  content: string;
  onOpenArtifact?: (language: string, code: string, mode?: 'preview' | 'code') => void;
  /** When true, closes open code fences and shows a streaming cursor. */
  streaming?: boolean;
  /**
   * When true, renders in a muted tone: smaller font and grayer text, for
   * secondary streams (e.g. reasoning/thinking transcripts) that must read
   * as visually subordinate to the main answer.
   */
  muted?: boolean;
}
