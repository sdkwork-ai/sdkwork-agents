use sdkwork_agent_kernel::AgentManifest;
use sdkwork_code_kernel::CodeTaskIntent;

pub const DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY: &str = "agent.business.manage";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentBusinessStatus {
    Draft,
    Active,
    Disabled,
    Archived,
    Deleted,
}

impl AgentBusinessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "active" => Some(Self::Active),
            "disabled" => Some(Self::Disabled),
            "archived" => Some(Self::Archived),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Draft => 0,
            Self::Active => 1,
            Self::Disabled => 2,
            Self::Archived => 3,
            Self::Deleted => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Draft),
            1 => Some(Self::Active),
            2 => Some(Self::Disabled),
            3 => Some(Self::Archived),
            4 => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentVisibility {
    Private,
    Organization,
    Tenant,
    Public,
}

impl AgentVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Organization => "organization",
            Self::Tenant => "tenant",
            Self::Public => "public",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "private" => Some(Self::Private),
            "organization" => Some(Self::Organization),
            "tenant" => Some(Self::Tenant),
            "public" => Some(Self::Public),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Private => 0,
            Self::Organization => 1,
            Self::Tenant => 2,
            Self::Public => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Private),
            1 => Some(Self::Organization),
            2 => Some(Self::Tenant),
            3 => Some(Self::Public),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAuditAction {
    Create,
    Update,
    Delete,
    Restore,
    ChangeStatus,
    RuntimeExecutionCompleted,
    RuntimeExecutionQueued,
    RuntimeExecutionFailed,
    ProviderBindingChanged,
    CompositionSlotCreated,
    CompositionSlotUpdated,
    CompositionSlotDeleted,
    SessionCreated,
    SessionRenamed,
    SessionMoved,
    SessionClosed,
    SessionArchived,
    SessionDeleted,
    SessionItemCreated,
    SessionItemFailed,
    ItemFeedbackChanged,
    InteractionCreated,
    InteractionClaimed,
    InteractionResolved,
    InteractionRejected,
    InteractionExpired,
    InteractionCancelled,
    TaskCreated,
    TaskUpdated,
    TaskPaused,
    TaskResumed,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    TaskRunCreated,
    TaskRunCancelRequested,
    TaskRunReconciled,
    WorkspaceCreated,
    WorkspaceUpdated,
    WorkspaceArchived,
    WorkspaceDeleted,
    ProjectCreated,
    ProjectUpdated,
    ProjectArchived,
    ProjectDeleted,
    ProjectCompositionSlotCreated,
    ProjectCompositionSlotUpdated,
    ProjectCompositionSlotDeleted,
    TurnRequested,
    TurnCompleted,
    TurnFailed,
    TurnCancelRequested,
    TurnCancelled,
    SessionRuntimeBindingCreated,
    SessionRuntimeBindingUpdated,
    SessionRuntimeBindingActivated,
    SessionRuntimeBindingDeactivated,
    SessionCheckpointCreated,
    SessionCheckpointRestored,
    SessionCheckpointInvalidated,
}

