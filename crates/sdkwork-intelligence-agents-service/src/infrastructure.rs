use crate::agent_turn::{AgentTurnRecord, AgentTurnStatus};
use crate::agent_turn_input_queue::{
    AgentTurnInputQueueEntry, AgentTurnInputQueueStatus, TurnInputQueueClaimOutcome,
    TurnInputQueueClaimRequest, TurnInputQueueFailureRequest, TurnInputQueueListQuery,
    TurnInputQueueReorderEntry, MAX_TURN_INPUT_QUEUE_CONTENT_BYTES_PER_SESSION,
    MAX_TURN_INPUT_QUEUE_ENTRIES_PER_SESSION,
};
use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentInteractionKind, AgentInteractionRecord, AgentItemDriveRefRecord, AgentItemFeedbackRecord,
    AgentProviderBindingRecord, AgentResourceType, AgentResourceUserStateRecord,
    AgentRuntimeExecutionRecord, AgentRuntimeExecutionStatus, AgentSessionCheckpointRecord,
    AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionRecord,
    AgentSessionRuntimeBindingRecord, AgentSessionRuntimeBindingStatus, AgentTaskRecord,
    AgentToolAssetRecord, AgentToolConfigurationRecord, AgentVersionRecord,
};
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
use crate::in_memory_pagination::{count_iterator, paginate_items, paginate_iterator};
use crate::ports::{
    validate_completed_turn_items, AgentAuditSink, AgentListQuery, AgentRepository,
    AgentVersionListQuery, AuditEventListQuery, CompositionSlotListQuery, InteractionListQuery,
    ItemFeedbackListQuery, McpMarketplaceListQuery, ProjectCompositionSlotListQuery,
    ProjectListQuery, ProviderBindingListQuery, ResourceUserStateListQuery,
    RuntimeExecutionListQuery, SessionActivitySummaryListQuery, SessionCheckpointListQuery,
    SessionItemListQuery, SessionItemListSort, SessionListQuery, SessionRuntimeBindingListQuery,
    TaskListQuery, TurnListQuery, TurnRequestWriteOutcome, WebhookSubscriptionListQuery,
    WorkspaceListQuery,
};
use crate::project::{
    project_names_equal, AgentProjectCompositionSlotRecord, AgentProjectRecord, AgentProjectStatus,
};
use crate::session_activity::{
    encode_session_activity_cursor, SessionActivityCursor, SessionActivitySource,
    SessionActivitySummaryParts, SessionActivitySummaryRecord,
};
use crate::task_scheduler::{
    plan_task_materialization, task_run_payload_hash, ClaimTaskRunsRequest, FailTaskRunRequest,
    MaterializeDueTasksRequest, ReconcileTaskRunRequest, TaskRunAttemptListQuery, TaskRunClaim,
    TaskRunFailureDisposition, TaskRunLease, TaskRunListQuery, TaskSchedulerRepository,
    TaskTransitionResult,
};
use crate::task_scheduling::{
    AgentTaskRunAttemptRecord, AgentTaskRunAttemptStatus, AgentTaskRunRecord, AgentTaskRunStatus,
    AgentTaskStatus, AgentTaskTriggerKind,
};
use crate::validation::{
    parse_rfc3339_datetime, validate_standard_id, ID_PREFIX_ATTEMPT, ID_PREFIX_RUN, ID_PREFIX_TURN,
};
use crate::webhook::{AgentWebhookDeliveryRecord, AgentWebhookRecord};
use crate::workspace::{AgentWorkspaceRecord, AgentWorkspaceStatus};
use sdkwork_agent_kernel::{
    KernelError, KernelEvent, KernelResult, PolicyDecision, PolicyProvider, PolicyRequest,
    ProviderHealth, ProviderManifest,
};
use sdkwork_utils_rust::{format_datetime, is_blank, parse_datetime, sha256_hash, trim};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{LazyLock, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

fn turn_input_queue_payload_bytes(entry: &AgentTurnInputQueueEntry) -> usize {
    entry
        .content
        .len()
        .saturating_add(entry.display_text.len())
        .saturating_add(
            entry
                .attachment_names
                .iter()
                .map(String::len)
                .sum::<usize>(),
        )
}

// ---------------------------------------------------------------------------
// Production environment safety checks
// ---------------------------------------------------------------------------

/// Environment variable name for development authentication bypass.
/// When set to "true", AllowAllPolicyProvider permits all requests without
/// IAM validation. This MUST NOT be enabled in production environments.
pub const ENV_DEV_AUTH_BYPASS: &str = "SDKWORK_AGENTS_DEV_AUTH_BYPASS";

/// Environment variable name indicating production deployment.
/// Prefer `sdkwork_agents_contract::agents_deployment_environment_name()` for gating.
pub const ENV_DEPLOYMENT_ENV: &str = "SDKWORK_DEPLOYMENT_ENV";

/// Validates that development authentication bypass is not enabled in production.
///
/// Delegates to `sdkwork-agents-contract::ensure_dev_auth_bypass_allowed` so
/// callers can fail bootstrap explicitly instead of crashing the process.
pub fn validate_production_security_config() -> Result<(), String> {
    sdkwork_agents_contract::ensure_dev_auth_bypass_allowed()?;

    if agents_dev_auth_bypass_enabled_from_env() {
        tracing::warn!(
            env_var = ENV_DEV_AUTH_BYPASS,
            deployment = %sdkwork_agents_contract::agents_deployment_environment_name(),
            "Development authentication bypass is enabled. This MUST NOT be used in production."
        );
    }
    Ok(())
}

fn agents_dev_auth_bypass_enabled_from_env() -> bool {
    std::env::var(ENV_DEV_AUTH_BYPASS)
        .ok()
        .and_then(|value| sdkwork_utils_rust::parse_bool(value.trim()))
        .unwrap_or(false)
}

/// Returns true if the current deployment is a production environment.
pub fn is_production_environment() -> bool {
    sdkwork_agents_contract::agents_is_production_like_environment()
}

// ---------------------------------------------------------------------------
// Metrics types for observability (O-01)
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

/// Prometheus snapshot for agents managed-store HTTP and business gauges.
#[derive(Debug, Clone, Default)]
pub struct AgentServiceMetrics {
    pub http_requests_total: u64,
    pub http_errors_total: u64,
    pub http_requests_per_second: f64,
    pub service_worker_rejections_total: u64,
    pub provider_worker_rejections_total: u64,
    /// Total number of agents across all tenants
    pub total_agents: u64,
    /// Number of active (non-deleted) agents
    pub active_agents: u64,
    /// Number of soft-deleted agents
    pub deleted_agents: u64,
    /// Total number of provider bindings
    pub total_provider_bindings: u64,
    /// Number of active provider bindings
    pub active_provider_bindings: u64,
    /// Total number of composition slots
    pub total_composition_slots: u64,
    /// Number of audit events recorded
    pub audit_events_count: u64,
    /// Request count by operation (operation -> count)
    pub request_counts: std::collections::HashMap<String, u64>,
    /// Error count by operation (operation -> count)
    pub error_counts: std::collections::HashMap<String, u64>,
}

#[derive(Debug)]
struct ScrapeState {
    instant: Instant,
    request_total: u64,
}

/// Process-wide agents metrics registry for Prometheus scraping.
pub struct AgentMetricsRegistry {
    http_requests_total: AtomicU64,
    http_errors_total: AtomicU64,
    service_worker_rejections_total: AtomicU64,
    provider_worker_rejections_total: AtomicU64,
    scrape_state: Mutex<ScrapeState>,
}

impl AgentMetricsRegistry {
    pub fn global() -> &'static Self {
        static REGISTRY: LazyLock<AgentMetricsRegistry> = LazyLock::new(|| AgentMetricsRegistry {
            http_requests_total: AtomicU64::new(0),
            http_errors_total: AtomicU64::new(0),
            service_worker_rejections_total: AtomicU64::new(0),
            provider_worker_rejections_total: AtomicU64::new(0),
            scrape_state: Mutex::new(ScrapeState {
                instant: Instant::now(),
                request_total: 0,
            }),
        });
        &REGISTRY
    }

    pub fn record_http_request(&self, status: u16) {
        self.http_requests_total
            .fetch_add(1, AtomicOrdering::Relaxed);
        if status >= 400 {
            self.http_errors_total.fetch_add(1, AtomicOrdering::Relaxed);
        }
    }

    pub fn record_service_worker_rejection(&self) {
        self.service_worker_rejections_total
            .fetch_add(1, AtomicOrdering::Relaxed);
    }

    pub fn record_provider_worker_rejection(&self) {
        self.provider_worker_rejections_total
            .fetch_add(1, AtomicOrdering::Relaxed);
    }

    pub fn snapshot(&self) -> AgentServiceMetrics {
        let http_requests_total = self.http_requests_total.load(AtomicOrdering::Relaxed);
        let http_errors_total = self.http_errors_total.load(AtomicOrdering::Relaxed);
        let service_worker_rejections_total = self
            .service_worker_rejections_total
            .load(AtomicOrdering::Relaxed);
        let provider_worker_rejections_total = self
            .provider_worker_rejections_total
            .load(AtomicOrdering::Relaxed);
        let http_requests_per_second = {
            let mut state = self.scrape_state.recovering_lock();
            let now = Instant::now();
            let elapsed = now.duration_since(state.instant).as_secs_f64();
            let delta = http_requests_total.saturating_sub(state.request_total);
            state.instant = now;
            state.request_total = http_requests_total;
            if elapsed > 0.0 {
                delta as f64 / elapsed
            } else {
                0.0
            }
        };

        AgentServiceMetrics {
            http_requests_total,
            http_errors_total,
            http_requests_per_second,
            service_worker_rejections_total,
            provider_worker_rejections_total,
            ..AgentServiceMetrics::default()
        }
    }
}

impl AgentServiceMetrics {
    /// Export metrics in Prometheus text exposition format
    pub fn to_prometheus_text(&self) -> String {
        let mut output = String::with_capacity(1024);

        output.push_str("# HELP sdkwork_agents_requests_total Total managed-store HTTP requests\n");
        output.push_str("# TYPE sdkwork_agents_requests_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_requests_total {}\n",
            self.http_requests_total
        ));

        output.push_str(
            "# HELP sdkwork_agents_errors_total Total managed-store HTTP error responses\n",
        );
        output.push_str("# TYPE sdkwork_agents_errors_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_errors_total {}\n",
            self.http_errors_total
        ));

        output.push_str("# HELP sdkwork_agents_requests_per_second Managed-store HTTP requests per second (scrape-window rate)\n");
        output.push_str("# TYPE sdkwork_agents_requests_per_second gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_requests_per_second {}\n",
            self.http_requests_per_second
        ));

        output.push_str("# HELP sdkwork_agents_service_worker_rejections_total Requests rejected before entering the blocking service pool\n");
        output.push_str("# TYPE sdkwork_agents_service_worker_rejections_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_service_worker_rejections_total {}\n",
            self.service_worker_rejections_total
        ));

        output.push_str("# HELP sdkwork_agents_provider_worker_rejections_total Provider executions rejected when bounded capacity is exhausted\n");
        output.push_str("# TYPE sdkwork_agents_provider_worker_rejections_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_provider_worker_rejections_total {}\n",
            self.provider_worker_rejections_total
        ));

        // Help and type declarations
        output.push_str("# HELP sdkwork_agents_total Total number of agents\n");
        output.push_str("# TYPE sdkwork_agents_total gauge\n");
        output.push_str(&format!("sdkwork_agents_total {}\n", self.total_agents));

        output.push_str("# HELP sdkwork_agents_active Number of active (non-deleted) agents\n");
        output.push_str("# TYPE sdkwork_agents_active gauge\n");
        output.push_str(&format!("sdkwork_agents_active {}\n", self.active_agents));

        output.push_str("# HELP sdkwork_agents_deleted Number of soft-deleted agents\n");
        output.push_str("# TYPE sdkwork_agents_deleted gauge\n");
        output.push_str(&format!("sdkwork_agents_deleted {}\n", self.deleted_agents));

        output.push_str(
            "# HELP sdkwork_agents_provider_bindings_total Total number of provider bindings\n",
        );
        output.push_str("# TYPE sdkwork_agents_provider_bindings_total gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_provider_bindings_total {}\n",
            self.total_provider_bindings
        ));

        output.push_str(
            "# HELP sdkwork_agents_provider_bindings_active Number of active provider bindings\n",
        );
        output.push_str("# TYPE sdkwork_agents_provider_bindings_active gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_provider_bindings_active {}\n",
            self.active_provider_bindings
        ));

        output.push_str(
            "# HELP sdkwork_agents_composition_slots_total Total number of composition slots\n",
        );
        output.push_str("# TYPE sdkwork_agents_composition_slots_total gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_composition_slots_total {}\n",
            self.total_composition_slots
        ));

        output.push_str("# HELP sdkwork_agents_audit_events_total Total number of audit events\n");
        output.push_str("# TYPE sdkwork_agents_audit_events_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_audit_events_total {}\n",
            self.audit_events_count
        ));

        // Request counts by operation
        output.push_str(
            "# HELP sdkwork_agents_requests_by_operation_total Total requests by operation\n",
        );
        output.push_str("# TYPE sdkwork_agents_requests_by_operation_total counter\n");
        for (operation, count) in &self.request_counts {
            output.push_str(&format!(
                "sdkwork_agents_requests_by_operation_total{{operation=\"{}\"}} {}\n",
                operation, count
            ));
        }

        // Error counts by operation
        output.push_str(
            "# HELP sdkwork_agents_errors_by_operation_total Total errors by operation\n",
        );
        output.push_str("# TYPE sdkwork_agents_errors_by_operation_total counter\n");
        for (operation, count) in &self.error_counts {
            output.push_str(&format!(
                "sdkwork_agents_errors_by_operation_total{{operation=\"{}\"}} {}\n",
                operation, count
            ));
        }

        output
    }
}

/// In-memory agent repository with interior mutability via `RwLock`.
///
/// All trait methods use `&self`; the `RwLock` handles concurrent access.
/// This makes the repository compatible with the stateless `AgentsService`
/// and eliminates the global `Mutex<AgentsService>` bottleneck.
type AgentPrimaryKey = (u64, String);
type AgentIndexKey = (u64, Reverse<String>, Reverse<u64>, String);
type ProjectPrimaryKey = (u64, u64, String);
type ProjectIndexKey = (u64, u64, Reverse<String>, Reverse<u64>, String);
type WorkspacePrimaryKey = (u64, u64, String);
type WorkspaceIndexKey = (
    u64,
    u64,
    u64,
    Reverse<bool>,
    Reverse<String>,
    Reverse<u64>,
    String,
);
type ProjectCompositionSlotPrimaryKey = (u64, u64, String, String);
type ProviderBindingPrimaryKey = (u64, String, String);
type ProviderBindingIndexKey = (u64, String, Reverse<bool>, Reverse<String>, String);
type CompositionSlotPrimaryKey = (u64, String, String);
type CompositionSlotIndexKey = (u64, String, i32, String);
type SessionPrimaryKey = (u64, u64, String);
type SessionIdempotencyKey = (u64, u64, u64, String);
type SessionRuntimeBindingPrimaryKey = (u64, u64, String, String);
type SessionCheckpointPrimaryKey = (u64, u64, String, String);
type ResourceUserStatePrimaryKey = (u64, u64, u64, i16, String);
type ToolConfigurationPrimaryKey = (u64, u64, String);
type ToolAssetPrimaryKey = (u64, u64, String, String);
type SessionIndexKey = (u64, u64, Reverse<String>, Reverse<u64>);
type SessionActivityIndexKey = (u64, u64, u64, String, u64);
type SessionItemPrimaryKey = (u64, u64, String, String);
type ItemFeedbackPrimaryKey = (u64, u64, String, u64);
type ItemDriveRefPrimaryKey = (u64, u64, String, String, String);
type SessionItemIndexKey = (u64, u64, String, u64, u64);
type TurnPrimaryKey = (u64, u64, String);
type TurnIdempotencyKey = (u64, u64, u64, String);
type TurnIndexKey = (u64, u64, String, Reverse<String>, Reverse<u64>);
type TurnInputQueuePrimaryKey = (u64, u64, String, String);
type InteractionPrimaryKey = (u64, u64, String, String);
type InteractionIndexKey = (u64, u64, String, Reverse<String>, String);
type PendingInteractionIndexKey = (u64, u64, String, i16, Reverse<String>, String);
type TaskPrimaryKey = (u64, u64, String);
type TaskIndexKey = (u64, u64, Reverse<String>, Reverse<u64>);
type TaskRunPrimaryKey = (u64, u64, String);
type TaskRunAttemptPrimaryKey = (u64, u64, String);
type RuntimeExecutionPrimaryKey = (u64, String, String);

trait RecoveringRwLock<T> {
    fn recovering_read(&self) -> RwLockReadGuard<'_, T>;

    fn recovering_write(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RecoveringRwLock<T> for RwLock<T> {
    fn recovering_read(&self) -> RwLockReadGuard<'_, T> {
        match self.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    "in-memory agents repository RwLock was poisoned; recovering guard"
                );
                poisoned.into_inner()
            }
        }
    }

    fn recovering_write(&self) -> RwLockWriteGuard<'_, T> {
        match self.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    "in-memory agents repository RwLock was poisoned; recovering mutable guard"
                );
                poisoned.into_inner()
            }
        }
    }
}

trait RecoveringMutex<T> {
    fn recovering_lock(&self) -> MutexGuard<'_, T>;
}

impl<T> RecoveringMutex<T> for Mutex<T> {
    fn recovering_lock(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!("in-memory agents mutex was poisoned; recovering guard");
                poisoned.into_inner()
            }
        }
    }
}

#[derive(Debug)]
pub struct InMemoryAgentRepository {
    id_generator: AgentBusinessIdGenerator,
    agents: RwLock<HashMap<AgentPrimaryKey, AgentBusinessRecord>>,
    agent_list_index: RwLock<BTreeMap<AgentIndexKey, AgentPrimaryKey>>,
    projects: RwLock<HashMap<ProjectPrimaryKey, AgentProjectRecord>>,
    project_index: RwLock<BTreeMap<ProjectIndexKey, ProjectPrimaryKey>>,
    workspaces: RwLock<HashMap<WorkspacePrimaryKey, AgentWorkspaceRecord>>,
    workspace_index: RwLock<BTreeMap<WorkspaceIndexKey, WorkspacePrimaryKey>>,
    project_composition_slots:
        RwLock<HashMap<ProjectCompositionSlotPrimaryKey, AgentProjectCompositionSlotRecord>>,
    provider_bindings: RwLock<HashMap<ProviderBindingPrimaryKey, AgentProviderBindingRecord>>,
    provider_binding_index: RwLock<BTreeMap<ProviderBindingIndexKey, ProviderBindingPrimaryKey>>,
    composition_slots: RwLock<HashMap<CompositionSlotPrimaryKey, AgentCompositionSlotRecord>>,
    composition_slot_index: RwLock<BTreeMap<CompositionSlotIndexKey, CompositionSlotPrimaryKey>>,
    sessions: RwLock<HashMap<SessionPrimaryKey, AgentSessionRecord>>,
    session_index: RwLock<BTreeMap<SessionIndexKey, SessionPrimaryKey>>,
    session_activity_index:
        RwLock<BTreeMap<SessionActivityIndexKey, (SessionPrimaryKey, SessionActivitySource)>>,
    session_activity_keys: RwLock<HashMap<SessionPrimaryKey, SessionActivityIndexKey>>,
    session_idempotency: RwLock<HashMap<SessionIdempotencyKey, SessionPrimaryKey>>,
    session_runtime_bindings:
        RwLock<HashMap<SessionRuntimeBindingPrimaryKey, AgentSessionRuntimeBindingRecord>>,
    current_session_runtime_bindings:
        RwLock<HashMap<SessionPrimaryKey, SessionRuntimeBindingPrimaryKey>>,
    session_checkpoints: RwLock<HashMap<SessionCheckpointPrimaryKey, AgentSessionCheckpointRecord>>,
    resource_user_states:
        RwLock<HashMap<ResourceUserStatePrimaryKey, AgentResourceUserStateRecord>>,
    tool_configurations: RwLock<HashMap<ToolConfigurationPrimaryKey, AgentToolConfigurationRecord>>,
    tool_assets: RwLock<HashMap<ToolAssetPrimaryKey, AgentToolAssetRecord>>,
    items: RwLock<HashMap<SessionItemPrimaryKey, AgentSessionItemRecord>>,
    item_feedback: RwLock<HashMap<ItemFeedbackPrimaryKey, AgentItemFeedbackRecord>>,
    item_drive_refs: RwLock<HashMap<ItemDriveRefPrimaryKey, AgentItemDriveRefRecord>>,
    session_item_index: RwLock<BTreeMap<SessionItemIndexKey, SessionItemPrimaryKey>>,
    turns: RwLock<HashMap<TurnPrimaryKey, AgentTurnRecord>>,
    turn_idempotency: RwLock<HashMap<TurnIdempotencyKey, TurnPrimaryKey>>,
    turn_index: RwLock<BTreeMap<TurnIndexKey, TurnPrimaryKey>>,
    turn_input_queue: Mutex<HashMap<TurnInputQueuePrimaryKey, AgentTurnInputQueueEntry>>,
    interactions: RwLock<HashMap<InteractionPrimaryKey, AgentInteractionRecord>>,
    interaction_index: RwLock<BTreeMap<InteractionIndexKey, InteractionPrimaryKey>>,
    pending_interaction_index: RwLock<BTreeMap<PendingInteractionIndexKey, InteractionPrimaryKey>>,
    tasks: RwLock<HashMap<TaskPrimaryKey, AgentTaskRecord>>,
    task_index: RwLock<BTreeMap<TaskIndexKey, TaskPrimaryKey>>,
    task_runs: RwLock<HashMap<TaskRunPrimaryKey, AgentTaskRunRecord>>,
    task_run_attempts: RwLock<HashMap<TaskRunAttemptPrimaryKey, AgentTaskRunAttemptRecord>>,
    turn_streaming_checkpoints: RwLock<HashMap<TurnStreamingCheckpointKey, String>>,
    runtime_executions: RwLock<HashMap<RuntimeExecutionPrimaryKey, AgentRuntimeExecutionRecord>>,
    agent_versions: RwLock<HashMap<AgentVersionPrimaryKey, AgentVersionRecord>>,
    webhook_subscriptions: RwLock<HashMap<WebhookPrimaryKey, AgentWebhookRecord>>,
    webhook_deliveries: RwLock<HashMap<WebhookDeliveryPrimaryKey, AgentWebhookDeliveryRecord>>,
}

/// Version history storage key: (tenant, organization, agent_id, version_id).
type AgentVersionPrimaryKey = (u64, u64, String, String);

/// Webhook subscription storage key: (tenant, organization, webhook_id).
type WebhookPrimaryKey = (u64, u64, String);

/// Webhook delivery storage key: (tenant, organization, webhook_id, delivery_id).
type WebhookDeliveryPrimaryKey = (u64, u64, String, String);

/// Streaming-checkpoint storage key: (tenant, organization, turn_id).
type TurnStreamingCheckpointKey = (u64, u64, String);

impl InMemoryAgentRepository {
    pub fn try_new() -> KernelResult<Self> {
        Ok(Self::with_id_generator(
            AgentBusinessIdGenerator::new_default()?,
        ))
    }

    pub fn new() -> Self {
        Self::try_new().expect("default agents in-memory repository constructor should succeed")
    }

    fn with_id_generator(id_generator: AgentBusinessIdGenerator) -> Self {
        Self {
            id_generator,
            agents: RwLock::new(HashMap::new()),
            agent_list_index: RwLock::new(BTreeMap::new()),
            projects: RwLock::new(HashMap::new()),
            project_index: RwLock::new(BTreeMap::new()),
            workspaces: RwLock::new(HashMap::new()),
            workspace_index: RwLock::new(BTreeMap::new()),
            project_composition_slots: RwLock::new(HashMap::new()),
            provider_bindings: RwLock::new(HashMap::new()),
            provider_binding_index: RwLock::new(BTreeMap::new()),
            composition_slots: RwLock::new(HashMap::new()),
            composition_slot_index: RwLock::new(BTreeMap::new()),
            sessions: RwLock::new(HashMap::new()),
            session_index: RwLock::new(BTreeMap::new()),
            session_activity_index: RwLock::new(BTreeMap::new()),
            session_activity_keys: RwLock::new(HashMap::new()),
            session_idempotency: RwLock::new(HashMap::new()),
            session_runtime_bindings: RwLock::new(HashMap::new()),
            current_session_runtime_bindings: RwLock::new(HashMap::new()),
            session_checkpoints: RwLock::new(HashMap::new()),
            resource_user_states: RwLock::new(HashMap::new()),
            tool_configurations: RwLock::new(HashMap::new()),
            tool_assets: RwLock::new(HashMap::new()),
            items: RwLock::new(HashMap::new()),
            item_feedback: RwLock::new(HashMap::new()),
            item_drive_refs: RwLock::new(HashMap::new()),
            session_item_index: RwLock::new(BTreeMap::new()),
            turns: RwLock::new(HashMap::new()),
            turn_idempotency: RwLock::new(HashMap::new()),
            turn_index: RwLock::new(BTreeMap::new()),
            turn_input_queue: Mutex::new(HashMap::new()),
            interactions: RwLock::new(HashMap::new()),
            interaction_index: RwLock::new(BTreeMap::new()),
            pending_interaction_index: RwLock::new(BTreeMap::new()),
            tasks: RwLock::new(HashMap::new()),
            task_index: RwLock::new(BTreeMap::new()),
            task_runs: RwLock::new(HashMap::new()),
            task_run_attempts: RwLock::new(HashMap::new()),
            turn_streaming_checkpoints: RwLock::new(HashMap::new()),
            runtime_executions: RwLock::new(HashMap::new()),
            agent_versions: RwLock::new(HashMap::new()),
            webhook_subscriptions: RwLock::new(HashMap::new()),
            webhook_deliveries: RwLock::new(HashMap::new()),
        }
    }

    fn workspace_project_ids(&self, query: &SessionListQuery) -> Option<HashSet<String>> {
        let workspace_id = query.workspace_id.as_deref()?;
        let projects = self.projects.recovering_read();
        Some(
            projects
                .values()
                .filter(|project| {
                    project.tenant_id == query.tenant_id
                        && query.organization_id.is_none_or(|organization_id| {
                            project.organization_id == organization_id
                        })
                        && project.workspace_id == workspace_id
                        && project.deleted_at.is_none()
                })
                .map(|project| project.project_id.clone())
                .collect(),
        )
    }

    fn advance_session_activity(
        &self,
        session: &AgentSessionRecord,
        occurred_at: &str,
        source: SessionActivitySource,
    ) {
        let normalized_occurred_at = parse_rfc3339_datetime(occurred_at, "activity occurredAt")
            .ok()
            .and_then(|timestamp| {
                timestamp
                    .to_offset(time::UtcOffset::UTC)
                    .format(&time::format_description::well_known::Rfc3339)
                    .ok()
            })
            .unwrap_or_else(|| occurred_at.to_string());
        let occurred_at = normalized_occurred_at.as_str();
        let primary_key = session_primary_key(session);
        let mut keys = self.session_activity_keys.recovering_write();
        let mut index = self.session_activity_index.recovering_write();
        let (next_activity_at, next_source) = keys
            .get(&primary_key)
            .and_then(|key| {
                index.get(key).and_then(|(_, current_source)| {
                    (key.3.as_str() > occurred_at
                        || (key.3.as_str() == occurred_at
                            && current_source.precedence() >= source.precedence()))
                    .then_some((key.3.clone(), *current_source))
                })
            })
            .unwrap_or_else(|| (occurred_at.to_string(), source));
        if let Some(previous_key) = keys.remove(&primary_key) {
            index.remove(&previous_key);
        }
        let next_key = (
            session.tenant_id,
            session.organization_id,
            session.owner_user_id,
            next_activity_at,
            session.id,
        );
        index.insert(next_key.clone(), (primary_key.clone(), next_source));
        keys.insert(primary_key, next_key);
    }

    fn latest_turn_for_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> Option<AgentTurnRecord> {
        self.turns
            .recovering_read()
            .values()
            .filter(|turn| {
                turn.tenant_id == tenant_id
                    && turn.organization_id == organization_id
                    && turn.session_id == session_id
            })
            .max_by_key(|turn| turn.id)
            .cloned()
    }

    fn pending_interaction_for_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> Option<AgentInteractionRecord> {
        let interactions = self.interactions.recovering_read();
        let index = self.pending_interaction_index.recovering_read();
        for kind in [
            AgentInteractionKind::Approval.as_db_code(),
            AgentInteractionKind::UserQuestion.as_db_code(),
        ] {
            let lower = (
                tenant_id,
                organization_id,
                session_id.to_string(),
                kind,
                Reverse("~".to_string()),
                String::new(),
            );
            let upper = (
                tenant_id,
                organization_id,
                session_id.to_string(),
                kind,
                Reverse(String::new()),
                "~".to_string(),
            );
            if let Some(interaction) = index
                .range(lower..=upper)
                .find_map(|(_, primary_key)| interactions.get(primary_key))
            {
                return Some(interaction.clone());
            }
        }
        None
    }

