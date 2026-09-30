use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use sdkwork_intelligence_agents_service::TaskSchedulerMetricsSnapshot;
use sdkwork_utils_rust::{diff_millis, parse_datetime};

#[derive(Default)]
pub struct SchedulerWorkerMetrics {
    materialized_total: AtomicU64,
    claimed_total: AtomicU64,
    outbox_events_claimed_total: AtomicU64,
    outbox_deliveries_sent_total: AtomicU64,
    outbox_events_published_total: AtomicU64,
    outbox_events_retried_total: AtomicU64,
    outbox_events_dead_lettered_total: AtomicU64,
    recovered_leases_total: AtomicU64,
    recovered_timeouts_total: AtomicU64,
    retries_total: AtomicU64,
    fencing_rejections_total: AtomicU64,
    reconciliation_examined_total: AtomicU64,
    reconciliation_reconciled_total: AtomicU64,
    reconciliation_pending_total: AtomicU64,
    reconciliation_conflicts_total: AtomicU64,
    heartbeats_total: AtomicU64,
    heartbeat_failures_total: AtomicU64,
    executions_succeeded_total: AtomicU64,
    executions_failed_total: AtomicU64,
    executions_reconciling_total: AtomicU64,
    executions_dead_letter_total: AtomicU64,
    executions_cancelled_total: AtomicU64,
    operation_errors_total: AtomicU64,
    forced_drain_total: AtomicU64,
    inflight: AtomicU64,
    materialization_duration_micros: AtomicU64,
    materialization_operations: AtomicU64,
    claim_duration_micros: AtomicU64,
    claim_operations: AtomicU64,
    execution_duration_micros: AtomicU64,
    execution_operations: AtomicU64,
    run_latency_micros: AtomicU64,
    completed_runs: AtomicU64,
    due_tasks: AtomicU64,
    materialization_lag_seconds: AtomicU64,
    eligible_runs: AtomicU64,
    eligible_run_oldest_age_seconds: AtomicU64,
    active_leases: AtomicU64,
    reconciling_runs: AtomicU64,
    reconciliation_oldest_age_seconds: AtomicU64,
    pending_outbox_events: AtomicU64,
    outbox_oldest_age_seconds: AtomicU64,
}

impl SchedulerWorkerMetrics {
    pub(crate) fn record_materialization(&self, count: usize, elapsed: Duration) {
        add(&self.materialized_total, count as u64);
        record_duration(
            &self.materialization_duration_micros,
            &self.materialization_operations,
            elapsed,
        );
    }

    pub(crate) fn record_claim(&self, count: usize, elapsed: Duration) {
        add(&self.claimed_total, count as u64);
        record_duration(&self.claim_duration_micros, &self.claim_operations, elapsed);
    }

    pub(crate) fn add_recovered(&self, count: u64) {
        add(&self.recovered_leases_total, count);
    }

    pub(crate) fn add_timed_out_recovered(&self, count: u64) {
        add(&self.recovered_timeouts_total, count);
    }

    /// Records one transactional-outbox dispatch round's outcome.
    pub(crate) fn record_outbox_dispatch(
        &self,
        summary: &sdkwork_intelligence_agents_service::OutboxDispatchSummary,
    ) {
        add(&self.outbox_events_claimed_total, summary.claimed as u64);
        add(
            &self.outbox_deliveries_sent_total,
            summary.deliveries_sent as u64,
        );
        add(&self.outbox_events_published_total, summary.delivered as u64);
        add(&self.outbox_events_retried_total, summary.retried as u64);
        add(
            &self.outbox_events_dead_lettered_total,
            summary.dead_lettered as u64,
        );
    }

    pub(crate) fn record_snapshot(&self, snapshot: TaskSchedulerMetricsSnapshot) {
        set(&self.due_tasks, snapshot.due_tasks);
        set(
            &self.materialization_lag_seconds,
            snapshot.materialization_lag_seconds,
        );
        set(&self.eligible_runs, snapshot.eligible_runs);
        set(
            &self.eligible_run_oldest_age_seconds,
            snapshot.eligible_run_oldest_age_seconds,
        );
        set(&self.active_leases, snapshot.active_leases);
        set(&self.reconciling_runs, snapshot.reconciling_runs);
        set(
            &self.reconciliation_oldest_age_seconds,
            snapshot.reconciliation_oldest_age_seconds,
        );
        set(&self.pending_outbox_events, snapshot.pending_outbox_events);
        set(
            &self.outbox_oldest_age_seconds,
            snapshot.outbox_oldest_age_seconds,
        );
    }

