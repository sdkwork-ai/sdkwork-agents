use std::sync::Arc;

use async_trait::async_trait;
use sdkwork_agent_kernel::{KernelErrorKind, KernelResult, PolicySubject};
use sdkwork_intelligence_agents_service::{
    AgentTaskRunRecord, AgentTaskWorkerHandle, ClaimTaskRunsRequest, MaterializeDueTasksRequest,
    TaskRunClaim, TaskRunLease, TaskRunReconciliationResult, TaskSchedulerMetricsSnapshot,
};
use sdkwork_utils_rust::{format_datetime, now};
use tokio::sync::watch;
use tokio::task::JoinSet;
use tokio::time::{Instant, MissedTickBehavior};

use crate::{SchedulerWorkerConfig, SchedulerWorkerControl, SchedulerWorkerMetrics};

#[async_trait]
pub trait TaskWorkerClient: Clone + Send + Sync + 'static {
    async fn materialize_due_tasks(
        &self,
        request: MaterializeDueTasksRequest,
    ) -> KernelResult<Vec<AgentTaskRunRecord>>;

    async fn claim_task_runs(
        &self,
        request: ClaimTaskRunsRequest,
    ) -> KernelResult<Vec<TaskRunClaim>>;

    async fn heartbeat_task_run(
        &self,
        lease: TaskRunLease,
        heartbeat_at: String,
        lease_seconds: u32,
    ) -> KernelResult<AgentTaskRunRecord>;

    async fn recover_expired_task_run_leases(&self, now: String, limit: usize)
        -> KernelResult<u64>;

    /// Recovers Runs whose configured execution budget elapsed while they
    /// were still claimed or running (independent of heartbeat liveness).
    async fn recover_timed_out_task_runs(&self, now: String, limit: usize)
        -> KernelResult<u64>;

    async fn scheduler_metrics_snapshot(
        &self,
        _now: String,
    ) -> KernelResult<TaskSchedulerMetricsSnapshot> {
        Ok(TaskSchedulerMetricsSnapshot::default())
    }

    async fn reconcile_task_runs(
        &self,
        updated_before: String,
        occurred_at: String,
        limit: usize,
    ) -> KernelResult<TaskRunReconciliationResult>;

    /// Runs one transactional-outbox dispatch round (webhook delivery +
    /// agent-call recovery). The default is a no-op for test clients.
    async fn dispatch_outbox_events(
        &self,
        _worker_id: String,
    ) -> KernelResult<sdkwork_intelligence_agents_service::OutboxDispatchSummary> {
        Ok(sdkwork_intelligence_agents_service::OutboxDispatchSummary::default())
    }

    async fn execute_task_run_claim(
        &self,
        claim: TaskRunClaim,
        requested_by: PolicySubject,
        requested_at: String,
    ) -> KernelResult<AgentTaskRunRecord>;
}

#[async_trait]
impl TaskWorkerClient for AgentTaskWorkerHandle {
    async fn materialize_due_tasks(
        &self,
        request: MaterializeDueTasksRequest,
    ) -> KernelResult<Vec<AgentTaskRunRecord>> {
        AgentTaskWorkerHandle::materialize_due_tasks(self, request).await
    }

    async fn claim_task_runs(
        &self,
        request: ClaimTaskRunsRequest,
    ) -> KernelResult<Vec<TaskRunClaim>> {
        AgentTaskWorkerHandle::claim_task_runs(self, request).await
    }

    async fn heartbeat_task_run(
        &self,
        lease: TaskRunLease,
        heartbeat_at: String,
        lease_seconds: u32,
    ) -> KernelResult<AgentTaskRunRecord> {
        AgentTaskWorkerHandle::heartbeat_task_run(self, lease, heartbeat_at, lease_seconds).await
    }

    async fn recover_expired_task_run_leases(
        &self,
        now: String,
        limit: usize,
    ) -> KernelResult<u64> {
        AgentTaskWorkerHandle::recover_expired_task_run_leases(self, now, limit).await
    }

    async fn recover_timed_out_task_runs(
        &self,
        now: String,
        limit: usize,
    ) -> KernelResult<u64> {
        AgentTaskWorkerHandle::recover_timed_out_task_runs(self, now, limit).await
    }