    fn latest_interaction_component_for_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> Option<(String, u64)> {
        self.interactions
            .recovering_read()
            .values()
            .filter(|interaction| {
                interaction.tenant_id == tenant_id
                    && interaction.organization_id == organization_id
                    && interaction.session_id == session_id
            })
            .max_by(|left, right| {
                left.updated_at
                    .cmp(&right.updated_at)
                    .then_with(|| left.id.cmp(&right.id))
            })
            .map(|interaction| (interaction.interaction_id.clone(), interaction.version))
    }

    fn current_runtime_binding_for_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> Option<AgentSessionRuntimeBindingRecord> {
        let session_key = (tenant_id, organization_id, session_id.to_string());
        let binding_key = self
            .current_session_runtime_bindings
            .recovering_read()
            .get(&session_key)
            .cloned()?;
        self.session_runtime_bindings
            .recovering_read()
            .get(&binding_key)
            .filter(|binding| {
                binding.is_current && binding.status == AgentSessionRuntimeBindingStatus::Active
            })
            .cloned()
    }

    fn latest_runtime_binding_for_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> Option<AgentSessionRuntimeBindingRecord> {
        self.session_runtime_bindings
            .recovering_read()
            .values()
            .filter(|binding| {
                binding.tenant_id == tenant_id
                    && binding.organization_id == organization_id
                    && binding.session_id == session_id
            })
            .max_by(|left, right| {
                left.updated_at
                    .cmp(&right.updated_at)
                    .then_with(|| left.id.cmp(&right.id))
            })
            .cloned()
    }

    fn session_user_state_for_owner(
        &self,
        session: &AgentSessionRecord,
    ) -> Option<AgentResourceUserStateRecord> {
        self.resource_user_states
            .recovering_read()
            .get(&(
                session.tenant_id,
                session.organization_id,
                session.owner_user_id,
                AgentResourceType::Session.as_db_code(),
                session.session_id.clone(),
            ))
            .cloned()
    }

    pub fn records(&self) -> Vec<AgentBusinessRecord> {
        self.agents.recovering_read().values().cloned().collect()
    }
}

fn agent_primary_key(record: &AgentBusinessRecord) -> AgentPrimaryKey {
    (record.tenant_id, record.agent_id.clone())
}

fn agent_index_key(record: &AgentBusinessRecord) -> AgentIndexKey {
    (
        record.tenant_id,
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
        record.agent_id.clone(),
    )
}

fn project_primary_key(record: &AgentProjectRecord) -> ProjectPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.project_id.clone(),
    )
}

fn project_index_key(record: &AgentProjectRecord) -> ProjectIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
        record.project_id.clone(),
    )
}

fn workspace_primary_key(record: &AgentWorkspaceRecord) -> WorkspacePrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.workspace_id.clone(),
    )
}

fn workspace_index_key(record: &AgentWorkspaceRecord) -> WorkspaceIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        record.owner_user_id,
        Reverse(record.is_default),
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
        record.workspace_id.clone(),
    )
}

fn provider_binding_primary_key(record: &AgentProviderBindingRecord) -> ProviderBindingPrimaryKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        record.binding_id.clone(),
    )
}

fn provider_binding_index_key(record: &AgentProviderBindingRecord) -> ProviderBindingIndexKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        Reverse(record.active),
        Reverse(record.updated_at.clone()),
        record.binding_id.clone(),
    )
}

fn composition_slot_primary_key(record: &AgentCompositionSlotRecord) -> CompositionSlotPrimaryKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        record.slot_id.clone(),
    )
}

fn composition_slot_index_key(record: &AgentCompositionSlotRecord) -> CompositionSlotIndexKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        record.priority,
        record.slot_id.clone(),
    )
}

fn session_primary_key(record: &AgentSessionRecord) -> SessionPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
    )
}

fn session_index_key(record: &AgentSessionRecord) -> SessionIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
    )
}

fn resource_user_state_primary_key(
    record: &AgentResourceUserStateRecord,
) -> ResourceUserStatePrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.user_id,
        record.resource_type.as_db_code(),
        record.resource_id.clone(),
    )
}

fn resource_user_state_matches_agent(
    record: &AgentResourceUserStateRecord,
    query: &ResourceUserStateListQuery,
    sessions: &HashMap<SessionPrimaryKey, AgentSessionRecord>,
) -> bool {
    let Some(agent_id) = query.agent_id.as_deref() else {
        return true;
    };
    if record.resource_type != AgentResourceType::Session {
        return false;
    }
    sessions
        .get(&(
            record.tenant_id,
            record.organization_id,
            record.resource_id.clone(),
        ))
        .is_some_and(|session| {
            session.organization_id == record.organization_id
                && session.owner_user_id == record.user_id
                && session.agent_id == agent_id
                && session.deleted_at.is_none()
        })
}

fn session_item_primary_key(record: &AgentSessionItemRecord) -> SessionItemPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        record.item_id.clone(),
    )
}

fn item_feedback_primary_key(record: &AgentItemFeedbackRecord) -> ItemFeedbackPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.item_id.clone(),
        record.user_id,
    )
}

fn session_item_index_key(record: &AgentSessionItemRecord) -> SessionItemIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        record.sequence,
        record.id,
    )
}

fn interaction_primary_key(record: &AgentInteractionRecord) -> InteractionPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        record.interaction_id.clone(),
    )
}

fn interaction_index_key(record: &AgentInteractionRecord) -> InteractionIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        Reverse(record.created_at.clone()),
        record.interaction_id.clone(),
    )
}

fn pending_interaction_index_key(record: &AgentInteractionRecord) -> PendingInteractionIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        record.kind.as_db_code(),
        Reverse(record.updated_at.clone()),
        record.interaction_id.clone(),
    )
}

fn runtime_execution_primary_key(
    record: &AgentRuntimeExecutionRecord,
) -> RuntimeExecutionPrimaryKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        record.execution_id.clone(),
    )
}

/// Shared filter predicate for the usage feeds: conjunctive tenant scope,
/// optional agent/session/model filters and an inclusive `from` / exclusive
/// `to` window over `created_at`.
fn usage_turn_matches(turn: &AgentTurnRecord, query: &crate::usage::UsageSummaryQuery) -> bool {
    if turn.tenant_id != query.tenant_id || turn.organization_id != query.organization_id {
        return false;
    }
    if let Some(agent_id) = query.agent_id.as_deref() {
        if turn.agent_id != agent_id {
            return false;
        }
    }
    if let Some(session_id) = query.session_id.as_deref() {
        if turn.session_id != session_id {
            return false;
        }
    }
    if let Some(model_id) = query.model_id.as_deref() {
        if turn.model_id.as_deref() != Some(model_id) {
            return false;
        }
    }
    if let Some(from) = query.from.as_deref() {
        if turn.created_at.as_str() < from {
            return false;
        }
    }
    if let Some(to) = query.to.as_deref() {
        if turn.created_at.as_str() >= to {
            return false;
        }
    }
    true
}

/// Same filter semantics as `usage_turn_matches` for the record-list query
/// (both query types carry identical filter fields).
fn usage_record_turn_matches(
    turn: &AgentTurnRecord,
    query: &crate::usage::UsageRecordListQuery,
) -> bool {
    let summary_view = crate::usage::UsageSummaryQuery {
        tenant_id: query.tenant_id,
        organization_id: query.organization_id,
        agent_id: query.agent_id.clone(),
        session_id: query.session_id.clone(),
        model_id: query.model_id.clone(),
        from: query.from.clone(),
        to: query.to.clone(),
    };
    usage_turn_matches(turn, &summary_view)
}

fn usage_record_from_turn(turn: &AgentTurnRecord) -> crate::usage::AgentUsageRecord {
    crate::usage::AgentUsageRecord {
        internal_id: turn.id,
        turn_id: turn.turn_id.clone(),
        session_id: turn.session_id.clone(),
        agent_id: turn.agent_id.clone(),
        owner_user_id: turn.owner_user_id,
        status: turn.status.as_str().to_string(),
        model_id: turn.model_id.clone(),
        provider_id: turn.provider_id.clone(),
        input_tokens: turn.input_tokens,
        output_tokens: turn.output_tokens,
        cached_tokens: turn.cached_tokens,
        created_at: turn.created_at.clone(),
        completed_at: turn.completed_at.clone(),
    }
}

fn agent_version_primary_key(record: &AgentVersionRecord) -> AgentVersionPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.agent_id.clone(),
        record.version_id.clone(),
    )
}

fn task_primary_key(record: &AgentTaskRecord) -> TaskPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.task_id.clone(),
    )
}

fn task_index_key(record: &AgentTaskRecord) -> TaskIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
    )
}

fn active_agent_ids_for_tenant(
    agents: &HashMap<AgentPrimaryKey, AgentBusinessRecord>,
    agent_list_index: &BTreeMap<AgentIndexKey, AgentPrimaryKey>,
    tenant_id: u64,
) -> HashSet<String> {
    agent_list_index
        .iter()
        .filter(|((indexed_tenant_id, _, _, _), _)| *indexed_tenant_id == tenant_id)
        .filter_map(|(_, primary_key)| agents.get(primary_key))
        .filter(|record| !record.is_deleted())
        .map(|record| record.agent_id.clone())
        .collect()
}

impl Default for InMemoryAgentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRepository for InMemoryAgentRepository {
    fn check_readiness(&self) -> KernelResult<()> {
        Ok(())
    }

