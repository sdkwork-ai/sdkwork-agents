use std::sync::Arc;

use chrono::Duration;
use sdkwork_agent_kernel::{
    KernelError, KernelErrorKind, KernelResult, PolicyProvider, PolicySubject,
};
use sdkwork_utils_rust::{format_datetime, parse_datetime, sha256_hash};

use crate::agent_turn::AgentTurnMode;
use crate::application::{AgentsService, CreateTurnCommand, TurnExecutionResult};
use crate::ports::{AgentAuditSink, AgentRepository, MAX_PAGE_SIZE};
use crate::task_execution_cursor::{
    task_run_attempt_scope_fingerprint, task_run_scope_fingerprint, TaskRunAttemptCursor,
    TaskRunCursor,
};
use crate::task_scheduling::{
    AgentTaskMisfirePolicy, AgentTaskRecord, AgentTaskRunStatus, AgentTaskScheduleKind,
    AgentTaskStatus, AgentTaskTriggerKind,
};
use crate::task_scheduling::{AgentTaskRunAttemptRecord, AgentTaskRunRecord};

pub const DEFAULT_MATERIALIZE_BATCH_SIZE: usize = 100;
pub const MAX_MATERIALIZE_BATCH_SIZE: usize = 1_000;
pub const DEFAULT_CLAIM_BATCH_SIZE: usize = 32;
pub const MAX_CLAIM_BATCH_SIZE: usize = 256;
pub const DEFAULT_RUN_LEASE_SECONDS: u32 = 60;
pub const MAX_RUN_LEASE_SECONDS: u32 = 300;
pub const DEFAULT_TENANT_CONCURRENT_RUNS: usize = 64;
pub const MAX_TENANT_CONCURRENT_RUNS: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializeDueTasksRequest {
    pub now: String,
    pub limit: usize,
}

