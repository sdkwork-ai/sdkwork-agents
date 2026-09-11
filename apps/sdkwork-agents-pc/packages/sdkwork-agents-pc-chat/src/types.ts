export type MessageRole = 'user' | 'model';

/** Structured tool/skill/MCP invocation attached to an assistant message. */
export interface ChatToolCall {
  id: string;
  name?: string;
  status: 'running' | 'completed' | 'error';
  /** Accumulated JSON arguments for the invocation. */
  arguments?: string;
  durationMs?: number;
}

/** Wire form of a streamed tool-call lifecycle event delivered to the chat UI. */
export interface ChatToolStreamEvent {
  phase: 'start' | 'delta' | 'stop';
  toolCallId?: string;
  toolName?: string;
  delta?: string;
}

export interface ChatMessage {
  id: string;
  role: MessageRole;
  text: string;
  /** Thinking/reasoning text rendered as a collapsible block. */
  reasoning?: string;
  /** Tool/skill/MCP invocations performed within this turn. */
  toolCalls?: ChatToolCall[];
  images?: string[];
  mediaResources?: import('@sdkwork/agents-pc-core/sdk/driveUploadService').AgentsDriveMediaResource[];
  feedback?: 'up' | 'down';
  feedbackVersion?: string;
}

export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  updatedAt: number;
  version: string;
  projectId?: string;
  pinned?: boolean;
  userStateVersion?: string;
}