    fn next_id(&self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        let primary_key = agent_primary_key(&record);
        let mut agents = self.agents.recovering_write();
        if agents
            .values()
            .any(|existing| existing.tenant_id == record.tenant_id && existing.code == record.code)
        {
            return Err(KernelError::conflict("agent code already exists"));
        }
        if agents.contains_key(&primary_key) {
            return Err(KernelError::conflict("agent already exists"));
        }
        let index_key = agent_index_key(&record);
        agents.insert(primary_key.clone(), record);
        self.agent_list_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        let primary_key = agent_primary_key(&record);
        let mut agents = self.agents.recovering_write();
        let existing = agents
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "agent version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if agents.iter().any(|(key, existing)| {
            *key != primary_key
                && existing.tenant_id == record.tenant_id
                && existing.code == record.code
        }) {
            return Err(KernelError::conflict("agent code already exists"));
        }
        let previous_index_key = agent_index_key(existing);
        let next_index_key = agent_index_key(&record);
        agents.insert(primary_key.clone(), record);
        let mut index = self.agent_list_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRecord>> {
        Ok(self
            .agents
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string()))
            .cloned())
    }

    fn list(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRecord>> {
        let agents = self.agents.recovering_read();
        let index = self.agent_list_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| agents.get(primary_key))
            .filter(|record| agent_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_agents(&self, query: &AgentListQuery) -> KernelResult<u64> {
        let agents = self.agents.recovering_read();
        let index = self.agent_list_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| agents.get(primary_key))
                .filter(|record| agent_matches_list_query(record, query)),
        ))
    }

    fn insert_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()> {
        let primary_key = workspace_primary_key(&record);
        let mut workspaces = self.workspaces.recovering_write();
        if workspaces.contains_key(&primary_key) {
            return Err(KernelError::conflict("workspace already exists"));
        }
        if record.is_default
            && workspaces.values().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.organization_id == record.organization_id
                    && existing.owner_user_id == record.owner_user_id
                    && existing.is_default
                    && existing.status != AgentWorkspaceStatus::Deleted
            })
        {
            return Err(KernelError::conflict("default workspace already exists"));
        }
        let index_key = workspace_index_key(&record);
        workspaces.insert(primary_key.clone(), record);
        self.workspace_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()> {
        let primary_key = workspace_primary_key(&record);
        let mut workspaces = self.workspaces.recovering_write();
        let existing = workspaces
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("workspace not found"))?;
        if existing.status == AgentWorkspaceStatus::Deleted {
            return Err(KernelError::not_found("workspace not found"));
        }
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict("workspace version mismatch"));
        }
        if record.owner_user_id != existing.owner_user_id
            || record.is_default != existing.is_default
            || record.created_by != existing.created_by
            || record.created_at != existing.created_at
        {
            return Err(KernelError::validation(
                "workspace immutable identity cannot be changed",
            ));
        }
        let previous_index_key = workspace_index_key(existing);
        let next_index_key = workspace_index_key(&record);
        workspaces.insert(primary_key.clone(), record);
        let mut index = self.workspace_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<AgentWorkspaceRecord>> {
        Ok(self
            .workspaces
            .recovering_read()
            .get(&(tenant_id, organization_id, workspace_id.to_string()))
            .filter(|record| record.status != AgentWorkspaceStatus::Deleted)
            .cloned())
    }

    fn get_default_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<AgentWorkspaceRecord>> {
        Ok(self
            .workspaces
            .recovering_read()
            .values()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.owner_user_id == owner_user_id
                    && record.is_default
                    && record.status == AgentWorkspaceStatus::Active
            })
            .cloned())
    }

    fn list_workspaces(
        &self,
        query: &WorkspaceListQuery,
    ) -> KernelResult<Vec<AgentWorkspaceRecord>> {
        let workspaces = self.workspaces.recovering_read();
        let index = self.workspace_index.recovering_read();
        let records = index
            .iter()
            .filter(
                |((tenant_id, organization_id, owner_user_id, _, _, _, _), _)| {
                    *tenant_id == query.tenant_id
                        && *organization_id == query.organization_id
                        && *owner_user_id == query.owner_user_id
                },
            )
            .filter_map(|(_, key)| workspaces.get(key))
            .filter(|record| workspace_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(records, &query.pagination))
    }

    fn count_workspaces(&self, query: &WorkspaceListQuery) -> KernelResult<u64> {
        Ok(count_iterator(
            self.workspaces
                .recovering_read()
                .values()
                .filter(|record| workspace_matches_list_query(record, query)),
        ))
    }

    fn insert_project(&self, record: AgentProjectRecord) -> KernelResult<()> {
        let primary_key = project_primary_key(&record);
        let mut projects = self.projects.recovering_write();
        if projects.contains_key(&primary_key) {
            return Err(KernelError::conflict("project already exists"));
        }
        if projects.values().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.organization_id == record.organization_id
                && existing.workspace_id == record.workspace_id
                && existing.status != AgentProjectStatus::Deleted
                && project_names_equal(&existing.name, &record.name)
        }) {
            return Err(KernelError::conflict(
                "project name already exists in workspace",
            ));
        }
        if let (Some(source_kind), Some(source_ref)) = (
            record.import_source_kind.as_deref(),
            record.import_source_ref.as_deref(),
        ) {
            let import_source_exists = projects.values().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.organization_id == record.organization_id
                    && existing.owner_user_id == record.owner_user_id
                    && existing.status != AgentProjectStatus::Deleted
                    && existing.import_source_kind.as_deref() == Some(source_kind)
                    && existing.import_source_ref.as_deref() == Some(source_ref)
            });
            if import_source_exists {
                return Err(KernelError::conflict(
                    "project import source already exists",
                ));
            }
        }
        let index_key = project_index_key(&record);
        projects.insert(primary_key.clone(), record);
        self.project_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_project(&self, record: AgentProjectRecord) -> KernelResult<()> {
        let primary_key = project_primary_key(&record);
        let mut projects = self.projects.recovering_write();
        let existing = projects
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("project not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "project version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if !project_names_equal(&existing.name, &record.name)
            && projects.iter().any(|(key, candidate)| {
                key != &primary_key
                    && candidate.tenant_id == record.tenant_id
                    && candidate.organization_id == record.organization_id
                    && candidate.workspace_id == record.workspace_id
                    && candidate.status != AgentProjectStatus::Deleted
                    && project_names_equal(&candidate.name, &record.name)
            })
        {
            return Err(KernelError::conflict(
                "project name already exists in workspace",
            ));
        }
        if let (Some(source_kind), Some(source_ref)) = (
            record.import_source_kind.as_deref(),
            record.import_source_ref.as_deref(),
        ) {
            let import_source_exists = projects.iter().any(|(key, candidate)| {
                key != &primary_key
                    && candidate.tenant_id == record.tenant_id
                    && candidate.organization_id == record.organization_id
                    && candidate.owner_user_id == record.owner_user_id
                    && candidate.status != AgentProjectStatus::Deleted
                    && candidate.import_source_kind.as_deref() == Some(source_kind)
                    && candidate.import_source_ref.as_deref() == Some(source_ref)
            });
            if import_source_exists {
                return Err(KernelError::conflict(
                    "project import source already exists",
                ));
            }
        }
        let previous_index_key = project_index_key(existing);
        let next_index_key = project_index_key(&record);
        projects.insert(primary_key.clone(), record);
        let mut index = self.project_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_project(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        Ok(self
            .projects
            .recovering_read()
            .get(&(tenant_id, organization_id, project_id.to_string()))
            .filter(|record| record.status != AgentProjectStatus::Deleted)
            .cloned())
    }

    fn get_project_by_workspace_name(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
        name: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        Ok(self
            .projects
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.workspace_id == workspace_id
                    && record.status != AgentProjectStatus::Deleted
                    && project_names_equal(&record.name, name)
            })
            .min_by_key(|record| {
                (
                    u8::from(record.status != AgentProjectStatus::Active),
                    record.created_at.as_str(),
                    record.id,
                )
            })
            .cloned())
    }

    fn get_project_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        Ok(self
            .projects
            .recovering_read()
            .values()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.owner_user_id == owner_user_id
                    && record.status != AgentProjectStatus::Deleted
                    && record.import_source_kind.as_deref() == Some(source_kind)
                    && record.import_source_ref.as_deref() == Some(source_ref)
            })
            .cloned())
    }

    fn list_projects(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRecord>> {
        let projects = self.projects.recovering_read();
        let index = self.project_index.recovering_read();
        let records = index
            .iter()
            .filter(|((tenant_id, organization_id, _, _, _), _)| {
                *tenant_id == query.tenant_id && *organization_id == query.organization_id
            })
            .filter_map(|(_, key)| projects.get(key))
            .filter(|record| project_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(records, &query.pagination))
    }

    fn count_projects(&self, query: &ProjectListQuery) -> KernelResult<u64> {
        let projects = self.projects.recovering_read();
        Ok(count_iterator(projects.values().filter(|record| {
            project_matches_list_query(record, query)
        })))
    }

    fn insert_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.project_id.clone(),
            record.slot_id.clone(),
        );
        let mut slots = self.project_composition_slots.recovering_write();
        if slots.contains_key(&key) {
            return Err(KernelError::conflict(
                "project composition slot already exists",
            ));
        }
        slots.insert(key, record);
        Ok(())
    }

    fn update_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.project_id.clone(),
            record.slot_id.clone(),
        );
        let mut slots = self.project_composition_slots.recovering_write();
        let existing = slots
            .get(&key)
            .ok_or_else(|| KernelError::not_found("project composition slot not found"))?;
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict(
                "project composition slot version mismatch",
            ));
        }
        slots.insert(key, record);
        Ok(())
    }

    fn get_project_composition_slot(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentProjectCompositionSlotRecord>> {
        Ok(self
            .project_composition_slots
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                project_id.to_string(),
                slot_id.to_string(),
            ))
            .cloned())
    }

    fn list_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentProjectCompositionSlotRecord>> {
        let mut slots = self
            .project_composition_slots
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.project_id == query.project_id
                    && record.deleted_at.is_none()
                    && query
                        .slot_kind
                        .as_ref()
                        .is_none_or(|kind| &record.slot_kind == kind)
                    && query
                        .enabled
                        .is_none_or(|enabled| record.enabled == enabled)
            })
            .cloned()
            .collect::<Vec<_>>();
        slots.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(paginate_iterator(slots.into_iter(), &query.pagination))
    }

    fn count_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64> {
        Ok(self
            .project_composition_slots
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.project_id == query.project_id
                    && record.deleted_at.is_none()
                    && query
                        .slot_kind
                        .as_ref()
                        .is_none_or(|kind| &record.slot_kind == kind)
                    && query
                        .enabled
                        .is_none_or(|enabled| record.enabled == enabled)
            })
            .count() as u64)
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        let primary_key = provider_binding_primary_key(&record);
        let mut bindings = self.provider_bindings.recovering_write();
        if bindings.contains_key(&primary_key) {
            return Err(KernelError::conflict(
                "agent provider binding already exists",
            ));
        }
        if record.active
            && bindings.values().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.active
            })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists",
            ));
        }
        let index_key = provider_binding_index_key(&record);
        bindings.insert(primary_key.clone(), record);
        self.provider_binding_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        let primary_key = provider_binding_primary_key(&record);
        let mut bindings = self.provider_bindings.recovering_write();
        let existing = bindings
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("agent provider binding not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "provider binding version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if record.active
            && bindings.iter().any(|(key, existing)| {
                *key != primary_key
                    && existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.active
            })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists",
            ));
        }
        let previous_index_key = provider_binding_index_key(existing);
        let next_index_key = provider_binding_index_key(&record);
        bindings.insert(primary_key.clone(), record);
        let mut index = self.provider_binding_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        Ok(self
            .provider_bindings
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string(), binding_id.to_string()))
            .cloned())
    }

    fn get_active_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        Ok(self
            .provider_bindings
            .recovering_read()
            .values()
            .find(|binding| {
                binding.tenant_id == tenant_id && binding.agent_id == agent_id && binding.active
            })
            .cloned())
    }

    fn list_provider_bindings(
        &self,
        query: &ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        let bindings = self.provider_bindings.recovering_read();
        let index = self.provider_binding_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, agent_id, _, _, _), _)| {
                *tenant_id == query.tenant_id && agent_id == &query.agent_id
            })
            .filter_map(|(_, primary_key)| bindings.get(primary_key))
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_provider_bindings(&self, query: &ProviderBindingListQuery) -> KernelResult<u64> {
        let bindings = self.provider_bindings.recovering_read();
        let index = self.provider_binding_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, agent_id, _, _, _), _)| {
                    *tenant_id == query.tenant_id && agent_id == &query.agent_id
                })
                .filter_map(|(_, primary_key)| bindings.get(primary_key)),
        ))
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        let primary_key = composition_slot_primary_key(&record);
        let mut slots = self.composition_slots.recovering_write();
        if slots.contains_key(&primary_key) {
            return Err(KernelError::conflict("composition slot already exists"));
        }
        let index_key = composition_slot_index_key(&record);
        slots.insert(primary_key.clone(), record);
        self.composition_slot_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        let primary_key = composition_slot_primary_key(&record);
        let mut slots = self.composition_slots.recovering_write();
        let existing = slots
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("composition slot not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "composition slot version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = composition_slot_index_key(existing);
        let next_index_key = composition_slot_index_key(&record);
        slots.insert(primary_key.clone(), record);
        let mut index = self.composition_slot_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRecord>> {
        Ok(self
            .composition_slots
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string(), slot_id.to_string()))
            .cloned())
    }

    fn list_composition_slots(
        &self,
        query: &CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, agent_id, _, _), _)| {
                *tenant_id == query.tenant_id && agent_id == &query.agent_id
            })
            .filter_map(|(_, primary_key)| slots.get(primary_key))
            .filter(|record| !record.is_deleted())
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_composition_slots(&self, query: &CompositionSlotListQuery) -> KernelResult<u64> {
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, agent_id, _, _), _)| {
                    *tenant_id == query.tenant_id && agent_id == &query.agent_id
                })
                .filter_map(|(_, primary_key)| slots.get(primary_key))
                .filter(|record| !record.is_deleted()),
        ))
    }

    fn list_mcp_marketplace_slots(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        let agents = self.agents.recovering_read();
        let agent_index = self.agent_list_index.recovering_read();
        let active_agent_ids = active_agent_ids_for_tenant(&agents, &agent_index, query.tenant_id);
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| slots.get(primary_key))
            .filter(|record| {
                record.slot_kind == AgentCompositionSlotKind::Mcp
                    && !record.is_deleted()
                    && active_agent_ids.contains(&record.agent_id)
                    && query
                        .q
                        .as_deref()
                        .map(|q| {
                            crate::mcp_marketplace::composition_slot_matches_mcp_search(record, q)
                        })
                        .unwrap_or(true)
            })
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_mcp_marketplace_slots(&self, query: &McpMarketplaceListQuery) -> KernelResult<u64> {
        let agents = self.agents.recovering_read();
        let agent_index = self.agent_list_index.recovering_read();
        let active_agent_ids = active_agent_ids_for_tenant(&agents, &agent_index, query.tenant_id);
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| slots.get(primary_key))
                .filter(|record| {
                    record.slot_kind == AgentCompositionSlotKind::Mcp
                        && !record.is_deleted()
                        && active_agent_ids.contains(&record.agent_id)
                        && query
                            .q
                            .as_deref()
                            .map(|q| {
                                crate::mcp_marketplace::composition_slot_matches_mcp_search(
                                    record, q,
                                )
                            })
                            .unwrap_or(true)
                }),
        ))
    }

    fn insert_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        let activity_record = record.clone();
        let primary_key = session_primary_key(&record);
        let mut sessions = self.sessions.recovering_write();
        if sessions.contains_key(&primary_key) {
            return Err(KernelError::conflict("session already exists"));
        }
        let idempotency_key = record.idempotency_key.as_ref().map(|idempotency_key| {
            (
                record.tenant_id,
                record.organization_id,
                record.owner_user_id,
                idempotency_key.clone(),
            )
        });
        let mut session_idempotency = self.session_idempotency.recovering_write();
        if idempotency_key
            .as_ref()
            .is_some_and(|key| session_idempotency.contains_key(key))
        {
            return Err(KernelError::conflict(
                "session creation idempotency key already exists",
            ));
        }
        let index_key = session_index_key(&record);
        sessions.insert(primary_key.clone(), record);
        self.session_index
            .recovering_write()
            .insert(index_key, primary_key.clone());
        if let Some(idempotency_key) = idempotency_key {
            session_idempotency.insert(idempotency_key, primary_key);
        }
        self.advance_session_activity(
            &activity_record,
            &activity_record.updated_at,
            SessionActivitySource::Session,
        );
        Ok(())
    }

    fn update_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        let activity_record = record.clone();
        let primary_key = session_primary_key(&record);
        let mut sessions = self.sessions.recovering_write();
        let existing = sessions
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "session version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = session_index_key(existing);
        let next_index_key = session_index_key(&record);
        sessions.insert(primary_key.clone(), record);
        let mut index = self.session_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        self.advance_session_activity(
            &activity_record,
            &activity_record.updated_at,
            SessionActivitySource::Session,
        );
        Ok(())
    }

    fn delete_session_and_purge_queue(
        &self,
        deleted_session: AgentSessionRecord,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<()> {
        // In-memory repository: mirror the transactional delete of the durable
        // adapter — soft-delete the session row and purge its queue entries
        // under the same lock so a partial failure cannot occur.
        let activity_record = deleted_session.clone();
        let primary_key = session_primary_key(&deleted_session);
        let mut sessions = self.sessions.recovering_write();
        let existing = sessions
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if deleted_session.version != expected_version {
            return Err(KernelError::conflict(format!(
                "session version mismatch: expected={expected_version}, actual={}",
                deleted_session.version
            )));
        }
        let previous_index_key = session_index_key(existing);
        let next_index_key = session_index_key(&deleted_session);
        sessions.insert(primary_key.clone(), deleted_session);
        let mut index = self.session_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        let mut queue = self.turn_input_queue.recovering_lock();
        queue.retain(|_, entry| {
            entry.tenant_id != tenant_id
                || entry.organization_id != organization_id
                || entry.session_id != session_id
                || entry.owner_user_id != owner_user_id
        });
        drop(queue);
        drop(index);
        drop(sessions);
        self.advance_session_activity(
            &activity_record,
            &activity_record.updated_at,
            SessionActivitySource::Session,
        );
        Ok(())
    }

    fn get_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRecord>> {
        Ok(self
            .sessions
            .recovering_read()
            .get(&(tenant_id, organization_id, session_id.to_string()))
            .filter(|record| record.deleted_at.is_none())
            .cloned())
    }

    fn get_session_by_creation_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentSessionRecord>> {
        let primary_key = self
            .session_idempotency
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key.to_string(),
            ))
            .cloned();
        Ok(primary_key.and_then(|key| self.sessions.recovering_read().get(&key).cloned()))
    }

    fn list_sessions(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRecord>> {
        let workspace_project_ids = self.workspace_project_ids(query);
        let sessions = self.sessions.recovering_read();
        let index = self.session_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| sessions.get(primary_key))
            .filter(|record| {
                session_matches_list_query(record, query, workspace_project_ids.as_ref())
            })
            .filter(|record| {
                query.cursor.as_ref().is_none_or(|cursor| {
                    record.updated_at < cursor.updated_at
                        || (record.updated_at == cursor.updated_at
                            && record.id < cursor.session_internal_id)
                })
            })
            .cloned();
        let store_limit = query.pagination.page_size.saturating_add(1);
        Ok(iter.take(store_limit).collect())
    }

    fn count_sessions(&self, query: &SessionListQuery) -> KernelResult<u64> {
        let workspace_project_ids = self.workspace_project_ids(query);
        let sessions = self.sessions.recovering_read();
        let index = self.session_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| sessions.get(primary_key))
                .filter(|record| {
                    session_matches_list_query(record, query, workspace_project_ids.as_ref())
                }),
        ))
    }

    fn list_session_activity_summaries(
        &self,
        query: &SessionActivitySummaryListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<SessionActivitySummaryRecord>> {
        let sessions = self.sessions.recovering_read();
        let projects = self.projects.recovering_read();
        let activity_index = self.session_activity_index.recovering_read();
        let lower = (
            query.tenant_id,
            query.organization_id,
            query.owner_user_id,
            String::new(),
            0,
        );
        let upper = query.cursor.as_ref().map_or_else(
            || {
                std::ops::Bound::Included((
                    query.tenant_id,
                    query.organization_id,
                    query.owner_user_id,
                    "~".to_string(),
                    u64::MAX,
                ))
            },
            |cursor| {
                std::ops::Bound::Excluded((
                    query.tenant_id,
                    query.organization_id,
                    query.owner_user_id,
                    cursor.activity_at.clone(),
                    cursor.session_internal_id,
                ))
            },
        );
        let mut heads = activity_index
            .range((std::ops::Bound::Included(lower), upper))
            .rev()
            .filter_map(|(index_key, (primary_key, source))| {
                sessions
                    .get(primary_key)
                    .map(|session| (index_key.3.clone(), *source, session.clone()))
            })
            .filter(|(_, _, session)| {
                query
                    .agent_id
                    .as_ref()
                    .is_none_or(|agent_id| session.agent_id == *agent_id)
                    && query
                        .project_id
                        .as_ref()
                        .is_none_or(|project_id| session.project_id.as_ref() == Some(project_id))
                    && query.workspace_id.as_ref().is_none_or(|workspace_id| {
                        session.project_id.as_ref().is_some_and(|project_id| {
                            projects
                                .get(&(
                                    session.tenant_id,
                                    session.organization_id,
                                    project_id.clone(),
                                ))
                                .is_some_and(|project| {
                                    project.deleted_at.is_none()
                                        && project.workspace_id == *workspace_id
                                })
                        })
                    })
            })
            .take(query.page_size.saturating_add(1))
            .collect::<Vec<_>>();
        drop(activity_index);
        drop(projects);
        drop(sessions);

        let has_more = heads.len() > query.page_size;
        if has_more {
            heads.pop();
        }
        let mut items = Vec::with_capacity(heads.len());
        for (activity_at, source, session) in heads {
            let latest_turn = self.latest_turn_for_session(
                session.tenant_id,
                session.organization_id,
                &session.session_id,
            );
            let pending_interaction = self.pending_interaction_for_session(
                session.tenant_id,
                session.organization_id,
                &session.session_id,
            );
            let current_runtime_binding = self.current_runtime_binding_for_session(
                session.tenant_id,
                session.organization_id,
                &session.session_id,
            );
            let latest_runtime_binding = self.latest_runtime_binding_for_session(
                session.tenant_id,
                session.organization_id,
                &session.session_id,
            );
            let user_state = self.session_user_state_for_owner(&session);
            let latest_interaction_component = self.latest_interaction_component_for_session(
                session.tenant_id,
                session.organization_id,
                &session.session_id,
            );
            items.push(SessionActivitySummaryRecord::from_parts(
                SessionActivitySummaryParts {
                    session,
                    latest_turn,
                    pending_interaction,
                    current_runtime_binding,
                    latest_runtime_binding,
                    user_state,
                    latest_interaction_component,
                    activity_at,
                    activity_source: source,
                },
            ));
        }
        let next_page_token = if has_more {
            items
                .last()
                .map(|summary| SessionActivityCursor {
                    activity_at: summary.freshness.activity_at.clone(),
                    session_internal_id: summary.session.id,
                    scope_fingerprint: query.scope_fingerprint(),
                })
                .map(|cursor| encode_session_activity_cursor(&cursor))
                .transpose()?
        } else {
            None
        };
        Ok(crate::ports::PaginatedResult {
            items,
            next_page_token,
            total_count: None,
            has_more,
        })
    }

    fn insert_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        let activity_at = record.updated_at.clone();
        let session_key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
        );
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.runtime_binding_id.clone(),
        );
        let mut bindings = self.session_runtime_bindings.recovering_write();
        if bindings.contains_key(&key) {
            return Err(KernelError::conflict(
                "session runtime binding already exists",
            ));
        }
        if record.is_current
            && bindings.values().any(|candidate| {
                candidate.tenant_id == record.tenant_id
                    && candidate.organization_id == record.organization_id
                    && candidate.session_id == record.session_id
                    && candidate.is_current
            })
        {
            return Err(KernelError::conflict(
                "session already has a current runtime binding",
            ));
        }
        let is_current = record.is_current;
        bindings.insert(key.clone(), record);
        if is_current {
            self.current_session_runtime_bindings
                .recovering_write()
                .insert(session_key.clone(), key);
        }
        if let Some(session) = self.sessions.recovering_read().get(&session_key).cloned() {
            self.advance_session_activity(
                &session,
                &activity_at,
                SessionActivitySource::RuntimeBinding,
            );
        }
        Ok(())
    }

    fn update_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        let activity_at = record.updated_at.clone();
        let session_key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
        );
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.runtime_binding_id.clone(),
        );
        let mut bindings = self.session_runtime_bindings.recovering_write();
        let existing = bindings
            .get(&key)
            .ok_or_else(|| KernelError::not_found("session runtime binding not found"))?;
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict(
                "session runtime binding version mismatch",
            ));
        }
        if record.is_current
            && bindings.iter().any(|(candidate_key, candidate)| {
                candidate_key != &key
                    && candidate.tenant_id == record.tenant_id
                    && candidate.organization_id == record.organization_id
                    && candidate.session_id == record.session_id
                    && candidate.is_current
            })
        {
            return Err(KernelError::conflict(
                "session already has a current runtime binding",
            ));
        }
        let is_current = record.is_current;
        bindings.insert(key.clone(), record);
        let mut current = self.current_session_runtime_bindings.recovering_write();
        if is_current {
            current.insert(session_key.clone(), key);
        } else if current.get(&session_key) == Some(&key) {
            current.remove(&session_key);
        }
        drop(current);
        if let Some(session) = self.sessions.recovering_read().get(&session_key).cloned() {
            self.advance_session_activity(
                &session,
                &activity_at,
                SessionActivitySource::RuntimeBinding,
            );
        }
        Ok(())
    }

    fn get_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>> {
        Ok(self
            .session_runtime_bindings
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                runtime_binding_id.to_string(),
            ))
            .cloned())
    }

    fn get_session_runtime_binding_by_provider_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        provider_binding_id: &str,
        provider_session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>> {
        Ok(self
            .session_runtime_bindings
            .recovering_read()
            .values()
            .find(|binding| {
                binding.tenant_id == tenant_id
                    && binding.organization_id == organization_id
                    && binding.owner_user_id == owner_user_id
                    && binding.provider_binding_id == provider_binding_id
                    && binding.provider_session_id.as_deref() == Some(provider_session_id)
            })
            .cloned())
    }

    fn get_current_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>> {
        Ok(self.current_runtime_binding_for_session(tenant_id, organization_id, session_id))
    }

    fn list_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<AgentSessionRuntimeBindingRecord>> {
        let mut records = self
            .session_runtime_bindings
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.session_id == query.session_id
                    && (!query.current_only || record.is_current)
                    && query
                        .status
                        .as_deref()
                        .map(|status| record.status.as_str() == status)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .is_current
                .cmp(&left.is_current)
                .then_with(|| right.updated_at.cmp(&left.updated_at))
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(paginate_iterator(records.into_iter(), &query.pagination))
    }

    fn count_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64> {
        Ok(count_iterator(
            self.session_runtime_bindings
                .recovering_read()
                .values()
                .filter(|record| {
                    record.tenant_id == query.tenant_id
                        && record.organization_id == query.organization_id
                        && record.session_id == query.session_id
                        && (!query.current_only || record.is_current)
                        && query
                            .status
                            .as_deref()
                            .map(|status| record.status.as_str() == status)
                            .unwrap_or(true)
                }),
        ))
    }

    fn activate_session_runtime_binding_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<AgentSessionRuntimeBindingRecord> {
        let target_key = (
            tenant_id,
            organization_id,
            session_id.to_string(),
            runtime_binding_id.to_string(),
        );
        let mut bindings = self.session_runtime_bindings.recovering_write();
        let target = bindings
            .get(&target_key)
            .cloned()
            .ok_or_else(|| KernelError::not_found("session runtime binding not found"))?;
        if target.version != expected_version {
            return Err(KernelError::conflict(
                "session runtime binding version mismatch",
            ));
        }
        if target.is_current && target.status == AgentSessionRuntimeBindingStatus::Active {
            return Ok(target);
        }
        for candidate in bindings.values_mut().filter(|candidate| {
            candidate.tenant_id == tenant_id
                && candidate.organization_id == organization_id
                && candidate.session_id == session_id
                && candidate.runtime_binding_id != runtime_binding_id
                && candidate.is_current
        }) {
            candidate.deactivate(
                AgentSessionRuntimeBindingStatus::Deactivated,
                updated_at.clone(),
            );
        }
        let target = bindings
            .get_mut(&target_key)
            .ok_or_else(|| KernelError::not_found("session runtime binding not found"))?;
        target.activate(updated_at);
        let target = target.clone();
        self.current_session_runtime_bindings
            .recovering_write()
            .insert(
                (tenant_id, organization_id, session_id.to_string()),
                target_key,
            );
        if let Some(session) = self
            .sessions
            .recovering_read()
            .get(&(tenant_id, organization_id, session_id.to_string()))
            .cloned()
        {
            self.advance_session_activity(
                &session,
                &target.updated_at,
                SessionActivitySource::RuntimeBinding,
            );
        }
        Ok(target)
    }

    fn insert_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.checkpoint_id.clone(),
        );
        let mut checkpoints = self.session_checkpoints.recovering_write();
        if checkpoints.contains_key(&key) {
            return Err(KernelError::conflict("session checkpoint already exists"));
        }
        checkpoints.insert(key, record);
        Ok(())
    }

    fn update_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.checkpoint_id.clone(),
        );
        let mut checkpoints = self.session_checkpoints.recovering_write();
        let existing = checkpoints
            .get(&key)
            .ok_or_else(|| KernelError::not_found("session checkpoint not found"))?;
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict("session checkpoint version mismatch"));
        }
        checkpoints.insert(key, record);
        Ok(())
    }

    fn get_session_checkpoint(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<AgentSessionCheckpointRecord>> {
        Ok(self
            .session_checkpoints
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                checkpoint_id.to_string(),
            ))
            .cloned())
    }

    fn list_session_checkpoints(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<Vec<AgentSessionCheckpointRecord>> {
        let mut records = self
            .session_checkpoints
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.session_id == query.session_id
                    && query
                        .status
                        .as_deref()
                        .map(|status| record.status.as_str() == status)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(paginate_iterator(records.into_iter(), &query.pagination))
    }

    fn count_session_checkpoints(&self, query: &SessionCheckpointListQuery) -> KernelResult<u64> {
        Ok(count_iterator(
            self.session_checkpoints
                .recovering_read()
                .values()
                .filter(|record| {
                    record.tenant_id == query.tenant_id
                        && record.organization_id == query.organization_id
                        && record.session_id == query.session_id
                        && query
                            .status
                            .as_deref()
                            .map(|status| record.status.as_str() == status)
                            .unwrap_or(true)
                }),
        ))
    }

    fn upsert_resource_user_state(
        &self,
        record: AgentResourceUserStateRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentResourceUserStateRecord> {
        let key = resource_user_state_primary_key(&record);
        let mut states = self.resource_user_states.recovering_write();
        match states.get(&key) {
            Some(existing) => {
                if expected_version != Some(existing.version) {
                    return Err(KernelError::conflict(
                        "resource user state version mismatch",
                    ));
                }
                if record.version != existing.version.saturating_add(1) {
                    return Err(KernelError::conflict(
                        "resource user state version mismatch",
                    ));
                }
            }
            None => {
                if expected_version.is_some() || record.version != 0 {
                    return Err(KernelError::conflict(
                        "resource user state version mismatch",
                    ));
                }
            }
        }
        states.insert(key, record.clone());
        drop(states);
        if record.resource_type == AgentResourceType::Session {
            let session_key = (
                record.tenant_id,
                record.organization_id,
                record.resource_id.clone(),
            );
            if let Some(session) = self.sessions.recovering_read().get(&session_key).cloned() {
                if session.owner_user_id == record.user_id {
                    self.advance_session_activity(
                        &session,
                        &record.updated_at,
                        SessionActivitySource::UserState,
                    );
                }
            }
        }
        Ok(record)
    }

    fn get_resource_user_state(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<AgentResourceUserStateRecord>> {
        Ok(self
            .resource_user_states
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                user_id,
                resource_type.as_db_code(),
                resource_id.to_owned(),
            ))
            .cloned())
    }

    /// InMemory fallback for contract tests: sorts and windows the caller's
    /// resource-user-state set in memory. Production postgres-sync uses SQL
    /// LIMIT/OFFSET with indexed tenant scope.
    fn list_resource_user_states(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<Vec<AgentResourceUserStateRecord>> {
        let sessions = self.sessions.recovering_read();
        let mut records = self
            .resource_user_states
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.resource_type == query.resource_type
                    && (query.resource_ids.is_empty()
                        || query.resource_ids.contains(&record.resource_id))
                    && (!query.pinned_only || record.pinned_at.is_some())
                    && (query.include_hidden || record.hidden_at.is_none())
            })
            .filter(|record| resource_user_state_matches_agent(record, query, &sessions))
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .pinned_at
                .cmp(&left.pinned_at)
                .then_with(|| right.last_opened_at.cmp(&left.last_opened_at))
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(records
            .into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.page_size)
            .collect())
    }

    fn count_resource_user_states(&self, query: &ResourceUserStateListQuery) -> KernelResult<u64> {
        let sessions = self.sessions.recovering_read();
        Ok(self
            .resource_user_states
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.resource_type == query.resource_type
                    && (query.resource_ids.is_empty()
                        || query.resource_ids.contains(&record.resource_id))
                    && (!query.pinned_only || record.pinned_at.is_some())
                    && (query.include_hidden || record.hidden_at.is_none())
            })
            .filter(|record| resource_user_state_matches_agent(record, query, &sessions))
            .count() as u64)
    }
    fn upsert_tool_configuration(
        &self,
        record: AgentToolConfigurationRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentToolConfigurationRecord> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.tool_id.clone(),
        );
        let mut configurations = self.tool_configurations.recovering_write();
        let existing = configurations.get(&key).cloned();
        if let Some(existing) = &existing {
            if let Some(expected) = expected_version {
                if existing.version != expected {
                    return Err(KernelError::conflict("tool configuration version mismatch"));
                }
            }
            let mut updated = record;
            updated.id = existing.id;
            updated.version = existing.version + 1;
            configurations.insert(key, updated.clone());
            return Ok(updated);
        }
        configurations.insert(key, record.clone());
        Ok(record)
    }

    fn get_tool_configuration(
        &self,
        tenant_id: u64,
        organization_id: u64,
        tool_id: &str,
    ) -> KernelResult<Option<AgentToolConfigurationRecord>> {
        Ok(self
            .tool_configurations
            .recovering_read()
            .get(&(tenant_id, organization_id, tool_id.to_string()))
            .cloned())
    }

    fn list_tool_configurations(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> KernelResult<Vec<AgentToolConfigurationRecord>> {
        let mut records: Vec<AgentToolConfigurationRecord> = self
            .tool_configurations
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == tenant_id && record.organization_id == organization_id
            })
            .cloned()
            .collect();
        records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        Ok(records)
    }

    fn insert_tool_asset(&self, record: AgentToolAssetRecord) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.drive_space_id.clone(),
            record.drive_node_id.clone(),
        );
        self.tool_assets
            .recovering_write()
            .entry(key)
            .or_insert(record);
        Ok(())
    }

    fn list_tool_assets(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        limit: u64,
    ) -> KernelResult<Vec<AgentToolAssetRecord>> {
        let mut records: Vec<AgentToolAssetRecord> = self
            .tool_assets
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && (user_id == 0 || record.user_id == user_id)
            })
            .cloned()
            .collect();
        records.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        records.truncate(limit as usize);
        Ok(records)
    }

    fn append_session_item(
        &self,
        mut record: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)> {
        if record.sequence != 0
            || record.turn_id.is_some()
            || record.status == AgentSessionItemStatus::Redacted
        {
            return Err(KernelError::validation(
                "standalone session item must be unsequenced, non-redacted, and without a turn",
            ));
        }
        let session_primary_key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
        );
        let mut sessions = self.sessions.recovering_write();
        let mut session_index = self.session_index.recovering_write();
        let mut items = self.items.recovering_write();
        let mut item_index = self.session_item_index.recovering_write();
        let existing_session = sessions
            .get(&session_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::not_found("active session not found"))?;
        if !existing_session.status.is_active()
            || existing_session.deleted_at.is_some()
            || existing_session.owner_user_id != record.created_by
        {
            return Err(KernelError::not_found("active session not found"));
        }
        record.sequence = existing_session.last_item_sequence.saturating_add(1);
        let primary_key = session_item_primary_key(&record);
        if items.contains_key(&primary_key) {
            return Err(KernelError::conflict("session item already exists"));
        }
        let index_key = session_item_index_key(&record);
        if item_index.contains_key(&index_key) {
            return Err(KernelError::conflict("session item sequence conflict"));
        }
        let mut updated_session = existing_session.clone();
        updated_session.updated_by = record.created_by;
        updated_session.record_item(
            record.input_tokens,
            record.output_tokens,
            record.updated_at.clone(),
        );
        let previous_session_index_key = session_index_key(&existing_session);
        let next_session_index_key = session_index_key(&updated_session);

        items.insert(primary_key.clone(), record.clone());
        item_index.insert(index_key, primary_key);
        sessions.insert(session_primary_key.clone(), updated_session.clone());
        session_index.remove(&previous_session_index_key);
        session_index.insert(next_session_index_key, session_primary_key);
        self.advance_session_activity(
            &updated_session,
            &record.updated_at,
            SessionActivitySource::Session,
        );
        Ok((updated_session, record))
    }

    fn update_session_item(&self, record: AgentSessionItemRecord) -> KernelResult<()> {
        let primary_key = session_item_primary_key(&record);
        let mut items = self.items.recovering_write();
        let Some(existing) = items.get(&primary_key) else {
            return Err(KernelError::not_found("session item not found"));
        };
        let expected_version = existing
            .version
            .checked_add(1)
            .ok_or_else(|| KernelError::conflict("session item version overflow"))?;
        if record.version != expected_version {
            return Err(KernelError::conflict("session item version conflict"));
        }
        let previous_index_key = session_item_index_key(existing);
        let next_index_key = session_item_index_key(&record);
        items.insert(primary_key.clone(), record);
        let mut index = self.session_item_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_session_item(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<AgentSessionItemRecord>> {
        Ok(self
            .items
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                item_id.to_string(),
            ))
            .cloned())
    }

    fn list_session_items(
        &self,
        query: &SessionItemListQuery,
    ) -> KernelResult<Vec<AgentSessionItemRecord>> {
        let items = self.items.recovering_read();
        let index = self.session_item_index.recovering_read();
        let scope_start = (
            query.tenant_id,
            query.organization_id,
            query.session_id.clone(),
            0,
            0,
        );
        let scope_end = (
            query.tenant_id,
            query.organization_id,
            query.session_id.clone(),
            u64::MAX,
            u64::MAX,
        );

        if query.cursor_mode {
            use std::ops::Bound::{Excluded, Included};

            let boundary = query.cursor.as_ref().map(|cursor| {
                (
                    query.tenant_id,
                    query.organization_id,
                    query.session_id.clone(),
                    cursor.sequence,
                    cursor.item_internal_id,
                )
            });
            let records = match query.sort {
                SessionItemListSort::SequenceAsc => {
                    let start = boundary.as_ref().map_or_else(
                        || Included(scope_start.clone()),
                        |value| Excluded(value.clone()),
                    );
                    index
                        .range((start, Included(scope_end.clone())))
                        .filter_map(|(_, primary_key)| items.get(primary_key))
                        .filter(|record| message_matches_list_query(record, query))
                        .take(query.repository_page_size())
                        .cloned()
                        .collect()
                }
                SessionItemListSort::SequenceDesc => {
                    let end = boundary.as_ref().map_or_else(
                        || Included(scope_end.clone()),
                        |value| Excluded(value.clone()),
                    );
                    index
                        .range((Included(scope_start.clone()), end))
                        .rev()
                        .filter_map(|(_, primary_key)| items.get(primary_key))
                        .filter(|record| message_matches_list_query(record, query))
                        .take(query.repository_page_size())
                        .cloned()
                        .collect()
                }
                SessionItemListSort::RecentContextDesc => {
                    return Err(KernelError::validation(
                        "recent context pagination does not accept a cursor",
                    ));
                }
            };
            return Ok(records);
        }

        let iter = index
            .range(scope_start..=scope_end)
            .filter_map(|(_, primary_key)| items.get(primary_key))
            .filter(|record| message_matches_list_query(record, query))
            .cloned();
        Ok(paginate_items(iter, &query.pagination, query.sort))
    }

    fn count_session_items(&self, query: &SessionItemListQuery) -> KernelResult<u64> {
        let items = self.items.recovering_read();
        let index = self.session_item_index.recovering_read();
        let scope_start = (
            query.tenant_id,
            query.organization_id,
            query.session_id.clone(),
            0,
            0,
        );
        let scope_end = (
            query.tenant_id,
            query.organization_id,
            query.session_id.clone(),
            u64::MAX,
            u64::MAX,
        );
        Ok(count_iterator(
            index
                .range(scope_start..=scope_end)
                .filter_map(|(_, primary_key)| items.get(primary_key))
                .filter(|record| message_matches_list_query(record, query)),
        ))
    }

    fn list_session_items_by_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        turn_id: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentSessionItemRecord>> {
        if limit == 0 {
            return Err(KernelError::validation(
                "turn session-item limit must be greater than zero",
            ));
        }
        let items = self.items.recovering_read();
        let index = self.session_item_index.recovering_read();
        let scope_start = (tenant_id, organization_id, session_id.to_string(), 0, 0);
        let scope_end = (
            tenant_id,
            organization_id,
            session_id.to_string(),
            u64::MAX,
            u64::MAX,
        );
        Ok(index
            .range(scope_start..=scope_end)
            .filter_map(|(_, primary_key)| items.get(primary_key))
            .filter(|record| record.turn_id.as_deref() == Some(turn_id))
            .take(limit)
            .cloned()
            .collect())
    }

    fn upsert_item_feedback(
        &self,
        record: AgentItemFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentItemFeedbackRecord> {
        let key = item_feedback_primary_key(&record);
        let mut feedback = self.item_feedback.recovering_write();
        match feedback.get(&key) {
            Some(existing) => {
                let reviving_without_version = existing.deleted_at.is_some()
                    && record.deleted_at.is_none()
                    && expected_version.is_none();
                if !reviving_without_version && expected_version != Some(existing.version) {
                    return Err(KernelError::conflict("item feedback version mismatch"));
                }
                if record.version != existing.version.saturating_add(1) {
                    return Err(KernelError::conflict("item feedback version mismatch"));
                }
            }
            None => {
                if expected_version.is_some() || record.version != 0 {
                    return Err(KernelError::conflict("item feedback version mismatch"));
                }
            }
        }
        feedback.insert(key, record.clone());
        Ok(record)
    }

    fn get_item_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentItemFeedbackRecord>> {
        Ok(self
            .item_feedback
            .recovering_read()
            .get(&(tenant_id, organization_id, item_id.to_string(), user_id))
            .filter(|record| include_deleted || record.deleted_at.is_none())
            .cloned())
    }

    fn list_item_feedback(
        &self,
        query: &ItemFeedbackListQuery,
    ) -> KernelResult<Vec<AgentItemFeedbackRecord>> {
        let items = self.items.recovering_read();
        let mut records = self
            .item_feedback
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.deleted_at.is_none()
            })
            .filter_map(|record| {
                items
                    .get(&(
                        query.tenant_id,
                        query.organization_id,
                        query.session_id.clone(),
                        record.item_id.clone(),
                    ))
                    .map(|message| (message.sequence, record.clone()))
            })
            .collect::<Vec<_>>();
        records.sort_by_key(|(sequence, record)| (*sequence, record.id));
        Ok(records
            .into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.page_size)
            .map(|(_, record)| record)
            .collect())
    }

    fn count_item_feedback(&self, query: &ItemFeedbackListQuery) -> KernelResult<u64> {
        let items = self.items.recovering_read();
        Ok(self
            .item_feedback
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.deleted_at.is_none()
                    && items.contains_key(&(
                        query.tenant_id,
                        query.organization_id,
                        query.session_id.clone(),
                        record.item_id.clone(),
                    ))
            })
            .count() as u64)
    }

    fn get_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        let index = self.turn_idempotency.recovering_read();
        let turns = self.turns.recovering_read();
        Ok(index
            .get(&(
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key.to_string(),
            ))
            .and_then(|primary_key| turns.get(primary_key))
            .cloned())
    }

    fn get_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        Ok(self
            .turns
            .recovering_read()
            .get(&(tenant_id, organization_id, turn_id.to_string()))
            .cloned())
    }

    /// InMemory fallback for contract tests: filters the session's Turn set
    /// in memory (bounded per session). Production postgres-sync uses SQL
    /// keyset pagination (PAGINATION_SPEC: no in-process pagination).
    fn list_turns(&self, query: &TurnListQuery) -> KernelResult<Vec<AgentTurnRecord>> {
        let mut records = self
            .turns
            .recovering_read()
            .values()
            .filter(|turn| {
                turn.tenant_id == query.tenant_id
                    && turn.organization_id == query.organization_id
                    && turn.session_id == query.session_id
                    && query
                        .status
                        .as_deref()
                        .map(|status| turn.status.as_str() == status)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        let store_limit = query.pagination.page_size.saturating_add(1);
        Ok(records
            .into_iter()
            .filter(|turn| {
                query.cursor.as_ref().is_none_or(|cursor| {
                    turn.created_at < cursor.created_at
                        || (turn.created_at == cursor.created_at && turn.id < cursor.internal_id)
                })
            })
            .take(store_limit)
            .collect())
    }

    fn count_turns(&self, query: &TurnListQuery) -> KernelResult<u64> {
        Ok(count_iterator(
            self.turns.recovering_read().values().filter(|turn| {
                turn.tenant_id == query.tenant_id
                    && turn.organization_id == query.organization_id
                    && turn.session_id == query.session_id
                    && query
                        .status
                        .as_deref()
                        .map(|status| turn.status.as_str() == status)
                        .unwrap_or(true)
            }),
        ))
    }

    fn get_turn_input_queue_entry(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        queue_entry_id: &str,
    ) -> KernelResult<Option<AgentTurnInputQueueEntry>> {
        Ok(self
            .turn_input_queue
            .recovering_lock()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                queue_entry_id.to_string(),
            ))
            .filter(|entry| entry.owner_user_id == owner_user_id)
            .cloned())
    }

    fn list_turn_input_queue_entries(
        &self,
        query: &TurnInputQueueListQuery,
        owner_user_id: u64,
    ) -> KernelResult<Vec<AgentTurnInputQueueEntry>> {
        let mut entries = self
            .turn_input_queue
            .recovering_lock()
            .values()
            .filter(|entry| {
                entry.tenant_id == query.tenant_id
                    && entry.organization_id == query.organization_id
                    && entry.session_id == query.session_id
                    && entry.owner_user_id == owner_user_id
            })
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(paginate_iterator(entries.into_iter(), &query.pagination))
    }

    fn count_turn_input_queue_entries(
        &self,
        query: &TurnInputQueueListQuery,
        owner_user_id: u64,
    ) -> KernelResult<u64> {
        Ok(self
            .turn_input_queue
            .recovering_lock()
            .values()
            .filter(|entry| {
                entry.tenant_id == query.tenant_id
                    && entry.organization_id == query.organization_id
                    && entry.session_id == query.session_id
                    && entry.owner_user_id == owner_user_id
            })
            .count() as u64)
    }

    fn insert_turn_input_queue_entry(
        &self,
        mut entry: AgentTurnInputQueueEntry,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        let key = (
            entry.tenant_id,
            entry.organization_id,
            entry.session_id.clone(),
            entry.queue_entry_id.clone(),
        );
        let mut queue = self.turn_input_queue.recovering_lock();
        if let Some(existing) = queue.get(&key) {
            return if existing.payload_hash == entry.payload_hash
                && existing.owner_user_id == entry.owner_user_id
            {
                Ok(existing.clone())
            } else {
                Err(KernelError::conflict(
                    "queued Turn input idempotency conflict",
                ))
            };
        }
        let scoped = queue.values().filter(|candidate| {
            candidate.tenant_id == entry.tenant_id
                && candidate.organization_id == entry.organization_id
                && candidate.session_id == entry.session_id
                && candidate.owner_user_id == entry.owner_user_id
        });
        let (count, content_bytes, maximum_position) = scoped.fold(
            (0usize, 0usize, 0u64),
            |(count, content_bytes, position), candidate| {
                (
                    count.saturating_add(1),
                    content_bytes.saturating_add(turn_input_queue_payload_bytes(candidate)),
                    position.max(candidate.position),
                )
            },
        );
        if count >= MAX_TURN_INPUT_QUEUE_ENTRIES_PER_SESSION {
            return Err(KernelError::conflict("queued Turn input limit reached"));
        }
        let entry_bytes = turn_input_queue_payload_bytes(&entry);
        if content_bytes.saturating_add(entry_bytes)
            > MAX_TURN_INPUT_QUEUE_CONTENT_BYTES_PER_SESSION
        {
            return Err(KernelError::conflict(
                "queued Turn input content budget reached",
            ));
        }
        entry.position = maximum_position
            .checked_add(1024)
            .ok_or_else(|| KernelError::conflict("queued Turn input position overflow"))?;
        queue.insert(key, entry.clone());
        Ok(entry)
    }

    fn update_turn_input_queue_entry(
        &self,
        entry: AgentTurnInputQueueEntry,
        expected_version: u64,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        let key = (
            entry.tenant_id,
            entry.organization_id,
            entry.session_id.clone(),
            entry.queue_entry_id.clone(),
        );
        let mut queue = self.turn_input_queue.recovering_lock();
        let existing = queue
            .get(&key)
            .ok_or_else(|| KernelError::not_found("queued Turn input not found"))?;
        if existing.owner_user_id != entry.owner_user_id
            || existing.version != expected_version
            || entry.version != expected_version.saturating_add(1)
            || existing.status == AgentTurnInputQueueStatus::Executing
        {
            return Err(KernelError::conflict("queued Turn input update conflict"));
        }
        let content_bytes = queue
            .values()
            .filter(|candidate| {
                candidate.tenant_id == entry.tenant_id
                    && candidate.organization_id == entry.organization_id
                    && candidate.session_id == entry.session_id
                    && candidate.owner_user_id == entry.owner_user_id
                    && candidate.queue_entry_id != entry.queue_entry_id
            })
            .fold(0usize, |total, candidate| {
                total.saturating_add(turn_input_queue_payload_bytes(candidate))
            });
        if content_bytes.saturating_add(turn_input_queue_payload_bytes(&entry))
            > MAX_TURN_INPUT_QUEUE_CONTENT_BYTES_PER_SESSION
        {
            return Err(KernelError::conflict(
                "queued Turn input content budget reached",
            ));
        }
        queue.insert(key, entry.clone());
        Ok(entry)
    }

    fn remove_turn_input_queue_entry(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        queue_entry_id: &str,
        expected_version: u64,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        let key = (
            tenant_id,
            organization_id,
            session_id.to_string(),
            queue_entry_id.to_string(),
        );
        let mut queue = self.turn_input_queue.recovering_lock();
        let entry = queue
            .get(&key)
            .ok_or_else(|| KernelError::not_found("queued Turn input not found"))?;
        if entry.owner_user_id != owner_user_id
            || entry.version != expected_version
            || entry.status == AgentTurnInputQueueStatus::Executing
        {
            return Err(KernelError::conflict("queued Turn input removal conflict"));
        }
        queue
            .remove(&key)
            .ok_or_else(|| KernelError::conflict("queued Turn input removal conflict"))
    }

    fn clear_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<u64> {
        let mut queue = self.turn_input_queue.recovering_lock();
        let before = queue.len();
        queue.retain(|_, entry| {
            entry.tenant_id != tenant_id
                || entry.organization_id != organization_id
                || entry.session_id != session_id
                || entry.owner_user_id != owner_user_id
                || entry.status == AgentTurnInputQueueStatus::Executing
        });
        Ok(u64::try_from(before.saturating_sub(queue.len())).unwrap_or(u64::MAX))
    }

    fn purge_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<u64> {
        let mut queue = self.turn_input_queue.recovering_lock();
        let before = queue.len();
        queue.retain(|_, entry| {
            entry.tenant_id != tenant_id
                || entry.organization_id != organization_id
                || entry.session_id != session_id
                || entry.owner_user_id != owner_user_id
        });
        Ok(u64::try_from(before.saturating_sub(queue.len())).unwrap_or(u64::MAX))
    }

    fn reorder_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        entries: &[TurnInputQueueReorderEntry],
        requested_at: &str,
    ) -> KernelResult<Vec<AgentTurnInputQueueEntry>> {
        let mut queue = self.turn_input_queue.recovering_lock();
        let mutable_ids = queue
            .values()
            .filter(|entry| {
                entry.tenant_id == tenant_id
                    && entry.organization_id == organization_id
                    && entry.session_id == session_id
                    && entry.owner_user_id == owner_user_id
                    && entry.status != AgentTurnInputQueueStatus::Executing
            })
            .map(|entry| entry.queue_entry_id.clone())
            .collect::<HashSet<_>>();
        let requested_ids = entries
            .iter()
            .map(|entry| entry.queue_entry_id.clone())
            .collect::<HashSet<_>>();
        if mutable_ids != requested_ids {
            return Err(KernelError::conflict(
                "queued Turn input reorder set changed",
            ));
        }
        for entry in entries {
            let key = (
                tenant_id,
                organization_id,
                session_id.to_string(),
                entry.queue_entry_id.clone(),
            );
            let current = queue
                .get(&key)
                .ok_or_else(|| KernelError::conflict("queued Turn input reorder set changed"))?;
            if current.version != entry.expected_version {
                return Err(KernelError::conflict("queued Turn input version mismatch"));
            }
        }
        let executing_position = queue
            .values()
            .filter(|entry| {
                entry.tenant_id == tenant_id
                    && entry.organization_id == organization_id
                    && entry.session_id == session_id
                    && entry.owner_user_id == owner_user_id
                    && entry.status == AgentTurnInputQueueStatus::Executing
            })
            .map(|entry| entry.position)
            .max()
            .unwrap_or(0);
        let mut reordered = Vec::with_capacity(entries.len());
        for (index, requested) in entries.iter().enumerate() {
            let key = (
                tenant_id,
                organization_id,
                session_id.to_string(),
                requested.queue_entry_id.clone(),
            );
            let current = queue
                .get_mut(&key)
                .ok_or_else(|| KernelError::conflict("queued Turn input reorder set changed"))?;
            current.position = u64::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
                .and_then(|value| value.checked_mul(1024))
                .and_then(|value| value.checked_add(executing_position))
                .ok_or_else(|| KernelError::conflict("queued Turn input position overflow"))?;
            current.version = current.version.saturating_add(1);
            current.updated_at = requested_at.to_string();
            reordered.push(current.clone());
        }
        Ok(reordered)
    }

    fn claim_next_turn_input_queue_entry(
        &self,
        request: &TurnInputQueueClaimRequest,
    ) -> KernelResult<TurnInputQueueClaimOutcome> {
        let requested_at = parse_rfc3339_datetime(&request.requested_at, "requestedAt")?;
        let mut queue = self.turn_input_queue.recovering_lock();
        let turns = self.turns.recovering_read();
        let mut scoped_keys = queue
            .iter()
            .filter(|(_, entry)| {
                entry.tenant_id == request.tenant_id
                    && entry.organization_id == request.organization_id
                    && entry.session_id == request.session_id
                    && entry.owner_user_id == request.owner_user_id
            })
            .map(|(key, entry)| (key.clone(), entry.position, entry.id))
            .collect::<Vec<_>>();
        scoped_keys.sort_by_key(|(_, position, id)| (*position, *id));

        if let Some(executing_key) = scoped_keys
            .iter()
            .find(|(key, _, _)| {
                queue
                    .get(key)
                    .is_some_and(|entry| entry.status == AgentTurnInputQueueStatus::Executing)
            })
            .map(|(key, _, _)| key.clone())
        {
            let executing = queue
                .get(&executing_key)
                .cloned()
                .ok_or_else(|| KernelError::conflict("queued Turn input claim changed"))?;
            let matching_turn = turns.values().find(|turn| {
                turn.tenant_id == request.tenant_id
                    && turn.organization_id == request.organization_id
                    && turn.session_id == request.session_id
                    && turn.owner_user_id == request.owner_user_id
                    && turn.idempotency_key == executing.idempotency_key
            });
            match matching_turn.map(|turn| turn.status) {
                Some(AgentTurnStatus::Completed) => {
                    queue.remove(&executing_key);
                    scoped_keys.retain(|(key, _, _)| key != &executing_key);
                }
                Some(AgentTurnStatus::Failed | AgentTurnStatus::Cancelled) => {
                    let entry = queue
                        .get_mut(&executing_key)
                        .ok_or_else(|| KernelError::conflict("queued Turn input claim changed"))?;
                    entry.status = AgentTurnInputQueueStatus::Failed;
                    entry.claim_owner = None;
                    entry.claim_token_hash = None;
                    entry.claim_expires_at = None;
                    entry.error_code = Some("turn_terminal_failure".to_string());
                    entry.error_detail = None;
                    entry.failed_at = Some(request.requested_at.clone());
                    entry.updated_at = request.requested_at.clone();
                    entry.version = entry.version.saturating_add(1);
                    return Ok(TurnInputQueueClaimOutcome::Blocked(entry.clone()));
                }
                Some(AgentTurnStatus::Requested | AgentTurnStatus::Running) => {
                    return Ok(TurnInputQueueClaimOutcome::Busy(executing));
                }
                None => {
                    let expired = executing
                        .claim_expires_at
                        .as_deref()
                        .and_then(|value| parse_rfc3339_datetime(value, "claimExpiresAt").ok())
                        .is_none_or(|expires_at| expires_at <= requested_at);
                    if !expired {
                        return Ok(TurnInputQueueClaimOutcome::Busy(executing));
                    }
                    let entry = queue
                        .get_mut(&executing_key)
                        .ok_or_else(|| KernelError::conflict("queued Turn input claim changed"))?;
                    entry.status = AgentTurnInputQueueStatus::Queued;
                    entry.claim_owner = None;
                    entry.claim_token_hash = None;
                    entry.claim_expires_at = None;
                    entry.updated_at = request.requested_at.clone();
                    entry.version = entry.version.saturating_add(1);
                }
            }
        }

        if turns.values().any(|turn| {
            turn.tenant_id == request.tenant_id
                && turn.organization_id == request.organization_id
                && turn.session_id == request.session_id
                && turn.owner_user_id == request.owner_user_id
                && matches!(
                    turn.status,
                    AgentTurnStatus::Requested | AgentTurnStatus::Running
                )
        }) {
            return Ok(TurnInputQueueClaimOutcome::ActiveTurn);
        }

        let head_key = scoped_keys
            .iter()
            .filter_map(|(key, _, _)| queue.get(key).map(|entry| (key, entry)))
            .min_by_key(|(_, entry)| (entry.position, entry.id))
            .map(|(key, _)| key.clone());
        let Some(head_key) = head_key else {
            return Ok(TurnInputQueueClaimOutcome::Empty);
        };
        let head = queue
            .get_mut(&head_key)
            .ok_or_else(|| KernelError::conflict("queued Turn input claim changed"))?;
        if head.status == AgentTurnInputQueueStatus::Failed {
            return Ok(TurnInputQueueClaimOutcome::Blocked(head.clone()));
        }
        head.status = AgentTurnInputQueueStatus::Executing;
        head.claim_owner = Some(request.claim_owner.clone());
        head.claim_token_hash = Some(request.claim_token_hash.clone());
        head.claim_expires_at = Some(request.claim_expires_at.clone());
        head.claimed_at = Some(request.requested_at.clone());
        head.updated_at = request.requested_at.clone();
        head.fencing_token = head.fencing_token.saturating_add(1);
        head.version = head.version.saturating_add(1);
        Ok(TurnInputQueueClaimOutcome::Claimed(head.clone()))
    }

    fn fail_turn_input_queue_entry(
        &self,
        request: &TurnInputQueueFailureRequest,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        let key = (
            request.tenant_id,
            request.organization_id,
            request.session_id.clone(),
            request.queue_entry_id.clone(),
        );
        let mut queue = self.turn_input_queue.recovering_lock();
        let entry = queue
            .get_mut(&key)
            .ok_or_else(|| KernelError::not_found("queued Turn input not found"))?;
        if entry.owner_user_id != request.owner_user_id
            || entry.status != AgentTurnInputQueueStatus::Executing
            || entry.version != request.expected_version
            || entry.fencing_token != request.expected_fencing_token
            || entry.claim_token_hash.as_deref() != Some(request.claim_token_hash.as_str())
        {
            return Err(KernelError::conflict("queued Turn input claim mismatch"));
        }
        entry.status = AgentTurnInputQueueStatus::Failed;
        entry.claim_owner = None;
        entry.claim_token_hash = None;
        entry.claim_expires_at = None;
        entry.error_code = Some(request.error_code.clone());
        entry.error_detail = request.error_detail.clone();
        entry.failed_at = Some(request.requested_at.clone());
        entry.updated_at = request.requested_at.clone();
        entry.version = entry.version.saturating_add(1);
        Ok(entry.clone())
    }

    fn list_reconcilable_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTurnRecord>> {
        let mut turns = self
            .turns
            .recovering_read()
            .values()
            .filter(|turn| {
                matches!(
                    turn.status,
                    crate::agent_turn::AgentTurnStatus::Requested
                        | crate::agent_turn::AgentTurnStatus::Running
                ) && turn.updated_at.as_str() < stale_before
            })
            .cloned()
            .collect::<Vec<_>>();
        turns.sort_by(|left, right| {
            left.updated_at
                .cmp(&right.updated_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        turns.truncate(limit.min(200));
        Ok(turns)
    }

    fn append_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
        content: &str,
        updated_at: &str,
    ) -> KernelResult<()> {
        // The in-memory repository retains the checkpoint so interrupted-turn
        // promotion behaves the same as the durable adapter in dev/test.
        let mut turns = self.turns.recovering_write();
        let mut matched = false;
        for (key, turn) in turns.iter_mut() {
            if key.0 != tenant_id || key.1 != organization_id || turn.turn_id != turn_id {
                continue;
            }
            let running = matches!(
                turn.status,
                crate::agent_turn::AgentTurnStatus::Requested
                    | crate::agent_turn::AgentTurnStatus::Running
            );
            if !running {
                continue;
            }
            turn.updated_at = updated_at.to_string();
            matched = true;
            break;
        }
        drop(turns);
        if !matched {
            return Err(KernelError::not_found("turn not found or not running"));
        }
        self.turn_streaming_checkpoints.recovering_write().insert(
            (tenant_id, organization_id, turn_id.to_string()),
            content.to_string(),
        );
        Ok(())
    }

    fn clear_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<()> {
        self.turn_streaming_checkpoints.recovering_write().remove(&(
            tenant_id,
            organization_id,
            turn_id.to_string(),
        ));
        Ok(())
    }

    fn read_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<String>> {
        Ok(self
            .turn_streaming_checkpoints
            .recovering_read()
            .get(&(tenant_id, organization_id, turn_id.to_string()))
            .cloned())
    }

    fn append_interrupted_turn_output(
        &self,
        mut record: AgentSessionItemRecord,
    ) -> KernelResult<Option<AgentSessionItemRecord>> {
        if record.kind != AgentSessionItemKind::AssistantOutput
            || record.status != AgentSessionItemStatus::Completed
            || record.sequence != 0
            || record.content.is_some()
        {
            return Err(KernelError::validation(
                "interrupted turn output must be a completed, unsequenced assistant item without inline content",
            ));
        }
        let turn_id = record
            .turn_id
            .clone()
            .ok_or_else(|| KernelError::validation("interrupted turn output requires a turn"))?;
        let turn_primary_key = (record.tenant_id, record.organization_id, turn_id.clone());
        let session_primary_key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
        );

        // Lock order matches complete_turn: turns, sessions, session_index,
        // items, session_item_index.
        let checkpoint = {
            let turns = self.turns.recovering_read();
            let turn = turns
                .get(&turn_primary_key)
                .ok_or_else(|| KernelError::not_found("turn not found"))?;
            if turn.session_id != record.session_id
                || turn.owner_user_id != record.created_by
                || turn.turn_id != turn_id
            {
                return Err(KernelError::validation(
                    "interrupted turn output scope mismatch",
                ));
            }
            if !matches!(
                turn.status,
                AgentTurnStatus::Failed | AgentTurnStatus::Cancelled
            ) {
                // Not a promotable terminal state: nothing to do (idempotent).
                return Ok(None);
            }
            self.turn_streaming_checkpoints
                .recovering_read()
                .get(&(
                    record.tenant_id,
                    record.organization_id,
                    turn_id.to_string(),
                ))
                .cloned()
        };
        let Some(checkpoint) = checkpoint.filter(|content| !content.trim().is_empty()) else {
            return Ok(None);
        };

        // Idempotency: an interrupted turn is promoted at most once.
        {
            let items = self.items.recovering_read();
            let index = self.session_item_index.recovering_read();
            let scope_start = (
                record.tenant_id,
                record.organization_id,
                record.session_id.clone(),
                0,
                0,
            );
            let scope_end = (
                record.tenant_id,
                record.organization_id,
                record.session_id.clone(),
                u64::MAX,
                u64::MAX,
            );
            if index
                .range(scope_start..=scope_end)
                .filter_map(|(_, primary_key)| items.get(primary_key))
                .any(|item| {
                    item.turn_id.as_deref() == Some(turn_id.as_str())
                        && item.kind == AgentSessionItemKind::AssistantOutput
                })
            {
                return Ok(None);
            }
        }

        let mut sessions = self.sessions.recovering_write();
        let mut session_index = self.session_index.recovering_write();
        let mut items = self.items.recovering_write();
        let mut item_index = self.session_item_index.recovering_write();
        let existing_session = sessions
            .get(&session_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::not_found("active session not found"))?;
        if !existing_session.status.is_active()
            || existing_session.deleted_at.is_some()
            || existing_session.owner_user_id != record.created_by
        {
            return Err(KernelError::not_found("active session not found"));
        }
        record.sequence = existing_session.last_item_sequence.saturating_add(1);
        record.content = Some(checkpoint);
        let primary_key = session_item_primary_key(&record);
        if items.contains_key(&primary_key) {
            return Err(KernelError::conflict("session item already exists"));
        }
        let index_key = session_item_index_key(&record);
        if item_index.contains_key(&index_key) {
            return Err(KernelError::conflict("session item sequence conflict"));
        }
        let mut updated_session = existing_session.clone();
        updated_session.updated_by = record.created_by;
        updated_session.record_item(
            record.input_tokens,
            record.output_tokens,
            record.updated_at.clone(),
        );
        let previous_session_index_key = session_index_key(&existing_session);
        let next_session_index_key = session_index_key(&updated_session);
        items.insert(primary_key.clone(), record.clone());
        item_index.insert(index_key, primary_key);
        sessions.insert(session_primary_key.clone(), updated_session.clone());
        session_index.remove(&previous_session_index_key);
        session_index.insert(next_session_index_key, session_primary_key);
        drop(sessions);
        drop(session_index);
        drop(items);
        drop(item_index);
        self.turn_streaming_checkpoints.recovering_write().remove(&(
            record.tenant_id,
            record.organization_id,
            turn_id.to_string(),
        ));
        self.advance_session_activity(
            &updated_session,
            &record.updated_at,
            SessionActivitySource::Session,
        );
        Ok(Some(record))
    }

    fn insert_turn_request(
        &self,
        turn: AgentTurnRecord,
        mut request_item: AgentSessionItemRecord,
        drive_refs: Vec<AgentItemDriveRefRecord>,
    ) -> KernelResult<TurnRequestWriteOutcome> {
        let activity_turn = turn.clone();
        if turn.status != AgentTurnStatus::Requested
            || turn.version != 0
            || turn.response_item_id.is_some()
            || turn.request_item_id != request_item.item_id
            || turn.tenant_id != request_item.tenant_id
            || turn.organization_id != request_item.organization_id
            || turn.session_id != request_item.session_id
            || request_item.turn_id.as_deref() != Some(turn.turn_id.as_str())
        {
            return Err(KernelError::validation(
                "turn request and session item scope mismatch",
            ));
        }

        let primary_key = (turn.tenant_id, turn.organization_id, turn.turn_id.clone());
        let idempotency_key = (
            turn.tenant_id,
            turn.organization_id,
            turn.owner_user_id,
            turn.idempotency_key.clone(),
        );
        let session_primary_key = (
            turn.tenant_id,
            turn.organization_id,
            turn.session_id.clone(),
        );
        let mut pending_drive_refs = Vec::with_capacity(drive_refs.len());
        for record in drive_refs {
            if record.tenant_id != request_item.tenant_id
                || record.organization_id != request_item.organization_id
                || record.item_id != request_item.item_id
            {
                return Err(KernelError::validation(
                    "session item Drive reference scope mismatch",
                ));
            }
            let key = (
                record.tenant_id,
                record.organization_id,
                record.item_id.clone(),
                record.drive_node_id.clone(),
                record.resource_role.as_str().to_string(),
            );
            if pending_drive_refs
                .iter()
                .any(|(candidate, _)| candidate == &key)
            {
                return Err(KernelError::conflict(
                    "duplicate session item Drive reference",
                ));
            }
            pending_drive_refs.push((key, record));
        }

        let mut turns = self.turns.recovering_write();
        let mut turn_idempotency = self.turn_idempotency.recovering_write();
        let mut turn_index = self.turn_index.recovering_write();
        let mut sessions = self.sessions.recovering_write();
        let mut session_index = self.session_index.recovering_write();
        let mut items = self.items.recovering_write();
        let mut session_item_index = self.session_item_index.recovering_write();
        let mut item_drive_refs = self.item_drive_refs.recovering_write();

        if let Some(existing) = turns.get(&primary_key) {
            if existing.idempotency_key == turn.idempotency_key {
                return Ok(TurnRequestWriteOutcome::Existing(Box::new(
                    existing.clone(),
                )));
            }
            return Err(KernelError::conflict("turn idempotency conflict"));
        }
        if let Some(existing_primary_key) = turn_idempotency.get(&idempotency_key) {
            let existing =
                turns
                    .get(existing_primary_key)
                    .cloned()
                    .ok_or_else(|| KernelError::Internal {
                        message: "turn idempotency index references a missing turn".to_string(),
                    })?;
            return Ok(TurnRequestWriteOutcome::Existing(Box::new(existing)));
        }
        if turns.values().any(|existing| {
            existing.tenant_id == turn.tenant_id
                && existing.organization_id == turn.organization_id
                && existing.session_id == turn.session_id
                && matches!(
                    existing.status,
                    AgentTurnStatus::Requested | AgentTurnStatus::Running
                )
        }) {
            return Err(KernelError::conflict(
                "another Turn is already active for this session",
            ));
        }
        let existing_session = sessions
            .get(&session_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::not_found("active session not found"))?;
        if !existing_session.status.is_active() || existing_session.deleted_at.is_some() {
            return Err(KernelError::not_found("active session not found"));
        }
        let request_primary_key = session_item_primary_key(&request_item);
        if items.contains_key(&request_primary_key) {
            return Err(KernelError::conflict("request item already exists"));
        }
        if pending_drive_refs
            .iter()
            .any(|(key, _)| item_drive_refs.contains_key(key))
        {
            return Err(KernelError::conflict(
                "duplicate session item Drive reference",
            ));
        }

        request_item.sequence = existing_session.last_item_sequence.saturating_add(1);
        let request_index_key = session_item_index_key(&request_item);
        if session_item_index.contains_key(&request_index_key) {
            return Err(KernelError::conflict("session item sequence conflict"));
        }
        let mut updated_session = existing_session.clone();
        updated_session.updated_by = request_item.created_by;
        updated_session.record_item(
            request_item.input_tokens,
            request_item.output_tokens,
            request_item.updated_at.clone(),
        );

        let previous_session_index_key = session_index_key(&existing_session);
        let next_session_index_key = session_index_key(&updated_session);
        turns.insert(primary_key.clone(), turn);
        turn_index.insert(
            (
                activity_turn.tenant_id,
                activity_turn.organization_id,
                activity_turn.session_id.clone(),
                Reverse(activity_turn.created_at.clone()),
                Reverse(activity_turn.id),
            ),
            primary_key.clone(),
        );
        turn_idempotency.insert(idempotency_key, primary_key);
        items.insert(request_primary_key.clone(), request_item.clone());
        session_item_index.insert(request_index_key, request_primary_key);
        sessions.insert(session_primary_key.clone(), updated_session.clone());
        session_index.remove(&previous_session_index_key);
        session_index.insert(next_session_index_key, session_primary_key);
        for (key, record) in pending_drive_refs {
            item_drive_refs.insert(key, record);
        }
        self.advance_session_activity(
            &updated_session,
            &activity_turn.updated_at,
            SessionActivitySource::Turn,
        );

        Ok(TurnRequestWriteOutcome::Inserted {
            session: Box::new(updated_session),
            request_item: Box::new(request_item),
        })
    }

    fn update_turn_state(
        &self,
        turn: AgentTurnRecord,
        expected_version: u64,
    ) -> KernelResult<AgentTurnRecord> {
        let primary_key = (turn.tenant_id, turn.organization_id, turn.turn_id.clone());
        let mut turns = self.turns.recovering_write();
        let existing = turns
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("turn not found"))?;
        if existing.version != expected_version
            || turn.version != expected_version.saturating_add(1)
        {
            return Err(KernelError::conflict("turn version mismatch"));
        }
        if existing.idempotency_key != turn.idempotency_key
            || existing.payload_hash != turn.payload_hash
            || existing.session_id != turn.session_id
            || existing.agent_id != turn.agent_id
            || existing.owner_user_id != turn.owner_user_id
        {
            return Err(KernelError::validation("turn immutable scope mismatch"));
        }
        turns.insert(primary_key, turn.clone());
        if let Some(session) = self
            .sessions
            .recovering_read()
            .get(&(
                turn.tenant_id,
                turn.organization_id,
                turn.session_id.clone(),
            ))
            .cloned()
        {
            self.advance_session_activity(&session, &turn.updated_at, SessionActivitySource::Turn);
        }
        Ok(turn)
    }

    fn complete_turn(
        &self,
        turn: AgentTurnRecord,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        mut completed_items: Vec<AgentSessionItemRecord>,
    ) -> KernelResult<(AgentSessionRecord, Vec<AgentSessionItemRecord>)> {
        validate_completed_turn_items(&turn, expected_turn_version, &completed_items)?;
        let turn_primary_key = (turn.tenant_id, turn.organization_id, turn.turn_id.clone());
        let session_primary_key = (
            turn.tenant_id,
            turn.organization_id,
            turn.session_id.clone(),
        );
        let mut turns = self.turns.recovering_write();
        let mut sessions = self.sessions.recovering_write();
        let mut session_index = self.session_index.recovering_write();
        let mut items = self.items.recovering_write();
        let mut session_item_index = self.session_item_index.recovering_write();

        let existing_turn = turns
            .get(&turn_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::not_found("turn not found"))?;
        if existing_turn.version != expected_turn_version
            || existing_turn.fencing_token != expected_fencing_token
            || existing_turn.lease_token != expected_lease_token
            || existing_turn.payload_hash != turn.payload_hash
            || existing_turn.idempotency_key != turn.idempotency_key
            || existing_turn.session_id != turn.session_id
            || existing_turn.agent_id != turn.agent_id
            || existing_turn.owner_user_id != turn.owner_user_id
            || existing_turn.status != AgentTurnStatus::Running
        {
            return Err(KernelError::conflict("turn completion conflict"));
        }
        let existing_session = sessions
            .get(&session_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        if existing_session.deleted_at.is_some() {
            return Err(KernelError::not_found("session not found"));
        }
        let mut updated_session = existing_session.clone();
        let mut pending_items = Vec::with_capacity(completed_items.len());
        let mut pending_primary_keys = HashSet::with_capacity(completed_items.len());
        let mut pending_index_keys = HashSet::with_capacity(completed_items.len());
        for (index, item) in completed_items.iter_mut().enumerate() {
            let sequence_offset = u64::try_from(index)
                .map_err(|_| KernelError::conflict("session item sequence overflow"))?
                .checked_add(1)
                .ok_or_else(|| KernelError::conflict("session item sequence overflow"))?;
            item.sequence = existing_session
                .last_item_sequence
                .checked_add(sequence_offset)
                .ok_or_else(|| KernelError::conflict("session item sequence overflow"))?;
            let primary_key = session_item_primary_key(item);
            if items.contains_key(&primary_key) || !pending_primary_keys.insert(primary_key.clone())
            {
                return Err(KernelError::conflict("completed turn item already exists"));
            }
            let index_key = session_item_index_key(item);
            if session_item_index.contains_key(&index_key)
                || !pending_index_keys.insert(index_key.clone())
            {
                return Err(KernelError::conflict("session item sequence conflict"));
            }
            updated_session.updated_by = item.created_by;
            updated_session.record_item(
                item.input_tokens,
                item.output_tokens,
                item.updated_at.clone(),
            );
            pending_items.push((primary_key, index_key));
        }

        let previous_session_index_key = session_index_key(&existing_session);
        let next_session_index_key = session_index_key(&updated_session);
        for ((primary_key, index_key), item) in pending_items
            .into_iter()
            .zip(completed_items.iter().cloned())
        {
            items.insert(primary_key.clone(), item);
            session_item_index.insert(index_key, primary_key);
        }
        sessions.insert(session_primary_key.clone(), updated_session.clone());
        session_index.remove(&previous_session_index_key);
        session_index.insert(next_session_index_key, session_primary_key);
        let activity_at = turn.updated_at.clone();
        turns.insert(turn_primary_key, turn);
        self.advance_session_activity(&updated_session, &activity_at, SessionActivitySource::Turn);

        Ok((updated_session, completed_items))
    }

    fn list_item_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        let mut records = self
            .item_drive_refs
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.item_id == item_id
                    && record.deleted_at.is_none()
                    && record.status == 0
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by_key(|record| (record.sort_order, record.id));
        Ok(records)
    }

    fn list_item_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        let item_ids = item_ids.iter().map(String::as_str).collect::<HashSet<_>>();
        let mut records = self
            .item_drive_refs
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && item_ids.contains(record.item_id.as_str())
                    && record.deleted_at.is_none()
                    && record.status == 0
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.item_id
                .cmp(&right.item_id)
                .then(left.sort_order.cmp(&right.sort_order))
                .then(left.id.cmp(&right.id))
        });
        Ok(records)
    }

    fn insert_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        let activity_record = record.clone();
        let primary_key = interaction_primary_key(&record);
        let mut interactions = self.interactions.recovering_write();
        if interactions.contains_key(&primary_key) {
            return Err(KernelError::conflict(format!(
                "interaction {} already exists for session {}",
                record.interaction_id, record.session_id
            )));
        }
        let index_key = interaction_index_key(&record);
        interactions.insert(primary_key.clone(), record);
        self.interaction_index
            .recovering_write()
            .insert(index_key, primary_key.clone());
        if activity_record.status.is_pending() {
            self.pending_interaction_index
                .recovering_write()
                .insert(pending_interaction_index_key(&activity_record), primary_key);
        }
        if let Some(session) = self
            .sessions
            .recovering_read()
            .get(&(
                activity_record.tenant_id,
                activity_record.organization_id,
                activity_record.session_id.clone(),
            ))
            .cloned()
        {
            self.advance_session_activity(
                &session,
                &activity_record.updated_at,
                SessionActivitySource::Interaction,
            );
        }
        Ok(())
    }

    fn update_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        let activity_record = record.clone();
        let primary_key = interaction_primary_key(&record);
        let mut interactions = self.interactions.recovering_write();
        let existing = interactions.get(&primary_key).ok_or_else(|| {
            KernelError::validation(format!(
                "interaction {} not found for session {}",
                record.interaction_id, record.session_id
            ))
        })?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "interaction version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = interaction_index_key(existing);
        let previous_pending_index_key = existing
            .status
            .is_pending()
            .then(|| pending_interaction_index_key(existing));
        let next_index_key = interaction_index_key(&record);
        let next_pending_index_key = record
            .status
            .is_pending()
            .then(|| pending_interaction_index_key(&record));
        interactions.insert(primary_key.clone(), record);
        let mut index = self.interaction_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key.clone());
        let mut pending_index = self.pending_interaction_index.recovering_write();
        if let Some(previous_pending_index_key) = previous_pending_index_key {
            pending_index.remove(&previous_pending_index_key);
        }
        if let Some(next_pending_index_key) = next_pending_index_key {
            pending_index.insert(next_pending_index_key, primary_key);
        }
        if let Some(session) = self
            .sessions
            .recovering_read()
            .get(&(
                activity_record.tenant_id,
                activity_record.organization_id,
                activity_record.session_id.clone(),
            ))
            .cloned()
        {
            self.advance_session_activity(
                &session,
                &activity_record.updated_at,
                SessionActivitySource::Interaction,
            );
        }
        Ok(())
    }

    fn get_interaction(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRecord>> {
        Ok(self
            .interactions
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                interaction_id.to_string(),
            ))
            .cloned())
    }

    /// InMemory fallback for contract tests: filters the session's
    /// Interaction set in memory. Production postgres-sync uses SQL keyset
    /// pagination (PAGINATION_SPEC).
    fn list_interactions(
        &self,
        query: &InteractionListQuery,
    ) -> KernelResult<Vec<AgentInteractionRecord>> {
        let interactions = self.interactions.recovering_read();
        let index = self.interaction_index.recovering_read();
        let mut records = index
            .iter()
            .filter(|((tenant_id, organization_id, session_id, _, _), _)| {
                *tenant_id == query.tenant_id
                    && *organization_id == query.organization_id
                    && session_id == &query.session_id
            })
            .filter_map(|(_, primary_key)| interactions.get(primary_key))
            .filter(|record| interaction_matches_list_query(record, query))
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        let store_limit = query.pagination.page_size.saturating_add(1);
        Ok(records
            .into_iter()
            .filter(|record| {
                query.cursor.as_ref().is_none_or(|cursor| {
                    record.created_at < cursor.created_at
                        || (record.created_at == cursor.created_at
                            && record.id < cursor.internal_id)
                })
            })
            .take(store_limit)
            .collect())
    }

    fn count_interactions(&self, query: &InteractionListQuery) -> KernelResult<u64> {
        let interactions = self.interactions.recovering_read();
        let index = self.interaction_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, organization_id, session_id, _, _), _)| {
                    *tenant_id == query.tenant_id
                        && *organization_id == query.organization_id
                        && session_id == &query.session_id
                })
                .filter_map(|(_, primary_key)| interactions.get(primary_key))
                .filter(|record| interaction_matches_list_query(record, query)),
        ))
    }

    fn insert_runtime_execution(&self, record: AgentRuntimeExecutionRecord) -> KernelResult<()> {
        let primary_key = runtime_execution_primary_key(&record);
        let mut executions = self.runtime_executions.recovering_write();
        if executions.contains_key(&primary_key) {
            return Err(KernelError::conflict(format!(
                "runtime execution {} already exists for agent {}",
                record.execution_id, record.agent_id
            )));
        }
        executions.insert(primary_key, record);
        Ok(())
    }

    fn update_runtime_execution(&self, record: AgentRuntimeExecutionRecord) -> KernelResult<()> {
        let primary_key = runtime_execution_primary_key(&record);
        let mut executions = self.runtime_executions.recovering_write();
        if !executions.contains_key(&primary_key) {
            return Err(KernelError::validation(format!(
                "runtime execution {} not found for agent {}",
                record.execution_id, record.agent_id
            )));
        }
        executions.insert(primary_key, record);
        Ok(())
    }

    fn get_runtime_execution(
        &self,
        tenant_id: u64,
        agent_id: &str,
        execution_id: &str,
    ) -> KernelResult<Option<AgentRuntimeExecutionRecord>> {
        Ok(self
            .runtime_executions
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string(), execution_id.to_string()))
            .cloned())
    }

    fn list_runtime_executions(
        &self,
        query: &RuntimeExecutionListQuery,
    ) -> KernelResult<Vec<AgentRuntimeExecutionRecord>> {
        let mut records: Vec<AgentRuntimeExecutionRecord> = self
            .runtime_executions
            .recovering_read()
            .iter()
            .filter(|((tenant_id, agent_id, _), _)| {
                *tenant_id == query.tenant_id && agent_id == &query.agent_id
            })
            .filter(|(_, record)| {
                query
                    .status
                    .as_deref()
                    .is_none_or(|status| record.status.as_str() == status)
            })
            .filter(|(_, record)| match query.cursor.as_ref() {
                // Keyset continuation: strictly after the cursor position in
                // the (requestedAt DESC, executionId DESC) ordering.
                Some(cursor) => {
                    (&record.requested_at, &record.execution_id)
                        < (&cursor.requested_at, &cursor.execution_id)
                }
                None => true,
            })
            .map(|(_, record)| record.clone())
            .collect();
        records.sort_by(|left, right| {
            (&right.requested_at, &right.execution_id)
                .cmp(&(&left.requested_at, &left.execution_id))
        });
        Ok(records)
    }

    fn list_stale_runtime_executions(
        &self,
        tenant_id: u64,
        updated_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentRuntimeExecutionRecord>> {
        let mut records: Vec<AgentRuntimeExecutionRecord> = self
            .runtime_executions
            .recovering_read()
            .iter()
            .filter(|((record_tenant_id, _, _), record)| {
                *record_tenant_id == tenant_id
                    && matches!(
                        record.status,
                        AgentRuntimeExecutionStatus::Queued | AgentRuntimeExecutionStatus::Running
                    )
                    && record.completed_at.as_str() < updated_before
            })
            .map(|(_, record)| record.clone())
            .collect();
        records.sort_by(|left, right| left.completed_at.cmp(&right.completed_at));
        records.truncate(limit.max(1));
        Ok(records)
    }

    fn summarize_usage(
        &self,
        query: &crate::usage::UsageSummaryQuery,
    ) -> KernelResult<crate::usage::AgentUsageSummary> {
        let turns = self.turns.recovering_read();
        let mut summary = crate::usage::AgentUsageSummary {
            turn_count: 0,
            session_count: 0,
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
        };
        let mut sessions = std::collections::BTreeSet::new();
        for turn in turns.values() {
            if !usage_turn_matches(turn, query) {
                continue;
            }
            summary.turn_count = summary.turn_count.saturating_add(1);
            summary.input_tokens = summary.input_tokens.saturating_add(turn.input_tokens);
            summary.output_tokens = summary.output_tokens.saturating_add(turn.output_tokens);
            summary.cached_tokens = summary.cached_tokens.saturating_add(turn.cached_tokens);
            sessions.insert(turn.session_id.clone());
        }
        summary.session_count = sessions.len() as u64;
        Ok(summary)
    }

    fn list_usage_records(
        &self,
        query: &crate::usage::UsageRecordListQuery,
    ) -> KernelResult<Vec<crate::usage::AgentUsageRecord>> {
        let mut records = self
            .turns
            .recovering_read()
            .values()
            .filter(|turn| usage_record_turn_matches(turn, query))
            .map(|turn| usage_record_from_turn(turn))
            .collect::<Vec<_>>();
        records.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        let store_limit = query.pagination.page_size.saturating_add(1);
        records.retain(|record| {
            query.cursor.as_ref().is_none_or(|cursor| {
                record.created_at < cursor.created_at
                    || (record.created_at == cursor.created_at
                        && record.internal_id < cursor.internal_id)
            })
        });
        records.truncate(store_limit);
        Ok(records)
    }

    fn insert_agent_version(&self, record: AgentVersionRecord) -> KernelResult<AgentVersionRecord> {
        let primary_key = agent_version_primary_key(&record);
        let mut versions = self.agent_versions.recovering_write();
        if versions.contains_key(&primary_key) {
            return Err(KernelError::conflict("agent version already exists"));
        }
        if versions.values().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.organization_id == record.organization_id
                && existing.agent_id == record.agent_id
                && existing.version_number >= record.version_number
        }) {
            return Err(KernelError::conflict(
                "agent version number must increase monotonically",
            ));
        }
        versions.insert(primary_key, record.clone());
        Ok(record)
    }

    fn get_agent_version(
        &self,
        tenant_id: u64,
        organization_id: u64,
        agent_id: &str,
        version_id: &str,
    ) -> KernelResult<Option<AgentVersionRecord>> {
        Ok(self
            .agent_versions
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                agent_id.to_string(),
                version_id.to_string(),
            ))
            .cloned())
    }

    fn list_agent_versions(
        &self,
        query: &AgentVersionListQuery,
    ) -> KernelResult<Vec<AgentVersionRecord>> {
        let mut records = self
            .agent_versions
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.agent_id == query.agent_id
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .version_number
                .cmp(&left.version_number)
                .then_with(|| right.id.cmp(&left.id))
        });
        let store_limit = query.pagination.page_size.saturating_add(1);
        records.retain(|record| {
            query.cursor.as_ref().is_none_or(|cursor| {
                // The version history orders by the monotonic version_number;
                // the cursor encodes the last seen version number in its
                // internal_id slot.
                record.version_number < cursor.internal_id
            })
        });
        records.truncate(store_limit);
        Ok(records)
    }

    fn activate_agent_version(
        &self,
        tenant_id: u64,
        organization_id: u64,
        agent_id: &str,
        version_id: &str,
        activated_at: &str,
    ) -> KernelResult<Option<AgentVersionRecord>> {
        let primary_key = (
            tenant_id,
            organization_id,
            agent_id.to_string(),
            version_id.to_string(),
        );
        let mut versions = self.agent_versions.recovering_write();
        if !versions.contains_key(&primary_key) {
            return Ok(None);
        }
        for (key, record) in versions.iter_mut() {
            if key.0 == tenant_id && key.1 == organization_id && key.2 == agent_id {
                record.activated_at = None;
            }
        }
        let record = versions
            .get_mut(&primary_key)
            .expect("version existence checked above");
        record.activated_at = Some(activated_at.to_string());
        Ok(Some(record.clone()))
    }

    fn insert_webhook_subscription(
        &self,
        record: AgentWebhookRecord,
    ) -> KernelResult<AgentWebhookRecord> {
        let primary_key = (
            record.tenant_id,
            record.organization_id,
            record.webhook_id.clone(),
        );
        let mut subscriptions = self.webhook_subscriptions.recovering_write();
        if subscriptions.contains_key(&primary_key) {
            return Err(KernelError::conflict("webhook subscription already exists"));
        }
        subscriptions.insert(primary_key, record.clone());
        Ok(record)
    }

    fn get_webhook_subscription(
        &self,
        tenant_id: u64,
        organization_id: u64,
        webhook_id: &str,
    ) -> KernelResult<Option<AgentWebhookRecord>> {
        Ok(self
            .webhook_subscriptions
            .recovering_read()
            .get(&(tenant_id, organization_id, webhook_id.to_string()))
            .cloned())
    }

    fn list_webhook_subscriptions(
        &self,
        query: &WebhookSubscriptionListQuery,
    ) -> KernelResult<Vec<AgentWebhookRecord>> {
        let mut records = self
            .webhook_subscriptions
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| left.webhook_id.cmp(&right.webhook_id))
        });
        let start = query.pagination.offset.min(records.len());
        let end = start
            .saturating_add(query.pagination.page_size)
            .min(records.len());
        Ok(records[start..end].to_vec())
    }

    fn delete_webhook_subscription(
        &self,
        tenant_id: u64,
        organization_id: u64,
        webhook_id: &str,
    ) -> KernelResult<Option<AgentWebhookRecord>> {
        let mut subscriptions = self.webhook_subscriptions.recovering_write();
        Ok(subscriptions.remove(&(tenant_id, organization_id, webhook_id.to_string())))
    }

    fn insert_webhook_delivery(
        &self,
        record: AgentWebhookDeliveryRecord,
    ) -> KernelResult<AgentWebhookDeliveryRecord> {
        let primary_key = (
            record.tenant_id,
            record.organization_id,
            record.webhook_id.clone(),
            record.delivery_id.clone(),
        );
        let mut deliveries = self.webhook_deliveries.recovering_write();
        if deliveries.contains_key(&primary_key) {
            return Err(KernelError::conflict("webhook delivery already exists"));
        }
        deliveries.insert(primary_key, record.clone());
        Ok(record)
    }

    fn complete_webhook_delivery(
        &self,
        tenant_id: u64,
        organization_id: u64,
        webhook_id: &str,
        delivery_id: &str,
        status: &str,
        response_code: Option<i32>,
        error_detail: Option<String>,
        completed_at: &str,
    ) -> KernelResult<AgentWebhookDeliveryRecord> {
        let mut deliveries = self.webhook_deliveries.recovering_write();
        let record = deliveries
            .get_mut(&(
                tenant_id,
                organization_id,
                webhook_id.to_string(),
                delivery_id.to_string(),
            ))
            .ok_or_else(|| {
                KernelError::not_found(format!("webhook delivery not found: {delivery_id}"))
            })?;
        record.status = status.to_string();
        record.response_code = response_code;
        record.error_detail = error_detail;
        record.completed_at = Some(completed_at.to_string());
        Ok(record.clone())
    }

    fn insert_task(&self, record: AgentTaskRecord) -> KernelResult<()> {
        let primary_key = task_primary_key(&record);
        let mut tasks = self.tasks.recovering_write();
        if tasks.contains_key(&primary_key) {
            return Err(KernelError::conflict("task already exists"));
        }
        let index_key = task_index_key(&record);
        tasks.insert(primary_key.clone(), record);
        self.task_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_task(&self, record: AgentTaskRecord) -> KernelResult<()> {
        let primary_key = task_primary_key(&record);
        let mut tasks = self.tasks.recovering_write();
        let existing = tasks
            .get(&primary_key)
            .ok_or_else(|| KernelError::not_found("task not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "task version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = task_index_key(existing);
        let next_index_key = task_index_key(&record);
        tasks.insert(primary_key.clone(), record);
        let mut index = self.task_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_task(
        &self,
        tenant_id: u64,
        organization_id: u64,
        task_id: &str,
    ) -> KernelResult<Option<AgentTaskRecord>> {
        Ok(self
            .tasks
            .recovering_read()
            .get(&(tenant_id, organization_id, task_id.to_string()))
            .cloned())
    }

    fn list_tasks(&self, query: &TaskListQuery) -> KernelResult<Vec<AgentTaskRecord>> {
        let tasks = self.tasks.recovering_read();
        let index = self.task_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, organization_id, _, _), _)| {
                *tenant_id == query.tenant_id && *organization_id == query.organization_id
            })
            .filter_map(|(_, primary_key)| tasks.get(primary_key))
            .filter(|record| task_matches_list_query(record, query))
            .filter(|record| {
                query.cursor.as_ref().is_none_or(|cursor| {
                    record.updated_at < cursor.updated_at
                        || (record.updated_at == cursor.updated_at
                            && record.id < cursor.task_internal_id)
                })
            })
            .take(query.store_limit())
            .cloned();
        Ok(iter.collect())
    }
}

