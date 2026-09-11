#![cfg(feature = "postgres-sync")]

use std::collections::HashSet;
use std::ffi::OsString;
use std::sync::{Arc, Barrier, Condvar, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sdkwork_agent_kernel::{AgentManifest, KernelErrorKind, PolicySubject};
use sdkwork_database_config::workspace_database::{
    build_postgres_database_url, workspace_postgres_test_database_url,
};
use sdkwork_intelligence_agents_service::{
    AgentBusinessIdGenerator, AgentCompositionSlotKind, AgentCompositionTargetModule,
    AgentIdGenerator, AgentImplementationKind, AgentItemDriveRefInput, AgentItemFeedbackRating,
    AgentItemResourceRole, AgentProjectDriveAccessMode, AgentProjectVisibility,
    AgentProviderBindingCommand, AgentRepository, AgentSessionEntrySurface, AgentSessionItemKind,
    AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionKind, AgentTaskMisfirePolicy,
    AgentTaskOverlapPolicy, AgentTaskRecord, AgentTaskRunAttemptStatus, AgentTaskScheduleKind,
    AgentTaskStatus, AgentTurnMode, AgentTurnRecord, AgentTurnStatus, AgentVisibility,
    AgentsService, CancelTurnCommand, ClaimTaskRunsRequest, CreateAgentCommand,
    CreateProjectCommand, CreateProjectCompositionSlotCommand, CreateSessionCommand,
    CreateSessionItemCommand, CreateSessionRuntimeBindingCommand, CreateTurnCommand,
    DeleteProjectCompositionSlotCommand, GetAgentCommand, IamGatedPolicyProvider,
    InMemoryAgentAuditSink, ItemFeedbackListQuery, ListItemFeedbackCommand,
    ListProjectCompositionSlotsCommand, ListProjectsCommand, ListSessionItemsCommand,
    ListSessionRuntimeBindingsCommand, ListSessionUserStatesCommand, MaterializeDueTasksRequest,
    ProjectCompositionSlotListQuery, ProjectListQuery, ResourceUserStateListQuery,
    SessionItemListQuery, SessionRuntimeBindingListQuery, SqlAgentAuditSink, SqlAgentRepository,
    SyncPostgresAdapter, TaskRunAttemptListQuery, TaskSchedulerRepository, TurnExecutionInput,
    TurnExecutionOutput, TurnExecutor, TurnListQuery, UpdateItemFeedbackCommand,
    UpdateProjectCompositionSlotCommand, UpdateSessionUserStateCommand,
    RUNTIME_MODE_INFERENCE_ERROR,
};

struct FailingTurnExecutor;

type LiveAgentsService = AgentsService<
    SqlAgentRepository<SyncPostgresAdapter>,
    InMemoryAgentAuditSink,
    IamGatedPolicyProvider,
>;

#[derive(Debug, Default)]
struct BlockingTurnState {
    started: bool,
    released: bool,
}

#[derive(Debug, Default)]
struct BlockingTurnExecutor {
    state: Mutex<BlockingTurnState>,
    changed: Condvar,
}

impl BlockingTurnExecutor {
    fn wait_until_started(&self) {
        let state = self
            .state
            .lock()
            .expect("blocking executor state should lock");
        let (state, timeout) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(10), |state| !state.started)
            .expect("blocking executor start wait should not be poisoned");
        assert!(state.started, "turn provider did not start before timeout");
        assert!(!timeout.timed_out(), "turn provider start wait timed out");
    }

    fn release(&self) {
        let mut state = self
            .state
            .lock()
            .expect("blocking executor state should lock");
        state.released = true;
        self.changed.notify_all();
    }
}

const DATABASE_URL_ENV: &str = "SDKWORK_DATABASE_URL";
const DATABASE_SCHEMA_ENV: &str = "SDKWORK_DATABASE_SCHEMA";
const DATABASE_SCHEMA_FALLBACK_PUBLIC_ENV: &str = "SDKWORK_DATABASE_SCHEMA_FALLBACK_PUBLIC";
const DATABASE_AUTO_MIGRATE_ENV: &str = "SDKWORK_DATABASE_AUTO_MIGRATE";

impl TurnExecutor for FailingTurnExecutor {
    fn complete(&self, _input: &TurnExecutionInput) -> TurnExecutionOutput {
        TurnExecutionOutput {
            model_request_id: None,
            finish_reason: None,
            content: "provider detail must not be persisted".to_string(),
            model_id: None,
            provider_id: None,
            provider_session_id: None,
            input_tokens: 0,
            output_tokens: 0,
            runtime_mode: RUNTIME_MODE_INFERENCE_ERROR,
            stream_deltas: Vec::new(),
            stream_events: Vec::new(),
        }
    }
}

impl TurnExecutor for BlockingTurnExecutor {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        let mut state = self
            .state
            .lock()
            .expect("blocking executor state should lock");
        state.started = true;
        self.changed.notify_all();
        state = self
            .changed
            .wait_while(state, |state| !state.released)
            .expect("blocking executor release wait should not be poisoned");
        drop(state);
        TurnExecutionOutput {
            model_request_id: Some(input.model_request_id.clone()),
            finish_reason: Some("stop".to_string()),
            content: "completion that must lose the cancellation race".to_string(),
            model_id: input.model_id.clone(),
            provider_id: input.provider_id.clone(),
            provider_session_id: input.provider_session_id.clone(),
            input_tokens: 3,
            output_tokens: 5,
            runtime_mode: "postgres-live-blocking",
            stream_deltas: Vec::new(),
            stream_events: Vec::new(),
        }
    }
}

struct IsolatedPostgresDatabase {
    admin: SyncPostgresAdapter,
    database: String,
    schema: String,
    _environment: Vec<EnvironmentVariableGuard>,
}

struct EnvironmentVariableGuard {
    key: &'static str,
    previous_value: Option<OsString>,
}

impl EnvironmentVariableGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous_value = std::env::var_os(key);
        std::env::set_var(key, value);
        Self {
            key,
            previous_value,
        }
    }
}

