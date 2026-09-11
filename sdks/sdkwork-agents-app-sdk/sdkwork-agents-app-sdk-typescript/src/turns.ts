import type { SdkworkAppClient } from '../generated/server-openapi/src/index.ts';
import { appApiPath } from '../generated/server-openapi/src/api/paths.ts';
import type { AgentTurnExecutionResponse } from '../generated/server-openapi/src/types/index.ts';
import type { CreateAgentTurnRequest } from '../generated/server-openapi/src/types/index.ts';

export type { CreateAgentTurnRequest };
import { NetworkError, TimeoutError } from '@sdkwork/sdk-common';

export type CompleteAgentTurnResult = AgentTurnExecutionResponse['data']['item'];

function pathSegment(value: string, name: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw new Error(`${name} is required.`);
  }
  return encodeURIComponent(normalized);
}

/** Execute one non-streaming turn through the canonical POST /turns operation. */
export async function completeAgentTurn(
  client: SdkworkAppClient,
  agentId: string,
  sessionId: string,
  body: CreateAgentTurnRequest,
): Promise<CompleteAgentTurnResult> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/sessions/${pathSegment(sessionId, 'sessionId')}/turns`,
  );
  return client.http.request<CompleteAgentTurnResult>(`${path}?stream=false`, {
    method: 'POST',
    body,
    contentType: 'application/json',
    sdkworkUnwrapKind: 'item',
    // Turn execution is idempotent-terminal server-side: a 5xx response means
    // the durable turn already recorded its outcome, and replaying the same
    // idempotency key cannot re-execute it. Only transport-level failures
    // (the request never reached the server) are worth retrying.
    retry: {
      retryCondition: (error) =>
        error instanceof NetworkError || error instanceof TimeoutError,
    },
  });
}

/** One SSE `delta` event from the streaming turn protocol (visible text). */
export interface TurnStreamDeltaEvent {
  eventType?: 'delta';
  index?: number;
  delta?: string;
}

/** One SSE `completion` event carrying the final SDKWork response envelope. */
export interface TurnStreamCompletionEvent {
  eventType: 'completion';
  response?: { data?: { item?: CompleteAgentTurnResult } };
}

/** Rich turn-stream protocol: emit `agent.stream.*` events (`event_protocol=kernel-v1`). */
export const TURN_EVENT_PROTOCOL_KERNEL_V1 = 'kernel-v1';

/**
 * One streamed tool/skill/MCP invocation event from the kernel-v1 protocol.
 * Keeps tool-call JSON out of visible text so chat UIs can render tool calls
 * as first-class, collapsible cards.
 */
export interface TurnRichToolEvent {
  phase: 'start' | 'delta' | 'stop';
  toolCallId?: string;
  toolName?: string;
  delta?: string;
}

/** Streaming delivery handlers for `completeAgentTurnStream`. */
export interface TurnStreamHandlers {
  /** Visible assistant-text delta. */
  onDelta?: (delta: string) => void;
  /** Reasoning/thinking delta (rendered as a collapsible block). */
  onReasoning?: (reasoning: string) => void;
  /** Tool/skill/MCP invocation lifecycle event. */
  onToolEvent?: (event: TurnRichToolEvent) => void;
}

/** One SSE `event` event from the kernel-v1 rich stream. */
export interface TurnStreamRichEvent {
  eventType?: 'event';
  event?: {
    type?: string;
    payload?: {
      message_id?: string;
      kind?: string;
      delta?: string;
      tool_call_id?: string;
      tool_name?: string;
    };
  };
}

export type TurnStreamEvent =
  | TurnStreamDeltaEvent
  | TurnStreamRichEvent
  | TurnStreamCompletionEvent;

function dispatchRichEvent(raw: TurnStreamRichEvent, handlers: TurnStreamHandlers): void {
  const event = raw.event;
  if (!event) return;
  const type = event.type ?? '';
  const payload = event.payload ?? {};
  if (type === 'agent.stream.message.delta') {
    // Reasoning deltas surface only via rich events. Visible answer text is
    // already delivered by the sibling `delta` SSE frames, so forwarding
    // text-kind events here would double-render the answer.
    if (payload.kind !== 'reasoning') return;
    const delta = payload.delta;
    if (typeof delta !== 'string' || !delta) return;
    handlers.onReasoning?.(delta);
    return;
  }
  if (type === 'agent.stream.tool.call.start') {
    handlers.onToolEvent?.({
      phase: 'start',
      toolCallId: payload.tool_call_id,
      toolName: payload.tool_name,
    });
    return;
  }
  if (type === 'agent.stream.tool.call.delta') {
    handlers.onToolEvent?.({
      phase: 'delta',
      toolCallId: payload.tool_call_id,
      delta: payload.delta,
    });
    return;
  }
  if (type === 'agent.stream.tool.call.stop') {
    handlers.onToolEvent?.({
      phase: 'stop',
      toolCallId: payload.tool_call_id,
      toolName: payload.tool_name,
    });
  }
}

/**
 * Executes one turn with SSE streaming (`?stream=true&event_protocol=kernel-v1`).
 * Visible text arrives as `delta` events (via `onDelta`); the rich `event`
 * events carry reasoning deltas (via `onReasoning`) and tool/skill/MCP
 * invocations (via `onToolEvent`). The trailing `completion` event carries the
 * final response envelope.
 */
export async function completeAgentTurnStream(
  client: SdkworkAppClient,
  agentId: string,
  sessionId: string,
  body: CreateAgentTurnRequest,
  handlers: TurnStreamHandlers | ((delta: string) => void),
): Promise<CompleteAgentTurnResult> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/sessions/${pathSegment(sessionId, 'sessionId')}/turns`,
  );
  // Backward-compatible: allow a positional onDelta callback as the last argument.
  const resolvedHandlers: TurnStreamHandlers =
    typeof handlers === 'function' ? { onDelta: handlers } : handlers;

  let result: CompleteAgentTurnResult | undefined;
  for await (const event of client.http.streamJson<TurnStreamEvent>(
    `${path}?stream=true&event_protocol=${TURN_EVENT_PROTOCOL_KERNEL_V1}`,
    {
      method: 'POST',
      body,
      contentType: 'application/json',
    },
  )) {
    if (event.eventType === 'event') {
      dispatchRichEvent(event as TurnStreamRichEvent, resolvedHandlers);
    } else if (event.eventType === 'delta' && typeof event.delta === 'string' && event.delta) {
      resolvedHandlers.onDelta?.(event.delta);
    } else if (event.eventType === 'completion' && event.response?.data?.item) {
      result = event.response.data.item;
    }
  }
  if (!result) {
    throw new Error('Agent turn stream did not return a completion.');
  }
  return result;
}