impl TaskSchedulerRepository for InMemoryAgentRepository {
    fn transition_task(
        &self,
        task: AgentTaskRecord,
        cancellation_reason: &str,
    ) -> KernelResult<TaskTransitionResult> {
        if cancellation_reason.trim().is_empty() || cancellation_reason.len() > 128 {
            return Err(KernelError::validation(
                "task transition cancellation reason is invalid",
            ));
        }
        let key = (task.tenant_id, task.organization_id, task.task_id.clone());
        let mut tasks = self.tasks.recovering_write();
        let existing = tasks
            .get(&key)
            .ok_or_else(|| KernelError::not_found("task not found"))?;
        if task.version != existing.version.saturating_add(1)
            || task.generation != existing.generation.saturating_add(1)
        {
            return Err(KernelError::conflict("task version or generation mismatch"));
        }
        let previous_index_key = task_index_key(existing);
        let mut runs = self.task_runs.recovering_write();
        let mut cancelled_pending_run_count = 0u64;
        for run in runs.values_mut().filter(|run| {
            run.tenant_id == task.tenant_id
                && run.organization_id == task.organization_id
                && run.task_id == task.task_id
                && run.status == AgentTaskRunStatus::Pending
                && run.schedule_generation < task.generation
        }) {
            run.status = AgentTaskRunStatus::Cancelled;
            run.failure_class = Some("task_generation_changed".to_string());
            run.error_code = Some(cancellation_reason.to_string());
            run.finished_at = Some(task.updated_at.clone());
            run.cancelled_at = Some(task.updated_at.clone());
            run.updated_at = task.updated_at.clone();
            run.version = run.version.saturating_add(1);
            cancelled_pending_run_count = cancelled_pending_run_count.saturating_add(1);
        }
        tasks.insert(key.clone(), task.clone());
        let mut index = self.task_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(task_index_key(&task), key);
        Ok(TaskTransitionResult {
            task,
            cancelled_pending_run_count,
        })
    }

