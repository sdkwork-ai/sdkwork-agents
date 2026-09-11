import 'dart:convert';
import '../http/client.dart';
import '../models.dart';

import 'paths.dart';
import 'response_helpers.dart';


class AiApi {
  final HttpClient _client;

  AiApi(this._client);

  /// List managed agents
  Future<AgentListResponse?> agentsList([bool? includeDeleted, String? scope, int? page, int? pageSize, String? q]) async {
    final query = buildQueryString([
      QueryParameterSpec('include_deleted', includeDeleted, 'form', true, false, null),
      QueryParameterSpec('scope', scope, 'form', true, false, null),
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('q', q, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentListResponse.fromJson(map);
    })();
  }

  /// Create a managed agent
  Future<AgentResponse?> agentsCreate(CreateAgentRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentResponse.fromJson(map);
    })();
  }

  /// Retrieve one managed agent
  Future<AgentResponse?> agentsRetrieve(String agentId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentResponse.fromJson(map);
    })();
  }

  /// Update one managed agent
  Future<AgentResponse?> agentsUpdate(String agentId, UpdateAgentRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentResponse.fromJson(map);
    })();
  }

  /// Soft-delete one managed agent
  Future<void> agentsDelete(String agentId) async {
    await _client.delete(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}'));
  }

  /// Restore one soft-deleted managed agent
  Future<AgentResponse?> agentsRestore(String agentId, RestoreAgentRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/restore'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentResponse.fromJson(map);
    })();
  }

  /// List provider bindings for one managed agent
  Future<AgentProviderBindingListResponse?> agentsProviderBindingsList(String agentId, [int? page, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/provider_bindings'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProviderBindingListResponse.fromJson(map);
    })();
  }

  /// Create a provider binding for one managed agent
  Future<AgentProviderBindingResponse?> agentsProviderBindingsCreate(String agentId, CreateAgentProviderBindingRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/provider_bindings'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProviderBindingResponse.fromJson(map);
    })();
  }

  /// Activate one managed agent provider binding
  Future<AgentProviderBindingResponse?> agentsProviderBindingsActivate(String agentId, String bindingId, ActivateAgentProviderBindingRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/provider_bindings/${serializePathParameter(bindingId, const PathParameterSpec('bindingId', 'simple', false))}/activate'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProviderBindingResponse.fromJson(map);
    })();
  }

  /// Create a preview response for one managed agent
  Future<AgentRuntimeExecutionResponse?> agentsPreviewResponsesCreate(String agentId, CreateAgentPreviewResponseRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/preview_responses'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentRuntimeExecutionResponse.fromJson(map);
    })();
  }

  /// Create a prompt optimization for one managed agent
  Future<AgentRuntimeExecutionResponse?> agentsPromptOptimizationsCreate(String agentId, CreateAgentPromptOptimizationRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/prompt_optimizations'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentRuntimeExecutionResponse.fromJson(map);
    })();
  }

  /// List Workspaces for the current user
  Future<AgentWorkspaceListResponse?> agentsWorkspacesList([int? page, int? pageSize, String? status, bool? includeDeleted]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('include_deleted', includeDeleted, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/workspaces'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentWorkspaceListResponse.fromJson(map);
    })();
  }

  /// Create a Workspace for the current user
  Future<AgentWorkspaceResponse?> agentsWorkspacesCreate(CreateAgentWorkspaceRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/workspaces'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentWorkspaceResponse.fromJson(map);
    })();
  }

  /// Ensure the current user has a default Workspace
  Future<AgentWorkspaceResponse?> agentsWorkspacesDefaultCreate(EnsureDefaultAgentWorkspaceRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/workspaces/default'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentWorkspaceResponse.fromJson(map);
    })();
  }

  /// Retrieve a Workspace owned by the current user
  Future<AgentWorkspaceResponse?> agentsWorkspacesRetrieve(String workspaceId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/workspaces/${serializePathParameter(workspaceId, const PathParameterSpec('workspaceId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentWorkspaceResponse.fromJson(map);
    })();
  }

  /// Update a Workspace owned by the current user
  Future<AgentWorkspaceResponse?> agentsWorkspacesUpdate(String workspaceId, UpdateAgentWorkspaceRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/workspaces/${serializePathParameter(workspaceId, const PathParameterSpec('workspaceId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentWorkspaceResponse.fromJson(map);
    })();
  }

  /// Soft-delete an empty, non-default Workspace
  Future<void> agentsWorkspacesDelete(String workspaceId, String expectedVersion) async {
    final query = buildQueryString([
      QueryParameterSpec('expectedVersion', expectedVersion, 'form', true, false, null)
    ]);
    await _client.delete(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/workspaces/${serializePathParameter(workspaceId, const PathParameterSpec('workspaceId', 'simple', false))}'), query));
  }

  /// Archive an empty, non-default Workspace
  Future<AgentWorkspaceResponse?> agentsWorkspacesArchive(String workspaceId, AgentWorkspaceMutationRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/workspaces/${serializePathParameter(workspaceId, const PathParameterSpec('workspaceId', 'simple', false))}/archive'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentWorkspaceResponse.fromJson(map);
    })();
  }

  /// List agent projects for the current user
  Future<AgentProjectListResponse?> agentsProjectsList([int? page, int? pageSize, String? workspaceId, String? q, String? nameExact, String? status, bool? includeDeleted]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('workspaceId', workspaceId, 'form', true, false, null),
      QueryParameterSpec('q', q, 'form', true, false, null),
      QueryParameterSpec('name_exact', nameExact, 'form', true, false, null),
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('include_deleted', includeDeleted, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/projects'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectListResponse.fromJson(map);
    })();
  }

  /// Create an agent project
  Future<AgentProjectResponse?> agentsProjectsCreate(CreateAgentProjectRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/projects'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectResponse.fromJson(map);
    })();
  }

  /// Import or reopen a Workspace-scoped Drive sandbox project
  Future<AgentProjectResponse?> agentsProjectsImport(ImportAgentProjectRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/projects/import'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectResponse.fromJson(map);
    })();
  }

  /// Retrieve an agent project
  Future<AgentProjectResponse?> agentsProjectsRetrieve(String projectId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectResponse.fromJson(map);
    })();
  }

  /// Update an agent project
  Future<AgentProjectResponse?> agentsProjectsUpdate(String projectId, UpdateAgentProjectRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectResponse.fromJson(map);
    })();
  }

  /// Soft-delete an agent project
  Future<void> agentsProjectsDelete(String projectId) async {
    await _client.delete(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}'));
  }

  /// Archive an agent project
  Future<AgentProjectResponse?> agentsProjectsArchive(String projectId, AgentProjectMutationRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/archive'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectResponse.fromJson(map);
    })();
  }

  /// List composition slots for an agent project
  Future<AgentProjectCompositionSlotListResponse?> agentsProjectCompositionSlotsList(String projectId, [int? page, int? pageSize, String? slotKind, bool? enabled]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('slotKind', slotKind, 'form', true, false, null),
      QueryParameterSpec('enabled', enabled, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/composition_slots'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectCompositionSlotListResponse.fromJson(map);
    })();
  }

  /// Add a composition slot to an agent project
  Future<AgentProjectCompositionSlotResponse?> agentsProjectCompositionSlotsCreate(String projectId, CreateAgentProjectCompositionSlotRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/composition_slots'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectCompositionSlotResponse.fromJson(map);
    })();
  }

  /// Retrieve a project composition slot
  Future<AgentProjectCompositionSlotResponse?> agentsProjectCompositionSlotsRetrieve(String projectId, String slotId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/composition_slots/${serializePathParameter(slotId, const PathParameterSpec('slotId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectCompositionSlotResponse.fromJson(map);
    })();
  }

  /// Update a project composition slot
  Future<AgentProjectCompositionSlotResponse?> agentsProjectCompositionSlotsUpdate(String projectId, String slotId, UpdateAgentProjectCompositionSlotRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/composition_slots/${serializePathParameter(slotId, const PathParameterSpec('slotId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentProjectCompositionSlotResponse.fromJson(map);
    })();
  }

  /// Soft-delete a project composition slot
  Future<void> agentsProjectCompositionSlotsDelete(String projectId, String slotId, String expectedVersion) async {
    final query = buildQueryString([
      QueryParameterSpec('expected_version', expectedVersion, 'form', true, false, null)
    ]);
    await _client.delete(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/composition_slots/${serializePathParameter(slotId, const PathParameterSpec('slotId', 'simple', false))}'), query));
  }

  /// List agent sessions for one workspace
  Future<AgentSessionListResponse?> agentsWorkspaceSessionsList(String workspaceId, [String? cursor, int? pageSize, String? status, bool? includeArchived]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('include_archived', includeArchived, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/workspaces/${serializePathParameter(workspaceId, const PathParameterSpec('workspaceId', 'simple', false))}/sessions'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionListResponse.fromJson(map);
    })();
  }

  /// List agent sessions for one project
  Future<AgentSessionListResponse?> agentsProjectSessionsList(String projectId, [String? cursor, int? pageSize, String? status, bool? includeArchived]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('include_archived', includeArchived, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/sessions'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionListResponse.fromJson(map);
    })();
  }

  /// Create an agent session in one project
  Future<AgentSessionResponse?> agentsProjectSessionsCreate(String projectId, CreateAgentSessionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/sessions'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionResponse.fromJson(map);
    })();
  }

  /// Synchronize provider Session inventory for one project
  Future<ProjectSessionSynchronizationResponse?> agentsProjectSessionsSynchronize(String projectId) async {
    final response = await _client.post(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/sessions/synchronize'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ProjectSessionSynchronizationResponse.fromJson(map);
    })();
  }

  /// Retrieve one project-scoped agent Session
  Future<AgentSessionResponse?> agentsProjectSessionsRetrieve(String projectId, String sessionId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/projects/${serializePathParameter(projectId, const PathParameterSpec('projectId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionResponse.fromJson(map);
    })();
  }

  /// List the authenticated owner's current Session activity snapshot
  Future<SessionActivitySummaryListResponse?> agentsSessionActivitySummariesList([String? cursor, int? pageSize, String? workspaceId, String? projectId, String? agentId]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('workspace_id', workspaceId, 'form', true, false, null),
      QueryParameterSpec('project_id', projectId, 'form', true, false, null),
      QueryParameterSpec('agent_id', agentId, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/session_activity_summaries'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : SessionActivitySummaryListResponse.fromJson(map);
    })();
  }

  /// List agent sessions for one managed agent
  Future<AgentSessionListResponse?> agentsSessionsList(String agentId, [String? cursor, int? pageSize, String? projectId, String? status, bool? includeArchived]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('project_id', projectId, 'form', true, false, null),
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('include_archived', includeArchived, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionListResponse.fromJson(map);
    })();
  }

  /// Create a agent session for one managed agent
  Future<AgentSessionResponse?> agentsSessionsCreate(String agentId, CreateAgentSessionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionResponse.fromJson(map);
    })();
  }

  /// List per-user state for agent sessions owned by the authenticated user
  Future<AgentResourceUserStateListResponse?> agentsSessionUserStatesList(String agentId, [int? page, int? pageSize, bool? pinnedOnly, bool? includeHidden, String? sessionIds]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('pinned_only', pinnedOnly, 'form', true, false, null),
      QueryParameterSpec('include_hidden', includeHidden, 'form', true, false, null),
      QueryParameterSpec('session_ids', sessionIds, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/user_states'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentResourceUserStateListResponse.fromJson(map);
    })();
  }

  /// Retrieve one agent session
  Future<AgentSessionResponse?> agentsSessionsRetrieve(String agentId, String sessionId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionResponse.fromJson(map);
    })();
  }

  /// Rename or move one agent session
  Future<AgentSessionResponse?> agentsSessionsUpdate(String agentId, String sessionId, AppUpdateAgentSessionRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionResponse.fromJson(map);
    })();
  }

  /// Soft delete one agent session
  Future<void> agentsSessionsDelete(String agentId, String sessionId) async {
    await _client.delete(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}'));
  }

  /// Close one agent session
  Future<AgentSessionResponse?> agentsSessionsClose(String agentId, String sessionId, CloseAgentSessionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/close'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionResponse.fromJson(map);
    })();
  }

  /// Retrieve the authenticated user's state for one agent session
  Future<AgentResourceUserStateResponse?> agentsSessionUserStatesRetrieve(String agentId, String sessionId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/user_state'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentResourceUserStateResponse.fromJson(map);
    })();
  }

  /// Update the authenticated user's state for one agent session
  Future<AgentResourceUserStateResponse?> agentsSessionUserStatesUpdate(String agentId, String sessionId, UpdateAgentSessionUserStateRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/user_state'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentResourceUserStateResponse.fromJson(map);
    })();
  }

  /// List item feedback for one agent session
  Future<AgentItemFeedbackListResponse?> agentsItemFeedbackList(String agentId, String sessionId, [int? page, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/item_feedback'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentItemFeedbackListResponse.fromJson(map);
    })();
  }

  /// List ordered items for one agent session
  Future<AgentSessionItemListResponse?> agentsSessionItemsList(String agentId, String sessionId, [String? cursor, int? pageSize, String? kind, String? status, String? sort]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('kind', kind, 'form', true, false, null),
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('sort', sort, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/items'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionItemListResponse.fromJson(map);
    })();
  }

  /// Synchronize provider Session history for one Session
  Future<AgentSessionItemSynchronizeResponse?> agentsSessionItemsSynchronize(String agentId, String sessionId) async {
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/items/synchronize'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionItemSynchronizeResponse.fromJson(map);
    })();
  }

  /// Retrieve one agent session item
  Future<AgentSessionItemResponse?> agentsSessionItemsRetrieve(String agentId, String sessionId, String itemId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/items/${serializePathParameter(itemId, const PathParameterSpec('itemId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionItemResponse.fromJson(map);
    })();
  }

  /// Create, update, or clear feedback for one agent session item
  Future<AgentItemFeedbackResponse?> agentsItemFeedbackUpdate(String agentId, String sessionId, String itemId, UpdateAgentItemFeedbackRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/items/${serializePathParameter(itemId, const PathParameterSpec('itemId', 'simple', false))}/feedback'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentItemFeedbackResponse.fromJson(map);
    })();
  }

  /// List durable turns for one agent session
  Future<AgentTurnListResponse?> agentsTurnsList(String agentId, String sessionId, [String? cursor, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turns'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnListResponse.fromJson(map);
    })();
  }

  /// Create one idempotent agent turn
  Stream<AgentTurnStreamEvent> agentsTurnsStream(String agentId, String sessionId, CreateAgentTurnRequest body, [bool? stream, String? eventProtocol]) {
    final query = buildQueryString([
      QueryParameterSpec('stream', stream, 'form', true, false, null),
      QueryParameterSpec('event_protocol', eventProtocol, 'form', true, false, null)
    ]);
    final payload = body.toJson();
    return _client.streamJson(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turns'), query), body: payload, contentType: 'application/json')
        .map((event) => AgentTurnStreamEvent.fromJson(event));
  }

  /// Retrieve one durable agent turn
  Future<AgentTurnResponse?> agentsTurnsRetrieve(String agentId, String sessionId, String turnId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turns/${serializePathParameter(turnId, const PathParameterSpec('turnId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnResponse.fromJson(map);
    })();
  }

  /// Request cancellation of one agent turn
  Future<AgentTurnResponse?> agentsTurnsCancel(String agentId, String sessionId, String turnId, CancelAgentTurnRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turns/${serializePathParameter(turnId, const PathParameterSpec('turnId', 'simple', false))}/cancel'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnResponse.fromJson(map);
    })();
  }

  /// List durable queued Turn inputs for one Session
  Future<AgentTurnInputQueueEntryListResponse?> agentsTurnInputQueueEntriesList(String agentId, String sessionId, [int? page, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnInputQueueEntryListResponse.fromJson(map);
    })();
  }

  /// Persist one user input in the Session Turn queue
  Future<AgentTurnInputQueueEntryResponse?> agentsTurnInputQueueEntriesCreate(String agentId, String sessionId, CreateAgentTurnInputQueueEntryRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnInputQueueEntryResponse.fromJson(map);
    })();
  }

  /// Remove all non-executing inputs from the Session Turn queue
  Future<ClearAgentTurnInputQueueEntriesResponse?> agentsTurnInputQueueEntriesClear(String agentId, String sessionId) async {
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue/clear'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ClearAgentTurnInputQueueEntriesResponse.fromJson(map);
    })();
  }

  /// Atomically reorder all non-executing inputs in the Session Turn queue
  Future<ReorderAgentTurnInputQueueEntriesResponse?> agentsTurnInputQueueEntriesReorder(String agentId, String sessionId, ReorderAgentTurnInputQueueEntriesRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue/reorder'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ReorderAgentTurnInputQueueEntriesResponse.fromJson(map);
    })();
  }

  /// Reconcile and lease the FIFO head when the Session has no active Turn
  Future<ClaimNextAgentTurnInputQueueEntryResponse?> agentsTurnInputQueueEntriesClaimNext(String agentId, String sessionId, ClaimNextAgentTurnInputQueueEntryRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue/claim_next'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ClaimNextAgentTurnInputQueueEntryResponse.fromJson(map);
    })();
  }

  /// Update one non-executing queued Turn input
  Future<AgentTurnInputQueueEntryResponse?> agentsTurnInputQueueEntriesUpdate(String agentId, String sessionId, String queueEntryId, UpdateAgentTurnInputQueueEntryRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue/${serializePathParameter(queueEntryId, const PathParameterSpec('queueEntryId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnInputQueueEntryResponse.fromJson(map);
    })();
  }

  /// Delete one non-executing queued Turn input
  Future<void> agentsTurnInputQueueEntriesDelete(String agentId, String sessionId, String queueEntryId, String expectedVersion) async {
    final query = buildQueryString([
      QueryParameterSpec('expected_version', expectedVersion, 'form', true, false, null)
    ]);
    await _client.delete(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue/${serializePathParameter(queueEntryId, const PathParameterSpec('queueEntryId', 'simple', false))}'), query));
  }

  /// Mark one claimed queue entry failed and pause automatic execution
  Future<AgentTurnInputQueueEntryResponse?> agentsTurnInputQueueEntriesFail(String agentId, String sessionId, String queueEntryId, FailAgentTurnInputQueueEntryRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue/${serializePathParameter(queueEntryId, const PathParameterSpec('queueEntryId', 'simple', false))}/fail'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnInputQueueEntryResponse.fromJson(map);
    })();
  }

  /// Reset one failed queued Turn input for an explicit retry
  Future<AgentTurnInputQueueEntryResponse?> agentsTurnInputQueueEntriesRetry(String agentId, String sessionId, String queueEntryId, RetryAgentTurnInputQueueEntryRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/turn_input_queue/${serializePathParameter(queueEntryId, const PathParameterSpec('queueEntryId', 'simple', false))}/retry'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTurnInputQueueEntryResponse.fromJson(map);
    })();
  }

  /// List durable interactions for one agent session
  Future<AgentInteractionListResponse?> agentsInteractionsList(String agentId, String sessionId, [String? cursor, int? pageSize, String? kind, String? status]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('kind', kind, 'form', true, false, null),
      QueryParameterSpec('status', status, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/interactions'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentInteractionListResponse.fromJson(map);
    })();
  }

  /// Create one durable typed or legacy agent interaction
  Future<AgentInteractionResponse?> agentsInteractionsCreate(String agentId, String sessionId, CreateAgentInteractionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/interactions'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentInteractionResponse.fromJson(map);
    })();
  }

  /// Retrieve one durable agent interaction
  Future<AgentInteractionResponse?> agentsInteractionsRetrieve(String agentId, String sessionId, String interactionId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/interactions/${serializePathParameter(interactionId, const PathParameterSpec('interactionId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentInteractionResponse.fromJson(map);
    })();
  }

  /// Claim one pending agent interaction for exclusive resolution
  Future<AgentInteractionClaimResponse?> agentsInteractionsClaim(String agentId, String sessionId, String interactionId, ClaimAgentInteractionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/interactions/${serializePathParameter(interactionId, const PathParameterSpec('interactionId', 'simple', false))}/claim'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentInteractionClaimResponse.fromJson(map);
    })();
  }

  /// Approve or reject one approval interaction
  Future<AgentInteractionResponse?> agentsInteractionsApprove(String agentId, String sessionId, String interactionId, ApproveAgentInteractionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/interactions/${serializePathParameter(interactionId, const PathParameterSpec('interactionId', 'simple', false))}/approve'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentInteractionResponse.fromJson(map);
    })();
  }

  /// Answer or reject one user-question interaction
  Future<AgentInteractionResponse?> agentsInteractionsAnswer(String agentId, String sessionId, String interactionId, AnswerAgentInteractionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/interactions/${serializePathParameter(interactionId, const PathParameterSpec('interactionId', 'simple', false))}/answer'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentInteractionResponse.fromJson(map);
    })();
  }

  /// Resolve one typed agent interaction
  Future<AgentInteractionResponse?> agentsInteractionsResolve(String agentId, String sessionId, String interactionId, ResolveAgentInteractionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/interactions/${serializePathParameter(interactionId, const PathParameterSpec('interactionId', 'simple', false))}/resolve'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentInteractionResponse.fromJson(map);
    })();
  }

  /// List resumable checkpoints for one agent session
  Future<AgentSessionCheckpointListResponse?> agentsCheckpointsList(String agentId, String sessionId, [int? page, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/checkpoints'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionCheckpointListResponse.fromJson(map);
    })();
  }

  /// Create one bounded agent session checkpoint
  Future<AgentSessionCheckpointResponse?> agentsCheckpointsCreate(String agentId, String sessionId, CreateAgentSessionCheckpointRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/checkpoints'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionCheckpointResponse.fromJson(map);
    })();
  }

  /// Retrieve one agent session checkpoint
  Future<AgentSessionCheckpointResponse?> agentsCheckpointsRetrieve(String agentId, String sessionId, String checkpointId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/checkpoints/${serializePathParameter(checkpointId, const PathParameterSpec('checkpointId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionCheckpointResponse.fromJson(map);
    })();
  }

  /// Restore one resumable agent session checkpoint
  Future<AgentSessionCheckpointResponse?> agentsCheckpointsRestore(String agentId, String sessionId, String checkpointId, RestoreAgentSessionCheckpointRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/checkpoints/${serializePathParameter(checkpointId, const PathParameterSpec('checkpointId', 'simple', false))}/restore'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionCheckpointResponse.fromJson(map);
    })();
  }

  /// Invalidate one agent session checkpoint
  Future<AgentSessionCheckpointResponse?> agentsCheckpointsInvalidate(String agentId, String sessionId, String checkpointId, InvalidateAgentSessionCheckpointRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/checkpoints/${serializePathParameter(checkpointId, const PathParameterSpec('checkpointId', 'simple', false))}/invalidate'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionCheckpointResponse.fromJson(map);
    })();
  }

  /// List runtime bindings for one agent session
  Future<AgentSessionRuntimeBindingListResponse?> agentsSessionRuntimeBindingsList(String agentId, String sessionId, [int? page, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/runtime_bindings'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionRuntimeBindingListResponse.fromJson(map);
    })();
  }

  /// Create the current runtime binding for one agent session
  Future<AgentSessionRuntimeBindingResponse?> agentsSessionRuntimeBindingsCreate(String agentId, String sessionId, CreateAgentSessionRuntimeBindingRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/runtime_bindings'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionRuntimeBindingResponse.fromJson(map);
    })();
  }

  /// Retrieve one agent session runtime binding
  Future<AgentSessionRuntimeBindingResponse?> agentsSessionRuntimeBindingsRetrieve(String agentId, String sessionId, String runtimeBindingId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/runtime_bindings/${serializePathParameter(runtimeBindingId, const PathParameterSpec('runtimeBindingId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionRuntimeBindingResponse.fromJson(map);
    })();
  }

  /// Update one agent session runtime binding
  Future<AgentSessionRuntimeBindingResponse?> agentsSessionRuntimeBindingsUpdate(String agentId, String sessionId, String runtimeBindingId, UpdateAgentSessionRuntimeBindingRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/runtime_bindings/${serializePathParameter(runtimeBindingId, const PathParameterSpec('runtimeBindingId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionRuntimeBindingResponse.fromJson(map);
    })();
  }

  /// Activate one agent session runtime binding as current
  Future<AgentSessionRuntimeBindingResponse?> agentsSessionRuntimeBindingsActivate(String agentId, String sessionId, String runtimeBindingId, ChangeAgentSessionRuntimeBindingStatusRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/runtime_bindings/${serializePathParameter(runtimeBindingId, const PathParameterSpec('runtimeBindingId', 'simple', false))}/activate'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionRuntimeBindingResponse.fromJson(map);
    })();
  }

  /// Deactivate one agent session runtime binding
  Future<AgentSessionRuntimeBindingResponse?> agentsSessionRuntimeBindingsDeactivate(String agentId, String sessionId, String runtimeBindingId, ChangeAgentSessionRuntimeBindingStatusRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/sessions/${serializePathParameter(sessionId, const PathParameterSpec('sessionId', 'simple', false))}/runtime_bindings/${serializePathParameter(runtimeBindingId, const PathParameterSpec('runtimeBindingId', 'simple', false))}/deactivate'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentSessionRuntimeBindingResponse.fromJson(map);
    })();
  }

  /// List scheduled tasks for one managed agent
  Future<AgentTaskListResponse?> agentsTasksList(String agentId, [String? status, String? cursor, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskListResponse.fromJson(map);
    })();
  }

  /// Create a scheduled task for one managed agent
  Future<AgentTaskResponse?> agentsTasksCreate(String agentId, CreateAgentTaskRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskResponse.fromJson(map);
    })();
  }

  /// Retrieve one scheduled task
  Future<AgentTaskResponse?> agentsTasksRetrieve(String agentId, String taskId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskResponse.fromJson(map);
    })();
  }

  /// Replace one scheduled task definition and execution policy
  Future<AgentTaskResponse?> agentsTasksUpdate(String agentId, String taskId, ReplaceAgentTaskRequest body) async {
    final payload = body.toJson();
    final response = await _client.put(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskResponse.fromJson(map);
    })();
  }

  /// Pause future materialization for one scheduled task
  Future<AgentTaskResponse?> agentsTasksPause(String agentId, String taskId, AgentTaskStateChangeRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/pause'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskResponse.fromJson(map);
    })();
  }

  /// Resume future materialization for one paused scheduled task
  Future<AgentTaskResponse?> agentsTasksResume(String agentId, String taskId, AgentTaskStateChangeRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/resume'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskResponse.fromJson(map);
    })();
  }

  /// Cancel one scheduled task
  Future<AgentTaskResponse?> agentsTasksCancel(String agentId, String taskId, CancelAgentTaskRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/cancel'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskResponse.fromJson(map);
    })();
  }

  /// Materialize one idempotent manual Run for an active scheduled task
  Future<AgentTaskRunResponse?> agentsTasksExecute(String agentId, String taskId, ExecuteAgentTaskRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/execute'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskRunResponse.fromJson(map);
    })();
  }

  /// List Runs for one scheduled task using opaque keyset pagination
  Future<AgentTaskRunListResponse?> agentsTaskRunsList(String agentId, String taskId, [String? status, String? triggerKind, String? cursor, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('status', status, 'form', true, false, null),
      QueryParameterSpec('trigger_kind', triggerKind, 'form', true, false, null),
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/runs'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskRunListResponse.fromJson(map);
    })();
  }

  /// Retrieve one scheduled task Run
  Future<AgentTaskRunResponse?> agentsTaskRunsRetrieve(String agentId, String taskId, String runId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/runs/${serializePathParameter(runId, const PathParameterSpec('runId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskRunResponse.fromJson(map);
    })();
  }

  /// Create an idempotent business retry Run from a terminal Run
  Future<AgentTaskRunResponse?> agentsTaskRunsRetry(String agentId, String taskId, String runId, RetryAgentTaskRunRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/runs/${serializePathParameter(runId, const PathParameterSpec('runId', 'simple', false))}/retry'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskRunResponse.fromJson(map);
    })();
  }

  /// Cancel a pending Run or request cancellation and reconciliation for an active Run
  Future<AgentTaskRunResponse?> agentsTaskRunsCancel(String agentId, String taskId, String runId, CancelAgentTaskRunRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/runs/${serializePathParameter(runId, const PathParameterSpec('runId', 'simple', false))}/cancel'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskRunResponse.fromJson(map);
    })();
  }

  /// List execution Attempts for one scheduled task Run
  Future<AgentTaskRunAttemptListResponse?> agentsTaskRunAttemptsList(String agentId, String taskId, String runId, [String? cursor, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('cursor', cursor, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/tasks/${serializePathParameter(taskId, const PathParameterSpec('taskId', 'simple', false))}/runs/${serializePathParameter(runId, const PathParameterSpec('runId', 'simple', false))}/attempts'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentTaskRunAttemptListResponse.fromJson(map);
    })();
  }

  /// List composition slots for one managed agent
  Future<AgentCompositionSlotListResponse?> agentsCompositionSlotsList(String agentId, [int? page, int? pageSize]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/composition_slots'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentCompositionSlotListResponse.fromJson(map);
    })();
  }

  /// Create a composition slot for one managed agent
  Future<AgentCompositionSlotResponse?> agentsCompositionSlotsCreate(String agentId, CreateAgentCompositionSlotRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/composition_slots'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentCompositionSlotResponse.fromJson(map);
    })();
  }

  /// Retrieve one managed agent composition slot
  Future<AgentCompositionSlotResponse?> agentsCompositionSlotsRetrieve(String agentId, String slotId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/composition_slots/${serializePathParameter(slotId, const PathParameterSpec('slotId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentCompositionSlotResponse.fromJson(map);
    })();
  }

  /// Update one managed agent composition slot
  Future<AgentCompositionSlotResponse?> agentsCompositionSlotsUpdate(String agentId, String slotId, UpdateAgentCompositionSlotRequest body) async {
    final payload = body.toJson();
    final response = await _client.patch(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/composition_slots/${serializePathParameter(slotId, const PathParameterSpec('slotId', 'simple', false))}'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentCompositionSlotResponse.fromJson(map);
    })();
  }

  /// Delete one managed agent composition slot
  Future<void> agentsCompositionSlotsDelete(String agentId, String slotId) async {
    await _client.delete(ApiPaths.appPath('/ai/agents/${serializePathParameter(agentId, const PathParameterSpec('agentId', 'simple', false))}/composition_slots/${serializePathParameter(slotId, const PathParameterSpec('slotId', 'simple', false))}'));
  }

  /// List canonical agent-engine catalog
  Future<AgentEngineCatalogListResponse?> agentsEnginesList() async {
    final response = await _client.get(ApiPaths.appPath('/ai/agent_engines'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentEngineCatalogListResponse.fromJson(map);
    })();
  }

  /// Apply one unified model configuration to an Agent provider
  Future<AppliedAgentModelConfigurationResponse?> agentsModelConfigurationsApply(ApplyAgentModelConfigurationRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/model_configurations/apply'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AppliedAgentModelConfigurationResponse.fromJson(map);
    })();
  }

  /// Apply a catalog or saved custom model selection to an Agent provider
  Future<AppliedAgentModelSelectionResponse?> agentsModelSelectionsApply(ApplyAgentModelSelectionRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/model_selections/apply'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AppliedAgentModelSelectionResponse.fromJson(map);
    })();
  }

  /// List applied model configuration profiles
  Future<ModelConfigurationSummaryListResponse?> agentsModelConfigurationsList([String? engineId]) async {
    final query = buildQueryString([
      QueryParameterSpec('engineId', engineId, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/model_configurations'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ModelConfigurationSummaryListResponse.fromJson(map);
    })();
  }

  /// Get one applied model configuration profile
  Future<ModelConfigurationSummaryResponse?> agentsModelConfigurationsGet(String engineId, String profileId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/model_configurations/${serializePathParameter(engineId, const PathParameterSpec('engineId', 'simple', false))}/${serializePathParameter(profileId, const PathParameterSpec('profileId', 'simple', false))}'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ModelConfigurationSummaryResponse.fromJson(map);
    })();
  }

  /// Read the provider-native configuration file with credentials masked
  Future<AgentEngineConfigFileResponse?> agentsModelConfigurationsConfigFile(String engineId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/model_configurations/${serializePathParameter(engineId, const PathParameterSpec('engineId', 'simple', false))}/config_file'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : AgentEngineConfigFileResponse.fromJson(map);
    })();
  }

  /// Read back the provider-native config state and detect drift
  Future<ModelConfigurationStatusResponse?> agentsModelConfigurationsStatus(String engineId, String profileId) async {
    final response = await _client.get(ApiPaths.appPath('/ai/model_configurations/${serializePathParameter(engineId, const PathParameterSpec('engineId', 'simple', false))}/${serializePathParameter(profileId, const PathParameterSpec('profileId', 'simple', false))}/status'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ModelConfigurationStatusResponse.fromJson(map);
    })();
  }

  /// Archive a model configuration profile and dematerialize the provider config
  Future<ModelConfigurationSummaryResponse?> agentsModelConfigurationsArchive(String engineId, String profileId) async {
    final response = await _client.post(ApiPaths.appPath('/ai/model_configurations/${serializePathParameter(engineId, const PathParameterSpec('engineId', 'simple', false))}/${serializePathParameter(profileId, const PathParameterSpec('profileId', 'simple', false))}/archive'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ModelConfigurationSummaryResponse.fromJson(map);
    })();
  }

  /// Plan and execute a model configuration profile upgrade
  Future<MigratedModelConfigurationResponse?> agentsModelConfigurationsMigrate(MigrateModelConfigurationRequest body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/model_configurations/migrate'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : MigratedModelConfigurationResponse.fromJson(map);
    })();
  }

  /// List MCP marketplace entries from agent composition slots
  Future<McpServerMarketplaceListResponse?> agentsMcpServersList([int? page, int? pageSize, String? q]) async {
    final query = buildQueryString([
      QueryParameterSpec('page', page, 'form', true, false, null),
      QueryParameterSpec('page_size', pageSize, 'form', true, false, null),
      QueryParameterSpec('q', q, 'form', true, false, null)
    ]);
    final response = await _client.get(ApiPaths.appendQueryString(ApiPaths.appPath('/ai/mcp_servers'), query));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : McpServerMarketplaceListResponse.fromJson(map);
    })();
  }

  /// List media tool directory with effective tenant configuration
  Future<MediaToolDirectoryResponse?> agentsToolsList() async {
    final response = await _client.get(ApiPaths.appPath('/ai/tools'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : MediaToolDirectoryResponse.fromJson(map);
    })();
  }

  /// Invoke one media tool (optional saveToDrive persistence)
  Future<MediaToolInvokeApiResponse?> agentsToolsInvoke(String toolId, MediaToolInvokeBody body) async {
    final payload = body.toJson();
    final response = await _client.post(ApiPaths.appPath('/ai/tools/${serializePathParameter(toolId, const PathParameterSpec('toolId', 'simple', false))}/invoke'), body: payload, contentType: 'application/json');
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : MediaToolInvokeApiResponse.fromJson(map);
    })();
  }

  /// List generated media assets persisted to Drive
  Future<ToolAssetListResponse?> agentsAssetsList() async {
    final response = await _client.get(ApiPaths.appPath('/ai/assets'));
    return (() {
      final map = sdkworkResponseAsMap(response);
      return map == null ? null : ToolAssetListResponse.fromJson(map);
    })();
  }
}

