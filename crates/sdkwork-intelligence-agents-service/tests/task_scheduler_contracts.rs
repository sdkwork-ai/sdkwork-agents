use sdkwork_agent_kernel::PolicySubject;
use sdkwork_intelligence_agents_service::{
    AgentRepository, AgentTaskMisfirePolicy, AgentTaskOverlapPolicy, AgentTaskRecord,
    AgentTaskRunAttemptStatus, AgentTaskRunStatus, AgentTaskScheduleKind, AgentTaskStatus,
    AgentTaskTriggerKind, AgentsService, ClaimTaskRunsRequest, FailTaskRunRequest,
    IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository, ListTaskRunsCommand,
    MaterializeDueTasksRequest, PauseTaskCommand, ReconcileTaskRunRequest, ResumeTaskCommand,
    TaskRunAttemptCursor, TaskRunAttemptListQuery, TaskRunCursor, TaskRunFailureDisposition,
    TaskRunListQuery, TaskSchedulerRepository,
};

fn task(task_id: &str, tenant_id: u64, max_concurrent_runs: u16) -> AgentTaskRecord {
    AgentTaskRecord {
        id: tenant_id + u64::try_from(task_id.len()).expect("task id length"),
        task_id: task_id.to_string(),
        tenant_id,
        organization_id: 10,
        agent_id: "agent.scheduler-contract".to_string(),
        owner_user_id: 100,
        session_id: format!("session.{task_id}"),
        title: None,
        prompt: "execute contract task".to_string(),
        schedule_kind: AgentTaskScheduleKind::Cron,
        cron_expression: Some("0 * * * * *".to_string()),
        timezone: "UTC".to_string(),
        scheduled_at: None,
        starts_at: None,
        ends_at: None,
        next_fire_at: Some("2026-08-01T00:00:00.000Z".to_string()),
        misfire_policy: AgentTaskMisfirePolicy::FireOnce,
        overlap_policy: AgentTaskOverlapPolicy::Queue,
        max_concurrent_runs,
        max_catch_up_runs: 2,
        max_attempts: 3,
        retry_initial_delay_seconds: 5,
        retry_max_delay_seconds: 60,
        timeout_seconds: 300,
        priority: 0,
        status: AgentTaskStatus::Active,
        generation: 1,
        external_ref: None,
        metadata_json: "{}".to_string(),
        version: 0,
        created_at: "2026-07-31T00:00:00.000Z".to_string(),
        updated_at: "2026-07-31T00:00:00.000Z".to_string(),
        completed_at: None,
        paused_at: None,
        cancelled_at: None,
    }
}

fn task_operator() -> PolicySubject {
    PolicySubject::new("user.scheduler-operator", "100001").with_role("ai.agents.manage")
}

#[test]
fn skip_misfire_still_fires_within_the_grace_window() {
    let repository = InMemoryAgentRepository::new();
    let mut due = task("task.skip-grace", 100_001, 1);
    due.misfire_policy = AgentTaskMisfirePolicy::Skip;
    repository.insert_task(due).expect("insert Task");

    // The occurrence was due at 00:00:00 but the worker polls at 00:00:30:
    // ordinary polling latency must not drop a Skip-policy occurrence.
    let runs = repository
        .materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
            "2026-08-01T00:00:30.000Z",
            10,
        ))
        .expect("materialization");
    assert_eq!(runs.len(), 1, "grace-window Skip occurrence must still fire");
    assert_eq!(
        runs[0].scheduled_for.as_str(),
        "2026-08-01T00:00:00.000Z"
    );
}

#[test]
fn scheduled_occurrences_are_unique_and_catch_up_is_bounded() {
    let repository = InMemoryAgentRepository::new();
    let mut catch_up = task("task.catch-up", 100_001, 1);
    catch_up.misfire_policy = AgentTaskMisfirePolicy::CatchUp;
    repository.insert_task(catch_up).expect("insert Task");

    let first = repository
        .materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
            "2026-08-01T00:03:00.000Z",
            10,
        ))
        .expect("first materialization");
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].scheduled_for, "2026-08-01T00:00:00.000Z");
    assert_eq!(first[1].scheduled_for, "2026-08-01T00:01:00.000Z");

    let second = repository
        .materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
            "2026-08-01T00:03:00.000Z",
            10,
        ))
        .expect("second materialization");
    assert_eq!(second.len(), 2);
    assert_eq!(second[0].scheduled_for, "2026-08-01T00:02:00.000Z");
    assert_eq!(second[1].scheduled_for, "2026-08-01T00:03:00.000Z");

    let duplicate_scan = repository
        .materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
            "2026-08-01T00:03:00.000Z",
            10,
        ))
        .expect("duplicate scan");
    assert!(duplicate_scan.is_empty());
}