    fn create_manual_task_run(
        &self,
        task: &AgentTaskRecord,
        idempotency_key: &str,
        requested_at: &str,
    ) -> KernelResult<AgentTaskRunRecord> {
        if is_blank(Some(idempotency_key)) || idempotency_key.len() > 256 {
            return Err(KernelError::validation("idempotencyKey is invalid"));
        }
        parse_datetime(requested_at, None)
            .ok_or_else(|| KernelError::validation("requestedAt is invalid"))?;
        let mut runs = self.task_runs.recovering_write();
        if let Some(existing) = runs.values().find(|run| {
            run.tenant_id == task.tenant_id
                && run.organization_id == task.organization_id
                && run.owner_user_id == task.owner_user_id
                && run.idempotency_key == idempotency_key
        }) {
            if existing.task_id == task.task_id && existing.schedule_generation == task.generation {
                return Ok(existing.clone());
            }
            return Err(KernelError::conflict("idempotency key payload mismatch"));
        }
        let id = self.id_generator.next_id()?;
        let run_id = format!("{ID_PREFIX_RUN}{id}");
        validate_standard_id(&run_id, "runId", Some(ID_PREFIX_RUN))?;
        let run = AgentTaskRunRecord {
            id,
            run_id: run_id.clone(),
            tenant_id: task.tenant_id,
            organization_id: task.organization_id,
            task_id: task.task_id.clone(),
            session_id: task.session_id.clone(),
            agent_id: task.agent_id.clone(),
            owner_user_id: task.owner_user_id,
            trigger_kind: AgentTaskTriggerKind::Manual,
            schedule_generation: task.generation,
            scheduled_for: requested_at.to_string(),
            retry_of_run_id: None,
            priority: task.priority,
            status: AgentTaskRunStatus::Pending,
            idempotency_key: idempotency_key.to_string(),
            payload_hash: task_run_payload_hash(
                &task.task_id,
                &task.session_id,
                task.generation,
                idempotency_key,
                &task.prompt,
            )?,
            turn_id: Some(format!("{ID_PREFIX_TURN}{}", self.id_generator.next_id()?)),
            attempt_count: 0,
            max_attempts: task.max_attempts,
            available_at: requested_at.to_string(),
            lease_owner: None,
            lease_token_hash: None,
            lease_expires_at: None,
            fencing_token: 0,
            timeout_at: None,
            failure_class: None,
            error_code: None,
            error_detail: None,
            version: 0,
            created_at: requested_at.to_string(),
            updated_at: requested_at.to_string(),
            claimed_at: None,
            started_at: None,
            finished_at: None,
            cancelled_at: None,
        };
        runs.insert(
            (run.tenant_id, run.organization_id, run.run_id.clone()),
            run.clone(),
        );
        Ok(run)
    }

    fn create_business_retry_task_run(
        &self,
        task: &AgentTaskRecord,
        retry_of: &AgentTaskRunRecord,
        idempotency_key: &str,
        requested_at: &str,
    ) -> KernelResult<AgentTaskRunRecord> {
        if is_blank(Some(idempotency_key)) || idempotency_key.len() > 256 {
            return Err(KernelError::validation("idempotencyKey is invalid"));
        }
        parse_datetime(requested_at, None)
            .ok_or_else(|| KernelError::validation("requestedAt is invalid"))?;
        if retry_of.tenant_id != task.tenant_id
            || retry_of.organization_id != task.organization_id
            || retry_of.task_id != task.task_id
        {
            return Err(KernelError::validation("retry source Run scope is invalid"));
        }
        let mut runs = self.task_runs.recovering_write();
        if let Some(existing) = runs.values().find(|run| {
            run.tenant_id == task.tenant_id
                && run.organization_id == task.organization_id
                && run.owner_user_id == task.owner_user_id
                && run.idempotency_key == idempotency_key
        }) {
            if existing.trigger_kind == AgentTaskTriggerKind::BusinessRetry
                && existing.retry_of_run_id.as_deref() == Some(retry_of.run_id.as_str())
                && existing.schedule_generation == task.generation
            {
                return Ok(existing.clone());
            }
            return Err(KernelError::conflict("idempotency key payload mismatch"));
        }
        let id = self.id_generator.next_id()?;
        let run_id = format!("{ID_PREFIX_RUN}{id}");
        validate_standard_id(&run_id, "runId", Some(ID_PREFIX_RUN))?;
        let run = AgentTaskRunRecord {
            id,
            run_id: run_id.clone(),
            tenant_id: task.tenant_id,
            organization_id: task.organization_id,
            task_id: task.task_id.clone(),
            session_id: task.session_id.clone(),
            agent_id: task.agent_id.clone(),
            owner_user_id: task.owner_user_id,
            trigger_kind: AgentTaskTriggerKind::BusinessRetry,
            schedule_generation: task.generation,
            scheduled_for: requested_at.to_string(),
            retry_of_run_id: Some(retry_of.run_id.clone()),
            priority: task.priority,
            status: AgentTaskRunStatus::Pending,
            idempotency_key: idempotency_key.to_string(),
            payload_hash: task_run_payload_hash(
                &task.task_id,
                &task.session_id,
                task.generation,
                idempotency_key,
                &task.prompt,
            )?,
            turn_id: Some(format!("{ID_PREFIX_TURN}{}", self.id_generator.next_id()?)),
            attempt_count: 0,
            max_attempts: task.max_attempts,
            available_at: requested_at.to_string(),
            lease_owner: None,
            lease_token_hash: None,
            lease_expires_at: None,
            fencing_token: 0,
            timeout_at: None,
            failure_class: None,
            error_code: None,
            error_detail: None,
            version: 0,
            created_at: requested_at.to_string(),
            updated_at: requested_at.to_string(),
            claimed_at: None,
            started_at: None,
            finished_at: None,
            cancelled_at: None,
        };
        runs.insert(
            (run.tenant_id, run.organization_id, run.run_id.clone()),
            run.clone(),
        );
        Ok(run)
    }

    fn materialize_due_tasks(
        &self,
        request: &MaterializeDueTasksRequest,
    ) -> KernelResult<Vec<AgentTaskRunRecord>> {
        let now = parse_datetime(&request.now, None)
            .ok_or_else(|| KernelError::validation("now must be an RFC 3339 instant"))?;
        let mut tasks = self.tasks.recovering_write();
        let mut task_index = self.task_index.recovering_write();
        let mut runs = self.task_runs.recovering_write();
        let mut due_keys = tasks
            .iter()
            .filter(|(_, task)| {
                task.status == AgentTaskStatus::Active
                    && task
                        .next_fire_at
                        .as_deref()
                        .and_then(|value| parse_datetime(value, None))
                        .is_some_and(|value| value <= now)
            })
            .map(|(key, task)| {
                (
                    task.next_fire_at.clone().unwrap_or_default(),
                    Reverse(task.priority),
                    task.id,
                    key.clone(),
                )
            })
            .collect::<Vec<_>>();
        due_keys.sort();
        due_keys.truncate(request.limit);

        let mut materialized = Vec::new();
        for (_, _, _, key) in due_keys {
            let task = tasks
                .get_mut(&key)
                .ok_or_else(|| KernelError::not_found("task not found"))?;
            let previous_index_key = task_index_key(task);
            let active_count = runs
                .values()
                .filter(|run| {
                    run.tenant_id == task.tenant_id
                        && run.organization_id == task.organization_id
                        && run.task_id == task.task_id
                        && matches!(
                            run.status,
                            AgentTaskRunStatus::Claimed
                                | AgentTaskRunStatus::Running
                                | AgentTaskRunStatus::Reconciling
                        )
                })
                .count();
            let overlap_blocked = task.overlap_policy
                == crate::task_scheduling::AgentTaskOverlapPolicy::Skip
                && active_count >= usize::from(task.max_concurrent_runs);
            let remaining = request.limit.saturating_sub(materialized.len()).max(1);
            let plan = plan_task_materialization(
                task,
                &request.now,
                usize::from(task.max_catch_up_runs).min(remaining),
                overlap_blocked,
            )?;

            for scheduled_for in plan.occurrences {
                if materialized.len() >= request.limit {
                    break;
                }
                let duplicate = runs.values().any(|run| {
                    run.tenant_id == task.tenant_id
                        && run.organization_id == task.organization_id
                        && run.task_id == task.task_id
                        && run.schedule_generation == task.generation
                        && run.scheduled_for == scheduled_for
                        && run.trigger_kind == AgentTaskTriggerKind::Scheduled
                });
                if duplicate {
                    continue;
                }
                let id = self.id_generator.next_id()?;
                let run_id = format!("{ID_PREFIX_RUN}{id}");
                validate_standard_id(&run_id, "runId", Some(ID_PREFIX_RUN))?;
                let turn_id = format!("{ID_PREFIX_TURN}{}", self.id_generator.next_id()?);
                let run = AgentTaskRunRecord {
                    id,
                    run_id: run_id.clone(),
                    tenant_id: task.tenant_id,
                    organization_id: task.organization_id,
                    task_id: task.task_id.clone(),
                    session_id: task.session_id.clone(),
                    agent_id: task.agent_id.clone(),
                    owner_user_id: task.owner_user_id,
                    trigger_kind: AgentTaskTriggerKind::Scheduled,
                    schedule_generation: task.generation,
                    scheduled_for: scheduled_for.clone(),
                    retry_of_run_id: None,
                    priority: task.priority,
                    status: AgentTaskRunStatus::Pending,
                    idempotency_key: format!("agent-task-run:{run_id}"),
                    payload_hash: task_run_payload_hash(
                        &task.task_id,
                        &task.session_id,
                        task.generation,
                        &scheduled_for,
                        &task.prompt,
                    )?,
                    turn_id: Some(turn_id),
                    attempt_count: 0,
                    max_attempts: task.max_attempts,
                    available_at: request.now.clone(),
                    lease_owner: None,
                    lease_token_hash: None,
                    lease_expires_at: None,
                    fencing_token: 0,
                    timeout_at: None,
                    failure_class: None,
                    error_code: None,
                    error_detail: None,
                    version: 0,
                    created_at: request.now.clone(),
                    updated_at: request.now.clone(),
                    claimed_at: None,
                    started_at: None,
                    finished_at: None,
                    cancelled_at: None,
                };
                runs.insert(
                    (run.tenant_id, run.organization_id, run.run_id.clone()),
                    run.clone(),
                );
                materialized.push(run);
            }

            task.next_fire_at = plan.next_fire_at;
            task.status = plan.status;
            task.completed_at = plan.completed_at;
            task.updated_at = request.now.clone();
            task.version = task.version.saturating_add(1);
            task_index.remove(&previous_index_key);
            task_index.insert(task_index_key(task), key);
        }
        Ok(materialized)
    }

