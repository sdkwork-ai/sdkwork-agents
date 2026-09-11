//! Command and result types for the agents application service.
//!
//! Each command is a self-contained value object that carries all the
//! information needed for one service operation: the tenant scope, the
//! target entity, optional optimistic-concurrency version, the requesting
//! subject, and the request timestamp.

use crate::agent_turn::AgentTurnRecord;
use crate::domain::{
    AgentAuditAction, AgentBusinessStatus, AgentCompositionSlotKind, AgentCompositionTargetModule,
    AgentImplementationKind, AgentImplementationType, AgentInteractionKind,
    AgentItemDriveRefRecord, AgentItemFeedbackRating, AgentItemFeedbackRecord,
    AgentItemResourceRole, AgentResourceUserStateRecord, AgentSessionCheckpointRecord,
    AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus,
    AgentSessionKind, AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentVisibility,
};
use crate::ports::{
    AgentListQuery, AuditEventListQuery, CompositionSlotListQuery, InteractionListQuery,
    ItemFeedbackListQuery, McpMarketplaceListQuery, ProjectCompositionSlotListQuery,
    ProjectListQuery, ProviderBindingListQuery, ResourceUserStateListQuery,
    SessionActivitySummaryListQuery, SessionCheckpointListQuery, SessionItemListQuery,
    SessionListQuery, SessionRuntimeBindingListQuery, TaskListQuery, TurnListQuery,
};
use crate::project::{AgentProjectDriveAccessMode, AgentProjectVisibility};
use crate::task_scheduling::{
    AgentTaskMisfirePolicy, AgentTaskOverlapPolicy, AgentTaskRunAttemptRecord, AgentTaskRunRecord,
    AgentTaskRunStatus, AgentTaskScheduleKind,
};
use crate::{PaginatedResult, TaskRunAttemptListQuery, TaskRunListQuery};
use sdkwork_agent_kernel::{AgentManifest, KernelEvent, PolicySubject};
use sdkwork_code_kernel::CodeTaskIntent;

// ---------------------------------------------------------------------------
// Audit event input (internal)
// ---------------------------------------------------------------------------

/// Input for marketplace/composition-slot audit events.
///
/// This is an internal type used by [`super::AgentsService`] to pass
/// audit metadata to `emit_marketplace_audit_event`.
pub(super) struct AgentBusinessAuditEventInput<'a> {
    pub action: AgentAuditAction,
    pub item_kind: &'a str,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub item_id: &'a str,
    pub status: AgentBusinessStatus,
    pub visibility: AgentVisibility,
    pub version: u64,
    pub subject: PolicySubject,
    pub occurred_at: String,
}

// ---------------------------------------------------------------------------
// Agent business commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAgentCommand {
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest: AgentManifest,
    pub visibility: AgentVisibility,
    pub tags: Vec<String>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<AgentImplementationKind>,
    pub implementation_type: Option<AgentImplementationType>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub manifest: Option<AgentManifest>,
    pub visibility: Option<AgentVisibility>,
    pub tags: Option<Vec<String>>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<Option<String>>,
    pub implementation_kind: Option<Option<AgentImplementationKind>>,
    pub implementation_type: Option<AgentImplementationType>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeAgentStatusCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub target_status: AgentBusinessStatus,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAgentsCommand {
    pub query: AgentListQuery,
    pub requested_by: PolicySubject,
}

// ---------------------------------------------------------------------------
// Provider binding commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: AgentImplementationKind,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub make_default: bool,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateAgentProviderBindingCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

// ---------------------------------------------------------------------------
// Runtime execution commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct AgentPreviewResponseCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub content: String,
    pub debug_mode: bool,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub input_payload_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPromptOptimizationCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub prompt: String,
    pub input_payload_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