impl AgentAuditAction {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Create => "agent.business.created",
            Self::Update => "agent.business.updated",
            Self::Delete => "agent.business.deleted",
            Self::Restore => "agent.business.restored",
            Self::ChangeStatus => "agent.business.status_changed",
            Self::RuntimeExecutionCompleted => "agent.business.runtime.executed",
            Self::RuntimeExecutionQueued => "agent.business.runtime.queued",
            Self::RuntimeExecutionFailed => "agent.business.runtime.failed",
            Self::ProviderBindingChanged => "agent.business.provider_binding_changed",
            Self::CompositionSlotCreated => "agent.business.composition_slot.created",
            Self::CompositionSlotUpdated => "agent.business.composition_slot.updated",
            Self::CompositionSlotDeleted => "agent.business.composition_slot.deleted",
            Self::SessionCreated => "agent.business.session.created",
            Self::SessionRenamed => "agent.business.session.renamed",
            Self::SessionMoved => "agent.business.session.moved",
            Self::SessionClosed => "agent.business.session.closed",
            Self::SessionArchived => "agent.business.session.archived",
            Self::SessionDeleted => "agent.business.session.deleted",
            Self::SessionItemCreated => "agent.business.session_item.created",
            Self::SessionItemFailed => "agent.business.session_item.failed",
            Self::ItemFeedbackChanged => "agent.business.item_feedback.changed",
            Self::InteractionCreated => "agent.business.interaction.created",
            Self::InteractionClaimed => "agent.business.interaction.claimed",
            Self::InteractionResolved => "agent.business.interaction.resolved",
            Self::InteractionRejected => "agent.business.interaction.rejected",
            Self::InteractionExpired => "agent.business.interaction.expired",
            Self::InteractionCancelled => "agent.business.interaction.cancelled",
            Self::TaskCreated => "agent.business.task.created",
            Self::TaskUpdated => "agent.business.task.updated",
            Self::TaskPaused => "agent.business.task.paused",
            Self::TaskResumed => "agent.business.task.resumed",
            Self::TaskCompleted => "agent.business.task.completed",
            Self::TaskFailed => "agent.business.task.failed",
            Self::TaskCancelled => "agent.business.task.cancelled",
            Self::TaskRunCreated => "agent.business.task_run.created",
            Self::TaskRunCancelRequested => "agent.business.task_run.cancel_requested",
            Self::TaskRunReconciled => "agent.business.task_run.reconciled",
            Self::WorkspaceCreated => "agent.business.workspace.created",
            Self::WorkspaceUpdated => "agent.business.workspace.updated",
            Self::WorkspaceArchived => "agent.business.workspace.archived",
            Self::WorkspaceDeleted => "agent.business.workspace.deleted",
            Self::ProjectCreated => "agent.business.project.created",
            Self::ProjectUpdated => "agent.business.project.updated",
            Self::ProjectArchived => "agent.business.project.archived",
            Self::ProjectDeleted => "agent.business.project.deleted",
            Self::ProjectCompositionSlotCreated => {
                "agent.business.project.composition_slot.created"
            }
            Self::ProjectCompositionSlotUpdated => {
                "agent.business.project.composition_slot.updated"
            }
            Self::ProjectCompositionSlotDeleted => {
                "agent.business.project.composition_slot.deleted"
            }
            Self::TurnRequested => "agent.business.turn.requested",
            Self::TurnCompleted => "agent.business.turn.completed",
            Self::TurnFailed => "agent.business.turn.failed",
            Self::TurnCancelRequested => "agent.business.turn.cancel_requested",
            Self::TurnCancelled => "agent.business.turn.cancelled",
            Self::SessionRuntimeBindingCreated => "agent.business.session_runtime_binding.created",
            Self::SessionRuntimeBindingUpdated => "agent.business.session_runtime_binding.updated",
            Self::SessionRuntimeBindingActivated => {
                "agent.business.session_runtime_binding.activated"
            }
            Self::SessionRuntimeBindingDeactivated => {
                "agent.business.session_runtime_binding.deactivated"
            }
            Self::SessionCheckpointCreated => "agent.business.session_checkpoint.created",
            Self::SessionCheckpointRestored => "agent.business.session_checkpoint.restored",
            Self::SessionCheckpointInvalidated => "agent.business.session_checkpoint.invalidated",
        }
    }

    pub fn action_code(&self) -> &'static str {
        match self {
            Self::Create => "created",
            Self::Update => "updated",
            Self::Delete => "deleted",
            Self::Restore => "restored",
            Self::ChangeStatus => "status_changed",
            Self::RuntimeExecutionCompleted => "runtime_executed",
            Self::RuntimeExecutionQueued => "runtime_queued",
            Self::RuntimeExecutionFailed => "runtime_failed",
            Self::ProviderBindingChanged => "provider_binding_changed",
            Self::CompositionSlotCreated => "composition_slot_created",
            Self::CompositionSlotUpdated => "composition_slot_updated",
            Self::CompositionSlotDeleted => "composition_slot_deleted",
            Self::SessionCreated => "session_created",
            Self::SessionRenamed => "session_renamed",
            Self::SessionMoved => "session_moved",
            Self::SessionClosed => "session_closed",
            Self::SessionArchived => "session_archived",
            Self::SessionDeleted => "session_deleted",
            Self::SessionItemCreated => "session_item_created",
            Self::SessionItemFailed => "session_item_failed",
            Self::ItemFeedbackChanged => "item_feedback_changed",
            Self::InteractionCreated => "interaction_created",
            Self::InteractionClaimed => "interaction_claimed",
            Self::InteractionResolved => "interaction_resolved",
            Self::InteractionRejected => "interaction_rejected",
            Self::InteractionExpired => "interaction_expired",
            Self::InteractionCancelled => "interaction_cancelled",
            Self::TaskCreated => "task_created",
            Self::TaskUpdated => "task_updated",
            Self::TaskPaused => "task_paused",
            Self::TaskResumed => "task_resumed",
            Self::TaskCompleted => "task_completed",
            Self::TaskFailed => "task_failed",
            Self::TaskCancelled => "task_cancelled",
            Self::TaskRunCreated => "task_run_created",
            Self::TaskRunCancelRequested => "task_run_cancel_requested",
            Self::TaskRunReconciled => "task_run_reconciled",
            Self::WorkspaceCreated => "workspace_created",
            Self::WorkspaceUpdated => "workspace_updated",
            Self::WorkspaceArchived => "workspace_archived",
            Self::WorkspaceDeleted => "workspace_deleted",
            Self::ProjectCreated => "project_created",
            Self::ProjectUpdated => "project_updated",
            Self::ProjectArchived => "project_archived",
            Self::ProjectDeleted => "project_deleted",
            Self::ProjectCompositionSlotCreated => "project_composition_slot_created",
            Self::ProjectCompositionSlotUpdated => "project_composition_slot_updated",
            Self::ProjectCompositionSlotDeleted => "project_composition_slot_deleted",
            Self::TurnRequested => "turn_requested",
            Self::TurnCompleted => "turn_completed",
            Self::TurnFailed => "turn_failed",
            Self::TurnCancelRequested => "turn_cancel_requested",
            Self::TurnCancelled => "turn_cancelled",
            Self::SessionRuntimeBindingCreated => "session_runtime_binding_created",
            Self::SessionRuntimeBindingUpdated => "session_runtime_binding_updated",
            Self::SessionRuntimeBindingActivated => "session_runtime_binding_activated",
            Self::SessionRuntimeBindingDeactivated => "session_runtime_binding_deactivated",
            Self::SessionCheckpointCreated => "session_checkpoint_created",
            Self::SessionCheckpointRestored => "session_checkpoint_restored",
            Self::SessionCheckpointInvalidated => "session_checkpoint_invalidated",
        }
    }
}