impl Drop for EnvironmentVariableGuard {
    fn drop(&mut self) {
        match &self.previous_value {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

impl Drop for IsolatedPostgresDatabase {
    fn drop(&mut self) {
        let drop_database = format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", self.database);
        let pool = self.admin.pool();
        // The database identifier is a fixed-size Snowflake suffix produced by
        // this test itself, never user input: assert-safe by construction.
        if let Err(error) =
            pool.run(sqlx::query(sqlx::AssertSqlSafe(drop_database.as_str())).execute(pool.pool()))
        {
            eprintln!(
                "failed to drop ephemeral workspace test database {}: {error}",
                self.database
            );
        }
    }
}

fn database_environment_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn test_id_generator() -> AgentBusinessIdGenerator {
    static GENERATOR: OnceLock<AgentBusinessIdGenerator> = OnceLock::new();
    GENERATOR
        .get_or_init(|| {
            AgentBusinessIdGenerator::with_node_id(911)
                .expect("postgres live test Snowflake generator should initialize")
        })
        .clone()
}

fn connect_test_adapter(
    connection_uri: &str,
) -> sdkwork_agent_kernel::KernelResult<SyncPostgresAdapter> {
    SyncPostgresAdapter::connect_with_id_generator(connection_uri, test_id_generator())
}

fn create_isolated_database(
    base_url: &str,
    suffix: u128,
) -> (IsolatedPostgresDatabase, String, SyncPostgresAdapter) {
    let admin_url = workspace_admin_database_url();
    let admin =
        connect_test_adapter(&admin_url).expect("workspace postgres admin adapter should connect");
    let database = format!("sdkwork_ai_test_{suffix}");
    let create_database = format!("CREATE DATABASE {database} OWNER sdkwork_ai_test");
    admin
        .pool()
        .run(
            sqlx::query(sqlx::AssertSqlSafe(create_database.as_str())).execute(admin.pool().pool()),
        )
        .expect("ephemeral workspace test database should be created");

    let isolated_url = workspace_postgres_test_database_url(base_url, &database)
        .expect("workspace test database URL should normalize");
    let schema = database.clone();
    let schema_owner =
        connect_test_adapter(&isolated_url).expect("workspace test database owner should connect");
    let create_schema = format!("CREATE SCHEMA {schema} AUTHORIZATION sdkwork_ai_test");
    schema_owner
        .pool()
        .run(
            sqlx::query(sqlx::AssertSqlSafe(create_schema.as_str()))
                .execute(schema_owner.pool().pool()),
        )
        .expect("same-named workspace test schema should be created");
    drop(schema_owner);

    let environment = vec![
        EnvironmentVariableGuard::set(DATABASE_URL_ENV, &isolated_url),
        EnvironmentVariableGuard::set(DATABASE_SCHEMA_ENV, &schema),
        EnvironmentVariableGuard::set(DATABASE_SCHEMA_FALLBACK_PUBLIC_ENV, "false"),
        EnvironmentVariableGuard::set(DATABASE_AUTO_MIGRATE_ENV, "false"),
    ];
    let isolated_database = IsolatedPostgresDatabase {
        admin,
        database,
        schema,
        _environment: environment,
    };
    let isolated = connect_test_adapter(&isolated_url)
        .expect("workspace test postgres adapter should connect");

    (isolated_database, isolated_url, isolated)
}

fn workspace_admin_database_url() -> String {
    let required = |key: &str| {
        std::env::var(key)
            .unwrap_or_else(|_| panic!("{key} must be set for ephemeral test provisioning"))
    };
    build_postgres_database_url(
        &required("SDKWORK_DATABASE_ADMIN_HOST"),
        std::env::var("SDKWORK_DATABASE_ADMIN_PORT").ok().as_deref(),
        &required("SDKWORK_DATABASE_ADMIN_DATABASE"),
        &required("SDKWORK_DATABASE_ADMIN_USERNAME"),
        &required("SDKWORK_DATABASE_ADMIN_PASSWORD"),
        std::env::var("SDKWORK_DATABASE_ADMIN_SSL_MODE")
            .ok()
            .as_deref(),
    )
}

fn bootstrap_isolated_database(base_url: &str, suffix: u128) -> (IsolatedPostgresDatabase, String) {
    let (isolated_database, isolated_url, isolated) = create_isolated_database(base_url, suffix);
    let database_pool = isolated.pool().database_pool().clone();
    isolated
        .pool()
        .block_on(sdkwork_agents_database_host::bootstrap_agents_database(
            database_pool,
        ))
        .expect("agents database lifecycle should bootstrap the isolated schema");
    assert_eq!(
        authoritative_agent_table_count(&isolated, &isolated_database.schema),
        23,
        "greenfield init must materialize exactly 23 authoritative Agents baseline tables without enabling auto-migrate",
    );

    (isolated_database, isolated_url)
}

fn authoritative_agent_table_count(adapter: &SyncPostgresAdapter, schema: &str) -> i64 {
    adapter
        .pool()
        .run(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM information_schema.tables \
                 WHERE table_schema = $1 \
                 AND table_type = 'BASE TABLE' \
                 AND table_name LIKE 'ai_agent%'",
            )
            .bind(schema)
            .fetch_one(adapter.pool().pool()),
        )
        .expect("authoritative Agents table count should be queryable")
}

fn subject() -> PolicySubject {
    PolicySubject::new("user.700", "700001").with_role("ai.agents.manage")
}

fn numeric_subject() -> PolicySubject {
    PolicySubject::new("700", "700001").with_role("ai.agents.manage")
}

fn manifest(agent_id: &str) -> AgentManifest {
    AgentManifest {
        schema_version: "1.0.0".to_string(),
        manifest_type: "agent".to_string(),
        agent_id: agent_id.to_string(),
        name: "postgres-user-state".to_string(),
        display_name: "Postgres User State".to_string(),
        description: "live contract fixture".to_string(),
        version: "0.1.0".to_string(),
        domain: "intelligence".to_string(),
        required_capabilities: vec!["model.chat".to_string()],
        optional_capabilities: Vec::new(),
        required_capability_requirements: Vec::new(),
        optional_capability_requirements: Vec::new(),
        event_families: vec!["agent.lifecycle".to_string()],
        owner_name: "sdkwork".to_string(),
        status: "active".to_string(),
    }
}

fn request_item_for_turn(id: u64, turn: &AgentTurnRecord) -> AgentSessionItemRecord {
    AgentSessionItemRecord {
        id,
        item_id: turn.request_item_id.clone(),
        tenant_id: turn.tenant_id,
        organization_id: turn.organization_id,
        session_id: turn.session_id.clone(),
        kind: AgentSessionItemKind::UserInput,
        content: Some("PostgreSQL lifecycle test request".to_string()),
        content_type: "text/plain".to_string(),
        status: AgentSessionItemStatus::Completed,
        sequence: 0,
        input_tokens: 0,
        output_tokens: 0,
        model_id: None,
        provider_id: None,
        provider_payload_json: None,
        tool_name: None,
        tool_call_id: None,
        tool_arguments_json: None,
        tool_result_json: None,
        parent_item_id: None,
        turn_id: Some(turn.turn_id.clone()),
        created_by: turn.owner_user_id,
        version: 0,
        created_at: turn.created_at.clone(),
        updated_at: turn.updated_at.clone(),
        completed_at: Some(turn.created_at.clone()),
        redacted_at: None,
        redacted_by: None,
        retention_until: None,
    }
}

fn create_live_turn_service(
    database_url: &str,
    suffix: u128,
    turn_executor: Option<Arc<dyn TurnExecutor>>,
) -> (Arc<LiveAgentsService>, String, String, String) {
    let service = AgentsService::new(
        SqlAgentRepository::new(
            connect_test_adapter(database_url).expect("live turn service adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-concurrency.postgres-live"),
    );
    let service = match turn_executor {
        Some(turn_executor) => service.with_turn_executor(turn_executor),
        None => service,
    };
    let agent_id = format!("agent.concurrency.{suffix}");
    service
        .create_agent(CreateAgentCommand {
            agent_id: agent_id.clone(),
            tenant_id: 700_001,
            organization_id: 0,
            owner_user_id: 700,
            code: format!("concurrency-{suffix}"),
            display_name: "Postgres Turn Concurrency".to_string(),
            description: None,
            manifest: manifest(&agent_id),
            visibility: AgentVisibility::Private,
            tags: Vec::new(),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:00Z".to_string(),
        })
        .expect("live concurrency agent should be created");
    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 700_001,
            agent_id: agent_id.clone(),
            binding_id: format!("binding.concurrency.{suffix}"),
            provider_id: format!("provider.concurrency.{suffix}"),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: format!("profile.concurrency.{suffix}"),
            capabilities: Vec::new(),
            make_default: true,
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:01Z".to_string(),
        })
        .expect("live concurrency provider binding should be created");
    (
        Arc::new(service),
        agent_id,
        provider_binding.binding_id,
        provider_binding.provider_id,
    )
}

fn create_live_turn_session(
    service: &LiveAgentsService,
    agent_id: &str,
    provider_binding_id: &str,
    provider_id: &str,
    suffix: u128,
    label: &str,
) -> (String, String, String) {
    let session_id = format!("session.{label}.{suffix}");
    let runtime_binding_id = format!("runtime_binding.{label}.{suffix}");
    let model_id = format!("model.{label}.{suffix}");
    service
        .create_session(CreateSessionCommand {
            tenant_id: 700_001,
            organization_id: 0,
            agent_id: agent_id.to_string(),
            owner_user_id: 700,
            session_id: session_id.clone(),
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: None,
            idempotency_key: None,
            payload_hash: None,
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:02Z".to_string(),
        })
        .expect("live concurrency session should be created");
    service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 700_001,
            organization_id: 0,
            path_agent_id: agent_id.to_string(),
            session_id: session_id.clone(),
            runtime_binding_id: Some(runtime_binding_id.clone()),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: provider_binding_id.to_string(),
            model_id: model_id.clone(),
            provider_id: provider_id.to_string(),
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            provider_directory: None,
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:03Z".to_string(),
        })
        .expect("live concurrency runtime binding should be created");
    (session_id, runtime_binding_id, model_id)
}