#[test]
fn skip_misfire_advances_without_creating_a_run() {
    let repository = InMemoryAgentRepository::new();
    let mut skipped = task("task.skip", 100_001, 1);
    skipped.misfire_policy = AgentTaskMisfirePolicy::Skip;
    repository.insert_task(skipped).expect("insert Task");

    let runs = repository
        .materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
            "2026-08-01T00:03:00.000Z",
            10,
        ))
        .expect("materialization");
    assert!(runs.is_empty());
    let stored = repository
        .get_task(100_001, 10, "task.skip")
        .expect("get Task")
        .expect("Task");
    assert_eq!(
        stored.next_fire_at.as_deref(),
        Some("2026-08-01T00:04:00.000Z")
    );
}

#[test]
fn claim_enforces_task_and_tenant_capacity() {
    let repository = InMemoryAgentRepository::new();
    let first = task("task.capacity-a", 100_001, 1);
    let second = task("task.capacity-b", 100_001, 1);
    let third = task("task.capacity-c", 200_001, 1);
    for record in [&first, &second, &third] {
        repository
            .insert_task(record.clone())
            .expect("insert capacity Task");
        repository
            .create_manual_task_run(
                record,
                &format!("manual:{}", record.task_id),
                "2026-08-01T00:00:00.000Z",
            )
            .expect("create manual Run");
    }

    let claims = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded_with_tenant_limit(
            "worker.capacity",
            "2026-08-01T00:00:01.000Z",
            60,
            10,
            1,
        ))
        .expect("claim Runs");
    assert_eq!(claims.len(), 2);
    assert_ne!(claims[0].run.tenant_id, claims[1].run.tenant_id);

    let extra_first_task_run = repository
        .create_manual_task_run(
            &first,
            "manual:task.capacity-a:second",
            "2026-08-01T00:00:02.000Z",
        )
        .expect("create second Run for Task");
    assert_eq!(extra_first_task_run.status, AgentTaskRunStatus::Pending);
    let blocked = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded_with_tenant_limit(
            "worker.capacity-2",
            "2026-08-01T00:00:03.000Z",
            60,
            10,
            10,
        ))
        .expect("claim with Task capacity");
    assert!(blocked
        .iter()
        .all(|claim| claim.run.task_id != first.task_id));
}

#[test]
fn manual_run_is_idempotent_and_payload_mismatch_is_rejected() {
    let repository = InMemoryAgentRepository::new();
    let first_task = task("task.manual-a", 100_001, 1);
    let second_task = task("task.manual-b", 100_001, 1);
    repository
        .insert_task(first_task.clone())
        .expect("insert Task");
    repository
        .insert_task(second_task.clone())
        .expect("insert Task");

    let first = repository
        .create_manual_task_run(
            &first_task,
            "manual.idempotency",
            "2026-08-01T00:00:00.000Z",
        )
        .expect("create manual Run");
    let duplicate = repository
        .create_manual_task_run(
            &first_task,
            "manual.idempotency",
            "2026-08-01T00:00:10.000Z",
        )
        .expect("repeat manual Run");
    assert_eq!(duplicate.run_id, first.run_id);
    assert_eq!(duplicate.turn_id, first.turn_id);
    assert!(repository
        .create_manual_task_run(
            &second_task,
            "manual.idempotency",
            "2026-08-01T00:00:20.000Z",
        )
        .is_err());
}