    async fn scheduler_metrics_snapshot(
        &self,
        now: String,
    ) -> KernelResult<TaskSchedulerMetricsSnapshot> {
        AgentTaskWorkerHandle::scheduler_metrics_snapshot(self, now).await
    }

    async fn reconcile_task_runs(
        &self,
        updated_before: String,
        occurred_at: String,
        limit: usize,
    ) -> KernelResult<TaskRunReconciliationResult> {
        AgentTaskWorkerHandle::reconcile_task_runs(self, updated_before, occurred_at, limit).await
    }

    async fn execute_task_run_claim(
        &self,
        claim: TaskRunClaim,
        requested_by: PolicySubject,
        requested_at: String,
    ) -> KernelResult<AgentTaskRunRecord> {
        AgentTaskWorkerHandle::execute_task_run_claim(self, claim, requested_by, requested_at).await
    }

    async fn dispatch_outbox_events(
        &self,
        worker_id: String,
    ) -> KernelResult<sdkwork_intelligence_agents_service::OutboxDispatchSummary> {
        AgentTaskWorkerHandle::dispatch_outbox_events(self, worker_id).await
    }
}

pub async fn run_scheduler_worker<C>(
    client: C,
    config: SchedulerWorkerConfig,
    control: Arc<SchedulerWorkerControl>,
    metrics: Arc<SchedulerWorkerMetrics>,
    mut shutdown: watch::Receiver<bool>,
) where
    C: TaskWorkerClient,
{
    let mut materialize = tokio::time::interval(config.materialize_interval);
    let mut claim = tokio::time::interval(config.claim_interval);
    let mut recover = tokio::time::interval(config.recovery_interval);
    let mut metrics_snapshot = tokio::time::interval(config.metrics_snapshot_interval);
    let mut reconcile = tokio::time::interval(config.reconciliation_interval);
    let mut outbox_dispatch = tokio::time::interval(config.outbox_dispatch_interval);
    materialize.set_missed_tick_behavior(MissedTickBehavior::Skip);
    claim.set_missed_tick_behavior(MissedTickBehavior::Skip);
    recover.set_missed_tick_behavior(MissedTickBehavior::Skip);
    metrics_snapshot.set_missed_tick_behavior(MissedTickBehavior::Skip);
    reconcile.set_missed_tick_behavior(MissedTickBehavior::Skip);
    outbox_dispatch.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut executions = JoinSet::new();
    let mut reconciliations = JoinSet::new();
    let mut outbox_rounds = JoinSet::new();
    control.mark_started();
    let mut materialize_failures = 0u32;
    let mut claim_failures = 0u32;
    let mut recover_failures = 0u32;

    loop {
        tokio::select! {
            biased;
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            Some(result) = executions.join_next(), if !executions.is_empty() => {
                if let Err(error) = result {
                    metrics.record_operation_error();
                    tracing::error!(error = %error, "task execution worker join failed");
                }
            }
            Some(result) = reconciliations.join_next(), if !reconciliations.is_empty() => {
                if let Err(error) = result {
                    metrics.record_operation_error();
                    tracing::error!(error = %error, "task run reconciliation worker join failed");
                }
            }
            Some(result) = outbox_rounds.join_next(), if !outbox_rounds.is_empty() => {
                match result {
                    Ok(Ok(summary)) => {
                        metrics.record_outbox_dispatch(&summary);
                        if summary.claimed > 0 {
                            tracing::debug!(
                                claimed = summary.claimed,
                                delivered = summary.delivered,
                                retried = summary.retried,
                                dead_lettered = summary.dead_lettered,
                                unmatched = summary.unmatched,
                                "outbox dispatch round completed"
                            );
                        }
                    }
                    Ok(Err(error)) => {
                        metrics.record_operation_error();
                        tracing::error!(error = %error, "outbox dispatch round failed");
                    }
                    Err(error) => {
                        metrics.record_operation_error();
                        tracing::error!(error = %error, "outbox dispatch worker join failed");
                    }
                }
            }
            _ = materialize.tick() => {
                let request = MaterializeDueTasksRequest::bounded(
                    current_time(),
                    config.materialize_batch_size,
                );
                let started_at = Instant::now();
                match client.materialize_due_tasks(request).await {
                    Ok(runs) => {
                        materialize_failures = 0;
                        metrics.record_materialization(runs.len(), started_at.elapsed());
                    }
                    Err(error) => {
                        metrics.record_operation_error();
                        materialize_failures = materialize_failures.saturating_add(1);
                        materialize.reset_at(
                            tokio::time::Instant::now() + poll_backoff(materialize_failures),
                        );
                        tracing::error!(error = %error, "task occurrence materialization failed");
                    }
                }
            }
            _ = recover.tick() => {
                match client
                    .recover_expired_task_run_leases(current_time(), config.recovery_batch_size)
                    .await
                {
                    Ok(count) => {
                        recover_failures = 0;
                        metrics.add_recovered(count);
                    }
                    Err(error) => {
                        metrics.record_operation_error();
                        recover_failures = recover_failures.saturating_add(1);
                        recover.reset_at(
                            tokio::time::Instant::now() + poll_backoff(recover_failures),
                        );
                        tracing::error!(error = %error, "expired task run lease recovery failed");
                    }
                }
                // Enforce configured execution budgets: a Run that is still
                // heartbeating past its timeout_at must be reclaimed even
                // though its lease is alive.
                match client
                    .recover_timed_out_task_runs(current_time(), config.recovery_batch_size)
                    .await
                {
                    Ok(count) => metrics.add_timed_out_recovered(count),
                    Err(error) => {
                        metrics.record_operation_error();
                        tracing::error!(error = %error, "timed-out task run recovery failed");
                    }
                }
            }
            _ = outbox_dispatch.tick(), if outbox_rounds.is_empty() => {
                // Bounded overlap guard: one round at a time, delivery
                // latency (bounded HTTP timeouts) must not stack rounds.
                let client = client.clone();
                let metrics = metrics.clone();
                let worker_id = config.worker_id.clone();
                outbox_rounds.spawn(async move {
                    client.dispatch_outbox_events(worker_id).await
                });
            }
            _ = metrics_snapshot.tick() => {
                match client.scheduler_metrics_snapshot(current_time()).await {
                    Ok(snapshot) => metrics.record_snapshot(snapshot),
                    Err(error) => {
                        metrics.record_operation_error();
                        tracing::error!(error = %error, "task scheduler metrics snapshot failed");
                    }
                }
            }
            _ = reconcile.tick(), if reconciliations.is_empty() => {                let (updated_before, occurred_at) = reconciliation_window(
                    config.reconciliation_min_age,
                );
                let client = client.clone();
                let metrics = metrics.clone();
                let limit = config.reconciliation_batch_size;
                reconciliations.spawn(async move {
                    match client
                        .reconcile_task_runs(updated_before, occurred_at, limit)
                        .await
                    {
                        Ok(result) => metrics.record_reconciliation(&result),
                        Err(error) => {
                            metrics.record_operation_error();
                            tracing::error!(error = %error, "task run reconciliation failed");
                        }
                    }
                });
            }
            _ = claim.tick() => {
                let capacity = config.max_concurrency.saturating_sub(executions.len());
                if capacity == 0 {
                    continue;
                }
                let request = ClaimTaskRunsRequest::bounded_with_tenant_limit(
                    config.worker_id.clone(),
                    current_time(),
                    config.lease_seconds,
                    capacity.min(config.claim_batch_size),
                    config.tenant_max_concurrency,
                );
                let started_at = Instant::now();
                match client.claim_task_runs(request).await {
                    Ok(claims) => {
                        claim_failures = 0;
                        metrics.record_claim(claims.len(), started_at.elapsed());
                        for claim in claims.into_iter().take(capacity) {
                            metrics.execution_started();
                            executions.spawn(execute_with_heartbeats(
                                client.clone(),
                                claim,
                                config.lease_seconds,
                                config.heartbeat_interval,
                                metrics.clone(),
                            ));
                        }
                    }
                    Err(error) => {
                        metrics.record_operation_error();
                        claim_failures = claim_failures.saturating_add(1);
                        claim.reset_at(
                            tokio::time::Instant::now() + poll_backoff(claim_failures),
                        );
                        tracing::error!(error = %error, "task run claim failed");
                    }
                }
            }
        }
    }

    control.begin_draining();
    reconciliations.abort_all();
    tracing::info!(
        inflight = executions.len(),
        drain_timeout_seconds = config.drain_timeout.as_secs(),
        "task worker stopped claiming and started graceful drain"
    );
    let deadline = Instant::now() + config.drain_timeout;
    while !executions.is_empty() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            metrics.record_forced_drain();
            executions.abort_all();
            tracing::warn!(
                inflight = executions.len(),
                "task worker drain timeout elapsed; remaining leases will be recovered"
            );
            break;
        }
        match tokio::time::timeout(remaining, executions.join_next()).await {
            Ok(Some(Err(error))) => {
                metrics.record_operation_error();
                tracing::error!(error = %error, "task execution worker join failed during drain");
            }
            Ok(Some(Ok(()))) => {}
            Ok(None) => break,
            Err(_) => {
                metrics.record_forced_drain();
                executions.abort_all();
                tracing::warn!(
                    inflight = executions.len(),
                    "task worker drain timeout elapsed; remaining leases will be recovered"
                );
                break;
            }
        }
    }
}

