import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { ActivateAgentProviderBindingRequest, AgentAuditEvent, AgentCompositionSlotRecord, AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus, AgentProviderBindingRecord, AgentRecord, AgentSessionCheckpointRecord, AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentTaskRecord, AgentTaskRunAttemptRecord, AgentTaskRunRecord, AgentTaskStateChangeRequest, AgentTurnRecord, AgentTurnStreamEvent, AnswerAgentInteractionRequest, ApproveAgentInteractionRequest, ArchiveAgentSessionRequest, AuditAction, CancelAgentTaskRequest, CancelAgentTaskRunRequest, CancelAgentTurnRequest, ChangeAgentSessionRuntimeBindingStatusRequest, ClaimAgentInteractionRequest, CloseAgentSessionRequest, CreateAgentCompositionSlotRequest, CreateAgentInteractionRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, CreateAgentSessionCheckpointRequest, CreateAgentSessionRequest, CreateAgentSessionRuntimeBindingRequest, CreateAgentTaskRequest, CreateAgentTurnRequest, ExecuteAgentTaskRequest, Int64String, InvalidateAgentSessionCheckpointRequest, MediaToolConfigurationBody, MediaToolDirectoryEntry, ReconcileAgentTaskRunRequest, ReplaceAgentTaskRequest, ResolveAgentInteractionRequest, RestoreAgentRequest, RestoreAgentSessionCheckpointRequest, RetryAgentTaskRunRequest, SdkWorkPageData, UpdateAgentCompositionSlotRequest, UpdateAgentRequest, UpdateAgentSessionRuntimeBindingRequest, UpdateAgentStatusRequest } from '../types';