#[test]
fn expired_lease_reuses_run_and_turn_and_rejects_stale_worker() {
    let repository = InMemoryAgentRepository::new();
    let task = task("task.lease", 100_001, 1);
    repository.insert_task(task.clone()).expect("insert Task");
    let original = repository
        .create_manual_task_run(&task, "manual.lease", "2026-08-01T00:00:00.000Z")
        .expect("create Run");
    let first = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.first",
            "2026-08-01T00:00:01.000Z",
            10,
            1,
        ))
        .expect("first claim")
        .pop()
        .expect("claim");
    repository
        .mark_task_run_running(&first.lease, "2026-08-01T00:00:02.000Z")
        .expect("mark running");

    assert_eq!(
        repository
            .recover_expired_task_run_leases("2026-08-01T00:00:12.000Z", 10)
            .expect("recover lease"),
        1
    );
    assert!(repository
        .complete_task_run(
            &first.lease,
            original.turn_id.as_deref().expect("Turn id"),
            "2026-08-01T00:00:13.000Z",
        )
        .is_err());

    let second = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.second",
            "2026-08-01T00:00:13.000Z",
            10,
            1,
        ))
        .expect("second claim")
        .pop()
        .expect("claim");
    assert_eq!(second.run.run_id, original.run_id);
    assert_eq!(second.run.turn_id, original.turn_id);
    assert!(second.lease.fencing_token > first.lease.fencing_token);
}

#[test]
fn execution_timeout_reclaims_run_even_while_the_lease_is_alive() {
    let repository = InMemoryAgentRepository::new();
    let mut task = task("task.timeout", 100_001, 1);
    // Shorter than the maximum lease so an expired budget can coexist with a
    // live lease: claim at 00:00:01 sets timeout_at = 00:02:01 while the
    // lease runs to 00:05:01.
    task.timeout_seconds = 120;
    repository.insert_task(task.clone()).expect("insert Task");
    let original = repository
        .create_manual_task_run(&task, "manual.timeout", "2026-08-01T00:00:00.000Z")
        .expect("create Run");
    let first = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.first",
            "2026-08-01T00:00:01.000Z",
            300,
            1,
        ))
        .expect("first claim")
        .pop()
        .expect("claim");
    repository
        .mark_task_run_running(&first.lease, "2026-08-01T00:00:02.000Z")
        .expect("mark running");

    // The worker keeps heartbeating after the budget elapsed: at 00:03:00
    // the lease is extended to 00:08:00, so lease recovery must NOT touch
    // the Run while timeout recovery must.
    repository
        .heartbeat_task_run(
            &first.lease,
            "2026-08-01T00:03:00.000Z",
            300,
        )
        .expect("heartbeat keeps the lease alive");

    assert_eq!(
        repository
            .recover_expired_task_run_leases("2026-08-01T00:03:30.000Z", 10)
            .expect("lease recovery"),
        0,
        "a live lease must not be reclaimed by lease recovery"
    );
    assert_eq!(
        repository
            .recover_timed_out_task_runs("2026-08-01T00:03:30.000Z", 10)
            .expect("timeout recovery"),
        1,
        "an expired execution budget must be reclaimed even with a live lease"
    );

    // The stale worker's completion is fenced after the timeout reclaim.
    assert!(repository
        .complete_task_run(
            &first.lease,
            original.turn_id.as_deref().expect("Turn id"),
            "2026-08-01T00:03:31.000Z",
        )
        .is_err());

    // The Run is claimable again by another worker.
    let second = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.second",
            "2026-08-01T00:03:32.000Z",
            300,
            1,
        ))
        .expect("second claim")
        .pop()
        .expect("claim");
    assert_eq!(second.run.run_id, original.run_id);
    assert_eq!(second.run.turn_id, original.turn_id);
    assert!(second.lease.fencing_token > first.lease.fencing_token);
}