/// Structured audit event payload for agent business operations.
///
/// This payload format replaces the legacy string concatenation approach,
/// providing a versioned, parseable JSON structure for audit events.
/// The `schema_version` field enables future schema evolution without
/// breaking existing consumers.
///
/// # Version History
/// - v1: Initial structured payload format
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentAuditPayload {
    /// Schema version for forward/backward compatibility
    pub schema_version: String,
    /// The action that triggered this audit event
    pub action: String,
    /// Agent identifier
    pub agent_id: String,
    /// Tenant identifier for multi-tenant isolation
    pub tenant_id: u64,
    /// Organization identifier within tenant
    pub organization_id: u64,
    /// User or service that owns this agent
    pub owner_user_id: u64,
    /// Agent business code (human-readable identifier)
    pub code: String,
    /// Current status after the action
    pub status: String,
    /// Visibility level
    pub visibility: String,
    /// Entity version after the action
    pub version: u64,
    /// Previous status (for status change events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_status: Option<String>,
}

impl AgentAuditPayload {
    /// Current schema version constant
    pub const SCHEMA_VERSION: &'static str = "v1";

    /// Create a new audit payload for agent business events
    pub fn new(action: AgentAuditAction, record: &AgentBusinessRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            code: record.code.clone(),
            status: record.status.as_str().to_string(),
            visibility: record.visibility.as_str().to_string(),
            version: record.version,
            previous_status: None,
        }
    }

    /// Add previous status for status change events
    pub fn with_previous_status(mut self, previous_status: AgentBusinessStatus) -> Self {
        self.previous_status = Some(previous_status.as_str().to_string());
        self
    }

    /// Convert to JSON string for storage
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for provider binding operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProviderBindingAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: String,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub active: bool,
    pub version: u64,
}

impl ProviderBindingAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentProviderBindingRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            binding_id: record.binding_id.clone(),
            provider_id: record.provider_id.clone(),
            implementation_kind: record.implementation_kind.as_str().to_string(),
            configuration_profile_id: record.configuration_profile_id.clone(),
            capabilities: record.capabilities.clone(),
            active: record.active,
            version: record.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for runtime execution operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeExecutionAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub execution_id: String,
    pub operation: String,
    pub status: String,
}

impl RuntimeExecutionAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentRuntimeExecutionRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            execution_id: record.execution_id.clone(),
            operation: record.operation.as_str().to_string(),
            status: record.status.as_str().to_string(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for marketplace/composition slot operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarketplaceAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub item_kind: String,
    pub item_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub status: String,
    pub visibility: String,
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketplaceAuditPayloadInput<'a> {
    pub action: AgentAuditAction,
    pub item_kind: &'a str,
    pub item_id: &'a str,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub status: AgentBusinessStatus,
    pub visibility: AgentVisibility,
    pub version: u64,
}

impl MarketplaceAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(input: MarketplaceAuditPayloadInput<'_>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: input.action.action_code().to_string(),
            item_kind: input.item_kind.to_string(),
            item_id: input.item_id.to_string(),
            tenant_id: input.tenant_id,
            organization_id: input.organization_id,
            status: input.status.as_str().to_string(),
            visibility: input.visibility.as_str().to_string(),
            version: input.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for agent session operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub session_id: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub status: String,
    pub item_count: u64,
    pub version: u64,
}

impl SessionAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentSessionRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            status: record.status.as_str().to_string(),
            item_count: record.item_count,
            version: record.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for agent session-item operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionItemAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub item_id: String,
    pub session_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub kind: String,
    pub status: String,
    pub sequence: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl SessionItemAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentSessionItemRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            item_id: record.item_id.clone(),
            session_id: record.session_id.clone(),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            kind: record.kind.as_str().to_string(),
            status: record.status.as_str().to_string(),
            sequence: record.sequence,
            input_tokens: record.input_tokens,
            output_tokens: record.output_tokens,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRuntimeExecutionOperation {
    PreviewResponse,
    PromptOptimization,
    AgentCall,
}

impl AgentRuntimeExecutionOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PreviewResponse => "preview_response",
            Self::PromptOptimization => "prompt_optimization",
            Self::AgentCall => "agent_call",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "preview_response" => Some(Self::PreviewResponse),
            "prompt_optimization" => Some(Self::PromptOptimization),
            "agent_call" => Some(Self::AgentCall),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRuntimeExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl AgentRuntimeExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Delivery mode of one structured agent call (`agents.calls.create`).
///
/// `Sync` executes the call inside the HTTP request and returns the terminal
/// record. `Async` persists a `queued` runtime-execution record, returns 202,
/// and the terminal state becomes observable through `agents.calls.retrieve`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentCallExecutionMode {
    #[default]
    Sync,
    Async,
}