class PathParameterSpec {
  final String name;
  final String style;
  final bool explode;

  const PathParameterSpec(this.name, this.style, this.explode);
}

String serializePathParameter(dynamic value, PathParameterSpec spec) {
  if (value == null) return '';
  final style = spec.style.trim().isEmpty ? 'simple' : spec.style;
  if (value is Iterable) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (value is Map) {
    return serializePathObject(spec.name, value, style, spec.explode);
  }
  return pathPrimitivePrefix(spec.name, style) + Uri.encodeComponent(value.toString());
}

String serializePathArray(String name, Iterable values, String style, bool explode) {
  final serialized = values.where((item) => item != null).map((item) => Uri.encodeComponent(item.toString())).toList();
  if (serialized.isEmpty) return pathPrefix(name, style);
  if (style == 'matrix') {
    if (explode) {
      return serialized.map((item) => ';$name=$item').join();
    }
    return ';$name=${serialized.join(',')}';
  }
  final separator = explode ? '.' : ',';
  return pathPrefix(name, style) + serialized.join(separator);
}

String serializePathObject(String name, Map values, String style, bool explode) {
  final entries = <String>[];
  final exploded = <String>[];
  values.forEach((key, value) {
    if (value == null) return;
    final escapedKey = Uri.encodeComponent(key.toString());
    final escapedValue = Uri.encodeComponent(value.toString());
    if (explode) {
      if (style == 'matrix') {
        exploded.add(';$escapedKey=$escapedValue');
      } else {
        exploded.add('$escapedKey=$escapedValue');
      }
    } else {
      entries.add(escapedKey);
      entries.add(escapedValue);
    }
  });
  if (style == 'matrix') {
    if (explode) return exploded.join();
    return ';$name=${entries.join(',')}';
  }
  if (explode) {
    final separator = style == 'label' ? '.' : ',';
    return pathPrefix(name, style) + exploded.join(separator);
  }
  return pathPrefix(name, style) + entries.join(',');
}

