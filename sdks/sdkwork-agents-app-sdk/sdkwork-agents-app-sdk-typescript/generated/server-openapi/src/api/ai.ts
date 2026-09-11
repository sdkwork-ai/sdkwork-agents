import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { ActivateAgentProviderBindingRequest, AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentEngineCatalog, AgentEngineConfigFileView, AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus, AgentItemFeedbackRecord, AgentModelProviderId, AgentProjectCompositionSlotRecord, AgentProjectMutationRequest, AgentProjectRecord, AgentProjectStatus, AgentProviderBindingRecord, AgentRecord, AgentResourceUserStateRecord, AgentRuntimeExecutionRecord, AgentSessionCheckpointRecord, AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionItemSynchronizationResult, AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentSessionStatus, AgentTaskRecord, AgentTaskRunAttemptRecord, AgentTaskRunRecord, AgentTaskStateChangeRequest, AgentTurnInputQueueEntry, AgentTurnRecord, AgentTurnStreamEvent, AgentWorkspaceMutationRequest, AgentWorkspaceRecord, AgentWorkspaceStatus, AnswerAgentInteractionRequest, AppliedAgentModelConfigurationRecord, AppliedAgentModelSelectionRecord, ApplyAgentModelConfigurationRequest, ApplyAgentModelSelectionRequest, ApproveAgentInteractionRequest, AppUpdateAgentSessionRequest, CancelAgentTaskRequest, CancelAgentTaskRunRequest, CancelAgentTurnRequest, ChangeAgentSessionRuntimeBindingStatusRequest, ClaimAgentInteractionRequest, ClaimNextAgentTurnInputQueueEntryRequest, ClaimNextAgentTurnInputQueueEntryResult, CloseAgentSessionRequest, CreateAgentCompositionSlotRequest, CreateAgentInteractionRequest, CreateAgentPreviewResponseRequest, CreateAgentProjectCompositionSlotRequest, CreateAgentProjectRequest, CreateAgentPromptOptimizationRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, CreateAgentSessionCheckpointRequest, CreateAgentSessionRequest, CreateAgentSessionRuntimeBindingRequest, CreateAgentTaskRequest, CreateAgentTurnInputQueueEntryRequest, CreateAgentTurnRequest, CreateAgentWorkspaceRequest, EnsureDefaultAgentWorkspaceRequest, ExecuteAgentTaskRequest, FailAgentTurnInputQueueEntryRequest, ImportAgentProjectRequest, Int64String, InvalidateAgentSessionCheckpointRequest, McpServerMarketplaceRecord, MediaToolDirectoryEntry, MediaToolInvokeBody, MediaToolInvokeResponse, MigrateModelConfigurationRequest, ModelConfigurationStatusRecord, ModelConfigurationSummaryRecord, PageInfo, ProjectSessionSynchronizationResult, ReorderAgentTurnInputQueueEntriesRequest, ReplaceAgentTaskRequest, ResolveAgentInteractionRequest, RestoreAgentRequest, RestoreAgentSessionCheckpointRequest, RetryAgentTaskRunRequest, RetryAgentTurnInputQueueEntryRequest, SdkWorkPageData, SessionActivitySummary, ToolAssetView, UpdateAgentCompositionSlotRequest, UpdateAgentItemFeedbackRequest, UpdateAgentProjectCompositionSlotRequest, UpdateAgentProjectRequest, UpdateAgentRequest, UpdateAgentSessionRuntimeBindingRequest, UpdateAgentSessionUserStateRequest, UpdateAgentTurnInputQueueEntryRequest, UpdateAgentWorkspaceRequest } from '../types';