impl AgentCallExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::Async => "async",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "sync" => Some(Self::Sync),
            "async" => Some(Self::Async),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeExecutionRecord {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub operation: AgentRuntimeExecutionOperation,
    pub status: AgentRuntimeExecutionStatus,
    pub input_payload_json: String,
    pub output_payload_json: String,
    pub requested_at: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentImplementationKind {
    ManifestOnly,
    TypedLocalProvider,
    ProcessAdapter,
    ProtocolAdapter,
}

impl AgentImplementationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManifestOnly => "manifest-only",
            Self::TypedLocalProvider => "typed-local-provider",
            Self::ProcessAdapter => "process-adapter",
            Self::ProtocolAdapter => "protocol-adapter",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "manifest-only" => Some(Self::ManifestOnly),
            "typed-local-provider" => Some(Self::TypedLocalProvider),
            "process-adapter" => Some(Self::ProcessAdapter),
            "protocol-adapter" => Some(Self::ProtocolAdapter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentImplementationType {
    #[default]
    SdkworkNative,
    RigRust,
    OpenAiAgents,
    LangChain,
    LangGraph,
    CrewAi,
    AutoGen,
    SemanticKernel,
    Custom,
}

impl AgentImplementationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SdkworkNative => "sdkwork-native",
            Self::RigRust => "rig-rust",
            Self::OpenAiAgents => "openai-agents",
            Self::LangChain => "langchain",
            Self::LangGraph => "langgraph",
            Self::CrewAi => "crewai",
            Self::AutoGen => "autogen",
            Self::SemanticKernel => "semantic-kernel",
            Self::Custom => "custom",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "sdkwork-native" => Some(Self::SdkworkNative),
            "rig-rust" => Some(Self::RigRust),
            "openai-agents" => Some(Self::OpenAiAgents),
            "langchain" => Some(Self::LangChain),
            "langgraph" => Some(Self::LangGraph),
            "crewai" => Some(Self::CrewAi),
            "autogen" => Some(Self::AutoGen),
            "semantic-kernel" => Some(Self::SemanticKernel),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBusinessRecord {
    pub id: u64,
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest: AgentManifest,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<AgentImplementationKind>,
    pub implementation_type: AgentImplementationType,
    pub status: AgentBusinessStatus,
    pub visibility: AgentVisibility,
    pub tags: Vec<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// Immutable published snapshot of one agent definition (`agents.versions.*`).
/// A version row is write-once: manifest and metadata never change after
/// creation; activation is tracked by `activated_at` (exactly one active
/// version per agent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentVersionRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub version_id: String,
    pub version_number: u64,
    pub manifest_json: String,
    pub default_code_task_intent_json: Option<String>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<AgentImplementationKind>,
    pub implementation_type: AgentImplementationType,
    pub description: Option<String>,
    pub created_by: u64,
    pub created_at: String,
    pub activated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: AgentImplementationKind,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub active: bool,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentProviderBindingRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCompositionSlotKind {
    Memory,
    Knowledge,
    Skill,
    Prompt,
    Drive,
    Document,
    Tool,
    Mcp,
}

impl AgentCompositionSlotKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Skill => "skill",
            Self::Prompt => "prompt",
            Self::Drive => "drive",
            Self::Document => "document",
            Self::Tool => "tool",
            Self::Mcp => "mcp",
        }
    }

    pub fn try_from_str(value: &str) -> Option<Self> {
        match value {
            "memory" => Some(Self::Memory),
            "knowledge" => Some(Self::Knowledge),
            "skill" => Some(Self::Skill),
            "prompt" => Some(Self::Prompt),
            "drive" => Some(Self::Drive),
            "document" => Some(Self::Document),
            "tool" => Some(Self::Tool),
            "mcp" => Some(Self::Mcp),
            _ => None,
        }
    }

    pub fn matches_target_module(self, target_module: AgentCompositionTargetModule) -> bool {
        matches!(
            (self, target_module),
            (Self::Memory, AgentCompositionTargetModule::Memory)
                | (Self::Knowledge, AgentCompositionTargetModule::Knowledgebase)
                | (Self::Skill, AgentCompositionTargetModule::Skills)
                | (Self::Prompt, AgentCompositionTargetModule::Prompts)
                | (Self::Drive, AgentCompositionTargetModule::Drive)
                | (Self::Document, AgentCompositionTargetModule::Documents)
                | (Self::Tool, AgentCompositionTargetModule::Tools)
                | (Self::Mcp, AgentCompositionTargetModule::Mcp)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCompositionTargetModule {
    Memory,
    Knowledgebase,
    Skills,
    Prompts,
    Drive,
    Documents,
    Tools,
    Mcp,
}

impl AgentCompositionTargetModule {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Knowledgebase => "knowledgebase",
            Self::Skills => "skills",
            Self::Prompts => "prompts",
            Self::Drive => "drive",
            Self::Documents => "documents",
            Self::Tools => "tools",
            Self::Mcp => "mcp",
        }
    }

    pub fn try_from_str(value: &str) -> Option<Self> {
        match value {
            "memory" => Some(Self::Memory),
            "knowledgebase" => Some(Self::Knowledgebase),
            "skills" => Some(Self::Skills),
            "prompts" => Some(Self::Prompts),
            "drive" => Some(Self::Drive),
            "documents" => Some(Self::Documents),
            "tools" => Some(Self::Tools),
            "mcp" => Some(Self::Mcp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotRecord {
    pub id: u64,
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
    pub status: AgentBusinessStatus,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentCompositionSlotRecord {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_deleted(&mut self, deleted_at: impl Into<String>) {
        self.deleted_at = Some(deleted_at.into());
        self.status = AgentBusinessStatus::Deleted;
        self.mark_updated(self.deleted_at.clone().unwrap_or_default());
    }
}

impl AgentBusinessRecord {
    pub fn is_deleted(&self) -> bool {
        self.status == AgentBusinessStatus::Deleted || self.deleted_at.is_some()
    }

    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_deleted(&mut self, deleted_at: impl Into<String>) {
        self.status = AgentBusinessStatus::Deleted;
        self.deleted_at = Some(deleted_at.into());
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_restored(&mut self, restored_at: impl Into<String>) {
        self.status = AgentBusinessStatus::Active;
        self.deleted_at = None;
        self.updated_at = restored_at.into();
        self.version = self.version.saturating_add(1);
    }
}

// ============================================================================
// Agent Session Management — aligns with kernel lifecycle SPI (AgentSession)
// ============================================================================

/// Lifecycle status of an agent chat session.
///
/// This mirrors the kernel's `SessionState` but uses business-friendly
/// values suitable for database storage and API exposure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionStatus {
    /// Session is active and accepting items
    Active,
    /// Session is idle (no recent activity, still resumable)
    Idle,
    /// Session has been explicitly closed by the user or system
    Closed,
    /// Session has been archived (read-only, retained for history)
    Archived,
}

impl AgentSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
            Self::Closed => "closed",
            Self::Archived => "archived",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "idle" => Some(Self::Idle),
            "closed" => Some(Self::Closed),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Active => 0,
            Self::Idle => 1,
            Self::Closed => 2,
            Self::Archived => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Idle),
            2 => Some(Self::Closed),
            3 => Some(Self::Archived),
            _ => None,
        }
    }

    /// Whether the session can accept new items.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Idle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionKind {
    Assistant,
    Coding,
    Automation,
    ImDispatch,
}

impl AgentSessionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Assistant => "assistant",
            Self::Coding => "coding",
            Self::Automation => "automation",
            Self::ImDispatch => "im_dispatch",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "assistant" => Some(Self::Assistant),
            "coding" => Some(Self::Coding),
            "automation" => Some(Self::Automation),
            "im_dispatch" => Some(Self::ImDispatch),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Assistant => 0,
            Self::Coding => 1,
            Self::Automation => 2,
            Self::ImDispatch => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Assistant),
            1 => Some(Self::Coding),
            2 => Some(Self::Automation),
            3 => Some(Self::ImDispatch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionEntrySurface {
    Pc,
    H5,
    Flutter,
    MiniProgram,
    Api,
    ImDispatch,
    Automation,
}

impl AgentSessionEntrySurface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pc => "pc",
            Self::H5 => "h5",
            Self::Flutter => "flutter",
            Self::MiniProgram => "mini_program",
            Self::Api => "api",
            Self::ImDispatch => "im_dispatch",
            Self::Automation => "automation",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "pc" => Some(Self::Pc),
            "h5" => Some(Self::H5),
            "flutter" => Some(Self::Flutter),
            "mini_program" => Some(Self::MiniProgram),
            "api" => Some(Self::Api),
            "im_dispatch" => Some(Self::ImDispatch),
            "automation" => Some(Self::Automation),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Pc => 0,
            Self::H5 => 1,
            Self::Flutter => 2,
            Self::MiniProgram => 3,
            Self::Api => 4,
            Self::ImDispatch => 5,
            Self::Automation => 6,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Pc),
            1 => Some(Self::H5),
            2 => Some(Self::Flutter),
            3 => Some(Self::MiniProgram),
            4 => Some(Self::Api),
            5 => Some(Self::ImDispatch),
            6 => Some(Self::Automation),
            _ => None,
        }
    }
}