async fn execute_with_heartbeats<C>(
    client: C,
    claim: TaskRunClaim,
    lease_seconds: u32,
    heartbeat_interval: std::time::Duration,
    metrics: Arc<SchedulerWorkerMetrics>,
) where
    C: TaskWorkerClient,
{
    let lease = claim.lease.clone();
    let subject = PolicySubject::new(
        claim.run.owner_user_id.to_string(),
        claim.run.tenant_id.to_string(),
    )
    .with_role("ai.agents.use");
    let execution = client.execute_task_run_claim(claim, subject, current_time());
    let started_at = Instant::now();
    tokio::pin!(execution);
    let mut heartbeat =
        tokio::time::interval_at(Instant::now() + heartbeat_interval, heartbeat_interval);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut lease_lost = false;

    let result = loop {
        tokio::select! {
            result = &mut execution => break result,
            _ = heartbeat.tick(), if !lease_lost => {
                match client
                    .heartbeat_task_run(lease.clone(), current_time(), lease_seconds)
                    .await
                {
                    Ok(_) => metrics.record_heartbeat(true, false),
                    Err(error) => {
                        lease_lost = true;
                        let fencing_rejected = error.kind() == KernelErrorKind::Conflict;
                        metrics.record_heartbeat(false, fencing_rejected);
                        tracing::error!(
                            error = %error,
                            "task run lease heartbeat failed; stale completion will be fenced"
                        );
                    }
                }
            }
        }
    };
    metrics.execution_finished(&result, started_at.elapsed());
    if let Err(error) = result {
        tracing::error!(error = %error, "task run execution failed");
    }
}

fn current_time() -> String {
    format_datetime(now(), None)
}

fn reconciliation_window(min_age: std::time::Duration) -> (String, String) {
    let occurred_at = now();
    let age_seconds = i64::try_from(min_age.as_secs()).unwrap_or(i64::MAX);
    let updated_before = occurred_at
        .checked_sub_signed(chrono::Duration::seconds(age_seconds))
        .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC);
    (
        format_datetime(updated_before, None),
        format_datetime(occurred_at, None),
    )
}

/// Exponential backoff for consecutive poll failures (250ms base doubled per
/// failure, capped at 30s) so a database outage cannot hammer the pool at
/// the base polling rate.
fn poll_backoff(consecutive_failures: u32) -> std::time::Duration {
    const BASE: std::time::Duration = std::time::Duration::from_millis(250);
    const CAP: std::time::Duration = std::time::Duration::from_secs(30);
    let multiplier = 1u32
        .checked_shl(consecutive_failures.min(12))
        .unwrap_or(4096);
    BASE.saturating_mul(multiplier).min(CAP)
}
