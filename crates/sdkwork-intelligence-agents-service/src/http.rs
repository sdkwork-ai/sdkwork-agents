mod context;
mod media_tools;
mod middleware;
pub mod testing;

pub use context::AgentRequestContext;
use context::RequestScope;
pub(crate) use media_tools::{
    app_invoke_media_tool, app_list_media_tools, app_list_tool_assets, backend_list_media_tools,
    backend_update_media_tool_configuration,
};

use crate::agent_turn_input_queue::{
    AgentTurnInputQueueDriveRef, AgentTurnInputQueueEntry, TurnInputQueueFailureRequest,
    TurnInputQueueListQuery, TurnInputQueueReorderEntry,
};
use crate::application::{
    AgentCompositionSlotCreateCommand, AgentCompositionSlotDeleteCommand,
    AgentCompositionSlotGetCommand, AgentCompositionSlotListCommand,
    AgentCompositionSlotUpdateCommand, AgentItemDriveRefInput, AgentsService,
    ArchiveSessionCommand, CancelTurnCommand, ClaimNextTurnInputQueueEntryCommand,
    ClearTurnInputQueueEntriesCommand, CloseSessionCommand, CreateProjectCommand,
    CreateProjectCompositionSlotCommand, CreateSessionCommand, CreateTurnCommand,
    CreateTurnInputQueueEntryCommand, CreateWorkspaceCommand, DeleteProjectCompositionSlotCommand,
    DeleteSessionCommand, EnsureDefaultWorkspaceCommand, FailTurnInputQueueEntryCommand,
    GetInteractionCommand, GetProjectCommand, GetProjectCompositionSlotCommand,
    GetProjectSessionCommand, GetSessionCheckpointCommand, GetSessionCommand,
    GetSessionItemCommand, GetSessionRuntimeBindingCommand, GetSessionUserStateCommand,
    GetTaskCommand, GetTaskRunCommand, GetTurnByIdempotencyCommand, GetTurnCommand,
    GetWorkspaceCommand, ImportProjectCommand, ListAgentAuditEventsCommand,
    ListItemFeedbackCommand, ListMcpMarketplaceCommand, ListProjectCompositionSlotsCommand,
    ListProjectsCommand, ListSessionActivitySummariesCommand, ListSessionCheckpointsCommand,
    ListSessionRuntimeBindingsCommand, ListSessionUserStatesCommand,
    ListTurnInputQueueEntriesCommand, ListTurnsCommand, ListWorkspacesCommand,
    PersistProviderInteractionEventCommand, ProjectMutationCommand, ProviderBindingListCommand,
    RemoveTurnInputQueueEntryCommand, ReorderTurnInputQueueEntriesCommand,
    RetryTurnInputQueueEntryCommand, UpdateItemFeedbackCommand, UpdateProjectCommand,
    UpdateProjectCompositionSlotCommand, UpdateSessionCommand, UpdateSessionUserStateCommand,
    UpdateTurnInputQueueEntryCommand, UpdateWorkspaceCommand, WorkspaceMutationCommand,
};
use crate::domain::{
    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentItemFeedbackRating, AgentItemResourceRole, AgentProviderBindingRecord,
    AgentSessionEntrySurface, AgentSessionKind, AgentSessionRuntimeBindingRecord,
    AgentSessionRuntimeBindingStatus,
};
use crate::dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotUpdateRequestDto, AgentInteractionRecordDto,
    AgentItemFeedbackRecordDto, AgentManagementProfileDto, AgentPreviewResponseRequestDto,
    AgentPromptOptimizationRequestDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentRecordDto, AgentResourceUserStateRecordDto,
    AgentRuntimeExecutionRecordDto, AgentSessionCheckpointRecordDto, AgentSessionItemRecordDto,
    AgentSessionRecordDto, AgentSessionRuntimeBindingRecordDto, AgentTaskRecordDto,
    AgentTaskRunAttemptRecordDto, AgentTaskRunRecordDto, AgentTurnExecutionDto, AgentTurnRecordDto,
    AnswerInteractionRequestDto, ApproveInteractionRequestDto, ArchiveSessionRequestDto,
    CancelTaskRequestDto, CancelTaskRunRequestDto, ChangeSessionCheckpointStatusRequestDto,
    ChangeSessionRuntimeBindingStatusRequestDto, ClaimInteractionRequestDto,
    CloseSessionRequestDto, CreateAgentRequestDto, CreateInteractionRequestDto,
    CreateSessionCheckpointRequestDto, CreateSessionRequestDto,
    CreateSessionRuntimeBindingRequestDto, CreateTaskRequestDto, DeleteAgentRequestDto,
    ExecuteTaskRequestDto, GetAgentRequestDto, InteractionClaimResultDto, ListAgentsRequestDto,
    ListInteractionsRequestDto, ListSessionItemsRequestDto, ListSessionsRequestDto,
    ListTaskRunAttemptsRequestDto, ListTaskRunsRequestDto, ListTasksRequestDto,
    ReconcileTaskRunRequestDto, ReplaceTaskRequestDto, ResolveInteractionRequestDto,
    RestoreAgentRequestDto, RetryTaskRunRequestDto, SessionActivitySummaryDto,
    TaskStateChangeRequestDto, UpdateAgentRequestDto, UpdateAgentStatusRequestDto,
    UpdateSessionRuntimeBindingRequestDto,
};
use crate::list_cursors::{
    decode_audit_event_list_cursor, decode_created_at_cursor, decode_session_list_cursor,
};
use crate::mcp_marketplace::McpServerMarketplaceRecord;
use crate::ports::{
    AgentAuditSink, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    ItemFeedbackListQuery, McpMarketplaceListQuery, PaginationParams,
    ProjectCompositionSlotListQuery, ProjectListQuery, ProviderBindingListQuery,
    ResourceUserStateListQuery, SessionActivitySummaryListQuery, SessionCheckpointListQuery,
    SessionRuntimeBindingListQuery, TurnListQuery, WorkspaceListQuery,
};
use crate::project::{
    AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode, AgentProjectRecord,
    AgentProjectStatus, AgentProjectVisibility,
};
use crate::postgres_model_configuration_store::{
    ProfileScope, ScopedAgentConfigurationStore, ScopedInMemoryAgentConfigurationStore,
};
use crate::response::{
    created_json, finish_api_json, finish_created_api_json, no_content, success_json, ApiProblem,
    ApiResult, PageData, PageInfo, PageMode, ResourceData,
};
use sdkwork_utils_rust::SdkWorkResultCode;
use crate::runtime_facade_bridge::{engine_key_for_provider_identity, shared_agent_engine_host};
use crate::session_activity::{
    decode_session_activity_cursor, SessionActivitySummaryRecord,
    SessionProviderActivityObservation,
};
use crate::session_item_cursor::decode_session_item_cursor;
use crate::task_execution_cursor::{
    decode_task_cursor, decode_task_run_attempt_cursor, decode_task_run_cursor,
};
use crate::task_scheduler::{
    ClaimTaskRunsRequest, FailTaskRunRequest, MaterializeDueTasksRequest, ReconcileTaskRunRequest,
    TaskRunAttemptListQuery, TaskRunClaim, TaskRunLease, TaskRunListQuery, TaskSchedulerRepository,
    TaskTransitionResult,
};
use crate::turn_runtime::{ContractTurnExecutor, TurnExecutionStreamSink, TurnExecutor};
use crate::validation::{
    is_trimmed_blank, parse_expected_version, parse_optional_rfc3339_datetime,
    parse_organization_id, parse_tenant_id, validate_requested_at, validate_standard_id,
    ID_PREFIX_MODEL, ID_PREFIX_PROFILE, ID_PREFIX_REQUEST,
};
use crate::workspace::{AgentWorkspaceRecord, AgentWorkspaceStatus};
use axum::body::{Body, Bytes};
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Extension, Path, Query, State};
use axum::http::header::{HeaderName, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
#[cfg(test)]
use sdkwork_agent_kernel::ProviderManifest;
use sdkwork_agent_kernel::{
    AgentConfigValue, AgentConfigurationProfile, AgentConfigurationUpgradeRequest, AgentManifest,
    AgentModelConfigurationFieldMapping, InMemorySecretProvider, KernelError, KernelErrorKind,
    KernelResult, PolicyDecision, PolicyProvider, PolicyRequest, PolicySubject, ProviderHealth,
    SecretAccessRequest, SecretCreateRequest, SecretProvider, SecretRotateRequest, SecretType,
};
use sdkwork_agents_runtime_facade::AgentEngineCatalog;
use sdkwork_code_kernel::CodeTaskIntent;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use time::OffsetDateTime;
use tokio::sync::{mpsc, Semaphore};
use tokio_stream::StreamExt;

const MAX_PAGE_SIZE: usize = 200;
const DEFAULT_SERVICE_WORKER_LIMIT: usize = 128;
const TURN_STREAM_CHANNEL_CAPACITY: usize = 128;
/// Wall-clock bound for the synchronous turn-cancellation service call. The
/// provider cancel can block when a provider process is hung; the HTTP client
/// must never wait indefinitely for it (the worker keeps running and its
/// result is dropped).
const TURN_CANCEL_SERVICE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub const ENV_TURN_RECONCILIATION_INTERVAL_SECONDS: &str =
    "SDKWORK_AGENTS_TURN_RECONCILIATION_INTERVAL_SECONDS";
pub const ENV_TURN_STALE_AFTER_SECONDS: &str = "SDKWORK_AGENTS_TURN_STALE_AFTER_SECONDS";
pub const ENV_TURN_RECONCILIATION_BATCH_SIZE: &str =
    "SDKWORK_AGENTS_TURN_RECONCILIATION_BATCH_SIZE";

/// Bounds synchronous repository work before it enters Tokio's blocking pool.
/// A bounded rejection is preferable to an unbounded blocking queue, which can
/// retain request payloads and tenant context until memory is exhausted.
static SERVICE_WORKER_LIMIT: LazyLock<Arc<Semaphore>> = LazyLock::new(|| {
    let configured = std::env::var("SDKWORK_AGENTS_SERVICE_WORKER_LIMIT")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or(DEFAULT_SERVICE_WORKER_LIMIT);
    Arc::new(Semaphore::new(configured))
});
const ALLOWED_AUDIT_ACTIONS: &[&str] = &[
    "created",
    "updated",
    "deleted",
    "restored",
    "status_changed",
    "started",
    "completed",
    "failed",
    "cancelled",
    "runtime_executed",
    "provider_binding_changed",
    "composition_slot_created",
    "composition_slot_updated",
    "composition_slot_deleted",
    "session_created",
    "session_closed",
    "session_archived",
    "session_item_created",
    "session_item_failed",
    "interaction_created",
    "interaction_resolved",
    "interaction_rejected",
    "interaction_expired",
    "interaction_cancelled",
];

trait AgentHttpRepository: AgentRepository + TaskSchedulerRepository {}

impl<T> AgentHttpRepository for T where T: AgentRepository + TaskSchedulerRepository {}

pub(crate) struct DynAgentRepository(Box<dyn AgentHttpRepository + Send + Sync>);
pub(crate) struct DynAgentAuditSink(Box<dyn AgentAuditSink + Send + Sync>);
pub(crate) struct DynPolicyProvider(Box<dyn PolicyProvider + Send + Sync>);

impl DynAgentRepository {
    fn new<R>(repository: R) -> Self
    where
        R: AgentRepository + TaskSchedulerRepository + Send + Sync + 'static,
    {
        Self(Box::new(repository))
    }
}

impl DynAgentAuditSink {
    fn new<A>(audit_sink: A) -> Self
    where
        A: AgentAuditSink + Send + Sync + 'static,
    {
        Self(Box::new(audit_sink))
    }
}

impl DynPolicyProvider {
    fn new<P>(policy_provider: P) -> Self
    where
        P: PolicyProvider + Send + Sync + 'static,
    {
        Self(Box::new(policy_provider))
    }
}

impl AgentRepository for DynAgentRepository {
    fn check_readiness(&self) -> KernelResult<()> {
        self.0.check_readiness()
    }

    fn next_id(&self) -> KernelResult<u64> {
        self.0.next_id()
    }

    fn insert(&self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.insert(record)
    }

    fn update(&self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.update(record)
    }

    fn get(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentBusinessRecord>> {
        self.0.get(tenant_id, agent_id)
    }

    fn list(
        &self,
        query: &crate::ports::AgentListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentBusinessRecord>> {
        self.0.list(query)
    }

    fn count_agents(&self, query: &crate::ports::AgentListQuery) -> KernelResult<u64> {
        self.0.count_agents(query)
    }

    fn insert_workspace(&self, record: crate::workspace::AgentWorkspaceRecord) -> KernelResult<()> {
        self.0.insert_workspace(record)
    }

    fn update_workspace(&self, record: crate::workspace::AgentWorkspaceRecord) -> KernelResult<()> {
        self.0.update_workspace(record)
    }

    fn get_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<crate::workspace::AgentWorkspaceRecord>> {
        self.0
            .get_workspace(tenant_id, organization_id, workspace_id)
    }

    fn get_default_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<crate::workspace::AgentWorkspaceRecord>> {
        self.0
            .get_default_workspace(tenant_id, organization_id, owner_user_id)
    }

    fn list_workspaces(
        &self,
        query: &crate::ports::WorkspaceListQuery,
    ) -> KernelResult<Vec<crate::workspace::AgentWorkspaceRecord>> {
        self.0.list_workspaces(query)
    }

    fn count_workspaces(&self, query: &crate::ports::WorkspaceListQuery) -> KernelResult<u64> {
        self.0.count_workspaces(query)
    }

    fn insert_project(&self, record: crate::project::AgentProjectRecord) -> KernelResult<()> {
        self.0.insert_project(record)
    }

    fn update_project(&self, record: crate::project::AgentProjectRecord) -> KernelResult<()> {
        self.0.update_project(record)
    }

    fn get_project(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<crate::project::AgentProjectRecord>> {
        self.0.get_project(tenant_id, organization_id, project_id)
    }

    fn get_project_by_workspace_name(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
        name: &str,
    ) -> KernelResult<Option<crate::project::AgentProjectRecord>> {
        self.0
            .get_project_by_workspace_name(tenant_id, organization_id, workspace_id, name)
    }

    fn get_project_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<crate::project::AgentProjectRecord>> {
        self.0.get_project_by_import_source(
            tenant_id,
            organization_id,
            owner_user_id,
            source_kind,
            source_ref,
        )
    }

    fn list_projects(
        &self,
        query: &crate::ports::ProjectListQuery,
    ) -> KernelResult<Vec<crate::project::AgentProjectRecord>> {
        self.0.list_projects(query)
    }

    fn count_projects(&self, query: &crate::ports::ProjectListQuery) -> KernelResult<u64> {
        self.0.count_projects(query)
    }

    fn insert_project_composition_slot(
        &self,
        record: crate::project::AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        self.0.insert_project_composition_slot(record)
    }

    fn update_project_composition_slot(
        &self,
        record: crate::project::AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        self.0.update_project_composition_slot(record)
    }

    fn get_project_composition_slot(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<crate::project::AgentProjectCompositionSlotRecord>> {
        self.0
            .get_project_composition_slot(tenant_id, organization_id, project_id, slot_id)
    }

    fn list_project_composition_slots(
        &self,
        query: &crate::ports::ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<crate::project::AgentProjectCompositionSlotRecord>> {
        self.0.list_project_composition_slots(query)
    }

    fn count_project_composition_slots(
        &self,
        query: &crate::ports::ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64> {
        self.0.count_project_composition_slots(query)
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.0.insert_provider_binding(record)
    }

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.0.update_provider_binding(record)
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        self.0.get_provider_binding(tenant_id, agent_id, binding_id)
    }

    fn get_active_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        self.0.get_active_provider_binding(tenant_id, agent_id)
    }

    fn list_provider_bindings(
        &self,
        query: &crate::ports::ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        self.0.list_provider_bindings(query)
    }

    fn count_provider_bindings(
        &self,
        query: &crate::ports::ProviderBindingListQuery,
    ) -> KernelResult<u64> {
        self.0.count_provider_bindings(query)
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.0.insert_composition_slot(record)
    }

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.0.update_composition_slot(record)
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRecord>> {
        self.0.get_composition_slot(tenant_id, agent_id, slot_id)
    }

    fn list_composition_slots(
        &self,
        query: &crate::ports::CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.0.list_composition_slots(query)
    }

    fn count_composition_slots(
        &self,
        query: &crate::ports::CompositionSlotListQuery,
    ) -> KernelResult<u64> {
        self.0.count_composition_slots(query)
    }

    fn list_mcp_marketplace_slots(
        &self,
        query: &crate::ports::McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.0.list_mcp_marketplace_slots(query)
    }

    fn count_mcp_marketplace_slots(
        &self,
        query: &crate::ports::McpMarketplaceListQuery,
    ) -> KernelResult<u64> {
        self.0.count_mcp_marketplace_slots(query)
    }

    fn insert_session(&self, record: crate::domain::AgentSessionRecord) -> KernelResult<()> {
        self.0.insert_session(record)
    }

    fn update_session(&self, record: crate::domain::AgentSessionRecord) -> KernelResult<()> {
        self.0.update_session(record)
    }

    fn delete_session_and_purge_queue(
        &self,
        deleted_session: crate::domain::AgentSessionRecord,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<()> {
        self.0.delete_session_and_purge_queue(
            deleted_session,
            tenant_id,
            organization_id,
            session_id,
            owner_user_id,
        )
    }

    fn get_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRecord>> {
        self.0.get_session(tenant_id, organization_id, session_id)
    }

    fn get_session_by_creation_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRecord>> {
        self.0.get_session_by_creation_idempotency(
            tenant_id,
            organization_id,
            owner_user_id,
            idempotency_key,
        )
    }

    fn list_sessions(
        &self,
        query: &crate::ports::SessionListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionRecord>> {
        self.0.list_sessions(query)
    }

    fn list_session_activity_summaries(
        &self,
        query: &crate::ports::SessionActivitySummaryListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<crate::SessionActivitySummaryRecord>> {
        self.0.list_session_activity_summaries(query)
    }

    fn count_sessions(&self, query: &crate::ports::SessionListQuery) -> KernelResult<u64> {
        self.0.count_sessions(query)
    }

    fn insert_session_runtime_binding(
        &self,
        record: crate::domain::AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        self.0.insert_session_runtime_binding(record)
    }

    fn update_session_runtime_binding(
        &self,
        record: crate::domain::AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        self.0.update_session_runtime_binding(record)
    }

    fn get_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRuntimeBindingRecord>> {
        self.0.get_session_runtime_binding(
            tenant_id,
            organization_id,
            session_id,
            runtime_binding_id,
        )
    }

    fn get_session_runtime_binding_by_provider_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        provider_binding_id: &str,
        provider_session_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRuntimeBindingRecord>> {
        self.0.get_session_runtime_binding_by_provider_session(
            tenant_id,
            organization_id,
            owner_user_id,
            provider_binding_id,
            provider_session_id,
        )
    }

    fn get_current_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRuntimeBindingRecord>> {
        self.0
            .get_current_session_runtime_binding(tenant_id, organization_id, session_id)
    }

    fn list_session_runtime_bindings(
        &self,
        query: &crate::ports::SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionRuntimeBindingRecord>> {
        self.0.list_session_runtime_bindings(query)
    }

    fn count_session_runtime_bindings(
        &self,
        query: &crate::ports::SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64> {
        self.0.count_session_runtime_bindings(query)
    }

    fn activate_session_runtime_binding_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<crate::domain::AgentSessionRuntimeBindingRecord> {
        self.0.activate_session_runtime_binding_atomic(
            tenant_id,
            organization_id,
            session_id,
            runtime_binding_id,
            expected_version,
            updated_at,
        )
    }

    fn insert_session_checkpoint(
        &self,
        record: crate::domain::AgentSessionCheckpointRecord,
    ) -> KernelResult<()> {
        self.0.insert_session_checkpoint(record)
    }

    fn update_session_checkpoint(
        &self,
        record: crate::domain::AgentSessionCheckpointRecord,
    ) -> KernelResult<()> {
        self.0.update_session_checkpoint(record)
    }

    fn get_session_checkpoint(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionCheckpointRecord>> {
        self.0
            .get_session_checkpoint(tenant_id, organization_id, session_id, checkpoint_id)
    }

    fn list_session_checkpoints(
        &self,
        query: &crate::ports::SessionCheckpointListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionCheckpointRecord>> {
        self.0.list_session_checkpoints(query)
    }

    fn count_session_checkpoints(
        &self,
        query: &crate::ports::SessionCheckpointListQuery,
    ) -> KernelResult<u64> {
        self.0.count_session_checkpoints(query)
    }

    fn upsert_resource_user_state(
        &self,
        record: crate::domain::AgentResourceUserStateRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<crate::domain::AgentResourceUserStateRecord> {
        self.0.upsert_resource_user_state(record, expected_version)
    }

    fn get_resource_user_state(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: crate::domain::AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentResourceUserStateRecord>> {
        self.0.get_resource_user_state(
            tenant_id,
            organization_id,
            user_id,
            resource_type,
            resource_id,
        )
    }

    fn list_resource_user_states(
        &self,
        query: &crate::ports::ResourceUserStateListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentResourceUserStateRecord>> {
        self.0.list_resource_user_states(query)
    }

    fn count_resource_user_states(
        &self,
        query: &crate::ports::ResourceUserStateListQuery,
    ) -> KernelResult<u64> {
        self.0.count_resource_user_states(query)
    }

    fn upsert_tool_configuration(
        &self,
        record: crate::domain::AgentToolConfigurationRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<crate::domain::AgentToolConfigurationRecord> {
        self.0.upsert_tool_configuration(record, expected_version)
    }

    fn get_tool_configuration(
        &self,
        tenant_id: u64,
        organization_id: u64,
        tool_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentToolConfigurationRecord>> {
        self.0
            .get_tool_configuration(tenant_id, organization_id, tool_id)
    }

    fn list_tool_configurations(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> KernelResult<Vec<crate::domain::AgentToolConfigurationRecord>> {
        self.0.list_tool_configurations(tenant_id, organization_id)
    }

    fn insert_tool_asset(&self, record: crate::domain::AgentToolAssetRecord) -> KernelResult<()> {
        self.0.insert_tool_asset(record)
    }

    fn list_tool_assets(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        limit: u64,
    ) -> KernelResult<Vec<crate::domain::AgentToolAssetRecord>> {
        self.0
            .list_tool_assets(tenant_id, organization_id, user_id, limit)
    }

    fn append_session_item(
        &self,
        record: crate::domain::AgentSessionItemRecord,
    ) -> KernelResult<(
        crate::domain::AgentSessionRecord,
        crate::domain::AgentSessionItemRecord,
    )> {
        self.0.append_session_item(record)
    }

    fn update_session_item(
        &self,
        record: crate::domain::AgentSessionItemRecord,
    ) -> KernelResult<()> {
        self.0.update_session_item(record)
    }

    fn get_session_item(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionItemRecord>> {
        self.0
            .get_session_item(tenant_id, organization_id, session_id, item_id)
    }

    fn list_session_items(
        &self,
        query: &crate::ports::SessionItemListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionItemRecord>> {
        self.0.list_session_items(query)
    }

    fn count_session_items(&self, query: &crate::ports::SessionItemListQuery) -> KernelResult<u64> {
        self.0.count_session_items(query)
    }

    fn list_session_items_by_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        turn_id: &str,
        limit: usize,
    ) -> KernelResult<Vec<crate::domain::AgentSessionItemRecord>> {
        self.0
            .list_session_items_by_turn(tenant_id, organization_id, session_id, turn_id, limit)
    }

    fn upsert_item_feedback(
        &self,
        record: crate::domain::AgentItemFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<crate::domain::AgentItemFeedbackRecord> {
        self.0.upsert_item_feedback(record, expected_version)
    }

    fn get_item_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<crate::domain::AgentItemFeedbackRecord>> {
        self.0.get_item_feedback(
            tenant_id,
            organization_id,
            item_id,
            user_id,
            include_deleted,
        )
    }

    fn list_item_feedback(
        &self,
        query: &crate::ports::ItemFeedbackListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentItemFeedbackRecord>> {
        self.0.list_item_feedback(query)
    }

    fn count_item_feedback(
        &self,
        query: &crate::ports::ItemFeedbackListQuery,
    ) -> KernelResult<u64> {
        self.0.count_item_feedback(query)
    }

    fn get_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<crate::agent_turn::AgentTurnRecord>> {
        self.0
            .get_turn_by_idempotency(tenant_id, organization_id, owner_user_id, idempotency_key)
    }

    fn get_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<crate::agent_turn::AgentTurnRecord>> {
        self.0.get_turn(tenant_id, organization_id, turn_id)
    }

    fn list_turns(
        &self,
        query: &crate::ports::TurnListQuery,
    ) -> KernelResult<Vec<crate::agent_turn::AgentTurnRecord>> {
        self.0.list_turns(query)
    }

    fn count_turns(&self, query: &crate::ports::TurnListQuery) -> KernelResult<u64> {
        self.0.count_turns(query)
    }

    fn get_turn_input_queue_entry(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        queue_entry_id: &str,
    ) -> KernelResult<Option<crate::agent_turn_input_queue::AgentTurnInputQueueEntry>> {
        self.0.get_turn_input_queue_entry(
            tenant_id,
            organization_id,
            session_id,
            owner_user_id,
            queue_entry_id,
        )
    }

    fn list_turn_input_queue_entries(
        &self,
        query: &crate::agent_turn_input_queue::TurnInputQueueListQuery,
        owner_user_id: u64,
    ) -> KernelResult<Vec<crate::agent_turn_input_queue::AgentTurnInputQueueEntry>> {
        self.0.list_turn_input_queue_entries(query, owner_user_id)
    }

    fn count_turn_input_queue_entries(
        &self,
        query: &crate::agent_turn_input_queue::TurnInputQueueListQuery,
        owner_user_id: u64,
    ) -> KernelResult<u64> {
        self.0.count_turn_input_queue_entries(query, owner_user_id)
    }

    fn insert_turn_input_queue_entry(
        &self,
        entry: crate::agent_turn_input_queue::AgentTurnInputQueueEntry,
    ) -> KernelResult<crate::agent_turn_input_queue::AgentTurnInputQueueEntry> {
        self.0.insert_turn_input_queue_entry(entry)
    }

    fn update_turn_input_queue_entry(
        &self,
        entry: crate::agent_turn_input_queue::AgentTurnInputQueueEntry,
        expected_version: u64,
    ) -> KernelResult<crate::agent_turn_input_queue::AgentTurnInputQueueEntry> {
        self.0
            .update_turn_input_queue_entry(entry, expected_version)
    }

    fn remove_turn_input_queue_entry(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        queue_entry_id: &str,
        expected_version: u64,
    ) -> KernelResult<crate::agent_turn_input_queue::AgentTurnInputQueueEntry> {
        self.0.remove_turn_input_queue_entry(
            tenant_id,
            organization_id,
            session_id,
            owner_user_id,
            queue_entry_id,
            expected_version,
        )
    }

    fn clear_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<u64> {
        self.0
            .clear_turn_input_queue_entries(tenant_id, organization_id, session_id, owner_user_id)
    }

    fn purge_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<u64> {
        self.0
            .purge_turn_input_queue_entries(tenant_id, organization_id, session_id, owner_user_id)
    }

    fn reorder_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        entries: &[crate::agent_turn_input_queue::TurnInputQueueReorderEntry],
        requested_at: &str,
    ) -> KernelResult<Vec<crate::agent_turn_input_queue::AgentTurnInputQueueEntry>> {
        self.0.reorder_turn_input_queue_entries(
            tenant_id,
            organization_id,
            session_id,
            owner_user_id,
            entries,
            requested_at,
        )
    }

    fn claim_next_turn_input_queue_entry(
        &self,
        request: &crate::agent_turn_input_queue::TurnInputQueueClaimRequest,
    ) -> KernelResult<crate::agent_turn_input_queue::TurnInputQueueClaimOutcome> {
        self.0.claim_next_turn_input_queue_entry(request)
    }

    fn fail_turn_input_queue_entry(
        &self,
        request: &TurnInputQueueFailureRequest,
    ) -> KernelResult<crate::agent_turn_input_queue::AgentTurnInputQueueEntry> {
        self.0.fail_turn_input_queue_entry(request)
    }

    fn list_reconcilable_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<crate::agent_turn::AgentTurnRecord>> {
        self.0.list_reconcilable_turns(stale_before, limit)
    }

    fn append_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
        content: &str,
        updated_at: &str,
    ) -> KernelResult<()> {
        self.0.append_turn_streaming_content(
            tenant_id,
            organization_id,
            turn_id,
            content,
            updated_at,
        )
    }

    fn clear_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<()> {
        self.0
            .clear_turn_streaming_content(tenant_id, organization_id, turn_id)
    }

    fn insert_turn_request(
        &self,
        turn: crate::agent_turn::AgentTurnRecord,
        request_item: crate::domain::AgentSessionItemRecord,
        drive_refs: Vec<crate::domain::AgentItemDriveRefRecord>,
    ) -> KernelResult<crate::ports::TurnRequestWriteOutcome> {
        self.0.insert_turn_request(turn, request_item, drive_refs)
    }

    fn update_turn_state(
        &self,
        turn: crate::agent_turn::AgentTurnRecord,
        expected_version: u64,
    ) -> KernelResult<crate::agent_turn::AgentTurnRecord> {
        self.0.update_turn_state(turn, expected_version)
    }

    fn complete_turn(
        &self,
        turn: crate::agent_turn::AgentTurnRecord,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        completed_items: Vec<crate::domain::AgentSessionItemRecord>,
    ) -> KernelResult<(
        crate::domain::AgentSessionRecord,
        Vec<crate::domain::AgentSessionItemRecord>,
    )> {
        self.0.complete_turn(
            turn,
            expected_turn_version,
            expected_fencing_token,
            expected_lease_token,
            completed_items,
        )
    }

    fn list_item_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<crate::domain::AgentItemDriveRefRecord>> {
        self.0
            .list_item_drive_refs(tenant_id, organization_id, item_id)
    }

    fn list_item_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<crate::domain::AgentItemDriveRefRecord>> {
        self.0
            .list_item_drive_refs_batch(tenant_id, organization_id, item_ids)
    }

    fn insert_task(&self, record: crate::domain::AgentTaskRecord) -> KernelResult<()> {
        self.0.insert_task(record)
    }

    fn update_task(&self, record: crate::domain::AgentTaskRecord) -> KernelResult<()> {
        self.0.update_task(record)
    }

    fn get_task(
        &self,
        tenant_id: u64,
        organization_id: u64,
        task_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentTaskRecord>> {
        self.0.get_task(tenant_id, organization_id, task_id)
    }

    fn list_tasks(
        &self,
        query: &crate::ports::TaskListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentTaskRecord>> {
        self.0.list_tasks(query)
    }

    fn insert_interaction(
        &self,
        record: crate::domain::AgentInteractionRecord,
    ) -> KernelResult<()> {
        self.0.insert_interaction(record)
    }

    fn update_interaction(
        &self,
        record: crate::domain::AgentInteractionRecord,
    ) -> KernelResult<()> {
        self.0.update_interaction(record)
    }

    fn get_interaction(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentInteractionRecord>> {
        self.0
            .get_interaction(tenant_id, organization_id, session_id, interaction_id)
    }

    fn list_interactions(
        &self,
        query: &crate::ports::InteractionListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentInteractionRecord>> {
        self.0.list_interactions(query)
    }

    fn count_interactions(&self, query: &crate::ports::InteractionListQuery) -> KernelResult<u64> {
        self.0.count_interactions(query)
    }

    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRecord> {
        self.0
            .activate_provider_binding_atomic(tenant_id, agent_id, binding_id, updated_at)
    }
}

impl TaskSchedulerRepository for DynAgentRepository {
    fn transition_task(
        &self,
        task: crate::AgentTaskRecord,
        cancellation_reason: &str,
    ) -> KernelResult<TaskTransitionResult> {
        self.0.transition_task(task, cancellation_reason)
    }

    fn create_manual_task_run(
        &self,
        task: &crate::AgentTaskRecord,
        idempotency_key: &str,
        requested_at: &str,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0
            .create_manual_task_run(task, idempotency_key, requested_at)
    }

    fn create_business_retry_task_run(
        &self,
        task: &crate::AgentTaskRecord,
        retry_of: &crate::AgentTaskRunRecord,
        idempotency_key: &str,
        requested_at: &str,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0
            .create_business_retry_task_run(task, retry_of, idempotency_key, requested_at)
    }

    fn materialize_due_tasks(
        &self,
        request: &MaterializeDueTasksRequest,
    ) -> KernelResult<Vec<crate::AgentTaskRunRecord>> {
        self.0.materialize_due_tasks(request)
    }

    fn claim_task_runs(&self, request: &ClaimTaskRunsRequest) -> KernelResult<Vec<TaskRunClaim>> {
        self.0.claim_task_runs(request)
    }

    fn mark_task_run_running(
        &self,
        lease: &TaskRunLease,
        started_at: &str,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0.mark_task_run_running(lease, started_at)
    }

    fn heartbeat_task_run(
        &self,
        lease: &TaskRunLease,
        heartbeat_at: &str,
        lease_seconds: u32,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0
            .heartbeat_task_run(lease, heartbeat_at, lease_seconds)
    }

    fn complete_task_run(
        &self,
        lease: &TaskRunLease,
        turn_id: &str,
        completed_at: &str,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0.complete_task_run(lease, turn_id, completed_at)
    }

    fn fail_task_run(
        &self,
        request: &FailTaskRunRequest,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0.fail_task_run(request)
    }

    fn recover_expired_task_run_leases(&self, now: &str, limit: usize) -> KernelResult<u64> {
        self.0.recover_expired_task_run_leases(now, limit)
    }

    fn recover_timed_out_task_runs(&self, now: &str, limit: usize) -> KernelResult<u64> {
        self.0.recover_timed_out_task_runs(now, limit)
    }

    fn request_task_run_cancellation(
        &self,
        tenant_id: u64,
        organization_id: u64,
        run_id: &str,
        expected_version: Option<u64>,
        requested_at: &str,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0.request_task_run_cancellation(
            tenant_id,
            organization_id,
            run_id,
            expected_version,
            requested_at,
        )
    }

    fn reconcile_task_run(
        &self,
        request: &ReconcileTaskRunRequest,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.0.reconcile_task_run(request)
    }

    fn list_reconciling_task_runs(
        &self,
        updated_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<crate::AgentTaskRunRecord>> {
        self.0.list_reconciling_task_runs(updated_before, limit)
    }

    fn list_task_runs(
        &self,
        query: &TaskRunListQuery,
    ) -> KernelResult<Vec<crate::AgentTaskRunRecord>> {
        self.0.list_task_runs(query)
    }

    fn list_task_run_attempts(
        &self,
        query: &TaskRunAttemptListQuery,
    ) -> KernelResult<Vec<crate::AgentTaskRunAttemptRecord>> {
        self.0.list_task_run_attempts(query)
    }

    fn get_task_run(
        &self,
        tenant_id: u64,
        organization_id: u64,
        run_id: &str,
    ) -> KernelResult<Option<crate::AgentTaskRunRecord>> {
        self.0.get_task_run(tenant_id, organization_id, run_id)
    }
}

impl AgentAuditSink for DynAgentAuditSink {
    fn record(&self, event: sdkwork_agent_kernel::KernelEvent) -> KernelResult<()> {
        self.0.record(event)
    }

    fn list_events(
        &self,
        query: &crate::ports::AuditEventListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<sdkwork_agent_kernel::KernelEvent>> {
        self.0.list_events(query)
    }
}

impl PolicyProvider for DynPolicyProvider {
    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        self.0.evaluate(request)
    }

    fn health(&self) -> ProviderHealth {
        self.0.health()
    }
}

pub(crate) type HttpService =
    AgentsService<DynAgentRepository, DynAgentAuditSink, DynPolicyProvider>;

pub(crate) struct AgentModelConfigurationRuntime {
    secrets: Mutex<Box<dyn SecretProvider>>,
    configurations: Mutex<Box<dyn ScopedAgentConfigurationStore>>,
}

impl AgentModelConfigurationRuntime {
    fn new() -> Self {
        Self::with_providers(
            Box::new(InMemorySecretProvider::new()),
            Box::new(ScopedInMemoryAgentConfigurationStore::new()),
        )
    }

    pub(crate) fn with_providers(
        secret_provider: Box<dyn SecretProvider>,
        configuration_store: Box<dyn ScopedAgentConfigurationStore>,
    ) -> Self {
        Self {
            secrets: Mutex::new(secret_provider),
            configurations: Mutex::new(configuration_store),
        }
    }

    /// Resolves a stored model-configuration credential by secret reference.
    ///
    /// Backs the kernel host secret surface so agent engines (e.g. the Rig
    /// simple agent) can resolve their configured API key at inference time
    /// through `HostProvider::resolve_secret`.
    pub(crate) fn resolve_secret_value(
        &self,
        secret_ref_id: &str,
        requester: &str,
    ) -> KernelResult<sdkwork_agent_kernel::ProviderSecretValue> {
        let guard = self.secrets.lock().map_err(|_| {
            KernelError::provider_error(
                "secret_store_unavailable",
                "model configuration secret store is unavailable",
            )
        })?;
        let result = guard
            .access_secret(SecretAccessRequest::new(secret_ref_id, requester))
            .map_err(|error| {
                KernelError::provider_error("secret_resolution_failed", error.to_string())
            })?;
        let value = result.value.ok_or_else(|| {
            KernelError::provider_error(
                "secret_not_configured",
                format!("model credential {secret_ref_id} is not configured"),
            )
        })?;
        Ok(sdkwork_agent_kernel::ProviderSecretValue::new(
            secret_ref_id,
            value,
        ))
    }
}

#[derive(Clone)]
pub struct AgentHttpState {
    pub(crate) service: Arc<HttpService>,
    model_configuration_runtime: Arc<AgentModelConfigurationRuntime>,
    provider_session_cwd_resolver:
        Option<Arc<dyn sdkwork_agents_runtime_facade::ProviderSessionProjectCwdResolver>>,
    media_tool_invocation: Option<Arc<crate::tool_invocation::MediaToolInvocationService>>,
}

#[derive(Clone)]
pub struct AgentTaskWorkerHandle {
    service: Arc<HttpService>,
}

impl AgentHttpState {
    pub fn new<R, A, P>(repository: R, audit_sink: A, policy_provider: P) -> Self
    where
        R: AgentRepository + TaskSchedulerRepository + Send + Sync + 'static,
        A: AgentAuditSink + Send + Sync + 'static,
        P: PolicyProvider + Send + Sync + 'static,
    {
        Self::with_turn_executor(
            repository,
            audit_sink,
            policy_provider,
            Arc::new(ContractTurnExecutor),
        )
    }

    pub fn with_turn_executor<R, A, P>(
        repository: R,
        audit_sink: A,
        policy_provider: P,
        turn_executor: Arc<dyn TurnExecutor>,
    ) -> Self
    where
        R: AgentRepository + TaskSchedulerRepository + Send + Sync + 'static,
        A: AgentAuditSink + Send + Sync + 'static,
        P: PolicyProvider + Send + Sync + 'static,
    {
        let service = AgentsService::new(
            DynAgentRepository::new(repository),
            DynAgentAuditSink::new(audit_sink),
            DynPolicyProvider::new(policy_provider),
        )
        .with_turn_executor(turn_executor);
        Self {
            service: Arc::new(service),
            model_configuration_runtime: Arc::new(AgentModelConfigurationRuntime::new()),
            provider_session_cwd_resolver: None,
            media_tool_invocation: None,
        }
    }

    pub fn with_provider_session_cwd_resolver(
        mut self,
        resolver: Arc<dyn sdkwork_agents_runtime_facade::ProviderSessionProjectCwdResolver>,
    ) -> Self {
        self.provider_session_cwd_resolver = Some(resolver);
        self
    }

    pub fn with_model_configuration_providers(
        mut self,
        secret_provider: Box<dyn SecretProvider>,
        configuration_store: Box<dyn ScopedAgentConfigurationStore>,
    ) -> Self {
        self.model_configuration_runtime = Arc::new(
            AgentModelConfigurationRuntime::with_providers(secret_provider, configuration_store),
        );
        self
    }

    /// Attaches the media tool invocation pipeline (registry + tenant
    /// configuration + optional drive persistence).
    pub fn with_media_tool_invocation(
        mut self,
        invocation: crate::tool_invocation::MediaToolInvocationService,
    ) -> Self {
        self.media_tool_invocation = Some(Arc::new(invocation));
        self
    }

    pub fn session_facade(&self) -> Arc<dyn sdkwork_agents_runtime_facade::AgentsSessionFacade> {
        Arc::new(HttpAgentsSessionFacade::new(self.service.clone()))
    }

    pub fn task_worker_handle(&self) -> AgentTaskWorkerHandle {
        AgentTaskWorkerHandle {
            service: self.service.clone(),
        }
    }

    /// Verify the repository dependency used by the same HTTP service state.
    pub fn check_readiness(&self) -> KernelResult<()> {
        self.service.check_readiness()
    }

    pub fn spawn_turn_reconciliation_worker(&self) -> Option<tokio::task::JoinHandle<()>> {
        let interval_seconds = env_usize(ENV_TURN_RECONCILIATION_INTERVAL_SECONDS, 30, 0, 3600);
        if interval_seconds == 0 {
            return None;
        }
        let stale_after_seconds = env_usize(ENV_TURN_STALE_AFTER_SECONDS, 300, 30, 86_400);
        let batch_size = env_usize(ENV_TURN_RECONCILIATION_BATCH_SIZE, 100, 1, 200);
        let service = self.service.clone();
        Some(tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(interval_seconds as u64));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticker.tick().await;
                let now = OffsetDateTime::now_utc();
                let stale_before = now - time::Duration::seconds(stale_after_seconds as i64);
                let occurred_at = format_utc_seconds(now);
                let stale_before = format_utc_seconds(stale_before);
                let service = service.clone();
                let result = tokio::task::spawn_blocking(move || {
                    service.reconcile_stale_turns(&stale_before, &occurred_at, batch_size)
                })
                .await;
                match result {
                    Ok(Ok(summary))
                        if !summary.failed.is_empty() || summary.skipped_conflicts > 0 =>
                    {
                        tracing::info!(
                            target: "sdkwork.agents.turn.reconciliation",
                            examined = summary.examined,
                            failed = summary.failed.len(),
                            skipped_conflicts = summary.skipped_conflicts,
                            "turn reconciliation completed"
                        );
                    }
                    Ok(Ok(_)) => {}
                    Ok(Err(error)) => tracing::error!(
                        target: "sdkwork.agents.turn.reconciliation",
                        error = %error,
                        "turn reconciliation failed"
                    ),
                    Err(error) => tracing::error!(
                        target: "sdkwork.agents.turn.reconciliation",
                        error = %error,
                        "turn reconciliation worker join failed"
                    ),
                }
            }
        }))
    }
}

impl AgentTaskWorkerHandle {
    pub fn check_readiness(&self) -> KernelResult<()> {
        self.service.check_readiness()
    }

    pub async fn materialize_due_tasks(
        &self,
        request: MaterializeDueTasksRequest,
    ) -> KernelResult<Vec<crate::AgentTaskRunRecord>> {
        self.run(move |service| service.materialize_scheduled_task_runs(&request))
            .await
    }

    pub async fn claim_task_runs(
        &self,
        request: ClaimTaskRunsRequest,
    ) -> KernelResult<Vec<TaskRunClaim>> {
        self.run(move |service| service.claim_scheduled_task_runs(&request))
            .await
    }

    pub async fn heartbeat_task_run(
        &self,
        lease: TaskRunLease,
        heartbeat_at: String,
        lease_seconds: u32,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.run(move |service| {
            service.heartbeat_scheduled_task_run(&lease, &heartbeat_at, lease_seconds)
        })
        .await
    }

    pub async fn recover_expired_task_run_leases(
        &self,
        now: String,
        limit: usize,
    ) -> KernelResult<u64> {
        self.run(move |service| service.recover_expired_scheduled_task_run_leases(&now, limit))
            .await
    }

    pub async fn recover_timed_out_task_runs(
        &self,
        now: String,
        limit: usize,
    ) -> KernelResult<u64> {
        self.run(move |service| service.recover_timed_out_scheduled_task_runs(&now, limit))
            .await
    }

    pub async fn scheduler_metrics_snapshot(
        &self,
        now: String,
    ) -> KernelResult<crate::TaskSchedulerMetricsSnapshot> {
        self.run(move |service| service.scheduled_task_metrics_snapshot(&now))
            .await
    }

    pub async fn reconcile_task_runs(
        &self,
        updated_before: String,
        occurred_at: String,
        limit: usize,
    ) -> KernelResult<crate::TaskRunReconciliationResult> {
        self.run(move |service| {
            service.reconcile_scheduled_task_runs(&updated_before, &occurred_at, limit)
        })
        .await
    }

    pub async fn execute_task_run_claim(
        &self,
        claim: TaskRunClaim,
        requested_by: PolicySubject,
        requested_at: String,
    ) -> KernelResult<crate::AgentTaskRunRecord> {
        self.run(move |service| {
            service.execute_scheduled_task_run_claim(&claim, requested_by, requested_at)
        })
        .await
    }

    async fn run<T>(
        &self,
        action: impl FnOnce(&HttpService) -> KernelResult<T> + Send + 'static,
    ) -> KernelResult<T>
    where
        T: Send + 'static,
    {
        let service = self.service.clone();
        tokio::task::spawn_blocking(move || action(service.as_ref()))
            .await
            .map_err(|error| KernelError::Internal {
                message: format!("task worker operation join failed: {error}"),
            })?
    }
}

pub(crate) struct HttpAgentsSessionFacade {
    pub(crate) service: Arc<HttpService>,
    provider_session_history_reconciliation: bool,
}

struct EnsureRuntimeBindingRequest<'a> {
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    agent_id: &'a str,
    session_id: &'a str,
    descriptor: Option<&'a sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor>,
    subject: sdkwork_agent_kernel::PolicySubject,
    requested_at: &'a str,
}

impl HttpAgentsSessionFacade {
    pub(crate) fn new(service: Arc<HttpService>) -> Self {
        Self {
            service,
            provider_session_history_reconciliation: false,
        }
    }

    pub(crate) fn for_provider_session_history_reconciliation(service: Arc<HttpService>) -> Self {
        Self {
            service,
            provider_session_history_reconciliation: true,
        }
    }

    /// Ensures the RIG simple-agent runtime binding for a session that the
    /// client did not explicitly bind: creates the agent's RIG provider
    /// binding on demand, then binds the session to the RIG engine
    /// (`binding.rig` / `provider.rig-rust`, model `rig.default-chat`).
    fn ensure_default_rig_runtime_binding(
        &self,
        request: EnsureRuntimeBindingRequest<'_>,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<()> {
        const RIG_BINDING_ID: &str = "binding.rig";
        const RIG_PROVIDER_ID: &str = "provider.rig-rust";
        const RIG_MODEL_ID: &str = "rig.default-chat";

        let binding_missing = match self.service.get_provider_binding(
            request.tenant_id,
            request.agent_id,
            RIG_BINDING_ID,
            request.subject.clone(),
        ) {
            Ok(_) => false,
            Err(error)
                if error.detail_value("sdkwork.not_found") == Some("true")
                    && error.message() == "provider binding not found" =>
            {
                true
            }
            Err(error) => {
                return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    error.to_string(),
                ));
            }
        };
        if binding_missing {
            self.service
                .add_provider_binding(crate::application::AgentProviderBindingCommand {
                    tenant_id: request.tenant_id,
                    agent_id: request.agent_id.to_string(),
                    binding_id: RIG_BINDING_ID.to_string(),
                    provider_id: RIG_PROVIDER_ID.to_string(),
                    implementation_kind: crate::domain::AgentImplementationKind::TypedLocalProvider,
                    configuration_profile_id: "profile.rig.local".to_string(),
                    capabilities: vec!["model.chat".to_string()],
                    make_default: true,
                    requested_by: request.subject.clone(),
                    requested_at: request.requested_at.to_string(),
                })
                .map_err(|error| {
                    sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(error.to_string())
                })?;
        }
        let default_descriptor = sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor {
            runtime_binding_id: format!("runtime_binding.rig.{}", request.session_id),
            runtime_location_id: None,
            host_mode: "local".to_string(),
            transport_kind: "rig".to_string(),
            provider_binding_id: RIG_BINDING_ID.to_string(),
            model_id: RIG_MODEL_ID.to_string(),
            provider_id: RIG_PROVIDER_ID.to_string(),
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            provider_directory: None,
        };
        self.ensure_runtime_binding(EnsureRuntimeBindingRequest {
            tenant_id: request.tenant_id,
            organization_id: request.organization_id,
            owner_user_id: request.owner_user_id,
            agent_id: request.agent_id,
            session_id: request.session_id,
            descriptor: Some(&default_descriptor),
            subject: request.subject,
            requested_at: request.requested_at,
        })
    }

    fn ensure_runtime_binding(
        &self,
        request: EnsureRuntimeBindingRequest<'_>,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<()> {
        let EnsureRuntimeBindingRequest {
            tenant_id,
            organization_id,
            owner_user_id,
            agent_id,
            session_id,
            descriptor,
            subject,
            requested_at,
        } = request;
        let Some(descriptor) = descriptor else {
            // Default to the RIG simple-agent engine: the default chat agent
            // binds the kernel `ModelProvider` SPI (`binding.rig` /
            // `provider.rig-rust`) so turns execute through the RIG (rig-core)
            // simple agent instead of the cloudrouter account pool.
            return self.ensure_default_rig_runtime_binding(EnsureRuntimeBindingRequest {
                tenant_id,
                organization_id,
                owner_user_id,
                agent_id,
                session_id,
                descriptor: None,
                subject,
                requested_at,
            });
        };
        let existing = self
            .service
            .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
                tenant_id,
                organization_id,
                path_agent_id: agent_id.to_string(),
                session_id: session_id.to_string(),
                runtime_binding_id: descriptor.runtime_binding_id.clone(),
                owner_scope: Some(owner_user_id),
                requested_by: subject.clone(),
            });
        match existing {
            Ok(existing) => {
                if !runtime_binding_identity_matches_descriptor(&existing, descriptor) {
                    return Err(
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                            "runtime binding descriptor conflicts with the current session binding"
                                .into(),
                        ),
                    );
                }
                self.reconcile_runtime_binding_provider_directory(
                    existing,
                    descriptor,
                    owner_user_id,
                    agent_id,
                    subject,
                    requested_at,
                )?;
                return Ok(());
            }
            Err(error)
                if error.detail_value("sdkwork.not_found") == Some("true")
                    && error.message() == "session runtime binding not found" =>
            {
                // No runtime binding yet: fall through to create it.
            }
            Err(error) => {
                return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    error.to_string(),
                ));
            }
        }

        let command = crate::application::CreateSessionRuntimeBindingCommand {
            tenant_id,
            organization_id,
            path_agent_id: agent_id.to_string(),
            session_id: session_id.to_string(),
            runtime_binding_id: Some(descriptor.runtime_binding_id.clone()),
            runtime_location_id: descriptor.runtime_location_id.clone(),
            host_mode: descriptor.host_mode.clone(),
            transport_kind: descriptor.transport_kind.clone(),
            provider_binding_id: descriptor.provider_binding_id.clone(),
            model_id: descriptor.model_id.clone(),
            provider_id: descriptor.provider_id.clone(),
            provider_session_id: descriptor.provider_session_id.clone(),
            provider_session_tree_id: descriptor.provider_session_tree_id.clone(),
            provider_parent_session_id: descriptor.provider_parent_session_id.clone(),
            provider_forked_from_session_id: descriptor.provider_forked_from_session_id.clone(),
            provider_directory: descriptor.provider_directory.clone(),
            owner_scope: Some(owner_user_id),
            requested_by: subject.clone(),
            requested_at: requested_at.to_string(),
        };
        let creation_result = if self.provider_session_history_reconciliation {
            self.service
                .reconcile_provider_session_history_runtime_binding(command)
        } else {
            self.service.create_session_runtime_binding(command)
        };
        match creation_result {
            Ok(_) => {}
            Err(error)
                if self.provider_session_history_reconciliation
                    && error.kind() == sdkwork_agent_kernel::KernelErrorKind::Conflict =>
            {
                let existing = self
                    .service
                    .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
                        tenant_id,
                        organization_id,
                        path_agent_id: agent_id.to_string(),
                        session_id: session_id.to_string(),
                        runtime_binding_id: descriptor.runtime_binding_id.clone(),
                        owner_scope: Some(owner_user_id),
                        requested_by: subject.clone(),
                    })
                    .map_err(|read_error| {
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                            read_error.to_string(),
                        )
                    })?;
                if !runtime_binding_identity_matches_descriptor(&existing, descriptor) {
                    return Err(
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                            "concurrent provider Session runtime binding conflicts with the requested descriptor"
                                .into(),
                        ),
                    );
                }
                self.reconcile_runtime_binding_provider_directory(
                    existing,
                    descriptor,
                    owner_user_id,
                    agent_id,
                    subject,
                    requested_at,
                )?;
            }
            Err(error) => {
                return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    error.to_string(),
                ));
            }
        }
        Ok(())
    }

    fn reconcile_runtime_binding_provider_directory(
        &self,
        existing: AgentSessionRuntimeBindingRecord,
        descriptor: &sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor,
        owner_user_id: u64,
        agent_id: &str,
        subject: sdkwork_agent_kernel::PolicySubject,
        requested_at: &str,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<()> {
        let Some(provider_directory) = descriptor.provider_directory.clone() else {
            return Ok(());
        };
        if !self.provider_session_history_reconciliation {
            if !runtime_binding_directory_matches_descriptor(&existing, descriptor) {
                return Err(
                    sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                        "runtime binding provider directory conflicts with the current session binding"
                            .into(),
                    ),
                );
            }
            return Ok(());
        }
        let mut current = existing;
        for attempt in 0..2 {
            match self
                .service
                .reconcile_provider_session_history_runtime_binding_directory(
                    crate::application::ReconcileProviderSessionRuntimeBindingDirectoryCommand {
                        tenant_id: current.tenant_id,
                        organization_id: current.organization_id,
                        path_agent_id: agent_id.to_string(),
                        session_id: current.session_id.clone(),
                        runtime_binding_id: current.runtime_binding_id.clone(),
                        expected_version: current.version,
                        provider_directory: provider_directory.clone(),
                        owner_scope: Some(owner_user_id),
                        requested_by: subject.clone(),
                        requested_at: requested_at.to_string(),
                    },
                ) {
                Ok(_) => return Ok(()),
                Err(error)
                    if attempt == 0
                        && error.kind() == sdkwork_agent_kernel::KernelErrorKind::Conflict =>
                {
                    current = self
                        .service
                        .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
                            tenant_id: current.tenant_id,
                            organization_id: current.organization_id,
                            path_agent_id: agent_id.to_string(),
                            session_id: current.session_id.clone(),
                            runtime_binding_id: current.runtime_binding_id.clone(),
                            owner_scope: Some(owner_user_id),
                            requested_by: subject.clone(),
                        })
                        .map_err(|read_error| {
                            sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                                read_error.to_string(),
                            )
                        })?;
                    if !runtime_binding_identity_matches_descriptor(&current, descriptor) {
                        return Err(
                            sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                                "concurrent provider Session runtime binding conflicts with the requested descriptor"
                                    .into(),
                            ),
                        );
                    }
                    if runtime_binding_directory_matches_descriptor(&current, descriptor) {
                        return Ok(());
                    }
                }
                Err(error) => {
                    return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                        error.to_string(),
                    ));
                }
            }
        }
        Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
            "provider Session runtime binding directory reconciliation exhausted its retry"
                .to_string(),
        ))
    }
}

