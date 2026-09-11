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

/** One SSE `delta` event from the streaming turn protocol. */
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

export type TurnStreamEvent = TurnStreamDeltaEvent | TurnStreamCompletionEvent;

/**
 * Executes one turn with SSE streaming (`?stream=true`): `delta` events carry
 * incremental text chunks (invoked on `onDelta`), and the trailing
 * `completion` event carries the final response envelope.
 */
export async function completeAgentTurnStream(
  client: SdkworkAppClient,
  agentId: string,
  sessionId: string,
  body: CreateAgentTurnRequest,
  onDelta: (delta: string) => void,
): Promise<CompleteAgentTurnResult> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/sessions/${pathSegment(sessionId, 'sessionId')}/turns`,
  );
  let result: CompleteAgentTurnResult | undefined;
  for await (const event of client.http.streamJson<TurnStreamEvent>(`${path}?stream=true`, {
    method: 'POST',
    body,
    contentType: 'application/json',
  })) {
    if (event.eventType === 'delta' && typeof event.delta === 'string' && event.delta) {
      onDelta(event.delta);
    } else if (event.eventType === 'completion' && event.response?.data?.item) {
      result = event.response.data.item;
    }
  }
  if (!result) {
    throw new Error('Agent turn stream did not return a completion.');
  }
  return result;
}