impl MaterializeDueTasksRequest {
    pub fn bounded(now: impl Into<String>, limit: usize) -> Self {
        Self {
            now: now.into(),
            limit: limit.clamp(1, MAX_MATERIALIZE_BATCH_SIZE),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimTaskRunsRequest {
    pub worker_id: String,
    pub now: String,
    pub lease_seconds: u32,
    pub limit: usize,
    pub max_concurrent_runs_per_tenant: usize,
}

impl ClaimTaskRunsRequest {
    pub fn bounded(
        worker_id: impl Into<String>,
        now: impl Into<String>,
        lease_seconds: u32,
        limit: usize,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            now: now.into(),
            lease_seconds: lease_seconds.clamp(1, MAX_RUN_LEASE_SECONDS),
            limit: limit.clamp(1, MAX_CLAIM_BATCH_SIZE),
            max_concurrent_runs_per_tenant: DEFAULT_TENANT_CONCURRENT_RUNS,
        }
    }

    pub fn bounded_with_tenant_limit(
        worker_id: impl Into<String>,
        now: impl Into<String>,
        lease_seconds: u32,
        limit: usize,
        max_concurrent_runs_per_tenant: usize,
    ) -> Self {
        let mut request = Self::bounded(worker_id, now, lease_seconds, limit);
        request.max_concurrent_runs_per_tenant =
            max_concurrent_runs_per_tenant.clamp(1, MAX_TENANT_CONCURRENT_RUNS);
        request
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunLease {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub run_id: String,
    pub attempt_id: String,
    pub worker_id: String,
    pub lease_token: String,
    pub fencing_token: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunClaim {
    pub run: AgentTaskRunRecord,
    pub attempt: AgentTaskRunAttemptRecord,
    pub lease: TaskRunLease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunFailureDisposition {
    Retry,
    Reconcile,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailTaskRunRequest {
    pub lease: TaskRunLease,
    pub disposition: TaskRunFailureDisposition,
    pub error_code: String,
    pub failure_class: String,
    pub retry_at: Option<String>,
    pub failed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileTaskRunRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub run_id: String,
    pub expected_version: u64,
    pub terminal_status: AgentTaskRunStatus,
    pub error_code: Option<String>,
    pub reconciled_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskTransitionResult {
    pub task: AgentTaskRecord,
    pub cancelled_pending_run_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TaskSchedulerMetricsSnapshot {
    pub due_tasks: u64,
    pub materialization_lag_seconds: u64,
    pub eligible_runs: u64,
    pub eligible_run_oldest_age_seconds: u64,
    pub active_leases: u64,
    pub reconciling_runs: u64,
    pub reconciliation_oldest_age_seconds: u64,
    pub pending_outbox_events: u64,
    pub outbox_oldest_age_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub task_id: String,
    pub owner_user_id: Option<u64>,
    pub status: Option<AgentTaskRunStatus>,
    pub trigger_kind: Option<AgentTaskTriggerKind>,
    pub cursor: Option<TaskRunCursor>,
    pub page_size: usize,
}

impl TaskRunListQuery {
    pub fn for_task(tenant_id: u64, organization_id: u64, task_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            organization_id,
            task_id: task_id.into(),
            owner_user_id: None,
            status: None,
            trigger_kind: None,
            cursor: None,
            page_size: crate::ports::DEFAULT_PAGE_SIZE,
        }
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_status(mut self, status: AgentTaskRunStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_trigger_kind(mut self, trigger_kind: AgentTaskTriggerKind) -> Self {
        self.trigger_kind = Some(trigger_kind);
        self
    }

    pub fn with_cursor_page(mut self, page_size: usize, cursor: Option<TaskRunCursor>) -> Self {
        self.page_size = page_size.clamp(1, MAX_PAGE_SIZE);
        self.cursor = cursor;
        self
    }

    pub fn scope_fingerprint(&self) -> String {
        task_run_scope_fingerprint(
            self.tenant_id,
            self.organization_id,
            &self.task_id,
            self.owner_user_id,
            self.status.map(AgentTaskRunStatus::as_str),
            self.trigger_kind.map(AgentTaskTriggerKind::as_str),
        )
    }

    pub fn store_limit(&self) -> usize {
        self.page_size.saturating_add(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunAttemptListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub run_id: String,
    pub cursor: Option<TaskRunAttemptCursor>,
    pub page_size: usize,
}

impl TaskRunAttemptListQuery {
    pub fn for_run(tenant_id: u64, organization_id: u64, run_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            organization_id,
            run_id: run_id.into(),
            cursor: None,
            page_size: crate::ports::DEFAULT_PAGE_SIZE,
        }
    }

    pub fn with_cursor_page(
        mut self,
        page_size: usize,
        cursor: Option<TaskRunAttemptCursor>,
    ) -> Self {
        self.page_size = page_size.clamp(1, MAX_PAGE_SIZE);
        self.cursor = cursor;
        self
    }

    pub fn scope_fingerprint(&self) -> String {
        task_run_attempt_scope_fingerprint(self.tenant_id, self.organization_id, &self.run_id)
    }

    pub fn store_limit(&self) -> usize {
        self.page_size.saturating_add(1)
    }
}

pub trait TaskSchedulerRepository: Send + Sync {
    /// Atomically applies a Task generation transition and cancels pending Runs
    /// that belong to older generations.
    fn transition_task(
        &self,
        task: AgentTaskRecord,
        cancellation_reason: &str,
    ) -> KernelResult<TaskTransitionResult>;

    fn create_manual_task_run(
        &self,
        task: &AgentTaskRecord,
        idempotency_key: &str,
        requested_at: &str,
    ) -> KernelResult<AgentTaskRunRecord>;

    fn create_business_retry_task_run(
        &self,
        task: &AgentTaskRecord,
        retry_of: &AgentTaskRunRecord,
        idempotency_key: &str,
        requested_at: &str,
    ) -> KernelResult<AgentTaskRunRecord>;

    fn materialize_due_tasks(
        &self,
        request: &MaterializeDueTasksRequest,
    ) -> KernelResult<Vec<AgentTaskRunRecord>>;

    fn claim_task_runs(&self, request: &ClaimTaskRunsRequest) -> KernelResult<Vec<TaskRunClaim>>;

    fn mark_task_run_running(
        &self,
        lease: &TaskRunLease,
        started_at: &str,
    ) -> KernelResult<AgentTaskRunRecord>;

    fn heartbeat_task_run(
        &self,
        lease: &TaskRunLease,
        heartbeat_at: &str,
        lease_seconds: u32,
    ) -> KernelResult<AgentTaskRunRecord>;

    fn complete_task_run(
        &self,
        lease: &TaskRunLease,
        turn_id: &str,
        completed_at: &str,
    ) -> KernelResult<AgentTaskRunRecord>;

    fn fail_task_run(&self, request: &FailTaskRunRequest) -> KernelResult<AgentTaskRunRecord>;

    fn recover_expired_task_run_leases(&self, now: &str, limit: usize) -> KernelResult<u64>;

    /// Recovers Runs whose configured execution budget (`timeout_at`) elapsed
    /// while the Run was still claimed or running. Unlike lease recovery this
    /// does not depend on heartbeats stopping: a Run whose provider keeps
    /// heartbeating past its budget must still be reclaimed. The fenced Run
    /// goes back to `Pending` (or `DeadLetter` when attempts are exhausted)
    /// exactly like a lost lease, so a stale worker's completion is fenced.
    fn recover_timed_out_task_runs(&self, now: &str, limit: usize) -> KernelResult<u64>;

    fn scheduler_metrics_snapshot(&self, _now: &str) -> KernelResult<TaskSchedulerMetricsSnapshot> {
        Ok(TaskSchedulerMetricsSnapshot::default())
    }

    /// Pending Runs are cancelled immediately. Claimed or running Runs move to
    /// reconciling until the canonical Turn/provider outcome is known.
    fn request_task_run_cancellation(
        &self,
        tenant_id: u64,
        organization_id: u64,
        run_id: &str,
        expected_version: Option<u64>,
        requested_at: &str,
    ) -> KernelResult<AgentTaskRunRecord>;

    fn reconcile_task_run(
        &self,
        request: &ReconcileTaskRunRequest,
    ) -> KernelResult<AgentTaskRunRecord>;

    fn list_reconciling_task_runs(
        &self,
        updated_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTaskRunRecord>>;

    fn list_task_runs(&self, query: &TaskRunListQuery) -> KernelResult<Vec<AgentTaskRunRecord>>;

    fn list_task_run_attempts(
        &self,
        query: &TaskRunAttemptListQuery,
    ) -> KernelResult<Vec<AgentTaskRunAttemptRecord>>;

    fn get_task_run(
        &self,
        tenant_id: u64,
        organization_id: u64,
        run_id: &str,
    ) -> KernelResult<Option<AgentTaskRunRecord>>;
}

pub trait TaskTurnExecutor: Send + Sync {
    fn execute_task_turn(&self, command: CreateTurnCommand) -> KernelResult<TurnExecutionResult>;
}

impl<R, A, P> TaskTurnExecutor for AgentsService<R, A, P>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider + Send + Sync,
{
    fn execute_task_turn(&self, command: CreateTurnCommand) -> KernelResult<TurnExecutionResult> {
        self.execute_turn(command)
    }
}

pub struct AgentTaskScheduler {
    scheduler_repository: Arc<dyn TaskSchedulerRepository>,
    agent_repository: Arc<dyn AgentRepository>,
    turn_executor: Arc<dyn TaskTurnExecutor>,
}

impl AgentTaskScheduler {
    pub fn new(
        scheduler_repository: Arc<dyn TaskSchedulerRepository>,
        agent_repository: Arc<dyn AgentRepository>,
        turn_executor: Arc<dyn TaskTurnExecutor>,
    ) -> Self {
        Self {
            scheduler_repository,
            agent_repository,
            turn_executor,
        }
    }

    pub fn materialize_due(
        &self,
        now: impl Into<String>,
        limit: usize,
    ) -> KernelResult<Vec<AgentTaskRunRecord>> {
        self.scheduler_repository
            .materialize_due_tasks(&MaterializeDueTasksRequest::bounded(now, limit))
    }

    pub fn claim(
        &self,
        worker_id: impl Into<String>,
        now: impl Into<String>,
        lease_seconds: u32,
        limit: usize,
    ) -> KernelResult<Vec<TaskRunClaim>> {
        self.scheduler_repository
            .claim_task_runs(&ClaimTaskRunsRequest::bounded(
                worker_id,
                now,
                lease_seconds,
                limit,
            ))
    }

    pub fn execute_claim(
        &self,
        claim: &TaskRunClaim,
        requested_by: PolicySubject,
        requested_at: impl Into<String>,
    ) -> KernelResult<AgentTaskRunRecord> {
        execute_task_run_claim(
            self.scheduler_repository.as_ref(),
            self.agent_repository.as_ref(),
            self.turn_executor.as_ref(),
            claim,
            requested_by,
            requested_at.into(),
        )
    }
}

pub(crate) fn execute_task_run_claim(
    scheduler_repository: &dyn TaskSchedulerRepository,
    agent_repository: &dyn AgentRepository,
    turn_executor: &dyn TaskTurnExecutor,
    claim: &TaskRunClaim,
    requested_by: PolicySubject,
    requested_at: String,
) -> KernelResult<AgentTaskRunRecord> {
    let running = scheduler_repository.mark_task_run_running(&claim.lease, &requested_at)?;
    let Some(task) =
        agent_repository.get_task(running.tenant_id, running.organization_id, &running.task_id)?
    else {
        return scheduler_repository.fail_task_run(&FailTaskRunRequest {
            lease: claim.lease.clone(),
            disposition: TaskRunFailureDisposition::Terminal,
            error_code: "task_not_found".to_string(),
            failure_class: "validation".to_string(),
            retry_at: None,
            failed_at: requested_at,
        });
    };
    if task.generation != running.schedule_generation {
        return scheduler_repository.fail_task_run(&FailTaskRunRequest {
            lease: claim.lease.clone(),
            disposition: TaskRunFailureDisposition::Terminal,
            error_code: "stale_task_generation".to_string(),
            failure_class: "fencing_conflict".to_string(),
            retry_at: None,
            failed_at: requested_at,
        });
    }
    let Some(turn_id) = running.turn_id.clone() else {
        return scheduler_repository.fail_task_run(&FailTaskRunRequest {
            lease: claim.lease.clone(),
            disposition: TaskRunFailureDisposition::Terminal,
            error_code: "task_run_turn_missing".to_string(),
            failure_class: "validation".to_string(),
            retry_at: None,
            failed_at: requested_at,
        });
    };
    let result = turn_executor.execute_task_turn(CreateTurnCommand {
        tenant_id: running.tenant_id,
        organization_id: running.organization_id,
        agent_id: running.agent_id.clone(),
        session_id: running.session_id.clone(),
        turn_id: Some(turn_id.clone()),
        content: task.prompt.clone(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Automation,
        runtime_binding_id: None,
        requested_model_id: None,
        access_mode_id: None,
        idempotency_key: running.idempotency_key.clone(),
        payload_hash: running.payload_hash.clone(),
        client_request_id: Some(running.run_id.clone()),
        drive_refs: Vec::new(),
        owner_scope: Some(running.owner_user_id),
        requested_by,
        requested_at: requested_at.clone(),
        prefer_stream: false,
        system_prompt: None,
        auth_token: None,
        access_token: None,
        wire_protocol: None,
    });
    match result {
        Ok(result) => scheduler_repository.complete_task_run(
            &claim.lease,
            &result.turn.turn_id,
            &requested_at,
        ),
        Err(error) => {
            let disposition = failure_disposition(&error);
            let retry_at = (disposition == TaskRunFailureDisposition::Retry).then(|| {
                calculate_retry_at(
                    &requested_at,
                    task.retry_initial_delay_seconds,
                    task.retry_max_delay_seconds,
                    running.attempt_count,
                    &running.run_id,
                )
            });
            scheduler_repository.fail_task_run(&FailTaskRunRequest {
                lease: claim.lease.clone(),
                disposition,
                error_code: error.code().to_string(),
                failure_class: error.kind().as_str().to_string(),
                retry_at: retry_at.transpose()?,
                failed_at: requested_at,
            })
        }
    }
}

fn failure_disposition(error: &KernelError) -> TaskRunFailureDisposition {
    match error.kind() {
        KernelErrorKind::Timeout | KernelErrorKind::ProviderError => {
            TaskRunFailureDisposition::Reconcile
        }
        KernelErrorKind::ProviderUnavailable
        | KernelErrorKind::RateLimited
        | KernelErrorKind::ResourceExhausted
        | KernelErrorKind::InternalError => TaskRunFailureDisposition::Retry,
        _ if error.retryable() => TaskRunFailureDisposition::Retry,
        _ => TaskRunFailureDisposition::Terminal,
    }
}

fn calculate_retry_at(
    failed_at: &str,
    initial_delay_seconds: u32,
    max_delay_seconds: u32,
    attempt_count: u16,
    run_id: &str,
) -> KernelResult<String> {
    let failed_at = parse_datetime(failed_at, None)
        .ok_or_else(|| KernelError::validation("failedAt must be an RFC 3339 instant"))?;
    let exponent = u32::from(attempt_count.saturating_sub(1)).min(20);
    let base = u64::from(initial_delay_seconds)
        .saturating_mul(1_u64 << exponent)
        .min(u64::from(max_delay_seconds));
    let digest = sha256_hash(format!("{run_id}:{attempt_count}").as_bytes());
    let jitter_seed = u64::from_str_radix(&digest[..8], 16).unwrap_or(0);
    let jitter_bound = (base / 5).max(1);
    let delay = base.saturating_add(jitter_seed % jitter_bound);
    Ok(format_datetime(
        failed_at + Duration::seconds(i64::try_from(delay).unwrap_or(i64::MAX)),
        None,
    ))
}

pub(crate) fn task_run_payload_hash(
    task_id: &str,
    session_id: &str,
    generation: u64,
    scheduled_for: &str,
    prompt: &str,
) -> KernelResult<String> {
    let payload = serde_json::to_vec(&serde_json::json!({
        "taskId": task_id,
        "sessionId": session_id,
        "generation": generation.to_string(),
        "scheduledFor": scheduled_for,
        "prompt": prompt,
    }))
    .map_err(|_| KernelError::Internal {
        message: "failed to canonicalize task Run payload".to_string(),
    })?;
    Ok(sha256_hash(&payload))
}

/// Grace window for Skip misfire evaluation.
///
/// Workers materialize occurrences on a polling cadence (default 1s) and the
/// schedule timestamps are millisecond-precision. Without a grace window a
/// `Skip` misfire policy would classify every occurrence that was due between
/// two consecutive polls as "missed" and silently drop it — in practice the
/// policy would never fire. An occurrence is only skipped when it is overdue
/// by more than this window (e.g. the worker was down or the DB unavailable);
/// within the window it still fires, just late.
pub const TASK_MISFIRE_GRACE_SECONDS: i64 = 60;

pub(crate) struct TaskMaterializationPlan {
    pub occurrences: Vec<String>,
    pub next_fire_at: Option<String>,
    pub status: AgentTaskStatus,
    pub completed_at: Option<String>,
}

/// Whether the occurrence is considered missed under a `Skip` policy:
/// only a delay beyond the grace window counts, so the ordinary polling
/// latency of a healthy worker never drops a due occurrence.
fn missed_by_skip_policy(
    first_at: chrono::DateTime<chrono::Utc>,
    now_at: chrono::DateTime<chrono::Utc>,
) -> bool {
    now_at.signed_duration_since(first_at) > chrono::Duration::seconds(TASK_MISFIRE_GRACE_SECONDS)
}

pub(crate) fn plan_task_materialization(
    task: &AgentTaskRecord,
    now: &str,
    occurrence_limit: usize,
    overlap_blocked: bool,
) -> KernelResult<TaskMaterializationPlan> {
    let now_at = parse_datetime(now, None)
        .ok_or_else(|| KernelError::validation("now must be an RFC 3339 instant"))?;
    let first = task
        .next_fire_at
        .clone()
        .ok_or_else(|| KernelError::conflict("active task has no next fire time"))?;
    let first_at = parse_datetime(&first, None)
        .ok_or_else(|| KernelError::validation("nextFireAt is invalid"))?;
    if overlap_blocked {
        let next = match task.schedule_kind {
            AgentTaskScheduleKind::OneTime => None,
            AgentTaskScheduleKind::Cron => task.schedule().next_after(now)?,
        };
        let completed = next.is_none();
        return Ok(TaskMaterializationPlan {
            occurrences: Vec::new(),
            next_fire_at: next,
            status: if completed {
                AgentTaskStatus::Completed
            } else {
                AgentTaskStatus::Active
            },
            completed_at: completed.then(|| now.to_string()),
        });
    }

    if task.schedule_kind == AgentTaskScheduleKind::OneTime {
        // Skip only drops occurrences that are overdue beyond the grace
        // window; a due occurrence that the poll just missed still runs.
        let should_run = !(task.misfire_policy == AgentTaskMisfirePolicy::Skip
            && missed_by_skip_policy(first_at, now_at));
        return Ok(TaskMaterializationPlan {
            occurrences: should_run.then_some(first).into_iter().collect(),
            next_fire_at: None,
            status: AgentTaskStatus::Completed,
            completed_at: Some(now.to_string()),
        });
    }

    match task.misfire_policy {
        AgentTaskMisfirePolicy::Skip if missed_by_skip_policy(first_at, now_at) => {
            let next = task.schedule().next_after(now)?;
            let completed = next.is_none();
            Ok(TaskMaterializationPlan {
                occurrences: Vec::new(),
                next_fire_at: next,
                status: if completed {
                    AgentTaskStatus::Completed
                } else {
                    AgentTaskStatus::Active
                },
                completed_at: completed.then(|| now.to_string()),
            })
        }
        AgentTaskMisfirePolicy::Skip | AgentTaskMisfirePolicy::FireOnce => {
            let next = task.schedule().next_after(now)?;
            let completed = next.is_none();
            Ok(TaskMaterializationPlan {
                occurrences: vec![first],
                next_fire_at: next,
                status: if completed {
                    AgentTaskStatus::Completed
                } else {
                    AgentTaskStatus::Active
                },
                completed_at: completed.then(|| now.to_string()),
            })
        }
        AgentTaskMisfirePolicy::CatchUp => {
            let mut occurrences = Vec::new();
            let mut cursor = first;
            for _ in 0..occurrence_limit.max(1) {
                let cursor_at = parse_datetime(&cursor, None)
                    .ok_or_else(|| KernelError::validation("task occurrence is invalid"))?;
                if cursor_at > now_at {
                    break;
                }
                occurrences.push(cursor.clone());
                let Some(next) = task.schedule().next_after(&cursor)? else {
                    cursor.clear();
                    break;
                };
                cursor = next;
            }
            let next = (!cursor.is_empty()).then_some(cursor);
            let completed = next.is_none();
            Ok(TaskMaterializationPlan {
                occurrences,
                next_fire_at: next,
                status: if completed {
                    AgentTaskStatus::Completed
                } else {
                    AgentTaskStatus::Active
                },
                completed_at: completed.then(|| now.to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_backoff_is_deterministic_and_capped() {
        let first =
            calculate_retry_at("2026-07-31T00:00:00.000Z", 5, 60, 8, "run.1").expect("retry");
        let second =
            calculate_retry_at("2026-07-31T00:00:00.000Z", 5, 60, 8, "run.1").expect("retry");
        assert_eq!(first, second);
        let retry = parse_datetime(&first, None).expect("retry timestamp");
        let failed = parse_datetime("2026-07-31T00:00:00.000Z", None).expect("failed timestamp");
        assert!((retry - failed).num_seconds() >= 60);
        assert!((retry - failed).num_seconds() < 72);
    }

    #[test]
    fn task_run_hash_changes_with_occurrence() {
        let first = task_run_payload_hash("task.1", "session.1", 1, "a", "prompt").expect("hash");
        let second = task_run_payload_hash("task.1", "session.1", 1, "b", "prompt").expect("hash");
        assert_ne!(first, second);
    }
}