#[test]
fn execution_timeout_dead_letters_run_when_attempts_are_exhausted() {
    let repository = InMemoryAgentRepository::new();
    let mut task = task("task.timeout-dead-letter", 100_001, 1);
    task.max_attempts = 1;
    task.timeout_seconds = 120;
    repository.insert_task(task.clone()).expect("insert Task");
    let original = repository
        .create_manual_task_run(&task, "manual.timeout-dl", "2026-08-01T00:00:00.000Z")
        .expect("create Run");
    let first = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.first",
            "2026-08-01T00:00:01.000Z",
            300,
            1,
        ))
        .expect("first claim")
        .pop()
        .expect("claim");
    repository
        .mark_task_run_running(&first.lease, "2026-08-01T00:00:02.000Z")
        .expect("mark running");
    repository
        .heartbeat_task_run(
            &first.lease,
            "2026-08-01T00:03:00.000Z",
            300,
        )
        .expect("heartbeat keeps the lease alive");

    assert_eq!(
        repository
            .recover_timed_out_task_runs("2026-08-01T00:03:30.000Z", 10)
            .expect("timeout recovery"),
        1
    );

    let run = repository
        .get_task_run(first.lease.tenant_id, first.lease.organization_id, &first.run.run_id)
        .expect("get Run")
        .expect("Run exists");
    assert_eq!(run.status, AgentTaskRunStatus::DeadLetter);
    assert_eq!(run.failure_class.as_deref(), Some("execution_timeout"));
    assert_eq!(run.error_code.as_deref(), Some("task_run_timed_out"));
}

#[test]
fn infrastructure_retry_reuses_run_and_turn() {
    let repository = InMemoryAgentRepository::new();
    let task = task("task.retry", 100_001, 1);
    repository.insert_task(task.clone()).expect("insert Task");
    let original = repository
        .create_manual_task_run(&task, "manual.retry", "2026-08-01T00:00:00.000Z")
        .expect("create Run");
    let first = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.retry-1",
            "2026-08-01T00:00:01.000Z",
            60,
            1,
        ))
        .expect("first claim")
        .pop()
        .expect("claim");
    repository
        .mark_task_run_running(&first.lease, "2026-08-01T00:00:02.000Z")
        .expect("mark running");
    let retry = repository
        .fail_task_run(&FailTaskRunRequest {
            lease: first.lease,
            disposition: TaskRunFailureDisposition::Retry,
            error_code: "dependency_unavailable".to_string(),
            failure_class: "provider_unavailable".to_string(),
            retry_at: Some("2026-08-01T00:00:10.000Z".to_string()),
            failed_at: "2026-08-01T00:00:03.000Z".to_string(),
        })
        .expect("schedule retry");
    assert_eq!(retry.status, AgentTaskRunStatus::Pending);

    let second = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.retry-2",
            "2026-08-01T00:00:10.000Z",
            60,
            1,
        ))
        .expect("retry claim")
        .pop()
        .expect("claim");
    assert_eq!(second.run.run_id, original.run_id);
    assert_eq!(second.run.turn_id, original.turn_id);
    assert_eq!(second.run.attempt_count, 2);
}

#[test]
fn task_generation_transition_cancels_pending_runs_but_preserves_active_runs() {
    let repository = InMemoryAgentRepository::new();
    let original_task = task("task.generation-transition", 100_001, 2);
    repository
        .insert_task(original_task.clone())
        .expect("insert Task");
    let active_run = repository
        .create_manual_task_run(
            &original_task,
            "manual.generation.active",
            "2026-08-01T00:00:00.000Z",
        )
        .expect("create active Run");
    let pending_run = repository
        .create_manual_task_run(
            &original_task,
            "manual.generation.pending",
            "2026-08-01T00:00:01.000Z",
        )
        .expect("create pending Run");
    let claim = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.generation",
            "2026-08-01T00:00:02.000Z",
            60,
            1,
        ))
        .expect("claim Run")
        .pop()
        .expect("claimed Run");
    assert_eq!(claim.run.run_id, active_run.run_id);

    let mut paused_task = original_task;
    paused_task.status = AgentTaskStatus::Paused;
    paused_task.paused_at = Some("2026-08-01T00:00:03.000Z".to_string());
    paused_task.generation = 2;
    paused_task.version = 1;
    paused_task.updated_at = "2026-08-01T00:00:03.000Z".to_string();
    let transition = repository
        .transition_task(paused_task, "task_paused")
        .expect("transition Task");
    assert_eq!(transition.cancelled_pending_run_count, 1);

    let active = repository
        .get_task_run(100_001, 10, &active_run.run_id)
        .expect("get active Run")
        .expect("active Run");
    assert_eq!(active.status, AgentTaskRunStatus::Claimed);
    let pending = repository
        .get_task_run(100_001, 10, &pending_run.run_id)
        .expect("get pending Run")
        .expect("pending Run");
    assert_eq!(pending.status, AgentTaskRunStatus::Cancelled);
    assert_eq!(pending.error_code.as_deref(), Some("task_paused"));
}