fn runtime_binding_identity_matches_descriptor(
    record: &AgentSessionRuntimeBindingRecord,
    descriptor: &sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor,
) -> bool {
    record.status == AgentSessionRuntimeBindingStatus::Active
        && record.is_current
        && record.runtime_binding_id == descriptor.runtime_binding_id
        && record.runtime_location_id == descriptor.runtime_location_id
        && record.host_mode == descriptor.host_mode
        && record.transport_kind == descriptor.transport_kind
        && record.provider_binding_id == descriptor.provider_binding_id
        && record.model_id == descriptor.model_id
        && record.provider_id == descriptor.provider_id
        && record.provider_session_id == descriptor.provider_session_id
        && record.provider_session_tree_id == descriptor.provider_session_tree_id
        && record.provider_parent_session_id == descriptor.provider_parent_session_id
        && record.provider_forked_from_session_id == descriptor.provider_forked_from_session_id
}

fn runtime_binding_directory_matches_descriptor(
    record: &AgentSessionRuntimeBindingRecord,
    descriptor: &sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor,
) -> bool {
    descriptor
        .provider_directory
        .as_ref()
        .is_none_or(|directory| {
            record.provider_title == directory.title
                && record.provider_title_source == directory.title_source
                && record.provider_preview == directory.preview
                && record.provider_created_at == directory.created_at
                && record.provider_updated_at == directory.updated_at
                && record.provider_recency_at == directory.recency_at
                && record.provider_pinned == directory.pinned
                && record.provider_archived == directory.archived
                && record.provider_visible == directory.visible
                && record.provider_sort_key
                    == (!directory.sort_key.trim().is_empty()).then(|| directory.sort_key.clone())
                && record.provider_source == directory.source
        })
}

#[cfg(test)]
fn runtime_binding_matches_descriptor(
    record: &AgentSessionRuntimeBindingRecord,
    descriptor: &sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor,
) -> bool {
    runtime_binding_identity_matches_descriptor(record, descriptor)
        && runtime_binding_directory_matches_descriptor(record, descriptor)
}

fn map_facade_session_kind(
    kind: sdkwork_agents_runtime_facade::AgentsSessionKind,
) -> AgentSessionKind {
    match kind {
        sdkwork_agents_runtime_facade::AgentsSessionKind::Assistant => AgentSessionKind::Assistant,
        sdkwork_agents_runtime_facade::AgentsSessionKind::Coding => AgentSessionKind::Coding,
        sdkwork_agents_runtime_facade::AgentsSessionKind::Automation => {
            AgentSessionKind::Automation
        }
        sdkwork_agents_runtime_facade::AgentsSessionKind::ImDispatch => {
            AgentSessionKind::ImDispatch
        }
    }
}

fn map_facade_entry_surface(
    surface: sdkwork_agents_runtime_facade::AgentsSessionEntrySurface,
) -> AgentSessionEntrySurface {
    match surface {
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Pc => {
            AgentSessionEntrySurface::Pc
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::H5 => {
            AgentSessionEntrySurface::H5
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Flutter => {
            AgentSessionEntrySurface::Flutter
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::MiniProgram => {
            AgentSessionEntrySurface::MiniProgram
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Api => {
            AgentSessionEntrySurface::Api
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::ImDispatch => {
            AgentSessionEntrySurface::ImDispatch
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Automation => {
            AgentSessionEntrySurface::Automation
        }
    }
}

impl sdkwork_agents_runtime_facade::AgentsSessionFacade for HttpAgentsSessionFacade {
    fn resolve_or_create_session(
        &self,
        request: sdkwork_agents_runtime_facade::ResolveAgentsSessionRequest,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<
        sdkwork_agents_runtime_facade::ResolvedAgentsSession,
    > {
        sdkwork_agents_runtime_facade::validate_resolve_agents_session_request(&request)?;
        let subject = facade_policy_subject(
            request.tenant_id,
            &request.actor.subject_id,
            &request.actor.roles,
        );
        match self.service.get_session(GetSessionCommand {
            tenant_id: request.tenant_id,
            organization_id: request.organization_id,
            path_agent_id: request.agent_id.clone(),
            session_id: request.session_id.clone(),
            owner_scope: Some(request.owner_user_id),
            requested_by: subject.clone(),
        }) {
            Ok(mut existing) => {
                if existing.organization_id != request.organization_id {
                    return Err(
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                            "session organization mismatch".into(),
                        ),
                    );
                }
                if existing
                    .idempotency_key
                    .as_deref()
                    .is_some_and(|value| value != request.idempotency_key)
                    || existing
                        .payload_hash
                        .as_deref()
                        .is_some_and(|value| value != request.payload_hash)
                {
                    return Err(
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                            "session resolution idempotency payload conflicts with the existing session"
                                .into(),
                        ),
                    );
                }
                if self.provider_session_history_reconciliation
                    && existing.title.as_deref() != Some(request.title.as_str())
                {
                    existing = match self
                        .service
                        .reconcile_provider_session_history_session_title(UpdateSessionCommand {
                            tenant_id: request.tenant_id,
                            organization_id: request.organization_id,
                            path_agent_id: request.agent_id.clone(),
                            session_id: request.session_id.clone(),
                            title: Some(request.title.clone()),
                            project_id: None,
                            expected_version: Some(existing.version),
                            owner_scope: Some(request.owner_user_id),
                            requested_by: subject.clone(),
                            requested_at: request.requested_at.clone(),
                        }) {
                        Ok(updated) => updated,
                        Err(error)
                            if error.kind() == sdkwork_agent_kernel::KernelErrorKind::Conflict =>
                        {
                            return self.resolve_or_create_session(request);
                        }
                        Err(error) => {
                            return Err(
                                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                                    error.to_string(),
                                ),
                            );
                        }
                    };
                }
                self.ensure_runtime_binding(EnsureRuntimeBindingRequest {
                    tenant_id: request.tenant_id,
                    organization_id: request.organization_id,
                    owner_user_id: request.owner_user_id,
                    agent_id: &request.agent_id,
                    session_id: &request.session_id,
                    descriptor: request.runtime_binding.as_ref(),
                    subject,
                    requested_at: &request.requested_at,
                })?;
                return Ok(sdkwork_agents_runtime_facade::ResolvedAgentsSession {
                    session_id: existing.session_id,
                    created: false,
                    version: existing.version,
                });
            }
            Err(error)
                if error.detail_value("sdkwork.not_found") == Some("true")
                    && error.message() == "session not found" =>
            {
                // Session does not exist yet: fall through to create it.
            }
            Err(error) => {
                return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    error.to_string(),
                ));
            }
        }
        let command = CreateSessionCommand {
            tenant_id: request.tenant_id,
            organization_id: request.organization_id,
            agent_id: request.agent_id.clone(),
            owner_user_id: request.owner_user_id,
            session_id: request.session_id.clone(),
            project_id: request.project_id.clone(),
            session_kind: map_facade_session_kind(request.session_kind),
            entry_surface: map_facade_entry_surface(request.entry_surface),
            source_module: request.source_module.clone(),
            source_context_kind: request.source_context_kind.clone(),
            source_context_id: request.source_context_id.clone(),
            parent_session_id: request.parent_session_id.clone(),
            forked_from_turn_id: request.forked_from_turn_id.clone(),
            title: Some(request.title.clone()),
            idempotency_key: Some(request.idempotency_key.clone()),
            payload_hash: Some(request.payload_hash.clone()),
            requested_by: subject.clone(),
            requested_at: request.requested_at.clone(),
        };
        let creation_result = if self.provider_session_history_reconciliation {
            self.service
                .reconcile_provider_session_history_session(command)
        } else {
            self.service.create_session(command)
        };
        let created = match creation_result {
            Ok(created) => created,
            Err(error)
                if self.provider_session_history_reconciliation
                    && error.kind() == sdkwork_agent_kernel::KernelErrorKind::Conflict =>
            {
                return self.resolve_or_create_session(request);
            }
            Err(error) => {
                return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    error.to_string(),
                ));
            }
        };
        if let Err(binding_error) = self.ensure_runtime_binding(EnsureRuntimeBindingRequest {
            tenant_id: request.tenant_id,
            organization_id: request.organization_id,
            owner_user_id: request.owner_user_id,
            agent_id: &request.agent_id,
            session_id: &created.session_id,
            descriptor: request.runtime_binding.as_ref(),
            subject: subject.clone(),
            requested_at: &request.requested_at,
        }) {
            // A Session created without its canonical runtime binding can
            // never be synchronized or read meaningfully; it would linger as
            // a permanently empty conversation. Best-effort close and archive
            // the just-created Session, then surface the binding failure.
            let close_result = self.service.close_session(CloseSessionCommand {
                tenant_id: request.tenant_id,
                organization_id: request.organization_id,
                path_agent_id: request.agent_id.clone(),
                session_id: created.session_id.clone(),
                expected_version: Some(created.version),
                owner_scope: Some(request.owner_user_id),
                requested_by: subject.clone(),
                requested_at: request.requested_at.clone(),
            });
            if let Ok(closed) = close_result {
                let _ = self.service.archive_session(ArchiveSessionCommand {
                    tenant_id: request.tenant_id,
                    organization_id: request.organization_id,
                    path_agent_id: request.agent_id.clone(),
                    session_id: created.session_id.clone(),
                    expected_version: Some(closed.version),
                    owner_scope: Some(request.owner_user_id),
                    requested_by: subject.clone(),
                    requested_at: request.requested_at.clone(),
                });
            }
            return Err(binding_error);
        }
        Ok(sdkwork_agents_runtime_facade::ResolvedAgentsSession {
            session_id: created.session_id,
            created: true,
            version: created.version,
        })
    }

    fn complete_turn(
        &self,
        request: sdkwork_agents_runtime_facade::CompleteAgentsTurnRequest,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<
        sdkwork_agents_runtime_facade::CompletedAgentsTurn,
    > {
        sdkwork_agents_runtime_facade::validate_session_actor(&request.actor)?;
        let subject = facade_policy_subject(
            request.tenant_id,
            &request.actor.subject_id,
            &request.actor.roles,
        );
        let payload_hash = sdkwork_utils_rust::sha256_hash(request.content.as_bytes());
        let result = self
            .service
            .execute_turn(CreateTurnCommand {
                tenant_id: request.tenant_id,
                organization_id: request.organization_id,
                agent_id: request.agent_id,
                session_id: request.session_id,
                turn_id: None,
                content: request.content,
                content_type: request.content_type,
                turn_mode: crate::agent_turn::AgentTurnMode::Interactive,
                runtime_binding_id: None,
                requested_model_id: None,
                access_mode_id: None,
                idempotency_key: request.idempotency_key.clone(),
                payload_hash,
                client_request_id: Some(request.client_request_id),
                drive_refs: Vec::new(),
                owner_scope: Some(request.owner_user_id),
                requested_by: subject,
                requested_at: request.requested_at,
                prefer_stream: false,
                system_prompt: None,
            auth_token: None,
            access_token: None,
            })
            .map_err(|error| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(error.to_string())
            })?;
        let turn_id = result
            .user_input_item
            .turn_id
            .clone()
            .or(result.assistant_output_item.turn_id.clone())
            .ok_or_else(|| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    "completed Agents turn did not return turnId".into(),
                )
            })?;
        let response_content = result.assistant_output_item.content.ok_or_else(|| {
            sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                "completed Agents turn returned no assistant content".into(),
            )
        })?;
        Ok(sdkwork_agents_runtime_facade::CompletedAgentsTurn {
            session_id: result.session.session_id,
            turn_id,
            request_item_id: result.user_input_item.item_id,
            response_item_id: result.assistant_output_item.item_id,
            response_content,
        })
    }

    fn get_turn_by_idempotency(
        &self,
        request: sdkwork_agents_runtime_facade::GetAgentsTurnByIdempotencyRequest,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<
        Option<sdkwork_agents_runtime_facade::AgentsTurnSnapshot>,
    > {
        sdkwork_agents_runtime_facade::validate_session_actor(&request.actor)?;
        let subject = facade_policy_subject(
            request.tenant_id,
            &request.actor.subject_id,
            &request.actor.roles,
        );
        let Some(turn) = self
            .service
            .get_turn_by_idempotency(GetTurnByIdempotencyCommand {
                tenant_id: request.tenant_id,
                organization_id: request.organization_id,
                path_agent_id: request.agent_id.clone(),
                session_id: request.session_id.clone(),
                owner_user_id: request.owner_user_id,
                idempotency_key: request.idempotency_key,
                requested_by: subject.clone(),
            })
            .map_err(|error| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(error.to_string())
            })?
        else {
            return Ok(None);
        };
        let response_content = match turn.response_item_id.as_deref() {
            Some(item_id) if turn.status == crate::AgentTurnStatus::Completed => {
                self.service
                    .get_session_item(GetSessionItemCommand {
                        tenant_id: request.tenant_id,
                        organization_id: request.organization_id,
                        path_agent_id: request.agent_id,
                        session_id: request.session_id,
                        item_id: item_id.to_owned(),
                        owner_scope: Some(request.owner_user_id),
                        requested_by: subject,
                    })
                    .map_err(|error| {
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                            error.to_string(),
                        )
                    })?
                    .content
            }
            _ => None,
        };
        let status = match turn.status {
            crate::AgentTurnStatus::Requested => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Requested
            }
            crate::AgentTurnStatus::Running => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Running
            }
            crate::AgentTurnStatus::Completed => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Completed
            }
            crate::AgentTurnStatus::Failed => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Failed
            }
            crate::AgentTurnStatus::Cancelled => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Cancelled
            }
        };
        Ok(Some(sdkwork_agents_runtime_facade::AgentsTurnSnapshot {
            session_id: turn.session_id,
            turn_id: turn.turn_id,
            status,
            request_item_id: turn.request_item_id,
            response_item_id: turn.response_item_id,
            response_content,
            error_code: turn.error_code,
        }))
    }
}

fn facade_policy_subject(
    tenant_id: u64,
    subject_id: &str,
    roles: &[String],
) -> sdkwork_agent_kernel::PolicySubject {
    roles.iter().fold(
        sdkwork_agent_kernel::PolicySubject::new(subject_id, tenant_id.to_string()),
        |subject, role| subject.with_role(role.clone()),
    )
}