struct LiveTurnTarget<'a> {
    agent_id: &'a str,
    session_id: &'a str,
    runtime_binding_id: &'a str,
    model_id: &'a str,
}

fn live_turn_command(
    target: LiveTurnTarget<'_>,
    turn_id: impl Into<String>,
    idempotency_key: impl Into<String>,
    payload_hash: impl Into<String>,
    content: impl Into<String>,
) -> CreateTurnCommand {
    CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: target.agent_id.to_string(),
        session_id: target.session_id.to_string(),
        turn_id: Some(turn_id.into()),
        content: content.into(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(target.runtime_binding_id.to_string()),
        requested_model_id: Some(target.model_id.to_string()),
        access_mode_id: None,
        idempotency_key: idempotency_key.into(),
        payload_hash: payload_hash.into(),
        client_request_id: None,
        drive_refs: Vec::new(),
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-20T00:00:04Z".to_string(),
        prefer_stream: false,
        auth_token: None,
        wire_protocol: None,
    }
}

fn read_session_aggregate(
    database_url: &str,
    session_id: &str,
) -> (
    sdkwork_intelligence_agents_service::AgentSessionRecord,
    Vec<AgentSessionItemRecord>,
    Vec<AgentTurnRecord>,
) {
    let repository = SqlAgentRepository::new(
        connect_test_adapter(database_url)
            .expect("live aggregate inspection adapter should connect"),
    );
    let session = repository
        .get_session(700_001, 0, session_id)
        .expect("live session should be queryable")
        .expect("live session should exist");
    let items = repository
        .list_session_items(&SessionItemListQuery::for_session(
            700_001,
            0,
            session_id.to_string(),
        ))
        .expect("live session items should be queryable");
    let turns = repository
        .list_turns(&TurnListQuery::for_session(
            700_001,
            0,
            session_id.to_string(),
        ))
        .expect("live turns should be queryable");
    (session, items, turns)
}

fn live_scheduler_task(
    suffix: u128,
    label: &str,
    agent_id: &str,
    session_id: &str,
    max_concurrent_runs: u16,
) -> sdkwork_agent_kernel::KernelResult<AgentTaskRecord> {
    Ok(AgentTaskRecord {
        id: test_id_generator().next_id()?,
        task_id: format!("task.scheduler.{label}.{suffix}"),
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: agent_id.to_string(),
        owner_user_id: 700,
        session_id: session_id.to_string(),
        title: Some(format!("PostgreSQL scheduler {label}")),
        prompt: "execute the PostgreSQL scheduler contract".to_string(),
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
        max_catch_up_runs: 1,
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
    })
}

fn create_live_scheduler_context(
    database_url: &str,
    suffix: u128,
) -> (String, String, SqlAgentRepository<SyncPostgresAdapter>) {
    let (service, agent_id, provider_binding_id, provider_id) =
        create_live_turn_service(database_url, suffix, None);
    let (session_id, _, _) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "scheduler",
    );
    let repository = SqlAgentRepository::new(
        connect_test_adapter(database_url).expect("scheduler repository should connect"),
    );
    (agent_id, session_id, repository)
}