    pub(crate) fn record_reconciliation(
        &self,
        result: &sdkwork_intelligence_agents_service::TaskRunReconciliationResult,
    ) {
        add(&self.reconciliation_examined_total, result.examined as u64);
        add(
            &self.reconciliation_reconciled_total,
            result.reconciled.len() as u64,
        );
        add(&self.reconciliation_pending_total, result.pending as u64);
        add(
            &self.reconciliation_conflicts_total,
            result.skipped_conflicts as u64,
        );
        for run in &result.reconciled {
            self.record_run_latency(run);
        }
    }

    pub(crate) fn record_heartbeat(&self, succeeded: bool, fencing_rejected: bool) {
        add(&self.heartbeats_total, 1);
        if !succeeded {
            add(&self.heartbeat_failures_total, 1);
        }
        if fencing_rejected {
            add(&self.fencing_rejections_total, 1);
        }
    }

    pub(crate) fn execution_started(&self) {
        add(&self.inflight, 1);
    }

    pub(crate) fn execution_finished(
        &self,
        result: &sdkwork_agent_kernel::KernelResult<
            sdkwork_intelligence_agents_service::AgentTaskRunRecord,
        >,
        elapsed: Duration,
    ) {
        subtract(&self.inflight, 1);
        record_duration(
            &self.execution_duration_micros,
            &self.execution_operations,
            elapsed,
        );
        use sdkwork_intelligence_agents_service::AgentTaskRunStatus;
        match result.as_ref() {
            Ok(run) => match run.status {
                AgentTaskRunStatus::Succeeded => {
                    add(&self.executions_succeeded_total, 1);
                    self.record_run_latency(run);
                }
                AgentTaskRunStatus::Pending => add(&self.retries_total, 1),
                AgentTaskRunStatus::Reconciling => add(&self.executions_reconciling_total, 1),
                AgentTaskRunStatus::DeadLetter => {
                    add(&self.executions_dead_letter_total, 1);
                    self.record_run_latency(run);
                }
                AgentTaskRunStatus::Cancelled => {
                    add(&self.executions_cancelled_total, 1);
                    self.record_run_latency(run);
                }
                AgentTaskRunStatus::Failed => {
                    add(&self.executions_failed_total, 1);
                    self.record_run_latency(run);
                }
                AgentTaskRunStatus::Claimed | AgentTaskRunStatus::Running => {
                    add(&self.executions_failed_total, 1);
                }
            },
            Err(_) => add(&self.executions_failed_total, 1),
        }
    }

    fn record_run_latency(&self, run: &sdkwork_intelligence_agents_service::AgentTaskRunRecord) {
        let Some(scheduled_for) = parse_datetime(&run.scheduled_for, None) else {
            return;
        };
        let completed_at = run.finished_at.as_deref().unwrap_or(&run.updated_at);
        let Some(completed_at) = parse_datetime(completed_at, None) else {
            return;
        };
        let elapsed_millis = diff_millis(scheduled_for, completed_at);
        if let Ok(elapsed_millis) = u64::try_from(elapsed_millis) {
            record_duration(
                &self.run_latency_micros,
                &self.completed_runs,
                Duration::from_millis(elapsed_millis),
            );
        }
    }

    pub(crate) fn record_operation_error(&self) {
        add(&self.operation_errors_total, 1);
    }

    pub(crate) fn record_forced_drain(&self) {
        add(&self.forced_drain_total, 1);
    }

    pub fn inflight(&self) -> u64 {
        self.inflight.load(Ordering::Relaxed)
    }