#[test]
fn pause_and_resume_advance_task_version_and_generation() {
    let repository = InMemoryAgentRepository::new();
    repository
        .insert_task(task("task.pause-resume", 100_001, 1))
        .expect("insert Task");
    let service = AgentsService::new(
        repository,
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.scheduler-contract"),
    );

    let paused = service
        .pause_task(PauseTaskCommand {
            tenant_id: 100_001,
            organization_id: 10,
            path_agent_id: "agent.scheduler-contract".to_string(),
            task_id: "task.pause-resume".to_string(),
            expected_version: 0,
            owner_scope: Some(100),
            requested_by: task_operator(),
            requested_at: "2026-08-01T00:00:01.000Z".to_string(),
        })
        .expect("pause Task");
    assert_eq!(paused.status, AgentTaskStatus::Paused);
    assert_eq!(paused.version, 1);
    assert_eq!(paused.generation, 2);

    let resumed = service
        .resume_task(ResumeTaskCommand {
            tenant_id: 100_001,
            organization_id: 10,
            path_agent_id: "agent.scheduler-contract".to_string(),
            task_id: "task.pause-resume".to_string(),
            expected_version: paused.version,
            owner_scope: Some(100),
            requested_by: task_operator(),
            requested_at: "2026-08-01T00:00:02.000Z".to_string(),
        })
        .expect("resume Task");
    assert_eq!(resumed.status, AgentTaskStatus::Active);
    assert_eq!(resumed.version, 2);
    assert_eq!(resumed.generation, 3);
    assert!(resumed.paused_at.is_none());
}

#[test]
fn business_retry_creates_a_new_run_and_turn() {
    let repository = InMemoryAgentRepository::new();
    let task = task("task.business-retry", 100_001, 1);
    repository.insert_task(task.clone()).expect("insert Task");
    let source = repository
        .create_manual_task_run(
            &task,
            "manual.business-retry.source",
            "2026-08-01T00:00:00.000Z",
        )
        .expect("create source Run");
    let claim = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.business-retry",
            "2026-08-01T00:00:01.000Z",
            60,
            1,
        ))
        .expect("claim source Run")
        .pop()
        .expect("claimed source Run");
    repository
        .mark_task_run_running(&claim.lease, "2026-08-01T00:00:02.000Z")
        .expect("mark source Run running");
    let failed = repository
        .fail_task_run(&FailTaskRunRequest {
            lease: claim.lease,
            disposition: TaskRunFailureDisposition::Terminal,
            error_code: "agent_execution_failed".to_string(),
            failure_class: "business_failure".to_string(),
            retry_at: None,
            failed_at: "2026-08-01T00:00:03.000Z".to_string(),
        })
        .expect("fail source Run");

    let retry = repository
        .create_business_retry_task_run(
            &task,
            &failed,
            "business-retry.idempotency",
            "2026-08-01T00:00:04.000Z",
        )
        .expect("create business retry Run");
    assert_ne!(retry.run_id, source.run_id);
    assert_ne!(retry.turn_id, source.turn_id);
    assert_eq!(
        retry.retry_of_run_id.as_deref(),
        Some(source.run_id.as_str())
    );
    assert_eq!(retry.trigger_kind, AgentTaskTriggerKind::BusinessRetry);
    assert_eq!(retry.attempt_count, 0);

    let duplicate = repository
        .create_business_retry_task_run(
            &task,
            &failed,
            "business-retry.idempotency",
            "2026-08-01T00:00:05.000Z",
        )
        .expect("repeat business retry");
    assert_eq!(duplicate.run_id, retry.run_id);
    assert_eq!(duplicate.turn_id, retry.turn_id);
}