// ---------------------------------------------------------------------------
// Composition slot commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotCreateCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub slot_kind: AgentCompositionSlotKind,
    pub target_module: AgentCompositionTargetModule,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub policy_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotUpdateCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub slot_kind: Option<AgentCompositionSlotKind>,
    pub target_module: Option<AgentCompositionTargetModule>,
    pub target_ref: Option<String>,
    pub target_version_ref: Option<Option<String>>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub policy_json: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotDeleteCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBindingListCommand {
    pub query: ProviderBindingListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAgentAuditEventsCommand {
    pub query: AuditEventListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListMcpMarketplaceCommand {
    pub query: McpMarketplaceListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotListCommand {
    pub query: CompositionSlotListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotGetCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub requested_by: PolicySubject,
}

// ---------------------------------------------------------------------------
// Session commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub workspace_id: Option<String>,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub visibility: AgentProjectVisibility,
    pub drive_access_mode: AgentProjectDriveAccessMode,
    pub default_agent_id: Option<String>,
    pub default_model_id: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsureDefaultWorkspaceCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub default_name: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateWorkspaceCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetWorkspaceCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub workspace_id: String,
    pub owner_user_id: u64,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateWorkspaceCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub workspace_id: String,
    pub owner_user_id: u64,
    pub expected_version: Option<u64>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMutationCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub workspace_id: String,
    pub owner_user_id: u64,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListWorkspacesCommand {
    pub query: crate::ports::WorkspaceListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportProjectCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub workspace_id: String,
    pub project_id: String,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub source_kind: String,
    pub source_ref: String,
    pub drive_space_id: String,
    pub drive_root_entry_id: String,
    pub drive_logical_path: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProjectCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_scope: Option<u64>,
    pub expected_version: Option<u64>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub visibility: Option<AgentProjectVisibility>,
    pub drive_access_mode: Option<AgentProjectDriveAccessMode>,
    pub default_agent_id: Option<Option<String>>,
    pub default_model_id: Option<Option<String>>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMutationCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_scope: Option<u64>,
    pub expected_version: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetProjectCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListProjectsCommand {
    pub query: ProjectListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub slot_kind: AgentCompositionSlotKind,
    pub target_module: AgentCompositionTargetModule,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub policy_json: String,
    pub owner_scope: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub slot_kind: Option<AgentCompositionSlotKind>,
    pub target_module: Option<AgentCompositionTargetModule>,
    pub target_ref: Option<String>,
    pub target_version_ref: Option<Option<String>>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub policy_json: Option<String>,
    pub owner_scope: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListProjectCompositionSlotsCommand {
    pub query: ProjectCompositionSlotListQuery,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub project_id: Option<String>,
    pub session_id: String,
    pub session_kind: AgentSessionKind,
    pub entry_surface: AgentSessionEntrySurface,
    pub source_module: Option<String>,
    pub source_context_kind: Option<String>,
    pub source_context_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub forked_from_turn_id: Option<String>,
    pub title: Option<String>,
    pub idempotency_key: Option<String>,
    pub payload_hash: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded session.
    pub path_agent_id: String,
    pub session_id: String,
    pub title: Option<String>,
    pub project_id: Option<Option<String>>,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded session.
    pub path_agent_id: String,
    pub session_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded session.
    pub path_agent_id: String,
    pub session_id: String,
    pub expected_version: Option<u64>,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded session.
    pub path_agent_id: String,
    pub session_id: String,
    pub expected_version: Option<u64>,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded session.
    pub path_agent_id: String,
    pub session_id: String,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetProjectSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub session_id: String,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionsCommand {
    pub query: SessionListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionActivitySummariesCommand {
    pub query: SessionActivitySummaryListQuery,
    pub requested_by: PolicySubject,
}

// ---------------------------------------------------------------------------
// Task commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub session_id: String,
    pub task_id: String,
    pub title: Option<String>,
    pub prompt: String,
    pub schedule_kind: AgentTaskScheduleKind,
    pub cron_expression: Option<String>,
    pub timezone: String,
    pub scheduled_at: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub misfire_policy: AgentTaskMisfirePolicy,
    pub overlap_policy: AgentTaskOverlapPolicy,
    pub max_concurrent_runs: u16,
    pub max_catch_up_runs: u16,
    pub max_attempts: u16,
    pub retry_initial_delay_seconds: u32,
    pub retry_max_delay_seconds: u32,
    pub timeout_seconds: u32,
    pub priority: i16,
    pub external_ref: Option<String>,
    pub metadata_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Nested route `{agentId}`; must match the loaded task.
    pub path_agent_id: String,
    pub task_id: String,
    pub expected_version: Option<u64>,
    /// When set, the task must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub title: Option<String>,
    pub prompt: String,
    pub schedule_kind: AgentTaskScheduleKind,
    pub cron_expression: Option<String>,
    pub timezone: String,
    pub scheduled_at: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub misfire_policy: AgentTaskMisfirePolicy,
    pub overlap_policy: AgentTaskOverlapPolicy,
    pub max_concurrent_runs: u16,
    pub max_catch_up_runs: u16,
    pub max_attempts: u16,
    pub retry_initial_delay_seconds: u32,
    pub retry_max_delay_seconds: u32,
    pub timeout_seconds: u32,
    pub priority: i16,
    pub external_ref: Option<String>,
    pub metadata_json: String,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecuteTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub idempotency_key: String,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded task.
    pub path_agent_id: String,
    pub task_id: String,
    /// When set, the task must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTasksCommand {
    pub query: TaskListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTaskRunsCommand {
    pub query: TaskRunListQuery,
    pub path_agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTaskRunCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub run_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryTaskRunCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub run_id: String,
    pub idempotency_key: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelTaskRunCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub run_id: String,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTaskRunAttemptsCommand {
    pub query: TaskRunAttemptListQuery,
    pub path_agent_id: String,
    pub task_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunReconciliationOutcome {
    Succeeded,
    Failed,
    Cancelled,
}

impl TaskRunReconciliationOutcome {
    pub fn terminal_status(self) -> AgentTaskRunStatus {
        match self {
            Self::Succeeded => AgentTaskRunStatus::Succeeded,
            Self::Failed => AgentTaskRunStatus::Failed,
            Self::Cancelled => AgentTaskRunStatus::Cancelled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileTaskRunCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub run_id: String,
    pub outcome: TaskRunReconciliationOutcome,
    pub error_code: Option<String>,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunReconciliationResult {
    pub examined: usize,
    pub reconciled: Vec<AgentTaskRunRecord>,
    pub pending: usize,
    pub skipped_conflicts: usize,
}

pub type TaskRunPage = PaginatedResult<AgentTaskRunRecord>;
pub type TaskRunAttemptPage = PaginatedResult<AgentTaskRunAttemptRecord>;

// ---------------------------------------------------------------------------
// Session item and turn commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionItemCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub item_id: String,
    pub kind: AgentSessionItemKind,
    pub content: String,
    pub content_type: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_payload_json: Option<String>,
    pub parent_item_id: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReconcileProviderSessionHistoryItemCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub item_id: String,
    pub kind: AgentSessionItemKind,
    pub content: Option<String>,
    pub content_type: String,
    pub status: AgentSessionItemStatus,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_arguments_json: Option<String>,
    pub tool_result_json: Option<String>,
    pub provider_payload_json: Option<String>,
    pub parent_item_id: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSessionItemCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded item.
    pub path_agent_id: String,
    pub session_id: String,
    pub item_id: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionItemsCommand {
    pub query: SessionItemListQuery,
    /// Agent id from the nested HTTP path; must match the loaded session.
    pub path_agent_id: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentItemDriveRefInput {
    pub resource_role: AgentItemResourceRole,
    pub drive_space_id: String,
    pub drive_node_id: String,
}

/// Execute one user-input turn and produce an assistant-output item.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTurnCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub content: String,
    pub content_type: String,
    pub turn_mode: crate::agent_turn::AgentTurnMode,
    pub runtime_binding_id: Option<String>,
    pub requested_model_id: Option<String>,
    pub access_mode_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub client_request_id: Option<String>,
    pub drive_refs: Vec<AgentItemDriveRefInput>,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
    /// When true, attempt provider stream chunks for SSE item delta events.
    pub prefer_stream: bool,
    /// Agent system prompt injected ahead of the turn history (from the
    /// authenticated client request; never persisted on the turn record).
    pub system_prompt: Option<String>,
    /// Original user auth token from the authenticated request (transient —
    /// never persisted). Enables cloudrouter account-pool routing execution.
    pub auth_token: Option<String>,
    /// Original user access token from the authenticated request (transient —
    /// never persisted). Paired with `auth_token` for dual-token access to
    /// the cloudrouter gateway (`Authorization: Bearer` + `Access-Token`).
    pub access_token: Option<String>,
}

/// Result of one completed Turn with its complete authoritative item set.
#[derive(Debug, Clone, PartialEq)]
pub struct TurnExecutionResult {
    pub session: AgentSessionRecord,
    pub turn: AgentTurnRecord,
    pub user_input_item: AgentSessionItemRecord,
    pub assistant_output_item: AgentSessionItemRecord,
    pub turn_items: Vec<AgentSessionItemRecord>,
    pub user_item_drive_refs: Vec<AgentItemDriveRefRecord>,
    pub stream_deltas: Vec<String>,
    pub stream_events: Vec<KernelEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionItemWithDriveRefs {
    pub item: AgentSessionItemRecord,
    pub drive_refs: Vec<AgentItemDriveRefRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTurnCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTurnsCommand {
    pub query: TurnListQuery,
    pub path_agent_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTurnByIdempotencyCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub owner_user_id: u64,
    pub idempotency_key: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelTurnCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnReconciliationResult {
    pub examined: usize,
    pub failed: Vec<AgentTurnRecord>,
    pub skipped_conflicts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionUserStatesCommand {
    pub query: ResourceUserStateListQuery,
    pub path_agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSessionUserStateCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSessionUserStateCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub pinned: Option<bool>,
    pub hidden: Option<bool>,
    pub mark_opened: bool,
    pub last_read_item_sequence: Option<u64>,
    pub custom_title: Option<Option<String>>,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

pub type SessionUserStateResult = AgentResourceUserStateRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItemFeedbackCommand {
    pub query: ItemFeedbackListQuery,
    pub path_agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateItemFeedbackCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub item_id: String,
    pub rating: Option<AgentItemFeedbackRating>,
    pub reason_code: Option<String>,
    pub comment: Option<String>,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

pub type ItemFeedbackResult = AgentItemFeedbackRecord;

// ---------------------------------------------------------------------------
// Interaction commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateInteractionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub path_agent_id: String,
    pub interaction_id: String,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub provider_interaction_id: Option<String>,
    pub kind: AgentInteractionKind,
    pub prompt: String,
    pub options_json: String,
    pub request_json: Option<String>,
    pub retention_until: Option<String>,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimInteractionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    pub claim_owner: String,
    pub lease_seconds: u32,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionClaimResult {
    pub interaction: crate::domain::AgentInteractionRecord,
    pub claim_token: String,
    pub claim_expires_at: String,
    pub fencing_token: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListInteractionsCommand {
    pub query: InteractionListQuery,
    /// Agent id from the nested HTTP path; must match the parent session.
    pub path_agent_id: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetInteractionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded interaction.
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproveInteractionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the parent session.
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    pub approved: bool,
    pub reason: Option<String>,
    pub claim_token: String,
    pub fencing_token: u64,
    pub expected_version: u64,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerInteractionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the parent session.
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    pub answer: String,
    pub selected_option_value: Option<String>,
    pub rejected: bool,
    pub claim_token: String,
    pub fencing_token: u64,
    pub expected_version: u64,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveInteractionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Agent id from the nested HTTP path; must match the parent session.
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    pub resolution_json: String,
    pub claim_token: String,
    pub fencing_token: u64,
    pub expected_version: u64,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersistProviderInteractionEventCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub requested_by: PolicySubject,
    pub received_at: String,
    pub event: KernelEvent,
}

// ---------------------------------------------------------------------------
// Session runtime binding commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionRuntimeBindingCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub runtime_binding_id: Option<String>,
    pub runtime_location_id: Option<String>,
    pub host_mode: String,
    pub transport_kind: String,
    pub provider_binding_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub provider_session_id: Option<String>,
    pub provider_session_tree_id: Option<String>,
    pub provider_parent_session_id: Option<String>,
    pub provider_forked_from_session_id: Option<String>,
    pub provider_directory: Option<sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionRuntimeBindingsCommand {
    pub query: SessionRuntimeBindingListQuery,
    pub path_agent_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSessionRuntimeBindingCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub runtime_binding_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSessionRuntimeBindingCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub runtime_binding_id: String,
    pub runtime_location_id: Option<Option<String>>,
    pub host_mode: Option<String>,
    pub transport_kind: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_session_id: Option<String>,
    pub provider_session_tree_id: Option<String>,
    pub provider_parent_session_id: Option<String>,
    pub provider_forked_from_session_id: Option<String>,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReconcileProviderSessionRuntimeBindingDirectoryCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub runtime_binding_id: String,
    pub expected_version: u64,
    pub provider_directory: sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSessionRuntimeBindingStatusCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub runtime_binding_id: String,
    pub expected_version: u64,
    pub reason: Option<String>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

// ---------------------------------------------------------------------------
// Session checkpoint commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionCheckpointCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub checkpoint_id: Option<String>,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub checkpoint_kind: String,
    pub provider_checkpoint_ref: Option<String>,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub resumable: bool,
    pub retention_until: Option<String>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionCheckpointsCommand {
    pub query: SessionCheckpointListQuery,
    pub path_agent_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSessionCheckpointCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub checkpoint_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSessionCheckpointStatusCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub checkpoint_id: String,
    pub expected_version: u64,
    pub reason: Option<String>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

pub type SessionRuntimeBindingResult = AgentSessionRuntimeBindingRecord;
pub type SessionCheckpointResult = AgentSessionCheckpointRecord;