/// Durable execution context between an owner and one managed agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionRecord {
    pub id: u64,
    pub session_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub project_id: Option<String>,
    pub session_kind: AgentSessionKind,
    pub entry_surface: AgentSessionEntrySurface,
    pub source_module: Option<String>,
    pub source_context_kind: Option<String>,
    pub source_context_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub forked_from_turn_id: Option<String>,
    pub title: Option<String>,
    pub title_source: AgentSessionTitleSource,
    pub status: AgentSessionStatus,
    pub item_count: u64,
    pub last_item_sequence: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub idempotency_key: Option<String>,
    pub payload_hash: Option<String>,
    pub created_by: u64,
    pub updated_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub last_item_at: Option<String>,
    pub closed_at: Option<String>,
    pub archived_at: Option<String>,
    pub archived_by: Option<u64>,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<u64>,
    pub retention_until: Option<String>,
}

/// Identifies the authority that may update a Session display title.
///
/// Provider history reconciliation may update titles only while the provider
/// remains authoritative. A user rename switches the authority permanently to
/// `User`, preventing later provider inventory refreshes from overwriting it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionTitleSource {
    Provider,
    User,
    System,
}

impl AgentSessionTitleSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Provider => "provider",
            Self::User => "user",
            Self::System => "system",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Provider => 0,
            Self::User => 1,
            Self::System => 2,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Provider),
            1 => Some(Self::User),
            2 => Some(Self::System),
            _ => None,
        }
    }
}