    fn claim_task_runs(&self, request: &ClaimTaskRunsRequest) -> KernelResult<Vec<TaskRunClaim>> {
        if is_blank(Some(&request.worker_id)) || request.worker_id.len() > 128 {
            return Err(KernelError::validation("workerId is invalid"));
        }
        let now = parse_datetime(&request.now, None)
            .ok_or_else(|| KernelError::validation("now must be an RFC 3339 instant"))?;
        let lease_expires_at = format_datetime(
            now + chrono::Duration::seconds(i64::from(request.lease_seconds)),
            None,
        );
        let tasks = self.tasks.recovering_read();
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let mut tenant_active_counts = HashMap::<u64, usize>::new();
        for run in runs.values().filter(|run| {
            matches!(
                run.status,
                AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running
            )
        }) {
            *tenant_active_counts.entry(run.tenant_id).or_insert(0) += 1;
        }
        let mut candidates = runs
            .iter()
            .filter(|(_, run)| {
                run.status == AgentTaskRunStatus::Pending
                    && parse_datetime(&run.available_at, None).is_some_and(|value| value <= now)
            })
            .map(|(key, run)| {
                (
                    Reverse(run.priority),
                    run.available_at.clone(),
                    run.scheduled_for.clone(),
                    run.id,
                    key.clone(),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort();

        let mut claims = Vec::new();
        for (_, _, _, _, key) in candidates {
            if claims.len() >= request.limit {
                break;
            }
            let run_snapshot = runs
                .get(&key)
                .cloned()
                .ok_or_else(|| KernelError::not_found("task Run not found"))?;
            let Some(task) = tasks.get(&(
                run_snapshot.tenant_id,
                run_snapshot.organization_id,
                run_snapshot.task_id.clone(),
            )) else {
                continue;
            };
            if task.generation != run_snapshot.schedule_generation {
                if let Some(run) = runs.get_mut(&key) {
                    run.status = AgentTaskRunStatus::DeadLetter;
                    run.failure_class = Some("fencing_conflict".to_string());
                    run.error_code = Some("stale_task_generation".to_string());
                    run.finished_at = Some(request.now.clone());
                    run.updated_at = request.now.clone();
                    run.version = run.version.saturating_add(1);
                }
                continue;
            }
            if tenant_active_counts
                .get(&run_snapshot.tenant_id)
                .copied()
                .unwrap_or(0)
                >= request.max_concurrent_runs_per_tenant
            {
                continue;
            }
            let active_count = runs
                .values()
                .filter(|run| {
                    run.tenant_id == run_snapshot.tenant_id
                        && run.organization_id == run_snapshot.organization_id
                        && run.task_id == run_snapshot.task_id
                        && matches!(
                            run.status,
                            AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running
                        )
                })
                .count();
            if active_count >= usize::from(task.max_concurrent_runs) {
                continue;
            }

            let raw_token = sdkwork_utils_rust::random_string(48);
            let token_hash = sha256_hash(raw_token.as_bytes());
            let attempt_id_value = self.id_generator.next_id()?;
            let attempt_id = format!("{ID_PREFIX_ATTEMPT}{attempt_id_value}");
            validate_standard_id(&attempt_id, "attemptId", Some(ID_PREFIX_ATTEMPT))?;
            let run = runs
                .get_mut(&key)
                .ok_or_else(|| KernelError::not_found("task Run not found"))?;
            run.status = AgentTaskRunStatus::Claimed;
            run.attempt_count = run.attempt_count.saturating_add(1);
            run.lease_owner = Some(request.worker_id.clone());
            run.lease_token_hash = Some(token_hash.clone());
            run.lease_expires_at = Some(lease_expires_at.clone());
            run.fencing_token = run.fencing_token.saturating_add(1);
            run.claimed_at = Some(request.now.clone());
            run.updated_at = request.now.clone();
            run.version = run.version.saturating_add(1);
            run.timeout_at = Some(format_datetime(
                now + chrono::Duration::seconds(i64::from(task.timeout_seconds)),
                None,
            ));
            let attempt = AgentTaskRunAttemptRecord {
                id: attempt_id_value,
                attempt_id: attempt_id.clone(),
                tenant_id: run.tenant_id,
                organization_id: run.organization_id,
                run_id: run.run_id.clone(),
                attempt_no: run.attempt_count,
                worker_id: request.worker_id.clone(),
                status: AgentTaskRunAttemptStatus::Claimed,
                lease_token_hash: token_hash,
                fencing_token: run.fencing_token,
                failure_class: None,
                error_code: None,
                error_detail: None,
                created_at: request.now.clone(),
                updated_at: request.now.clone(),
                started_at: None,
                heartbeat_at: None,
                finished_at: None,
            };
            attempts.insert(
                (
                    attempt.tenant_id,
                    attempt.organization_id,
                    attempt.attempt_id.clone(),
                ),
                attempt.clone(),
            );
            claims.push(TaskRunClaim {
                run: run.clone(),
                attempt,
                lease: TaskRunLease {
                    tenant_id: run.tenant_id,
                    organization_id: run.organization_id,
                    run_id: run.run_id.clone(),
                    attempt_id,
                    worker_id: request.worker_id.clone(),
                    lease_token: raw_token,
                    fencing_token: run.fencing_token,
                },
            });
            *tenant_active_counts.entry(run.tenant_id).or_insert(0) += 1;
        }
        Ok(claims)
    }

    fn mark_task_run_running(
        &self,
        lease: &TaskRunLease,
        started_at: &str,
    ) -> KernelResult<AgentTaskRunRecord> {
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let run = checked_in_memory_run_lease(&mut runs, lease, started_at)?;
        if run.status != AgentTaskRunStatus::Claimed {
            return Err(KernelError::conflict("task Run is not claimed"));
        }
        run.status = AgentTaskRunStatus::Running;
        run.started_at = Some(started_at.to_string());
        run.updated_at = started_at.to_string();
        run.version = run.version.saturating_add(1);
        let attempt = attempts
            .get_mut(&(
                lease.tenant_id,
                lease.organization_id,
                lease.attempt_id.clone(),
            ))
            .ok_or_else(|| KernelError::not_found("task Run Attempt not found"))?;
        attempt.status = AgentTaskRunAttemptStatus::Running;
        attempt.started_at = Some(started_at.to_string());
        attempt.heartbeat_at = Some(started_at.to_string());
        attempt.updated_at = started_at.to_string();
        Ok(run.clone())
    }

    fn heartbeat_task_run(
        &self,
        lease: &TaskRunLease,
        heartbeat_at: &str,
        lease_seconds: u32,
    ) -> KernelResult<AgentTaskRunRecord> {
        let heartbeat = parse_datetime(heartbeat_at, None)
            .ok_or_else(|| KernelError::validation("heartbeatAt is invalid"))?;
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let run = checked_in_memory_run_lease(&mut runs, lease, heartbeat_at)?;
        if !matches!(
            run.status,
            AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running
        ) {
            return Err(KernelError::conflict("task Run lease is not active"));
        }
        run.lease_expires_at = Some(format_datetime(
            heartbeat + chrono::Duration::seconds(i64::from(lease_seconds)),
            None,
        ));
        run.updated_at = heartbeat_at.to_string();
        run.version = run.version.saturating_add(1);
        let attempt = attempts
            .get_mut(&(
                lease.tenant_id,
                lease.organization_id,
                lease.attempt_id.clone(),
            ))
            .ok_or_else(|| KernelError::not_found("task Run Attempt not found"))?;
        attempt.heartbeat_at = Some(heartbeat_at.to_string());
        attempt.updated_at = heartbeat_at.to_string();
        Ok(run.clone())
    }

    fn complete_task_run(
        &self,
        lease: &TaskRunLease,
        turn_id: &str,
        completed_at: &str,
    ) -> KernelResult<AgentTaskRunRecord> {
        let tasks = self.tasks.recovering_read();
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let run = checked_in_memory_run_lease(&mut runs, lease, completed_at)?;
        if !matches!(
            run.status,
            AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running
        ) {
            return Err(KernelError::conflict("task Run is not completable"));
        }
        if run.turn_id.as_deref() != Some(turn_id) {
            return Err(KernelError::conflict("task Run Turn identity mismatch"));
        }
        let current_generation = tasks
            .get(&(run.tenant_id, run.organization_id, run.task_id.clone()))
            .map(|task| task.generation)
            .ok_or_else(|| KernelError::conflict("task Run Task not found"))?;
        if current_generation != run.schedule_generation {
            return Err(KernelError::conflict("stale task Run generation"));
        }
        run.status = AgentTaskRunStatus::Succeeded;
        run.lease_owner = None;
        run.lease_token_hash = None;
        run.lease_expires_at = None;
        run.finished_at = Some(completed_at.to_string());
        run.updated_at = completed_at.to_string();
        run.version = run.version.saturating_add(1);
        let attempt = attempts
            .get_mut(&(
                lease.tenant_id,
                lease.organization_id,
                lease.attempt_id.clone(),
            ))
            .ok_or_else(|| KernelError::not_found("task Run Attempt not found"))?;
        attempt.status = AgentTaskRunAttemptStatus::Succeeded;
        attempt.finished_at = Some(completed_at.to_string());
        attempt.updated_at = completed_at.to_string();
        Ok(run.clone())
    }

    fn fail_task_run(&self, request: &FailTaskRunRequest) -> KernelResult<AgentTaskRunRecord> {
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let run = checked_in_memory_run_lease(&mut runs, &request.lease, &request.failed_at)?;
        let retry_allowed = request.disposition == TaskRunFailureDisposition::Retry
            && run.attempt_count < run.max_attempts;
        run.status = if retry_allowed {
            AgentTaskRunStatus::Pending
        } else {
            match request.disposition {
                TaskRunFailureDisposition::Reconcile => AgentTaskRunStatus::Reconciling,
                TaskRunFailureDisposition::Retry => AgentTaskRunStatus::DeadLetter,
                TaskRunFailureDisposition::Terminal => AgentTaskRunStatus::Failed,
            }
        };
        run.available_at = if retry_allowed {
            request
                .retry_at
                .clone()
                .ok_or_else(|| KernelError::validation("retryAt is required"))?
        } else {
            run.available_at.clone()
        };
        run.lease_owner = None;
        run.lease_token_hash = None;
        run.lease_expires_at = None;
        run.failure_class = Some(request.failure_class.clone());
        run.error_code = Some(request.error_code.clone());
        run.error_detail = None;
        run.finished_at = matches!(
            run.status,
            AgentTaskRunStatus::Failed | AgentTaskRunStatus::DeadLetter
        )
        .then(|| request.failed_at.clone());
        run.updated_at = request.failed_at.clone();
        run.version = run.version.saturating_add(1);
        let attempt = attempts
            .get_mut(&(
                request.lease.tenant_id,
                request.lease.organization_id,
                request.lease.attempt_id.clone(),
            ))
            .ok_or_else(|| KernelError::not_found("task Run Attempt not found"))?;
        attempt.status = AgentTaskRunAttemptStatus::Failed;
        attempt.failure_class = Some(request.failure_class.clone());
        attempt.error_code = Some(request.error_code.clone());
        attempt.error_detail = None;
        attempt.finished_at = Some(request.failed_at.clone());
        attempt.updated_at = request.failed_at.clone();
        Ok(run.clone())
    }

    fn recover_expired_task_run_leases(&self, now: &str, limit: usize) -> KernelResult<u64> {
        let now_at = parse_datetime(now, None)
            .ok_or_else(|| KernelError::validation("now must be an RFC 3339 instant"))?;
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let mut keys = runs
            .iter()
            .filter(|(_, run)| {
                matches!(
                    run.status,
                    AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running
                ) && run
                    .lease_expires_at
                    .as_deref()
                    .and_then(|value| parse_datetime(value, None))
                    .is_some_and(|value| value < now_at)
            })
            .map(|(key, run)| (run.lease_expires_at.clone(), run.id, key.clone()))
            .collect::<Vec<_>>();
        keys.sort();
        keys.truncate(limit.clamp(1, 1_000));
        for (_, _, key) in &keys {
            let run = runs
                .get_mut(key)
                .ok_or_else(|| KernelError::not_found("task Run not found"))?;
            if let Some(attempt) = attempts.values_mut().find(|attempt| {
                attempt.tenant_id == run.tenant_id
                    && attempt.organization_id == run.organization_id
                    && attempt.run_id == run.run_id
                    && attempt.attempt_no == run.attempt_count
            }) {
                attempt.status = AgentTaskRunAttemptStatus::LeaseExpired;
                attempt.failure_class = Some("lease_lost".to_string());
                attempt.error_code = Some("task_run_lease_expired".to_string());
                attempt.finished_at = Some(now.to_string());
                attempt.updated_at = now.to_string();
            }
            let exhausted = run.attempt_count >= run.max_attempts;
            run.status = if exhausted {
                AgentTaskRunStatus::DeadLetter
            } else {
                AgentTaskRunStatus::Pending
            };
            run.available_at = now.to_string();
            run.lease_owner = None;
            run.lease_token_hash = None;
            run.lease_expires_at = None;
            run.failure_class = Some("lease_lost".to_string());
            run.error_code = Some("task_run_lease_expired".to_string());
            run.finished_at = exhausted.then(|| now.to_string());
            run.updated_at = now.to_string();
            run.version = run.version.saturating_add(1);
        }
        u64::try_from(keys.len())
            .map_err(|_| KernelError::conflict("recovered task Run count overflow"))
    }

    fn recover_timed_out_task_runs(&self, now: &str, limit: usize) -> KernelResult<u64> {
        let now_at = parse_datetime(now, None)
            .ok_or_else(|| KernelError::validation("now must be an RFC 3339 instant"))?;
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let mut keys = runs
            .iter()
            .filter(|(_, run)| {
                matches!(
                    run.status,
                    AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running
                ) && run
                    .timeout_at
                    .as_deref()
                    .and_then(|value| parse_datetime(value, None))
                    .is_some_and(|value| value < now_at)
            })
            .map(|(key, run)| (run.timeout_at.clone(), run.id, key.clone()))
            .collect::<Vec<_>>();
        keys.sort();
        keys.truncate(limit.clamp(1, 1_000));
        for (_, _, key) in &keys {
            let run = runs
                .get_mut(key)
                .ok_or_else(|| KernelError::not_found("task Run not found"))?;
            if let Some(attempt) = attempts.values_mut().find(|attempt| {
                attempt.tenant_id == run.tenant_id
                    && attempt.organization_id == run.organization_id
                    && attempt.run_id == run.run_id
                    && attempt.attempt_no == run.attempt_count
            }) {
                attempt.status = AgentTaskRunAttemptStatus::LeaseExpired;
                attempt.failure_class = Some("execution_timeout".to_string());
                attempt.error_code = Some("task_run_timed_out".to_string());
                attempt.finished_at = Some(now.to_string());
                attempt.updated_at = now.to_string();
            }
            let exhausted = run.attempt_count >= run.max_attempts;
            run.status = if exhausted {
                AgentTaskRunStatus::DeadLetter
            } else {
                AgentTaskRunStatus::Pending
            };
            run.available_at = now.to_string();
            run.lease_owner = None;
            run.lease_token_hash = None;
            run.lease_expires_at = None;
            run.timeout_at = None;
            run.failure_class = Some("execution_timeout".to_string());
            run.error_code = Some("task_run_timed_out".to_string());
            run.finished_at = exhausted.then(|| now.to_string());
            run.updated_at = now.to_string();
            run.version = run.version.saturating_add(1);
        }
        u64::try_from(keys.len())
            .map_err(|_| KernelError::conflict("timed-out task Run count overflow"))
    }

    fn request_task_run_cancellation(
        &self,
        tenant_id: u64,
        organization_id: u64,
        run_id: &str,
        expected_version: Option<u64>,
        requested_at: &str,
    ) -> KernelResult<AgentTaskRunRecord> {
        parse_datetime(requested_at, None)
            .ok_or_else(|| KernelError::validation("requestedAt is invalid"))?;
        let mut runs = self.task_runs.recovering_write();
        let mut attempts = self.task_run_attempts.recovering_write();
        let run = runs
            .get_mut(&(tenant_id, organization_id, run_id.to_string()))
            .ok_or_else(|| KernelError::not_found("task Run not found"))?;
        if expected_version.is_some_and(|version| version != run.version) {
            return Err(KernelError::conflict("task Run version mismatch"));
        }
        match run.status {
            AgentTaskRunStatus::Cancelled | AgentTaskRunStatus::Reconciling => {
                return Ok(run.clone());
            }
            AgentTaskRunStatus::Pending => {
                run.status = AgentTaskRunStatus::Cancelled;
                run.failure_class = Some("cancelled".to_string());
                run.error_code = Some("task_run_cancelled_before_claim".to_string());
                run.finished_at = Some(requested_at.to_string());
                run.cancelled_at = Some(requested_at.to_string());
            }
            AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running => {
                if let Some(attempt) = attempts.values_mut().find(|attempt| {
                    attempt.tenant_id == run.tenant_id
                        && attempt.organization_id == run.organization_id
                        && attempt.run_id == run.run_id
                        && attempt.attempt_no == run.attempt_count
                        && matches!(
                            attempt.status,
                            AgentTaskRunAttemptStatus::Claimed | AgentTaskRunAttemptStatus::Running
                        )
                }) {
                    attempt.status = AgentTaskRunAttemptStatus::Failed;
                    attempt.failure_class = Some("outcome_unknown".to_string());
                    attempt.error_code = Some("task_run_cancellation_requested".to_string());
                    attempt.finished_at = Some(requested_at.to_string());
                    attempt.updated_at = requested_at.to_string();
                }
                run.status = AgentTaskRunStatus::Reconciling;
                run.failure_class = Some("outcome_unknown".to_string());
                run.error_code = Some("task_run_cancellation_requested".to_string());
                run.lease_owner = None;
                run.lease_token_hash = None;
                run.lease_expires_at = None;
            }
            AgentTaskRunStatus::Succeeded
            | AgentTaskRunStatus::Failed
            | AgentTaskRunStatus::DeadLetter => {
                return Err(KernelError::validation("task Run cannot be cancelled"));
            }
        }
        run.updated_at = requested_at.to_string();
        run.version = run.version.saturating_add(1);
        Ok(run.clone())
    }

    fn reconcile_task_run(
        &self,
        request: &ReconcileTaskRunRequest,
    ) -> KernelResult<AgentTaskRunRecord> {
        parse_datetime(&request.reconciled_at, None)
            .ok_or_else(|| KernelError::validation("reconciledAt is invalid"))?;
        if !matches!(
            request.terminal_status,
            AgentTaskRunStatus::Succeeded
                | AgentTaskRunStatus::Failed
                | AgentTaskRunStatus::Cancelled
        ) {
            return Err(KernelError::validation(
                "task Run reconciliation status is invalid",
            ));
        }
        let mut runs = self.task_runs.recovering_write();
        let run = runs
            .get_mut(&(
                request.tenant_id,
                request.organization_id,
                request.run_id.clone(),
            ))
            .ok_or_else(|| KernelError::not_found("task Run not found"))?;
        if run.status != AgentTaskRunStatus::Reconciling || run.version != request.expected_version
        {
            return Err(KernelError::conflict(
                "task Run reconciliation version mismatch",
            ));
        }
        run.status = request.terminal_status;
        run.failure_class = match request.terminal_status {
            AgentTaskRunStatus::Succeeded => None,
            AgentTaskRunStatus::Cancelled => Some("cancelled".to_string()),
            AgentTaskRunStatus::Failed => Some("reconciled_failure".to_string()),
            _ => unreachable!(),
        };
        run.error_code = request.error_code.clone().or_else(|| {
            (request.terminal_status == AgentTaskRunStatus::Cancelled)
                .then(|| "task_run_cancelled".to_string())
        });
        run.finished_at = Some(request.reconciled_at.clone());
        run.cancelled_at = (request.terminal_status == AgentTaskRunStatus::Cancelled)
            .then(|| request.reconciled_at.clone());
        run.updated_at = request.reconciled_at.clone();
        run.version = run.version.saturating_add(1);
        Ok(run.clone())
    }

    fn list_reconciling_task_runs(
        &self,
        updated_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTaskRunRecord>> {
        let cutoff = parse_datetime(updated_before, None)
            .ok_or_else(|| KernelError::validation("updatedBefore is invalid"))?;
        let mut runs = self
            .task_runs
            .recovering_read()
            .values()
            .filter(|run| {
                run.status == AgentTaskRunStatus::Reconciling
                    && parse_datetime(&run.updated_at, None).is_some_and(|value| value <= cutoff)
            })
            .cloned()
            .collect::<Vec<_>>();
        runs.sort_by(|left, right| {
            left.updated_at
                .cmp(&right.updated_at)
                .then(left.id.cmp(&right.id))
        });
        runs.truncate(limit.clamp(1, 1_000));
        Ok(runs)
    }

    fn list_task_runs(&self, query: &TaskRunListQuery) -> KernelResult<Vec<AgentTaskRunRecord>> {
        let mut runs = self
            .task_runs
            .recovering_read()
            .values()
            .filter(|run| {
                run.tenant_id == query.tenant_id
                    && run.organization_id == query.organization_id
                    && run.task_id == query.task_id
                    && query
                        .owner_user_id
                        .is_none_or(|owner_user_id| run.owner_user_id == owner_user_id)
                    && query.status.is_none_or(|status| run.status == status)
                    && query
                        .trigger_kind
                        .is_none_or(|trigger_kind| run.trigger_kind == trigger_kind)
                    && query
                        .cursor
                        .as_ref()
                        .is_none_or(|cursor| run.id < cursor.run_internal_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        runs.sort_by(|left, right| right.id.cmp(&left.id));
        runs.truncate(query.store_limit());
        Ok(runs)
    }

    fn list_task_run_attempts(
        &self,
        query: &TaskRunAttemptListQuery,
    ) -> KernelResult<Vec<AgentTaskRunAttemptRecord>> {
        let mut attempts = self
            .task_run_attempts
            .recovering_read()
            .values()
            .filter(|attempt| {
                attempt.tenant_id == query.tenant_id
                    && attempt.organization_id == query.organization_id
                    && attempt.run_id == query.run_id
                    && query.cursor.as_ref().is_none_or(|cursor| {
                        attempt.attempt_no < cursor.attempt_no
                            || (attempt.attempt_no == cursor.attempt_no
                                && attempt.id < cursor.attempt_internal_id)
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        attempts.sort_by(|left, right| {
            right
                .attempt_no
                .cmp(&left.attempt_no)
                .then(right.id.cmp(&left.id))
        });
        attempts.truncate(query.store_limit());
        Ok(attempts)
    }

    fn get_task_run(
        &self,
        tenant_id: u64,
        organization_id: u64,
        run_id: &str,
    ) -> KernelResult<Option<AgentTaskRunRecord>> {
        Ok(self
            .task_runs
            .recovering_read()
            .get(&(tenant_id, organization_id, run_id.to_string()))
            .cloned())
    }
}

fn checked_in_memory_run_lease<'a>(
    runs: &'a mut HashMap<TaskRunPrimaryKey, AgentTaskRunRecord>,
    lease: &TaskRunLease,
    at: &str,
) -> KernelResult<&'a mut AgentTaskRunRecord> {
    let at = parse_datetime(at, None)
        .ok_or_else(|| KernelError::validation("lease operation timestamp is invalid"))?;
    let run = runs
        .get_mut(&(lease.tenant_id, lease.organization_id, lease.run_id.clone()))
        .ok_or_else(|| KernelError::not_found("task Run not found"))?;
    if run.lease_owner.as_deref() != Some(lease.worker_id.as_str())
        || run.lease_token_hash.as_deref()
            != Some(sha256_hash(lease.lease_token.as_bytes()).as_str())
        || run.fencing_token != lease.fencing_token
        || run
            .lease_expires_at
            .as_deref()
            .and_then(|value| parse_datetime(value, None))
            .is_none_or(|expires_at| expires_at < at)
    {
        return Err(KernelError::conflict("task Run lease lost"));
    }
    Ok(run)
}

// ---------------------------------------------------------------------------
// Policy provider infrastructure
// ---------------------------------------------------------------------------

/// Policy mode that determines the outcome of every policy evaluation.
/// `Allow` permits all requests with the given reason; `Deny` rejects all
/// requests with the given reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyMode {
    Allow(String),
    Deny(String),
}

/// A simple static policy provider that returns the same decision for every
/// request based on its configured [`PolicyMode`].
///
/// **Never use this provider in production.** It does not evaluate subject
/// roles or resource attributes. Production deployments use
/// [`IamGatedPolicyProvider`]. Use [`DenyAllPolicyProvider`] as the
/// fail-closed default whenever no policy provider is explicitly configured.
/// This type is retained only for local development and integration-test
/// scenarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowAllPolicyProvider {
    pub provider_id: String,
    pub mode: PolicyMode,
}

impl AllowAllPolicyProvider {
    /// Create a dev-only provider that allows every request. The `provider_id`
    /// should identify the policy source (e.g. `"policy.agents.dev"` for
    /// development).
    ///
    /// # Security Check
    /// This method validates that the application is not running with
    /// development auth bypass in a production-like profile.
    pub fn try_allow(provider_id: impl Into<String>) -> Result<Self, String> {
        validate_production_security_config()?;
        Ok(Self {
            provider_id: provider_id.into(),
            mode: PolicyMode::Allow("static.allow".to_string()),
        })
    }

    /// Compatibility constructor for tests and local-only callers.
    ///
    /// If the environment is misconfigured as production-like with dev auth
    /// bypass enabled, it logs the violation and returns a deny-all static
    /// provider instead of panicking or allowing requests.
    pub fn allow(provider_id: impl Into<String>) -> Self {
        let provider_id = provider_id.into();
        match Self::try_allow(provider_id.clone()) {
            Ok(provider) => provider,
            Err(message) => {
                tracing::error!(
                    env_var = ENV_DEV_AUTH_BYPASS,
                    deployment = %sdkwork_agents_contract::agents_deployment_environment_name(),
                    error = %message,
                    "development auth bypass policy provider refused production-like configuration"
                );
                Self {
                    provider_id,
                    mode: PolicyMode::Deny("static.deny.security_misconfiguration".to_string()),
                }
            }
        }
    }
}

impl PolicyProvider for AllowAllPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            self.provider_id.clone(),
            "policy",
            "static-policy-provider",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        let decision_id = format!(
            "decision_{}_{}",
            self.provider_id, request.policy_request_id
        );
        match &self.mode {
            PolicyMode::Allow(reason) => Ok(PolicyDecision::allow(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
            )
            .with_safe_reason(reason.clone())),
            PolicyMode::Deny(reason) => Ok(PolicyDecision::deny(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
                reason,
            )),
        }
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

/// Fail-closed policy provider that denies every request. Use this as the
/// default when no IAM integration is configured, so that misconfiguration
/// can never accidentally allow unauthorized access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenyAllPolicyProvider {
    pub provider_id: String,
    pub reason: String,
}

impl DenyAllPolicyProvider {
    pub fn new(provider_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            reason: reason.into(),
        }
    }
}

impl Default for DenyAllPolicyProvider {
    fn default() -> Self {
        Self::new(
            "policy.agents.deny-all",
            "no policy provider configured; access denied by default",
        )
    }
}

impl PolicyProvider for DenyAllPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            self.provider_id.clone(),
            "policy",
            "deny-all-policy-provider",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        let decision_id = format!(
            "decision_{}_{}",
            self.provider_id, request.policy_request_id
        );
        Ok(PolicyDecision::deny(
            decision_id,
            request.policy_request_id,
            self.provider_id.clone(),
            self.reason.clone(),
        ))
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

/// IAM permission codes that gate agent business operations. These align with
/// the `ai` module manifest in `sdkwork-iam/iam/modules/ai/iam.module.manifest.json`.
pub const IAM_PERMISSION_AGENTS_MANAGE: &str = "ai.agents.manage";
pub const IAM_PERMISSION_AGENTS_READ: &str = "ai.agents.read";
pub const IAM_PERMISSION_AGENTS_USE: &str = "ai.agents.use";

/// Role codes that grant wildcard AI permissions per the IAM module manifest
/// `roleGrantExtensions` (org_admin and org_operations both map to `ai.*`).
const IAM_ADMIN_ROLE_CODES: &[&str] = &["org_admin", "org_operations"];

/// Canonical read-only operation suffixes used by `AgentsService::authorize`.
/// Resource-qualified actions such as `project.list` and `agent_engine.list`
/// resolve through the same operation suffix as unqualified actions.
const READ_ONLY_POLICY_OPERATIONS: &[&str] = &["list", "read", "retrieve"];

/// Owner-scoped and runtime mutations available to ordinary app users. This
/// list is intentionally exact so future actions remain manage-only until
/// their ownership checks and permission classification are reviewed.
const SELF_SERVICE_POLICY_ACTIONS: &[&str] = &[
    "checkpoint.create",
    "checkpoint.invalidate",
    "checkpoint.restore",
    "create",
    "interaction.answer",
    "interaction.approve",
    "interaction.claim",
    "interaction.create",
    "interaction.resolve",
    "item_feedback.update",
    "project.archive",
    "project.composition_slot.create",
    "project.composition_slot.delete",
    "project.composition_slot.update",
    "project.create",
    "project.delete",
    "project.update",
    "runtime.preview_response",
    "runtime.prompt_optimization",
    "session.archive",
    "session.close",
    "session.create",
    "session.delete",
    "session.update",
    "session.user_state.update",
    "session_item.create",
    "session_runtime_binding.activate",
    "session_runtime_binding.create",
    "session_runtime_binding.deactivate",
    "session_runtime_binding.update",
    "task.cancel",
    "task.create",
    "task.execute",
    "turn.cancel",
    "turn.create",
    "workspace.archive",
    "workspace.create",
    "workspace.delete",
    "workspace.ensureDefault",
    "workspace.update",
];

/// IAM-gated policy provider that maps agent business actions to IAM permission
/// scopes and evaluates the request subject's roles/scopes against the
/// required permission.
///
/// This provider implements defense-in-depth at the application service layer.
/// The web framework layer (`IamAuthorizationPolicy` from `sdkwork-iam-web-adapter`)
/// performs HTTP route-level authorization first; this provider performs
/// resource-action-level authorization as a second gate.
///
/// Subject roles are populated from gateway-injected headers
/// (`x-subject-roles` or `x-sdkwork-permission-scope`) and may contain either
/// IAM permission scopes (e.g. `ai.agents.read`) or role codes (e.g.
/// `org_admin`). Both are honored:
///
/// - Permission scopes are matched directly, with `ai.*` and `*` wildcards.
/// - Admin role codes (`org_admin`, `org_operations`) are granted `ai.*`.
///
/// Requests without a subject or with no matching permission are denied
/// (fail-closed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IamGatedPolicyProvider {
    pub provider_id: String,
}

impl IamGatedPolicyProvider {
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
        }
    }

    fn is_read_only_action(action: &str) -> bool {
        let operation = action.rsplit('.').next().unwrap_or(action);
        READ_ONLY_POLICY_OPERATIONS.contains(&operation)
    }

    fn is_self_service_action(action: &str) -> bool {
        SELF_SERVICE_POLICY_ACTIONS.contains(&action)
    }

    /// Determine the required IAM permission for the given policy action.
    fn required_permission_for_action(action: Option<&str>) -> &'static str {
        match action {
            Some(action) if Self::is_read_only_action(action) => IAM_PERMISSION_AGENTS_READ,
            Some(action) if Self::is_self_service_action(action) => IAM_PERMISSION_AGENTS_USE,
            _ => IAM_PERMISSION_AGENTS_MANAGE,
        }
    }

    /// Return `true` if the subject's role/scope entry satisfies the required
    /// permission. Supports wildcards `ai.*` and `*`, and known admin role
    /// codes that grant `ai.*`. Also honors the implication that
    /// `ai.agents.manage` grants both read and use capabilities.
    fn entry_grants_permission(entry: &str, required_permission: &str) -> bool {
        let entry = entry.trim();
        if entry.is_empty() {
            return false;
        }
        if entry == "*" {
            return true;
        }
        if entry == required_permission {
            return true;
        }
        // Manage permission implies all narrower capabilities for the same resource.
        if entry == IAM_PERMISSION_AGENTS_MANAGE
            && matches!(
                required_permission,
                IAM_PERMISSION_AGENTS_READ | IAM_PERMISSION_AGENTS_USE
            )
        {
            return true;
        }
        // Wildcard within the ai domain (e.g. `ai.*` matches `ai.agents.read`).
        if entry == "ai.*" && required_permission.starts_with("ai.") {
            return true;
        }
        // Admin role codes that the IAM module grants `ai.*` to.
        if IAM_ADMIN_ROLE_CODES.contains(&entry) && required_permission.starts_with("ai.") {
            return true;
        }
        false
    }

    /// Return `true` if the subject has any role/scope entry that satisfies
    /// the required permission.
    fn subject_has_permission(
        subject: Option<&sdkwork_agent_kernel::PolicySubject>,
        required_permission: &str,
    ) -> bool {
        let Some(subject) = subject else {
            return false;
        };
        subject
            .roles
            .iter()
            .any(|entry| Self::entry_grants_permission(entry.as_str(), required_permission))
    }
}

impl Default for IamGatedPolicyProvider {
    fn default() -> Self {
        Self::new("policy.agents.production.iam-gated")
    }
}

impl PolicyProvider for IamGatedPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            self.provider_id.clone(),
            "policy",
            "iam-gated-policy-provider",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        let decision_id = format!(
            "decision_{}_{}",
            self.provider_id, request.policy_request_id
        );
        let action = request.action.as_deref();
        let required_permission = Self::required_permission_for_action(action);
        if Self::subject_has_permission(request.subject.as_ref(), required_permission) {
            Ok(PolicyDecision::allow(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
            )
            .with_safe_reason(format!("iam.permission.satisfied:{required_permission}")))
        } else {
            Ok(PolicyDecision::deny(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
                "iam.permission.missing",
            )
            .with_safe_reason(format!("iam.permission.missing:{required_permission}")))
        }
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

// ---------------------------------------------------------------------------
// In-memory audit sink infrastructure
// ---------------------------------------------------------------------------

/// Maximum number of audit events retained by [`InMemoryAgentAuditSink`].
/// When exceeded, the oldest events (smallest sort key) are evicted. This
/// prevents unbounded memory growth in long-running processes.
const MAX_IN_MEMORY_AUDIT_EVENTS: usize = 10_000;

/// Sort key for audit events: `(occurred_at, event_id)` wrapped in `Reverse`
/// for descending chronological order. The `BTreeMap` maintains this index
/// incrementally, satisfying PAGINATION_SPEC §5.3 (no per-request rebuild).
type AuditEventIndexKey = Reverse<(time::OffsetDateTime, String)>;

fn audit_event_sort_key(event: &KernelEvent) -> KernelResult<AuditEventIndexKey> {
    let occurred_at = event
        .occurred_at
        .as_deref()
        .ok_or_else(|| KernelError::validation("audit event occurred_at is required"))?;
    let occurred_at = parse_rfc3339_datetime(occurred_at, "audit event occurred_at")?;
    Ok(Reverse((occurred_at, event.event_id.clone())))
}

/// In-memory audit sink backed by a `BTreeMap` with bounded capacity.
/// Uses `Mutex` for interior mutability. Events are lost when the process
/// exits. Use [`crate::persistence::SqlAgentAuditSink`] for production
/// deployments that require persistent audit trails.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAgentAuditSink {
    events: std::sync::Arc<Mutex<BTreeMap<AuditEventIndexKey, KernelEvent>>>,
}