#[test]
fn cancellation_distinguishes_pending_and_active_run_outcomes() {
    let repository = InMemoryAgentRepository::new();
    let task = task("task.cancellation", 100_001, 2);
    repository.insert_task(task.clone()).expect("insert Task");
    let active = repository
        .create_manual_task_run(
            &task,
            "manual.cancellation.active",
            "2026-08-01T00:00:00.000Z",
        )
        .expect("create active Run");
    let pending = repository
        .create_manual_task_run(
            &task,
            "manual.cancellation.pending",
            "2026-08-01T00:00:01.000Z",
        )
        .expect("create pending Run");
    let claim = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.cancellation",
            "2026-08-01T00:00:02.000Z",
            60,
            1,
        ))
        .expect("claim active Run")
        .pop()
        .expect("claimed active Run");
    assert_eq!(claim.run.run_id, active.run_id);
    repository
        .mark_task_run_running(&claim.lease, "2026-08-01T00:00:03.000Z")
        .expect("mark active Run running");

    let cancelled_pending = repository
        .request_task_run_cancellation(
            100_001,
            10,
            &pending.run_id,
            Some(pending.version),
            "2026-08-01T00:00:04.000Z",
        )
        .expect("cancel pending Run");
    assert_eq!(cancelled_pending.status, AgentTaskRunStatus::Cancelled);

    let reconciling = repository
        .request_task_run_cancellation(
            100_001,
            10,
            &active.run_id,
            None,
            "2026-08-01T00:00:05.000Z",
        )
        .expect("request active Run cancellation");
    assert_eq!(reconciling.status, AgentTaskRunStatus::Reconciling);
    assert!(reconciling.lease_owner.is_none());
    assert!(reconciling.lease_token_hash.is_none());

    let attempts = repository
        .list_task_run_attempts(&TaskRunAttemptListQuery::for_run(
            100_001,
            10,
            &active.run_id,
        ))
        .expect("list attempts");
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].status, AgentTaskRunAttemptStatus::Failed);
    assert_eq!(
        attempts[0].failure_class.as_deref(),
        Some("outcome_unknown")
    );

    assert!(repository
        .reconcile_task_run(&ReconcileTaskRunRequest {
            tenant_id: 100_001,
            organization_id: 10,
            run_id: active.run_id.clone(),
            expected_version: reconciling.version.saturating_sub(1),
            terminal_status: AgentTaskRunStatus::Succeeded,
            error_code: None,
            reconciled_at: "2026-08-01T00:00:06.000Z".to_string(),
        })
        .is_err());
    let succeeded = repository
        .reconcile_task_run(&ReconcileTaskRunRequest {
            tenant_id: 100_001,
            organization_id: 10,
            run_id: active.run_id.clone(),
            expected_version: reconciling.version,
            terminal_status: AgentTaskRunStatus::Succeeded,
            error_code: None,
            reconciled_at: "2026-08-01T00:00:06.000Z".to_string(),
        })
        .expect("reconcile active Run");
    assert_eq!(succeeded.status, AgentTaskRunStatus::Succeeded);
}

#[test]
fn completion_rejects_a_stale_task_generation() {
    let repository = InMemoryAgentRepository::new();
    let original_task = task("task.stale-completion", 100_001, 1);
    repository
        .insert_task(original_task.clone())
        .expect("insert Task");
    let run = repository
        .create_manual_task_run(
            &original_task,
            "manual.stale-completion",
            "2026-08-01T00:00:00.000Z",
        )
        .expect("create Run");
    let claim = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.stale-completion",
            "2026-08-01T00:00:01.000Z",
            60,
            1,
        ))
        .expect("claim Run")
        .pop()
        .expect("claimed Run");
    repository
        .mark_task_run_running(&claim.lease, "2026-08-01T00:00:02.000Z")
        .expect("mark Run running");

    let mut replaced_task = original_task;
    replaced_task.generation = 2;
    replaced_task.version = 1;
    replaced_task.updated_at = "2026-08-01T00:00:03.000Z".to_string();
    repository
        .transition_task(replaced_task, "task_definition_replaced")
        .expect("replace Task generation");

    assert!(repository
        .complete_task_run(
            &claim.lease,
            run.turn_id.as_deref().expect("Turn id"),
            "2026-08-01T00:00:04.000Z",
        )
        .is_err());
}