impl AgentSessionRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn record_item(
        &mut self,
        input_tokens: u64,
        output_tokens: u64,
        occurred_at: impl Into<String>,
    ) {
        self.item_count = self.item_count.saturating_add(1);
        self.last_item_sequence = self.last_item_sequence.saturating_add(1);
        self.total_input_tokens = self.total_input_tokens.saturating_add(input_tokens);
        self.total_output_tokens = self.total_output_tokens.saturating_add(output_tokens);
        self.last_item_at = Some(occurred_at.into());
        self.updated_at = self
            .last_item_at
            .clone()
            .unwrap_or_else(|| self.updated_at.clone());
        self.version = self.version.saturating_add(1);
    }

    /// Record one persisted input + output turn with one optimistic version bump.
    pub fn record_turn(
        &mut self,
        input_tokens: u64,
        output_tokens: u64,
        occurred_at: impl Into<String>,
    ) {
        self.item_count = self.item_count.saturating_add(2);
        self.last_item_sequence = self.last_item_sequence.saturating_add(2);
        self.total_input_tokens = self.total_input_tokens.saturating_add(input_tokens);
        self.total_output_tokens = self.total_output_tokens.saturating_add(output_tokens);
        self.last_item_at = Some(occurred_at.into());
        self.updated_at = self
            .last_item_at
            .clone()
            .unwrap_or_else(|| self.updated_at.clone());
        self.version = self.version.saturating_add(1);
    }

    pub fn close(&mut self, closed_at: impl Into<String>) {
        let ts = closed_at.into();
        self.status = AgentSessionStatus::Closed;
        self.closed_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    pub fn archive(&mut self, archived_at: impl Into<String>) {
        let ts = archived_at.into();
        self.status = AgentSessionStatus::Archived;
        self.archived_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    pub fn soft_delete(&mut self, deleted_at: impl Into<String>) {
        let ts = deleted_at.into();
        self.deleted_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionRuntimeBindingStatus {
    Active,
    Deactivated,
    Failed,
    Deleted,
}

impl AgentSessionRuntimeBindingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Deactivated => "deactivated",
            Self::Failed => "failed",
            Self::Deleted => "deleted",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Active => 0,
            Self::Deactivated => 1,
            Self::Failed => 2,
            Self::Deleted => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Deactivated),
            2 => Some(Self::Failed),
            3 => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionRuntimeBindingRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub session_id: String,
    pub runtime_binding_id: String,
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
    pub provider_title: Option<String>,
    pub provider_title_source: Option<String>,
    pub provider_preview: Option<String>,
    pub provider_created_at: Option<String>,
    pub provider_updated_at: Option<String>,
    pub provider_recency_at: Option<String>,
    pub provider_pinned: bool,
    pub provider_archived: bool,
    pub provider_visible: bool,
    pub provider_sort_key: Option<String>,
    pub provider_source: Option<String>,
    pub status: AgentSessionRuntimeBindingStatus,
    pub is_current: bool,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub activated_at: Option<String>,
    pub deactivated_at: Option<String>,
}

impl AgentSessionRuntimeBindingRecord {
    pub fn mark_updated(&mut self, occurred_at: impl Into<String>) {
        self.updated_at = occurred_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn activate(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentSessionRuntimeBindingStatus::Active;
        self.is_current = true;
        self.activated_at = Some(occurred_at.clone());
        self.deactivated_at = None;
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    pub fn deactivate(
        &mut self,
        status: AgentSessionRuntimeBindingStatus,
        occurred_at: impl Into<String>,
    ) {
        let occurred_at = occurred_at.into();
        self.status = status;
        self.is_current = false;
        self.deactivated_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionCheckpointStatus {
    Active,
    Restored,
    Invalidated,
    Expired,
    Deleted,
}

impl AgentSessionCheckpointStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Restored => "restored",
            Self::Invalidated => "invalidated",
            Self::Expired => "expired",
            Self::Deleted => "deleted",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Active => 0,
            Self::Restored => 1,
            Self::Invalidated => 2,
            Self::Expired => 3,
            Self::Deleted => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Restored),
            2 => Some(Self::Invalidated),
            3 => Some(Self::Expired),
            4 => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionCheckpointRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub checkpoint_id: String,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub checkpoint_kind: String,
    pub provider_checkpoint_ref: Option<String>,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub resumable: bool,
    pub status: AgentSessionCheckpointStatus,
    pub created_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub restored_at: Option<String>,
    pub invalidated_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentSessionCheckpointRecord {
    pub fn mark_restored(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentSessionCheckpointStatus::Restored;
        self.restored_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    pub fn invalidate(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentSessionCheckpointStatus::Invalidated;
        self.resumable = false;
        self.invalidated_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentResourceType {
    Session,
    Project,
}

impl AgentResourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Project => "project",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Session => 0,
            Self::Project => 1,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Session),
            1 => Some(Self::Project),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentResourceUserStateRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub resource_type: AgentResourceType,
    pub resource_id: String,
    pub pinned_at: Option<String>,
    pub hidden_at: Option<String>,
    pub last_opened_at: Option<String>,
    pub last_read_item_sequence: Option<u64>,
    pub custom_title: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// Agent session-item management. Kernel model messages remain an SPI detail.
// ============================================================================

/// Semantic kind of one ordered Agents transcript or execution item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionItemKind {
    UserInput,
    SystemInstruction,
    AssistantOutput,
    Reasoning,
    ToolCall,
    ToolResult,
    ArtifactReference,
    StatusNotice,
    ErrorNotice,
}

impl AgentSessionItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserInput => "user_input",
            Self::SystemInstruction => "system_instruction",
            Self::AssistantOutput => "assistant_output",
            Self::Reasoning => "reasoning",
            Self::ToolCall => "tool_call",
            Self::ToolResult => "tool_result",
            Self::ArtifactReference => "artifact_reference",
            Self::StatusNotice => "status_notice",
            Self::ErrorNotice => "error_notice",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "user_input" => Some(Self::UserInput),
            "system_instruction" => Some(Self::SystemInstruction),
            "assistant_output" => Some(Self::AssistantOutput),
            "reasoning" => Some(Self::Reasoning),
            "tool_call" => Some(Self::ToolCall),
            "tool_result" => Some(Self::ToolResult),
            "artifact_reference" => Some(Self::ArtifactReference),
            "status_notice" => Some(Self::StatusNotice),
            "error_notice" => Some(Self::ErrorNotice),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::UserInput => 0,
            Self::SystemInstruction => 1,
            Self::AssistantOutput => 2,
            Self::Reasoning => 3,
            Self::ToolCall => 4,
            Self::ToolResult => 5,
            Self::ArtifactReference => 6,
            Self::StatusNotice => 7,
            Self::ErrorNotice => 8,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::UserInput),
            1 => Some(Self::SystemInstruction),
            2 => Some(Self::AssistantOutput),
            3 => Some(Self::Reasoning),
            4 => Some(Self::ToolCall),
            5 => Some(Self::ToolResult),
            6 => Some(Self::ArtifactReference),
            7 => Some(Self::StatusNotice),
            8 => Some(Self::ErrorNotice),
            _ => None,
        }
    }
}

/// Durable lifecycle of one session item. Delivery/read state belongs to IM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionItemStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
    Redacted,
}