impl InMemoryAgentAuditSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<KernelEvent> {
        self.events.recovering_lock().values().cloned().collect()
    }
}

impl AgentAuditSink for InMemoryAgentAuditSink {
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        let key = audit_event_sort_key(&event)?;
        let mut events = self.events.recovering_lock();
        events.insert(key, event);
        // Reverse ordering places the oldest timestamp at the back of the map.
        while events.len() > MAX_IN_MEMORY_AUDIT_EVENTS {
            if let Some(oldest_key) = events.keys().next_back().cloned() {
                events.remove(&oldest_key);
            } else {
                break;
            }
        }
        Ok(())
    }

    fn list_events(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<KernelEvent>> {
        use crate::list_cursors::{encode_audit_event_list_cursor, AuditEventListCursor};
        use crate::ports::PaginatedResult;
        let from = query
            .from
            .as_deref()
            .map(|value| parse_rfc3339_datetime(value, "from"))
            .transpose()?;
        let to = query
            .to
            .as_deref()
            .map(|value| parse_rfc3339_datetime(value, "to"))
            .transpose()?;
        let scope_fingerprint = query.scope_fingerprint();
        if query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
        {
            return Err(KernelError::validation(
                "cursor does not match the requested audit scope",
            ));
        }
        let cursor_ts = query
            .cursor
            .as_ref()
            .map(|cursor| parse_rfc3339_datetime(&cursor.created_at, "cursor.created_at"))
            .transpose()?;
        let cursor_ref = query
            .cursor
            .as_ref()
            .map(|cursor| cursor.event_ref.as_str());
        let events = self.events.recovering_lock();
        // BTreeMap<Reverse<...>> iterates in descending order (newest first).
        // Iterate the incrementally maintained index directly — no collect/sort.
        let page_size = query.pagination.page_size;
        let store_limit = page_size.saturating_add(1);
        let mut items = Vec::with_capacity(store_limit);
        let mut boundary: Option<(String, String)> = None;
        for (Reverse((occurred_at, event_id)), event) in events.iter() {
            if let Some(cursor_ts) = cursor_ts.as_ref() {
                // Keyset continuation: the index is descending, so skip every
                // entry at or above the cursor boundary.
                if *occurred_at > *cursor_ts
                    || (*occurred_at == *cursor_ts
                        && cursor_ref.is_some_and(|cursor_ref| event_id.as_str() >= cursor_ref))
                {
                    continue;
                }
            }
            if !from.map(|from| *occurred_at >= from).unwrap_or(true)
                || !to.map(|to| *occurred_at <= to).unwrap_or(true)
            {
                continue;
            }
            if crate::persistence::extract_event_context(event.payload.as_str(), "agent_id")
                .map(|id| id == query.agent_id)
                .unwrap_or(false)
                && query
                    .action
                    .as_ref()
                    .map(|action| {
                        event
                            .event_type
                            .rsplit('.')
                            .next()
                            .map(|value| value == action)
                            .unwrap_or(false)
                    })
                    .unwrap_or(true)
            {
                items.push(event.clone());
                boundary = Some((
                    occurred_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default(),
                    event_id.clone(),
                ));
                if items.len() >= store_limit {
                    break;
                }
            }
        }
        let has_more = items.len() > page_size;
        items.truncate(page_size);
        let next_page_token = if has_more {
            let (created_at, event_ref) = boundary
                .ok_or_else(|| KernelError::validation("audit page ended without a cursor row"))?;
            encode_audit_event_list_cursor(&AuditEventListCursor {
                created_at,
                event_ref,
                scope_fingerprint,
            })?
            .into()
        } else {
            None
        };
        Ok(PaginatedResult::new(items, next_page_token, None))
    }
}

// ---------------------------------------------------------------------------
// Shared comparison helpers for in-memory repository sorting
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Shared comparison helpers for in-memory repository sorting
// ---------------------------------------------------------------------------