    pub fn render_prometheus(&self) -> String {
        let mut output = String::with_capacity(8_192);
        counter(
            &mut output,
            "sdkwork_agents_task_worker_materialized_total",
            "Logical task Runs materialized.",
            load(&self.materialized_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_task_worker_claimed_total",
            "Task Run delivery Attempts claimed.",
            load(&self.claimed_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_task_worker_recovered_leases_total",
            "Expired Task Run leases recovered.",
            load(&self.recovered_leases_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_outbox_events_claimed_total",
            "Outbox events claimed by dispatch rounds.",
            load(&self.outbox_events_claimed_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_outbox_deliveries_sent_total",
            "Webhook deliveries attempted for outbox events.",
            load(&self.outbox_deliveries_sent_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_outbox_events_published_total",
            "Outbox events fully published.",
            load(&self.outbox_events_published_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_outbox_events_retried_total",
            "Outbox events rescheduled for retry after a failed delivery.",
            load(&self.outbox_events_retried_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_outbox_events_dead_lettered_total",
            "Outbox events dead-lettered after exhausting attempts.",
            load(&self.outbox_events_dead_lettered_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_task_worker_recovered_timeouts_total",
            "Task Runs recovered after their configured execution timeout.",
            load(&self.recovered_timeouts_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_task_worker_retries_total",
            "Infrastructure retries scheduled for the same logical Run.",
            load(&self.retries_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_task_worker_fencing_rejections_total",
            "Lease heartbeats rejected by ownership or fencing checks.",
            load(&self.fencing_rejections_total),
        );
        for (suffix, help, value) in [
            (
                "examined",
                "Reconciling Task Runs examined.",
                load(&self.reconciliation_examined_total),
            ),
            (
                "reconciled",
                "Task Runs moved to a canonical terminal outcome.",
                load(&self.reconciliation_reconciled_total),
            ),
            (
                "pending",
                "Task Runs whose canonical Turn is not terminal.",
                load(&self.reconciliation_pending_total),
            ),
            (
                "conflicts",
                "Reconciliation writes skipped after a version conflict.",
                load(&self.reconciliation_conflicts_total),
            ),
        ] {
            counter(
                &mut output,
                &format!("sdkwork_agents_task_worker_reconciliation_{suffix}_total"),
                help,
                value,
            );
        }
        counter(
            &mut output,
            "sdkwork_agents_task_worker_heartbeats_total",
            "Task Run lease heartbeat operations.",
            load(&self.heartbeats_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_task_worker_heartbeat_failures_total",
            "Failed Task Run lease heartbeats.",
            load(&self.heartbeat_failures_total),
        );
        let _ = writeln!(
            output,
            "# HELP sdkwork_agents_task_worker_executions_total Task Run executions by bounded outcome.\n# TYPE sdkwork_agents_task_worker_executions_total counter"
        );
        for (outcome, value) in [
            ("succeeded", load(&self.executions_succeeded_total)),
            ("failed", load(&self.executions_failed_total)),
            ("reconciling", load(&self.executions_reconciling_total)),
            ("dead_letter", load(&self.executions_dead_letter_total)),
            ("cancelled", load(&self.executions_cancelled_total)),
        ] {
            let _ = writeln!(
                output,
                "sdkwork_agents_task_worker_executions_total{{outcome=\"{outcome}\"}} {value}"
            );
        }
        counter(
            &mut output,
            "sdkwork_agents_task_worker_operation_errors_total",
            "Scheduler repository operation errors.",
            load(&self.operation_errors_total),
        );
        counter(
            &mut output,
            "sdkwork_agents_task_worker_forced_drain_total",
            "Shutdowns that exceeded the configured drain timeout.",
            load(&self.forced_drain_total),
        );
        gauge(
            &mut output,
            "sdkwork_agents_task_worker_inflight",
            "Current Task Run executions.",
            load(&self.inflight),
        );
        duration_summary(
            &mut output,
            "sdkwork_agents_task_worker_materialization_duration_seconds",
            "PostgreSQL due-Task materialization duration.",
            load(&self.materialization_duration_micros),
            load(&self.materialization_operations),
        );
        duration_summary(
            &mut output,
            "sdkwork_agents_task_worker_claim_duration_seconds",
            "PostgreSQL Run claim transaction duration.",
            load(&self.claim_duration_micros),
            load(&self.claim_operations),
        );
        duration_summary(
            &mut output,
            "sdkwork_agents_task_worker_execution_duration_seconds",
            "Single Task Run delivery Attempt execution duration.",
            load(&self.execution_duration_micros),
            load(&self.execution_operations),
        );
        duration_summary(
            &mut output,
            "sdkwork_agents_task_run_latency_seconds",
            "Logical Task Run latency from scheduled occurrence to terminal state.",
            load(&self.run_latency_micros),
            load(&self.completed_runs),
        );
        for (name, help, value) in [
            (
                "sdkwork_agents_task_due",
                "Active Tasks currently due for materialization.",
                load(&self.due_tasks),
            ),
            (
                "sdkwork_agents_task_materialization_lag_seconds",
                "Age of the oldest due Task occurrence.",
                load(&self.materialization_lag_seconds),
            ),
            (
                "sdkwork_agents_task_run_eligible",
                "Pending Task Runs eligible for claim.",
                load(&self.eligible_runs),
            ),
            (
                "sdkwork_agents_task_run_eligible_oldest_age_seconds",
                "Age of the oldest eligible Task Run.",
                load(&self.eligible_run_oldest_age_seconds),
            ),
            (
                "sdkwork_agents_task_run_active_leases",
                "Unexpired claimed or running Task Run leases.",
                load(&self.active_leases),
            ),
            (
                "sdkwork_agents_task_run_reconciling",
                "Task Runs awaiting canonical outcome reconciliation.",
                load(&self.reconciling_runs),
            ),
            (
                "sdkwork_agents_task_run_reconciliation_oldest_age_seconds",
                "Age of the oldest reconciling Task Run.",
                load(&self.reconciliation_oldest_age_seconds),
            ),
            (
                "sdkwork_agents_outbox_pending",
                "Undelivered transactional outbox events.",
                load(&self.pending_outbox_events),
            ),
            (
                "sdkwork_agents_outbox_oldest_age_seconds",
                "Age of the oldest undelivered outbox event.",
                load(&self.outbox_oldest_age_seconds),
            ),
        ] {
            gauge(&mut output, name, help, value);
        }
        output
    }
}

fn counter(output: &mut String, name: &str, help: &str, value: u64) {
    let _ = writeln!(
        output,
        "# HELP {name} {help}\n# TYPE {name} counter\n{name} {value}"
    );
}

fn gauge(output: &mut String, name: &str, help: &str, value: u64) {
    let _ = writeln!(
        output,
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}"
    );
}

fn duration_summary(output: &mut String, name: &str, help: &str, micros: u64, count: u64) {
    let seconds = micros as f64 / 1_000_000.0;
    let _ = writeln!(
        output,
        "# HELP {name} {help}\n# TYPE {name} summary\n{name}_sum {seconds:.6}\n{name}_count {count}"
    );
}

fn record_duration(total: &AtomicU64, count: &AtomicU64, elapsed: Duration) {
    let micros = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
    add(total, micros);
    add(count, 1);
}

fn load(value: &AtomicU64) -> u64 {
    value.load(Ordering::Relaxed)
}

fn set(value: &AtomicU64, current: u64) {
    value.store(current, Ordering::Relaxed);
}

fn add(value: &AtomicU64, amount: u64) {
    let _ = value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(amount))
    });
}

fn subtract(value: &AtomicU64, amount: u64) {
    let _ = value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_sub(amount))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_use_only_bounded_outcome_labels_and_publish_scheduler_gauges() {
        let metrics = SchedulerWorkerMetrics::default();
        metrics.record_snapshot(TaskSchedulerMetricsSnapshot {
            due_tasks: 2,
            eligible_runs: 3,
            active_leases: 1,
            ..TaskSchedulerMetricsSnapshot::default()
        });
        let rendered = metrics.render_prometheus();
        assert!(rendered.contains("outcome=\"succeeded\""));
        assert!(rendered.contains("sdkwork_agents_task_due 2"));
        assert!(rendered.contains("sdkwork_agents_task_run_eligible 3"));
        assert!(rendered.contains("sdkwork_agents_task_run_latency_seconds_count 0"));
        assert!(!rendered.contains("tenant"));
        assert!(!rendered.contains("run_id"));
        assert!(!rendered.contains("worker_id"));
    }
}