impl AgentSessionItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Redacted => "redacted",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            "redacted" => Some(Self::Redacted),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Pending => 0,
            Self::Completed => 1,
            Self::Cancelled => 2,
            Self::Failed => 3,
            Self::Redacted => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::Completed),
            2 => Some(Self::Cancelled),
            3 => Some(Self::Failed),
            4 => Some(Self::Redacted),
            _ => None,
        }
    }
}

/// One immutable ordered transcript or execution item in an Agents session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionItemRecord {
    pub id: u64,
    pub item_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub kind: AgentSessionItemKind,
    pub content: Option<String>,
    pub content_type: String,
    pub status: AgentSessionItemStatus,
    pub sequence: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_arguments_json: Option<String>,
    pub tool_result_json: Option<String>,
    pub provider_payload_json: Option<String>,
    pub parent_item_id: Option<String>,
    pub turn_id: Option<String>,
    pub created_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub redacted_at: Option<String>,
    pub redacted_by: Option<u64>,
    pub retention_until: Option<String>,
}

impl AgentSessionItemRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentItemFeedbackRating {
    Up,
    Down,
}

impl AgentItemFeedbackRating {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Up => 1,
            Self::Down => -1,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Up),
            -1 => Some(Self::Down),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentItemFeedbackRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub item_id: String,
    pub user_id: u64,
    pub rating: AgentItemFeedbackRating,
    pub reason_code: Option<String>,
    pub comment: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentItemResourceRole {
    Attachment,
    Image,
    Audio,
    GeneratedOutput,
    Artifact,
}

impl AgentItemResourceRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::GeneratedOutput => "generated_output",
            Self::Artifact => "artifact",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentItemDriveRefRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub item_id: String,
    pub resource_role: AgentItemResourceRole,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub media_resource_id: Option<String>,
    pub object_blob_id: Option<String>,
    pub resource_hash: Option<String>,
    pub alt_text: Option<String>,
    pub sort_order: u32,
    pub status: i16,
    pub created_by: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub retention_until: Option<String>,
}

// ============================================================================
// Agent media tool per-tenant configuration (admin-managed)
// ============================================================================

/// Per-tenant configuration of one media tool, managed through the backend
/// admin surface and merged at invocation time.
///
/// Defaults mirror the cloudrouter-backed tool family: tools are enabled by
/// default, generated content is not saved to Drive unless requested or
/// configured, and `default_arguments` supplies model/voice/quality defaults
/// that the caller may override per invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentToolConfigurationRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub tool_id: String,
    pub enabled: bool,
    pub save_to_drive_default: bool,
    pub default_arguments_json: String,
    pub version: u64,
    pub created_by: u64,
    pub updated_by: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// A generated-media asset persisted to Drive through a media tool
/// invocation with `saveToDrive`, registered independently of session items
/// so front-end-driven saves appear in the unified asset entry surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentToolAssetRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub tool_id: String,
    pub tool_call_id: String,
    pub media_kind: String,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub drive_uri: String,
    pub source_url: Option<String>,
    pub created_by: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

// ============================================================================
// Agent Live Interaction Management — agent-engine pause/resume lifecycle
// ============================================================================

/// Lifecycle status of a live interaction during a agent-engine turn.
///
/// Agent engines (codex, claude-code, opencode, gemini) may pause execution
/// to request user input. This enum tracks the interaction from creation
/// through resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentInteractionStatus {
    /// Interaction is pending user response.
    Pending,
    /// User has approved or answered the interaction.
    Resolved,
    /// User has rejected the interaction.
    Rejected,
    /// Interaction expired without response.
    Expired,
    /// Interaction was cancelled by the engine.
    Cancelled,
}

impl AgentInteractionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Resolved => "resolved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "resolved" => Some(Self::Resolved),
            "rejected" => Some(Self::Rejected),
            "expired" => Some(Self::Expired),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Pending => 0,
            Self::Resolved => 1,
            Self::Rejected => 2,
            Self::Expired => 3,
            Self::Cancelled => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::Resolved),
            2 => Some(Self::Rejected),
            3 => Some(Self::Expired),
            4 => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// Whether the interaction still awaits a user response.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

/// Kind of live interaction requested by a agent engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentInteractionKind {
    /// Permission approval request (e.g. file write, command execution).
    Approval,
    /// User question with selectable options or free-text answer.
    UserQuestion,
    /// Structured MCP or provider elicitation.
    Elicitation,
    /// Structured provider setup step.
    Setup,
}

impl AgentInteractionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approval => "approval",
            Self::UserQuestion => "user_question",
            Self::Elicitation => "elicitation",
            Self::Setup => "setup",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "approval" => Some(Self::Approval),
            "user_question" => Some(Self::UserQuestion),
            "elicitation" => Some(Self::Elicitation),
            "setup" => Some(Self::Setup),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Approval => 0,
            Self::UserQuestion => 1,
            Self::Elicitation => 2,
            Self::Setup => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Approval),
            1 => Some(Self::UserQuestion),
            2 => Some(Self::Elicitation),
            3 => Some(Self::Setup),
            _ => None,
        }
    }
}