fn session_matches_list_query(
    record: &AgentSessionRecord,
    query: &SessionListQuery,
    workspace_project_ids: Option<&HashSet<String>>,
) -> bool {
    if record.tenant_id != query.tenant_id {
        return false;
    }
    if record.deleted_at.is_some() {
        return false;
    }
    if let Some(organization_id) = query.organization_id {
        if record.organization_id != organization_id {
            return false;
        }
    }
    if let Some(agent_id) = query.agent_id.as_ref() {
        if record.agent_id != *agent_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(project_id) = query.project_id.as_ref() {
        if record.project_id.as_ref() != Some(project_id) {
            return false;
        }
    }
    if let Some(project_ids) = workspace_project_ids {
        if !record
            .project_id
            .as_ref()
            .is_some_and(|project_id| project_ids.contains(project_id))
        {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    query.include_archived || record.status.as_str() != "archived"
}

fn project_matches_list_query(record: &AgentProjectRecord, query: &ProjectListQuery) -> bool {
    if record.tenant_id != query.tenant_id || record.organization_id != query.organization_id {
        return false;
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(workspace_id) = query.workspace_id.as_deref() {
        if record.workspace_id != workspace_id {
            return false;
        }
    }
    if let Some(exact_name) = query.exact_name.as_deref() {
        if !project_names_equal(&record.name, exact_name) {
            return false;
        }
    }
    if let Some(status) = query.status {
        if record.status != status {
            return false;
        }
    }
    if !query.include_deleted && record.status == AgentProjectStatus::Deleted {
        return false;
    }
    if let Some(search_query) = query.search_query.as_deref() {
        let needle = trim(search_query).to_lowercase();
        let description = record.description.as_deref().unwrap_or_default();
        if !record.name.to_lowercase().contains(&needle)
            && !description.to_lowercase().contains(&needle)
        {
            return false;
        }
    }
    true
}

fn workspace_matches_list_query(record: &AgentWorkspaceRecord, query: &WorkspaceListQuery) -> bool {
    if record.tenant_id != query.tenant_id
        || record.organization_id != query.organization_id
        || record.owner_user_id != query.owner_user_id
    {
        return false;
    }
    if let Some(status) = query.status {
        if record.status != status {
            return false;
        }
    }
    query.include_deleted || record.status != AgentWorkspaceStatus::Deleted
}

fn message_matches_list_query(
    record: &AgentSessionItemRecord,
    query: &SessionItemListQuery,
) -> bool {
    if record.tenant_id != query.tenant_id
        || record.organization_id != query.organization_id
        || record.session_id != query.session_id
    {
        return false;
    }
    if let Some(kind) = query.kind.as_ref() {
        if record.kind.as_str() != kind {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    true
}

fn interaction_matches_list_query(
    record: &AgentInteractionRecord,
    query: &InteractionListQuery,
) -> bool {
    if record.tenant_id != query.tenant_id
        || record.organization_id != query.organization_id
        || record.session_id != query.session_id
    {
        return false;
    }
    if let Some(kind) = query.kind.as_ref() {
        if record.kind.as_str() != kind {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    true
}

fn task_matches_list_query(record: &AgentTaskRecord, query: &TaskListQuery) -> bool {
    if record.tenant_id != query.tenant_id || record.organization_id != query.organization_id {
        return false;
    }
    if let Some(agent_id) = query.agent_id.as_ref() {
        if record.agent_id != *agent_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    true
}

fn agent_matches_list_query(record: &AgentBusinessRecord, query: &AgentListQuery) -> bool {
    if record.tenant_id != query.tenant_id {
        return false;
    }
    if let Some(organization_id) = query.organization_id {
        if record.organization_id != organization_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if !query.include_deleted && record.is_deleted() {
        return false;
    }
    if let Some(visibility) = query.visibility {
        if record.visibility != visibility {
            return false;
        }
    }
    if let Some(search_query) = query.search_query.as_ref() {
        if !is_blank(Some(search_query.as_str())) {
            let normalized_query = trim(search_query).to_lowercase();
            let description = record.description.as_deref().unwrap_or("");
            let matches = record
                .agent_id
                .to_lowercase()
                .contains(normalized_query.as_str())
                || record
                    .code
                    .to_lowercase()
                    .contains(normalized_query.as_str())
                || record
                    .display_name
                    .to_lowercase()
                    .contains(normalized_query.as_str())
                || description
                    .to_lowercase()
                    .contains(normalized_query.as_str());
            if !matches {
                return false;
            }
        }
    }
    true
}

/// Sort order for provider bindings: active first, then by updated_at desc,
/// then by binding_id ascending. Encoded in `provider_binding_index_key`.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_turn::AgentTurnMode;

    fn usage_turn(
        id: u64,
        turn_id: &str,
        session_id: &str,
        agent_id: &str,
        model_id: Option<&str>,
        input_tokens: u64,
        output_tokens: u64,
        created_at: &str,
    ) -> AgentTurnRecord {
        AgentTurnRecord {
            id,
            turn_id: turn_id.to_string(),
            tenant_id: 100001,
            organization_id: 0,
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            owner_user_id: 100,
            runtime_binding_id: None,
            client_request_id: None,
            idempotency_key: format!("idem.{turn_id}"),
            payload_hash: "hash".to_string(),
            request_item_id: format!("item.{turn_id}"),
            response_item_id: None,
            turn_mode: AgentTurnMode::Interactive,
            status: crate::agent_turn::AgentTurnStatus::Completed,
            requested_model_id: model_id.map(|value| value.to_string()),
            provider_binding_id: None,
            model_id: model_id.map(|value| value.to_string()),
            provider_id: None,
            input_tokens,
            output_tokens,
            cached_tokens: 0,
            finish_reason: None,
            error_code: None,
            error_detail: None,
            trace_id: None,
            attempt_count: 1,
            max_attempts: 3,
            next_retry_at: None,
            available_at: created_at.to_string(),
            lease_owner: None,
            lease_token: None,
            lease_expires_at: None,
            fencing_token: 0,
            version: 1,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            started_at: None,
            completed_at: None,
            cancel_requested_at: None,
            cancelled_at: None,
            retention_until: None,
        }
    }

    #[test]
    fn usage_summary_and_records_aggregate_turn_facts() {
        let repository = InMemoryAgentRepository::new();
        let turns = [
            usage_turn(
                1,
                "turn.u.1",
                "session.u.1",
                "agent.usage.demo",
                Some("model.a"),
                100,
                20,
                "2026-09-01T10:00:00.000Z",
            ),
            usage_turn(
                2,
                "turn.u.2",
                "session.u.1",
                "agent.usage.demo",
                Some("model.a"),
                50,
                10,
                "2026-09-01T11:00:00.000Z",
            ),
            usage_turn(
                3,
                "turn.u.3",
                "session.u.2",
                "agent.usage.other",
                Some("model.b"),
                7,
                3,
                "2026-09-02T09:00:00.000Z",
            ),
        ];
        {
            let mut map = repository.turns.recovering_write();
            for turn in turns {
                map.insert(
                    (turn.tenant_id, turn.organization_id, turn.turn_id.clone()),
                    turn,
                );
            }
        }

        // Tenant-wide summary over an explicit window that excludes turn 3.
        let summary = repository
            .summarize_usage(
                &crate::usage::UsageSummaryQuery::for_tenant(100001, 0)
                    .with_window("2026-09-01T00:00:00Z", "2026-09-02T00:00:00Z")
                    .validated()
                    .expect("valid window"),
            )
            .expect("summary");
        assert_eq!(summary.turn_count, 2);
        assert_eq!(summary.session_count, 1);
        assert_eq!(summary.input_tokens, 150);
        assert_eq!(summary.output_tokens, 30);

        // Agent filter narrows the aggregate.
        let per_agent = repository
            .summarize_usage(
                &crate::usage::UsageSummaryQuery::for_tenant(100001, 0)
                    .with_agent("agent.usage.other")
                    .validated()
                    .expect("valid"),
            )
            .expect("summary");
        assert_eq!(per_agent.turn_count, 1);
        assert_eq!(per_agent.input_tokens, 7);

        // Records feed: newest first; the store returns page_size + 1 rows so
        // the application layer can detect has_more (same contract as
        // list_turns).
        let page = repository
            .list_usage_records(
                &crate::usage::UsageRecordListQuery::for_tenant(100001, 0)
                    .with_pagination(PaginationParams {
                        page_size: 2,
                        ..PaginationParams::default()
                    })
                    .validated()
                    .expect("valid"),
            )
            .expect("records");
        assert_eq!(page.len(), 3);
        assert_eq!(page[0].turn_id, "turn.u.3");
        assert_eq!(page[1].turn_id, "turn.u.2");
        assert_eq!(page[2].turn_id, "turn.u.1");

        // Model filter only returns the matching model's turn.
        let filtered = repository
            .list_usage_records(
                &crate::usage::UsageRecordListQuery::for_tenant(100001, 0)
                    .with_model("model.b")
                    .validated()
                    .expect("valid"),
            )
            .expect("records");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].turn_id, "turn.u.3");
    }
    use crate::domain::{
        AgentBusinessStatus, AgentImplementationKind, AgentImplementationType,
        AgentProviderBindingRecord, AgentSessionEntrySurface, AgentSessionKind, AgentSessionStatus,
        AgentSessionTitleSource, AgentVisibility,
    };
    use crate::ports::{PaginationParams, ProviderBindingListQuery, MAX_PAGE_SIZE};
    use crate::validation::ID_PREFIX_ITEM;
    use sdkwork_agent_kernel::AgentManifest;
    use sdkwork_agent_kernel::KernelErrorKind;

    fn sample_manifest(agent_id: &str) -> AgentManifest {
        AgentManifest {
            schema_version: "1.0.0".to_string(),
            manifest_type: "agent".to_string(),
            agent_id: agent_id.to_string(),
            name: "sample-agent".to_string(),
            display_name: "Sample Agent".to_string(),
            description: "sample".to_string(),
            version: "0.1.0".to_string(),
            domain: "intelligence".to_string(),
            required_capabilities: vec!["model.chat".to_string()],
            optional_capabilities: vec!["tool.invoke".to_string()],
            required_capability_requirements: vec![],
            optional_capability_requirements: vec![],
            event_families: vec!["agent.lifecycle".to_string()],
            owner_name: "sdkwork".to_string(),
            status: "active".to_string(),
        }
    }

    #[test]
    fn in_memory_repository_rejects_stale_record_version_update() {
        let repository = InMemoryAgentRepository::new();
        let record = AgentBusinessRecord {
            id: 1,
            agent_id: "agent.alpha".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 100,
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: None,
            manifest: sample_manifest("agent.alpha"),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: AgentImplementationType::SdkworkNative,
            status: AgentBusinessStatus::Draft,
            visibility: AgentVisibility::Organization,
            tags: vec!["starter".to_string()],
            version: 1,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            deleted_at: None,
        };
        repository
            .insert(record.clone())
            .expect("initial insert should succeed");

        let mut stale = record.clone();
        stale.display_name = "Alpha stale".to_string();
        let error = repository
            .update(stale)
            .expect_err("stale version should fail");
        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error.message().contains("version mismatch"));
    }

    #[test]
    fn in_memory_repository_records_do_not_panic_after_poisoned_lock() {
        let repository = InMemoryAgentRepository::new();
        let poison_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = repository
                .agents
                .write()
                .expect("test lock should be available");
            panic!("poison in-memory repository lock");
        }));
        assert!(poison_result.is_err());

        let read_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| repository.records()));

        assert!(
            read_result.is_ok(),
            "in-memory repository must not panic after RwLock poisoning"
        );
    }

    fn audit_event(event_id: String, occurred_at: &str) -> KernelEvent {
        KernelEvent::new(
            event_id,
            "agent.business.updated",
            sdkwork_agent_kernel::KernelEventSeverity::Info,
            r#"{"_context":{"agent_id":"agent.audit"}}"#,
        )
        .from_source(sdkwork_agent_kernel::KernelEventSource::Runtime)
        .occurred_at(occurred_at)
    }

    #[test]
    fn in_memory_audit_sink_rejects_invalid_occurred_at() {
        let sink = InMemoryAgentAuditSink::new();
        let error = sink
            .record(audit_event("event.invalid".to_string(), "not-a-timestamp"))
            .expect_err("invalid audit timestamps must fail closed");

        assert_eq!(error.kind(), KernelErrorKind::ValidationError);
        assert!(sink.events().is_empty());
    }

    #[test]
    fn in_memory_audit_sink_evicts_oldest_event_at_capacity() {
        let sink = InMemoryAgentAuditSink::new();
        sink.record(audit_event(
            "event.oldest".to_string(),
            "2026-05-31T23:59:59Z",
        ))
        .expect("oldest event should be accepted");

        for index in 0..MAX_IN_MEMORY_AUDIT_EVENTS {
            sink.record(audit_event(
                format!("event.current.{index:05}"),
                "2026-06-01T00:00:00Z",
            ))
            .expect("current event should be accepted");
        }

        let events = sink.events();
        assert_eq!(events.len(), MAX_IN_MEMORY_AUDIT_EVENTS);
        assert!(events.iter().all(|event| event.event_id != "event.oldest"));
        assert!(events
            .iter()
            .any(|event| event.event_id == "event.current.09999"));
    }

    #[test]
    fn allow_all_policy_provider_fails_closed_without_panic_in_production_like_bypass() {
        let _guard = sdkwork_agents_contract::env_test_lock();
        let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();
        let previous_deploy = std::env::var("SDKWORK_DEPLOYMENT_ENV").ok();
        let previous_environment = std::env::var("ENVIRONMENT").ok();
        let previous_agents_env = std::env::var("SDKWORK_AGENTS_ENVIRONMENT").ok();
        let previous_profile = std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE").ok();

        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_CONFIG_PROFILE");
        std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");
        std::env::set_var("SDKWORK_DEPLOYMENT_ENV", "production");

        let provider_result = std::panic::catch_unwind(|| {
            AllowAllPolicyProvider::allow("policy.agents.misconfigured")
        });

        restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
        restore_optional_env("SDKWORK_DEPLOYMENT_ENV", previous_deploy);
        restore_optional_env("ENVIRONMENT", previous_environment);
        restore_optional_env("SDKWORK_AGENTS_ENVIRONMENT", previous_agents_env);
        restore_optional_env("SDKWORK_AGENTS_CONFIG_PROFILE", previous_profile);

        let provider = provider_result
            .expect("AllowAllPolicyProvider::allow must fail closed instead of panicking");
        assert!(
            matches!(provider.mode, PolicyMode::Deny(_)),
            "misconfigured AllowAllPolicyProvider must deny instead of allowing"
        );
    }

    #[test]
    fn in_memory_repository_rejects_stale_provider_binding_update() {
        let repository = InMemoryAgentRepository::new();
        let record = AgentProviderBindingRecord {
            id: 101,
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id: "provider.rig-rust".to_string(),
            implementation_kind: AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id: "profile.rig.local".to_string(),
            capabilities: vec!["model.chat".to_string()],
            active: true,
            version: 1,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
        };
        repository
            .insert_provider_binding(record.clone())
            .expect("initial binding insert should succeed");

        let mut stale = record.clone();
        stale.provider_id = "provider.rig-alt".to_string();
        let error = repository
            .update_provider_binding(stale)
            .expect_err("stale binding version should fail");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("provider binding version mismatch"));
    }

    #[test]
    fn in_memory_repository_rejects_second_active_provider_binding() {
        let repository = InMemoryAgentRepository::new();
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 102,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.local".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:00:00Z".to_string(),
            })
            .expect("first active binding insert should succeed");

        let error = repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 103,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            })
            .expect_err("second active binding should fail");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("active provider binding already exists"));
    }

    #[test]
    fn in_memory_repository_rejects_update_that_creates_second_active_provider_binding() {
        let repository = InMemoryAgentRepository::new();
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 104,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.local".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:00:00Z".to_string(),
            })
            .expect("first active binding insert should succeed");
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 105,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            })
            .expect("inactive binding insert should succeed");

        let error = repository
            .update_provider_binding(AgentProviderBindingRecord {
                id: 105,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 2,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            })
            .expect_err("update cannot create a second active binding");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("active provider binding already exists"));
    }

    #[test]
    fn in_memory_repository_lists_provider_bindings_in_standard_order() {
        let repository = InMemoryAgentRepository::new();
        for record in [
            AgentProviderBindingRecord {
                id: 106,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.beta".to_string(),
                provider_id: "provider.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.beta".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            },
            AgentProviderBindingRecord {
                id: 107,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.default".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            },
            AgentProviderBindingRecord {
                id: 108,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alpha".to_string(),
                provider_id: "provider.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alpha".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            },
        ] {
            repository
                .insert_provider_binding(record)
                .expect("binding insert should succeed");
        }

        let binding_ids: Vec<String> = repository
            .list_provider_bindings(
                &ProviderBindingListQuery::for_agent(100_001, "agent.alpha")
                    .with_pagination(PaginationParams::default().with_page_size(MAX_PAGE_SIZE)),
            )
            .expect("binding list should succeed")
            .into_iter()
            .map(|record| record.binding_id)
            .collect();

        assert_eq!(
            binding_ids,
            vec![
                "binding.rig.default".to_string(),
                "binding.rig.alpha".to_string(),
                "binding.rig.beta".to_string()
            ]
        );
    }

    // --- IamGatedPolicyProvider tests ---

    use sdkwork_agent_kernel::{PolicyDecisionValue, PolicySubject};

    fn policy_request_with_action_and_roles(action: &str, roles: &[&str]) -> PolicyRequest {
        let mut subject = PolicySubject::new("user.test", "100001");
        for role in roles {
            subject = subject.with_role(*role);
        }
        PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_subject(subject)
        .with_action(action)
    }

    #[test]
    fn iam_gated_provider_allows_read_action_with_read_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("list", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_resource_qualified_read_actions_with_read_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "agent_engine.list",
            "project.list",
            "project.retrieve",
            "session.user_state.list",
            "audit.read",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.read"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Allow,
                "{action} must require only ai.agents.read"
            );
        }
    }

    #[test]
    fn iam_gated_provider_allows_read_action_with_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("retrieve", &["ai.agents.manage"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_self_service_actions_with_use_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "create",
            "project.create",
            "project.update",
            "project.archive",
            "project.delete",
            "project.composition_slot.create",
            "project.composition_slot.update",
            "project.composition_slot.delete",
            "session.create",
            "session.update",
            "session.delete",
            "session.close",
            "session.archive",
            "session.user_state.update",
            "session_item.create",
            "item_feedback.update",
            "turn.create",
            "turn.cancel",
            "task.create",
            "task.cancel",
            "task.execute",
            "interaction.create",
            "interaction.claim",
            "interaction.approve",
            "interaction.answer",
            "interaction.resolve",
            "checkpoint.create",
            "checkpoint.restore",
            "checkpoint.invalidate",
            "session_runtime_binding.create",
            "session_runtime_binding.update",
            "session_runtime_binding.activate",
            "session_runtime_binding.deactivate",
            "runtime.preview_response",
            "runtime.prompt_optimization",
            "workspace.ensureDefault",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.use"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Allow,
                "{action} must require ai.agents.use"
            );
        }
    }

    #[test]
    fn iam_gated_provider_keeps_management_actions_behind_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "update",
            "delete",
            "change_status",
            "provider_binding.add",
            "provider_binding.activate",
            "composition_slot.create",
            "session.unclassified_mutation",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.use"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Deny,
                "{action} must remain restricted to ai.agents.manage"
            );
            assert!(decision
                .safe_reason
                .as_deref()
                .unwrap_or_default()
                .contains("iam.permission.missing:ai.agents.manage"));
        }
    }

    #[test]
    fn iam_gated_provider_denies_manage_action_with_only_read_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("update", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.manage"));
    }

    #[test]
    fn iam_gated_provider_allows_manage_action_with_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in ["update", "project.create", "session.create"] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.manage"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(decision.decision, PolicyDecisionValue::Allow);
        }
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_ai_wildcard() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "list",
            "retrieve",
            "create",
            "update",
            "delete",
            "change_status",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.*"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Allow,
                "action {action} should be allowed with ai.* wildcard"
            );
        }
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_global_wildcard() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("delete", &["*"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_org_admin_role() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("delete", &["org_admin"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_org_operations_role() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("create", &["org_operations"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_denies_when_subject_has_no_permissions() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("list", &[]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn iam_gated_provider_denies_when_subject_is_missing() {
        let provider = IamGatedPolicyProvider::default();
        let request = PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_action("list");
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn iam_gated_provider_denies_unrecognized_permission_scope() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("list", &["iam.users.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn iam_gated_provider_treats_unknown_action_as_manage() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("unknown_action", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    // --- Self-service action tests ---

    #[test]
    fn iam_gated_provider_denies_self_service_action_without_use_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("project.create", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.use"));
    }

    #[test]
    fn iam_gated_provider_allows_create_with_use_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("create", &["ai.agents.use"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_denies_self_service_action_without_subject() {
        let provider = IamGatedPolicyProvider::default();
        let request = PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_action("project.create");
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.use"));
    }

    #[test]
    fn iam_gated_provider_treats_missing_action_as_manage() {
        let provider = IamGatedPolicyProvider::default();
        let request = PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_subject(PolicySubject::new("user.test", "100001").with_role("ai.agents.use"));
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.manage"));
    }

    #[test]
    fn iam_gated_provider_denies_change_status_without_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        // change_status (activation) must still require ai.agents.manage,
        // preserving the review workflow while draft creation requires use permission.
        let request = policy_request_with_action_and_roles("change_status", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.manage"));
    }

    // --- DenyAllPolicyProvider tests ---

    #[test]
    fn deny_all_provider_denies_every_request() {
        let provider = DenyAllPolicyProvider::default();
        let request =
            policy_request_with_action_and_roles("list", &["ai.agents.manage", "ai.*", "*"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn deny_all_provider_default_has_descriptive_reason() {
        let provider = DenyAllPolicyProvider::default();
        assert!(!provider.reason.is_empty());
        assert!(!provider.provider_id.is_empty());
    }

    fn cursor_session_item(id: u64, sequence: u64) -> AgentSessionItemRecord {
        AgentSessionItemRecord {
            id,
            item_id: format!("{ID_PREFIX_ITEM}cursor.{id}"),
            tenant_id: 10,
            organization_id: 20,
            session_id: "session.cursor".to_string(),
            kind: crate::domain::AgentSessionItemKind::AssistantOutput,
            content: Some(format!("message {sequence}")),
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            sequence,
            input_tokens: 0,
            output_tokens: 0,
            model_id: None,
            provider_id: None,
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: None,
            turn_id: None,
            created_by: 30,
            version: 0,
            created_at: "2026-07-30T09:00:00Z".to_string(),
            updated_at: "2026-07-30T09:00:00Z".to_string(),
            completed_at: Some("2026-07-30T09:00:00Z".to_string()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        }
    }

    fn insert_cursor_session_item(
        repository: &InMemoryAgentRepository,
        record: AgentSessionItemRecord,
    ) {
        let primary_key = session_item_primary_key(&record);
        repository
            .session_item_index
            .recovering_write()
            .insert(session_item_index_key(&record), primary_key.clone());
        repository
            .items
            .recovering_write()
            .insert(primary_key, record);
    }

    #[test]
    fn session_item_cursor_keyset_is_stable_when_new_head_items_arrive() {
        let repository = InMemoryAgentRepository::new();
        for sequence in 1..=5 {
            insert_cursor_session_item(&repository, cursor_session_item(100 + sequence, sequence));
        }
        let first_query = SessionItemListQuery::for_session(10, 20, "session.cursor")
            .with_sort(SessionItemListSort::SequenceDesc)
            .with_cursor_page(2, None);
        let first = repository.list_session_items(&first_query).unwrap();
        assert_eq!(
            first.iter().map(|item| item.sequence).collect::<Vec<_>>(),
            vec![5, 4, 3]
        );

        let boundary = &first[1];
        let cursor = crate::session_item_cursor::SessionItemCursor {
            sequence: boundary.sequence,
            item_internal_id: boundary.id,
            scope_fingerprint: first_query.cursor_scope_fingerprint(),
        };
        insert_cursor_session_item(&repository, cursor_session_item(106, 6));

        let second = repository
            .list_session_items(&first_query.with_cursor_page(2, Some(cursor)))
            .unwrap();
        assert_eq!(
            second.iter().map(|item| item.sequence).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
    }

    fn activity_session(id: u64, session_id: &str, updated_at: &str) -> AgentSessionRecord {
        AgentSessionRecord {
            id,
            session_id: session_id.to_string(),
            tenant_id: 10,
            organization_id: 20,
            agent_id: "agent.activity".to_string(),
            owner_user_id: 30,
            project_id: None,
            session_kind: crate::domain::AgentSessionKind::Coding,
            entry_surface: crate::domain::AgentSessionEntrySurface::Pc,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some(session_id.to_string()),
            title_source: crate::domain::AgentSessionTitleSource::System,
            status: crate::domain::AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: None,
            payload_hash: None,
            created_by: 30,
            updated_by: 30,
            version: 0,
            created_at: updated_at.to_string(),
            updated_at: updated_at.to_string(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        }
    }

    fn activity_turn(
        id: u64,
        turn_id: &str,
        status: AgentTurnStatus,
        updated_at: &str,
    ) -> AgentTurnRecord {
        AgentTurnRecord {
            id,
            turn_id: turn_id.to_string(),
            tenant_id: 10,
            organization_id: 20,
            session_id: "session.activity.turn-order".to_string(),
            agent_id: "agent.activity".to_string(),
            owner_user_id: 30,
            runtime_binding_id: None,
            client_request_id: None,
            idempotency_key: format!("idempotency.{turn_id}"),
            payload_hash: format!("sha256:{turn_id}"),
            request_item_id: format!("{ID_PREFIX_ITEM}request.{id}"),
            response_item_id: None,
            turn_mode: crate::agent_turn::AgentTurnMode::Interactive,
            status,
            requested_model_id: None,
            provider_binding_id: None,
            model_id: None,
            provider_id: None,
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            finish_reason: None,
            error_code: None,
            error_detail: None,
            trace_id: None,
            attempt_count: 0,
            max_attempts: 1,
            next_retry_at: None,
            available_at: updated_at.to_string(),
            lease_owner: None,
            lease_token: None,
            lease_expires_at: (status == AgentTurnStatus::Running)
                .then(|| "2099-07-27T13:00:00Z".to_string()),
            fencing_token: 0,
            version: 0,
            created_at: updated_at.to_string(),
            updated_at: updated_at.to_string(),
            started_at: None,
            completed_at: None,
            cancel_requested_at: None,
            cancelled_at: None,
            retention_until: None,
        }
    }

    #[test]
    fn canonical_latest_turn_uses_creation_identity_not_update_recency() {
        let repository = InMemoryAgentRepository::new();
        let session = activity_session(1, "session.activity.turn-order", "2099-07-27T09:00:00Z");
        repository.insert_session(session.clone()).unwrap();
        let mut older = activity_turn(
            2,
            "turn.activity.older",
            AgentTurnStatus::Completed,
            "2099-07-27T09:01:00Z",
        );
        let latest = activity_turn(
            3,
            "turn.activity.latest",
            AgentTurnStatus::Running,
            "2099-07-27T09:02:00Z",
        );
        {
            let mut turns = repository.turns.recovering_write();
            turns.insert((10, 20, older.turn_id.clone()), older.clone());
            turns.insert((10, 20, latest.turn_id.clone()), latest);
        }
        repository.advance_session_activity(
            &session,
            "2099-07-27T09:02:00Z",
            SessionActivitySource::Turn,
        );
        older.updated_at = "2099-07-27T09:03:00Z".to_string();
        older.version = older.version.saturating_add(1);
        repository
            .turns
            .recovering_write()
            .insert((10, 20, older.turn_id.clone()), older);
        repository.advance_session_activity(
            &session,
            "2099-07-27T09:03:00Z",
            SessionActivitySource::Turn,
        );

        let page = repository
            .list_session_activity_summaries(&SessionActivitySummaryListQuery::for_owner(
                10, 20, 30,
            ))
            .unwrap();
        assert_eq!(
            page.items[0]
                .latest_turn
                .as_ref()
                .map(|turn| turn.turn_id.as_str()),
            Some("turn.activity.latest")
        );
        assert_eq!(
            page.items[0].presentation_phase,
            crate::session_activity::SessionPresentationPhase::Running
        );
        assert_eq!(page.items[0].freshness.activity_at, "2099-07-27T09:03:00Z");
    }

    #[test]
    fn session_activity_snapshot_is_newest_first_and_cursor_is_stable() {
        let repository = InMemoryAgentRepository::new();
        repository
            .insert_session(activity_session(
                1,
                "session.activity.old",
                "2099-07-27T09:00:00Z",
            ))
            .unwrap();
        repository
            .insert_session(activity_session(
                2,
                "session.activity.tie-low",
                "2099-07-27T10:00:00Z",
            ))
            .unwrap();
        repository
            .insert_session(activity_session(
                3,
                "session.activity.tie-high",
                "2099-07-27T10:00:00Z",
            ))
            .unwrap();

        let query = SessionActivitySummaryListQuery::for_owner(10, 20, 30).with_page_size(2);
        let first = repository.list_session_activity_summaries(&query).unwrap();
        assert_eq!(
            first
                .items
                .iter()
                .map(|item| item.session.id)
                .collect::<Vec<_>>(),
            vec![3, 2]
        );
        assert!(first.has_more);
        let first_cursor = first.next_page_token.expect("continuation cursor");

        let cursor = crate::session_activity::decode_session_activity_cursor(&first_cursor)
            .expect("decode cursor");
        let second = repository
            .list_session_activity_summaries(&query.clone().after(cursor))
            .unwrap();
        assert_eq!(
            second
                .items
                .iter()
                .map(|item| item.session.id)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert!(!second.has_more);
        assert!(second.next_page_token.is_none());

        let exhausted_cursor = crate::session_activity::SessionActivityCursor {
            activity_at: second.items[0].freshness.activity_at.clone(),
            session_internal_id: second.items[0].session.id,
            scope_fingerprint: query.scope_fingerprint(),
        };
        let exhausted = repository
            .list_session_activity_summaries(&query.clone().after(exhausted_cursor))
            .unwrap();
        assert!(exhausted.items.is_empty());
        assert!(exhausted.next_page_token.is_none());

        repository
            .insert_session(activity_session(
                4,
                "session.activity.new-head",
                "2099-07-27T11:00:00Z",
            ))
            .unwrap();
        let refreshed = repository.list_session_activity_summaries(&query).unwrap();
        assert_eq!(refreshed.items[0].session.id, 4);

        repository
            .upsert_resource_user_state(
                AgentResourceUserStateRecord {
                    id: 5,
                    tenant_id: 10,
                    organization_id: 20,
                    user_id: 30,
                    resource_type: AgentResourceType::Session,
                    resource_id: "session.activity.old".to_string(),
                    pinned_at: Some("2099-07-27T12:00:00Z".to_string()),
                    hidden_at: None,
                    last_opened_at: None,
                    last_read_item_sequence: Some(0),
                    custom_title: Some("Pinned from another app".to_string()),
                    version: 0,
                    created_at: "2099-07-27T12:00:00Z".to_string(),
                    updated_at: "2099-07-27T12:00:00Z".to_string(),
                },
                None,
            )
            .unwrap();
        let user_state_refreshed = repository.list_session_activity_summaries(&query).unwrap();
        assert_eq!(user_state_refreshed.items[0].session.id, 1);
        assert_eq!(
            user_state_refreshed.items[0].freshness.source,
            SessionActivitySource::UserState
        );
        assert_eq!(
            user_state_refreshed.items[0].freshness.user_state_version,
            Some(0)
        );
        assert_eq!(
            user_state_refreshed.items[0]
                .user_state
                .as_ref()
                .and_then(|state| state.custom_title.as_deref()),
            Some("Pinned from another app")
        );
    }

    #[test]
    fn in_memory_activity_index_normalizes_offsets_and_rejects_head_regression() {
        let repository = InMemoryAgentRepository::new();
        let offset_session =
            activity_session(1, "session.activity.offset", "2099-07-27T17:00:00+08:00");
        repository.insert_session(offset_session.clone()).unwrap();
        repository
            .insert_session(activity_session(
                2,
                "session.activity.utc",
                "2099-07-27T09:30:00Z",
            ))
            .unwrap();
        let query = SessionActivitySummaryListQuery::for_owner(10, 20, 30);
        let first = repository.list_session_activity_summaries(&query).unwrap();
        assert_eq!(first.items[0].session.id, 2);
        assert_eq!(
            first
                .items
                .iter()
                .find(|item| item.session.id == 1)
                .map(|item| item.freshness.activity_at.as_str()),
            Some("2099-07-27T09:00:00Z")
        );

        let mut regressed = offset_session;
        regressed.version = 1;
        regressed.updated_at = "2099-07-27T08:00:00Z".to_string();
        repository.update_session(regressed).unwrap();
        let after_regression = repository.list_session_activity_summaries(&query).unwrap();
        assert_eq!(after_regression.items[0].session.id, 2);
        assert_eq!(
            after_regression
                .items
                .iter()
                .find(|item| item.session.id == 1)
                .map(|item| item.freshness.activity_at.as_str()),
            Some("2099-07-27T09:00:00Z")
        );
    }

    #[test]
    fn pending_approval_precedes_newer_pending_question() {
        let repository = InMemoryAgentRepository::new();
        repository
            .insert_session(activity_session(
                1,
                "session.activity.interaction",
                "2099-07-27T09:00:00Z",
            ))
            .unwrap();
        let interaction =
            |id, interaction_id: &str, kind, updated_at: &str| AgentInteractionRecord {
                id,
                interaction_id: interaction_id.to_string(),
                tenant_id: 10,
                organization_id: 20,
                session_id: "session.activity.interaction".to_string(),
                turn_id: None,
                runtime_binding_id: None,
                provider_interaction_id: None,
                kind,
                status: crate::domain::AgentInteractionStatus::Pending,
                prompt: interaction_id.to_string(),
                request_json: None,
                options_json: "[]".to_string(),
                resolution_json: None,
                claim_owner: None,
                claim_token_hash: None,
                claim_expires_at: None,
                fencing_token: 0,
                version: 0,
                created_at: updated_at.to_string(),
                updated_at: updated_at.to_string(),
                resolved_at: None,
                retention_until: None,
            };
        repository
            .insert_interaction(interaction(
                1,
                "interaction.approval",
                AgentInteractionKind::Approval,
                "2099-07-27T09:01:00Z",
            ))
            .unwrap();
        repository
            .insert_interaction(interaction(
                2,
                "interaction.question",
                AgentInteractionKind::UserQuestion,
                "2099-07-27T09:02:00Z",
            ))
            .unwrap();

        let page = repository
            .list_session_activity_summaries(&SessionActivitySummaryListQuery::for_owner(
                10, 20, 30,
            ))
            .unwrap();
        assert_eq!(
            page.items[0]
                .pending_interaction
                .as_ref()
                .map(|interaction| interaction.kind),
            Some(AgentInteractionKind::Approval)
        );
        assert_eq!(
            page.items[0].presentation_phase,
            crate::session_activity::SessionPresentationPhase::AwaitingInput
        );
        assert!(page.items[0].freshness.fresh_until.is_none());

        let mut approval = repository
            .get_interaction(
                10,
                20,
                "session.activity.interaction",
                "interaction.approval",
            )
            .unwrap()
            .unwrap();
        approval.resolve(
            crate::domain::AgentInteractionStatus::Resolved,
            "{}",
            "2099-07-27T09:03:00Z",
        );
        repository.update_interaction(approval).unwrap();
        let mut question = repository
            .get_interaction(
                10,
                20,
                "session.activity.interaction",
                "interaction.question",
            )
            .unwrap()
            .unwrap();
        question.resolve(
            crate::domain::AgentInteractionStatus::Resolved,
            "{}",
            "2099-07-27T09:04:00Z",
        );
        repository.update_interaction(question).unwrap();

        let resolved_page = repository
            .list_session_activity_summaries(&SessionActivitySummaryListQuery::for_owner(
                10, 20, 30,
            ))
            .unwrap();
        assert!(resolved_page.items[0].pending_interaction.is_none());
        assert_eq!(
            resolved_page.items[0]
                .freshness
                .latest_interaction_id
                .as_deref(),
            Some("interaction.question")
        );
        assert_eq!(
            resolved_page.items[0].freshness.latest_interaction_version,
            Some(1)
        );
        assert_eq!(
            resolved_page.items[0].freshness.source,
            SessionActivitySource::Interaction
        );
    }

    #[test]
    fn latest_failed_runtime_binding_is_preserved_and_drives_failed_phase() {
        let repository = InMemoryAgentRepository::new();
        repository
            .insert_session(activity_session(
                1,
                "session.activity.failed-binding",
                "2099-07-27T09:00:00Z",
            ))
            .unwrap();
        repository
            .insert_session_runtime_binding(AgentSessionRuntimeBindingRecord {
                id: 2,
                tenant_id: 10,
                organization_id: 20,
                owner_user_id: 30,
                session_id: "session.activity.failed-binding".to_string(),
                runtime_binding_id: "runtime_binding.activity.failed".to_string(),
                runtime_location_id: None,
                host_mode: "managed".to_string(),
                transport_kind: "in_process".to_string(),
                provider_binding_id: "binding.activity.failed".to_string(),
                model_id: "model.activity.failed".to_string(),
                provider_id: "provider.activity.failed".to_string(),
                provider_session_id: None,
                provider_session_tree_id: None,
                provider_parent_session_id: None,
                provider_forked_from_session_id: None,
                provider_title: None,
                provider_title_source: None,
                provider_preview: None,
                provider_created_at: None,
                provider_updated_at: None,
                provider_recency_at: None,
                provider_pinned: false,
                provider_archived: false,
                provider_visible: false,
                provider_sort_key: None,
                provider_source: None,
                status: AgentSessionRuntimeBindingStatus::Failed,
                is_current: false,
                version: 1,
                created_at: "2099-07-27T09:00:00Z".to_string(),
                updated_at: "2099-07-27T09:01:00Z".to_string(),
                activated_at: Some("2099-07-27T09:00:00Z".to_string()),
                deactivated_at: None,
            })
            .unwrap();

        let page = repository
            .list_session_activity_summaries(&SessionActivitySummaryListQuery::for_owner(
                10, 20, 30,
            ))
            .unwrap();
        assert!(page.items[0].current_runtime_binding.is_none());
        assert_eq!(
            page.items[0]
                .latest_runtime_binding
                .as_ref()
                .map(|binding| binding.status),
            Some(AgentSessionRuntimeBindingStatus::Failed)
        );
        assert_eq!(
            page.items[0].presentation_phase,
            crate::session_activity::SessionPresentationPhase::Failed
        );

        let mut binding = repository
            .get_session_runtime_binding(
                10,
                20,
                "session.activity.failed-binding",
                "runtime_binding.activity.failed",
            )
            .unwrap()
            .unwrap();
        binding.deactivate(
            AgentSessionRuntimeBindingStatus::Deactivated,
            "2099-07-27T09:02:00Z",
        );
        repository.update_session_runtime_binding(binding).unwrap();
        let deactivated_page = repository
            .list_session_activity_summaries(&SessionActivitySummaryListQuery::for_owner(
                10, 20, 30,
            ))
            .unwrap();
        assert!(deactivated_page.items[0].current_runtime_binding.is_none());
        assert_eq!(
            deactivated_page.items[0]
                .freshness
                .latest_runtime_binding_id
                .as_deref(),
            Some("runtime_binding.activity.failed")
        );
        assert_eq!(
            deactivated_page.items[0]
                .freshness
                .latest_runtime_binding_version,
            Some(2)
        );
        assert_eq!(
            deactivated_page.items[0].freshness.source,
            SessionActivitySource::RuntimeBinding
        );
        assert_ne!(
            deactivated_page.items[0].presentation_phase,
            crate::session_activity::SessionPresentationPhase::Failed
        );
    }

    fn restore_optional_env(key: &str, value: Option<String>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    fn interrupted_turn_fixture() -> (InMemoryAgentRepository, AgentTurnRecord) {
        let repository = InMemoryAgentRepository::new();
        let session = AgentSessionRecord {
            id: 9001,
            session_id: "session.promote".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            agent_id: "agent.alpha".to_string(),
            owner_user_id: 100,
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Pc,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: None,
            title_source: AgentSessionTitleSource::Provider,
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: None,
            payload_hash: None,
            created_by: 100,
            updated_by: 100,
            version: 1,
            created_at: "2026-09-04T00:00:00Z".to_string(),
            updated_at: "2026-09-04T00:00:00Z".to_string(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        repository
            .insert_session(session.clone())
            .expect("session insert should succeed");

        let turn = AgentTurnRecord {
            id: 9101,
            turn_id: "turn.promote".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            session_id: session.session_id.clone(),
            agent_id: session.agent_id.clone(),
            owner_user_id: session.owner_user_id,
            runtime_binding_id: None,
            client_request_id: None,
            idempotency_key: "idem.promote".to_string(),
            payload_hash: "hash.promote".to_string(),
            request_item_id: "item.request.promote".to_string(),
            response_item_id: None,
            turn_mode: AgentTurnMode::Interactive,
            status: AgentTurnStatus::Requested,
            requested_model_id: Some("model.promote".to_string()),
            provider_binding_id: None,
            model_id: None,
            provider_id: None,
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            finish_reason: None,
            error_code: None,
            error_detail: None,
            trace_id: None,
            attempt_count: 0,
            max_attempts: 3,
            next_retry_at: None,
            available_at: "2026-09-04T00:00:00Z".to_string(),
            lease_owner: None,
            lease_token: None,
            lease_expires_at: None,
            fencing_token: 0,
            version: 0,
            created_at: "2026-09-04T00:00:00Z".to_string(),
            updated_at: "2026-09-04T00:00:00Z".to_string(),
            started_at: None,
            completed_at: None,
            cancel_requested_at: None,
            cancelled_at: None,
            retention_until: None,
        };
        let user_input_item = AgentSessionItemRecord {
            id: 9201,
            item_id: turn.request_item_id.clone(),
            tenant_id: turn.tenant_id,
            organization_id: turn.organization_id,
            session_id: turn.session_id.clone(),
            kind: AgentSessionItemKind::UserInput,
            content: Some("hello".to_string()),
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            sequence: 0,
            input_tokens: 0,
            output_tokens: 0,
            model_id: None,
            provider_id: None,
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: None,
            turn_id: Some(turn.turn_id.clone()),
            created_by: turn.owner_user_id,
            version: 0,
            created_at: "2026-09-04T00:00:00Z".to_string(),
            updated_at: "2026-09-04T00:00:00Z".to_string(),
            completed_at: Some("2026-09-04T00:00:00Z".to_string()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };
        repository
            .insert_turn_request(turn.clone(), user_input_item, Vec::new())
            .expect("turn request insert should succeed");
        (repository, turn)
    }

    fn partial_output_record(turn: &AgentTurnRecord) -> AgentSessionItemRecord {
        AgentSessionItemRecord {
            id: 9301,
            item_id: "item.partial.promote".to_string(),
            tenant_id: turn.tenant_id,
            organization_id: turn.organization_id,
            session_id: turn.session_id.clone(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: None,
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            sequence: 0,
            input_tokens: 0,
            output_tokens: 0,
            model_id: turn.model_id.clone(),
            provider_id: turn.provider_id.clone(),
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: Some(turn.request_item_id.clone()),
            turn_id: Some(turn.turn_id.clone()),
            created_by: turn.owner_user_id,
            version: 0,
            created_at: "2026-09-04T00:01:00Z".to_string(),
            updated_at: "2026-09-04T00:01:00Z".to_string(),
            completed_at: Some("2026-09-04T00:01:00Z".to_string()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        }
    }

    #[test]
    fn in_memory_repository_promotes_interrupted_turn_output_once() {
        let (repository, mut turn) = interrupted_turn_fixture();
        turn.mark_running("2026-09-04T00:00:30Z");
        let running_turn = repository
            .update_turn_state(turn, 0)
            .expect("running state update should succeed");
        repository
            .append_turn_streaming_content(
                running_turn.tenant_id,
                running_turn.organization_id,
                &running_turn.turn_id,
                "partial answer so far",
                "2026-09-04T00:00:40Z",
            )
            .expect("streaming checkpoint should be accepted while running");
        let mut failed_turn_input = running_turn;
        failed_turn_input.mark_failed(
            "turn_inference_failed",
            "managed turn inference failed",
            "2026-09-04T00:01:00Z",
        );
        let failed_turn = repository
            .update_turn_state(failed_turn_input, 1)
            .expect("failed state update should succeed");

        let promoted = repository
            .append_interrupted_turn_output(partial_output_record(&failed_turn))
            .expect("promotion should not error");
        let promoted = promoted.expect("checkpointed turn output should promote");

        assert_eq!(promoted.kind, AgentSessionItemKind::AssistantOutput);
        assert_eq!(
            promoted.content.as_deref(),
            Some("partial answer so far"),
            "promoted content must come from the turn checkpoint"
        );
        assert_eq!(promoted.sequence, 2, "sequence follows the request item");
        assert_eq!(
            promoted.parent_item_id.as_deref(),
            Some(failed_turn.request_item_id.as_str())
        );
        assert_eq!(
            repository
                .read_turn_streaming_content(
                    failed_turn.tenant_id,
                    failed_turn.organization_id,
                    &failed_turn.turn_id,
                )
                .expect("checkpoint read should not error"),
            None,
            "checkpoint must be consumed by the promotion"
        );

        // Idempotent: a second promotion for the same turn is a no-op.
        let replay = repository
            .append_interrupted_turn_output(partial_output_record(&failed_turn))
            .expect("replay should not error");
        assert!(replay.is_none(), "promotion must happen at most once");

        let items = repository
            .list_session_items_by_turn(
                failed_turn.tenant_id,
                failed_turn.organization_id,
                &failed_turn.session_id,
                &failed_turn.turn_id,
                50,
            )
            .expect("turn item listing should not error");
        let assistant_items = items
            .iter()
            .filter(|item| item.kind == AgentSessionItemKind::AssistantOutput)
            .count();
        assert_eq!(assistant_items, 1, "exactly one partial assistant item");
    }

    #[test]
    fn in_memory_repository_skips_promotion_without_checkpoint() {
        let (repository, mut turn) = interrupted_turn_fixture();
        turn.mark_running("2026-09-04T00:00:30Z");
        let running_turn = repository
            .update_turn_state(turn, 0)
            .expect("running state update should succeed");
        let mut failed_turn_input = running_turn;
        failed_turn_input.mark_failed("turn_inference_failed", "failed", "2026-09-04T00:01:00Z");
        let failed_turn = repository
            .update_turn_state(failed_turn_input, 1)
            .expect("failed state update should succeed");

        let promoted = repository
            .append_interrupted_turn_output(partial_output_record(&failed_turn))
            .expect("promotion should not error");
        assert!(promoted.is_none(), "no checkpoint means nothing to promote");
    }

    #[test]
    fn in_memory_repository_skips_promotion_for_running_turn() {
        let (repository, mut turn) = interrupted_turn_fixture();
        turn.mark_running("2026-09-04T00:00:30Z");
        let running_turn = repository
            .update_turn_state(turn, 0)
            .expect("running state update should succeed");
        repository
            .append_turn_streaming_content(
                running_turn.tenant_id,
                running_turn.organization_id,
                &running_turn.turn_id,
                "still streaming",
                "2026-09-04T00:00:40Z",
            )
            .expect("streaming checkpoint should be accepted while running");

        let promoted = repository
            .append_interrupted_turn_output(partial_output_record(&running_turn))
            .expect("promotion should not error");
        assert!(
            promoted.is_none(),
            "running turns are never promoted (only failed/cancelled terminal states)"
        );
    }
}