#[test]
fn run_and_attempt_keyset_pagination_is_continuous_and_scope_bound() {
    let repository = InMemoryAgentRepository::new();
    let first_task = task("task.cursor-a", 100_001, 4);
    let second_task = task("task.cursor-b", 100_001, 1);
    repository
        .insert_task(first_task.clone())
        .expect("insert first Task");
    repository
        .insert_task(second_task.clone())
        .expect("insert second Task");
    for index in 0..4 {
        repository
            .create_manual_task_run(
                &first_task,
                &format!("manual.cursor.{index}"),
                &format!("2026-08-01T00:00:0{index}.000Z"),
            )
            .expect("create cursor Run");
    }

    let first_query = TaskRunListQuery::for_task(100_001, 10, &first_task.task_id)
        .for_owner(first_task.owner_user_id)
        .with_cursor_page(2, None);
    let first_page = repository
        .list_task_runs(&first_query)
        .expect("list first Run page");
    assert_eq!(first_page.len(), 3);
    let first_visible_ids = first_page[..2]
        .iter()
        .map(|run| run.run_id.clone())
        .collect::<Vec<_>>();
    let run_cursor = TaskRunCursor {
        run_internal_id: first_page[1].id,
        scope_fingerprint: first_query.scope_fingerprint(),
    };
    let second_page = repository
        .list_task_runs(
            &TaskRunListQuery::for_task(100_001, 10, &first_task.task_id)
                .for_owner(first_task.owner_user_id)
                .with_cursor_page(2, Some(run_cursor.clone())),
        )
        .expect("list second Run page");
    assert_eq!(second_page.len(), 2);
    assert!(second_page
        .iter()
        .all(|run| !first_visible_ids.contains(&run.run_id)));

    let service = AgentsService::new(
        repository,
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.scheduler-contract"),
    );
    let wrong_scope = service.list_task_runs(ListTaskRunsCommand {
        query: TaskRunListQuery::for_task(100_001, 10, &second_task.task_id)
            .for_owner(second_task.owner_user_id)
            .with_cursor_page(2, Some(run_cursor)),
        path_agent_id: second_task.agent_id.clone(),
        requested_by: task_operator(),
    });
    assert!(wrong_scope.is_err());
}

#[test]
fn attempt_keyset_cursor_tracks_compound_sort_position() {
    let repository = InMemoryAgentRepository::new();
    let task = task("task.attempt-cursor", 100_001, 1);
    repository.insert_task(task.clone()).expect("insert Task");
    let run = repository
        .create_manual_task_run(&task, "manual.attempt-cursor", "2026-08-01T00:00:00.000Z")
        .expect("create Run");
    for attempt_no in 1..=3 {
        let claim_second = attempt_no * 2 - 1;
        let failed_second = attempt_no * 2;
        let retry_second = failed_second + 1;
        let claim = repository
            .claim_task_runs(&ClaimTaskRunsRequest::bounded(
                format!("worker.attempt-cursor.{attempt_no}"),
                format!("2026-08-01T00:00:{claim_second:02}.000Z"),
                60,
                1,
            ))
            .expect("claim Run")
            .pop()
            .expect("claimed Run");
        repository
            .mark_task_run_running(
                &claim.lease,
                &format!("2026-08-01T00:00:{claim_second:02}.500Z"),
            )
            .expect("mark Run running");
        repository
            .fail_task_run(&FailTaskRunRequest {
                lease: claim.lease,
                disposition: TaskRunFailureDisposition::Retry,
                error_code: "dependency_unavailable".to_string(),
                failure_class: "provider_unavailable".to_string(),
                retry_at: Some(format!("2026-08-01T00:00:{retry_second:02}.000Z")),
                failed_at: format!("2026-08-01T00:00:{failed_second:02}.000Z"),
            })
            .expect("fail Run attempt");
    }

    let first_query =
        TaskRunAttemptListQuery::for_run(100_001, 10, &run.run_id).with_cursor_page(1, None);
    let first_page = repository
        .list_task_run_attempts(&first_query)
        .expect("list first Attempt page");
    assert_eq!(first_page.len(), 2);
    assert_eq!(first_page[0].attempt_no, 3);
    let cursor = TaskRunAttemptCursor {
        attempt_no: first_page[0].attempt_no,
        attempt_internal_id: first_page[0].id,
        scope_fingerprint: first_query.scope_fingerprint(),
    };
    let second_page = repository
        .list_task_run_attempts(
            &TaskRunAttemptListQuery::for_run(100_001, 10, &run.run_id)
                .with_cursor_page(1, Some(cursor)),
        )
        .expect("list second Attempt page");
    assert_eq!(second_page[0].attempt_no, 2);
}