/// A live interaction record representing a agent-engine pause point.
///
/// When a agent engine (codex, claude-code, opencode) pauses execution to
/// request user input, an interaction record is created. The record tracks
/// the prompt, optional selectable options, and the user's resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInteractionRecord {
    pub id: u64,
    pub interaction_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub provider_interaction_id: Option<String>,
    pub kind: AgentInteractionKind,
    pub status: AgentInteractionStatus,
    /// The prompt text shown to the user (e.g. "Allow file write to /src/main.rs?").
    pub prompt: String,
    /// Selectable options for user-question interactions (JSON array of strings).
    pub options_json: String,
    /// Versioned provider-neutral typed request envelope.
    pub request_json: Option<String>,
    /// The user's resolution payload (JSON: approved/answer/reason).
    pub resolution_json: Option<String>,
    pub claim_owner: Option<String>,
    /// SHA-256 hash of the short-lived claim credential; raw tokens are never persisted.
    pub claim_token_hash: Option<String>,
    pub claim_expires_at: Option<String>,
    pub fencing_token: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
    pub retention_until: Option<String>,
}

/// Structured audit payload for agent task operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub task_id: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub status: String,
    pub version: u64,
}

impl TaskAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentTaskRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            task_id: record.task_id.clone(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            status: record.status.as_str().to_string(),
            version: record.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl AgentInteractionRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn claim(
        &mut self,
        claim_owner: impl Into<String>,
        claim_token_hash: impl Into<String>,
        claim_expires_at: impl Into<String>,
        occurred_at: impl Into<String>,
    ) {
        self.claim_owner = Some(claim_owner.into());
        self.claim_token_hash = Some(claim_token_hash.into());
        self.claim_expires_at = Some(claim_expires_at.into());
        self.fencing_token = self.fencing_token.saturating_add(1);
        self.updated_at = occurred_at.into();
        self.version = self.version.saturating_add(1);
    }

    /// Resolve the interaction with the given resolution JSON.
    pub fn resolve(
        &mut self,
        status: AgentInteractionStatus,
        resolution_json: impl Into<String>,
        resolved_at: impl Into<String>,
    ) {
        let ts = resolved_at.into();
        self.status = status;
        self.resolution_json = Some(resolution_json.into());
        self.claim_owner = None;
        self.claim_token_hash = None;
        self.claim_expires_at = None;
        self.resolved_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    /// Whether the interaction still awaits a user response.
    pub fn is_pending(&self) -> bool {
        self.status.is_pending()
    }
}

pub use crate::task_scheduling::{AgentTaskRecord, AgentTaskStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_slot_kind_roundtrips_all_variants() {
        for kind in [
            AgentCompositionSlotKind::Memory,
            AgentCompositionSlotKind::Knowledge,
            AgentCompositionSlotKind::Skill,
            AgentCompositionSlotKind::Prompt,
            AgentCompositionSlotKind::Drive,
            AgentCompositionSlotKind::Document,
            AgentCompositionSlotKind::Tool,
            AgentCompositionSlotKind::Mcp,
        ] {
            let s = kind.as_str();
            assert_eq!(AgentCompositionSlotKind::try_from_str(s), Some(kind));
        }
    }

    #[test]
    fn composition_target_module_roundtrips_all_variants() {
        for module in [
            AgentCompositionTargetModule::Memory,
            AgentCompositionTargetModule::Knowledgebase,
            AgentCompositionTargetModule::Skills,
            AgentCompositionTargetModule::Prompts,
            AgentCompositionTargetModule::Drive,
            AgentCompositionTargetModule::Documents,
            AgentCompositionTargetModule::Tools,
            AgentCompositionTargetModule::Mcp,
        ] {
            let s = module.as_str();
            assert_eq!(AgentCompositionTargetModule::try_from_str(s), Some(module));
        }
    }

    #[test]
    fn composition_slot_kinds_match_only_their_canonical_target_modules() {
        let kinds = [
            AgentCompositionSlotKind::Memory,
            AgentCompositionSlotKind::Knowledge,
            AgentCompositionSlotKind::Skill,
            AgentCompositionSlotKind::Prompt,
            AgentCompositionSlotKind::Drive,
            AgentCompositionSlotKind::Document,
            AgentCompositionSlotKind::Tool,
            AgentCompositionSlotKind::Mcp,
        ];
        let target_modules = [
            AgentCompositionTargetModule::Memory,
            AgentCompositionTargetModule::Knowledgebase,
            AgentCompositionTargetModule::Skills,
            AgentCompositionTargetModule::Prompts,
            AgentCompositionTargetModule::Drive,
            AgentCompositionTargetModule::Documents,
            AgentCompositionTargetModule::Tools,
            AgentCompositionTargetModule::Mcp,
        ];
        let pairs = [
            (
                AgentCompositionSlotKind::Memory,
                AgentCompositionTargetModule::Memory,
            ),
            (
                AgentCompositionSlotKind::Knowledge,
                AgentCompositionTargetModule::Knowledgebase,
            ),
            (
                AgentCompositionSlotKind::Skill,
                AgentCompositionTargetModule::Skills,
            ),
            (
                AgentCompositionSlotKind::Prompt,
                AgentCompositionTargetModule::Prompts,
            ),
            (
                AgentCompositionSlotKind::Drive,
                AgentCompositionTargetModule::Drive,
            ),
            (
                AgentCompositionSlotKind::Document,
                AgentCompositionTargetModule::Documents,
            ),
            (
                AgentCompositionSlotKind::Tool,
                AgentCompositionTargetModule::Tools,
            ),
            (
                AgentCompositionSlotKind::Mcp,
                AgentCompositionTargetModule::Mcp,
            ),
        ];

        for kind in kinds {
            for target_module in target_modules {
                assert_eq!(
                    kind.matches_target_module(target_module),
                    pairs.contains(&(kind, target_module)),
                    "unexpected composition mapping: {}/{}",
                    kind.as_str(),
                    target_module.as_str()
                );
            }
        }
    }

    #[test]
    fn composition_slot_kind_rejects_unknown_value() {
        assert!(AgentCompositionSlotKind::try_from_str("unknown").is_none());
    }
}