String pathPrefix(String name, String style) {
  if (style == 'label') return '.';
  if (style == 'matrix') return ';$name';
  return '';
}

String pathPrimitivePrefix(String name, String style) {
  return style == 'matrix' ? ';$name=' : pathPrefix(name, style);
}
class QueryParameterSpec {
  final String name;
  final dynamic value;
  final String style;
  final bool explode;
  final bool allowReserved;
  final String? contentType;

  const QueryParameterSpec(
    this.name,
    this.value,
    this.style,
    this.explode,
    this.allowReserved,
    this.contentType,
  );
}

String buildQueryString(List<QueryParameterSpec> parameters) {
  final pairs = <String>[];
  for (final parameter in parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

void appendSerializedParameter(List<String> pairs, QueryParameterSpec parameter) {
  final value = parameter.value;
  if (value == null) return;

  final contentType = parameter.contentType;
  if (contentType != null && contentType.trim().isNotEmpty) {
    pairs.add('${urlEncode(parameter.name)}=${encodeQueryValue(jsonEncode(value), parameter.allowReserved)}');
    return;
  }

  final style = parameter.style.trim().isEmpty ? 'form' : parameter.style;
  if (style == 'deepObject' && value is Map) {
    appendDeepObjectParameter(pairs, parameter.name, value, parameter.allowReserved);
    return;
  }
  if (value is Iterable) {
    appendArrayParameter(pairs, parameter.name, value, style, parameter.explode, parameter.allowReserved);
    return;
  }
  if (value is Map) {
    appendObjectParameter(pairs, parameter.name, value, style, parameter.explode, parameter.allowReserved);
    return;
  }
  pairs.add('${urlEncode(parameter.name)}=${encodeQueryValue(value.toString(), parameter.allowReserved)}');
}

void appendArrayParameter(
  List<String> pairs,
  String name,
  Iterable values,
  String style,
  bool explode,
  bool allowReserved,
) {
  final serialized = values.where((item) => item != null).map((item) => item.toString()).toList();
  if (serialized.isEmpty) return;
  if (style == 'form' && explode) {
    for (final item in serialized) {
      pairs.add('${urlEncode(name)}=${encodeQueryValue(item, allowReserved)}');
    }
    return;
  }
  pairs.add('${urlEncode(name)}=${encodeQueryValue(serialized.join(','), allowReserved)}');
}

void appendObjectParameter(
  List<String> pairs,
  String name,
  Map values,
  String style,
  bool explode,
  bool allowReserved,
) {
  final serialized = <String>[];
  values.forEach((key, value) {
    if (value == null) return;
    if (style == 'form' && explode) {
      pairs.add('${urlEncode(key.toString())}=${encodeQueryValue(value.toString(), allowReserved)}');
      return;
    }
    serialized.add(key.toString());
    serialized.add(value.toString());
  });
  if (serialized.isNotEmpty) {
    pairs.add('${urlEncode(name)}=${encodeQueryValue(serialized.join(','), allowReserved)}');
  }
}

void appendDeepObjectParameter(List<String> pairs, String name, Map values, bool allowReserved) {
  values.forEach((key, value) {
    if (value != null) {
      pairs.add('${urlEncode('$name[$key]')}=${encodeQueryValue(value.toString(), allowReserved)}');
    }
  });
}

String encodeQueryValue(String value, bool allowReserved) {
  var encoded = urlEncode(value);
  if (!allowReserved) return encoded;
  const replacements = <String, String>{
    '%3A': ':',
    '%2F': '/',
    '%3F': '?',
    '%23': '#',
    '%5B': '[',
    '%5D': ']',
    '%40': '@',
    '%21': '!',
    '%24': r'$',
    '%26': '&',
    '%27': "'",
    '%28': '(',
    '%29': ')',
    '%2A': '*',
    '%2B': '+',
    '%2C': ',',
    '%3B': ';',
    '%3D': '=',
  };
  replacements.forEach((escaped, reserved) {
    encoded = encoded.replaceAll(escaped, reserved);
  });
  return encoded;
}

String urlEncode(String value) => Uri.encodeQueryComponent(value);