export class AiAgentsToolsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List media tool directory with tenant configuration (admin) */
  async list(requestOptions?: ApiRequestOptions): Promise<MediaToolDirectoryEntry[]> {
    return this.client.request<MediaToolDirectoryEntry[]>(backendApiPath(`/ai/tools`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Update one media tool tenant configuration (admin) */
  async update(toolId: string, body: MediaToolConfigurationBody, requestOptions?: ApiRequestOptions): Promise<MediaToolDirectoryEntry> {
    return this.client.request<MediaToolDirectoryEntry>(backendApiPath(`/ai/tools/${serializePathParameter(toolId, { name: 'toolId', style: 'simple', explode: false })}/configuration`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export class AiAgentsCompositionSlotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List composition slots for one managed agent */
  async list(agentId: string, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }> {
    return this.client.request<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a composition slot for one managed agent */
  async create(agentId: string, body: CreateAgentCompositionSlotRequest, requestOptions?: ApiRequestOptions): Promise<AgentCompositionSlotRecord> {
    return this.client.request<AgentCompositionSlotRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one managed agent composition slot */
  async retrieve(agentId: string, slotId: string, requestOptions?: ApiRequestOptions): Promise<AgentCompositionSlotRecord> {
    return this.client.request<AgentCompositionSlotRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update one managed agent composition slot */
  async update(agentId: string, slotId: string, body: UpdateAgentCompositionSlotRequest, requestOptions?: ApiRequestOptions): Promise<AgentCompositionSlotRecord> {
    return this.client.request<AgentCompositionSlotRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Delete one managed agent composition slot */
  async delete(agentId: string, slotId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface AiAgentsTaskRunAttemptsListParams {
  cursor?: string;
  pageSize?: number;
}

export class AiAgentsTaskRunAttemptsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List execution Attempts for one scheduled task Run */
  async list(agentId: string, taskId: string, runId: string, params?: AiAgentsTaskRunAttemptsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentTaskRunAttemptRecord[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentTaskRunAttemptRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}/attempts`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface AiAgentsTaskRunsListParams {
  status?: 'pending' | 'claimed' | 'running' | 'succeeded' | 'failed' | 'cancelled' | 'reconciling' | 'dead_letter';
  triggerKind?: 'scheduled' | 'manual' | 'business_retry';
  cursor?: string;
  pageSize?: number;
}

export class AiAgentsTaskRunsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List Runs for one scheduled task using opaque keyset pagination */
  async list(agentId: string, taskId: string, params?: AiAgentsTaskRunsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentTaskRunRecord[]; }> {
    const query = buildQueryString([
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'trigger_kind', value: params?.triggerKind, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentTaskRunRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Retrieve one scheduled task Run */
  async retrieve(agentId: string, taskId: string, runId: string, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Create an idempotent business retry Run from a terminal Run */
  async retry(agentId: string, taskId: string, runId: string, body: RetryAgentTaskRunRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}/retry`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Cancel a pending Run or request cancellation and reconciliation for an active Run */
  async cancel(agentId: string, taskId: string, runId: string, body: CancelAgentTaskRunRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}/cancel`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Reconcile an indeterminate task Run to a verified terminal outcome */
  async reconcile(agentId: string, taskId: string, runId: string, body: ReconcileAgentTaskRunRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}/reconcile`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsTasksListParams {
  status?: 'active' | 'paused' | 'completed' | 'cancelled';
  cursor?: string;
  pageSize?: number;
}

export class AiAgentsTasksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List scheduled tasks for one managed agent */
  async list(agentId: string, params?: AiAgentsTasksListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentTaskRecord[]; }> {
    const query = buildQueryString([
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentTaskRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a scheduled task for one managed agent */
  async create(agentId: string, body: CreateAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one scheduled task */
  async retrieve(agentId: string, taskId: string, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Replace one scheduled task definition and execution policy */
  async update(agentId: string, taskId: string, body: ReplaceAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Pause future materialization for one scheduled task */
  async pause(agentId: string, taskId: string, body: AgentTaskStateChangeRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/pause`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Resume future materialization for one paused scheduled task */
  async resume(agentId: string, taskId: string, body: AgentTaskStateChangeRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/resume`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Cancel one scheduled task */
  async cancel(agentId: string, taskId: string, body: CancelAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/cancel`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Materialize one idempotent manual Run for an active scheduled task */
  async execute(agentId: string, taskId: string, body: ExecuteAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/execute`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsSessionRuntimeBindingsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsSessionRuntimeBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List runtime bindings for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsSessionRuntimeBindingsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentSessionRuntimeBindingRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentSessionRuntimeBindingRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create the current runtime binding for one agent session */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionRuntimeBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one agent session runtime binding */
  async retrieve(agentId: string, sessionId: string, runtimeBindingId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update one agent session runtime binding */
  async update(agentId: string, sessionId: string, runtimeBindingId: string, body: UpdateAgentSessionRuntimeBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Activate one agent session runtime binding as current */
  async activate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/activate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Deactivate one agent session runtime binding */
  async deactivate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/deactivate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsCheckpointsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsCheckpointsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List resumable checkpoints for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsCheckpointsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentSessionCheckpointRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentSessionCheckpointRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create one bounded agent session checkpoint */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionCheckpointRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one agent session checkpoint */
  async retrieve(agentId: string, sessionId: string, checkpointId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Restore one resumable agent session checkpoint */
  async restore(agentId: string, sessionId: string, checkpointId: string, body: RestoreAgentSessionCheckpointRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/restore`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Invalidate one agent session checkpoint */
  async invalidate(agentId: string, sessionId: string, checkpointId: string, body: InvalidateAgentSessionCheckpointRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/invalidate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsInteractionsListParams {
  cursor?: string;
  pageSize?: number;
  kind?: AgentInteractionKind;
  status?: AgentInteractionStatus;
}

export class AiAgentsInteractionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List durable interactions for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsInteractionsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentInteractionRecord[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'kind', value: params?.kind, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentInteractionRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create one durable typed or legacy agent interaction */
  async create(agentId: string, sessionId: string, body: CreateAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one durable agent interaction */
  async retrieve(agentId: string, sessionId: string, interactionId: string, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Claim one pending agent interaction for exclusive resolution */
  async claim(agentId: string, sessionId: string, interactionId: string, body: ClaimAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<{ interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }> {
    return this.client.request<{ interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/claim`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Approve or reject one approval interaction */
  async approve(agentId: string, sessionId: string, interactionId: string, body: ApproveAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/approve`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Answer or reject one user-question interaction */
  async answer(agentId: string, sessionId: string, interactionId: string, body: AnswerAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/answer`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Resolve one typed agent interaction */
  async resolve(agentId: string, sessionId: string, interactionId: string, body: ResolveAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/resolve`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsTurnsListParams {
  cursor?: string;
  pageSize?: number;
}

export interface AiAgentsTurnsStreamParams {
  stream?: boolean;
  eventProtocol?: 'kernel-v1';
}

export class AiAgentsTurnsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List durable turns for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsTurnsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentTurnRecord[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentTurnRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create one idempotent agent turn */
  async stream(agentId: string, sessionId: string, body: CreateAgentTurnRequest, params?: AiAgentsTurnsStreamParams, requestOptions?: ApiRequestOptions): Promise<AsyncIterable<AgentTurnStreamEvent>> {
    const query = buildQueryString([
      { name: 'stream', value: params?.stream, style: 'form', explode: true, allowReserved: false },
      { name: 'event_protocol', value: params?.eventProtocol, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.streamJson<AgentTurnStreamEvent>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json' });
  }

/** Retrieve one durable agent turn */
  async retrieve(agentId: string, sessionId: string, turnId: string, requestOptions?: ApiRequestOptions): Promise<AgentTurnRecord> {
    return this.client.request<AgentTurnRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Request cancellation of one agent turn */
  async cancel(agentId: string, sessionId: string, turnId: string, body: CancelAgentTurnRequest, requestOptions?: ApiRequestOptions): Promise<AgentTurnRecord> {
    return this.client.request<AgentTurnRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}/cancel`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsSessionItemsListParams {
  cursor?: string;
  pageSize?: number;
  kind?: AgentSessionItemKind;
  status?: AgentSessionItemStatus;
  sort?: 'sequence' | '-sequence';
}

export class AiAgentsSessionItemsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List ordered items for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsSessionItemsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentSessionItemRecord[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'kind', value: params?.kind, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'sort', value: params?.sort, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentSessionItemRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Retrieve one agent session item */
  async retrieve(agentId: string, sessionId: string, itemId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionItemRecord> {
    return this.client.request<AgentSessionItemRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items/${serializePathParameter(itemId, { name: 'itemId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsSessionsListParams {
  cursor?: string;
  pageSize?: number;
}

export class AiAgentsSessionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List agent sessions for one managed agent */
  async list(agentId: string, params?: AiAgentsSessionsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentSessionRecord[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentSessionRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a agent session for one managed agent */
  async create(agentId: string, body: CreateAgentSessionRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one agent session */
  async retrieve(agentId: string, sessionId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Close one agent session */
  async close(agentId: string, sessionId: string, body: CloseAgentSessionRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/close`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Archive one agent session */
  async archive(agentId: string, sessionId: string, body: ArchiveAgentSessionRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/archive`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsProviderBindingsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsProviderBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List provider bindings for one managed agent */
  async list(agentId: string, params?: AiAgentsProviderBindingsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentProviderBindingRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentProviderBindingRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a provider binding for one managed agent */
  async create(agentId: string, body: CreateAgentProviderBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentProviderBindingRecord> {
    return this.client.request<AgentProviderBindingRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Activate one managed agent provider binding */
  async activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentProviderBindingRecord> {
    return this.client.request<AgentProviderBindingRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsAuditEventsListParams {
  cursor?: string;
  pageSize?: number;
  action?: AuditAction;
  from_?: string;
  to?: string;
}

export class AiAgentsAuditEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List managed agent audit events */
  async list(agentId: string, params?: AiAgentsAuditEventsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentAuditEvent[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'action', value: params?.action, style: 'form', explode: true, allowReserved: false },
      { name: 'from', value: params?.from_, style: 'form', explode: true, allowReserved: false },
      { name: 'to', value: params?.to, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentAuditEvent[]; }>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/audit_events`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export class AiAgentsStatusApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Update managed agent status */
  async create(agentId: string, body: UpdateAgentStatusRequest, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/status`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsListParams {
  includeDeleted?: boolean;
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAgentsApi {
  private client: HttpClient;
  public readonly status: AiAgentsStatusApi;
  public readonly auditEvents: AiAgentsAuditEventsApi;
  public readonly providerBindings: AiAgentsProviderBindingsApi;
  public readonly sessions: AiAgentsSessionsApi;
  public readonly sessionItems: AiAgentsSessionItemsApi;
  public readonly turns: AiAgentsTurnsApi;
  public readonly interactions: AiAgentsInteractionsApi;
  public readonly checkpoints: AiAgentsCheckpointsApi;
  public readonly sessionRuntimeBindings: AiAgentsSessionRuntimeBindingsApi;
  public readonly tasks: AiAgentsTasksApi;
  public readonly taskRuns: AiAgentsTaskRunsApi;
  public readonly taskRunAttempts: AiAgentsTaskRunAttemptsApi;
  public readonly compositionSlots: AiAgentsCompositionSlotsApi;
  public readonly tools: AiAgentsToolsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.status = new AiAgentsStatusApi(client);
    this.auditEvents = new AiAgentsAuditEventsApi(client);
    this.providerBindings = new AiAgentsProviderBindingsApi(client);
    this.sessions = new AiAgentsSessionsApi(client);
    this.sessionItems = new AiAgentsSessionItemsApi(client);
    this.turns = new AiAgentsTurnsApi(client);
    this.interactions = new AiAgentsInteractionsApi(client);
    this.checkpoints = new AiAgentsCheckpointsApi(client);
    this.sessionRuntimeBindings = new AiAgentsSessionRuntimeBindingsApi(client);
    this.tasks = new AiAgentsTasksApi(client);
    this.taskRuns = new AiAgentsTaskRunsApi(client);
    this.taskRunAttempts = new AiAgentsTaskRunAttemptsApi(client);
    this.compositionSlots = new AiAgentsCompositionSlotsApi(client);
    this.tools = new AiAgentsToolsApi(client);
  }


/** List managed agents for backend administration */
  async list(params?: AiAgentsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentRecord[]; }> {
    const query = buildQueryString([
      { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentRecord[]; }>(appendQueryString(backendApiPath(`/ai/agents`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a managed agent */
  async create(body: CreateAgentRequest, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(backendApiPath(`/ai/agents`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one managed agent */
  async retrieve(agentId: string, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update one managed agent */
  async update(agentId: string, body: UpdateAgentRequest, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Restore one soft-deleted managed agent */
  async restore(agentId: string, body: RestoreAgentRequest, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/restore`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class AiApi {
  public readonly agents: AiAgentsApi;

  constructor(client: HttpClient) {
    this.agents = new AiAgentsApi(client);
  }

}

export function createAiApi(client: HttpClient): AiApi {
  return new AiApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
}