fn scalar_count(database_url: &str, sql: &'static str, task_id: &str) -> i64 {
    let adapter =
        connect_test_adapter(database_url).expect("scheduler count adapter should connect");
    adapter
        .pool()
        .run(
            sqlx::query_scalar::<_, i64>(sql)
                .bind(task_id)
                .fetch_one(adapter.pool().pool()),
        )
        .expect("scheduler count should be queryable")
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_concurrent_materialization_creates_one_run_and_one_outbox_fact() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let (agent_id, session_id, repository) = create_live_scheduler_context(&database_url, suffix);
    let task = live_scheduler_task(suffix, "materialize", &agent_id, &session_id, 1)
        .expect("scheduler Task should build");
    repository
        .insert_task(task.clone())
        .expect("scheduler Task should persist");

    const WORKER_COUNT: usize = 8;
    let start = Arc::new(Barrier::new(WORKER_COUNT + 1));
    let workers = (0..WORKER_COUNT)
        .map(|_| {
            let start = Arc::clone(&start);
            let database_url = database_url.clone();
            thread::spawn(move || {
                let repository = SqlAgentRepository::new(
                    connect_test_adapter(&database_url)
                        .expect("materializer connection should initialize"),
                );
                start.wait();
                repository.materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
                    "2026-08-01T00:00:00.000Z",
                    10,
                ))
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    let materialized = workers
        .into_iter()
        .flat_map(|worker| {
            worker
                .join()
                .expect("materializer thread should not panic")
                .expect("materialization should succeed")
        })
        .collect::<Vec<_>>();
    let run_ids = materialized
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(materialized.len(), 1);
    assert_eq!(run_ids.len(), 1);
    assert_eq!(
        scalar_count(
            &database_url,
            "SELECT COUNT(*)::bigint FROM ai_agent_task_run WHERE task_id = $1",
            &task.task_id,
        ),
        1
    );
    assert_eq!(
        scalar_count(
            &database_url,
            "SELECT COUNT(*)::bigint FROM ai_agent_outbox_event WHERE event_type = 'agent.task.run.materialized' AND payload_json->>'taskId' = $1",
            &task.task_id,
        ),
        1
    );
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_concurrent_claims_are_unique_and_capacity_bounded() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let (agent_id, session_id, repository) = create_live_scheduler_context(&database_url, suffix);
    let task_a = live_scheduler_task(suffix, "claim-a", &agent_id, &session_id, 2)
        .expect("first scheduler Task should build");
    let task_b = live_scheduler_task(suffix, "claim-b", &agent_id, &session_id, 2)
        .expect("second scheduler Task should build");
    for task in [&task_a, &task_b] {
        repository
            .insert_task(task.clone())
            .expect("scheduler Task should persist");
        for index in 0..4 {
            repository
                .create_manual_task_run(
                    task,
                    &format!("manual:{}:{index}", task.task_id),
                    "2026-08-01T00:00:00.000Z",
                )
                .expect("manual scheduler Run should persist");
        }
    }

    const WORKER_COUNT: usize = 8;
    let start = Arc::new(Barrier::new(WORKER_COUNT + 1));
    let workers = (0..WORKER_COUNT)
        .map(|index| {
            let start = Arc::clone(&start);
            let database_url = database_url.clone();
            thread::spawn(move || {
                let repository = SqlAgentRepository::new(
                    connect_test_adapter(&database_url)
                        .expect("claim worker connection should initialize"),
                );
                start.wait();
                repository.claim_task_runs(&ClaimTaskRunsRequest::bounded_with_tenant_limit(
                    format!("worker.claim.{index}"),
                    "2026-08-01T00:00:01.000Z",
                    60,
                    2,
                    3,
                ))
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    let claims = workers
        .into_iter()
        .flat_map(|worker| {
            worker
                .join()
                .expect("claim worker thread should not panic")
                .expect("claim should succeed")
        })
        .collect::<Vec<_>>();
    let unique_runs = claims
        .iter()
        .map(|claim| claim.run.run_id.as_str())
        .collect::<HashSet<_>>();
    let unique_attempts = claims
        .iter()
        .map(|claim| claim.attempt.attempt_id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(claims.len(), 3);
    assert_eq!(unique_runs.len(), claims.len());
    assert_eq!(unique_attempts.len(), claims.len());
    for task in [&task_a, &task_b] {
        assert!(
            claims
                .iter()
                .filter(|claim| claim.run.task_id == task.task_id)
                .count()
                <= 2
        );
        assert!(
            scalar_count(
                &database_url,
                "SELECT COUNT(*)::bigint FROM ai_agent_task_run WHERE task_id = $1 AND status IN (1, 2)",
                &task.task_id,
            ) <= 2
        );
    }
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_expired_lease_recovery_fences_the_previous_worker() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let (agent_id, session_id, repository) = create_live_scheduler_context(&database_url, suffix);
    let task = live_scheduler_task(suffix, "lease", &agent_id, &session_id, 1)
        .expect("scheduler Task should build");
    repository
        .insert_task(task.clone())
        .expect("scheduler Task should persist");
    let original = repository
        .create_manual_task_run(
            &task,
            &format!("manual.lease.{suffix}"),
            "2026-08-01T00:00:00.000Z",
        )
        .expect("manual scheduler Run should persist");
    let first = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.lease.first",
            "2026-08-01T00:00:01.000Z",
            10,
            1,
        ))
        .expect("first claim should succeed")
        .pop()
        .expect("first claim should exist");
    repository
        .mark_task_run_running(&first.lease, "2026-08-01T00:00:02.000Z")
        .expect("first claim should enter running");

    assert_eq!(
        repository
            .recover_expired_task_run_leases("2026-08-01T00:00:12.000Z", 10)
            .expect("expired lease recovery should succeed"),
        1
    );
    let stale_heartbeat =
        repository.heartbeat_task_run(&first.lease, "2026-08-01T00:00:13.000Z", 10);
    assert!(stale_heartbeat.is_err());

    let second = repository
        .claim_task_runs(&ClaimTaskRunsRequest::bounded(
            "worker.lease.second",
            "2026-08-01T00:00:13.000Z",
            10,
            1,
        ))
        .expect("second claim should succeed")
        .pop()
        .expect("second claim should exist");
    assert_eq!(second.run.run_id, original.run_id);
    assert_eq!(second.run.turn_id, original.turn_id);
    assert_eq!(second.run.attempt_count, 2);
    assert!(second.lease.fencing_token > first.lease.fencing_token);

    let attempts = repository
        .list_task_run_attempts(&TaskRunAttemptListQuery::for_run(
            700_001,
            0,
            &original.run_id,
        ))
        .expect("Run attempts should be queryable");
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].status, AgentTaskRunAttemptStatus::Claimed);
    assert_eq!(attempts[1].status, AgentTaskRunAttemptStatus::LeaseExpired);
    assert!(attempts[0].fencing_token > attempts[1].fencing_token);
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_materialization_and_outbox_are_one_atomic_transaction() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let (agent_id, session_id, repository) = create_live_scheduler_context(&database_url, suffix);
    let task = live_scheduler_task(suffix, "atomic", &agent_id, &session_id, 1)
        .expect("scheduler Task should build");
    repository
        .insert_task(task.clone())
        .expect("scheduler Task should persist");
    let adapter = connect_test_adapter(&database_url).expect("trigger adapter should connect");
    adapter
        .pool()
        .run(
            sqlx::query(
                r#"
CREATE FUNCTION reject_task_run_materialized_outbox() RETURNS trigger
LANGUAGE plpgsql AS $function$
BEGIN
    IF NEW.event_type = 'agent.task.run.materialized' THEN
        RAISE EXCEPTION 'injected outbox failure';
    END IF;
    RETURN NEW;
END;
$function$
                "#,
            )
            .execute(adapter.pool().pool()),
        )
        .expect("outbox failure function should install");
    adapter
        .pool()
        .run(
            sqlx::query(
                "CREATE TRIGGER reject_task_run_materialized_outbox \
                 BEFORE INSERT ON ai_agent_outbox_event \
                 FOR EACH ROW EXECUTE FUNCTION reject_task_run_materialized_outbox()",
            )
            .execute(adapter.pool().pool()),
        )
        .expect("outbox failure trigger should install");

    let failed = repository.materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
        "2026-08-01T00:00:00.000Z",
        10,
    ));
    assert!(failed.is_err());
    assert_eq!(
        scalar_count(
            &database_url,
            "SELECT COUNT(*)::bigint FROM ai_agent_task_run WHERE task_id = $1",
            &task.task_id,
        ),
        0
    );
    let unchanged = repository
        .get_task(700_001, 0, &task.task_id)
        .expect("Task should be queryable")
        .expect("Task should exist");
    assert_eq!(unchanged.version, 0);
    assert_eq!(unchanged.next_fire_at, task.next_fire_at);

    adapter
        .pool()
        .run(
            sqlx::query(
                "DROP TRIGGER reject_task_run_materialized_outbox ON ai_agent_outbox_event",
            )
            .execute(adapter.pool().pool()),
        )
        .expect("outbox failure trigger should be removed");
    adapter
        .pool()
        .run(
            sqlx::query("DROP FUNCTION reject_task_run_materialized_outbox()")
                .execute(adapter.pool().pool()),
        )
        .expect("outbox failure function should be removed");
    let materialized = repository
        .materialize_due_tasks(&MaterializeDueTasksRequest::bounded(
            "2026-08-01T00:00:00.000Z",
            10,
        ))
        .expect("materialization should succeed after removing trigger");
    assert_eq!(materialized.len(), 1);
    assert_eq!(
        scalar_count(
            &database_url,
            "SELECT COUNT(*)::bigint FROM ai_agent_outbox_event WHERE event_type = 'agent.task.run.materialized' AND payload_json->>'taskId' = $1",
            &task.task_id,
        ),
        1
    );
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_partial_schema_fails_closed_without_auto_migrate_authorization() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let (isolated_database, _database_url, isolated) =
        create_isolated_database(&base_database_url, suffix);
    isolated
        .pool()
        .run(
            sqlx::query("CREATE TABLE ai_agent_outbox_event (id BIGINT PRIMARY KEY)")
                .execute(isolated.pool().pool()),
        )
        .expect("partial schema completion anchor should be created");

    let bootstrap =
        isolated
            .pool()
            .block_on(sdkwork_agents_database_host::bootstrap_agents_database(
                isolated.pool().database_pool().clone(),
            ));
    let error = match bootstrap {
        Ok(_) => panic!("partial schema must not bootstrap when auto-migrate is disabled"),
        Err(error) => error,
    };

    assert!(error.contains("agents database schema is incomplete"));
    assert!(error.contains("missing table: ai_agent"));
    assert_eq!(
        authoritative_agent_table_count(&isolated, &isolated_database.schema),
        1,
        "init must preserve an anchored partial schema for drift rejection instead of replaying the greenfield baseline",
    );
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_resource_user_state_round_trip_and_stale_write_rollback() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let repository = SqlAgentRepository::new(
        connect_test_adapter(&database_url).expect("postgres adapter should connect"),
    );
    let service = AgentsService::new(
        repository,
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.user-state.postgres-live"),
    );
    let agent_id = format!("agent.live.{suffix}");
    let session_id = format!("session.live.{suffix}");

    service
        .create_agent(CreateAgentCommand {
            agent_id: agent_id.clone(),
            tenant_id: 700_001,
            organization_id: 0,
            owner_user_id: 700,
            code: format!("live-{suffix}"),
            display_name: "Postgres User State".to_string(),
            description: None,
            manifest: manifest(&agent_id),
            visibility: AgentVisibility::Private,
            tags: Vec::new(),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:00Z".to_string(),
        })
        .unwrap();
    let created_agent = service
        .get_agent(GetAgentCommand {
            tenant_id: 700_001,
            agent_id: agent_id.clone(),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(created_agent.agent_id, agent_id);
    let project_id = format!("project.live.{suffix}");
    service
        .create_project(CreateProjectCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            workspace_id: None,
            owner_user_id: 700,
            name: "Postgres composition project".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::ExplicitResources,
            default_agent_id: Some(agent_id.clone()),
            default_model_id: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:10Z".to_string(),
        })
        .unwrap();
    let projects = service
        .list_projects(ListProjectsCommand {
            query: ProjectListQuery::for_organization(700_001, 0).for_owner(700),
            requested_by: subject(),
        })
        .expect("live projects should be listable after project creation");
    assert!(
        projects
            .items
            .iter()
            .any(|project| project.project_id == project_id),
        "the newly created project should be visible in its owner-scoped project list"
    );
    let project_slot = service
        .create_project_composition_slot(CreateProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: "slot.project.instructions".to_string(),
            slot_kind: AgentCompositionSlotKind::Prompt,
            target_module: AgentCompositionTargetModule::Prompts,
            target_ref: format!("prompt.live.{suffix}"),
            target_version_ref: Some("version.1".to_string()),
            priority: -1000,
            enabled: true,
            policy_json: "{\"role\":\"system\"}".to_string(),
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:20Z".to_string(),
        })
        .unwrap();
    let mut slot_query =
        ProjectCompositionSlotListQuery::for_project(700_001, 0, project_id.clone());
    slot_query.slot_kind = Some(AgentCompositionSlotKind::Prompt);
    slot_query.enabled = Some(true);
    let project_slots = service
        .list_project_composition_slots(ListProjectCompositionSlotsCommand {
            query: slot_query,
            owner_scope: Some(700),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(project_slots.items.len(), 1);

    let stale_slot_update =
        service.update_project_composition_slot(UpdateProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: project_slot.slot_id.clone(),
            expected_version: Some(9),
            slot_kind: None,
            target_module: None,
            target_ref: None,
            target_version_ref: None,
            priority: None,
            enabled: Some(false),
            policy_json: None,
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:30Z".to_string(),
        });
    assert!(stale_slot_update.is_err());
    let updated_slot = service
        .update_project_composition_slot(UpdateProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: project_slot.slot_id.clone(),
            expected_version: Some(0),
            slot_kind: None,
            target_module: None,
            target_ref: None,
            target_version_ref: None,
            priority: None,
            enabled: Some(false),
            policy_json: None,
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:40Z".to_string(),
        })
        .unwrap();
    assert_eq!(updated_slot.version, 1);
    service
        .delete_project_composition_slot(DeleteProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: project_slot.slot_id,
            expected_version: Some(1),
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:50Z".to_string(),
        })
        .unwrap();
    service
        .create_session(CreateSessionCommand {
            tenant_id: 700_001,
            organization_id: 0,
            agent_id: agent_id.clone(),
            owner_user_id: 700,
            session_id: session_id.clone(),
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: None,
            idempotency_key: None,
            payload_hash: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:01:00Z".to_string(),
        })
        .unwrap();

    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 700_001,
            agent_id: agent_id.clone(),
            binding_id: format!("binding.live.{suffix}"),
            provider_id: format!("provider.live.{suffix}"),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: format!("profile.live.{suffix}"),
            capabilities: Vec::new(),
            make_default: true,
            requested_by: subject(),
            requested_at: "2026-07-19T00:01:10Z".to_string(),
        })
        .unwrap();
    let provider_session_id = format!("provider_session.live.{suffix}");
    let provider_session_tree_id = format!("provider_session_tree.live.{suffix}");
    let provider_parent_session_id = format!("provider_session.parent.live.{suffix}");
    let provider_forked_from_session_id = format!("provider_session.fork.live.{suffix}");
    let runtime_binding = service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 700_001,
            organization_id: 0,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            runtime_binding_id: Some(format!("runtime_binding.live.{suffix}")),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: provider_binding.binding_id,
            model_id: format!("model.live.{suffix}"),
            provider_id: provider_binding.provider_id,
            provider_session_id: Some(provider_session_id.clone()),
            provider_session_tree_id: Some(provider_session_tree_id.clone()),
            provider_parent_session_id: Some(provider_parent_session_id.clone()),
            provider_forked_from_session_id: Some(provider_forked_from_session_id.clone()),
            provider_directory: None,
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-19T00:01:20Z".to_string(),
        })
        .unwrap();
    let runtime_bindings = service
        .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
            query: SessionRuntimeBindingListQuery::for_session(700_001, 0, session_id.clone())
                .current_only(),
            path_agent_id: agent_id.clone(),
            owner_scope: Some(700),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(runtime_bindings.total_count, Some(1));
    assert_eq!(runtime_bindings.items.len(), 1);
    assert_eq!(
        runtime_bindings.items[0].runtime_binding_id,
        runtime_binding.runtime_binding_id,
    );
    assert_eq!(
        runtime_bindings.items[0].provider_session_id.as_deref(),
        Some(provider_session_id.as_str()),
    );
    assert_eq!(
        runtime_bindings.items[0]
            .provider_session_tree_id
            .as_deref(),
        Some(provider_session_tree_id.as_str()),
    );
    assert_eq!(
        runtime_bindings.items[0]
            .provider_parent_session_id
            .as_deref(),
        Some(provider_parent_session_id.as_str()),
    );
    assert_eq!(
        runtime_bindings.items[0]
            .provider_forked_from_session_id
            .as_deref(),
        Some(provider_forked_from_session_id.as_str()),
    );

    let created = service
        .update_session_user_state(UpdateSessionUserStateCommand {
            tenant_id: 700_001,
            organization_id: 0,
            user_id: 700,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            pinned: Some(true),
            hidden: None,
            mark_opened: true,
            last_read_item_sequence: Some(0),
            custom_title: None,
            expected_version: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(created.version, 0);

    let states = service
        .list_session_user_states(ListSessionUserStatesCommand {
            query: ResourceUserStateListQuery::for_user_sessions(700_001, 0, 700).pinned_only(),
            path_agent_id: agent_id.clone(),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(states.items.len(), 1);
    assert_eq!(states.items[0].resource_id, session_id);

    let item_id = format!("item.live.{suffix}");
    service
        .create_session_item(CreateSessionItemCommand {
            tenant_id: 700_001,
            organization_id: 0,
            session_id: session_id.clone(),
            item_id: item_id.clone(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: "Live answer".to_string(),
            content_type: "text/plain".to_string(),
            input_tokens: 0,
            output_tokens: 2,
            model_id: None,
            provider_id: None,
            provider_payload_json: None,
            parent_item_id: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:01Z".to_string(),
        })
        .unwrap();
    let feedback = service
        .update_item_feedback(UpdateItemFeedbackCommand {
            tenant_id: 700_001,
            organization_id: 0,
            user_id: 700,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            item_id: item_id.clone(),
            rating: Some(AgentItemFeedbackRating::Up),
            reason_code: None,
            comment: None,
            expected_version: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:02Z".to_string(),
        })
        .unwrap();
    assert_eq!(feedback.version, 0);
    let feedback_page = service
        .list_item_feedback(ListItemFeedbackCommand {
            query: ItemFeedbackListQuery::for_user_session(700_001, 0, 700, session_id.clone()),
            path_agent_id: agent_id.clone(),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(feedback_page.items.len(), 1);
    assert_eq!(feedback_page.items[0].item_id, item_id);

    let drive_ref = AgentItemDriveRefInput {
        resource_role: AgentItemResourceRole::Image,
        drive_space_id: format!("space-live-{suffix}"),
        drive_node_id: format!("node-live-{suffix}"),
    };
    let turn = service
        .execute_turn(CreateTurnCommand {
            tenant_id: 700_001,
            organization_id: 0,
            agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            turn_id: Some(format!("turn.live.{suffix}")),
            content: "Describe this image".to_string(),
            content_type: "image/png".to_string(),
            turn_mode: AgentTurnMode::Interactive,
            runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
            requested_model_id: Some(runtime_binding.model_id.clone()),
            access_mode_id: None,
            idempotency_key: format!("live-drive-{suffix}"),
            payload_hash: format!("sha256:live-drive-{suffix}"),
            client_request_id: Some(format!("request-live-drive-{suffix}")),
            drive_refs: vec![drive_ref.clone()],
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:10Z".to_string(),
            prefer_stream: false,
            auth_token: None,
            wire_protocol: None,
        })
        .unwrap();
    assert_eq!(turn.session.item_count, 3);
    assert_eq!(turn.user_item_drive_refs.len(), 1);
    assert_eq!(
        turn.user_item_drive_refs[0].drive_node_id,
        drive_ref.drive_node_id
    );

    let page = service
        .list_session_items_with_drive_refs(ListSessionItemsCommand {
            query: SessionItemListQuery::for_session(700_001, 0, session_id.clone()),
            path_agent_id: agent_id.clone(),
            owner_scope: Some(700),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(page.items.len(), 3);
    assert_eq!(
        page.items
            .iter()
            .find(|item| item.item.item_id == turn.user_input_item.item_id)
            .unwrap()
            .drive_refs
            .len(),
        1
    );

    let mut invalid = drive_ref.clone();
    invalid.drive_node_id.clear();
    let invalid_result = service.execute_turn(CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        turn_id: None,
        content: "Invalid attachment".to_string(),
        content_type: "image/png".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        requested_model_id: Some(runtime_binding.model_id.clone()),
        access_mode_id: None,
        idempotency_key: format!("live-drive-invalid-{suffix}"),
        payload_hash: format!("sha256:live-drive-invalid-{suffix}"),
        client_request_id: None,
        drive_refs: vec![invalid],
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-19T00:02:11Z".to_string(),
        prefer_stream: false,
        auth_token: None,
        wire_protocol: None,
    });
    assert!(invalid_result.is_err());

    let duplicate_result = service.execute_turn(CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        turn_id: None,
        content: "Duplicate attachment".to_string(),
        content_type: "image/png".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        requested_model_id: Some(runtime_binding.model_id.clone()),
        access_mode_id: None,
        idempotency_key: format!("live-drive-duplicate-{suffix}"),
        payload_hash: format!("sha256:live-drive-duplicate-{suffix}"),
        client_request_id: None,
        drive_refs: vec![drive_ref.clone(), drive_ref],
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-19T00:02:12Z".to_string(),
        prefer_stream: false,
        auth_token: None,
        wire_protocol: None,
    });
    assert!(duplicate_result.is_err());
    let page_after_rejections = service
        .list_session_items_with_drive_refs(ListSessionItemsCommand {
            query: SessionItemListQuery::for_session(700_001, 0, session_id.clone()),
            path_agent_id: agent_id.clone(),
            owner_scope: Some(700),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(page_after_rejections.items.len(), 3);

    let failed_idempotency_key = format!("live-turn-failed-{suffix}");
    let failure_service = AgentsService::new(
        SqlAgentRepository::new(
            connect_test_adapter(&database_url).expect("failure adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-failure.postgres-live"),
    )
    .with_turn_executor(Arc::new(FailingTurnExecutor));
    let failed_result = failure_service.execute_turn(CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        turn_id: None,
        content: "Persist this provider failure".to_string(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        requested_model_id: Some(runtime_binding.model_id.clone()),
        access_mode_id: None,
        idempotency_key: failed_idempotency_key.clone(),
        payload_hash: format!("sha256:live-turn-failed-{suffix}"),
        client_request_id: None,
        drive_refs: Vec::new(),
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-19T00:02:20Z".to_string(),
        prefer_stream: false,
        auth_token: None,
        wire_protocol: None,
    });
    assert!(failed_result.is_err());
    let lifecycle_repository = SqlAgentRepository::new(
        connect_test_adapter(&database_url).expect("lifecycle adapter should connect"),
    );
    let failed_turn = lifecycle_repository
        .get_turn_by_idempotency(700_001, 0, 700, &failed_idempotency_key)
        .unwrap()
        .unwrap();
    assert_eq!(failed_turn.status, AgentTurnStatus::Failed);
    assert_eq!(failed_turn.version, 2);
    assert_eq!(
        failed_turn.error_code.as_deref(),
        Some("turn_inference_failed")
    );
    assert_eq!(
        failed_turn.error_detail.as_deref(),
        Some("managed turn inference failed")
    );

    let base_id = u64::try_from(suffix).unwrap();
    let cancellable_turn_id = format!("turn.cancel.{suffix}");
    let cancellable_turn = AgentTurnRecord {
        id: base_id.saturating_add(10_000),
        turn_id: cancellable_turn_id.clone(),
        tenant_id: 700_001,
        organization_id: 0,
        session_id: session_id.clone(),
        agent_id: agent_id.clone(),
        owner_user_id: 700,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        client_request_id: None,
        idempotency_key: format!("live-cancel-{suffix}"),
        payload_hash: format!("payload-cancel-{suffix}"),
        request_item_id: format!("item.cancel.{suffix}"),
        response_item_id: None,
        turn_mode: AgentTurnMode::Interactive,
        status: AgentTurnStatus::Requested,
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
        max_attempts: 3,
        next_retry_at: None,
        available_at: "2026-07-18T00:00:00Z".to_string(),
        lease_owner: None,
        lease_token: None,
        lease_expires_at: None,
        fencing_token: 0,
        version: 0,
        created_at: "2026-07-18T00:00:00Z".to_string(),
        updated_at: "2026-07-18T00:00:00Z".to_string(),
        started_at: None,
        completed_at: None,
        cancel_requested_at: None,
        cancelled_at: None,
        retention_until: None,
    };
    lifecycle_repository
        .insert_turn_request(
            cancellable_turn.clone(),
            request_item_for_turn(base_id.saturating_add(10_001), &cancellable_turn),
            Vec::new(),
        )
        .unwrap();
    let cancellation_service = AgentsService::new(
        SqlAgentRepository::new(
            connect_test_adapter(&database_url).expect("cancellation adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-cancel.postgres-live"),
    );
    let cancelled = cancellation_service
        .cancel_turn(CancelTurnCommand {
            tenant_id: 700_001,
            organization_id: 0,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            turn_id: cancellable_turn_id,
            expected_version: Some(0),
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:30Z".to_string(),
        })
        .unwrap();
    assert_eq!(cancelled.status, AgentTurnStatus::Cancelled);
    assert_eq!(cancelled.version, 1);
    assert!(cancelled.cancelled_at.is_some());

    let stale_turn_id = format!("turn.stale.{suffix}");
    let stale_turn = AgentTurnRecord {
        id: base_id.saturating_add(20_000),
        turn_id: stale_turn_id.clone(),
        tenant_id: 700_001,
        organization_id: 0,
        session_id: session_id.clone(),
        agent_id: agent_id.clone(),
        owner_user_id: 700,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        client_request_id: None,
        idempotency_key: format!("live-stale-{suffix}"),
        payload_hash: format!("payload-stale-{suffix}"),
        request_item_id: format!("item.stale.{suffix}"),
        response_item_id: None,
        turn_mode: AgentTurnMode::Interactive,
        status: AgentTurnStatus::Requested,
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
        max_attempts: 3,
        next_retry_at: None,
        available_at: "2026-07-18T00:00:00Z".to_string(),
        lease_owner: None,
        lease_token: None,
        lease_expires_at: None,
        fencing_token: 0,
        version: 0,
        created_at: "2026-07-18T00:00:00Z".to_string(),
        updated_at: "2026-07-18T00:00:00Z".to_string(),
        started_at: None,
        completed_at: None,
        cancel_requested_at: None,
        cancelled_at: None,
        retention_until: None,
    };
    lifecycle_repository
        .insert_turn_request(
            stale_turn.clone(),
            request_item_for_turn(base_id.saturating_add(20_001), &stale_turn),
            Vec::new(),
        )
        .unwrap();
    let reconciliation_service = AgentsService::new(
        SqlAgentRepository::new(
            connect_test_adapter(&database_url).expect("reconciliation adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-reconcile.postgres-live"),
    );
    let reconciliation = reconciliation_service
        .reconcile_stale_turns("2026-07-19T00:00:00Z", "2026-07-19T00:02:40Z", 100)
        .unwrap();
    assert_eq!(reconciliation.failed.len(), 1);
    assert_eq!(reconciliation.failed[0].turn_id, stale_turn_id);
    assert_eq!(reconciliation.failed[0].status, AgentTurnStatus::Failed);

    let stale = service.update_session_user_state(UpdateSessionUserStateCommand {
        tenant_id: 700_001,
        organization_id: 0,
        user_id: 700,
        path_agent_id: agent_id,
        session_id,
        pinned: Some(false),
        hidden: None,
        mark_opened: false,
        last_read_item_sequence: None,
        custom_title: None,
        expected_version: Some(9),
        requested_by: subject(),
        requested_at: "2026-07-19T00:03:00Z".to_string(),
    });
    assert!(stale.is_err());
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_project_create_persists_sql_audit_events() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let repository_adapter =
        connect_test_adapter(&database_url).expect("postgres adapter should connect");
    let audit_adapter = repository_adapter.clone();
    let service = AgentsService::new(
        SqlAgentRepository::new(repository_adapter),
        SqlAgentAuditSink::new_global(audit_adapter),
        IamGatedPolicyProvider::new("policy.agents.project-audit.postgres-live"),
    );

    let project = service
        .create_project(CreateProjectCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: format!("project.audit.{suffix}"),
            workspace_id: None,
            owner_user_id: 700,
            name: "Postgres audited project".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::ExplicitResources,
            default_agent_id: None,
            default_model_id: None,
            requested_by: numeric_subject(),
            requested_at: "2026-07-27T00:00:00Z".to_string(),
        })
        .expect("project creation should persist workspace and project audit events");

    assert_eq!(project.owner_user_id, 700);
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_turn_idempotency_and_session_sequences_are_concurrency_safe() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let (service, agent_id, provider_binding_id, provider_id) =
        create_live_turn_service(&database_url, suffix, None);

    let (idempotent_session_id, idempotent_runtime_id, idempotent_model_id) =
        create_live_turn_session(
            service.as_ref(),
            &agent_id,
            &provider_binding_id,
            &provider_id,
            suffix,
            "idempotent",
        );
    let idempotent_command = live_turn_command(
        LiveTurnTarget {
            agent_id: &agent_id,
            session_id: &idempotent_session_id,
            runtime_binding_id: &idempotent_runtime_id,
            model_id: &idempotent_model_id,
        },
        format!("turn.idempotent.{suffix}"),
        format!("idempotency.concurrent.{suffix}"),
        format!("sha256:idempotency.concurrent.{suffix}"),
        "concurrent idempotent request",
    );
    let start = Arc::new(Barrier::new(3));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let service = Arc::clone(&service);
        let command = idempotent_command.clone();
        let start = Arc::clone(&start);
        workers.push(thread::spawn(move || {
            start.wait();
            service.execute_turn(command)
        }));
    }
    start.wait();
    let outcomes = workers
        .into_iter()
        .map(|worker| worker.join().expect("idempotency worker should not panic"))
        .collect::<Vec<_>>();
    assert!(outcomes.iter().any(Result::is_ok));
    assert!(outcomes.iter().all(|outcome| {
        outcome.is_ok()
            || outcome
                .as_ref()
                .is_err_and(|error| error.kind() == KernelErrorKind::Conflict)
    }));

    let (session, items, turns) = read_session_aggregate(&database_url, &idempotent_session_id);
    assert_eq!(session.item_count, 2);
    assert_eq!(session.last_item_sequence, 2);
    assert_eq!(items.len(), 2);
    assert_eq!(turns.len(), 1);
    assert_eq!(
        items.iter().map(|item| item.sequence).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(turns[0].status, AgentTurnStatus::Completed);

    service
        .execute_turn(idempotent_command.clone())
        .expect("same idempotency key and payload should replay the completed turn");
    let mut conflicting_payload = idempotent_command;
    conflicting_payload.payload_hash = format!("sha256:different.{suffix}");
    let conflict = service
        .execute_turn(conflicting_payload)
        .expect_err("same idempotency key with a different payload must conflict");
    assert_eq!(conflict.kind(), KernelErrorKind::Conflict);
    let (session, items, turns) = read_session_aggregate(&database_url, &idempotent_session_id);
    assert_eq!((session.item_count, session.last_item_sequence), (2, 2));
    assert_eq!((items.len(), turns.len()), (2, 1));

    let (scope_a_session_id, scope_a_runtime_id, scope_a_model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "scope_a",
    );
    let (scope_b_session_id, scope_b_runtime_id, scope_b_model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "scope_b",
    );
    let shared_key = format!("idempotency.cross_session.{suffix}");
    let scope_commands = vec![
        live_turn_command(
            LiveTurnTarget {
                agent_id: &agent_id,
                session_id: &scope_a_session_id,
                runtime_binding_id: &scope_a_runtime_id,
                model_id: &scope_a_model_id,
            },
            format!("turn.scope_a.{suffix}"),
            shared_key.clone(),
            format!("sha256:scope_a.{suffix}"),
            "cross-session contender A",
        ),
        live_turn_command(
            LiveTurnTarget {
                agent_id: &agent_id,
                session_id: &scope_b_session_id,
                runtime_binding_id: &scope_b_runtime_id,
                model_id: &scope_b_model_id,
            },
            format!("turn.scope_b.{suffix}"),
            shared_key,
            format!("sha256:scope_b.{suffix}"),
            "cross-session contender B",
        ),
    ];
    let start = Arc::new(Barrier::new(3));
    let workers = scope_commands
        .into_iter()
        .map(|command| {
            let service = Arc::clone(&service);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                start.wait();
                service.execute_turn(command)
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    let outcomes = workers
        .into_iter()
        .map(|worker| worker.join().expect("scope worker should not panic"))
        .collect::<Vec<_>>();
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                outcome
                    .as_ref()
                    .is_err_and(|error| error.kind() == KernelErrorKind::Conflict)
            })
            .count(),
        1
    );
    let scope_a = read_session_aggregate(&database_url, &scope_a_session_id);
    let scope_b = read_session_aggregate(&database_url, &scope_b_session_id);
    assert_eq!(scope_a.0.item_count + scope_b.0.item_count, 2);
    assert_eq!(
        scope_a.0.last_item_sequence + scope_b.0.last_item_sequence,
        2
    );
    assert_eq!(scope_a.1.len() + scope_b.1.len(), 2);
    assert_eq!(scope_a.2.len() + scope_b.2.len(), 1);
    assert!(
        (scope_a.0.item_count == 2 && scope_b.0.item_count == 0)
            || (scope_a.0.item_count == 0 && scope_b.0.item_count == 2)
    );

    let (sequence_session_id, sequence_runtime_id, sequence_model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "sequence",
    );
    let sequence_commands = (0..2)
        .map(|index| {
            live_turn_command(
                LiveTurnTarget {
                    agent_id: &agent_id,
                    session_id: &sequence_session_id,
                    runtime_binding_id: &sequence_runtime_id,
                    model_id: &sequence_model_id,
                },
                format!("turn.sequence.{index}.{suffix}"),
                format!("idempotency.sequence.{index}.{suffix}"),
                format!("sha256:sequence.{index}.{suffix}"),
                format!("parallel sequence request {index}"),
            )
        })
        .collect::<Vec<_>>();
    let start = Arc::new(Barrier::new(3));
    let workers = sequence_commands
        .into_iter()
        .map(|command| {
            let service = Arc::clone(&service);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                start.wait();
                service.execute_turn(command)
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    for worker in workers {
        worker
            .join()
            .expect("sequence worker should not panic")
            .expect("different idempotency keys should both complete");
    }
    let (session, items, turns) = read_session_aggregate(&database_url, &sequence_session_id);
    assert_eq!((session.item_count, session.last_item_sequence), (4, 4));
    assert_eq!((items.len(), turns.len()), (4, 2));
    assert_eq!(
        items.iter().map(|item| item.sequence).collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

#[test]
#[ignore = "requires SDKWORK_DATABASE_URL and SDKWORK_DATABASE_ADMIN_* for ephemeral test provisioning"]
fn postgres_cancel_wins_completion_race_without_partial_response_state() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url =
        std::env::var(DATABASE_URL_ENV).expect("SDKWORK_DATABASE_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_database, database_url) =
        bootstrap_isolated_database(&base_database_url, suffix);
    let blocking_executor = Arc::new(BlockingTurnExecutor::default());
    let (service, agent_id, provider_binding_id, provider_id) =
        create_live_turn_service(&database_url, suffix, Some(blocking_executor.clone()));
    let (session_id, runtime_binding_id, model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "cancel_race",
    );
    let turn_id = format!("turn.cancel_race.{suffix}");
    let command = live_turn_command(
        LiveTurnTarget {
            agent_id: &agent_id,
            session_id: &session_id,
            runtime_binding_id: &runtime_binding_id,
            model_id: &model_id,
        },
        turn_id.clone(),
        format!("idempotency.cancel_race.{suffix}"),
        format!("sha256:cancel_race.{suffix}"),
        "cancel this turn while its provider is running",
    );
    let execution_service = Arc::clone(&service);
    let execution = thread::spawn(move || execution_service.execute_turn(command));
    blocking_executor.wait_until_started();

    let cancellation = service.cancel_turn(CancelTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        path_agent_id: agent_id,
        session_id: session_id.clone(),
        turn_id: turn_id.clone(),
        expected_version: Some(1),
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-20T00:00:05Z".to_string(),
    });
    blocking_executor.release();
    let completion = execution
        .join()
        .expect("completion worker should not panic")
        .expect_err("completion must lose after cancellation commits");
    assert_eq!(completion.kind(), KernelErrorKind::Conflict);
    let cancelled = cancellation.expect("running turn should be cancelled");
    assert_eq!(cancelled.status, AgentTurnStatus::Cancelled);
    assert_eq!(cancelled.version, 2);

    let (session, items, turns) = read_session_aggregate(&database_url, &session_id);
    assert_eq!((session.item_count, session.last_item_sequence), (1, 1));
    assert_eq!(
        (session.total_input_tokens, session.total_output_tokens),
        (0, 0)
    );
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, AgentSessionItemKind::UserInput);
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].turn_id, turn_id);
    assert_eq!(turns[0].status, AgentTurnStatus::Cancelled);
    assert!(turns[0].response_item_id.is_none());
    assert_eq!((turns[0].input_tokens, turns[0].output_tokens), (0, 0));
}