export class AiAgentsAssetsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List generated media assets persisted to Drive */
  async list(requestOptions?: ApiRequestOptions): Promise<ToolAssetView[]> {
    return this.client.request<ToolAssetView[]>(appApiPath(`/ai/assets`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }
}

export class AiAgentsToolsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List media tool directory with effective tenant configuration */
  async list(requestOptions?: ApiRequestOptions): Promise<MediaToolDirectoryEntry[]> {
    return this.client.request<MediaToolDirectoryEntry[]>(appApiPath(`/ai/tools`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'data' });
  }

/** Invoke one media tool (optional saveToDrive persistence) */
  async invoke(toolId: string, body: MediaToolInvokeBody, requestOptions?: ApiRequestOptions): Promise<MediaToolInvokeResponse> {
    return this.client.request<MediaToolInvokeResponse>(appApiPath(`/ai/tools/${serializePathParameter(toolId, { name: 'toolId', style: 'simple', explode: false })}/invoke`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }
}

export interface AiAgentsMcpServersListParams {
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAgentsMcpServersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List MCP marketplace entries from agent composition slots */
  async list(params?: AiAgentsMcpServersListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: McpServerMarketplaceRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: McpServerMarketplaceRecord[]; }>(appendQueryString(appApiPath(`/ai/mcp_servers`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export class AiAgentsModelSelectionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Apply a catalog or saved custom model selection to an Agent provider */
  async apply(body: ApplyAgentModelSelectionRequest, requestOptions?: ApiRequestOptions): Promise<AppliedAgentModelSelectionRecord> {
    return this.client.request<AppliedAgentModelSelectionRecord>(appApiPath(`/ai/model_selections/apply`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsModelConfigurationsListParams {
  engineId?: string;
}

export class AiAgentsModelConfigurationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Apply one unified model configuration to an Agent provider */
  async apply(body: ApplyAgentModelConfigurationRequest, requestOptions?: ApiRequestOptions): Promise<AppliedAgentModelConfigurationRecord> {
    return this.client.request<AppliedAgentModelConfigurationRecord>(appApiPath(`/ai/model_configurations/apply`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** List applied model configuration profiles */
  async list(params?: AiAgentsModelConfigurationsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: ModelConfigurationSummaryRecord[]; pageInfo: PageInfo; }> {
    const query = buildQueryString([
      { name: 'engineId', value: params?.engineId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: ModelConfigurationSummaryRecord[]; pageInfo: PageInfo; }>(appendQueryString(appApiPath(`/ai/model_configurations`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Get one applied model configuration profile */
  async get(engineId: string, profileId: string, requestOptions?: ApiRequestOptions): Promise<ModelConfigurationSummaryRecord> {
    return this.client.request<ModelConfigurationSummaryRecord>(appApiPath(`/ai/model_configurations/${serializePathParameter(engineId, { name: 'engineId', style: 'simple', explode: false })}/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Read the provider-native configuration file with credentials masked */
  async configFile(engineId: string, requestOptions?: ApiRequestOptions): Promise<AgentEngineConfigFileView> {
    return this.client.request<AgentEngineConfigFileView>(appApiPath(`/ai/model_configurations/${serializePathParameter(engineId, { name: 'engineId', style: 'simple', explode: false })}/config_file`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Read back the provider-native config state and detect drift */
  async status(engineId: string, profileId: string, requestOptions?: ApiRequestOptions): Promise<ModelConfigurationStatusRecord> {
    return this.client.request<ModelConfigurationStatusRecord>(appApiPath(`/ai/model_configurations/${serializePathParameter(engineId, { name: 'engineId', style: 'simple', explode: false })}/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/status`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Archive a model configuration profile and dematerialize the provider config */
  async archive(engineId: string, profileId: string, requestOptions?: ApiRequestOptions): Promise<ModelConfigurationSummaryRecord> {
    return this.client.request<ModelConfigurationSummaryRecord>(appApiPath(`/ai/model_configurations/${serializePathParameter(engineId, { name: 'engineId', style: 'simple', explode: false })}/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/archive`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, sdkworkUnwrapKind: 'item' });
  }

/** Plan and execute a model configuration profile upgrade */
  async migrate(body: MigrateModelConfigurationRequest, requestOptions?: ApiRequestOptions): Promise<{ profileId: string; engineId: AgentModelProviderId; agentId: string; configurationVersion: string; status: 'draft' | 'active' | 'deprecated' | 'archived'; migrationPlanId: string; }> {
    return this.client.request<{ profileId: string; engineId: AgentModelProviderId; agentId: string; configurationVersion: string; status: 'draft' | 'active' | 'deprecated' | 'archived'; migrationPlanId: string; }>(appApiPath(`/ai/model_configurations/migrate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class AiAgentsAgentEnginesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List canonical agent-engine catalog */
  async list(requestOptions?: ApiRequestOptions): Promise<AgentEngineCatalog> {
    return this.client.request<AgentEngineCatalog>(appApiPath(`/ai/agent_engines`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsCompositionSlotsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsCompositionSlotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List composition slots for one managed agent */
  async list(agentId: string, params?: AiAgentsCompositionSlotsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a composition slot for one managed agent */
  async create(agentId: string, body: CreateAgentCompositionSlotRequest, requestOptions?: ApiRequestOptions): Promise<AgentCompositionSlotRecord> {
    return this.client.request<AgentCompositionSlotRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one managed agent composition slot */
  async retrieve(agentId: string, slotId: string, requestOptions?: ApiRequestOptions): Promise<AgentCompositionSlotRecord> {
    return this.client.request<AgentCompositionSlotRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update one managed agent composition slot */
  async update(agentId: string, slotId: string, body: UpdateAgentCompositionSlotRequest, requestOptions?: ApiRequestOptions): Promise<AgentCompositionSlotRecord> {
    return this.client.request<AgentCompositionSlotRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Delete one managed agent composition slot */
  async delete(agentId: string, slotId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
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
    return this.client.request<SdkWorkPageData & { items: AgentTaskRunAttemptRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}/attempts`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
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
    return this.client.request<SdkWorkPageData & { items: AgentTaskRunRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Retrieve one scheduled task Run */
  async retrieve(agentId: string, taskId: string, runId: string, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Create an idempotent business retry Run from a terminal Run */
  async retry(agentId: string, taskId: string, runId: string, body: RetryAgentTaskRunRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}/retry`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Cancel a pending Run or request cancellation and reconciliation for an active Run */
  async cancel(agentId: string, taskId: string, runId: string, body: CancelAgentTaskRunRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/runs/${serializePathParameter(runId, { name: 'runId', style: 'simple', explode: false })}/cancel`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
    return this.client.request<SdkWorkPageData & { items: AgentTaskRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a scheduled task for one managed agent */
  async create(agentId: string, body: CreateAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one scheduled task */
  async retrieve(agentId: string, taskId: string, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Replace one scheduled task definition and execution policy */
  async update(agentId: string, taskId: string, body: ReplaceAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Pause future materialization for one scheduled task */
  async pause(agentId: string, taskId: string, body: AgentTaskStateChangeRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/pause`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Resume future materialization for one paused scheduled task */
  async resume(agentId: string, taskId: string, body: AgentTaskStateChangeRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/resume`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Cancel one scheduled task */
  async cancel(agentId: string, taskId: string, body: CancelAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRecord> {
    return this.client.request<AgentTaskRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/cancel`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Materialize one idempotent manual Run for an active scheduled task */
  async execute(agentId: string, taskId: string, body: ExecuteAgentTaskRequest, requestOptions?: ApiRequestOptions): Promise<AgentTaskRunRecord> {
    return this.client.request<AgentTaskRunRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/execute`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
    return this.client.request<SdkWorkPageData & { items: AgentSessionRuntimeBindingRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create the current runtime binding for one agent session */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionRuntimeBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one agent session runtime binding */
  async retrieve(agentId: string, sessionId: string, runtimeBindingId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update one agent session runtime binding */
  async update(agentId: string, sessionId: string, runtimeBindingId: string, body: UpdateAgentSessionRuntimeBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Activate one agent session runtime binding as current */
  async activate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/activate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Deactivate one agent session runtime binding */
  async deactivate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.request<AgentSessionRuntimeBindingRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/deactivate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
    return this.client.request<SdkWorkPageData & { items: AgentSessionCheckpointRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create one bounded agent session checkpoint */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionCheckpointRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one agent session checkpoint */
  async retrieve(agentId: string, sessionId: string, checkpointId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Restore one resumable agent session checkpoint */
  async restore(agentId: string, sessionId: string, checkpointId: string, body: RestoreAgentSessionCheckpointRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/restore`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Invalidate one agent session checkpoint */
  async invalidate(agentId: string, sessionId: string, checkpointId: string, body: InvalidateAgentSessionCheckpointRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionCheckpointRecord> {
    return this.client.request<AgentSessionCheckpointRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/invalidate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
    return this.client.request<SdkWorkPageData & { items: AgentInteractionRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create one durable typed or legacy agent interaction */
  async create(agentId: string, sessionId: string, body: CreateAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one durable agent interaction */
  async retrieve(agentId: string, sessionId: string, interactionId: string, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Claim one pending agent interaction for exclusive resolution */
  async claim(agentId: string, sessionId: string, interactionId: string, body: ClaimAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<{ interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }> {
    return this.client.request<{ interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/claim`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Approve or reject one approval interaction */
  async approve(agentId: string, sessionId: string, interactionId: string, body: ApproveAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/approve`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Answer or reject one user-question interaction */
  async answer(agentId: string, sessionId: string, interactionId: string, body: AnswerAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/answer`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Resolve one typed agent interaction */
  async resolve(agentId: string, sessionId: string, interactionId: string, body: ResolveAgentInteractionRequest, requestOptions?: ApiRequestOptions): Promise<AgentInteractionRecord> {
    return this.client.request<AgentInteractionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/resolve`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsTurnInputQueueEntriesListParams {
  page?: number;
  pageSize?: number;
}

export interface AiAgentsTurnInputQueueEntriesDeleteParams {
  expectedVersion: Int64String;
}

export class AiAgentsTurnInputQueueEntriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List durable queued Turn inputs for one Session */
  async list(agentId: string, sessionId: string, params?: AiAgentsTurnInputQueueEntriesListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentTurnInputQueueEntry[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentTurnInputQueueEntry[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Persist one user input in the Session Turn queue */
  async create(agentId: string | number, sessionId: string | number, body: CreateAgentTurnInputQueueEntryRequest, requestOptions?: ApiRequestOptions): Promise<AgentTurnInputQueueEntry> {
    return this.client.request<AgentTurnInputQueueEntry>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Remove all non-executing inputs from the Session Turn queue */
  async clear(agentId: string, sessionId: string, requestOptions?: ApiRequestOptions): Promise<{ clearedCount: Int64String; }> {
    return this.client.request<{ clearedCount: Int64String; }>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue/clear`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, sdkworkUnwrapKind: 'data' });
  }

/** Atomically reorder all non-executing inputs in the Session Turn queue */
  async reorder(agentId: string, sessionId: string, body: ReorderAgentTurnInputQueueEntriesRequest, requestOptions?: ApiRequestOptions): Promise<{ items: AgentTurnInputQueueEntry[]; }> {
    return this.client.request<{ items: AgentTurnInputQueueEntry[]; }>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue/reorder`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

/** Reconcile and lease the FIFO head when the Session has no active Turn */
  async claimNext(agentId: string, sessionId: string, body: ClaimNextAgentTurnInputQueueEntryRequest, requestOptions?: ApiRequestOptions): Promise<ClaimNextAgentTurnInputQueueEntryResult> {
    return this.client.request<ClaimNextAgentTurnInputQueueEntryResult>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue/claim_next`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'data' });
  }

/** Update one non-executing queued Turn input */
  async update(agentId: string, sessionId: string, queueEntryId: string, body: UpdateAgentTurnInputQueueEntryRequest, requestOptions?: ApiRequestOptions): Promise<AgentTurnInputQueueEntry> {
    return this.client.request<AgentTurnInputQueueEntry>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue/${serializePathParameter(queueEntryId, { name: 'queueEntryId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Delete one non-executing queued Turn input */
  async delete(agentId: string, sessionId: string, queueEntryId: string, params: AiAgentsTurnInputQueueEntriesDeleteParams, requestOptions?: ApiRequestOptions): Promise<void> {
    const query = buildQueryString([
      { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<void>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue/${serializePathParameter(queueEntryId, { name: 'queueEntryId', style: 'simple', explode: false })}`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

/** Mark one claimed queue entry failed and pause automatic execution */
  async fail(agentId: string, sessionId: string, queueEntryId: string, body: FailAgentTurnInputQueueEntryRequest, requestOptions?: ApiRequestOptions): Promise<AgentTurnInputQueueEntry> {
    return this.client.request<AgentTurnInputQueueEntry>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue/${serializePathParameter(queueEntryId, { name: 'queueEntryId', style: 'simple', explode: false })}/fail`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Reset one failed queued Turn input for an explicit retry */
  async retry(agentId: string, sessionId: string, queueEntryId: string, body: RetryAgentTurnInputQueueEntryRequest, requestOptions?: ApiRequestOptions): Promise<AgentTurnInputQueueEntry> {
    return this.client.request<AgentTurnInputQueueEntry>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turn_input_queue/${serializePathParameter(queueEntryId, { name: 'queueEntryId', style: 'simple', explode: false })}/retry`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
    return this.client.request<SdkWorkPageData & { items: AgentTurnRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create one idempotent agent turn */
  async stream(agentId: string, sessionId: string, body: CreateAgentTurnRequest, params?: AiAgentsTurnsStreamParams, requestOptions?: ApiRequestOptions): Promise<AsyncIterable<AgentTurnStreamEvent>> {
    const query = buildQueryString([
      { name: 'stream', value: params?.stream, style: 'form', explode: true, allowReserved: false },
      { name: 'event_protocol', value: params?.eventProtocol, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.streamJson<AgentTurnStreamEvent>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json' });
  }

/** Retrieve one durable agent turn */
  async retrieve(agentId: string, sessionId: string, turnId: string, requestOptions?: ApiRequestOptions): Promise<AgentTurnRecord> {
    return this.client.request<AgentTurnRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Request cancellation of one agent turn */
  async cancel(agentId: string, sessionId: string, turnId: string, body: CancelAgentTurnRequest, requestOptions?: ApiRequestOptions): Promise<AgentTurnRecord> {
    return this.client.request<AgentTurnRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}/cancel`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
    return this.client.request<SdkWorkPageData & { items: AgentSessionItemRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Synchronize provider Session history for one Session */
  async synchronize(agentId: string, sessionId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionItemSynchronizationResult> {
    return this.client.request<AgentSessionItemSynchronizationResult>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items/synchronize`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one agent session item */
  async retrieve(agentId: string, sessionId: string, itemId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionItemRecord> {
    return this.client.request<AgentSessionItemRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items/${serializePathParameter(itemId, { name: 'itemId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsItemFeedbackListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsItemFeedbackApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List item feedback for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsItemFeedbackListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentItemFeedbackRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentItemFeedbackRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/item_feedback`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create, update, or clear feedback for one agent session item */
  async update(agentId: string, sessionId: string, itemId: string, body: UpdateAgentItemFeedbackRequest, requestOptions?: ApiRequestOptions): Promise<AgentItemFeedbackRecord> {
    return this.client.request<AgentItemFeedbackRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items/${serializePathParameter(itemId, { name: 'itemId', style: 'simple', explode: false })}/feedback`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsSessionUserStatesListParams {
  page?: number;
  pageSize?: number;
  pinnedOnly?: boolean;
  includeHidden?: boolean;
  sessionIds?: string;
}

export class AiAgentsSessionUserStatesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List per-user state for agent sessions owned by the authenticated user */
  async list(agentId: string, params?: AiAgentsSessionUserStatesListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentResourceUserStateRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'pinned_only', value: params?.pinnedOnly, style: 'form', explode: true, allowReserved: false },
      { name: 'include_hidden', value: params?.includeHidden, style: 'form', explode: true, allowReserved: false },
      { name: 'session_ids', value: params?.sessionIds, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentResourceUserStateRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/user_states`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Retrieve the authenticated user's state for one agent session */
  async retrieve(agentId: string, sessionId: string, requestOptions?: ApiRequestOptions): Promise<AgentResourceUserStateRecord> {
    return this.client.request<AgentResourceUserStateRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/user_state`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update the authenticated user's state for one agent session */
  async update(agentId: string, sessionId: string, body: UpdateAgentSessionUserStateRequest, requestOptions?: ApiRequestOptions): Promise<AgentResourceUserStateRecord> {
    return this.client.request<AgentResourceUserStateRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/user_state`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsSessionsListParams {
  cursor?: string;
  pageSize?: number;
  projectId?: string;
  status?: AgentSessionStatus;
  includeArchived?: boolean;
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
      { name: 'project_id', value: params?.projectId, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'include_archived', value: params?.includeArchived, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentSessionRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a agent session for one managed agent */
  async create(agentId: string, body: CreateAgentSessionRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one agent session */
  async retrieve(agentId: string, sessionId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Rename or move one agent session */
  async update(agentId: string, sessionId: string, body: AppUpdateAgentSessionRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Soft delete one agent session */
  async delete(agentId: string, sessionId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

/** Close one agent session */
  async close(agentId: string, sessionId: string, body: CloseAgentSessionRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/close`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsSessionActivitySummariesListParams {
  cursor?: string;
  pageSize?: number;
  workspaceId?: string;
  projectId?: string;
  agentId?: string;
}

export class AiAgentsSessionActivitySummariesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List the authenticated owner's current Session activity snapshot */
  async list(params?: AiAgentsSessionActivitySummariesListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: SessionActivitySummary[]; pageInfo: PageInfo & { mode: 'cursor'; }; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'workspace_id', value: params?.workspaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'project_id', value: params?.projectId, style: 'form', explode: true, allowReserved: false },
      { name: 'agent_id', value: params?.agentId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: SessionActivitySummary[]; pageInfo: PageInfo & { mode: 'cursor'; }; }>(appendQueryString(appApiPath(`/ai/session_activity_summaries`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface AiAgentsProjectSessionsListParams {
  cursor?: string;
  pageSize?: number;
  status?: AgentSessionStatus;
  includeArchived?: boolean;
}

export class AiAgentsProjectSessionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List agent sessions for one project */
  async list(projectId: string, params?: AiAgentsProjectSessionsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentSessionRecord[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'include_archived', value: params?.includeArchived, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentSessionRecord[]; }>(appendQueryString(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/sessions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create an agent session in one project */
  async create(projectId: string, body: CreateAgentSessionRequest, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/sessions`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Synchronize provider Session inventory for one project */
  async synchronize(projectId: string, requestOptions?: ApiRequestOptions): Promise<ProjectSessionSynchronizationResult> {
    return this.client.request<ProjectSessionSynchronizationResult>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/sessions/synchronize`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one project-scoped agent Session */
  async retrieve(projectId: string, sessionId: string, requestOptions?: ApiRequestOptions): Promise<AgentSessionRecord> {
    return this.client.request<AgentSessionRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsWorkspaceSessionsListParams {
  cursor?: string;
  pageSize?: number;
  status?: AgentSessionStatus;
  includeArchived?: boolean;
}

export class AiAgentsWorkspaceSessionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List agent sessions for one workspace */
  async list(workspaceId: string, params?: AiAgentsWorkspaceSessionsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentSessionRecord[]; }> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'include_archived', value: params?.includeArchived, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentSessionRecord[]; }>(appendQueryString(appApiPath(`/ai/workspaces/${serializePathParameter(workspaceId, { name: 'workspaceId', style: 'simple', explode: false })}/sessions`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export interface AiAgentsProjectCompositionSlotsListParams {
  page?: number;
  pageSize?: number;
  slotKind?: AgentCompositionSlotKind;
  enabled?: boolean;
}

export interface AiAgentsProjectCompositionSlotsDeleteParams {
  expectedVersion: Int64String;
}

export class AiAgentsProjectCompositionSlotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List composition slots for an agent project */
  async list(projectId: string, params?: AiAgentsProjectCompositionSlotsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentProjectCompositionSlotRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'slotKind', value: params?.slotKind, style: 'form', explode: true, allowReserved: false },
      { name: 'enabled', value: params?.enabled, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentProjectCompositionSlotRecord[]; }>(appendQueryString(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Add a composition slot to an agent project */
  async create(projectId: string, body: CreateAgentProjectCompositionSlotRequest, requestOptions?: ApiRequestOptions): Promise<AgentProjectCompositionSlotRecord> {
    return this.client.request<AgentProjectCompositionSlotRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve a project composition slot */
  async retrieve(projectId: string, slotId: string, requestOptions?: ApiRequestOptions): Promise<AgentProjectCompositionSlotRecord> {
    return this.client.request<AgentProjectCompositionSlotRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update a project composition slot */
  async update(projectId: string, slotId: string, body: UpdateAgentProjectCompositionSlotRequest, requestOptions?: ApiRequestOptions): Promise<AgentProjectCompositionSlotRecord> {
    return this.client.request<AgentProjectCompositionSlotRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Soft-delete a project composition slot */
  async delete(projectId: string, slotId: string, params: AiAgentsProjectCompositionSlotsDeleteParams, requestOptions?: ApiRequestOptions): Promise<void> {
    const query = buildQueryString([
      { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<void>(appendQueryString(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }
}

export interface AiAgentsProjectsListParams {
  page?: number;
  pageSize?: number;
  workspaceId?: string;
  q?: string;
  nameExact?: string;
  status?: AgentProjectStatus;
  includeDeleted?: boolean;
}

export class AiAgentsProjectsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List agent projects for the current user */
  async list(params?: AiAgentsProjectsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentProjectRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'workspaceId', value: params?.workspaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'name_exact', value: params?.nameExact, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentProjectRecord[]; }>(appendQueryString(appApiPath(`/ai/projects`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create an agent project */
  async create(body: CreateAgentProjectRequest, requestOptions?: ApiRequestOptions): Promise<AgentProjectRecord> {
    return this.client.request<AgentProjectRecord>(appApiPath(`/ai/projects`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Import or reopen a Workspace-scoped Drive sandbox project */
  async import(body: ImportAgentProjectRequest, requestOptions?: ApiRequestOptions): Promise<AgentProjectRecord> {
    return this.client.request<AgentProjectRecord>(appApiPath(`/ai/projects/import`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve an agent project */
  async retrieve(projectId: string, requestOptions?: ApiRequestOptions): Promise<AgentProjectRecord> {
    return this.client.request<AgentProjectRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update an agent project */
  async update(projectId: string, body: UpdateAgentProjectRequest, requestOptions?: ApiRequestOptions): Promise<AgentProjectRecord> {
    return this.client.request<AgentProjectRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Soft-delete an agent project */
  async delete(projectId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

/** Archive an agent project */
  async archive(projectId: string, body: AgentProjectMutationRequest, requestOptions?: ApiRequestOptions): Promise<AgentProjectRecord> {
    return this.client.request<AgentProjectRecord>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/archive`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class AiAgentsWorkspacesDefaultApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Ensure the current user has a default Workspace */
  async create(body: EnsureDefaultAgentWorkspaceRequest, requestOptions?: ApiRequestOptions): Promise<AgentWorkspaceRecord> {
    return this.client.request<AgentWorkspaceRecord>(appApiPath(`/ai/workspaces/default`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsWorkspacesListParams {
  page?: number;
  pageSize?: number;
  status?: AgentWorkspaceStatus;
  includeDeleted?: boolean;
}

export interface AiAgentsWorkspacesDeleteParams {
  expectedVersion: Int64String;
}

export class AiAgentsWorkspacesApi {
  private client: HttpClient;
  public readonly default: AiAgentsWorkspacesDefaultApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.default = new AiAgentsWorkspacesDefaultApi(client);
  }


/** List Workspaces for the current user */
  async list(params?: AiAgentsWorkspacesListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentWorkspaceRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentWorkspaceRecord[]; }>(appendQueryString(appApiPath(`/ai/workspaces`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a Workspace for the current user */
  async create(body: CreateAgentWorkspaceRequest, requestOptions?: ApiRequestOptions): Promise<AgentWorkspaceRecord> {
    return this.client.request<AgentWorkspaceRecord>(appApiPath(`/ai/workspaces`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve a Workspace owned by the current user */
  async retrieve(workspaceId: string, requestOptions?: ApiRequestOptions): Promise<AgentWorkspaceRecord> {
    return this.client.request<AgentWorkspaceRecord>(appApiPath(`/ai/workspaces/${serializePathParameter(workspaceId, { name: 'workspaceId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update a Workspace owned by the current user */
  async update(workspaceId: string, body: UpdateAgentWorkspaceRequest, requestOptions?: ApiRequestOptions): Promise<AgentWorkspaceRecord> {
    return this.client.request<AgentWorkspaceRecord>(appApiPath(`/ai/workspaces/${serializePathParameter(workspaceId, { name: 'workspaceId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Soft-delete an empty, non-default Workspace */
  async delete(workspaceId: string, params: AiAgentsWorkspacesDeleteParams, requestOptions?: ApiRequestOptions): Promise<void> {
    const query = buildQueryString([
      { name: 'expectedVersion', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<void>(appendQueryString(appApiPath(`/ai/workspaces/${serializePathParameter(workspaceId, { name: 'workspaceId', style: 'simple', explode: false })}`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

/** Archive an empty, non-default Workspace */
  async archive(workspaceId: string, body: AgentWorkspaceMutationRequest, requestOptions?: ApiRequestOptions): Promise<AgentWorkspaceRecord> {
    return this.client.request<AgentWorkspaceRecord>(appApiPath(`/ai/workspaces/${serializePathParameter(workspaceId, { name: 'workspaceId', style: 'simple', explode: false })}/archive`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class AiAgentsPromptOptimizationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a prompt optimization for one managed agent */
  async create(agentId: string, body: CreateAgentPromptOptimizationRequest, requestOptions?: ApiRequestOptions): Promise<AgentRuntimeExecutionRecord> {
    return this.client.request<AgentRuntimeExecutionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/prompt_optimizations`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class AiAgentsPreviewResponsesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a preview response for one managed agent */
  async create(agentId: string, body: CreateAgentPreviewResponseRequest, requestOptions?: ApiRequestOptions): Promise<AgentRuntimeExecutionRecord> {
    return this.client.request<AgentRuntimeExecutionRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/preview_responses`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
    return this.client.request<SdkWorkPageData & { items: AgentProviderBindingRecord[]; }>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a provider binding for one managed agent */
  async create(agentId: string, body: CreateAgentProviderBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentProviderBindingRecord> {
    return this.client.request<AgentProviderBindingRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Activate one managed agent provider binding */
  async activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest, requestOptions?: ApiRequestOptions): Promise<AgentProviderBindingRecord> {
    return this.client.request<AgentProviderBindingRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiAgentsListParams {
  includeDeleted?: boolean;
  scope?: 'market' | 'public' | 'published' | 'mine' | 'workspace';
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAgentsApi {
  private client: HttpClient;
  public readonly providerBindings: AiAgentsProviderBindingsApi;
  public readonly previewResponses: AiAgentsPreviewResponsesApi;
  public readonly promptOptimizations: AiAgentsPromptOptimizationsApi;
  public readonly workspaces: AiAgentsWorkspacesApi;
  public readonly projects: AiAgentsProjectsApi;
  public readonly projectCompositionSlots: AiAgentsProjectCompositionSlotsApi;
  public readonly workspaceSessions: AiAgentsWorkspaceSessionsApi;
  public readonly projectSessions: AiAgentsProjectSessionsApi;
  public readonly sessionActivitySummaries: AiAgentsSessionActivitySummariesApi;
  public readonly sessions: AiAgentsSessionsApi;
  public readonly sessionUserStates: AiAgentsSessionUserStatesApi;
  public readonly itemFeedback: AiAgentsItemFeedbackApi;
  public readonly sessionItems: AiAgentsSessionItemsApi;
  public readonly turns: AiAgentsTurnsApi;
  public readonly turnInputQueueEntries: AiAgentsTurnInputQueueEntriesApi;
  public readonly interactions: AiAgentsInteractionsApi;
  public readonly checkpoints: AiAgentsCheckpointsApi;
  public readonly sessionRuntimeBindings: AiAgentsSessionRuntimeBindingsApi;
  public readonly tasks: AiAgentsTasksApi;
  public readonly taskRuns: AiAgentsTaskRunsApi;
  public readonly taskRunAttempts: AiAgentsTaskRunAttemptsApi;
  public readonly compositionSlots: AiAgentsCompositionSlotsApi;
  public readonly agentEngines: AiAgentsAgentEnginesApi;
  public readonly modelConfigurations: AiAgentsModelConfigurationsApi;
  public readonly modelSelections: AiAgentsModelSelectionsApi;
  public readonly mcpServers: AiAgentsMcpServersApi;
  public readonly tools: AiAgentsToolsApi;
  public readonly assets: AiAgentsAssetsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.providerBindings = new AiAgentsProviderBindingsApi(client);
    this.previewResponses = new AiAgentsPreviewResponsesApi(client);
    this.promptOptimizations = new AiAgentsPromptOptimizationsApi(client);
    this.workspaces = new AiAgentsWorkspacesApi(client);
    this.projects = new AiAgentsProjectsApi(client);
    this.projectCompositionSlots = new AiAgentsProjectCompositionSlotsApi(client);
    this.workspaceSessions = new AiAgentsWorkspaceSessionsApi(client);
    this.projectSessions = new AiAgentsProjectSessionsApi(client);
    this.sessionActivitySummaries = new AiAgentsSessionActivitySummariesApi(client);
    this.sessions = new AiAgentsSessionsApi(client);
    this.sessionUserStates = new AiAgentsSessionUserStatesApi(client);
    this.itemFeedback = new AiAgentsItemFeedbackApi(client);
    this.sessionItems = new AiAgentsSessionItemsApi(client);
    this.turns = new AiAgentsTurnsApi(client);
    this.turnInputQueueEntries = new AiAgentsTurnInputQueueEntriesApi(client);
    this.interactions = new AiAgentsInteractionsApi(client);
    this.checkpoints = new AiAgentsCheckpointsApi(client);
    this.sessionRuntimeBindings = new AiAgentsSessionRuntimeBindingsApi(client);
    this.tasks = new AiAgentsTasksApi(client);
    this.taskRuns = new AiAgentsTaskRunsApi(client);
    this.taskRunAttempts = new AiAgentsTaskRunAttemptsApi(client);
    this.compositionSlots = new AiAgentsCompositionSlotsApi(client);
    this.agentEngines = new AiAgentsAgentEnginesApi(client);
    this.modelConfigurations = new AiAgentsModelConfigurationsApi(client);
    this.modelSelections = new AiAgentsModelSelectionsApi(client);
    this.mcpServers = new AiAgentsMcpServersApi(client);
    this.tools = new AiAgentsToolsApi(client);
    this.assets = new AiAgentsAssetsApi(client);
  }


/** List managed agents */
  async list(params?: AiAgentsListParams, requestOptions?: ApiRequestOptions): Promise<SdkWorkPageData & { items: AgentRecord[]; }> {
    const query = buildQueryString([
      { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
      { name: 'scope', value: params?.scope, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<SdkWorkPageData & { items: AgentRecord[]; }>(appendQueryString(appApiPath(`/ai/agents`), query), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }

/** Create a managed agent */
  async create(body: CreateAgentRequest, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(appApiPath(`/ai/agents`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Retrieve one managed agent */
  async retrieve(agentId: string, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }

/** Update one managed agent */
  async update(agentId: string, body: UpdateAgentRequest, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'PATCH' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

/** Soft-delete one managed agent */
  async delete(agentId: string, requestOptions?: ApiRequestOptions): Promise<void> {
    return this.client.request<void>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'DELETE' as any });
  }

/** Restore one soft-deleted managed agent */
  async restore(agentId: string, body: RestoreAgentRequest, requestOptions?: ApiRequestOptions): Promise<AgentRecord> {
    return this.client.request<AgentRecord>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/restore`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
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