/// Raw app-api route tree without gateway or web-framework middleware.
pub fn build_app_routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/app/v3/api/ai/workspaces",
            get(app_list_workspaces).post(app_create_workspace),
        )
        .route(
            "/app/v3/api/ai/workspaces/default",
            post(app_ensure_default_workspace),
        )
        .route(
            "/app/v3/api/ai/workspaces/{workspaceId}",
            get(app_get_workspace)
                .patch(app_update_workspace)
                .delete(app_delete_workspace),
        )
        .route(
            "/app/v3/api/ai/workspaces/{workspaceId}/archive",
            post(app_archive_workspace),
        )
        .route(
            "/app/v3/api/ai/workspaces/{workspaceId}/sessions",
            get(app_list_workspace_sessions),
        )
        .route("/app/v3/api/ai/projects/import", post(app_import_project))
        .route(
            "/app/v3/api/ai/projects",
            get(app_list_projects).post(app_create_project),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}",
            get(app_get_project)
                .patch(app_update_project)
                .delete(app_delete_project),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/archive",
            post(app_archive_project),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/composition_slots",
            get(app_list_project_composition_slots).post(app_create_project_composition_slot),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/composition_slots/{slotId}",
            get(app_get_project_composition_slot)
                .patch(app_update_project_composition_slot)
                .delete(app_delete_project_composition_slot),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/sessions",
            get(app_list_project_sessions).post(app_create_project_session),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/sessions/synchronize",
            post(app_synchronize_project_sessions),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/sessions/{sessionId}",
            get(app_get_project_session),
        )
        .route(
            "/app/v3/api/ai/session_activity_summaries",
            get(app_list_session_activity_summaries),
        )
        .route(
            "/app/v3/api/ai/agents",
            get(app_list_agents).post(app_create_agent),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}",
            get(app_get_agent)
                .patch(app_update_agent)
                .delete(app_delete_agent),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/restore",
            post(app_restore_agent),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/provider_bindings",
            get(app_list_provider_bindings).post(app_add_provider_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            post(app_activate_provider_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/preview_responses",
            post(app_create_preview_response),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/prompt_optimizations",
            post(app_create_prompt_optimization),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/composition_slots",
            get(app_list_composition_slots).post(app_create_composition_slot),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
            get(app_get_composition_slot)
                .patch(app_update_composition_slot)
                .delete(app_delete_composition_slot),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions",
            get(app_list_sessions).post(app_create_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/user_states",
            get(app_list_session_user_states),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
            get(app_get_session)
                .patch(app_update_session)
                .delete(app_delete_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
            post(app_close_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/user_state",
            get(app_get_session_user_state).patch(app_update_session_user_state),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/item_feedback",
            get(app_list_item_feedback),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
            get(app_list_session_items),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/synchronize",
            post(app_synchronize_session_items),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
            get(app_get_session_item),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}/feedback",
            axum::routing::patch(app_update_item_feedback),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
            get(app_list_turns).post(app_create_turn),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
            get(app_get_turn),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
            post(app_cancel_turn),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue",
            get(app_list_turn_input_queue_entries).post(app_create_turn_input_queue_entry),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/clear",
            post(app_clear_turn_input_queue_entries),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/reorder",
            post(app_reorder_turn_input_queue_entries),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/claim_next",
            post(app_claim_next_turn_input_queue_entry),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/{queueEntryId}",
            axum::routing::patch(app_update_turn_input_queue_entry)
                .delete(app_remove_turn_input_queue_entry),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/{queueEntryId}/fail",
            post(app_fail_turn_input_queue_entry),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/{queueEntryId}/retry",
            post(app_retry_turn_input_queue_entry),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
            get(app_list_interactions).post(app_create_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
            get(app_get_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
            post(app_claim_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(app_approve_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(app_answer_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/resolve",
            post(app_resolve_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
            get(app_list_session_checkpoints).post(app_create_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
            get(app_get_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
            post(app_restore_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
            post(app_invalidate_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
            get(app_list_session_runtime_bindings).post(app_create_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
            get(app_get_session_runtime_binding).patch(app_update_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
            post(app_activate_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
            post(app_deactivate_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks",
            get(app_list_tasks).post(app_create_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}",
            get(app_get_task).put(replace_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/pause",
            post(pause_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/resume",
            post(resume_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
            post(app_cancel_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
            post(app_execute_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs",
            get(list_task_runs),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}",
            get(get_task_run),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/retry",
            post(retry_task_run),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/cancel",
            post(cancel_task_run),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/attempts",
            get(list_task_run_attempts),
        )
        .route(
            "/app/v3/api/ai/agent_engines",
            get(app_list_agent_engines),
        )
        .route(
            "/app/v3/api/ai/model_configurations/apply",
            post(app_apply_model_configuration),
        )
        .route(
            "/app/v3/api/ai/model_configurations",
            get(app_list_model_configurations),
        )
        .route(
            "/app/v3/api/ai/model_configurations/{engineId}/{profileId}",
            get(app_get_model_configuration),
        )
        .route(
            "/app/v3/api/ai/model_configurations/{engineId}/{profileId}/status",
            get(app_get_model_configuration_status),
        )
        .route(
            "/app/v3/api/ai/model_configurations/{engineId}/config_file",
            get(app_get_agent_engine_config_file),
        )
        .route(
            "/app/v3/api/ai/model_configurations/{engineId}/{profileId}/archive",
            post(app_archive_model_configuration),
        )
        .route(
            "/app/v3/api/ai/model_configurations/migrate",
            post(app_migrate_model_configuration),
        )
        .route(
            "/app/v3/api/ai/model_selections/apply",
            post(app_apply_model_selection),
        )
        .route(
            "/app/v3/api/ai/mcp_servers",
            get(app_list_mcp_servers),
        )
        .route(
            "/app/v3/api/ai/tools",
            get(app_list_media_tools),
        )
        .route(
            "/app/v3/api/ai/tools/{toolId}/invoke",
            post(app_invoke_media_tool),
        )
        .route(
            "/app/v3/api/ai/assets",
            get(app_list_tool_assets),
        )
        .layer(axum::middleware::from_fn(
            middleware::reject_client_scope_selectors,
        ))
}

/// Raw open-api route tree without gateway or web-framework middleware.
pub fn build_open_routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/agent/v3/api/ai/agents",
            get(backend_list_agents).post(backend_create_agent),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}",
            get(backend_get_agent)
                .patch(backend_update_agent)
                .delete(open_delete_agent),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
            get(backend_list_provider_bindings).post(backend_add_provider_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            post(backend_activate_provider_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/preview_responses",
            post(open_create_preview_response),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/prompt_optimizations",
            post(open_create_prompt_optimization),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/composition_slots",
            get(backend_list_composition_slots).post(backend_create_composition_slot),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
            get(backend_get_composition_slot)
                .patch(backend_update_composition_slot)
                .delete(backend_delete_composition_slot),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions",
            get(backend_list_sessions).post(backend_create_session),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
            get(backend_get_session),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
            post(backend_close_session),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
            get(backend_list_session_items),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
            get(backend_get_session_item),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
            get(backend_list_turns).post(backend_create_turn),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
            get(backend_get_turn),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
            post(backend_cancel_turn),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
            get(backend_list_interactions).post(backend_create_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
            get(backend_get_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
            post(backend_claim_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(backend_approve_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(backend_answer_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/resolve",
            post(backend_resolve_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
            get(backend_list_session_checkpoints).post(backend_create_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
            get(backend_get_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
            post(backend_restore_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
            post(backend_invalidate_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
            get(backend_list_session_runtime_bindings).post(backend_create_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
            get(backend_get_session_runtime_binding).patch(backend_update_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
            post(backend_activate_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
            post(backend_deactivate_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks",
            get(backend_list_tasks).post(backend_create_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}",
            get(backend_get_task).put(replace_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/pause",
            post(pause_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/resume",
            post(resume_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
            post(backend_cancel_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
            post(backend_execute_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs",
            get(list_task_runs),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}",
            get(get_task_run),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/retry",
            post(retry_task_run),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/cancel",
            post(cancel_task_run),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/attempts",
            get(list_task_run_attempts),
        )
        .layer(axum::middleware::from_fn(
            middleware::reject_client_scope_selectors,
        ))
}

/// Raw backend-api route tree without gateway or web-framework middleware.
pub fn build_backend_routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/backend/v3/api/ai/agents",
            get(backend_list_agents).post(backend_create_agent),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}",
            get(backend_get_agent).patch(backend_update_agent),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/status",
            post(backend_update_agent_status),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/restore",
            post(backend_restore_agent),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/audit_events",
            get(backend_list_agent_audit_events),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
            get(backend_list_provider_bindings).post(backend_add_provider_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            post(backend_activate_provider_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/composition_slots",
            get(backend_list_composition_slots).post(backend_create_composition_slot),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
            get(backend_get_composition_slot)
                .patch(backend_update_composition_slot)
                .delete(backend_delete_composition_slot),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions",
            get(backend_list_sessions).post(backend_create_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
            get(backend_get_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
            post(backend_close_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/archive",
            post(backend_archive_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
            get(backend_list_session_items),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
            get(backend_get_session_item),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
            get(backend_list_turns).post(backend_create_turn),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
            get(backend_get_turn),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
            post(backend_cancel_turn),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
            get(backend_list_interactions).post(backend_create_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
            get(backend_get_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
            post(backend_claim_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(backend_approve_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(backend_answer_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/resolve",
            post(backend_resolve_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
            get(backend_list_session_checkpoints).post(backend_create_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
            get(backend_get_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
            post(backend_restore_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
            post(backend_invalidate_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
            get(backend_list_session_runtime_bindings).post(backend_create_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
            get(backend_get_session_runtime_binding).patch(backend_update_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
            post(backend_activate_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
            post(backend_deactivate_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks",
            get(backend_list_tasks).post(backend_create_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}",
            get(backend_get_task).put(replace_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/pause",
            post(pause_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/resume",
            post(resume_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
            post(backend_cancel_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
            post(backend_execute_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs",
            get(list_task_runs),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}",
            get(get_task_run),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/retry",
            post(retry_task_run),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/cancel",
            post(cancel_task_run),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/attempts",
            get(list_task_run_attempts),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/reconcile",
            post(reconcile_task_run),
        )
        .route(
            "/backend/v3/api/ai/tools",
            get(backend_list_media_tools),
        )
        .route(
            "/backend/v3/api/ai/tools/{toolId}/configuration",
            put(backend_update_media_tool_configuration),
        )
        .layer(axum::middleware::from_fn(
            middleware::reject_client_scope_selectors,
        ))
}

/// Prometheus metrics endpoint handler (O-01).
/// Exposes service-level metrics in Prometheus text exposition format.
pub async fn serve_agents_metrics() -> impl IntoResponse {
    let metrics = crate::infrastructure::AgentMetricsRegistry::global().snapshot();
    let prometheus_text = metrics.to_prometheus_text();

    let mut response = prometheus_text.into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    response
}

/// Raw combined route tree for served production mounts.
pub fn build_combined_routes() -> Router<AgentHttpState> {
    build_open_routes()
        .merge(build_app_routes())
        .merge(build_backend_routes())
        .route("/metrics/agents", get(serve_agents_metrics))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApplyAgentModelConfigurationRequest {
    configuration_id: String,
    engine_id: String,
    /// Optional configuration subject. Defaults to the canonical engine
    /// agent; a user-created agent (e.g. `agent.chat.default`) becomes the
    /// owner of the custom LLM provider configuration.
    agent_id: Option<String>,
    vendor_code: String,
    base_url: String,
    api_key: Option<String>,
    default_model_id: String,
    supported_model_ids: Vec<String>,
    #[serde(default)]
    supported_provider_ids: Vec<String>,
    #[serde(with = "sdkwork_utils_rust::serde_int64::option", default)]
    input_context_tokens: Option<i64>,
    #[serde(with = "sdkwork_utils_rust::serde_int64::option", default)]
    output_context_tokens: Option<i64>,
    #[serde(with = "sdkwork_utils_rust::serde_int64::option", default)]
    tool_call_rounds: Option<i64>,
    #[serde(default)]
    supports_multimodal: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppliedAgentModelConfigurationResponse {
    configuration_id: String,
    profile_id: String,
    engine_id: String,
    agent_id: String,
    provider_scope: String,
    vendor_code: String,
    base_url: String,
    default_model_id: String,
    supported_model_ids: Vec<String>,
    supported_provider_ids: Vec<String>,
    input_context_tokens: Option<i64>,
    output_context_tokens: Option<i64>,
    tool_call_rounds: Option<i64>,
    supports_multimodal: bool,
    api_key_configured: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApplyAgentModelSelectionRequest {
    configuration_id: Option<String>,
    engine_id: String,
    /// Optional configuration subject; defaults to the canonical engine agent.
    agent_id: Option<String>,
    model_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppliedAgentModelSelectionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    configuration_id: Option<String>,
    profile_id: String,
    engine_id: String,
    agent_id: String,
    provider_scope: String,
    model_id: String,
}

async fn app_apply_model_configuration(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<ApplyAgentModelConfigurationRequest>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AppliedAgentModelConfigurationResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let item =
            apply_agent_model_configuration(state, scope, web_ctx.request_id.0.as_str(), body)?;
        Ok(ResourceData { item })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

fn apply_agent_model_configuration(
    state: AgentHttpState,
    scope: RequestScope,
    request_id: &str,
    body: ApplyAgentModelConfigurationRequest,
) -> ApiResult<AppliedAgentModelConfigurationResponse> {
    validate_model_configuration_identifier("configurationId", &body.configuration_id, 160)?;
    let engine_id = body.engine_id.trim();
    let agent_id = resolve_model_configuration_agent_id(body.agent_id.as_deref(), engine_id)?;

    let supported_provider_ids =
        normalize_supported_model_provider_ids(body.supported_provider_ids, engine_id)?;
    let profile_id = model_configuration_profile_id(
        &scope,
        &agent_id,
        engine_id,
        body.configuration_id.trim(),
    );
    let profile_scope = profile_scope_from_request(&scope)?;
    let existing_profile = state
        .model_configuration_runtime
        .configurations
        .lock()
        .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?
        .find_profile_in_scope(agent_id.as_str(), &profile_id, &profile_scope)
        .map_err(|_| ApiProblem::internal("model configuration could not be loaded"))?;
    let existing_secret_ref = existing_profile.as_ref().and_then(|profile| {
        profile
            .secret_bindings
            .iter()
            .find(|binding| {
                matches!(
                    binding.binding_kind,
                    sdkwork_agent_kernel::AgentSecretBindingKind::LlmApiKey
                )
            })
            .map(|binding| binding.secret_ref.clone())
    });
    if existing_profile.is_some() && existing_secret_ref.is_none() {
        return Err(ApiProblem::internal(
            "stored model configuration is missing its credential binding",
        ));
    }
    let supplied_api_key = body.api_key.filter(|value| !value.trim().is_empty());
    if supplied_api_key
        .as_ref()
        .is_some_and(|value| value.len() > 16_384)
    {
        return Err(ApiProblem::validation("apiKey exceeds the maximum length"));
    }

    let mut created_secret_ref = None;
    let api_key_secret_ref = if let Some(secret_ref) = &existing_secret_ref {
        secret_ref.clone()
    } else {
        let api_key = supplied_api_key.as_ref().ok_or_else(|| {
            ApiProblem::validation("apiKey is required for a new model configuration")
        })?;
        let metadata = state
            .model_configuration_runtime
            .secrets
            .lock()
            .map_err(|_| ApiProblem::internal("model credential store is unavailable"))?
            .create_secret(
                SecretCreateRequest::new(
                    format!("Model credential for {}", body.configuration_id),
                    SecretType::ApiKey,
                    api_key,
                )
                .with_description("Agent model configuration credential")
                .with_tag("tenant_id", &scope.tenant_id)
                .with_tag("owner_user_id", &scope.owner_user_id),
            )
            .map_err(|_| ApiProblem::internal("model credential could not be stored"))?;
        created_secret_ref = Some(metadata.secret_id.clone());
        metadata.secret_id
    };

    // Resolve the plaintext credential for provider config-file materialization:
    // prefer the freshly supplied key, then read the stored secret back so a
    // re-apply refreshes the external CLI config with the same credential.
    // The value is transient on the kernel request (never persisted into
    // profiles) and is redacted from its Debug output.
    let materialization_api_key = supplied_api_key.clone().or_else(|| {
        state
            .model_configuration_runtime
            .secrets
            .lock()
            .ok()
            .and_then(|secrets| {
                secrets
                    .access_secret(SecretAccessRequest::new(&api_key_secret_ref, agent_id.clone()))
                    .ok()
                    .and_then(|result| result.value)
            })
    });
    let mut kernel_request = sdkwork_agents_runtime_facade::AgentModelConfigurationRequest::new(
        request_id,
        agent_id.as_str(),
        &profile_id,
        body.vendor_code.trim(),
        body.base_url.trim(),
        &api_key_secret_ref,
        body.default_model_id.trim(),
    )
    .with_supported_models(body.supported_model_ids)
    .with_multimodal_support(body.supports_multimodal);
    if let Some(api_key) = materialization_api_key {
        kernel_request = kernel_request.with_api_key_materialization(api_key);
    }
    if let Some(value) = body.input_context_tokens {
        kernel_request = kernel_request.with_input_context_tokens(value);
    }
    if let Some(value) = body.output_context_tokens {
        kernel_request = kernel_request.with_output_context_tokens(value);
    }
    if let Some(value) = body.tool_call_rounds {
        kernel_request = kernel_request.with_tool_call_rounds(value);
    }

    let application = match sdkwork_agents_runtime_facade::apply_agent_engine_model_configuration(
        engine_id,
        &kernel_request,
    ) {
        Ok(application) => application,
        Err(error) => {
            if let Some(secret_ref) = created_secret_ref {
                if let Ok(mut secrets) = state.model_configuration_runtime.secrets.lock() {
                    let _ = secrets.delete_secret(&secret_ref);
                }
            }
            return Err(ApiProblem::validation(error.to_string()));
        }
    };

    if let Err(_error) = state
        .model_configuration_runtime
        .configurations
        .lock()
        .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?
        .save_profile_in_scope(application.profile.clone(), &profile_scope)
    {
        if let Some(secret_ref) = created_secret_ref {
            if let Ok(mut secrets) = state.model_configuration_runtime.secrets.lock() {
                let _ = secrets.delete_secret(&secret_ref);
            }
        }
        return Err(ApiProblem::internal(
            "provider model configuration could not be stored",
        ));
    }

    // Apply a freshly materialized rig (simple agent) configuration to the
    // agent-scoped engine host so the live backend takes effect immediately
    // without affecting other agents' scopes.
    if engine_id == "rig" {
        let refresh_result = crate::runtime_facade_bridge::refresh_rig_agent_engine_for(
            parse_scope_tenant_id(&scope),
            &agent_id,
            &application.profile.configuration,
            std::sync::Arc::new(
                crate::runtime_facade_bridge::ModelConfigurationRuntimeHostProvider::new(
                    state.model_configuration_runtime.clone(),
                ),
            ),
        );
        if let Err(error) = refresh_result {
            tracing::warn!(
                engine_id,
                error = %error,
                "rig agent engine refresh after model configuration failed"
            );
        }
    }

    if let (Some(api_key), Some(secret_ref)) = (supplied_api_key, existing_secret_ref.as_ref()) {
        state
            .model_configuration_runtime
            .secrets
            .lock()
            .map_err(|_| ApiProblem::internal("model credential store is unavailable"))?
            .rotate_secret(
                SecretRotateRequest::new(secret_ref, api_key, scope.owner_user_id.clone())
                    .with_reason("Agent model configuration updated"),
            )
            .map_err(|_| ApiProblem::internal("model credential could not be updated"))?;
    }

    let response = AppliedAgentModelConfigurationResponse {
        configuration_id: body.configuration_id.trim().to_string(),
        profile_id: application.profile.profile_id,
        engine_id: engine_id.to_string(),
        agent_id: agent_id.to_string(),
        provider_scope: application.provider_scope,
        vendor_code: kernel_request.vendor_code,
        base_url: kernel_request.base_url,
        default_model_id: kernel_request.default_model_id,
        supported_model_ids: kernel_request.supported_model_ids,
        supported_provider_ids,
        input_context_tokens: kernel_request.input_context_tokens,
        output_context_tokens: kernel_request.output_context_tokens,
        tool_call_rounds: kernel_request.tool_call_rounds,
        supports_multimodal: kernel_request.supports_multimodal,
        api_key_configured: true,
    };
    Ok(response)
}

async fn app_apply_model_selection(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<ApplyAgentModelSelectionRequest>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AppliedAgentModelSelectionResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        // Built-in model selection without a saved configuration switches the
        // model directly on the provider-default credential binding. Platform
        // catalog models are validated by the client model selector and are not
        // part of the agent-engine catalog, so no catalog membership check is
        // applied here; malformed identifiers are rejected below.
        let item = apply_agent_model_selection(state, scope, web_ctx.request_id.0.as_str(), body)?;
        Ok(ResourceData { item })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

fn apply_agent_model_selection(
    state: AgentHttpState,
    scope: RequestScope,
    request_id: &str,
    body: ApplyAgentModelSelectionRequest,
) -> ApiResult<AppliedAgentModelSelectionResponse> {
    let engine_id = body.engine_id.trim();
    let agent_id = resolve_model_configuration_agent_id(body.agent_id.as_deref(), engine_id)?;
    validate_model_configuration_identifier("modelId", &body.model_id, 256)?;
    let configuration_id = body
        .configuration_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(configuration_id) = &configuration_id {
        validate_model_configuration_identifier("configurationId", configuration_id, 160)?;
    }

    let profile_id = configuration_id
        .as_deref()
        .map(|configuration_id| {
            model_configuration_profile_id(&scope, &agent_id, engine_id, configuration_id)
        })
        .unwrap_or_else(|| {
            model_configuration_profile_id(&scope, &agent_id, engine_id, "model.selection.builtin")
        });
    let profile_scope = profile_scope_from_request(&scope)?;
    let current_profile = if configuration_id.is_some() {
        let profile = state
            .model_configuration_runtime
            .configurations
            .lock()
            .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?
            .find_profile_in_scope(agent_id.as_str(), &profile_id, &profile_scope)
            .map_err(|_| ApiProblem::internal("model configuration could not be loaded"))?
            .ok_or_else(|| {
                ApiProblem::validation(
                    "configurationId does not identify a saved model configuration",
                )
            })?;
        Some(profile)
    } else {
        None
    };

    let mut kernel_request = sdkwork_agents_runtime_facade::AgentModelSelectionRequest::new(
        request_id,
        agent_id.as_str(),
        &profile_id,
        body.model_id.trim(),
    );
    if let Some(profile) = current_profile {
        kernel_request = kernel_request
            .with_current_profile(profile)
            .with_supported_model_enforcement();
    }
    let application = sdkwork_agents_runtime_facade::apply_agent_engine_model_selection(
        engine_id,
        &kernel_request,
    )
    .map_err(|error| ApiProblem::validation(error.to_string()))?;
    state
        .model_configuration_runtime
        .configurations
        .lock()
        .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?
        .save_profile_in_scope(application.profile.clone(), &profile_scope)
        .map_err(|_| ApiProblem::internal("provider model selection could not be stored"))?;

    // A rig model selection rewrites the default model inside the profile;
    // refresh the agent-scoped host so the custom provider picks it up.
    if engine_id == "rig" {
        let refresh_result = crate::runtime_facade_bridge::refresh_rig_agent_engine_for(
            parse_scope_tenant_id(&scope),
            &agent_id,
            &application.profile.configuration,
            std::sync::Arc::new(
                crate::runtime_facade_bridge::ModelConfigurationRuntimeHostProvider::new(
                    state.model_configuration_runtime.clone(),
                ),
            ),
        );
        if let Err(error) = refresh_result {
            tracing::warn!(
                engine_id,
                error = %error,
                "rig agent engine refresh after model selection failed"
            );
        }
    }

    Ok(AppliedAgentModelSelectionResponse {
        configuration_id,
        profile_id: application.profile.profile_id,
        engine_id: engine_id.to_string(),
        agent_id: agent_id.to_string(),
        provider_scope: application.provider_scope,
        model_id: kernel_request.model_id,
    })
}

/// Resolves the configuration subject: the optional client `agentId` or the
/// canonical engine agent id. The engine id must always name a supported
/// Agent provider.
fn resolve_model_configuration_agent_id(
    agent_id: Option<&str>,
    engine_id: &str,
) -> ApiResult<String> {
    let canonical = sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_id)
        .ok_or_else(|| ApiProblem::validation("engineId is not a supported Agent provider"))?;
    match agent_id.map(str::trim).filter(|value| !value.is_empty()) {
        Some(agent_id) => {
            if agent_id.len() > 128 {
                return Err(ApiProblem::validation(
                    "agentId exceeds the maximum length",
                ));
            }
            Ok(agent_id.to_string())
        }
        None => Ok(canonical.to_string()),
    }
}

/// Numeric tenant id from the request scope (string form); a malformed value
/// falls back to `0` so host scoping degrades to the shared default host.
fn parse_scope_tenant_id(scope: &RequestScope) -> u64 {
    scope.tenant_id.parse::<u64>().unwrap_or(0)
}

fn model_configuration_profile_id(
    scope: &RequestScope,
    agent_id: &str,
    engine_id: &str,
    configuration_id: &str,
) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for value in [
        scope.tenant_id.as_str(),
        scope.organization_id.as_str(),
        scope.owner_user_id.as_str(),
        agent_id,
        engine_id,
        configuration_id,
    ] {
        for byte in value
            .as_bytes()
            .iter()
            .copied()
            .chain(std::iter::once(0xff))
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("{ID_PREFIX_PROFILE}model_configuration.{hash:016x}")
}

// ---------------------------------------------------------------------------
// Model configuration read-back and lifecycle endpoints.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListModelConfigurationsQuery {
    engine_id: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelConfigurationSummaryView {
    profile_id: String,
    engine_id: String,
    agent_id: String,
    provider_scope: String,
    configuration_version: String,
    status: String,
    base_url: String,
    default_model_id: String,
    supported_model_ids: Vec<String>,
    api_key_configured: bool,
}

impl ModelConfigurationSummaryView {
    fn from_profile(profile: &AgentConfigurationProfile, engine_id: &str) -> ApiResult<Self> {
        let provider_scope = sdkwork_agents_runtime_facade::agent_engine_provider_scope(engine_id)
            .ok_or_else(|| ApiProblem::validation("engineId is not a supported Agent provider"))?;
        let mapping = AgentModelConfigurationFieldMapping::namespaced(provider_scope);
        let config = &profile.configuration;
        Ok(Self {
            profile_id: profile.profile_id.clone(),
            engine_id: engine_id.to_string(),
            agent_id: profile.agent_id.clone(),
            provider_scope: provider_scope.to_string(),
            configuration_version: profile.configuration_version.clone(),
            status: profile.status.as_str().to_string(),
            base_url: config_string(config, &mapping.base_url_key).unwrap_or_default(),
            default_model_id: config_string(config, &mapping.default_model_key).unwrap_or_default(),
            supported_model_ids: config_string_list(config, &mapping.supported_models_key),
            api_key_configured: profile.requires_secret(&mapping.api_key_key),
        })
    }
}

fn config_string(config: &sdkwork_agent_kernel::AgentConfiguration, key: &str) -> Option<String> {
    match config.value(key) {
        Some(AgentConfigValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn config_string_list(config: &sdkwork_agent_kernel::AgentConfiguration, key: &str) -> Vec<String> {
    match config.value(key) {
        Some(AgentConfigValue::StringList(values)) => values.clone(),
        _ => Vec::new(),
    }
}

async fn app_list_model_configurations(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<ListModelConfigurationsQuery>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<ModelConfigurationSummaryView>> = async {
        let query = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let profile_scope = profile_scope_from_request(&scope)?;
        let engine_ids = match &query.engine_id {
            Some(engine_id) => vec![engine_id.clone()],
            None => sdkwork_agents_runtime_facade::bootstrappable_engine_keys()
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        };
        let mut items = Vec::new();
        let configurations = state
            .model_configuration_runtime
            .configurations
            .lock()
            .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?;
        for engine_id in engine_ids {
            let Some(agent_id) = sdkwork_agents_runtime_facade::agent_engine_agent_id(&engine_id)
            else {
                return Err(ApiProblem::validation(
                    "engineId is not a supported Agent provider",
                ));
            };
            let profiles = configurations
                .list_profiles_in_scope(agent_id, &profile_scope)
                .map_err(|_| ApiProblem::internal("model configuration could not be loaded"))?;
            for profile in profiles {
                items.push(ModelConfigurationSummaryView::from_profile(
                    &profile, &engine_id,
                )?);
            }
        }
        let total_items = items.len();
        let page_info = PageInfo {
            mode: PageMode::Offset,
            page: Some(1),
            page_size: Some(total_items as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(if total_items == 0 { 0 } else { 1 }),
            next_cursor: None,
            has_more: Some(false),
        };
        Ok(PageData { items, page_info })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_model_configuration(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    Path(path): Path<ModelConfigurationPath>,
) -> Response {
    let result: ApiResult<ResourceData<ModelConfigurationSummaryView>> = async {
        let scope = RequestScope::from_context(context);
        let profile = load_model_configuration_profile(&state, &scope, &path.engine_id, &path.profile_id)?;
        Ok(ResourceData {
            item: ModelConfigurationSummaryView::from_profile(&profile, &path.engine_id)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelConfigurationPath {
    engine_id: String,
    profile_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelConfigurationStatusView {
    profile_id: String,
    engine_id: String,
    agent_id: String,
    provider_scope: String,
    status: String,
    /// Provider-level read-back state of the native config surface.
    materialization: String,
    /// Derived state after comparing the native config surface with the
    /// stored profile: `materialized`, `diverged`, `not_materialized`,
    /// `unsupported`.
    derived_state: String,
    expected_base_url: Option<String>,
    expected_default_model: Option<String>,
    effective_base_url: Option<String>,
    effective_default_model: Option<String>,
    credential_configured: bool,
    issues: Vec<String>,
}

async fn app_get_model_configuration_status(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    Path(path): Path<ModelConfigurationPath>,
) -> Response {
    let result: ApiResult<ResourceData<ModelConfigurationStatusView>> = async {
        let scope = RequestScope::from_context(context);
        let profile = load_model_configuration_profile(&state, &scope, &path.engine_id, &path.profile_id)?;
        let provider_scope =
            sdkwork_agents_runtime_facade::agent_engine_provider_scope(&path.engine_id)
                .ok_or_else(|| {
                    ApiProblem::validation("engineId is not a supported Agent provider")
                })?;
        let mapping = AgentModelConfigurationFieldMapping::namespaced(provider_scope);
        let expected_base_url = config_string(&profile.configuration, &mapping.base_url_key);
        let expected_default_model =
            config_string(&profile.configuration, &mapping.default_model_key);
        let read_back = sdkwork_agents_runtime_facade::read_agent_engine_model_configuration(
            &path.engine_id,
            &profile.agent_id,
            &path.profile_id,
        )
        .map_err(|error| ApiProblem::validation(error.to_string()))?;

        // The provider reports its native surface; the divergence is derived
        // here against the stored profile. The default model is only compared
        // when the provider surfaces one (providers passing the model per
        // turn report none, which is not a divergence).
        let derived_state = match read_back.materialization {
            sdkwork_agent_kernel::ProviderModelMaterializationState::Unsupported => "unsupported",
            sdkwork_agent_kernel::ProviderModelMaterializationState::NotMaterialized => {
                "not_materialized"
            }
            sdkwork_agent_kernel::ProviderModelMaterializationState::Materialized
            | sdkwork_agent_kernel::ProviderModelMaterializationState::Diverged => {
                let base_url_matches = read_back
                    .effective_base_url
                    .as_deref()
                    .is_some_and(|effective| Some(effective) == expected_base_url.as_deref());
                let model_matches =
                    match (&read_back.effective_default_model, &expected_default_model) {
                        (Some(effective), Some(expected)) => effective == expected,
                        // Per-turn model providers cannot be verified on read-back.
                        (None, _) => true,
                        (Some(_), None) => false,
                    };
                if base_url_matches && model_matches {
                    "materialized"
                } else {
                    "diverged"
                }
            }
        };

        Ok(ResourceData {
            item: ModelConfigurationStatusView {
                profile_id: profile.profile_id.clone(),
                engine_id: path.engine_id.clone(),
                agent_id: profile.agent_id.clone(),
                provider_scope: provider_scope.to_string(),
                status: profile.status.as_str().to_string(),
                materialization: read_back.materialization.as_str().to_string(),
                derived_state: derived_state.to_string(),
                expected_base_url,
                expected_default_model,
                effective_base_url: read_back.effective_base_url,
                effective_default_model: read_back.effective_default_model,
                credential_configured: read_back.credential_configured,
                issues: read_back.issues,
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

#[derive(Deserialize)]
struct AgentEngineConfigFilePath {
    engine_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentEngineConfigFileView {
    engine_id: String,
    config_file_path: String,
    format: String,
    content: String,
    exists: bool,
}

async fn app_get_agent_engine_config_file(
    State(_state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    Path(path): Path<AgentEngineConfigFilePath>,
) -> Response {
    let result: ApiResult<ResourceData<AgentEngineConfigFileView>> = async {
        let _scope = RequestScope::from_context(context);
        let file = sdkwork_agents_runtime_facade::read_agent_engine_config_file(&path.engine_id)
            .map_err(|error| ApiProblem::validation(error.to_string()))?;
        Ok(ResourceData {
            item: AgentEngineConfigFileView {
                engine_id: file.engine_key,
                config_file_path: file.config_file_path,
                format: file.format,
                content: file.content,
                exists: file.exists,
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_archive_model_configuration(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    Path(path): Path<ModelConfigurationPath>,
) -> Response {
    let result: ApiResult<ResourceData<ModelConfigurationSummaryView>> = async {
        let scope = RequestScope::from_context(context);
        let profile_scope = profile_scope_from_request(&scope)?;
        let profile = load_model_configuration_profile(&state, &scope, &path.engine_id, &path.profile_id)?;

        // Only revert the CLI-native config when it actually carries the
        // SDKWork-managed materialization; never touch a user-owned surface.
        let read_back = sdkwork_agents_runtime_facade::read_agent_engine_model_configuration(
            &path.engine_id,
            &profile.agent_id,
            &path.profile_id,
        )
        .map_err(|error| ApiProblem::validation(error.to_string()))?;
        if read_back.materialization
            == sdkwork_agent_kernel::ProviderModelMaterializationState::Materialized
            || read_back.materialization
                == sdkwork_agent_kernel::ProviderModelMaterializationState::Diverged
        {
            sdkwork_agents_runtime_facade::dematerialize_agent_engine_model_configuration(
                &path.engine_id,
                &profile.agent_id,
                &path.profile_id,
            )
            .map_err(|error| ApiProblem::validation(error.to_string()))?;
        }

        let request_id = format!("{ID_PREFIX_REQUEST}{}", sdkwork_utils_rust::uuid());
        validate_standard_id(&request_id, "requestId", Some(ID_PREFIX_REQUEST))
            .map_err(ApiProblem::from_kernel_error)?;
        let archived = state
            .model_configuration_runtime
            .configurations
            .lock()
            .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?
            .archive_profile_in_scope(
                &sdkwork_agent_kernel::AgentProfileArchiveRequest::new(
                    &request_id,
                    &profile.agent_id,
                    &path.profile_id,
                ),
                &profile_scope,
            )
            .map_err(|_| ApiProblem::internal("model configuration could not be archived"))?;
        Ok(ResourceData {
            item: ModelConfigurationSummaryView::from_profile(&archived.profile, &path.engine_id)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MigrateModelConfigurationRequest {
    engine_id: String,
    profile_id: String,
    from_configuration_version: String,
    to_configuration_version: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigratedModelConfigurationResponse {
    profile_id: String,
    engine_id: String,
    agent_id: String,
    configuration_version: String,
    status: String,
    migration_plan_id: String,
}

async fn app_migrate_model_configuration(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<MigrateModelConfigurationRequest>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<MigratedModelConfigurationResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let profile_scope = profile_scope_from_request(&scope)?;
        let profile = load_model_configuration_profile(&state, &scope, &body.engine_id, &body.profile_id)?;
        if profile.configuration_version != body.from_configuration_version {
            return Err(ApiProblem::validation(
                "fromConfigurationVersion does not match the stored profile version",
            ));
        }
        let request_id = format!("{ID_PREFIX_REQUEST}{}", sdkwork_utils_rust::uuid());
        validate_standard_id(&request_id, "requestId", Some(ID_PREFIX_REQUEST))
            .map_err(ApiProblem::from_kernel_error)?;
        let plan_request = AgentConfigurationUpgradeRequest::new(
            &request_id,
            &profile.agent_id,
            &body.profile_id,
            &body.from_configuration_version,
            &body.to_configuration_version,
        )
        .with_current_profile(profile.clone());
        let plan = sdkwork_agents_runtime_facade::plan_agent_engine_configuration_upgrade(
            &body.engine_id,
            &plan_request,
        )
        .map_err(|error| ApiProblem::validation(error.to_string()))?;
        let record = state
            .model_configuration_runtime
            .configurations
            .lock()
            .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?
            .migrate_profile_in_scope(&plan, profile, &profile_scope)
            .map_err(|_| ApiProblem::internal("model configuration could not be migrated"))?;
        Ok(ResourceData {
            item: MigratedModelConfigurationResponse {
                profile_id: record.profile.profile_id.clone(),
                engine_id: body.engine_id,
                agent_id: record.profile.agent_id.clone(),
                configuration_version: record.profile.configuration_version.clone(),
                status: record.profile.status.as_str().to_string(),
                migration_plan_id: record
                    .migration_plan_id
                    .clone()
                    .unwrap_or_else(|| plan.plan_id.clone()),
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

fn load_model_configuration_profile(
    state: &AgentHttpState,
    scope: &RequestScope,
    engine_id: &str,
    profile_id: &str,
) -> ApiResult<AgentConfigurationProfile> {
    let agent_id = sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_id)
        .ok_or_else(|| ApiProblem::validation("engineId is not a supported Agent provider"))?;
    let profile_scope = profile_scope_from_request(scope)?;
    let profile = state
        .model_configuration_runtime
        .configurations
        .lock()
        .map_err(|_| ApiProblem::internal("model configuration store is unavailable"))?
        .find_profile_in_scope(agent_id, profile_id, &profile_scope)
        .map_err(|_| ApiProblem::internal("model configuration could not be loaded"))?
        .ok_or_else(|| ApiProblem::not_found("model configuration profile not found"))?;
    Ok(profile)
}

/// Derives the owner scope for model configuration store access from the
/// trusted request context. Fails closed when the scoped identifiers cannot
/// be parsed, so no request can ever address another tenant's rows.
fn profile_scope_from_request(scope: &RequestScope) -> ApiResult<ProfileScope> {
    ProfileScope::try_parse(
        &scope.tenant_id,
        &scope.organization_id,
        &scope.owner_user_id,
    )
    .map_err(ApiProblem::from_kernel_error)
}

fn normalize_supported_model_provider_ids(
    provider_ids: Vec<String>,
    selected_engine_id: &str,
) -> ApiResult<Vec<String>> {
    let supported = sdkwork_agents_runtime_facade::bootstrappable_engine_keys();
    let requested = if provider_ids.is_empty() {
        supported.iter().map(|value| (*value).to_string()).collect()
    } else {
        provider_ids
    };
    let mut normalized = Vec::new();
    for provider_id in requested {
        let provider_id = provider_id.trim();
        if !supported.contains(&provider_id) {
            return Err(ApiProblem::validation(
                "supportedProviderIds contains an unsupported Agent provider",
            ));
        }
        if !normalized.iter().any(|value| value == provider_id) {
            normalized.push(provider_id.to_string());
        }
    }
    if !normalized.iter().any(|value| value == selected_engine_id) {
        return Err(ApiProblem::validation(
            "engineId must be included in supportedProviderIds",
        ));
    }
    Ok(normalized)
}

fn validate_model_configuration_identifier(
    field: &str,
    value: &str,
    max_length: usize,
) -> ApiResult<()> {
    let value = value.trim();
    if value.is_empty() || value.len() > max_length {
        return Err(ApiProblem::validation(format!(
            "{field} must contain between 1 and {max_length} characters"
        )));
    }
    if !value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | '/')
    }) {
        return Err(ApiProblem::validation(format!(
            "{field} contains unsupported characters"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ListCompositionSlotsQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ListMcpServersQueryParams {
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListAgentsQueryParams {
    scope: Option<String>,
    include_deleted: Option<bool>,
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListAgentsQueryParams {
    scope: Option<String>,
    include_deleted: Option<bool>,
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppListProjectsQueryParams {
    #[serde(rename = "workspaceId", alias = "workspace_id")]
    workspace_id: Option<String>,
    q: Option<String>,
    name_exact: Option<String>,
    status: Option<String>,
    include_deleted: Option<bool>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppListWorkspacesQueryParams {
    status: Option<String>,
    include_deleted: Option<bool>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppEnsureDefaultWorkspaceBody {
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCreateWorkspaceBody {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateWorkspaceBody {
    expected_version: Option<String>,
    name: Option<String>,
    description: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppWorkspaceMutationBody {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppDeleteWorkspaceQuery {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppCreateProjectBody {
    project_id: Option<String>,
    workspace_id: Option<String>,
    name: String,
    description: Option<String>,
    visibility: Option<String>,
    drive_access_mode: Option<String>,
    default_agent_id: Option<String>,
    default_model_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppImportProjectBody {
    workspace_id: String,
    project_id: Option<String>,
    name: String,
    description: Option<String>,
    source_kind: String,
    source_ref: String,
    drive_space_id: String,
    drive_root_entry_id: String,
    drive_logical_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppUpdateProjectBody {
    expected_version: Option<String>,
    name: Option<String>,
    description: Option<Option<String>>,
    visibility: Option<String>,
    drive_access_mode: Option<String>,
    default_agent_id: Option<Option<String>>,
    default_model_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppProjectMutationBody {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppListProjectCompositionSlotsQuery {
    slot_kind: Option<String>,
    enabled: Option<bool>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCreateProjectCompositionSlotBody {
    slot_id: String,
    slot_kind: String,
    target_module: String,
    target_ref: String,
    target_version_ref: Option<String>,
    priority: Option<i32>,
    enabled: Option<bool>,
    policy_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateProjectCompositionSlotBody {
    expected_version: Option<String>,
    slot_kind: Option<String>,
    target_module: Option<String>,
    target_ref: Option<String>,
    target_version_ref: Option<String>,
    clear_target_version_ref: Option<bool>,
    priority: Option<i32>,
    enabled: Option<bool>,
    policy_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppDeleteProjectCompositionSlotQuery {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateSessionBody {
    expected_version: Option<String>,
    title: Option<String>,
    project_id: Option<String>,
    clear_project: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateSessionUserStateBody {
    expected_version: Option<String>,
    pinned: Option<bool>,
    hidden: Option<bool>,
    mark_opened: Option<bool>,
    last_read_item_sequence: Option<String>,
    custom_title: Option<String>,
    clear_custom_title: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateItemFeedbackBody {
    expected_version: Option<String>,
    rating: Option<String>,
    clear_feedback: Option<bool>,
    reason_code: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCancelTurnBody {
    expected_version: String,
    requested_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProjectRecordResponse {
    id: String,
    project_id: String,
    workspace_id: String,
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    name: String,
    description: Option<String>,
    visibility: String,
    status: String,
    drive_access_mode: String,
    default_agent_id: Option<String>,
    default_model_id: Option<String>,
    import_source_kind: Option<String>,
    import_source_ref: Option<String>,
    drive_space_id: Option<String>,
    drive_root_entry_id: Option<String>,
    drive_logical_path: Option<String>,
    version: String,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentWorkspaceRecordResponse {
    id: String,
    workspace_id: String,
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    name: String,
    description: Option<String>,
    is_default: bool,
    status: String,
    version: String,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProjectCompositionSlotRecordResponse {
    id: String,
    tenant_id: String,
    organization_id: String,
    project_id: String,
    slot_id: String,
    slot_kind: String,
    target_module: String,
    target_ref: String,
    target_version_ref: Option<String>,
    priority: i32,
    enabled: bool,
    policy_json: String,
    created_by: String,
    updated_by: String,
    version: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppCreateTurnQueryParams {
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    event_protocol: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct BackendCreateTurnQueryParams {
    #[serde(default)]
    pub(crate) stream: Option<bool>,
    #[serde(default)]
    pub(crate) event_protocol: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppTurnsQueryParams {
    cursor: Option<String>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TenantAgentPathParams {
    #[serde(rename = "agentId")]
    pub(crate) agent_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TenantAgentBindingPathParams {
    #[serde(rename = "agentId")]
    agent_id: String,
    #[serde(rename = "bindingId")]
    binding_id: String,
}
#[derive(Debug, Clone, Deserialize)]
struct TenantAgentSlotPathParams {
    #[serde(rename = "agentId")]
    agent_id: String,
    #[serde(rename = "slotId")]
    slot_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListSessionsQueryParams {
    pub(crate) status: Option<String>,
    pub(crate) include_archived: Option<bool>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListProjectSessionsQueryParams {
    status: Option<String>,
    include_archived: Option<bool>,
    cursor: Option<String>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSessionSynchronizationResultDto {
    /// `completed` (result served from the refresh cache), `accepted`
    /// (synchronization enqueued on a background worker), or `pending`
    /// (a synchronization is already running for this project).
    status: String,
    failed_session_count: String,
    issues: Vec<ProjectSessionSynchronizationIssueDto>,
    project_id: String,
    skipped_session_count: String,
    synchronized_session_count: String,
}

impl ProjectSessionSynchronizationResultDto {
    fn from_result(
        result: &crate::provider_session_sync::ProviderSessionSynchronizationResult,
        project_id: String,
        status: &str,
    ) -> Self {
        Self {
            status: status.to_string(),
            failed_session_count: result.failed_session_count.to_string(),
            issues: result
                .issues
                .iter()
                .map(|issue| ProjectSessionSynchronizationIssueDto {
                    code: issue.code.to_string(),
                    count: issue.count.to_string(),
                    disposition: issue.disposition.as_str().to_string(),
                })
                .collect(),
            project_id,
            skipped_session_count: result.skipped_session_count.to_string(),
            synchronized_session_count: result.synchronized_session_count.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSessionSynchronizationIssueDto {
    code: String,
    count: String,
    disposition: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListSessionsQueryParams {
    pub(crate) project_id: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) include_archived: Option<bool>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListSessionActivitySummariesQueryParams {
    cursor: Option<String>,
    page_size: Option<usize>,
    workspace_id: Option<String>,
    project_id: Option<String>,
    agent_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListSessionUserStatesQueryParams {
    pinned_only: Option<bool>,
    include_hidden: Option<bool>,
    session_ids: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListItemFeedbackQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListTasksQueryParams {
    pub(crate) status: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListTasksQueryParams {
    pub(crate) status: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListTaskRunsQueryParams {
    status: Option<String>,
    trigger_kind: Option<String>,
    cursor: Option<String>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListTaskRunAttemptsQueryParams {
    cursor: Option<String>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListItemsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) sort: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListItemsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) sort: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListInteractionsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListInteractionsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentCompositionSlotRecordResponse {
    id: String,
    tenant_id: String,
    organization_id: String,
    agent_id: String,
    slot_id: String,
    slot_kind: String,
    target_module: String,
    target_ref: String,
    target_version_ref: Option<String>,
    priority: i32,
    enabled: bool,
    policy_json: String,
    status: String,
    version: String,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditEventsQueryParams {
    cursor: Option<String>,
    page_size: Option<usize>,
    action: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateAgentBody {
    agent_id: String,
    code: String,
    display_name: String,
    description: Option<String>,
    manifest: Value,
    default_code_task_intent: Option<CodeTaskIntentBody>,
    management_profile: Option<AgentManagementProfileBody>,
    implementation_provider_id: Option<String>,
    implementation_kind: Option<String>,
    implementation_type: Option<String>,
    visibility: String,
    tags: Option<Vec<String>>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentProviderBindingBody {
    binding_id: String,
    provider_id: String,
    implementation_kind: String,
    configuration_profile_id: String,
    capabilities: Option<Vec<String>>,
    make_default: Option<bool>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivateProviderBindingBody {
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentPreviewResponseBody {
    execution_id: String,
    content: String,
    debug_mode: Option<bool>,
    model: Option<String>,
    temperature: Option<f32>,
    input_payload: Option<Value>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentPromptOptimizationBody {
    execution_id: String,
    prompt: String,
    input_payload: Option<Value>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateAgentBody {
    display_name: Option<String>,
    description: Option<String>,
    manifest: Option<Value>,
    visibility: Option<String>,
    tags: Option<Vec<String>>,
    default_code_task_intent: Option<CodeTaskIntentBody>,
    management_profile: Option<AgentManagementProfileBody>,
    implementation_provider_id: Option<Option<String>>,
    implementation_kind: Option<Option<String>>,
    implementation_type: Option<String>,
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateAgentStatusBody {
    target_status: String,
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestoreAgentBody {
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProviderBindingRecordResponse {
    tenant_id: String,
    agent_id: String,
    binding_id: String,
    provider_id: String,
    implementation_kind: String,
    configuration_profile_id: String,
    capabilities: Vec<String>,
    active: bool,
    version: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentRuntimeExecutionRecordResponse {
    tenant_id: String,
    agent_id: String,
    execution_id: String,
    operation: String,
    status: String,
    input_payload: Value,
    output_payload: Value,
    requested_at: String,
    completed_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentRecordResponse {
    id: String,
    agent_id: String,
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    code: String,
    display_name: String,
    description: Option<String>,
    manifest: Value,
    default_code_task_intent: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    management_profile: Option<AgentManagementProfileResponse>,
    implementation_provider_id: Option<String>,
    implementation_kind: Option<String>,
    implementation_type: String,
    status: String,
    visibility: String,
    tags: Vec<String>,
    version: String,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentManagementProfileResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    json_mode: Option<bool>,
    knowledge_base_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    skill_ids: Vec<String>,
    suggested_prompts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    tool_ids: Vec<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<String>,
    voice_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    welcome_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentAuditEventResponse {
    event_id: String,
    event_type: String,
    severity: String,
    payload: String,
    occurred_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeTaskIntentBody {
    prompt: String,
    context_paths: Option<Vec<String>>,
    constraints: Option<Vec<String>>,
}

impl From<CodeTaskIntentBody> for CodeTaskIntent {
    fn from(value: CodeTaskIntentBody) -> Self {
        Self {
            prompt: value.prompt,
            context_paths: value.context_paths.unwrap_or_default(),
            constraints: value.constraints.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentManagementProfileBody {
    author: Option<String>,
    avatar: Option<String>,
    category_id: Option<String>,
    color: Option<String>,
    debug_mode: Option<bool>,
    icon_name: Option<String>,
    json_mode: Option<bool>,
    knowledge_base_ids: Option<Vec<String>>,
    memory_enabled: Option<bool>,
    model: Option<String>,
    skill_ids: Option<Vec<String>>,
    suggested_prompts: Option<Vec<String>>,
    system_prompt: Option<String>,
    temperature: Option<f64>,
    tool_ids: Option<Vec<String>>,
    #[serde(rename = "type")]
    agent_type: Option<String>,
    users: Option<String>,
    voice_ids: Option<Vec<String>>,
    welcome_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCreateTurnBody {
    turn_id: Option<String>,
    content: String,
    content_type: Option<String>,
    turn_mode: String,
    system_prompt: Option<String>,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    access_mode_id: Option<String>,
    idempotency_key: String,
    payload_hash: String,
    client_request_id: Option<String>,
    #[serde(default)]
    drive_refs: Vec<AgentItemDriveRefBody>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCreateTurnInputQueueEntryBody {
    queue_entry_id: Option<String>,
    content: String,
    #[serde(default)]
    display_text: String,
    content_type: Option<String>,
    #[serde(default)]
    attachment_names: Vec<String>,
    #[serde(default)]
    drive_refs: Vec<AgentItemDriveRefBody>,
    turn_mode: String,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    access_mode_id: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateTurnInputQueueEntryBody {
    content: String,
    #[serde(default)]
    display_text: String,
    content_type: Option<String>,
    #[serde(default)]
    attachment_names: Vec<String>,
    #[serde(default)]
    drive_refs: Vec<AgentItemDriveRefBody>,
    turn_mode: String,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    access_mode_id: Option<String>,
    expected_version: String,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppRemoveTurnInputQueueEntryQueryParams {
    expected_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppReorderTurnInputQueueEntriesBody {
    ordered_entries: Vec<AppTurnInputQueueReorderEntryBody>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppTurnInputQueueReorderEntryBody {
    queue_entry_id: String,
    expected_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppClaimNextTurnInputQueueEntryBody {
    claim_owner: String,
    #[serde(default = "default_turn_input_queue_lease_seconds")]
    lease_seconds: u32,
    requested_at: String,
}

const fn default_turn_input_queue_lease_seconds() -> u32 {
    120
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppFailTurnInputQueueEntryBody {
    expected_version: String,
    fencing_token: String,
    claim_token: String,
    error_code: String,
    error_detail: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppRetryTurnInputQueueEntryBody {
    expected_version: String,
    requested_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentTurnInputQueueDriveRefResponse {
    resource_role: String,
    drive_space_id: String,
    drive_node_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentTurnInputQueueEntryResponse {
    queue_entry_id: String,
    session_id: String,
    agent_id: String,
    content: String,
    display_text: String,
    content_type: String,
    attachment_names: Vec<String>,
    drive_refs: Vec<AgentTurnInputQueueDriveRefResponse>,
    turn_mode: String,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    access_mode_id: Option<String>,
    idempotency_key: String,
    payload_hash: String,
    client_request_id: String,
    position: String,
    status: String,
    claim_owner: Option<String>,
    claim_expires_at: Option<String>,
    fencing_token: String,
    error_code: Option<String>,
    error_detail: Option<String>,
    version: String,
    created_at: String,
    updated_at: String,
    claimed_at: Option<String>,
    failed_at: Option<String>,
}

impl AgentTurnInputQueueEntryResponse {
    fn from_record(record: &AgentTurnInputQueueEntry) -> Self {
        Self {
            queue_entry_id: record.queue_entry_id.clone(),
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            content: record.content.clone(),
            display_text: record.display_text.clone(),
            content_type: record.content_type.clone(),
            attachment_names: record.attachment_names.clone(),
            drive_refs: record
                .drive_refs
                .iter()
                .map(|value| AgentTurnInputQueueDriveRefResponse {
                    resource_role: value.resource_role.as_str().to_string(),
                    drive_space_id: value.drive_space_id.clone(),
                    drive_node_id: value.drive_node_id.clone(),
                })
                .collect(),
            turn_mode: record.turn_mode.as_str().to_string(),
            runtime_binding_id: record.runtime_binding_id.clone(),
            requested_model_id: record.requested_model_id.clone(),
            access_mode_id: record.access_mode_id.clone(),
            idempotency_key: record.idempotency_key.clone(),
            payload_hash: record.payload_hash.clone(),
            client_request_id: record.client_request_id.clone(),
            position: record.position.to_string(),
            status: record.status.as_str().to_string(),
            claim_owner: record.claim_owner.clone(),
            claim_expires_at: record.claim_expires_at.clone(),
            fencing_token: record.fencing_token.to_string(),
            error_code: record.error_code.clone(),
            error_detail: record.error_detail.clone(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            claimed_at: record.claimed_at.clone(),
            failed_at: record.failed_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClaimNextTurnInputQueueEntryResponse {
    outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    entry: Option<AgentTurnInputQueueEntryResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    claim_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClearTurnInputQueueEntriesResponse {
    cleared_count: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReorderTurnInputQueueEntriesResponse {
    items: Vec<AgentTurnInputQueueEntryResponse>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateTurnBody {
    turn_id: Option<String>,
    content: String,
    content_type: Option<String>,
    turn_mode: String,
    system_prompt: Option<String>,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    idempotency_key: String,
    payload_hash: String,
    client_request_id: Option<String>,
    #[serde(default)]
    drive_refs: Vec<AgentItemDriveRefBody>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AgentItemDriveRefBody {
    resource_role: String,
    drive_space_id: String,
    drive_node_id: String,
}

impl AgentItemDriveRefBody {
    fn into_input(self) -> Result<AgentItemDriveRefInput, ApiProblem> {
        let resource_role = match self.resource_role.as_str() {
            "attachment" => AgentItemResourceRole::Attachment,
            "image" => AgentItemResourceRole::Image,
            "audio" => AgentItemResourceRole::Audio,
            "artifact" => AgentItemResourceRole::Artifact,
            _ => return Err(ApiProblem::validation("invalid driveRefs.resourceRole")),
        };
        Ok(AgentItemDriveRefInput {
            resource_role,
            drive_space_id: self.drive_space_id,
            drive_node_id: self.drive_node_id,
        })
    }

    fn into_queue_ref(self) -> Result<AgentTurnInputQueueDriveRef, ApiProblem> {
        let resource_role = match self.resource_role.as_str() {
            "attachment" => AgentItemResourceRole::Attachment,
            "image" => AgentItemResourceRole::Image,
            "audio" => AgentItemResourceRole::Audio,
            _ => return Err(ApiProblem::validation("invalid driveRefs.resourceRole")),
        };
        Ok(AgentTurnInputQueueDriveRef {
            resource_role,
            drive_space_id: self.drive_space_id,
            drive_node_id: self.drive_node_id,
        })
    }
}

impl From<AgentManagementProfileBody> for AgentManagementProfileDto {
    fn from(value: AgentManagementProfileBody) -> Self {
        Self {
            author: value.author,
            avatar: value.avatar,
            category_id: value.category_id,
            color: value.color,
            debug_mode: value.debug_mode,
            icon_name: value.icon_name,
            json_mode: value.json_mode,
            knowledge_base_ids: value.knowledge_base_ids.unwrap_or_default(),
            memory_enabled: value.memory_enabled,
            model: value.model,
            skill_ids: value.skill_ids.unwrap_or_default(),
            suggested_prompts: value.suggested_prompts.unwrap_or_default(),
            system_prompt: value.system_prompt,
            temperature: value.temperature,
            tool_ids: value.tool_ids.unwrap_or_default(),
            agent_type: value.agent_type,
            users: value.users,
            voice_ids: value.voice_ids.unwrap_or_default(),
            welcome_message: value.welcome_message,
        }
    }
}

impl AgentManagementProfileBody {
    fn into_validated_dto(self) -> Result<AgentManagementProfileDto, ApiProblem> {
        validate_agent_management_profile_body(&self)?;
        Ok(self.into())
    }
}

fn validate_agent_management_profile_body(
    profile: &AgentManagementProfileBody,
) -> Result<(), ApiProblem> {
    validate_optional_profile_string(profile.author.as_deref(), "managementProfile.author", 128)?;
    validate_optional_profile_string(profile.avatar.as_deref(), "managementProfile.avatar", 512)?;
    validate_optional_profile_string(
        profile.category_id.as_deref(),
        "managementProfile.categoryId",
        64,
    )?;
    validate_optional_profile_string(profile.color.as_deref(), "managementProfile.color", 32)?;
    validate_optional_profile_string(
        profile.icon_name.as_deref(),
        "managementProfile.iconName",
        64,
    )?;
    if let Some(model) = profile.model.as_deref() {
        validate_standard_id(model, "managementProfile.model", Some(ID_PREFIX_MODEL))
            .map_err(ApiProblem::from_kernel_error)?;
    }
    validate_profile_suggested_prompts(profile.suggested_prompts.as_deref().unwrap_or_default())?;
    validate_optional_profile_string(
        profile.system_prompt.as_deref(),
        "managementProfile.systemPrompt",
        32768,
    )?;
    if let Some(temperature) = profile.temperature {
        if temperature < 0.0 {
            return Err(ApiProblem::validation(
                "managementProfile.temperature must be greater than or equal to 0",
            ));
        }
        if temperature > 2.0 {
            return Err(ApiProblem::validation(
                "managementProfile.temperature must be less than or equal to 2",
            ));
        }
    }
    if let Some(agent_type) = profile.agent_type.as_deref() {
        if !matches!(agent_type, "normal" | "independent") {
            return Err(ApiProblem::validation(
                "managementProfile.type must be one of normal, independent",
            ));
        }
    }
    validate_optional_profile_string(profile.users.as_deref(), "managementProfile.users", 128)?;
    validate_optional_profile_string(
        profile.welcome_message.as_deref(),
        "managementProfile.welcomeMessage",
        4096,
    )?;
    validate_profile_standard_id_array(
        profile.knowledge_base_ids.as_deref(),
        "managementProfile.knowledgeBaseIds",
        "knowledge.base.",
        128,
    )?;
    validate_profile_standard_id_array(
        profile.skill_ids.as_deref(),
        "managementProfile.skillIds",
        "skill.",
        128,
    )?;
    validate_profile_standard_id_array(
        profile.tool_ids.as_deref(),
        "managementProfile.toolIds",
        "tool.",
        128,
    )?;
    validate_profile_standard_id_array(
        profile.voice_ids.as_deref(),
        "managementProfile.voiceIds",
        "voice.",
        16,
    )?;
    Ok(())
}

fn validate_profile_standard_id_array(
    values: Option<&[String]>,
    field_name: &str,
    prefix: &str,
    max_items: usize,
) -> Result<(), ApiProblem> {
    let Some(values) = values else {
        return Ok(());
    };
    if values.len() > max_items {
        return Err(ApiProblem::validation(format!(
            "{field_name} must contain at most {max_items} items"
        )));
    }
    for value in values {
        if is_trimmed_blank(value) {
            return Err(ApiProblem::validation(format!(
                "{field_name} items is required"
            )));
        }
        if !value.starts_with(prefix) {
            return Err(ApiProblem::validation(format!(
                "{field_name} items must start with {prefix}"
            )));
        }
        validate_standard_id(value, field_name, Some(prefix))
            .map_err(ApiProblem::from_kernel_error)?;
    }
    Ok(())
}

fn validate_optional_profile_string(
    value: Option<&str>,
    field_name: &str,
    max_length: usize,
) -> Result<(), ApiProblem> {
    let Some(value) = value else {
        return Ok(());
    };
    let length = value.chars().count();
    if length == 0 {
        return Err(ApiProblem::validation(format!("{field_name} is required")));
    }
    if length > max_length {
        return Err(ApiProblem::validation(format!(
            "{field_name} must be at most {max_length} characters"
        )));
    }
    Ok(())
}

fn validate_profile_suggested_prompts(values: &[String]) -> Result<(), ApiProblem> {
    if values.len() > 12 {
        return Err(ApiProblem::validation(
            "managementProfile.suggestedPrompts must contain at most 12 items",
        ));
    }
    for value in values {
        let length = value.chars().count();
        if length == 0 {
            return Err(ApiProblem::validation(
                "managementProfile.suggestedPrompts items is required",
            ));
        }
        if length > 256 {
            return Err(ApiProblem::validation(
                "managementProfile.suggestedPrompts items must be at most 256 characters",
            ));
        }
    }
    Ok(())
}

/// Kernel- and axum-rejection-aware constructors for the canonical
/// `crate::response::ApiProblem`. These live here (not in `response.rs`) because
/// they depend on `sdkwork-agent-kernel` and axum extractors that the generic
/// response module must not import.
impl ApiProblem {
    pub(crate) fn from_kernel_error(error: KernelError) -> Self {
        // Structured not-found classification first: adapters may construct
        // validation errors whose message contains "not found" without meaning
        // HTTP 404, so the mapping must key on the structured marker.
        if error.detail_value("sdkwork.not_found") == Some("true") {
            return Self::not_found(error.safe_message());
        }
        let safe_message = error.safe_message();
        match error.kind() {
            KernelErrorKind::ValidationError => Self::validation(error.safe_message()),
            KernelErrorKind::Conflict => {
                if error.detail_value("sdkwork.version_mismatch") == Some("true")
                    || safe_message.contains("version mismatch")
                {
                    Self::version_conflict(safe_message)
                } else {
                    Self::conflict(safe_message)
                }
            }
            KernelErrorKind::PermissionRequired | KernelErrorKind::PolicyDenied => {
                Self::permission(error.safe_message())
            }
            KernelErrorKind::CapabilityMissing | KernelErrorKind::SecurityViolation => {
                Self::permission(error.safe_message())
            }
            KernelErrorKind::ProviderUnavailable | KernelErrorKind::ProviderError => {
                // Carry the standard 50301 result code so the problem body
                // includes `code`/`i18nKey` and keeps the business-safe
                // message (provider/account-pool reason) in `detail`.
                Self::dependency_unavailable(error.safe_message())
                    .with_result_code(SdkWorkResultCode::ServiceUnavailable)
            }
            KernelErrorKind::Timeout => Self::gateway_timeout(error.safe_message()),
            KernelErrorKind::Cancelled => Self::conflict(error.safe_message()),
            KernelErrorKind::RateLimited => Self::too_many_requests(error.safe_message(), None),
            KernelErrorKind::ResourceExhausted => {
                Self::dependency_unavailable(error.safe_message())
                    .with_result_code(SdkWorkResultCode::ServiceUnavailable)
            }
            KernelErrorKind::UnsafeContent => Self::unprocessable(error.safe_message()),
            KernelErrorKind::InternalError => Self::internal(error.safe_message()),
        }
    }

    pub(crate) fn from_json_rejection(rejection: JsonRejection) -> Self {
        Self::validation(format!("invalid json request: {}", rejection.body_text()))
    }

    pub(crate) fn from_query_rejection(rejection: QueryRejection) -> Self {
        Self::validation(format!("invalid query request: {}", rejection.body_text()))
    }

    pub(crate) fn from_path_rejection(rejection: PathRejection) -> Self {
        Self::validation(format!("invalid path request: {}", rejection.body_text()))
    }
}

async fn app_list_agents(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<AppListAgentsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let query = ListAgentsQueryParams {
            scope: query.scope,
            include_deleted: query.include_deleted,
            q: query.q,
            page: query.page,
            page_size: query.page_size,
        };
        execute_list(state, query, scope, true).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_agents(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<ListAgentsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list(state, query, RequestScope::from_context(context), false).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<CreateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create(state, RequestScope::from_context(context), body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_create_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<CreateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create(state, RequestScope::from_context(context), body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_get(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_get_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_get(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<UpdateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update(state, RequestScope::from_context(context), agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_agent(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update(state, RequestScope::from_context(context), agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_delete(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn open_delete_agent(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_delete(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_restore_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<RestoreAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_restore(state, RequestScope::from_context(context), agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_agent_status(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentStatusBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let subject = scope.subject.clone();
        let command = UpdateAgentStatusRequestDto {
            tenant_id: scope.tenant_id,
            agent_id,
            expected_version: body.expected_version,
            target_status: body.target_status,
            requested_at: body.requested_at,
        }
        .into_command(subject)
        .map_err(ApiProblem::from_kernel_error)?;

        let record = with_service(&state, move |service| service.change_status(command)).await?;
        Ok(ResourceData {
            item: map_agent_record(&AgentRecordDto::from_record(&record))?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_restore_agent(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<RestoreAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        execute_restore(state, scope, agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_agent_audit_events(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AuditEventsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentAuditEventResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let subject = scope.subject.clone();
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_audit_event_list_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        validate_audit_action_filter(query.action.as_deref())?;
        validate_audit_range(query.from.as_deref(), query.to.as_deref())?;
        let audit_query = AuditEventListQuery::for_agent(tenant_id, path.agent_id.clone())
            .with_cursor_page(page_size, cursor);
        let audit_query = if let Some(action) = query.action.clone() {
            audit_query.with_action(action)
        } else {
            audit_query
        };
        let audit_query = if let Some(from) = query.from.clone() {
            audit_query.with_from(from)
        } else {
            audit_query
        };
        let audit_query = if let Some(to) = query.to.clone() {
            audit_query.with_to(to)
        } else {
            audit_query
        };
        let result = with_service(&state, move |service| {
            service.list_agent_audit_events(ListAgentAuditEventsCommand {
                query: audit_query,
                requested_by: subject,
            })
        })
        .await?;
        let items: Vec<AgentAuditEventResponse> = result
            .items
            .into_iter()
            .map(|event| AgentAuditEventResponse {
                event_id: event.event_id,
                event_type: event.event_type,
                severity: kernel_event_severity(event.severity).to_string(),
                payload: event.payload,
                occurred_at: event.occurred_at.unwrap_or_default(),
            })
            .collect();

        Ok(PageData {
            items,
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                result.next_page_token,
                result.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_provider_bindings(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_provider_bindings(
            state,
            RequestScope::from_context(context),
            query.page,
            query.page_size,
            path.agent_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_provider_bindings(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_provider_bindings(
            state,
            RequestScope::from_context(context),
            query.page,
            query.page_size,
            path.agent_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_add_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_add_provider_binding(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_add_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_add_provider_binding(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_activate_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentBindingPathParams>, PathRejection>,
    body: Result<Json<ActivateProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_activate_provider_binding(state, RequestScope::from_context(context), path, body)
            .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_activate_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentBindingPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ActivateProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_activate_provider_binding(state, RequestScope::from_context(context), path, body)
            .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_preview_response(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentPreviewResponseBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_preview_response(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_create_prompt_optimization(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentPromptOptimizationBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_prompt_optimization(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn open_create_preview_response(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPreviewResponseBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_preview_response(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn open_create_prompt_optimization(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPromptOptimizationBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_prompt_optimization(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_list_composition_slots(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<ListCompositionSlotsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_composition_slots(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_agent_engines(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentEngineCatalog>> = async {
        let scope = RequestScope::from_context(context);
        let subject = scope.subject().clone();
        let catalog =
            with_service(&state, |service| service.list_agent_engine_catalog(subject)).await?;
        Ok(ResourceData { item: catalog })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_mcp_servers(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<ListMcpServersQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<McpServerMarketplaceRecord>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let subject = scope.subject().clone();
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut list_query = McpMarketplaceListQuery::for_tenant(tenant_id).with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        if let Some(q) = query.q {
            list_query = list_query.with_q(q);
        }
        let result = with_service(&state, move |service| {
            service.list_mcp_marketplace(ListMcpMarketplaceCommand {
                query: list_query,
                requested_by: subject,
            })
        })
        .await?;
        let total_items = result.total_count.unwrap_or(0) as usize;
        Ok(PageData {
            items: result.items,
            page_info: PageInfo {
                mode: PageMode::Offset,
                page: Some(page as i32),
                page_size: Some(page_size as i32),
                total_items: Some(total_items.to_string()),
                total_pages: Some(total_pages(total_items, page_size) as i32),
                next_cursor: None,
                has_more: Some(result.has_more),
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentCompositionSlotCreateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_get_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    body: Result<Json<AgentCompositionSlotUpdateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_delete_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_list_composition_slots(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_composition_slots(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentCompositionSlotCreateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_get_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentCompositionSlotUpdateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_delete_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_delete_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}
async fn execute_list_composition_slots(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    page: Option<usize>,
    page_size: Option<usize>,
) -> ApiResult<PageData<AgentCompositionSlotRecordResponse>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let tenant_id = scope.tenant_id_u64()?;
    let subject = scope.subject.clone();
    let result = with_service(&state, move |service| {
        service.list_composition_slots(AgentCompositionSlotListCommand {
            query: CompositionSlotListQuery::for_agent(tenant_id, agent_id).with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            ),
            requested_by: subject,
        })
    })
    .await?;
    let items: Vec<AgentCompositionSlotRecordResponse> = result
        .items
        .iter()
        .map(|record| {
            map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(record))
        })
        .collect();
    let total_items = result.total_count.unwrap_or(0) as usize;
    Ok(PageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page as i32),
            page_size: Some(page_size as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(total_pages(total_items, page_size) as i32),
            next_cursor: None,
            has_more: Some(result.has_more),
        },
    })
}

async fn execute_create_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentCompositionSlotCreateRequestDto,
) -> ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> {
    let tenant_id = scope.tenant_id_u64()?;
    let organization_id =
        parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
    validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let slot_kind = AgentCompositionSlotKind::try_from_str(body.slot_kind.as_str())
        .ok_or_else(|| ApiProblem::bad_request("invalid slotKind"))?;
    let target_module = AgentCompositionTargetModule::try_from_str(body.target_module.as_str())
        .ok_or_else(|| ApiProblem::bad_request("invalid targetModule"))?;
    let command = AgentCompositionSlotCreateCommand {
        tenant_id,
        organization_id,
        agent_id,
        slot_id: body.slot_id,
        slot_kind,
        target_module,
        target_ref: body.target_ref,
        target_version_ref: body.target_version_ref,
        priority: body.priority.unwrap_or(0),
        enabled: body.enabled.unwrap_or(true),
        policy_json: body.policy_json.unwrap_or_else(|| "{}".to_string()),
        requested_by: scope.subject,
        requested_at: body.requested_at,
    };
    let record = with_service(&state, move |service| {
        service.create_composition_slot(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    })
}

async fn execute_get_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
) -> ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> {
    let tenant_id = scope.tenant_id_u64()?;
    let command = AgentCompositionSlotGetCommand {
        tenant_id,
        agent_id,
        slot_id,
        requested_by: scope.subject,
    };
    let record = with_service(&state, move |service| service.get_composition_slot(command)).await?;
    Ok(ResourceData {
        item: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    })
}

async fn execute_update_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
    body: AgentCompositionSlotUpdateRequestDto,
) -> ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> {
    let tenant_id = scope.tenant_id_u64()?;
    validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let expected_version = body
        .expected_version
        .as_deref()
        .map(parse_expected_version)
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let slot_kind = body
        .slot_kind
        .as_deref()
        .map(|value| {
            AgentCompositionSlotKind::try_from_str(value)
                .ok_or_else(|| KernelError::validation("invalid slotKind"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let target_module = body
        .target_module
        .as_deref()
        .map(|value| {
            AgentCompositionTargetModule::try_from_str(value)
                .ok_or_else(|| KernelError::validation("invalid targetModule"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let command = AgentCompositionSlotUpdateCommand {
        tenant_id,
        agent_id,
        slot_id,
        expected_version,
        slot_kind,
        target_module,
        target_ref: body.target_ref,
        target_version_ref: body.target_version_ref,
        priority: body.priority,
        enabled: body.enabled,
        policy_json: body.policy_json,
        requested_by: scope.subject,
        requested_at: body.requested_at,
    };
    let record = with_service(&state, move |service| {
        service.update_composition_slot(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    })
}

async fn execute_delete_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
) -> ApiResult<()> {
    let tenant_id = scope.tenant_id_u64()?;
    let command = AgentCompositionSlotDeleteCommand {
        tenant_id,
        agent_id,
        slot_id,
        expected_version: None,
        requested_by: scope.subject,
        requested_at: server_requested_at(),
    };
    with_service(&state, move |service| {
        service.delete_composition_slot(command)
    })
    .await?;
    Ok(())
}

fn map_composition_slot_record(
    record: &AgentCompositionSlotRecordDto,
) -> AgentCompositionSlotRecordResponse {
    AgentCompositionSlotRecordResponse {
        id: record.id.clone(),
        tenant_id: record.tenant_id.clone(),
        organization_id: record.organization_id.clone(),
        agent_id: record.agent_id.clone(),
        slot_id: record.slot_id.clone(),
        slot_kind: record.slot_kind.clone(),
        target_module: record.target_module.clone(),
        target_ref: record.target_ref.clone(),
        target_version_ref: record.target_version_ref.clone(),
        priority: record.priority,
        enabled: record.enabled,
        policy_json: record.policy_json.clone(),
        status: record.status.clone(),
        version: record.version.clone(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        deleted_at: record.deleted_at.clone(),
    }
}

// ===========================================================================
// Project handlers - App API
// ===========================================================================

async fn app_ensure_default_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppEnsureDefaultWorkspaceBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = EnsureDefaultWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            owner_user_id,
            default_name: body.name,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.ensure_default_workspace(command)
        })
        .await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_create_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppCreateWorkspaceBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = CreateWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            owner_user_id,
            name: body.name,
            description: body.description,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.create_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_get_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = GetWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppUpdateWorkspaceBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            name: body.name,
            description: body.description,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.update_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_archive_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppWorkspaceMutationBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = WorkspaceMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record =
            with_service(&state, move |service| service.archive_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppDeleteWorkspaceQuery>, QueryRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = WorkspaceMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            expected_version: query
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| service.delete_workspace(command)).await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_list_workspaces(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<AppListWorkspacesQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentWorkspaceRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut workspace_query = WorkspaceListQuery::for_owner(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            owner_user_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        workspace_query.status = query
            .status
            .as_deref()
            .map(parse_workspace_status)
            .transpose()?;
        workspace_query.include_deleted = query.include_deleted.unwrap_or(false);
        let records = with_service(&state, move |service| {
            service.list_workspaces(ListWorkspacesCommand {
                query: workspace_query,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records.items.iter().map(workspace_response).collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_projects(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<AppListProjectsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentProjectRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut project_query = ProjectListQuery::for_organization(tenant_id, organization_id)
            .for_owner(owner_user_id)
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            );
        if query.name_exact.is_some() && query.workspace_id.is_none() {
            return Err(ApiProblem::validation(
                "workspaceId is required when name_exact is provided",
            ));
        }
        if let Some(workspace_id) = query.workspace_id {
            project_query = project_query.for_workspace(workspace_id);
        }
        if let Some(exact_name) = query.name_exact {
            if exact_name.trim().len() > 255 {
                return Err(ApiProblem::validation("name_exact exceeds 255 bytes"));
            }
            project_query = project_query.with_exact_name(exact_name);
        }
        if let Some(search) = query.q {
            project_query = project_query.with_search(search);
        }
        if let Some(status) = query.status {
            project_query = project_query.with_status(parse_project_status(&status)?);
        }
        project_query.include_deleted = query.include_deleted.unwrap_or(false);
        let records = with_service(&state, move |service| {
            service.list_projects(ListProjectsCommand {
                query: project_query,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records.items.iter().map(project_response).collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppCreateProjectBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = CreateProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id: body.project_id.unwrap_or_default(),
            workspace_id: body.workspace_id,
            owner_user_id,
            name: body.name,
            description: body.description,
            visibility: parse_project_visibility(body.visibility.as_deref().unwrap_or("private"))?,
            drive_access_mode: parse_project_drive_access(
                body.drive_access_mode.as_deref().unwrap_or("owner_library"),
            )?,
            default_agent_id: body.default_agent_id,
            default_model_id: body.default_model_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.create_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_import_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppImportProjectBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = ImportProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id: body.workspace_id,
            project_id: body.project_id.unwrap_or_default(),
            owner_user_id,
            name: body.name,
            description: body.description,
            source_kind: body.source_kind,
            source_ref: body.source_ref,
            drive_space_id: body.drive_space_id,
            drive_root_entry_id: body.drive_root_entry_id,
            drive_logical_path: body.drive_logical_path.unwrap_or_default(),
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.import_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppUpdateProjectBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: Some(requested_user_id),
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            name: body.name,
            description: body.description,
            visibility: body
                .visibility
                .as_deref()
                .map(parse_project_visibility)
                .transpose()?,
            drive_access_mode: body
                .drive_access_mode
                .as_deref()
                .map(parse_project_drive_access)
                .transpose()?,
            default_agent_id: body.default_agent_id,
            default_model_id: body.default_model_id,
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.update_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_archive_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppProjectMutationBody>, JsonRejection>,
) -> Response {
    app_mutate_project(state, context, web_ctx, project_id, body, false).await
}

async fn app_delete_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = ProjectMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: Some(requested_user_id),
            expected_version: None,
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| service.delete_project(command)).await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_mutate_project(
    state: AgentHttpState,
    context: AgentRequestContext,
    web_ctx: sdkwork_web_core::WebRequestContext,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppProjectMutationBody>, JsonRejection>,
    _delete_project: bool,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = ProjectMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: Some(requested_user_id),
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.archive_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

fn project_response(record: &AgentProjectRecord) -> AgentProjectRecordResponse {
    AgentProjectRecordResponse {
        id: record.id.to_string(),
        project_id: record.project_id.clone(),
        workspace_id: record.workspace_id.clone(),
        tenant_id: record.tenant_id.to_string(),
        organization_id: record.organization_id.to_string(),
        owner_user_id: record.owner_user_id.to_string(),
        name: record.name.clone(),
        description: record.description.clone(),
        visibility: record.visibility.as_str().to_string(),
        status: record.status.as_str().to_string(),
        drive_access_mode: record.drive_access_mode.as_str().to_string(),
        default_agent_id: record.default_agent_id.clone(),
        default_model_id: record.default_model_id.clone(),
        import_source_kind: record.import_source_kind.clone(),
        import_source_ref: record.import_source_ref.clone(),
        drive_space_id: record.drive_space_id.clone(),
        drive_root_entry_id: record.drive_root_entry_id.clone(),
        drive_logical_path: record.drive_logical_path.clone(),
        version: record.version.to_string(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        archived_at: record.archived_at.clone(),
    }
}

fn workspace_response(record: &AgentWorkspaceRecord) -> AgentWorkspaceRecordResponse {
    AgentWorkspaceRecordResponse {
        id: record.id.to_string(),
        workspace_id: record.workspace_id.clone(),
        tenant_id: record.tenant_id.to_string(),
        organization_id: record.organization_id.to_string(),
        owner_user_id: record.owner_user_id.to_string(),
        name: record.name.clone(),
        description: record.description.clone(),
        is_default: record.is_default,
        status: record.status.as_str().to_string(),
        version: record.version.to_string(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        archived_at: record.archived_at.clone(),
    }
}

async fn app_list_project_composition_slots(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListProjectCompositionSlotsQuery>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut slot_query = ProjectCompositionSlotListQuery::for_project(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            project_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        slot_query.slot_kind = query
            .slot_kind
            .as_deref()
            .map(parse_project_composition_slot_kind)
            .transpose()?;
        slot_query.enabled = query.enabled;
        let records = with_service(&state, move |service| {
            service.list_project_composition_slots(ListProjectCompositionSlotsCommand {
                query: slot_query,
                owner_scope: Some(owner_user_id),
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(project_composition_slot_response)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppCreateProjectCompositionSlotBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = CreateProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id: body.slot_id,
            slot_kind: parse_project_composition_slot_kind(&body.slot_kind)?,
            target_module: parse_project_composition_target_module(&body.target_module)?,
            target_ref: body.target_ref,
            target_version_ref: body.target_version_ref,
            priority: body.priority.unwrap_or(0),
            enabled: body.enabled.unwrap_or(true),
            policy_json: body.policy_json.unwrap_or_else(|| "{}".to_string()),
            owner_scope: Some(requested_user_id),
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.create_project_composition_slot(command)
        })
        .await?;
        Ok(ResourceData {
            item: project_composition_slot_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_get_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path((project_id, slot_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_project_composition_slot(command)
        })
        .await?;
        Ok(ResourceData {
            item: project_composition_slot_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppUpdateProjectCompositionSlotBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path((project_id, slot_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        if body.target_version_ref.is_some() && body.clear_target_version_ref.unwrap_or(false) {
            return Err(ApiProblem::validation(
                "targetVersionRef and clearTargetVersionRef cannot be supplied together",
            ));
        }
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let target_version_ref = if body.clear_target_version_ref.unwrap_or(false) {
            Some(None)
        } else {
            body.target_version_ref.map(Some)
        };
        let command = UpdateProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            slot_kind: body
                .slot_kind
                .as_deref()
                .map(parse_project_composition_slot_kind)
                .transpose()?,
            target_module: body
                .target_module
                .as_deref()
                .map(parse_project_composition_target_module)
                .transpose()?,
            target_ref: body.target_ref,
            target_version_ref,
            priority: body.priority,
            enabled: body.enabled,
            policy_json: body.policy_json,
            owner_scope: Some(requested_user_id),
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.update_project_composition_slot(command)
        })
        .await?;
        Ok(ResourceData {
            item: project_composition_slot_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppDeleteProjectCompositionSlotQuery>, QueryRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path((project_id, slot_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = DeleteProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id,
            expected_version: query
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            owner_scope: Some(requested_user_id),
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| {
            service.delete_project_composition_slot(command)
        })
        .await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

fn project_composition_slot_response(
    record: &AgentProjectCompositionSlotRecord,
) -> AgentProjectCompositionSlotRecordResponse {
    AgentProjectCompositionSlotRecordResponse {
        id: record.id.to_string(),
        tenant_id: record.tenant_id.to_string(),
        organization_id: record.organization_id.to_string(),
        project_id: record.project_id.clone(),
        slot_id: record.slot_id.clone(),
        slot_kind: record.slot_kind.as_str().to_string(),
        target_module: record.target_module.as_str().to_string(),
        target_ref: record.target_ref.clone(),
        target_version_ref: record.target_version_ref.clone(),
        priority: record.priority,
        enabled: record.enabled,
        policy_json: record.policy_json.clone(),
        created_by: record.created_by.to_string(),
        updated_by: record.updated_by.to_string(),
        version: record.version.to_string(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn parse_project_composition_slot_kind(value: &str) -> ApiResult<AgentCompositionSlotKind> {
    AgentCompositionSlotKind::try_from_str(value)
        .ok_or_else(|| ApiProblem::validation("invalid project composition slot kind"))
}

fn parse_project_composition_target_module(value: &str) -> ApiResult<AgentCompositionTargetModule> {
    AgentCompositionTargetModule::try_from_str(value)
        .ok_or_else(|| ApiProblem::validation("invalid project composition target module"))
}

fn parse_project_status(value: &str) -> ApiResult<AgentProjectStatus> {
    match value {
        "active" => Ok(AgentProjectStatus::Active),
        "archived" => Ok(AgentProjectStatus::Archived),
        "deleted" => Ok(AgentProjectStatus::Deleted),
        _ => Err(ApiProblem::validation("invalid project status")),
    }
}

fn parse_workspace_status(value: &str) -> ApiResult<AgentWorkspaceStatus> {
    match value {
        "active" => Ok(AgentWorkspaceStatus::Active),
        "archived" => Ok(AgentWorkspaceStatus::Archived),
        "deleted" => Ok(AgentWorkspaceStatus::Deleted),
        _ => Err(ApiProblem::validation("invalid workspace status")),
    }
}

fn parse_project_visibility(value: &str) -> ApiResult<AgentProjectVisibility> {
    match value {
        "private" => Ok(AgentProjectVisibility::Private),
        "organization" => Ok(AgentProjectVisibility::Organization),
        "shared" => Ok(AgentProjectVisibility::Shared),
        _ => Err(ApiProblem::validation("invalid project visibility")),
    }
}

fn parse_project_drive_access(value: &str) -> ApiResult<AgentProjectDriveAccessMode> {
    match value {
        "disabled" => Ok(AgentProjectDriveAccessMode::Disabled),
        "owner_library" => Ok(AgentProjectDriveAccessMode::OwnerLibrary),
        "explicit_resources" => Ok(AgentProjectDriveAccessMode::ExplicitResources),
        _ => Err(ApiProblem::validation("invalid project drive access mode")),
    }
}

// ===========================================================================
// Session handlers  - App API
// ===========================================================================

async fn app_list_session_activity_summaries(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<AppListSessionActivitySummariesQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<SessionActivitySummaryDto>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let page_size = query.page_size.unwrap_or(20);
        if page_size == 0 || page_size > MAX_PAGE_SIZE {
            return Err(ApiProblem::invalid_parameter(
                "page_size must be between 1 and 200",
            ));
        }

        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let cursor = match query.cursor.as_deref() {
            Some(cursor) if !(1..=2048).contains(&cursor.chars().count()) => {
                return Err(ApiProblem::invalid_parameter(
                    "cursor must be between 1 and 2048 characters",
                ));
            }
            cursor => cursor,
        };
        let cursor = cursor
            .map(decode_session_activity_cursor)
            .transpose()
            .map_err(|error| ApiProblem::invalid_parameter(error.message()))?;

        let validate_scope_id =
            |value: Option<String>, field_name: &str, prefix: &str| -> ApiResult<Option<String>> {
                if let Some(value) = value {
                    validate_standard_id(&value, field_name, Some(prefix))
                        .map_err(|error| ApiProblem::invalid_parameter(error.message()))?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            };
        let workspace_id = validate_scope_id(query.workspace_id, "workspace_id", "workspace.")?;
        let project_id = validate_scope_id(query.project_id, "project_id", "project.")?;
        let agent_id = validate_scope_id(query.agent_id, "agent_id", "agent.")?;

        let mut activity_query =
            SessionActivitySummaryListQuery::for_owner(tenant_id, organization_id, owner_user_id)
                .with_page_size(page_size);
        if let Some(workspace_id) = workspace_id {
            activity_query = activity_query.for_workspace(workspace_id);
        }
        if let Some(project_id) = project_id {
            activity_query = activity_query.for_project(project_id);
        }
        if let Some(agent_id) = agent_id {
            activity_query = activity_query.for_agent(agent_id);
        }
        if let Some(cursor) = cursor {
            if cursor.scope_fingerprint != activity_query.scope_fingerprint() {
                return Err(ApiProblem::invalid_parameter(
                    "cursor does not belong to the requested Session activity scope",
                ));
            }
            activity_query = activity_query.after(cursor);
        }

        let records = with_service(&state, move |service| {
            service.list_session_activity_summaries(ListSessionActivitySummariesCommand {
                query: activity_query,
                requested_by: scope.subject,
            })
        })
        .await?;
        let items = records
            .items
            .into_iter()
            .map(enrich_provider_session_activity)
            .map(|record| SessionActivitySummaryDto::from_record(&record))
            .collect::<KernelResult<Vec<_>>>()
            .map_err(ApiProblem::from_kernel_error)?;
        Ok(PageData {
            items,
            page_info: PageInfo {
                mode: PageMode::Cursor,
                page: None,
                page_size: Some(page_size as i32),
                total_items: None,
                total_pages: None,
                next_cursor: records.next_page_token,
                has_more: Some(records.has_more),
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

fn enrich_provider_session_activity(
    summary: SessionActivitySummaryRecord,
) -> SessionActivitySummaryRecord {
    let Some(provider_session_id) = summary.provider_identity.provider_session_id.as_deref() else {
        return summary;
    };
    let observation = engine_key_for_provider_identity(
        summary.provider_identity.provider_binding_id.as_deref(),
        summary.provider_identity.provider_id.as_deref(),
    )
    .and_then(|engine_key| {
        shared_agent_engine_host().map(|host| {
            host.get_provider_session_activity(engine_key, provider_session_id)
                .map(|snapshot| {
                    SessionProviderActivityObservation::from_provider_snapshot(
                        provider_session_id,
                        snapshot,
                    )
                })
                .unwrap_or_else(|error| {
                    tracing::warn!(
                        target: "sdkwork.agents.session_activity",
                        engine_key,
                        provider_session_id,
                        error = %error,
                        "provider Session activity observation is unavailable"
                    );
                    SessionProviderActivityObservation::unavailable(provider_session_id)
                })
        })
    })
    .unwrap_or_else(|| SessionProviderActivityObservation::unavailable(provider_session_id));
    summary.with_provider_activity(observation)
}

async fn app_list_sessions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListSessionsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_session_list_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(scope.owner_user_id),
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_organization(
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
            )
            .for_agent(agent_id)
            .with_cursor_page(page_size, cursor);
        if let Some(project_id) = query.project_id {
            command.query = command.query.for_project(project_id);
        }
        let records = with_service(&state, move |service| service.list_sessions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentSessionRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_project_sessions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    query: Result<Query<ListProjectSessionsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionRecordDto>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_session_list_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(owner_user_id.to_string()),
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject.clone())
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_organization(organization_id)
            .for_project(project_id.clone())
            .with_cursor_page(page_size, cursor);
        let records = with_service(&state, move |service| service.list_sessions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentSessionRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_synchronize_project_sessions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
) -> Response {
    use crate::provider_session_sync::{
        clear_provider_session_sync_in_flight, mark_provider_session_sync_in_flight,
        provider_session_sync_cache_key, read_completed_provider_session_sync,
        synchronize_project_provider_sessions, synchronize_project_provider_sessions_at_cwd,
        ProviderSessionSynchronizationResult, PROVIDER_SESSION_SYNC_REFRESH_TTL,
    };

    let trace_id = web_ctx.resolved_trace_id();
    let result: Result<Response, ApiProblem> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let subject = scope.subject;
        let subject_for_project = subject.clone();
        let provider_session_cwd_resolver = state.provider_session_cwd_resolver.clone();
        // Project validation and cwd resolution stay in the request (fast);
        // the provider discovery scan and the inventory sweeps run on a
        // background worker so a cold synchronization never occupies an HTTP
        // request for up to the 15-second reconciliation timeout.
        let project = with_owned_service(&state, move |service| {
            service.get_project(GetProjectCommand {
                tenant_id,
                organization_id,
                project_id: project_id.clone(),
                owner_scope: Some(owner_user_id),
                requested_by: subject_for_project,
            })
        })
        .await?;
        let exact_cwd = provider_session_cwd_resolver
            .as_ref()
            .map(|resolver| {
                resolver.resolve_project_cwd(
                    &sdkwork_agents_runtime_facade::ProviderSessionProjectCwdSelector {
                        tenant_id: project.tenant_id,
                        organization_id: project.organization_id,
                        owner_user_id: project.owner_user_id,
                        project_id: project.project_id.clone(),
                        project_name: project.name.clone(),
                    },
                )
            })
            .transpose()
            .map_err(crate::provider_session_sync::runtime_facade_error)
            .map_err(ApiProblem::from_kernel_error)?
            .flatten();
        let cache_key = provider_session_sync_cache_key(&project);
        let project_id_dto = project.project_id.clone();
        // Fast path: a completed synchronization inside the refresh window is
        // returned immediately with the cached outcome.
        if let Some(cached) = read_completed_provider_session_sync(&cache_key) {
            if cached.completed_at.elapsed() < PROVIDER_SESSION_SYNC_REFRESH_TTL {
                tracing::debug!(
                    target: "sdkwork.agents.provider_session_sync",
                    project_id = %project.project_id,
                    "provider Session synchronization served from the refresh cache"
                );
                let item = ProjectSessionSynchronizationResultDto::from_result(
                    &cached.result,
                    project_id_dto,
                    "completed",
                );
                return Ok(crate::response::success_response(
                    &web_ctx,
                    StatusCode::OK,
                    ResourceData { item },
                )?);
            }
        }
        // In-flight dedupe: a burst of concurrent requests for the same
        // project must never duplicate the discovery scan or the sweeps.
        // The RAII guard releases the marker on every exit path (including
        // worker panics), so a project can never wedge into a permanent
        // `202 pending`.
        if !mark_provider_session_sync_in_flight(&cache_key) {
            tracing::debug!(
                target: "sdkwork.agents.provider_session_sync",
                project_id = %project.project_id,
                "provider Session synchronization already in flight"
            );
            let item = ProjectSessionSynchronizationResultDto::from_result(
                &ProviderSessionSynchronizationResult::default(),
                project_id_dto,
                "pending",
            );
            return Ok(crate::response::success_response(
                &web_ctx,
                StatusCode::ACCEPTED,
                ResourceData { item },
            )?);
        }
        // Enqueue the synchronization on a bounded background worker. The
        // worker permit keeps the total in-process synchronization concurrency
        // inside SERVICE_WORKER_LIMIT; the 15-second reconciliation timeout
        // bounds each run.
        let _sync_guard =
            crate::provider_session_sync::ProviderSessionSyncGuard::new(cache_key.clone());
        let service = Arc::clone(&state.service);
        let project_for_worker = project.clone();
        let subject_for_worker = subject.clone();
        let permit = SERVICE_WORKER_LIMIT
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                clear_provider_session_sync_in_flight(&cache_key);
                crate::infrastructure::AgentMetricsRegistry::global()
                    .record_service_worker_rejection();
                ApiProblem::too_many_requests("agents service concurrency limit reached", Some(1))
            })?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let outcome = if exact_cwd.is_some() {
                synchronize_project_provider_sessions_at_cwd(
                    service,
                    &project_for_worker,
                    subject_for_worker,
                    exact_cwd,
                )
            } else {
                synchronize_project_provider_sessions(
                    service,
                    &project_for_worker,
                    subject_for_worker,
                )
            };
            match outcome {
                Ok(result) => tracing::info!(
                    target: "sdkwork.agents.provider_session_sync",
                    project_id = %project_for_worker.project_id,
                    synchronized_session_count = result.synchronized_session_count,
                    failed_session_count = result.failed_session_count,
                    "provider Session inventory synchronization completed in background"
                ),
                Err(error) => tracing::error!(
                    target: "sdkwork.agents.provider_session_sync",
                    project_id = %project_for_worker.project_id,
                    error_kind = error.kind().as_str(),
                    "background provider Session synchronization failed: {error}"
                ),
            }
            // In-flight marker released by ProviderSessionSyncGuard on drop.
        });
        let item = ProjectSessionSynchronizationResultDto::from_result(
            &ProviderSessionSynchronizationResult::default(),
            project_id_dto,
            "accepted",
        );
        tracing::info!(
            target: "sdkwork.agents.provider_session_sync",
            trace_id = %trace_id,
            operation_id = "agents.projectSessions.synchronize",
            project_id = %project.project_id,
            "provider Session inventory synchronization accepted for background execution"
        );
        Ok(crate::response::success_response(
            &web_ctx,
            StatusCode::ACCEPTED,
            ResourceData { item },
        )?)
    }
    .await;
    crate::response::finish_api_response(&web_ctx, result)
}

async fn app_get_project_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((project_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let record = with_service(&state, move |service| {
            service.get_project_session(GetProjectSessionCommand {
                tenant_id,
                organization_id,
                project_id,
                session_id,
                owner_scope: Some(owner_user_id),
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_workspace_sessions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
    query: Result<Query<ListSessionsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionRecordDto>> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_session_list_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(owner_user_id.to_string()),
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject.clone())
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_organization(organization_id)
            .for_workspace(workspace_id.clone())
            .with_cursor_page(page_size, cursor);
        let records = with_service(&state, move |service| {
            service.get_workspace(GetWorkspaceCommand {
                tenant_id,
                organization_id,
                workspace_id,
                owner_user_id,
                requested_by: scope.subject,
            })?;
            service.list_sessions(command)
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentSessionRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_session_user_states(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListSessionUserStatesQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentResourceUserStateRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let session_ids = query
            .session_ids
            .as_deref()
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if session_ids.iter().any(String::is_empty) {
            return Err(ApiProblem::validation(
                "session_ids must contain only non-empty Session ids",
            ));
        }
        let mut state_query = ResourceUserStateListQuery::for_user_sessions(
            scope.tenant_id_u64()?,
            organization_id,
            owner_user_id,
        )
        .for_agent(agent_id.clone())
        .for_resource_ids(session_ids)
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        state_query.pinned_only = query.pinned_only.unwrap_or(false);
        state_query.include_hidden = query.include_hidden.unwrap_or(false);
        let records = with_service(&state, move |service| {
            service.list_session_user_states(ListSessionUserStatesCommand {
                query: state_query,
                path_agent_id: agent_id,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentResourceUserStateRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_session_user_state(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentResourceUserStateRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = GetSessionUserStateCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            user_id,
            path_agent_id: agent_id,
            session_id,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_session_user_state(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentResourceUserStateRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_session_user_state(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppUpdateSessionUserStateBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentResourceUserStateRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        if body.custom_title.is_some() && body.clear_custom_title.unwrap_or(false) {
            return Err(ApiProblem::validation(
                "customTitle and clearCustomTitle cannot be supplied together",
            ));
        }
        let custom_title = if body.clear_custom_title.unwrap_or(false) {
            Some(None)
        } else {
            body.custom_title.map(Some)
        };
        let last_read_item_sequence = body
            .last_read_item_sequence
            .map(|value| {
                value.parse::<u64>().map_err(|_| {
                    ApiProblem::validation("lastReadItemSequence must be an unsigned integer")
                })
            })
            .transpose()?;
        let expected_version = body
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateSessionUserStateCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            user_id,
            path_agent_id: agent_id,
            session_id,
            pinned: body.pinned,
            hidden: body.hidden,
            mark_opened: body.mark_opened.unwrap_or(false),
            last_read_item_sequence,
            custom_title,
            expected_version,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.update_session_user_state(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentResourceUserStateRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_item_feedback(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListItemFeedbackQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentItemFeedbackRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let feedback_query = ItemFeedbackListQuery::for_user_session(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            user_id,
            session_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| {
            service.list_item_feedback(ListItemFeedbackCommand {
                query: feedback_query,
                path_agent_id: agent_id,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentItemFeedbackRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_item_feedback(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AppUpdateItemFeedbackBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentItemFeedbackRecordDto>> = async {
        let Path((agent_id, session_id, item_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let clear_feedback = body.clear_feedback.unwrap_or(false);
        if clear_feedback && body.rating.is_some() {
            return Err(ApiProblem::validation(
                "rating and clearFeedback cannot be supplied together",
            ));
        }
        let rating = if clear_feedback {
            None
        } else {
            match body.rating.as_deref() {
                Some("up") => Some(AgentItemFeedbackRating::Up),
                Some("down") => Some(AgentItemFeedbackRating::Down),
                Some(_) => return Err(ApiProblem::validation("rating must be up or down")),
                None => {
                    return Err(ApiProblem::validation(
                        "rating or clearFeedback is required",
                    ))
                }
            }
        };
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateItemFeedbackCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            user_id,
            path_agent_id: agent_id,
            session_id,
            item_id,
            rating,
            reason_code: body.reason_code,
            comment: body.comment,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record =
            with_service(&state, move |service| service.update_item_feedback(command)).await?;
        Ok(ResourceData {
            item: AgentItemFeedbackRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        if let Some(body_agent_id) = body.agent_id.take() {
            if body_agent_id != agent_id {
                return Err(ApiProblem::validation(
                    "agentId must match the agentId path parameter",
                ));
            }
        }
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_create_project_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        if let Some(body_project_id) = body.project_id.as_deref() {
            if body_project_id != project_id {
                return Err(ApiProblem::validation(
                    "projectId must match the projectId path parameter",
                ));
            }
        }
        body.project_id = Some(project_id.clone());

        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let requested_by = scope.subject;
        let record = with_service(&state, move |service| {
            let project = service.get_project(GetProjectCommand {
                tenant_id,
                organization_id,
                project_id,
                owner_scope: Some(owner_user_id),
                requested_by: requested_by.clone(),
            })?;
            let agent_id = body
                .agent_id
                .take()
                .or(project.default_agent_id)
                .ok_or_else(|| {
                    KernelError::validation(
                        "agentId is required when the project has no defaultAgentId",
                    )
                })?;
            let command = body.into_command(
                tenant_id,
                organization_id,
                owner_user_id,
                agent_id,
                requested_by,
            )?;
            service.create_session(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetSessionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppUpdateSessionBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        if body.title.is_none() && body.project_id.is_none() && !body.clear_project.unwrap_or(false)
        {
            return Err(ApiProblem::validation(
                "session update requires a changed field",
            ));
        }
        if body.project_id.is_some() && body.clear_project.unwrap_or(false) {
            return Err(ApiProblem::validation(
                "projectId and clearProject cannot be supplied together",
            ));
        }
        let scope = RequestScope::from_context(context);
        let project_id = if body.clear_project.unwrap_or(false) {
            Some(None)
        } else {
            body.project_id.map(Some)
        };
        let command = UpdateSessionCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            title: body.title,
            project_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.update_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = DeleteSessionCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| service.delete_session(command)).await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_close_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CloseSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.close_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Task handlers  - App API
// ===========================================================================

async fn app_list_tasks(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListTasksQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_task_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListTasksRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            owner_user_id: Some(scope.owner_user_id),
            status: query.status,
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_agent(agent_id)
            .with_cursor_page(page_size, cursor);
        let records = with_service(&state, move |service| service.list_tasks(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTaskRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetTaskCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            task_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_cancel_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CancelTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.cancel_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_execute_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<ExecuteTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRunRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let run = with_service(&state, move |service| service.execute_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRunRecordDto::from_record(&run),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn replace_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<ReplaceTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                scope.owner_scope()?,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.replace_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn pause_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<TaskStateChangeRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_pause_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                scope.owner_scope()?,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.pause_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn resume_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<TaskStateChangeRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_resume_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                scope.owner_scope()?,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.resume_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn list_task_runs(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<ListTaskRunsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentTaskRunRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_task_run_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let command = ListTaskRunsRequestDto {
            status: query.status,
            trigger_kind: query.trigger_kind,
        }
        .into_command(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            task_id,
            scope.owner_scope()?,
            page_size,
            cursor,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
        let records = with_service(&state, move |service| service.list_task_runs(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTaskRunRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn get_task_run(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRunRecordDto>> = async {
        let Path((agent_id, task_id, run_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetTaskRunCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            task_id,
            run_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let run = with_service(&state, move |service| service.get_task_run(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRunRecordDto::from_record(&run),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn retry_task_run(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<RetryTaskRunRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRunRecordDto>> = async {
        let Path((agent_id, task_id, run_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                run_id,
                scope.owner_scope()?,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let run = with_service(&state, move |service| service.retry_task_run(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRunRecordDto::from_record(&run),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn cancel_task_run(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<CancelTaskRunRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRunRecordDto>> = async {
        let Path((agent_id, task_id, run_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                run_id,
                scope.owner_scope()?,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let run = with_service(&state, move |service| service.cancel_task_run(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRunRecordDto::from_record(&run),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn list_task_run_attempts(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    query: Result<Query<ListTaskRunAttemptsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentTaskRunAttemptRecordDto>> = async {
        let Path((agent_id, task_id, run_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_task_run_attempt_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let command = ListTaskRunAttemptsRequestDto::into_command(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            task_id,
            run_id,
            scope.owner_scope()?,
            page_size,
            cursor,
            scope.subject,
        );
        let records = with_service(&state, move |service| {
            service.list_task_run_attempts(command)
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTaskRunAttemptRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn reconcile_task_run(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ReconcileTaskRunRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRunRecordDto>> = async {
        let Path((agent_id, task_id, run_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                run_id,
                scope.owner_scope()?,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let run = with_service(&state, move |service| service.reconcile_task_run(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRunRecordDto::from_record(&run),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Interaction handlers  - App API
// ===========================================================================

async fn app_list_interactions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListInteractionsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(|value| decode_created_at_cursor(value, "interaction"))
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListInteractionsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.path_agent_id = agent_id;
        command.owner_scope = owner_scope;
        command.query = command.query.with_cursor_page(page_size, cursor);
        let records =
            with_service(&state, move |service| service.list_interactions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentInteractionRecordDto::from_record)
                .collect::<KernelResult<Vec<_>>>()
                .map_err(ApiProblem::from_kernel_error)?,
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.create_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetInteractionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            interaction_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_claim_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ClaimInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<InteractionClaimResultDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let claim = with_service(&state, move |service| service.claim_interaction(command)).await?;
        Ok(ResourceData {
            item: InteractionClaimResultDto::from_result(&claim)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_approve_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ApproveInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.approve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_answer_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AnswerInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.answer_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_resolve_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ResolveInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.resolve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Session item and turn handlers  - App API
// ===========================================================================

/// Outcome of the best-effort provider transcript synchronization returned by
/// `agents.sessionItems.synchronize`. The command never returns an item
/// window: the persisted window is read through `agents.sessionItems.list`
/// (API_SPEC §14.1.3 — commands MUST NOT hide list behavior behind POST).
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSessionItemSynchronizationResultDto {
    pub(crate) status: &'static str,
    pub(crate) imported_item_count: String,
}

async fn app_list_session_items(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListItemsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionItemRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_session_item_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListSessionItemsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
            sort: query.sort,
        }
        .into_command(agent_id, session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        command.query = command.query.with_cursor_page(page_size, cursor);
        let records = with_service(&state, move |service| {
            service.list_session_items_with_drive_refs(command)
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(|item| {
                    AgentSessionItemRecordDto::from_record_with_drive_refs(
                        &item.item,
                        &item.drive_refs,
                    )
                    .map_err(ApiProblem::from_kernel_error)
                })
                .collect::<Result<Vec<_>, _>>()?,
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_synchronize_session_items(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionItemSynchronizationResultDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id =
            owner_scope.ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let cwd_resolver = state.provider_session_cwd_resolver.clone();
        let outcome = with_service(&state, move |service| {
            crate::provider_session_sync::synchronize_provider_session_transcript(
                service,
                tenant_id,
                organization_id,
                owner_user_id,
                agent_id.clone(),
                session_id.clone(),
                scope.subject.clone(),
                cwd_resolver.as_deref(),
            )
        })
        .await?;
        tracing::info!(
            target: "sdkwork.agents.provider_session_sync",
            trace_id = %web_ctx.resolved_trace_id(),
            operation_id = "agents.sessionItems.synchronize",
            synchronization_status = outcome.status_code(),
            synchronized_item_count = outcome.imported_item_count(),
            "provider Session transcript synchronization completed"
        );
        Ok(ResourceData {
            item: AgentSessionItemSynchronizationResultDto {
                status: outcome.status_code(),
                imported_item_count: outcome.imported_item_count().to_string(),
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_turns(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppTurnsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(|value| decode_created_at_cursor(value, "turn"))
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let command = ListTurnsCommand {
            query: TurnListQuery::for_session(
                parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
            )
            .with_cursor_page(page_size, cursor),
            path_agent_id: agent_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let records = with_service(&state, move |service| service.list_turns(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTurnRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    headers: HeaderMap,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppCreateTurnQueryParams>, QueryRejection>,
    body: Result<Json<AppCreateTurnBody>, JsonRejection>,
) -> Response {
    let result: Result<Response, ApiProblem> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let stream_requested = query.stream.unwrap_or(false);
        let rich_events_requested =
            resolve_rich_turn_event_protocol(stream_requested, query.event_protocol.as_deref())?;
        validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
        let drive_refs = body
            .drive_refs
            .into_iter()
            .map(AgentItemDriveRefBody::into_input)
            .collect::<Result<Vec<_>, _>>()?;
        let command = CreateTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            turn_id: body.turn_id,
            content: body.content,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            turn_mode: crate::agent_turn::AgentTurnMode::from_code(&body.turn_mode)
                .ok_or_else(|| ApiProblem::validation("invalid turnMode"))?,
            system_prompt: body.system_prompt,
            runtime_binding_id: body.runtime_binding_id,
            requested_model_id: body.requested_model_id,
            access_mode_id: body.access_mode_id,
            idempotency_key: body.idempotency_key,
            payload_hash: body.payload_hash,
            client_request_id: body.client_request_id,
            drive_refs,
            owner_scope,
            requested_by: scope.subject,
            requested_at: body.requested_at,
            prefer_stream: stream_requested,
            // Transient auth token for cloudrouter account-pool routing; the
            // bearer value is never persisted on the turn record.
            auth_token: extract_bearer_auth_token(&headers),
            access_token: extract_access_token(&headers),
        };
        execute_turn_http_response(
            &state,
            &web_ctx,
            command,
            stream_requested,
            rich_events_requested,
        )
        .await
    }
    .await;
    crate::response::finish_api_response(&web_ctx, result)
}

async fn app_get_session_item(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionItemRecordDto>> = async {
        let Path((agent_id, session_id, item_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetSessionItemCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            item_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_session_item_with_drive_refs(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentSessionItemRecordDto::from_record_with_drive_refs(
                &record.item,
                &record.drive_refs,
            )
            .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_turn(command)).await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_cancel_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AppCancelTurnBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        validate_requested_at(&body.requested_at).map_err(ApiProblem::from_kernel_error)?;
        let scope = RequestScope::from_context(context);
        let command = CancelTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            expected_version: Some(
                parse_expected_version(&body.expected_version)
                    .map_err(ApiProblem::from_kernel_error)?,
            ),
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_owned_service_timeout(
            &state,
            TURN_CANCEL_SERVICE_TIMEOUT,
            move |service| service.cancel_turn(command),
        )
        .await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_turn_input_queue_entries(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentTurnInputQueueEntryResponse>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let command = ListTurnInputQueueEntriesCommand {
            query: TurnInputQueueListQuery::for_session(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
            )
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            ),
            path_agent_id: agent_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let records = with_service(&state, move |service| {
            service.list_turn_input_queue_entries(command)
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTurnInputQueueEntryResponse::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_turn_input_queue_entry(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppCreateTurnInputQueueEntryBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnInputQueueEntryResponse>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let drive_refs = body
            .drive_refs
            .into_iter()
            .map(AgentItemDriveRefBody::into_queue_ref)
            .collect::<Result<Vec<_>, _>>()?;
        let command = CreateTurnInputQueueEntryCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            queue_entry_id: body.queue_entry_id,
            content: body.content,
            display_text: body.display_text,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            attachment_names: body.attachment_names,
            drive_refs,
            turn_mode: crate::agent_turn::AgentTurnMode::from_code(&body.turn_mode)
                .ok_or_else(|| ApiProblem::validation("invalid turnMode"))?,
            runtime_binding_id: body.runtime_binding_id,
            requested_model_id: body.requested_model_id,
            access_mode_id: body.access_mode_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_service(&state, move |service| {
            service.create_turn_input_queue_entry(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentTurnInputQueueEntryResponse::from_record(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_update_turn_input_queue_entry(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AppUpdateTurnInputQueueEntryBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnInputQueueEntryResponse>> = async {
        let Path((agent_id, session_id, queue_entry_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let drive_refs = body
            .drive_refs
            .into_iter()
            .map(AgentItemDriveRefBody::into_queue_ref)
            .collect::<Result<Vec<_>, _>>()?;
        let command = UpdateTurnInputQueueEntryCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            queue_entry_id,
            content: body.content,
            display_text: body.display_text,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            attachment_names: body.attachment_names,
            drive_refs,
            turn_mode: crate::agent_turn::AgentTurnMode::from_code(&body.turn_mode)
                .ok_or_else(|| ApiProblem::validation("invalid turnMode"))?,
            runtime_binding_id: body.runtime_binding_id,
            requested_model_id: body.requested_model_id,
            access_mode_id: body.access_mode_id,
            expected_version: parse_expected_version(&body.expected_version)
                .map_err(ApiProblem::from_kernel_error)?,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_service(&state, move |service| {
            service.update_turn_input_queue_entry(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentTurnInputQueueEntryResponse::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_remove_turn_input_queue_entry(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    query: Result<Query<AppRemoveTurnInputQueueEntryQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path((agent_id, session_id, queue_entry_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = RemoveTurnInputQueueEntryCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            queue_entry_id,
            expected_version: parse_expected_version(&query.expected_version)
                .map_err(ApiProblem::from_kernel_error)?,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        with_service(&state, move |service| {
            service.remove_turn_input_queue_entry(command)
        })
        .await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_clear_turn_input_queue_entries(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ClearTurnInputQueueEntriesResponse> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = ClearTurnInputQueueEntriesCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let cleared_count = with_service(&state, move |service| {
            service.clear_turn_input_queue_entries(command)
        })
        .await?;
        Ok(ClearTurnInputQueueEntriesResponse {
            cleared_count: cleared_count.to_string(),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_reorder_turn_input_queue_entries(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppReorderTurnInputQueueEntriesBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ReorderTurnInputQueueEntriesResponse> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let entries = body
            .ordered_entries
            .into_iter()
            .map(|entry| {
                Ok(TurnInputQueueReorderEntry {
                    queue_entry_id: entry.queue_entry_id,
                    expected_version: parse_expected_version(&entry.expected_version)
                        .map_err(ApiProblem::from_kernel_error)?,
                })
            })
            .collect::<Result<Vec<_>, ApiProblem>>()?;
        let command = ReorderTurnInputQueueEntriesCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            entries,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let records = with_service(&state, move |service| {
            service.reorder_turn_input_queue_entries(command)
        })
        .await?;
        Ok(ReorderTurnInputQueueEntriesResponse {
            items: records
                .iter()
                .map(AgentTurnInputQueueEntryResponse::from_record)
                .collect(),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_claim_next_turn_input_queue_entry(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppClaimNextTurnInputQueueEntryBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ClaimNextTurnInputQueueEntryResponse> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = ClaimNextTurnInputQueueEntryCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            claim_owner: body.claim_owner,
            lease_seconds: body.lease_seconds,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let outcome = with_service(&state, move |service| {
            service.claim_next_turn_input_queue_entry(command)
        })
        .await?;
        Ok(match outcome {
            crate::application::ClaimNextTurnInputQueueEntryResult::Claimed {
                entry,
                claim_token,
            } => ClaimNextTurnInputQueueEntryResponse {
                outcome: "claimed".to_string(),
                entry: Some(AgentTurnInputQueueEntryResponse::from_record(&entry)),
                claim_token: Some(claim_token),
            },
            crate::application::ClaimNextTurnInputQueueEntryResult::Busy(entry) => {
                ClaimNextTurnInputQueueEntryResponse {
                    outcome: "busy".to_string(),
                    entry: Some(AgentTurnInputQueueEntryResponse::from_record(&entry)),
                    claim_token: None,
                }
            }
            crate::application::ClaimNextTurnInputQueueEntryResult::Blocked(entry) => {
                ClaimNextTurnInputQueueEntryResponse {
                    outcome: "blocked".to_string(),
                    entry: Some(AgentTurnInputQueueEntryResponse::from_record(&entry)),
                    claim_token: None,
                }
            }
            crate::application::ClaimNextTurnInputQueueEntryResult::ActiveTurn => {
                ClaimNextTurnInputQueueEntryResponse {
                    outcome: "active_turn".to_string(),
                    entry: None,
                    claim_token: None,
                }
            }
            crate::application::ClaimNextTurnInputQueueEntryResult::Empty => {
                ClaimNextTurnInputQueueEntryResponse {
                    outcome: "empty".to_string(),
                    entry: None,
                    claim_token: None,
                }
            }
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_fail_turn_input_queue_entry(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AppFailTurnInputQueueEntryBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnInputQueueEntryResponse>> = async {
        let Path((agent_id, session_id, queue_entry_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = FailTurnInputQueueEntryCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            queue_entry_id,
            expected_version: parse_expected_version(&body.expected_version)
                .map_err(ApiProblem::from_kernel_error)?,
            fencing_token: body
                .fencing_token
                .parse::<u64>()
                .map_err(|_| ApiProblem::validation("fencingToken must be int64 string"))?,
            claim_token: body.claim_token,
            error_code: body.error_code,
            error_detail: body.error_detail,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_service(&state, move |service| {
            service.fail_turn_input_queue_entry(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentTurnInputQueueEntryResponse::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_retry_turn_input_queue_entry(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AppRetryTurnInputQueueEntryBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnInputQueueEntryResponse>> = async {
        let Path((agent_id, session_id, queue_entry_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = RetryTurnInputQueueEntryCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            queue_entry_id,
            expected_version: parse_expected_version(&body.expected_version)
                .map_err(ApiProblem::from_kernel_error)?,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_service(&state, move |service| {
            service.retry_turn_input_queue_entry(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentTurnInputQueueEntryResponse::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn list_session_checkpoints_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> ApiResult<PageData<AgentSessionCheckpointRecordDto>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let command = ListSessionCheckpointsCommand {
        query: SessionCheckpointListQuery::for_session(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            session_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        ),
        path_agent_id: agent_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let records = with_service(state, move |service| {
        service.list_session_checkpoints(command)
    })
    .await?;
    Ok(PageData {
        items: records
            .items
            .iter()
            .map(AgentSessionCheckpointRecordDto::from_record)
            .collect(),
        page_info: offset_page_info(
            page,
            page_size,
            records.total_count.unwrap_or(0),
            records.has_more,
        ),
    })
}

async fn create_session_checkpoint_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    body: CreateSessionCheckpointRequestDto,
) -> ApiResult<ResourceData<AgentSessionCheckpointRecordDto>> {
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| {
        service.create_session_checkpoint(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionCheckpointRecordDto::from_record(&record),
    })
}

async fn get_session_checkpoint_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    checkpoint_id: String,
    owner_scope: Option<u64>,
) -> ApiResult<ResourceData<AgentSessionCheckpointRecordDto>> {
    let command = GetSessionCheckpointCommand {
        tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
        organization_id: parse_organization_id(&scope.organization_id)
            .map_err(ApiProblem::from_kernel_error)?,
        path_agent_id: agent_id,
        session_id,
        checkpoint_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let record = with_service(state, move |service| {
        service.get_session_checkpoint(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionCheckpointRecordDto::from_record(&record),
    })
}

enum SessionCheckpointTransition {
    Restore,
    Invalidate,
}

struct ChangeSessionCheckpointInput {
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    checkpoint_id: String,
    owner_scope: Option<u64>,
    body: ChangeSessionCheckpointStatusRequestDto,
    transition: SessionCheckpointTransition,
}

async fn change_session_checkpoint_data(
    state: &AgentHttpState,
    input: ChangeSessionCheckpointInput,
) -> ApiResult<ResourceData<AgentSessionCheckpointRecordDto>> {
    let ChangeSessionCheckpointInput {
        scope,
        agent_id,
        session_id,
        checkpoint_id,
        owner_scope,
        body,
        transition,
    } = input;
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            checkpoint_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| match transition {
        SessionCheckpointTransition::Restore => service.restore_session_checkpoint(command),
        SessionCheckpointTransition::Invalidate => service.invalidate_session_checkpoint(command),
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionCheckpointRecordDto::from_record(&record),
    })
}

async fn list_session_runtime_bindings_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> ApiResult<PageData<AgentSessionRuntimeBindingRecordDto>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let command = ListSessionRuntimeBindingsCommand {
        query: SessionRuntimeBindingListQuery::for_session(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            session_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        ),
        path_agent_id: agent_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let records = with_service(state, move |service| {
        service.list_session_runtime_bindings(command)
    })
    .await?;
    Ok(PageData {
        items: records
            .items
            .iter()
            .map(AgentSessionRuntimeBindingRecordDto::from_record)
            .collect(),
        page_info: offset_page_info(
            page,
            page_size,
            records.total_count.unwrap_or(0),
            records.has_more,
        ),
    })
}

async fn create_session_runtime_binding_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    body: CreateSessionRuntimeBindingRequestDto,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| {
        service.create_session_runtime_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

async fn get_session_runtime_binding_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    runtime_binding_id: String,
    owner_scope: Option<u64>,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let command = GetSessionRuntimeBindingCommand {
        tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
        organization_id: parse_organization_id(&scope.organization_id)
            .map_err(ApiProblem::from_kernel_error)?,
        path_agent_id: agent_id,
        session_id,
        runtime_binding_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let record = with_service(state, move |service| {
        service.get_session_runtime_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

async fn update_session_runtime_binding_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    runtime_binding_id: String,
    owner_scope: Option<u64>,
    body: UpdateSessionRuntimeBindingRequestDto,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            runtime_binding_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| {
        service.update_session_runtime_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

enum SessionRuntimeBindingTransition {
    Activate,
    Deactivate,
}

struct ChangeSessionRuntimeBindingInput {
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    runtime_binding_id: String,
    owner_scope: Option<u64>,
    body: ChangeSessionRuntimeBindingStatusRequestDto,
    transition: SessionRuntimeBindingTransition,
}

async fn change_session_runtime_binding_data(
    state: &AgentHttpState,
    input: ChangeSessionRuntimeBindingInput,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let ChangeSessionRuntimeBindingInput {
        scope,
        agent_id,
        session_id,
        runtime_binding_id,
        owner_scope,
        body,
        transition,
    } = input;
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            runtime_binding_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| match transition {
        SessionRuntimeBindingTransition::Activate => {
            service.activate_session_runtime_binding(command)
        }
        SessionRuntimeBindingTransition::Deactivate => {
            service.deactivate_session_runtime_binding(command)
        }
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

async fn app_list_session_checkpoints(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        list_session_checkpoints_data(
            &state,
            scope,
            agent_id,
            session_id,
            owner_scope,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateSessionCheckpointRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        create_session_checkpoint_data(&state, scope, agent_id, session_id, owner_scope, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        get_session_checkpoint_data(
            &state,
            scope,
            agent_id,
            session_id,
            checkpoint_id,
            owner_scope,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_restore_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope,
                body,
                transition: SessionCheckpointTransition::Restore,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_invalidate_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope,
                body,
                transition: SessionCheckpointTransition::Invalidate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_session_runtime_bindings(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        list_session_runtime_bindings_data(
            &state,
            scope,
            agent_id,
            session_id,
            owner_scope,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        create_session_runtime_binding_data(&state, scope, agent_id, session_id, owner_scope, body)
            .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        get_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            owner_scope,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<UpdateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        update_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            owner_scope,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_activate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope,
                body,
                transition: SessionRuntimeBindingTransition::Activate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_deactivate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope,
                body,
                transition: SessionRuntimeBindingTransition::Deactivate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Session handlers  - Backend API
// ===========================================================================

async fn backend_list_sessions(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<ListSessionsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_session_list_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: None,
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_organization(
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
            )
            .for_agent(agent_id)
            .with_cursor_page(page_size, cursor);
        let records = with_service(&state, move |service| service.list_sessions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentSessionRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_session(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetSessionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_close_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CloseSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.close_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_archive_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<ArchiveSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.archive_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Session item and turn handlers  - Backend API
// ===========================================================================

async fn backend_list_session_items(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<ListItemsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionItemRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        // High-volume ordered item feed: keyset cursor (opaque, scope-bound),
        // never deep OFFSET (PAGINATION_SPEC: cursor mode for P1 lists).
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_session_item_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListSessionItemsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
            sort: query.sort,
        }
        .into_command(agent_id, session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = None;
        command.query = command.query.with_cursor_page(page_size, cursor);
        let records = with_service(&state, move |service| {
            service.list_session_items_with_drive_refs(command)
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(|item| {
                    AgentSessionItemRecordDto::from_record_with_drive_refs(
                        &item.item,
                        &item.drive_refs,
                    )
                    .map_err(ApiProblem::from_kernel_error)
                })
                .collect::<Result<Vec<_>, _>>()?,
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_turns(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppTurnsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(|value| decode_created_at_cursor(value, "turn"))
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let command = ListTurnsCommand {
            query: TurnListQuery::for_session(
                parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
            )
            .with_cursor_page(page_size, cursor),
            path_agent_id: agent_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let records = with_service(&state, move |service| service.list_turns(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTurnRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    headers: HeaderMap,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<BackendCreateTurnQueryParams>, QueryRejection>,
    body: Result<Json<CreateTurnBody>, JsonRejection>,
) -> Response {
    let result: Result<Response, ApiProblem> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let stream_requested = query.stream.unwrap_or(false);
        let rich_events_requested =
            resolve_rich_turn_event_protocol(stream_requested, query.event_protocol.as_deref())?;
        validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
        let drive_refs = body
            .drive_refs
            .into_iter()
            .map(AgentItemDriveRefBody::into_input)
            .collect::<Result<Vec<_>, _>>()?;
        let command = CreateTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            turn_id: body.turn_id,
            content: body.content,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            turn_mode: crate::agent_turn::AgentTurnMode::from_code(&body.turn_mode)
                .ok_or_else(|| ApiProblem::validation("invalid turnMode"))?,
            system_prompt: body.system_prompt,
            runtime_binding_id: body.runtime_binding_id,
            requested_model_id: body.requested_model_id,
            access_mode_id: None,
            idempotency_key: body.idempotency_key,
            payload_hash: body.payload_hash,
            client_request_id: body.client_request_id,
            drive_refs,
            owner_scope: None,
            requested_by: scope.subject,
            requested_at: body.requested_at,
            prefer_stream: stream_requested,
            // Transient auth token for cloudrouter account-pool routing; the
            // bearer value is never persisted on the turn record.
            auth_token: extract_bearer_auth_token(&headers),
            access_token: extract_access_token(&headers),
        };
        execute_turn_http_response(
            &state,
            &web_ctx,
            command,
            stream_requested,
            rich_events_requested,
        )
        .await
    }
    .await;
    crate::response::finish_api_response(&web_ctx, result)
}

async fn backend_get_session_item(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionItemRecordDto>> = async {
        let Path((agent_id, session_id, item_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetSessionItemCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            item_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_session_item_with_drive_refs(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentSessionItemRecordDto::from_record_with_drive_refs(
                &record.item,
                &record.drive_refs,
            )
            .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_get_turn(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_turn(command)).await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_cancel_turn(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AppCancelTurnBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = CancelTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            expected_version: Some(
                parse_expected_version(&body.expected_version)
                    .map_err(ApiProblem::from_kernel_error)?,
            ),
            owner_scope: None,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_service(&state, move |service| service.cancel_turn(command)).await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_session_checkpoints(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        list_session_checkpoints_data(
            &state,
            scope,
            agent_id,
            session_id,
            None,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CreateSessionCheckpointRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        create_session_checkpoint_data(&state, scope, agent_id, session_id, None, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        get_session_checkpoint_data(&state, scope, agent_id, session_id, checkpoint_id, None).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_restore_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope: None,
                body,
                transition: SessionCheckpointTransition::Restore,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_invalidate_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope: None,
                body,
                transition: SessionCheckpointTransition::Invalidate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_session_runtime_bindings(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        list_session_runtime_bindings_data(
            &state,
            scope,
            agent_id,
            session_id,
            None,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CreateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        create_session_runtime_binding_data(&state, scope, agent_id, session_id, None, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        get_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            None,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        update_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            None,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_activate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope: None,
                body,
                transition: SessionRuntimeBindingTransition::Activate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_deactivate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope: None,
                body,
                transition: SessionRuntimeBindingTransition::Deactivate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Task handlers  - Backend API
// ===========================================================================

async fn backend_list_tasks(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<ListTasksQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(decode_task_cursor)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListTasksRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            owner_user_id: Some(scope.owner_user_id),
            status: query.status,
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_agent(agent_id)
            .with_cursor_page(page_size, cursor);
        let records = with_service(&state, move |service| service.list_tasks(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTaskRecordDto::from_record)
                .collect(),
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_task(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetTaskCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            task_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_cancel_task(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CancelTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.cancel_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_execute_task(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ExecuteTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRunRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                agent_id,
                task_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let run = with_service(&state, move |service| service.execute_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRunRecordDto::from_record(&run),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Interaction handlers  - Backend API
// ===========================================================================

async fn backend_list_interactions(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<ListInteractionsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let page_size = normalized_cursor_page_size(query.page_size)?;
        let cursor = query
            .cursor
            .as_deref()
            .map(|value| decode_created_at_cursor(value, "interaction"))
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let mut command = ListInteractionsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.path_agent_id = agent_id;
        command.query = command.query.with_cursor_page(page_size, cursor);
        let records =
            with_service(&state, move |service| service.list_interactions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentInteractionRecordDto::from_record)
                .collect::<KernelResult<Vec<_>>>()
                .map_err(ApiProblem::from_kernel_error)?,
            page_info: sdkwork_utils_rust::http_api::cursor_window_page_info(
                Some(page_size),
                records.next_page_token,
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.create_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetInteractionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            interaction_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_claim_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ClaimInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<InteractionClaimResultDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let claim = with_service(&state, move |service| service.claim_interaction(command)).await?;
        Ok(ResourceData {
            item: InteractionClaimResultDto::from_result(&claim)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_approve_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ApproveInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.approve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_answer_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AnswerInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.answer_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_resolve_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ResolveInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.resolve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

/// Invoke a service operation without Mutex locking.
///
/// Since `AgentsService`, `AgentRepository`, and `AgentAuditSink` are all
/// `&self`-based (Send + Sync), there is no need for a global Mutex. This
/// function simply calls the action directly, enabling true concurrent request
/// processing.
pub(crate) async fn with_service<T>(
    state: &AgentHttpState,
    action: impl FnOnce(&HttpService) -> KernelResult<T> + Send + 'static,
) -> Result<T, ApiProblem>
where
    T: Send + 'static,
{
    with_owned_service(state, move |service| action(service.as_ref())).await
}

async fn with_owned_service<T>(
    state: &AgentHttpState,
    action: impl FnOnce(Arc<HttpService>) -> KernelResult<T> + Send + 'static,
) -> Result<T, ApiProblem>
where
    T: Send + 'static,
{
    let permit = SERVICE_WORKER_LIMIT
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            crate::infrastructure::AgentMetricsRegistry::global().record_service_worker_rejection();
            ApiProblem::too_many_requests("agents service concurrency limit reached", Some(1))
        })?;
    let service = Arc::clone(&state.service);
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        action(service)
    })
    .await
    .map_err(|error| ApiProblem::internal(format!("agents service worker failed: {error}")))?
    .map_err(ApiProblem::from_kernel_error)
}

/// Executes a service action with a hard wall-clock bound.
///
/// Used for operations that synchronously call into provider machinery
/// (e.g. turn cancellation) where a hung provider must never leave the HTTP
/// request suspended indefinitely. The worker keeps running after the bound
/// (results are dropped); only the client wait is bounded.
async fn with_owned_service_timeout<T>(
    state: &AgentHttpState,
    timeout: std::time::Duration,
    action: impl FnOnce(Arc<HttpService>) -> KernelResult<T> + Send + 'static,
) -> Result<T, ApiProblem>
where
    T: Send + 'static,
{
    let permit = SERVICE_WORKER_LIMIT
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            crate::infrastructure::AgentMetricsRegistry::global().record_service_worker_rejection();
            ApiProblem::too_many_requests("agents service concurrency limit reached", Some(1))
        })?;
    let service = Arc::clone(&state.service);
    tokio::time::timeout(
        timeout,
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            action(service)
        }),
    )
    .await
    .map_err(|_| ApiProblem::gateway_timeout("agents service operation timed out"))?
    .map_err(|error| ApiProblem::internal(format!("agents service worker failed: {error}")))?
    .map_err(ApiProblem::from_kernel_error)
}
async fn execute_list(
    state: AgentHttpState,
    query: ListAgentsQueryParams,
    scope: RequestScope,
    owner_scoped: bool,
) -> ApiResult<PageData<AgentRecordResponse>> {
    let include_deleted = query.include_deleted.unwrap_or(false);
    let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
    let visibility = if matches!(
        query.scope.as_deref(),
        Some("market" | "public" | "published")
    ) {
        Some("public".to_string())
    } else {
        None
    };
    let request_dto = ListAgentsRequestDto {
        tenant_id: scope.tenant_id,
        organization_id: Some(scope.organization_id),
        owner_user_id: if owner_scoped && visibility.is_none() {
            Some(scope.owner_user_id)
        } else {
            None
        },
        include_deleted,
        search_query: query.q,
        visibility,
        pagination: PaginationParams::default()
            .with_page_size(page_size)
            .with_page(page),
    };
    let command = request_dto
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;

    let result = with_service(&state, move |service| service.list_agents(command)).await?;
    let total_items = result.total_count.unwrap_or(0) as usize;
    let total_pages = total_pages(total_items, page_size);

    let items: Vec<AgentRecordResponse> = result
        .items
        .iter()
        .map(|record| map_agent_record(&AgentRecordDto::from_record(record)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page as i32),
            page_size: Some(page_size as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(total_pages as i32),
            next_cursor: None,
            has_more: Some(result.has_more),
        },
    })
}

async fn execute_create(
    state: AgentHttpState,
    scope: RequestScope,
    body: CreateAgentBody,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let manifest = parse_manifest(body.manifest)?;
    let mut default_code_task_intent = body.default_code_task_intent.map(Into::into);
    if let Some(management_profile) = body.management_profile {
        default_code_task_intent = management_profile
            .into_validated_dto()?
            .merge_into_default_code_task_intent(default_code_task_intent)
            .map_err(ApiProblem::from_kernel_error)?;
    }

    let command = CreateAgentRequestDto {
        agent_id: body.agent_id,
        tenant_id: scope.tenant_id,
        organization_id: scope.organization_id,
        owner_user_id: scope.owner_user_id,
        code: body.code,
        display_name: body.display_name,
        description: body.description,
        manifest,
        visibility: body.visibility,
        tags: body.tags.unwrap_or_default(),
        default_code_task_intent,
        implementation_provider_id: body.implementation_provider_id,
        implementation_kind: body.implementation_kind,
        implementation_type: body.implementation_type,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.create_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_get(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let command = GetAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.get_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_update(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: UpdateAgentBody,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let mut default_code_task_intent = body.default_code_task_intent.map(Into::into);
    if let Some(management_profile) = body.management_profile {
        let base_intent = match default_code_task_intent.take() {
            Some(intent) => Some(intent),
            None => {
                let command = GetAgentRequestDto {
                    tenant_id: scope.tenant_id.clone(),
                    agent_id: agent_id.clone(),
                }
                .into_command(scope.subject.clone())
                .map_err(ApiProblem::from_kernel_error)?;
                let current =
                    with_service(&state, move |service| service.get_agent(command)).await?;
                current.default_code_task_intent
            }
        };
        default_code_task_intent = management_profile
            .into_validated_dto()?
            .merge_into_default_code_task_intent(base_intent)
            .map_err(ApiProblem::from_kernel_error)?;
    }
    let command = UpdateAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: body.expected_version,
        display_name: body.display_name,
        description: body.description,
        manifest: body.manifest.map(parse_manifest).transpose()?,
        visibility: body.visibility,
        tags: body.tags,
        default_code_task_intent,
        implementation_provider_id: body.implementation_provider_id,
        implementation_kind: body.implementation_kind,
        implementation_type: body.implementation_type,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.update_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_delete(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
) -> ApiResult<()> {
    let command = DeleteAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: None,
        requested_at: server_requested_at(),
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    with_service(&state, move |service| service.delete_agent(command)).await?;
    Ok(())
}

async fn execute_restore(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: RestoreAgentBody,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let command = RestoreAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: body.expected_version,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.restore_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_list_provider_bindings(
    state: AgentHttpState,
    scope: RequestScope,
    page: Option<usize>,
    page_size: Option<usize>,
    agent_id: String,
) -> ApiResult<PageData<AgentProviderBindingRecordResponse>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let tenant_id = scope.tenant_id_u64()?;
    let subject = scope.subject.clone();
    let result = with_service(&state, move |service| {
        service.list_provider_bindings(ProviderBindingListCommand {
            query: ProviderBindingListQuery::for_agent(tenant_id, agent_id).with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            ),
            requested_by: subject,
        })
    })
    .await?;
    let total_items = result.total_count.unwrap_or(0) as usize;
    let items = result
        .items
        .iter()
        .map(|record| {
            map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(record))
        })
        .collect();
    let total_pages = total_pages(total_items, page_size);

    Ok(PageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page as i32),
            page_size: Some(page_size as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(total_pages as i32),
            next_cursor: None,
            has_more: Some(result.has_more),
        },
    })
}

async fn execute_add_provider_binding(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentProviderBindingBody,
) -> ApiResult<ResourceData<AgentProviderBindingRecordResponse>> {
    let command = AgentProviderBindingRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        binding_id: body.binding_id,
        provider_id: body.provider_id,
        implementation_kind: body.implementation_kind,
        configuration_profile_id: body.configuration_profile_id,
        capabilities: body.capabilities.unwrap_or_default(),
        make_default: body.make_default.unwrap_or(false),
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.add_provider_binding(command)).await?;
    Ok(ResourceData {
        item: map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(&record)),
    })
}

async fn execute_activate_provider_binding(
    state: AgentHttpState,
    scope: RequestScope,
    path: TenantAgentBindingPathParams,
    body: ActivateProviderBindingBody,
) -> ApiResult<ResourceData<AgentProviderBindingRecordResponse>> {
    let command = ActivateAgentProviderBindingRequestDto {
        tenant_id: scope.tenant_id,
        agent_id: path.agent_id,
        binding_id: path.binding_id,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| {
        service.activate_provider_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(&record)),
    })
}

async fn execute_create_preview_response(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentPreviewResponseBody,
) -> ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> {
    let input_payload_json = json_value_to_string(
        body.input_payload
            .unwrap_or_else(|| json!({ "content": body.content })),
        "inputPayload",
    )?;
    let command = AgentPreviewResponseRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        execution_id: body.execution_id,
        content: body.content,
        debug_mode: body.debug_mode.unwrap_or(false),
        model: body.model,
        temperature: body.temperature,
        input_payload_json,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| {
        service.create_preview_response(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_runtime_execution_record(&AgentRuntimeExecutionRecordDto::from_record(&record))?,
    })
}

async fn execute_create_prompt_optimization(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentPromptOptimizationBody,
) -> ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> {
    let input_payload_json = json_value_to_string(
        body.input_payload
            .unwrap_or_else(|| json!({ "prompt": body.prompt })),
        "inputPayload",
    )?;
    let command = AgentPromptOptimizationRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        execution_id: body.execution_id,
        prompt: body.prompt,
        input_payload_json,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| {
        service.create_prompt_optimization(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_runtime_execution_record(&AgentRuntimeExecutionRecordDto::from_record(&record))?,
    })
}

fn parse_manifest(value: Value) -> Result<AgentManifest, ApiProblem> {
    let json_string = serde_json::to_string(&value)
        .map_err(|error| ApiProblem::validation(format!("manifest json encode failed: {error}")))?;
    AgentManifest::from_json(json_string.as_str()).map_err(ApiProblem::from_kernel_error)
}

fn map_agent_record(record: &AgentRecordDto) -> Result<AgentRecordResponse, ApiProblem> {
    let manifest_value = manifest_to_value(&record.manifest)?;
    let default_code_task_intent = record
        .default_code_task_intent
        .as_ref()
        .map(intent_to_value);

    Ok(AgentRecordResponse {
        id: record.id.clone(),
        agent_id: record.agent_id.clone(),
        tenant_id: record.tenant_id.clone(),
        organization_id: record.organization_id.clone(),
        owner_user_id: record.owner_user_id.clone(),
        code: record.code.clone(),
        display_name: record.display_name.clone(),
        description: record.description.clone(),
        manifest: manifest_value,
        default_code_task_intent,
        management_profile: record
            .management_profile
            .as_ref()
            .map(map_agent_management_profile),
        implementation_provider_id: record.implementation_provider_id.clone(),
        implementation_kind: record.implementation_kind.clone(),
        implementation_type: record.implementation_type.clone(),
        status: record.status.clone(),
        visibility: record.visibility.clone(),
        tags: record.tags.clone(),
        version: record.version.clone(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        deleted_at: record.deleted_at.clone(),
    })
}

fn map_agent_management_profile(
    profile: &AgentManagementProfileDto,
) -> AgentManagementProfileResponse {
    AgentManagementProfileResponse {
        author: profile.author.clone(),
        avatar: profile.avatar.clone(),
        category_id: profile.category_id.clone(),
        color: profile.color.clone(),
        debug_mode: profile.debug_mode,
        icon_name: profile.icon_name.clone(),
        json_mode: profile.json_mode,
        knowledge_base_ids: profile.knowledge_base_ids.clone(),
        memory_enabled: profile.memory_enabled,
        model: profile.model.clone(),
        skill_ids: profile.skill_ids.clone(),
        suggested_prompts: profile.suggested_prompts.clone(),
        system_prompt: profile.system_prompt.clone(),
        temperature: profile.temperature,
        tool_ids: profile.tool_ids.clone(),
        agent_type: profile.agent_type.clone(),
        users: profile.users.clone(),
        voice_ids: profile.voice_ids.clone(),
        welcome_message: profile.welcome_message.clone(),
    }
}

fn map_provider_binding_record(
    record: &AgentProviderBindingRecordDto,
) -> AgentProviderBindingRecordResponse {
    AgentProviderBindingRecordResponse {
        tenant_id: record.tenant_id.clone(),
        agent_id: record.agent_id.clone(),
        binding_id: record.binding_id.clone(),
        provider_id: record.provider_id.clone(),
        implementation_kind: record.implementation_kind.clone(),
        configuration_profile_id: record.configuration_profile_id.clone(),
        capabilities: record.capabilities.clone(),
        active: record.active,
        version: record.version.clone(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn map_runtime_execution_record(
    record: &AgentRuntimeExecutionRecordDto,
) -> Result<AgentRuntimeExecutionRecordResponse, ApiProblem> {
    Ok(AgentRuntimeExecutionRecordResponse {
        tenant_id: record.tenant_id.clone(),
        agent_id: record.agent_id.clone(),
        execution_id: record.execution_id.clone(),
        operation: record.operation.clone(),
        status: record.status.clone(),
        input_payload: json_string_to_value(record.input_payload_json.as_str(), "inputPayload")?,
        output_payload: json_string_to_value(record.output_payload_json.as_str(), "outputPayload")?,
        requested_at: record.requested_at.clone(),
        completed_at: record.completed_at.clone(),
    })
}

fn manifest_to_value(manifest: &AgentManifest) -> Result<Value, ApiProblem> {
    let value = json!({
        "schema_version": manifest.schema_version,
        "manifest_type": manifest.manifest_type,
        "agent_id": manifest.agent_id,
        "name": manifest.name,
        "display_name": manifest.display_name,
        "description": manifest.description,
        "version": manifest.version,
        "domain": manifest.domain,
        "required_capabilities": manifest.required_capabilities,
        "optional_capabilities": manifest.optional_capabilities,
        "event_families": manifest.event_families,
        "owner": {
            "name": manifest.owner_name,
        },
        "status": manifest.status,
    });
    serde_json::from_value(value)
        .map_err(|error| ApiProblem::internal(format!("manifest json decode failed: {error}")))
}

fn intent_to_value(intent: &CodeTaskIntent) -> Value {
    json!({
        "prompt": intent.prompt,
        "contextPaths": intent.context_paths,
        "constraints": intent.constraints,
    })
}

fn json_value_to_string(value: Value, field_name: &str) -> Result<String, ApiProblem> {
    serde_json::to_string(&value).map_err(|error| {
        ApiProblem::validation(format!("{field_name} json encode failed: {error}"))
    })
}

fn json_string_to_value(value: &str, field_name: &str) -> Result<Value, ApiProblem> {
    serde_json::from_str(value)
        .map_err(|error| ApiProblem::internal(format!("{field_name} json decode failed: {error}")))
}

fn kernel_event_severity(severity: sdkwork_agent_kernel::KernelEventSeverity) -> &'static str {
    match severity {
        sdkwork_agent_kernel::KernelEventSeverity::Debug => "debug",
        sdkwork_agent_kernel::KernelEventSeverity::Info => "info",
        sdkwork_agent_kernel::KernelEventSeverity::Warn => "warn",
        sdkwork_agent_kernel::KernelEventSeverity::Error => "error",
    }
}

fn validate_audit_action_filter(action: Option<&str>) -> Result<(), ApiProblem> {
    let Some(action) = action else {
        return Ok(());
    };
    if !ALLOWED_AUDIT_ACTIONS.contains(&action) {
        return Err(ApiProblem::validation(format!(
            "action must be one of {}",
            ALLOWED_AUDIT_ACTIONS.join(", ")
        )));
    }
    Ok(())
}

fn validate_audit_range(from: Option<&str>, to: Option<&str>) -> Result<(), ApiProblem> {
    let from = parse_optional_query_datetime("from", from)?;
    let to = parse_optional_query_datetime("to", to)?;
    if let (Some(from_value), Some(to_value)) = (from.as_ref(), to.as_ref()) {
        if from_value > to_value {
            return Err(ApiProblem::validation(
                "from must be less than or equal to to",
            ));
        }
    }
    Ok(())
}

fn parse_optional_query_datetime(
    field_name: &str,
    value: Option<&str>,
) -> Result<Option<OffsetDateTime>, ApiProblem> {
    parse_optional_rfc3339_datetime(value, field_name).map_err(ApiProblem::from_kernel_error)
}

fn server_requested_at() -> String {
    format_utc_seconds(OffsetDateTime::now_utc())
}

fn format_utc_seconds(value: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        value.year(),
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute(),
        value.second()
    )
}

fn env_usize(key: &str, default: usize, minimum: usize, maximum: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| (minimum..=maximum).contains(value))
        .unwrap_or(default)
}

pub(crate) fn normalized_pagination(
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<(usize, usize), ApiProblem> {
    if page == Some(0) {
        return Err(ApiProblem::validation(
            "page must be greater than or equal to 1",
        ));
    }
    if page_size == Some(0) {
        return Err(ApiProblem::validation(
            "page_size must be greater than or equal to 1",
        ));
    }
    if let Some(size) = page_size {
        if size > MAX_PAGE_SIZE {
            return Err(ApiProblem::validation(format!(
                "page_size must be less than or equal to {MAX_PAGE_SIZE}"
            )));
        }
    }
    if page.is_some_and(|value| value as i64 > sdkwork_utils_rust::http_api::MAX_LIST_PAGE) {
        return Err(ApiProblem::validation(format!(
            "page must be less than or equal to {}",
            sdkwork_utils_rust::http_api::MAX_LIST_PAGE
        )));
    }

    let params = sdkwork_utils_rust::http_api::OffsetListPageParams::parse(
        page.map(|value| value as i64),
        page_size.map(|value| value as i64),
    );
    Ok((params.page as usize, params.page_size as usize))
}

fn normalized_cursor_page_size(page_size: Option<usize>) -> Result<usize, ApiProblem> {
    let page_size = page_size.unwrap_or(crate::ports::DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&page_size) {
        return Err(ApiProblem::validation(format!(
            "page_size must be between 1 and {MAX_PAGE_SIZE}",
        )));
    }
    Ok(page_size)
}

async fn execute_turn_http_response(
    state: &AgentHttpState,
    ctx: &sdkwork_web_core::WebRequestContext,
    command: CreateTurnCommand,
    stream_requested: bool,
    rich_events_requested: bool,
) -> Result<Response, ApiProblem> {
    if !stream_requested {
        let result = with_service(state, move |service| service.execute_turn(command)).await?;
        return turn_execution_http_response(ctx, &result, false, rich_events_requested);
    }

    streaming_turn_execution_http_response(state, ctx, command, rich_events_requested).await
}

#[derive(Debug)]
enum TurnHttpStreamSignal {
    Chunk(String),
    Failed(ApiProblem),
}

#[derive(Debug, PartialEq, Eq)]
struct TurnHttpStreamIdentity {
    session_id: String,
    turn_id: String,
}

struct HttpTurnExecutionStreamSink {
    service: Arc<HttpService>,
    tenant_id: u64,
    organization_id: u64,
    agent_id: String,
    requested_by: PolicySubject,
    requested_at: String,
    sender: mpsc::Sender<TurnHttpStreamSignal>,
    rich_events_requested: bool,
    identity: OnceLock<TurnHttpStreamIdentity>,
    delta_index: AtomicUsize,
    event_sequence: AtomicUsize,
    closed: AtomicBool,
    // Streaming checkpoint (H4): accumulated deltas flushed to the turn row on
    // a size/time throttle so a crash mid-turn retains the partial reply.
    streaming_buffer: std::sync::Mutex<String>,
    streaming_last_persist: std::sync::Mutex<std::time::Instant>,
}

/// Flush the streaming checkpoint after at most this many accumulated bytes.
const TURN_STREAMING_CHECKPOINT_BYTES: usize = 8 * 1024;
/// Flush the streaming checkpoint at least this often while deltas arrive.
const TURN_STREAMING_CHECKPOINT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

impl HttpTurnExecutionStreamSink {
    fn new(
        service: Arc<HttpService>,
        command: &CreateTurnCommand,
        sender: mpsc::Sender<TurnHttpStreamSignal>,
        rich_events_requested: bool,
    ) -> Self {
        Self {
            service,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id.clone(),
            requested_by: command.requested_by.clone(),
            requested_at: command.requested_at.clone(),
            sender,
            rich_events_requested,
            identity: OnceLock::new(),
            delta_index: AtomicUsize::new(0),
            event_sequence: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
            streaming_buffer: std::sync::Mutex::new(String::new()),
            streaming_last_persist: std::sync::Mutex::new(std::time::Instant::now()),
        }
    }

    fn flush_streaming_checkpoint(&self) {
        let Some(identity) = self.identity.get() else {
            return;
        };
        let content = {
            let mut buffer = self.streaming_buffer.lock().expect("streaming buffer lock");
            if buffer.is_empty() {
                return;
            }
            std::mem::take(&mut *buffer)
        };
        *self
            .streaming_last_persist
            .lock()
            .expect("streaming persist lock") = std::time::Instant::now();
        if let Err(error) = self.service.checkpoint_turn_streaming_content(
            self.tenant_id,
            self.organization_id,
            &identity.turn_id,
            &content,
        ) {
            // Checkpoint failures are non-fatal for streaming: the durable
            // completion write remains authoritative. Log for observability.
            tracing::warn!(
                target: "sdkwork.agents.streaming",
                turn_id = %identity.turn_id,
                error = %error,
                "failed to checkpoint streaming content"
            );
        }
    }

    fn send_chunk(&self, chunk: Result<String, ApiProblem>) {
        match chunk {
            Ok(chunk) => {
                if self.closed.load(Ordering::Acquire) {
                    return;
                }
                if self
                    .sender
                    .blocking_send(TurnHttpStreamSignal::Chunk(chunk))
                    .is_err()
                {
                    self.closed.store(true, Ordering::Release);
                }
            }
            Err(problem) => self.fail(problem),
        }
    }

    fn fail(&self, problem: ApiProblem) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let _ = self
            .sender
            .blocking_send(TurnHttpStreamSignal::Failed(problem));
    }
}

impl TurnExecutionStreamSink for HttpTurnExecutionStreamSink {
    fn begin_turn(&self, session_id: &str, turn_id: &str) {
        let identity = TurnHttpStreamIdentity {
            session_id: session_id.to_string(),
            turn_id: turn_id.to_string(),
        };
        if self
            .identity
            .set(TurnHttpStreamIdentity {
                session_id: identity.session_id.clone(),
                turn_id: identity.turn_id.clone(),
            })
            .is_err()
            && self.identity.get() != Some(&identity)
        {
            self.fail(ApiProblem::internal(
                "Turn stream identity changed during execution",
            ));
        }
    }

    fn push_delta(&self, delta: &str) {
        let index = self.delta_index.fetch_add(1, Ordering::AcqRel);
        let mut chunk = String::new();
        let encoded = append_turn_delta_event(&mut chunk, index, delta).map(|_| chunk);
        self.send_chunk(encoded);

        // Streaming checkpoint (H4): accumulate and flush on a size/time
        // throttle so a crash mid-turn retains the partial reply.
        let mut buffer = self.streaming_buffer.lock().expect("streaming buffer lock");
        buffer.push_str(delta);
        let size_threshold_reached = buffer.len() >= TURN_STREAMING_CHECKPOINT_BYTES;
        let interval_elapsed = self
            .streaming_last_persist
            .lock()
            .expect("streaming persist lock")
            .elapsed()
            >= TURN_STREAMING_CHECKPOINT_INTERVAL;
        drop(buffer);
        if size_threshold_reached || interval_elapsed {
            self.flush_streaming_checkpoint();
        }
    }

    fn push_event(&self, event: &sdkwork_agent_kernel::KernelEvent) -> KernelResult<()> {
        let Some(identity) = self.identity.get() else {
            self.fail(ApiProblem::internal(
                "Turn stream event arrived before execution identity was established",
            ));
            return Err(KernelError::Internal {
                message: "Turn stream event arrived before execution identity was established"
                    .to_string(),
            });
        };
        if matches!(
            event.event_type.as_str(),
            "agent.policy.paused" | "agent.message.paused"
        ) {
            self.service.persist_provider_interaction_event(
                PersistProviderInteractionEventCommand {
                    tenant_id: self.tenant_id,
                    organization_id: self.organization_id,
                    path_agent_id: self.agent_id.clone(),
                    session_id: identity.session_id.clone(),
                    turn_id: identity.turn_id.clone(),
                    requested_by: self.requested_by.clone(),
                    received_at: event
                        .occurred_at
                        .clone()
                        .unwrap_or_else(|| self.requested_at.clone()),
                    event: event.clone(),
                },
            )?;
        }
        if !self.rich_events_requested || self.closed.load(Ordering::Acquire) {
            return Ok(());
        }
        let sequence = self.event_sequence.fetch_add(1, Ordering::AcqRel);
        let encoded =
            agent_turn_runtime_event_json(sequence, event, &identity.session_id, &identity.turn_id)
                .and_then(|payload| {
                    let mut chunk = String::new();
                    append_sse_json_event(&mut chunk, "event", &payload, "turn runtime event")?;
                    Ok(chunk)
                });
        self.send_chunk(encoded);
        Ok(())
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Release);
        // Flush any remaining checkpoint so the last deltas are durable even
        // if the consumer disconnects mid-turn.
        self.flush_streaming_checkpoint();
    }
}

async fn streaming_turn_execution_http_response(
    state: &AgentHttpState,
    ctx: &sdkwork_web_core::WebRequestContext,
    command: CreateTurnCommand,
    rich_events_requested: bool,
) -> Result<Response, ApiProblem> {
    let permit = SERVICE_WORKER_LIMIT
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            crate::infrastructure::AgentMetricsRegistry::global().record_service_worker_rejection();
            ApiProblem::too_many_requests("agents service concurrency limit reached", Some(1))
        })?;
    let service = Arc::clone(&state.service);
    let trace_id = ctx.resolved_trace_id();
    let (sender, mut receiver) = mpsc::channel(TURN_STREAM_CHANNEL_CAPACITY);
    let transport_sink = Arc::new(HttpTurnExecutionStreamSink::new(
        Arc::clone(&service),
        &command,
        sender.clone(),
        rich_events_requested,
    ));
    let execution_sink: Arc<dyn TurnExecutionStreamSink> = transport_sink.clone();
    let terminal_sender = sender.clone();
    let completion_trace_id = trace_id.clone();
    let execution = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let result = service.execute_turn_with_stream_sink(command, execution_sink);
        transport_sink.close();
        let signal = match result {
            Ok(result) => turn_completion_sse_chunk(&completion_trace_id, &result)
                .map(TurnHttpStreamSignal::Chunk)
                .unwrap_or_else(TurnHttpStreamSignal::Failed),
            Err(error) => TurnHttpStreamSignal::Failed(ApiProblem::from_kernel_error(error)),
        };
        let _ = terminal_sender.blocking_send(signal);
    });
    tokio::spawn(async move {
        if let Err(error) = execution.await {
            let _ = sender
                .send(TurnHttpStreamSignal::Failed(ApiProblem::internal(format!(
                    "agents service worker failed: {error}"
                ))))
                .await;
        }
    });

    let first = receiver.recv().await.ok_or_else(|| {
        ApiProblem::internal("Turn stream closed before the first event or error")
    })?;
    let first = match first {
        TurnHttpStreamSignal::Failed(problem) => return Err(problem),
        chunk @ TurnHttpStreamSignal::Chunk(_) => chunk,
    };

    // Heartbeat: long inference silences must not look like a dead stream to
    // intermediate proxies. A comment line every 15 seconds is the standard
    // SSE keep-alive and is ignored by event parsers. The stream ends when
    // the receiver closes (terminal completion or disconnect) — the
    // heartbeat must never keep the response open.
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(15));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let first_bytes = match first {
        TurnHttpStreamSignal::Chunk(chunk) => Ok::<Bytes, std::io::Error>(Bytes::from(chunk)),
        TurnHttpStreamSignal::Failed(problem) => Err(std::io::Error::other(problem.message)),
    };
    let body_stream = tokio_stream::iter([first_bytes]).chain(futures_util::stream::unfold(
        (receiver, heartbeat),
        |(mut receiver, mut heartbeat)| async move {
            tokio::select! {
                signal = receiver.recv() => {
                    match signal {
                        Some(TurnHttpStreamSignal::Chunk(chunk)) => Some((
                            Ok::<Bytes, std::io::Error>(Bytes::from(chunk)),
                            (receiver, heartbeat),
                        )),
                        Some(TurnHttpStreamSignal::Failed(problem)) => Some((
                            Err(std::io::Error::other(problem.message)),
                            (receiver, heartbeat),
                        )),
                        None => None,
                    }
                }
                _ = heartbeat.tick() => Some((
                    Ok::<Bytes, std::io::Error>(Bytes::from_static(b": keep-alive

")),
                    (receiver, heartbeat),
                )),
            }
        },
    ));
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/event-stream")
        .body(Body::from_stream(body_stream))
        .map_err(|error| ApiProblem::internal(format!("failed to build SSE response: {error}")))?;
    if let Ok(value) = HeaderValue::from_str(&trace_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-sdkwork-trace-id"), value);
    }
    Ok(response)
}

fn turn_completion_sse_chunk(
    trace_id: &str,
    result: &crate::application::TurnExecutionResult,
) -> Result<String, ApiProblem> {
    let execution =
        AgentTurnExecutionDto::from_result(result).map_err(ApiProblem::from_kernel_error)?;
    let envelope = sdkwork_utils_rust::SdkWorkApiResponse::success(
        ResourceData { item: execution },
        trace_id.to_string(),
    );
    let payload = json!({
        "eventType": "completion",
        "response": envelope,
    });
    let mut chunk = String::new();
    append_sse_json_event(&mut chunk, "completion", &payload, "turn completion")?;
    Ok(chunk)
}

/// Build the durable turn execution response.
/// Non-streaming returns `200 OK` with the SDKWork response envelope.
/// Streaming returns ordered delta events followed by a completion envelope.
fn turn_execution_http_response(
    ctx: &sdkwork_web_core::WebRequestContext,
    result: &crate::application::TurnExecutionResult,
    stream_requested: bool,
    rich_events_requested: bool,
) -> Result<Response, ApiProblem> {
    let execution =
        AgentTurnExecutionDto::from_result(result).map_err(ApiProblem::from_kernel_error)?;
    let trace_id = ctx.resolved_trace_id();

    if stream_requested {
        let mut body = String::new();
        if rich_events_requested {
            append_rich_turn_events(&mut body, result)?;
        } else {
            for (index, delta) in result.stream_deltas.iter().enumerate() {
                append_turn_delta_event(&mut body, index, delta)?;
            }
        }
        body.push_str(&turn_completion_sse_chunk(&trace_id, result)?);
        let mut response = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/event-stream")
            .body(Body::from(body))
            .map_err(|error| {
                ApiProblem::internal(format!("failed to build SSE response: {error}"))
            })?;
        if let Ok(value) = HeaderValue::from_str(&trace_id) {
            response
                .headers_mut()
                .insert(HeaderName::from_static("x-sdkwork-trace-id"), value);
        }
        return Ok(response);
    }

    success_json(ctx, ResourceData { item: execution })
}

const TURN_EVENT_PROTOCOL_KERNEL_V1: &str = "kernel-v1";

fn resolve_rich_turn_event_protocol(
    stream_requested: bool,
    event_protocol: Option<&str>,
) -> Result<bool, ApiProblem> {
    let Some(event_protocol) = event_protocol
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(false);
    };
    if !stream_requested {
        return Err(ApiProblem::validation(
            "event_protocol requires stream=true",
        ));
    }
    if event_protocol != TURN_EVENT_PROTOCOL_KERNEL_V1 {
        return Err(ApiProblem::validation(format!(
            "event_protocol must be {TURN_EVENT_PROTOCOL_KERNEL_V1}",
        )));
    }
    Ok(true)
}

fn append_rich_turn_events(
    body: &mut String,
    result: &crate::application::TurnExecutionResult,
) -> Result<(), ApiProblem> {
    let mut delta_index = 0usize;
    let mut agent_message_text_by_item = HashMap::new();
    for (event_index, event) in result.stream_events.iter().enumerate() {
        let payload = agent_turn_runtime_event_json(
            event_index,
            event,
            &result.session.session_id,
            &result.turn.turn_id,
        )?;
        append_sse_json_event(body, "event", &payload, "turn runtime event")?;
        if let Some(expected_delta) =
            kernel_event_agent_message_delta(event, &mut agent_message_text_by_item)
        {
            if let Some(delta) = result
                .stream_deltas
                .get(delta_index)
                .filter(|delta| delta.as_str() == expected_delta)
            {
                append_turn_delta_event(body, delta_index, delta)?;
                delta_index += 1;
            }
        }
    }
    for (index, delta) in result.stream_deltas.iter().enumerate().skip(delta_index) {
        append_turn_delta_event(body, index, delta)?;
    }
    Ok(())
}

fn append_turn_delta_event(body: &mut String, index: usize, delta: &str) -> Result<(), ApiProblem> {
    append_sse_json_event(
        body,
        "delta",
        &json!({
            "eventType": "delta",
            "index": index,
            "delta": delta,
        }),
        "turn delta",
    )
}

fn append_sse_json_event(
    body: &mut String,
    event_name: &str,
    value: &Value,
    description: &str,
) -> Result<(), ApiProblem> {
    let payload = serde_json::to_string(value).map_err(|error| {
        ApiProblem::internal(format!("failed to encode {description}: {error}"))
    })?;
    body.push_str("event: ");
    body.push_str(event_name);
    body.push('\n');
    body.push_str("data: ");
    body.push_str(&payload);
    body.push_str("\n\n");
    Ok(())
}

fn kernel_event_agent_message_delta(
    event: &sdkwork_agent_kernel::KernelEvent,
    text_by_item: &mut HashMap<String, String>,
) -> Option<String> {
    if !matches!(
        event.event_type.as_str(),
        "agent.message.started" | "agent.message.updated" | "agent.message.completed"
    ) {
        return None;
    }
    let payload = serde_json::from_str::<Value>(&event.payload).ok()?;
    let item = payload.get("item")?;
    if item.get("type")?.as_str()? != "agent_message" {
        return None;
    }
    let item_id = item.get("id")?.as_str()?.to_string();
    let current = item.get("text")?.as_str()?.to_string();
    let previous = text_by_item
        .insert(item_id, current.clone())
        .unwrap_or_default();
    if event.event_type == "agent.message.started"
        || !current.starts_with(&previous)
        || current.len() == previous.len()
    {
        return None;
    }
    Some(current[previous.len()..].to_string())
}

fn agent_turn_runtime_event_json(
    sequence: usize,
    event: &sdkwork_agent_kernel::KernelEvent,
    session_id: &str,
    turn_id: &str,
) -> Result<Value, ApiProblem> {
    let payload = match event.redaction_classification {
        sdkwork_agent_kernel::KernelEventRedaction::Secret
        | sdkwork_agent_kernel::KernelEventRedaction::Regulated => json!({ "redacted": true }),
        _ => serde_json::from_str(&event.payload).map_err(|error| {
            ApiProblem::internal(format!(
                "turn runtime event payload is invalid JSON: {error}"
            ))
        })?,
    };
    let provider_session_id = payload
        .get("providerSessionId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        // Compatibility for provider events emitted before the normalized
        // payload adopted providerSessionId. event.session_id is canonical.
        .or_else(|| {
            payload
                .get("threadId")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        });
    let trace_context = event.trace_context.as_ref().map(|trace| {
        json!({
            "traceId": trace.trace_id,
            "spanId": trace.span_id,
            "parentSpanId": trace.parent_span_id,
        })
    });
    Ok(json!({
        "eventType": "event",
        "event": {
            "eventId": event.event_id,
            "type": event.event_type,
            "version": event.event_version,
            "sequence": sequence,
            "occurredAt": event.occurred_at,
            "source": kernel_event_source(event.source),
            "severity": kernel_event_severity(event.severity),
            "sessionId": session_id,
            "turnId": turn_id,
            "providerSessionId": provider_session_id,
            "taskId": event.task_id,
            "runId": event.run_id,
            "itemId": event.step_id,
            "traceContext": trace_context,
            "correlationId": event.correlation_id,
            "causationId": event.causation_id,
            "redactionClassification": kernel_event_redaction(event.redaction_classification),
            "payloadSchema": event.payload_schema,
            "payload": payload,
            "replay": event.replay,
        }
    }))
}

fn kernel_event_source(source: sdkwork_agent_kernel::KernelEventSource) -> &'static str {
    match source {
        sdkwork_agent_kernel::KernelEventSource::Runtime => "runtime",
        sdkwork_agent_kernel::KernelEventSource::Manifest => "manifest",
        sdkwork_agent_kernel::KernelEventSource::Provider => "provider",
        sdkwork_agent_kernel::KernelEventSource::Model => "model",
        sdkwork_agent_kernel::KernelEventSource::Tool => "tool",
        sdkwork_agent_kernel::KernelEventSource::Context => "context",
        sdkwork_agent_kernel::KernelEventSource::Memory => "memory",
        sdkwork_agent_kernel::KernelEventSource::Policy => "policy",
        sdkwork_agent_kernel::KernelEventSource::Host => "host",
        sdkwork_agent_kernel::KernelEventSource::ProtocolAdapter => "protocol_adapter",
        sdkwork_agent_kernel::KernelEventSource::KernelUi => "kernel_ui",
        sdkwork_agent_kernel::KernelEventSource::CodeKernel => "code_kernel",
        sdkwork_agent_kernel::KernelEventSource::Telemetry => "telemetry",
        sdkwork_agent_kernel::KernelEventSource::Unknown => "unknown",
    }
}

fn kernel_event_redaction(redaction: sdkwork_agent_kernel::KernelEventRedaction) -> &'static str {
    match redaction {
        sdkwork_agent_kernel::KernelEventRedaction::Public => "public",
        sdkwork_agent_kernel::KernelEventRedaction::Internal => "internal",
        sdkwork_agent_kernel::KernelEventRedaction::TenantSensitive => "tenant_sensitive",
        sdkwork_agent_kernel::KernelEventRedaction::PersonalData => "personal_data",
        sdkwork_agent_kernel::KernelEventRedaction::Secret => "secret",
        sdkwork_agent_kernel::KernelEventRedaction::Regulated => "regulated",
        sdkwork_agent_kernel::KernelEventRedaction::Unknown => "unknown",
    }
}

fn total_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        0
    } else {
        total_items.div_ceil(page_size)
    }
}

/// Extracts the `Access-Token` header from the request for dual-token
/// cloudrouter routing (API_SPEC §819/§824). Returns `None` when the header is
/// absent; the value is never persisted.
pub(crate) fn extract_access_token(headers: &HeaderMap) -> Option<String> {
    let token = headers
        .get("Access-Token")
        .or_else(|| headers.get("access-token"))?
        .to_str()
        .ok()?
        .trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// Extracts the raw `Authorization: Bearer <token>` credential from request
/// headers for transient cloudrouter account-pool routing. Returns `None` when
/// the header is absent or does not use the Bearer scheme; the value is never
/// persisted.
pub(crate) fn extract_bearer_auth_token(headers: &HeaderMap) -> Option<String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .trim();
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    let token = token.trim();
    (!token.is_empty()).then(|| token.to_string())
}

fn offset_page_info(page: usize, page_size: usize, total_count: u64, has_more: bool) -> PageInfo {
    let params = sdkwork_utils_rust::http_api::OffsetListPageParams {
        page: page as i64,
        page_size: page_size as i64,
        offset: ((page as i64) - 1) * (page_size as i64),
    };
    let mut info = sdkwork_utils_rust::http_api::offset_list_page_info(total_count as i64, params);
    info.has_more = Some(has_more);
    info
}

#[cfg(test)]
mod tests {
    use super::testing::test_web_context;
    use super::*;
    use crate::infrastructure::{
        IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::Extension;
    use tower::ServiceExt;

    /// Serializes model configuration tests: applying a configuration now
    /// materializes into the real CLI-native config files (codex config.toml,
    /// claude settings.json, ...), which parallel tests would corrupt with
    /// concurrent writes.
    static MODEL_CONFIGURATION_TEST_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> =
        std::sync::OnceLock::new();

    fn model_configuration_test_guard() -> std::sync::MutexGuard<'static, ()> {
        MODEL_CONFIGURATION_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    struct AmbiguousProviderSessionProjectCwdResolver;

    impl sdkwork_agents_runtime_facade::ProviderSessionProjectCwdResolver
        for AmbiguousProviderSessionProjectCwdResolver
    {
        fn resolve_project_cwd(
            &self,
            _selector: &sdkwork_agents_runtime_facade::ProviderSessionProjectCwdSelector,
        ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<Option<String>> {
            Err(
                sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                    "multiple desktop roots are bound to this project".to_string(),
                ),
            )
        }
    }

    fn test_manifest() -> Value {
        json!({
            "schema_version": "1.0.0",
            "manifest_type": "agent",
            "agent_id": "agent.alpha",
            "name": "sample-agent",
            "display_name": "Sample Agent",
            "description": "sample",
            "version": "0.1.0",
            "domain": "intelligence",
            "required_capabilities": ["model.chat"],
            "optional_capabilities": ["tool.invoke"],
            "event_families": ["agent.lifecycle"],
            "owner": { "name": "sdkwork" },
            "status": "active"
        })
    }

    fn test_policy_provider() -> IamGatedPolicyProvider {
        IamGatedPolicyProvider::new("policy.agents.test.iam-gated")
    }

    fn build_test_router(state: AgentHttpState) -> axum::Router {
        build_test_router_with_context(state, test_agent_context())
    }

    fn build_test_router_with_context(
        state: AgentHttpState,
        context: AgentRequestContext,
    ) -> axum::Router {
        build_combined_routes()
            .with_state(state)
            .layer(Extension(context))
            .layer(Extension(test_web_context()))
    }

    fn create_agent_body(agent_id: &str, code: &str) -> Value {
        json!({
            "agentId": agent_id,
            "code": code,
            "displayName": code,
            "description": "contract test agent",
            "manifest": test_manifest(),
            "visibility": "organization",
            "requestedAt": "2026-06-01T00:00:00Z"
        })
    }

    async fn create_app_agent(app: &axum::Router, agent_id: &str, code: &str) {
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_agent_body(agent_id, code).to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    async fn create_app_provider_binding(app: &axum::Router, agent_id: &str) {
        let binding_body = json!({
            "bindingId": "binding.codex",
            "providerId": "provider.codex",
            "implementationKind": "process-adapter",
            "configurationProfileId": "profile.codex.default",
            "capabilities": ["agent.runtime.preview", "agent.runtime.prompt_optimization"],
            "makeDefault": true,
            "requestedAt": "2026-06-01T00:01:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri(format!(
                "/app/v3/api/ai/agents/{agent_id}/provider_bindings"
            ))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(binding_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("provider binding request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    fn test_agent_context() -> AgentRequestContext {
        AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.manage"])
            .with_trace_id("trace-test-fixed")
            .with_request_id("req-test-fixed")
    }

    /// Picks an engine whose provider is always compiled into the service
    /// build. Codex is optional (embedded client-local SQLite runtime store);
    /// every other engine is available in all builds.
    fn model_configuration_test_engine() -> &'static str {
        if sdkwork_agents_runtime_facade::codex_engine_enabled() {
            "codex"
        } else {
            "gemini"
        }
    }

    fn model_configuration_body(
        engine_id: &str,
        configuration_id: &str,
        api_key: Option<&str>,
    ) -> Value {
        let mut body = json!({
            "configurationId": configuration_id,
            "engineId": engine_id,
            "vendorCode": "openai-compatible",
            "baseUrl": "https://models.example.test/v1",
            "defaultModelId": "example-chat",
            "supportedModelIds": ["example-chat", "example-reasoning"],
            "inputContextTokens": "128000",
            "outputContextTokens": "16000",
            "toolCallRounds": "32",
            "supportsMultimodal": true
        });
        if let Some(api_key) = api_key {
            body["apiKey"] = Value::String(api_key.to_string());
        }
        body
    }

    async fn post_model_configuration(app: &axum::Router, body: Value) -> Response {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/app/v3/api/ai/model_configurations/apply")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("model configuration request should be built"),
            )
            .await
            .expect("model configuration request should complete")
    }

    fn model_selection_body(
        engine_id: &str,
        model_id: &str,
        configuration_id: Option<&str>,
    ) -> Value {
        let mut body = json!({
            "engineId": engine_id,
            "modelId": model_id
        });
        if let Some(configuration_id) = configuration_id {
            body["configurationId"] = Value::String(configuration_id.to_string());
        }
        body
    }

    async fn post_model_selection(app: &axum::Router, body: Value) -> Response {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/app/v3/api/ai/model_selections/apply")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("model selection request should be built"),
            )
            .await
            .expect("model selection request should complete")
    }

    #[tokio::test]
    async fn model_configuration_dispatches_all_providers_without_exposing_credentials() {
        let _guard = model_configuration_test_guard();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        for engine_id in sdkwork_agents_runtime_facade::bootstrappable_engine_keys() {
            if engine_id == "codex" && !sdkwork_agents_runtime_facade::codex_engine_enabled() {
                // Codex is an optional per-application provider (embedded
                // client-local SQLite store); builds without the feature
                // reject codex model configuration by contract.
                continue;
            }
            let api_key = format!("secret-value-for-{engine_id}");
            let response = post_model_configuration(
                &app,
                model_configuration_body(
                    engine_id,
                    &format!("{ID_PREFIX_MODEL}{engine_id}.custom"),
                    Some(&api_key),
                ),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK, "engine {engine_id}");
            let bytes = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("response body should be readable");
            let response_text =
                String::from_utf8(bytes.to_vec()).expect("response body should be UTF-8");
            assert!(!response_text.contains(&api_key));
            assert!(!response_text.contains("secret-"));
            let payload: Value =
                serde_json::from_str(&response_text).expect("response should be JSON");
            let item = &payload["data"]["item"];
            assert_eq!(item["engineId"], engine_id);
            assert_eq!(item["apiKeyConfigured"], true);
            assert_eq!(
                item["supportedProviderIds"]
                    .as_array()
                    .expect("provider ids should be an array")
                    .len(),
                sdkwork_agents_runtime_facade::bootstrappable_engine_keys().len()
            );
        }
    }

    #[tokio::test]
    async fn model_configuration_rejects_numeric_int64_settings() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);
        let mut body = model_configuration_body("codex", "model.codex.custom", None);
        body["inputContextTokens"] = json!(128000);
        let response = post_model_configuration(&app, body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn model_configuration_requires_a_credential_only_for_new_profiles() {
        let _guard = model_configuration_test_guard();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);
        let engine_id = model_configuration_test_engine();
        let configuration_id = "model.reusable";

        let missing = post_model_configuration(
            &app,
            model_configuration_body(engine_id, configuration_id, None),
        )
        .await;
        assert_eq!(missing.status(), StatusCode::BAD_REQUEST);

        let created = post_model_configuration(
            &app,
            model_configuration_body(engine_id, configuration_id, Some("initial-secret")),
        )
        .await;
        assert_eq!(created.status(), StatusCode::OK);
        let created_payload: Value = serde_json::from_slice(
            &to_bytes(created.into_body(), usize::MAX)
                .await
                .expect("created response should be readable"),
        )
        .expect("created response should be JSON");

        let reapplied = post_model_configuration(
            &app,
            model_configuration_body(engine_id, configuration_id, None),
        )
        .await;
        assert_eq!(reapplied.status(), StatusCode::OK);
        let reapplied_payload: Value = serde_json::from_slice(
            &to_bytes(reapplied.into_body(), usize::MAX)
                .await
                .expect("reapplied response should be readable"),
        )
        .expect("reapplied response should be JSON");
        assert_eq!(
            created_payload["data"]["item"]["profileId"],
            reapplied_payload["data"]["item"]["profileId"]
        );
    }

    #[tokio::test]
    async fn model_configuration_is_isolated_across_tenants_on_http_surface() {
        let _guard = model_configuration_test_guard();
        let store = ScopedInMemoryAgentConfigurationStore::new();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        )
        .with_model_configuration_providers(
            Box::new(sdkwork_agent_kernel::InMemorySecretProvider::new()),
            Box::new(store),
        );
        let engine_id = model_configuration_test_engine();
        let tenant_a_app = build_test_router_with_context(state.clone(), test_agent_context());
        let tenant_b_context = AgentRequestContext::new("200002", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.manage"])
            .with_trace_id("trace-test-fixed-b")
            .with_request_id("req-test-fixed-b");
        let tenant_b_app = build_test_router_with_context(state, tenant_b_context);

        // Tenant A applies a model configuration.
        let created = post_model_configuration(
            &tenant_a_app,
            model_configuration_body(engine_id, "model.cross-tenant", Some("tenant-a-secret")),
        )
        .await;
        assert_eq!(created.status(), StatusCode::OK);
        let created_payload: Value = serde_json::from_slice(
            &to_bytes(created.into_body(), usize::MAX)
                .await
                .expect("created response should be readable"),
        )
        .expect("created response should be JSON");
        let profile_id = created_payload["data"]["item"]["profileId"]
            .as_str()
            .expect("profileId should be present")
            .to_string();

        // Tenant B cannot list tenant A profiles.
        let tenant_b_list = get_model_configuration(
            &tenant_b_app,
            &format!("/app/v3/api/ai/model_configurations?engineId={engine_id}"),
        )
        .await;
        assert_eq!(tenant_b_list.status(), StatusCode::OK);
        let list_payload: Value = serde_json::from_slice(
            &to_bytes(tenant_b_list.into_body(), usize::MAX)
                .await
                .expect("list response should be readable"),
        )
        .expect("list response should be JSON");
        assert_eq!(
            list_payload["data"]["items"].as_array().expect("items").len(),
            0,
            "tenant B must not observe tenant A model configurations"
        );

        // Tenant B cannot read or archive tenant A profiles by id.
        let detail_uri = format!("/app/v3/api/ai/model_configurations/{engine_id}/{profile_id}");
        let tenant_b_get = get_model_configuration(&tenant_b_app, &detail_uri).await;
        assert_eq!(tenant_b_get.status(), StatusCode::NOT_FOUND);
        let archive_request = Request::builder()
            .method("POST")
            .uri(format!("{detail_uri}/archive"))
            .body(Body::empty())
            .expect("archive request should be built");
        let tenant_b_archive = tenant_b_app
            .clone()
            .oneshot(archive_request)
            .await
            .expect("archive request should complete");
        assert!(
            tenant_b_archive.status() == StatusCode::NOT_FOUND
                || tenant_b_archive.status() == StatusCode::BAD_REQUEST,
            "tenant B archive must fail, got {}",
            tenant_b_archive.status()
        );

        // Tenant A still owns the profile.
        let tenant_a_get = get_model_configuration(&tenant_a_app, &detail_uri).await;
        assert_eq!(tenant_a_get.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn model_configuration_requires_selected_provider_support() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);
        let mut body = model_configuration_body(
            "codex",
            "model.codex.unsupported-selection",
            Some("unused-secret"),
        );
        body["supportedProviderIds"] = json!(["claude-code"]);

        let response = post_model_configuration(&app, body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn get_model_configuration(app: &axum::Router, uri: &str) -> Response {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(Body::empty())
                    .expect("model configuration read request should be built"),
            )
            .await
            .expect("model configuration read request should complete")
    }

    #[tokio::test]
    async fn model_configuration_read_back_list_get_status_and_archive() {
        let _guard = model_configuration_test_guard();
        // An in-memory profile store isolates the persistence surface for the
        // read-back lifecycle endpoints.
        let store = ScopedInMemoryAgentConfigurationStore::new();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        )
        .with_model_configuration_providers(
            Box::new(sdkwork_agent_kernel::InMemorySecretProvider::new()),
            Box::new(store),
        );
        let app = build_test_router(state);

        // Codex is an optional build-time provider; any always-available
        // engine exercises the same lifecycle surface.
        let engine_id = model_configuration_test_engine();
        let created = post_model_configuration(
            &app,
            model_configuration_body(engine_id, "model.readback", Some("readback-secret")),
        )
        .await;
        assert_eq!(created.status(), StatusCode::OK);
        let created_payload: Value = serde_json::from_slice(
            &to_bytes(created.into_body(), usize::MAX)
                .await
                .expect("created response should be readable"),
        )
        .expect("created response should be JSON");
        let profile_id = created_payload["data"]["item"]["profileId"]
            .as_str()
            .expect("profileId should be present")
            .to_string();
        let detail_uri = format!("/app/v3/api/ai/model_configurations/{engine_id}/{profile_id}");
        let status_uri = format!("{detail_uri}/status");
        let archive_uri = format!("{detail_uri}/archive");

        // List returns the applied profile without exposing credentials.
        let list =
            get_model_configuration(&app, &format!("/app/v3/api/ai/model_configurations?engineId={engine_id}"))
                .await;
        assert_eq!(list.status(), StatusCode::OK);
        let list_bytes = to_bytes(list.into_body(), usize::MAX)
            .await
            .expect("list response should be readable");
        let list_text = String::from_utf8(list_bytes.to_vec()).expect("list should be UTF-8");
        assert!(
            !list_text.contains("readback-secret"),
            "credentials must not be listed"
        );
        let list_payload: Value =
            serde_json::from_str(&list_text).expect("list response should be JSON");
        let items = list_payload["data"]["items"]
            .as_array()
            .expect("items should be an array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["profileId"], profile_id);
        assert_eq!(items[0]["engineId"], engine_id);
        assert_eq!(items[0]["providerScope"], sdkwork_agents_runtime_facade::agent_engine_provider_scope(engine_id).expect("engine scope"));
        assert_eq!(items[0]["baseUrl"], "https://models.example.test/v1");
        assert_eq!(items[0]["defaultModelId"], "example-chat");
        assert_eq!(items[0]["apiKeyConfigured"], true);
        assert_eq!(items[0]["status"], "active");

        // Detail resolves the stored profile with the same redaction rules.
        let detail = get_model_configuration(&app, &detail_uri).await;
        assert_eq!(detail.status(), StatusCode::OK);
        let detail_payload: Value = serde_json::from_slice(
            &to_bytes(detail.into_body(), usize::MAX)
                .await
                .expect("detail response should be readable"),
        )
        .expect("detail response should be JSON");
        assert_eq!(detail_payload["data"]["item"]["profileId"], profile_id);

        // Status reports the stored expectations plus the provider-native
        // read-back (the native surface state depends on the host).
        let status = get_model_configuration(&app, &status_uri).await;
        assert_eq!(status.status(), StatusCode::OK);
        let status_payload: Value = serde_json::from_slice(
            &to_bytes(status.into_body(), usize::MAX)
                .await
                .expect("status response should be readable"),
        )
        .expect("status response should be JSON");
        let status_item = &status_payload["data"]["item"];
        assert_eq!(status_item["profileId"], profile_id);
        assert_eq!(
            status_item["expectedBaseUrl"],
            "https://models.example.test/v1"
        );
        assert_eq!(status_item["expectedDefaultModel"], "example-chat");
        assert_eq!(status_item["credentialConfigured"], true);
        let derived_state = status_item["derivedState"]
            .as_str()
            .expect("derivedState should be present");
        assert!(
            [
                "materialized",
                "diverged",
                "not_materialized",
                "unsupported"
            ]
            .contains(&derived_state),
            "unexpected derived state {derived_state}"
        );

        // Archive marks the profile archived and restores the CLI config.
        let archive = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&archive_uri)
                    .body(Body::empty())
                    .expect("archive request should be built"),
            )
            .await
            .expect("archive request should complete");
        assert_eq!(archive.status(), StatusCode::OK);
        let archive_payload: Value = serde_json::from_slice(
            &to_bytes(archive.into_body(), usize::MAX)
                .await
                .expect("archive response should be readable"),
        )
        .expect("archive response should be JSON");
        assert_eq!(archive_payload["data"]["item"]["profileId"], profile_id);
        assert_eq!(archive_payload["data"]["item"]["status"], "archived");

        // The detail still resolves with the archived status.
        let detail_after = get_model_configuration(&app, &detail_uri).await;
        assert_eq!(detail_after.status(), StatusCode::OK);
        let detail_after_payload: Value = serde_json::from_slice(
            &to_bytes(detail_after.into_body(), usize::MAX)
                .await
                .expect("detail response should be readable"),
        )
        .expect("detail response should be JSON");
        assert_eq!(detail_after_payload["data"]["item"]["status"], "archived");

        // Unknown profiles are 404 on every read surface.
        let missing = get_model_configuration(
            &app,
            "/app/v3/api/ai/model_configurations/codex/profile.model_configuration.missing",
        )
        .await;
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn catalog_model_selection_dispatches_every_published_provider_config_spi() {
        let _guard = model_configuration_test_guard();
        // The OpenCode catalog model id is resolved from OPENCODE_MODEL or the
        // user config; pin it so the dispatch test does not depend on the host
        // machine's opencode configuration state.
        std::env::set_var("OPENCODE_MODEL", "opencode/deepseek-v4-flash-free");
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);
        let catalog = crate::agent_engine_catalog::list_agent_engine_catalog();
        assert!(!catalog.engines.is_empty(), "app catalog must not be empty");

        // The complete catalog always lists every bootstrappable engine, but
        // engines that failed to bootstrap in this runtime profile carry no
        // published models; only available engines can dispatch a selection.
        for engine in catalog.engines.iter().filter(|engine| engine.available) {
            let model_id = engine
                .models
                .first()
                .expect("available engine should publish a model")
                .model_id
                .clone();
            let response = post_model_selection(
                &app,
                model_selection_body(&engine.engine_key, &model_id, None),
            )
            .await;
            let response_status = response.status();
            let response_bytes = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("selection response should be readable");
            assert_eq!(
                response_status,
                StatusCode::OK,
                "engine {} model {} body {}",
                engine.engine_key,
                model_id,
                String::from_utf8_lossy(&response_bytes)
            );
            let payload: Value =
                serde_json::from_slice(&response_bytes).expect("selection response should be JSON");
            let item = &payload["data"]["item"];
            assert_eq!(item["engineId"], engine.engine_key);
            assert_eq!(item["modelId"], model_id);
            assert!(item.get("configurationId").is_none());
        }
    }

    #[tokio::test]
    async fn custom_model_selection_reuses_saved_configuration_and_supported_models() {
        let _guard = model_configuration_test_guard();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);
        let configuration_id = "model.codex.selection";
        let configured = post_model_configuration(
            &app,
            model_configuration_body("codex", configuration_id, Some("selection-secret")),
        )
        .await;
        assert_eq!(configured.status(), StatusCode::OK);

        let selected = post_model_selection(
            &app,
            model_selection_body("codex", "example-reasoning", Some(configuration_id)),
        )
        .await;
        assert_eq!(selected.status(), StatusCode::OK);
        let response_text = String::from_utf8(
            to_bytes(selected.into_body(), usize::MAX)
                .await
                .expect("selection response should be readable")
                .to_vec(),
        )
        .expect("selection response should be UTF-8");
        assert!(!response_text.contains("selection-secret"));
        let payload: Value =
            serde_json::from_str(&response_text).expect("selection response should be JSON");
        assert_eq!(payload["data"]["item"]["configurationId"], configuration_id);
        assert_eq!(payload["data"]["item"]["modelId"], "example-reasoning");

        let unsupported = post_model_selection(
            &app,
            model_selection_body("codex", "outside-configuration", Some(configuration_id)),
        )
        .await;
        assert_eq!(unsupported.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn builtin_platform_model_selection_switches_without_saved_configuration() {
        let _guard = model_configuration_test_guard();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);
        // Platform catalog models (for example the built-in official channels)
        // are not part of the agent-engine catalog. Switching one directly must
        // succeed without a saved configuration and without an API key.
        let response =
            post_model_selection(&app, model_selection_body("codex", "gpt-5.4", None)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let payload: Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("selection response should be readable")
                .to_vec(),
        )
        .expect("selection response should be JSON");
        assert_eq!(payload["data"]["item"]["modelId"], "gpt-5.4");
    }

    fn facade_policy_subject() -> sdkwork_agent_kernel::PolicySubject {
        sdkwork_agent_kernel::PolicySubject::new("user:100", "100001")
            .with_role("ai.agents.manage")
    }

    fn facade_actor() -> sdkwork_agents_runtime_facade::AgentsSessionActor {
        sdkwork_agents_runtime_facade::AgentsSessionActor {
            subject_id: "user:100".to_string(),
            roles: vec!["ai.agents.manage".to_string()],
        }
    }

    fn facade_runtime_binding(
        runtime_binding_id: &str,
    ) -> sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor {
        sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor {
            runtime_binding_id: runtime_binding_id.to_string(),
            runtime_location_id: Some("birdcoder-workspace-001".to_string()),
            host_mode: "desktop".to_string(),
            transport_kind: "process".to_string(),
            provider_binding_id: "binding.codex".to_string(),
            model_id: "model.gpt-5".to_string(),
            provider_id: "provider.codex".to_string(),
            provider_session_id: Some(format!("provider-{runtime_binding_id}")),
            provider_session_tree_id: Some("provider-tree-001".to_string()),
            provider_parent_session_id: Some("provider-parent-001".to_string()),
            provider_forked_from_session_id: Some("provider-origin-001".to_string()),
            provider_directory: None,
        }
    }

    fn facade_session_request(
        session_id: &str,
        runtime_binding_id: Option<&str>,
    ) -> sdkwork_agents_runtime_facade::ResolveAgentsSessionRequest {
        sdkwork_agents_runtime_facade::ResolveAgentsSessionRequest {
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 100,
            agent_id: "agent.facade".to_string(),
            session_id: session_id.to_string(),
            project_id: None,
            session_kind: sdkwork_agents_runtime_facade::AgentsSessionKind::Coding,
            entry_surface: sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Pc,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: format!("Facade session {session_id}"),
            idempotency_key: format!("create-{session_id}"),
            payload_hash: format!("sha256:create-{session_id}"),
            runtime_binding: runtime_binding_id.map(facade_runtime_binding),
            actor: facade_actor(),
            requested_at: "2026-07-22T12:00:00Z".to_string(),
        }
    }

    #[derive(Clone)]
    struct SessionRetrieveFailurePolicy;

    impl PolicyProvider for SessionRetrieveFailurePolicy {
        fn provider_manifest(&self) -> ProviderManifest {
            ProviderManifest::new(
                "policy.agents.test.session-retrieve-failure",
                "policy",
                "session-retrieve-failure-policy",
                "0.1.0",
                vec!["policy.evaluate".to_string()],
            )
        }

        fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
            if request.action.as_deref() == Some("session.retrieve") {
                return Err(KernelError::Internal {
                    message: "injected session retrieval failure".to_string(),
                });
            }
            Ok(PolicyDecision::allow(
                format!("decision.{}", request.policy_request_id),
                request.policy_request_id,
                "policy.agents.test.session-retrieve-failure",
            ))
        }

        fn health(&self) -> ProviderHealth {
            ProviderHealth::available()
        }
    }

    #[tokio::test]
    async fn session_facade_persists_canonical_session_lineage_and_runtime_binding() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.facade", "facade").await;
        create_app_provider_binding(&app, "agent.facade").await;

        let subject = sdkwork_agent_kernel::PolicySubject::new("user:100", "100001")
            .with_role("ai.agents.manage");
        state
            .service
            .create_project(CreateProjectCommand {
                tenant_id: 100_001,
                organization_id: 0,
                project_id: "project.facade".to_string(),
                workspace_id: None,
                owner_user_id: 100,
                name: "Facade project".to_string(),
                description: Some("Canonical session ownership".to_string()),
                visibility: AgentProjectVisibility::Private,
                drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
                default_agent_id: Some("agent.facade".to_string()),
                default_model_id: Some("model.gpt-5".to_string()),
                requested_by: subject.clone(),
                requested_at: "2026-07-22T11:59:00Z".to_string(),
            })
            .expect("facade project should be created");

        let facade = state.session_facade();
        let parent_request = facade_session_request(
            "session.test.facade.parent",
            Some("runtime_binding.test.facade.parent"),
        );
        facade
            .resolve_or_create_session(parent_request)
            .expect("parent session should resolve");
        let parent_turn_facade = facade.clone();
        let parent_turn = tokio::task::spawn_blocking(move || {
            parent_turn_facade.complete_turn(
                sdkwork_agents_runtime_facade::CompleteAgentsTurnRequest {
                    tenant_id: 100_001,
                    organization_id: 0,
                    owner_user_id: 100,
                    agent_id: "agent.facade".to_string(),
                    session_id: "session.test.facade.parent".to_string(),
                    content: "Create a fork point".to_string(),
                    content_type: "text/plain".to_string(),
                    idempotency_key: "turn.facade.parent".to_string(),
                    client_request_id: "request.facade.parent".to_string(),
                    actor: facade_actor(),
                    requested_at: "2026-07-22T12:00:01Z".to_string(),
                },
            )
        })
        .await
        .expect("parent turn worker should complete")
        .expect("parent turn should complete");

        let mut child_request = facade_session_request(
            "session.test.facade.child",
            Some("runtime_binding.test.facade.child"),
        );
        child_request.project_id = Some("project.facade".to_string());
        child_request.source_module = Some("birdcoder".to_string());
        child_request.source_context_kind = Some("coding_project".to_string());
        child_request.source_context_id = Some("workspace-001".to_string());
        child_request.parent_session_id = Some("session.test.facade.parent".to_string());
        child_request.forked_from_turn_id = Some(parent_turn.turn_id.clone());

        let created = facade
            .resolve_or_create_session(child_request.clone())
            .expect("child session should resolve");
        assert!(created.created);

        let stored = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.facade".to_string(),
                session_id: "session.test.facade.child".to_string(),
                owner_scope: Some(100),
                requested_by: subject.clone(),
            })
            .expect("child session should be persisted");
        assert_eq!(stored.project_id.as_deref(), Some("project.facade"));
        assert_eq!(stored.session_kind, AgentSessionKind::Coding);
        assert_eq!(stored.entry_surface, AgentSessionEntrySurface::Pc);
        assert_eq!(stored.source_module.as_deref(), Some("birdcoder"));
        assert_eq!(
            stored.source_context_kind.as_deref(),
            Some("coding_project")
        );
        assert_eq!(stored.source_context_id.as_deref(), Some("workspace-001"));
        assert_eq!(
            stored.parent_session_id.as_deref(),
            Some("session.test.facade.parent")
        );
        assert_eq!(
            stored.forked_from_turn_id.as_deref(),
            Some(parent_turn.turn_id.as_str())
        );

        let binding = state
            .service
            .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.facade".to_string(),
                session_id: "session.test.facade.child".to_string(),
                runtime_binding_id: "runtime_binding.test.facade.child".to_string(),
                owner_scope: Some(100),
                requested_by: subject,
            })
            .expect("child runtime binding should be persisted");
        let descriptor = child_request
            .runtime_binding
            .as_ref()
            .expect("child request has a runtime binding");
        assert!(runtime_binding_matches_descriptor(&binding, descriptor));

        let replay = facade
            .resolve_or_create_session(child_request.clone())
            .expect("same descriptor should resolve idempotently");
        assert!(!replay.created);
        assert_eq!(replay.session_id, created.session_id);

        child_request
            .runtime_binding
            .as_mut()
            .expect("child request has a runtime binding")
            .model_id = "model.conflict".to_string();
        let error = facade
            .resolve_or_create_session(child_request)
            .expect_err("conflicting descriptor must be rejected");
        assert!(matches!(
            error,
            sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(message)
                if message.contains("runtime binding descriptor conflicts")
        ));
    }

    #[tokio::test]
    async fn session_facade_defaults_to_rig_runtime_binding() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.facade", "facade-no-binding").await;
        let facade = state.session_facade();
        facade
            .resolve_or_create_session(facade_session_request("session.test.facade.unbound", None))
            .expect("session without an explicit binding should still resolve");

        // Without a client descriptor the session is bound to the RIG
        // simple-agent engine by default (`binding.rig`), so the session
        // carries an active runtime binding after resolution.
        let bindings = state
            .service
            .list_session_runtime_bindings(crate::application::ListSessionRuntimeBindingsCommand {
                path_agent_id: "agent.facade".to_string(),
                query: crate::ports::SessionRuntimeBindingListQuery::for_session(
                    100_001,
                    0,
                    "session.test.facade.unbound",
                ),
                owner_scope: Some(100),
                requested_by: facade_policy_subject(),
            })
            .expect("session runtime bindings must list");
        assert_eq!(bindings.items.len(), 1, "default RIG binding must be created");
        assert_eq!(
            bindings.items[0].provider_binding_id, "binding.rig",
            "default binding must target the RIG simple-agent engine"
        );
        assert_eq!(bindings.items[0].model_id, "rig.default-chat");
        let provider_binding = state
            .service
            .get_provider_binding(100_001, "agent.facade", "binding.rig", facade_policy_subject())
            .expect("RIG provider binding must be bootstrapped");
        assert!(provider_binding.active);
    }

    #[tokio::test]
    async fn session_facade_archives_created_session_when_runtime_binding_creation_fails() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.facade", "facade-binding-failure").await;
        let facade = state.session_facade();
        // The descriptor passes request-level validation, but the agent has no
        // provider binding, so runtime binding creation fails after the
        // Session row has already been created.
        let request = facade_session_request(
            "session.test.facade.orphan-prevention",
            Some("runtime_binding.test.facade.invalid"),
        );

        let error = facade
            .resolve_or_create_session(request)
            .expect_err("binding creation failure must surface");

        assert!(matches!(
            error,
            sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(_)
        ));
        // The Session created without its canonical runtime binding must be
        // retired immediately instead of lingering as a permanently empty
        // conversation.
        let stored = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.facade".to_string(),
                session_id: "session.test.facade.orphan-prevention".to_string(),
                owner_scope: Some(100),
                requested_by: sdkwork_agent_kernel::PolicySubject::new("user:100", "100001")
                    .with_role("ai.agents.manage"),
            })
            .expect("created Session should remain readable for audit");
        assert_eq!(
            stored.status,
            crate::domain::AgentSessionStatus::Archived,
            "an orphaned provider-history Session must be archived when its binding cannot be created"
        );
    }

    #[tokio::test]
    async fn session_facade_does_not_create_after_non_not_found_read_error() {
        let audit_sink = InMemoryAgentAuditSink::default();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            audit_sink.clone(),
            SessionRetrieveFailurePolicy,
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.facade", "facade-read-error").await;
        let audit_count_before = audit_sink.events().len();

        let error = state
            .session_facade()
            .resolve_or_create_session(facade_session_request(
                "session.test.facade.read-error",
                None,
            ))
            .expect_err("non-not-found read errors must fail closed");
        assert!(matches!(
            error,
            sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(message)
                if message.contains("injected session retrieval failure")
        ));
        assert_eq!(audit_sink.events().len(), audit_count_before);
    }

    #[tokio::test]
    async fn app_create_and_retrieve_agent_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        let create_body = json!({
            "agentId": "agent.alpha",
            "code": "alpha",
            "displayName": "Alpha",
            "description": "first",
            "manifest": test_manifest(),
            "defaultCodeTaskIntent": {
                "prompt": "Refactor runtime",
                "contextPaths": ["src/lib.rs"],
                "constraints": ["safe"]
            },
            "visibility": "organization",
            "tags": ["starter"],
            "requestedAt": "2026-06-01T00:00:00Z"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("get request should succeed");
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        // 信封形状：{ code: 0, data: { item: { agentId, ... } }, traceId }
        assert_eq!(body_json["code"], 0);
        assert_eq!(body_json["data"]["item"]["agentId"], "agent.alpha");
    }

    #[tokio::test]
    async fn open_api_create_and_retrieve_agent_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        let mut manifest = test_manifest();
        manifest["agent_id"] = json!("agent.open");
        let create_body = json!({
            "agentId": "agent.open",
            "code": "open",
            "displayName": "Open Agent",
            "description": "developer api",
            "manifest": manifest,
            "visibility": "organization",
            "tags": ["developer"],
            "requestedAt": "2026-06-01T00:00:00Z"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/agent/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("GET")
            .uri("/agent/v3/api/ai/agents/agent.open")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("get request should succeed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_delete_agent_uses_204_without_json_body() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        create_app_agent(&app, "agent.delete", "delete").await;

        let request = Request::builder()
            .method("DELETE")
            .uri("/app/v3/api/ai/agents/agent.delete")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("delete request should succeed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        assert!(body_bytes.is_empty());
    }

    #[tokio::test]
    async fn app_runtime_create_operations_use_201_status() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        create_app_agent(&app, "agent.runtime", "runtime").await;
        create_app_provider_binding(&app, "agent.runtime").await;

        let preview_body = json!({
            "executionId": "execution.preview",
            "content": "Summarize the repository state",
            "debugMode": false,
            "model": "codex",
            "temperature": 0.2,
            "requestedAt": "2026-06-01T00:02:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.runtime/preview_responses")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(preview_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("preview request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let optimization_body = json!({
            "executionId": "execution.prompt",
            "prompt": "Make this prompt concise",
            "requestedAt": "2026-06-01T00:03:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.runtime/prompt_optimizations")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(optimization_body.to_string()))
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("prompt optimization request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn app_delete_composition_slot_uses_204_without_json_body() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        create_app_agent(&app, "agent.slot-delete", "slot-delete").await;
        let slot_body = json!({
            "slotId": "slot.skill.primary",
            "slotKind": "skill",
            "targetModule": "skills",
            "targetRef": "skill.primary",
            "priority": 10,
            "enabled": true,
            "policyJson": "{}",
            "requestedAt": "2026-06-01T00:04:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.slot-delete/composition_slots")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(slot_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("composition slot create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("DELETE")
            .uri("/app/v3/api/ai/agents/agent.slot-delete/composition_slots/slot.skill.primary?requestedAt=2026-06-01T00:05:00Z")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("composition slot delete request should succeed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        assert!(body_bytes.is_empty());
    }

    #[tokio::test]
    async fn backend_status_update_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        let create_body = json!({
            "agentId": "agent.beta",
            "code": "beta",
            "displayName": "Beta",
            "description": null,
            "manifest": {
                "schema_version": "1.0.0",
                "manifest_type": "agent",
                "agent_id": "agent.beta",
                "name": "sample-agent",
                "display_name": "Sample Agent",
                "description": "sample",
                "version": "0.1.0",
                "domain": "intelligence",
                "required_capabilities": ["model.chat"],
                "optional_capabilities": ["tool.invoke"],
                "event_families": ["agent.lifecycle"],
                "owner": { "name": "sdkwork" },
                "status": "active"
            },
            "visibility": "private",
            "requestedAt": "2026-06-01T01:00:00Z"
        });

        let create_request = Request::builder()
            .method("POST")
            .uri("/backend/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create request should succeed");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let update_status_body = json!({
            "targetStatus": "active",
            "expectedVersion": "1",
            "requestedAt": "2026-06-01T01:05:00Z"
        });

        let status_request = Request::builder()
            .method("POST")
            .uri("/backend/v3/api/ai/agents/agent.beta/status")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(update_status_body.to_string()))
            .expect("request should be built");
        let status_response = app
            .oneshot(status_request)
            .await
            .expect("status request should succeed");

        assert_eq!(status_response.status(), StatusCode::OK);
        let body_bytes = to_bytes(status_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        // 信封形状：{ code: 0, data: { item: { status, ... } }, traceId }
        assert_eq!(body_json["code"], 0);
        assert_eq!(body_json["data"]["item"]["status"], "active");
    }

    #[tokio::test]
    async fn app_project_and_agent_engine_lists_allow_read_only_subjects() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let read_context = AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.read"]);
        let app = build_test_router_with_context(state, read_context);

        for uri in [
            "/app/v3/api/ai/agent_engines",
            "/app/v3/api/ai/projects?page=1&page_size=20",
        ] {
            let request = Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .expect("request should be built");
            let response = app
                .clone()
                .oneshot(request)
                .await
                .expect("list request should succeed");
            assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
        }
    }

    #[tokio::test]
    async fn app_user_can_create_project_with_use_permission() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app_user_context = AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.read", "ai.agents.use"]);
        let app = build_test_router_with_context(state, app_user_context);
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "projectId": "project.app-user",
                    "name": "BirdCoder project"
                })
                .to_string(),
            ))
            .expect("request should be built");

        let response = app
            .oneshot(request)
            .await
            .expect("project create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn app_project_list_supports_workspace_scoped_exact_name_lookup() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app_user_context = AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.read", "ai.agents.use"]);
        let app = build_test_router_with_context(state, app_user_context);

        let mut workspace_id = String::new();
        for (project_id, name) in [
            ("project.exact-name", "Alpha Project"),
            ("project.exact-name-prefix", "Alpha Project Extra"),
        ] {
            let request = Request::builder()
                .method("POST")
                .uri("/app/v3/api/ai/projects")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "projectId": project_id,
                        "name": name
                    })
                    .to_string(),
                ))
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let payload: Value = serde_json::from_slice(&body).unwrap();
            workspace_id = payload["data"]["item"]["workspaceId"]
                .as_str()
                .unwrap()
                .to_string();
        }

        let request = Request::builder()
            .method("GET")
            .uri(format!(
                "/app/v3/api/ai/projects?workspaceId={workspace_id}&name_exact=alpha%20project&page=1&page_size=20"
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        let items = payload["data"]["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["projectId"], "project.exact-name");
        assert_eq!(payload["data"]["pageInfo"]["totalItems"], "1");

        let request = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/projects?name_exact=Alpha%20Project&page=1&page_size=20")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn app_session_list_routes_are_read_only_and_cwd_ambiguity_is_a_client_error() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        )
        .with_provider_session_cwd_resolver(std::sync::Arc::new(
            AmbiguousProviderSessionProjectCwdResolver,
        ));
        let app = build_test_router(state);
        create_app_agent(&app, "agent.alpha", "alpha").await;

        let create_project = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "projectId": "project.sessions",
                    "name": "Project sessions",
                    "defaultAgentId": "agent.alpha"
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(create_project).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        let workspace_id = payload["data"]["item"]["workspaceId"]
            .as_str()
            .expect("created project must expose its workspaceId")
            .to_string();

        let empty_list = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/projects/project.sessions/sessions?page_size=20")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(empty_list).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert!(payload["data"]["items"].as_array().unwrap().is_empty());

        let synchronize = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects/project.sessions/sessions/synchronize")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(synchronize).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let problem: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(problem["status"], StatusCode::BAD_REQUEST.as_u16());
        assert!(problem["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("multiple desktop roots")));

        let create_session = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects/project.sessions/sessions")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "sessionId": "session.project-scoped",
                    "sessionKind": "coding",
                    "entrySurface": "pc",
                    "idempotencyKey": "create-session.project-scoped",
                    "payloadHash": "sha256:create-session.project-scoped",
                    "requestedAt": "2026-07-26T00:00:00Z"
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(create_session).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["projectId"], "project.sessions");
        assert_eq!(payload["data"]["item"]["agentId"], "agent.alpha");

        let retrieve_project_session = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/projects/project.sessions/sessions/session.project-scoped")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(retrieve_project_session).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            payload["data"]["item"]["sessionId"],
            "session.project-scoped"
        );
        assert_eq!(payload["data"]["item"]["projectId"], "project.sessions");

        let retrieve_from_wrong_project = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/projects/project.other/sessions/session.project-scoped")
            .body(Body::empty())
            .unwrap();
        let response = app
            .clone()
            .oneshot(retrieve_from_wrong_project)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let create_second_project = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "projectId": "project.sessions-secondary",
                    "workspaceId": workspace_id,
                    "name": "Secondary project"
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(create_second_project).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let create_second_session = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects/project.sessions-secondary/sessions")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "agentId": "agent.alpha",
                    "sessionId": "session.workspace-secondary",
                    "sessionKind": "assistant",
                    "entrySurface": "pc",
                    "idempotencyKey": "create-session.workspace-secondary",
                    "payloadHash": "sha256:create-session.workspace-secondary",
                    "requestedAt": "2026-07-26T00:00:01Z"
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(create_second_session).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let list = Request::builder()
            .method("GET")
            .uri(
                "/app/v3/api/ai/projects/project.sessions/sessions?page_size=20&status=active&include_archived=false",
            )
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(list).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            payload["data"]["items"][0]["sessionId"],
            "session.project-scoped"
        );

        let missing_state_list = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/user_states?session_ids=session.missing&include_hidden=true&page=1&page_size=20")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(missing_state_list).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert!(payload["data"]["items"].as_array().unwrap().is_empty());

        for (uri, expected_session_count) in [
            (
                "/app/v3/api/ai/agents/agent.alpha/sessions?page_size=20&include_archived=false"
                    .to_string(),
                2,
            ),
            (
                format!(
                    "/app/v3/api/ai/workspaces/{workspace_id}/sessions?page_size=20&include_archived=false"
                ),
                2,
            ),
        ] {
            let list = Request::builder()
                .method("GET")
                .uri(&uri)
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(list).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let payload: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                payload["data"]["items"].as_array().unwrap().len(),
                expected_session_count,
                "GET {uri} must return its complete scoped page"
            );
        }

        let invalid_query = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/projects/project.sessions/sessions?projectId=project.sessions")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(invalid_query).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn project_session_synchronize_enqueues_background_run_and_serves_cache_hits() {
        use crate::provider_session_sync::read_completed_provider_session_sync;

        crate::provider_session_sync::reset_provider_session_sync_cache_for_testing();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.sync-async", "sync-async").await;

        let create_project = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "projectId": "project.sync-async",
                    "name": "Sync async project",
                    "defaultAgentId": "agent.sync-async"
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(create_project).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        // Cold request: the run is enqueued on a background worker instead of
        // occupying the HTTP request → 202 accepted.
        let synchronize = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects/project.sync-async/sessions/synchronize")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(synchronize).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["status"], "accepted");
        assert_eq!(payload["data"]["item"]["projectId"], "project.sync-async");

        // Seed the completed outcome deterministically (the background worker
        // is best-effort; the no-engine-host run records the skipped outcome
        // synchronously here), then verify the refresh-cache fast path.
        let subject = PolicySubject {
            subject_id: "100".to_string(),
            tenant_id: "100001".to_string(),
            roles: vec!["ai.agents.manage".to_string()],
        };
        let service = Arc::clone(&state.service);
        let project = service
            .get_project(GetProjectCommand {
                tenant_id: 100001,
                organization_id: 0,
                project_id: "project.sync-async".to_string(),
                owner_scope: Some(100),
                requested_by: subject.clone(),
            })
            .expect("project must exist");
        crate::provider_session_sync::synchronize_project_provider_sessions(
            Arc::clone(&service),
            &project,
            subject,
        )
        .expect("no-engine-host synchronization must settle with a skipped issue");
        let cache_key = format!("100001/0/100:project.sync-async");
        assert!(
            read_completed_provider_session_sync(&cache_key).is_some(),
            "the completed outcome must be recorded"
        );
        let response = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects/project.sync-async/sessions/synchronize")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(response).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["status"], "completed");
        assert_eq!(payload["data"]["item"]["projectId"], "project.sync-async");
        assert!(
            payload["data"]["item"]["issues"]
                .as_array()
                .is_some_and(|issues| !issues.is_empty()),
            "the no-engine-host run must record a bounded aggregate issue"
        );
    }

    #[tokio::test]
    async fn read_only_subject_cannot_create_project() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let read_context = AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.read"]);
        let app = build_test_router_with_context(state, read_context);
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "name": "Denied project" }).to_string()))
            .expect("request should be built");

        let response = app
            .oneshot(request)
            .await
            .expect("project create request should complete");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn request_scope_rejects_zero_tenant_context() {
        let scope = RequestScope::from_context(AgentRequestContext::new("0", "100"));
        let err = scope
            .tenant_id_u64()
            .expect_err("tenant_id 0 must be rejected at the service scope boundary");
        assert!(err.message.contains("greater than 0"));
    }

    #[tokio::test]
    async fn app_session_user_state_routes_use_trusted_scope_and_optimistic_version() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.alpha", "alpha").await;

        let create_session = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "sessionId": "session.user-state",
                    "sessionKind": "assistant",
                    "entrySurface": "api",
                    "title": "User state contract",
                    "idempotencyKey": "create-session.user-state",
                    "payloadHash": "sha256:create-session.user-state",
                    "requestedAt": "2026-07-19T00:00:00Z"
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(create_session).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        with_service(&state, |service| {
            service.create_session_item(crate::application::CreateSessionItemCommand {
                tenant_id: 100_001,
                organization_id: 0,
                session_id: "session.user-state".to_string(),
                item_id: "item.assistant".to_string(),
                kind: crate::domain::AgentSessionItemKind::AssistantOutput,
                content: "Assistant answer".to_string(),
                content_type: "text/plain".to_string(),
                input_tokens: 0,
                output_tokens: 2,
                model_id: None,
                provider_id: None,
                provider_payload_json: None,
                parent_item_id: None,
                requested_by: sdkwork_agent_kernel::PolicySubject::new("u-1", "100001")
                    .with_role("ai.agents.manage"),
                requested_at: "2026-07-19T00:00:01Z".to_string(),
            })
        })
        .await
        .expect("assistant item fixture should be created");

        let pin = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/user_state")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "pinned": true }).to_string()))
            .unwrap();
        let response = app.clone().oneshot(pin).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["resourceId"], "session.user-state");
        assert_eq!(payload["data"]["item"]["version"], "0");
        assert!(payload["data"]["item"]["pinnedAt"].is_string());
        assert!(payload["data"]["item"].get("hiddenAt").is_none());

        let list = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/user_states?pinned_only=true&page=1")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(list).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["items"].as_array().unwrap().len(), 1);

        let missing_version = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/user_state")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "pinned": false }).to_string()))
            .unwrap();
        let response = app.clone().oneshot(missing_version).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let unpin = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/user_state")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "pinned": false, "expectedVersion": "0" }).to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(unpin).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["version"], "1");
        assert!(payload["data"]["item"].get("pinnedAt").is_none());

        let feedback = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/items/item.assistant/feedback")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "rating": "up" }).to_string()))
            .unwrap();
        let response = app.clone().oneshot(feedback).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["rating"], "up");
        assert_eq!(payload["data"]["item"]["version"], "0");

        let feedback_list = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/item_feedback")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(feedback_list).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["items"].as_array().unwrap().len(), 1);

        let clear_feedback = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/items/item.assistant/feedback")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "clearFeedback": true, "expectedVersion": "0" }).to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(clear_feedback).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["version"], "1");
        assert!(payload["data"]["item"]["deletedAt"].is_string());
    }

    #[tokio::test]
    async fn app_session_item_get_is_read_only_and_post_synchronization_is_cursor_bounded() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.alpha", "alpha").await;

        let create_session = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "sessionId": "session.item-sync-contract",
                    "sessionKind": "assistant",
                    "entrySurface": "api",
                    "title": "Session item synchronization contract",
                    "idempotencyKey": "create-session.item-sync-contract",
                    "payloadHash": "sha256:create-session.item-sync-contract",
                    "requestedAt": "2026-07-30T00:00:00Z"
                })
                .to_string(),
            ))
            .expect("create Session request");
        let response = app.clone().oneshot(create_session).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        with_service(&state, |service| {
            service.create_session_item(crate::application::CreateSessionItemCommand {
                tenant_id: 100_001,
                organization_id: 0,
                session_id: "session.item-sync-contract".to_string(),
                item_id: "item.item-sync-contract".to_string(),
                kind: crate::domain::AgentSessionItemKind::AssistantOutput,
                content: "Canonical Session Item".to_string(),
                content_type: "text/plain".to_string(),
                input_tokens: 0,
                output_tokens: 3,
                model_id: None,
                provider_id: None,
                provider_payload_json: None,
                parent_item_id: None,
                requested_by: sdkwork_agent_kernel::PolicySubject::new("100", "100001")
                    .with_role("ai.agents.manage"),
                requested_at: "2026-07-30T00:00:01Z".to_string(),
            })
        })
        .await
        .expect("Session Item fixture");

        let item_window_uri = "/app/v3/api/ai/agents/agent.alpha/sessions/\
            session.item-sync-contract/items?page_size=50&sort=-sequence"
            .replace(char::is_whitespace, "");
        let synchronization_uri = "/app/v3/api/ai/agents/agent.alpha/sessions/\
            session.item-sync-contract/items/synchronize"
            .replace(char::is_whitespace, "");
        let read_window = |method: &str, uri: &str| {
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("Session Item window request")
        };

        let response = app
            .clone()
            .oneshot(read_window("GET", &item_window_uri))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let get_payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(get_payload["data"]["pageInfo"]["mode"], "cursor");
        assert_eq!(get_payload["data"]["pageInfo"]["hasMore"], false);
        assert_eq!(
            get_payload["data"]["items"][0]["itemId"],
            "item.item-sync-contract"
        );

        // The synchronization command reports its best-effort outcome and
        // never returns the item window (API_SPEC §14.1.3); a non-agent engine
        // Session reports a skipped synchronization instead of hiding it.
        let response = app
            .clone()
            .oneshot(read_window("POST", &synchronization_uri))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let synchronization_payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            synchronization_payload["data"],
            json!({
                "item": {
                    "status": "not-provider-session",
                    "importedItemCount": "0",
                }
            })
        );

        let response = app
            .clone()
            .oneshot(read_window(
                "POST",
                &format!("{synchronization_uri}?cursor=continuation-token"),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(read_window("GET", &item_window_uri))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let repeated_get_payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(repeated_get_payload["data"], get_payload["data"]);
    }
}
