use std::sync::{Arc, Mutex};

use sdkwork_agent_kernel::{AgentManifest, KernelEvent, KernelResult, PolicySubject};
use sdkwork_code_kernel::CodeTaskIntent;
use sdkwork_intelligence_agents_service::{
    ActivateAgentProviderBindingCommand, AgentAuditSink, AgentBusinessStatus,
    AgentImplementationKind, AgentItemDriveRefInput, AgentProviderBindingCommand,
    AgentSessionEntrySurface, AgentSessionKind, AgentTurnMode, AgentVisibility, AgentsService,
    AuditEventListQuery, ChangeAgentStatusCommand, CreateAgentCommand, CreateSessionCommand,
    CreateSessionRuntimeBindingCommand, CreateTurnCommand, GetSessionRuntimeBindingCommand,
    IamGatedPolicyProvider, InMemoryAgentRepository, PaginatedResult, RuntimeFacadeTurnExecutor,
    TurnExecutionStreamSink,
};

/// Live end-to-end proof: one real provider turn through the agents business
/// service stream sink (the same path the HTTP turns SSE endpoint uses).
///
/// Run with, for example:
/// `OPENCODE_MODEL=opencode/deepseek-v4-flash-free cargo test -p sdkwork-intelligence-agents-service --test live_turns_sse_test -- --ignored --nocapture`
#[test]
#[ignore = "requires live provider SDK packages and reachable model providers"]
fn live_turns_sse_flow_with_real_opencode_provider() {
    std::env::set_var("SDKWORK_KERNEL_ENVIRONMENT", "development");
    run_live_turns_flow(LiveProviderConfig {
        label: "opencode",
        agent_id: "agent.live.opencode",
        code: "live-opencode",
        display_name: "Live OpenCode",
        binding_id: "binding.opencode",
        provider_id: "provider.opencode",
        model_id: configured_opencode_model_id(),
    });
}

/// Same end-to-end proof through the official Claude agent SDK (`query()`),
/// which also live-exercises the Claude streaming projection over the turns
/// stream sink for the first time.
#[test]
#[ignore = "requires the @anthropic-ai/claude-agent-sdk package and a live Claude model provider"]
fn live_turns_sse_flow_with_real_claude_code_provider() {
    std::env::set_var("SDKWORK_KERNEL_ENVIRONMENT", "development");
    run_live_turns_flow(LiveProviderConfig {
        label: "claude-code",
        agent_id: "agent.live.claude",
        code: "live-claude",
        display_name: "Live Claude Code",
        binding_id: "binding.claude-code",
        provider_id: "provider.claude-code",
        model_id: configured_claude_code_model_id(),
    });
}

struct LiveProviderConfig {
    label: &'static str,
    agent_id: &'static str,
    code: &'static str,
    display_name: &'static str,
    binding_id: &'static str,
    provider_id: &'static str,
    model_id: String,
}

fn run_live_turns_flow(config: LiveProviderConfig) {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = IamGatedPolicyProvider::new("policy.agents.test.iam-gated");
    let service = AgentsService::new(repository, audit_sink, policy_provider)
        .with_turn_executor(Arc::new(RuntimeFacadeTurnExecutor));

    let created = service
        .create_agent(create_agent_cmd(
            config.agent_id,
            100_001,
            0,
            100,
            config.code,
            config.display_name,
            "2026-08-01T00:00:00Z",
        ))
        .expect("create should succeed");
    service
        .change_status(ChangeAgentStatusCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            expected_version: Some(created.version),
            target_status: AgentBusinessStatus::Active,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:00:30Z".to_string(),
        })
        .expect("activate agent should succeed");

    let session = service
        .create_session(CreateSessionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            agent_id: created.agent_id.clone(),
            owner_user_id: 100,
            session_id: String::new(),
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Live SSE session".to_string()),
            idempotency_key: None,
            payload_hash: None,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:00Z".to_string(),
        })
        .expect("create session should succeed");

    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            binding_id: config.binding_id.to_string(),
            provider_id: config.provider_id.to_string(),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: format!("profile.live.{}", config.label),
            capabilities: vec!["model.chat".to_string()],
            make_default: true,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:10Z".to_string(),
        })
        .expect("provider binding should be created");
    service
        .activate_provider_binding(ActivateAgentProviderBindingCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            binding_id: provider_binding.binding_id.clone(),
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:15Z".to_string(),
        })
        .expect("provider binding should be activated");

    let runtime_binding_id = format!("runtime_binding.live.{}", config.label);
    let _runtime_binding = service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: created.agent_id.clone(),
            session_id: session.session_id.clone(),
            runtime_binding_id: Some(runtime_binding_id.clone()),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: provider_binding.binding_id,
            model_id: config.model_id.clone(),
            provider_id: config.provider_id.to_string(),
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            provider_directory: None,
            owner_scope: None,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:20Z".to_string(),
        })
        .expect("session runtime binding should be created");

    let sink = RecordingTurnStreamSink::new();
    let turn_command = CreateTurnCommand {
        tenant_id: 100_001,
        organization_id: 0,
        agent_id: created.agent_id.clone(),
        session_id: session.session_id.clone(),
        turn_id: Some("turn.live.sse.one".to_string()),
        content: "Reply with exactly one word: OK".to_string(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding_id.clone()),
        requested_model_id: Some(config.model_id.clone()),
        access_mode_id: None,
        idempotency_key: "live-sse-idempotency-1".to_string(),
        payload_hash: "sha256:live-sse-payload-1".to_string(),
        client_request_id: Some("request.live.sse.1".to_string()),
        drive_refs: Vec::<AgentItemDriveRefInput>::new(),
        owner_scope: None,
        requested_by: sample_subject(),
        requested_at: "2026-08-01T00:01:30Z".to_string(),
        prefer_stream: true,
        auth_token: None,
        wire_protocol: None,
    };
    let result = service
        .execute_turn_with_stream_sink(turn_command.clone(), Arc::new(sink.clone()))
        .expect("live turn execution should succeed");

    let runtime_binding_after_turn = service
        .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: created.agent_id.clone(),
            session_id: session.session_id.clone(),
            runtime_binding_id: runtime_binding_id.clone(),
            owner_scope: None,
            requested_by: sample_subject(),
        })
        .expect("runtime binding must be readable after the turn");
    let provider_session_id = runtime_binding_after_turn
        .provider_session_id
        .clone()
        .expect("live turn must persist the provider session id into the runtime binding");

    eprintln!(
        "live_turns_sse_phase={}_stream_sink begin={} deltas={} events={}",
        config.label,
        sink.begin_count(),
        sink.deltas().len(),
        sink.events().len(),
    );
    eprintln!(
        "live_turns_sse_phase={}_result content={:?} provider_session_id={provider_session_id:?} deltas={} events={}",
        config.label,
        result
            .assistant_output_item
            .content
            .as_deref()
            .map(str::trim),
        result.stream_deltas.len(),
        result.stream_events.len(),
    );

    let content = result
        .assistant_output_item
        .content
        .as_deref()
        .expect("live turn must produce assistant content")
        .trim()
        .to_string();
    assert!(
        !content.is_empty() && content.to_ascii_uppercase().contains("OK"),
        "live {} turn must return the real model reply: {content:?}",
        config.label
    );
    assert!(
        !provider_session_id.is_empty(),
        "live turn must return a provider session id for resumption"
    );
    assert!(
        !result.stream_events.is_empty(),
        "live streamed turn must emit kernel events through the stream sink"
    );
    assert!(
        sink.begin_count() >= 1,
        "stream sink must observe the turn begin"
    );
    assert!(
        sink.events().len() >= result.stream_events.len(),
        "stream sink must observe the same kernel events as the turn result"
    );
    let terminal_events = ["agent.turn.started", "agent.turn.completed"];
    for terminal in terminal_events {
        assert!(
            result
                .stream_events
                .iter()
                .any(|event| event.event_type == terminal),
            "live turn stream must include {terminal}"
        );
    }

    // Second turn resumes the exact provider session through the same service.
    let resumed_sink = RecordingTurnStreamSink::new();
    let resumed = service
        .execute_turn_with_stream_sink(
            CreateTurnCommand {
                turn_id: Some("turn.live.sse.two".to_string()),
                content: "Reply with exactly one word: DONE".to_string(),
                idempotency_key: "live-sse-idempotency-2".to_string(),
                payload_hash: "sha256:live-sse-payload-2".to_string(),
                client_request_id: Some("request.live.sse.2".to_string()),
                requested_at: "2026-08-01T00:02:30Z".to_string(),
                auth_token: None,
                ..turn_command.clone()
            },
            Arc::new(resumed_sink.clone()),
        )
        .expect("live resumed turn should succeed");
    let resumed_content = resumed
        .assistant_output_item
        .content
        .as_deref()
        .expect("resumed live turn must produce assistant content")
        .trim()
        .to_string();
    assert!(
        !resumed_content.is_empty() && resumed_content.to_ascii_uppercase().contains("DONE"),
        "live resumed {} turn must return the real model reply: {resumed_content:?}",
        config.label
    );
    assert!(
        resumed_sink.events().len() >= 1,
        "resumed live turn must emit kernel events through the stream sink"
    );
    eprintln!(
        "live_turns_sse_phase={}_resume_ok content={resumed_content:?} deltas={} events={}",
        config.label,
        resumed.stream_deltas.len(),
        resumed.stream_events.len(),
    );
    eprintln!("live_turns_sse_phase={}_all_ok", config.label);
}

fn configured_opencode_model_id() -> String {
    std::env::var("OPENCODE_MODEL").unwrap_or_else(|_| {
        std::env::var("OPENCODE_CONFIG")
            .ok()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|json| {
                serde_json::from_str::<serde_json::Value>(&json)
                    .ok()
                    .and_then(|value| value.get("model")?.as_str().map(str::to_string))
            })
            // The durable v2 runner resolves models from the server's built-in
            // catalog; config-file providers are not part of that registry.
            .unwrap_or_else(|| "opencode/deepseek-v4-flash-free".to_string())
    })
}

fn configured_claude_code_model_id() -> String {
    std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| {
        let home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(std::path::PathBuf::from)
            .unwrap_or_default();
        std::fs::read_to_string(home.join(".claude").join("settings.json"))
            .ok()
            .and_then(|content| {
                serde_json::from_str::<serde_json::Value>(&content)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("env")?
                            .get("ANTHROPIC_MODEL")?
                            .as_str()
                            .map(str::to_string)
                    })
            })
            .unwrap_or_else(|| "gpt-5.6-sol".to_string())
    })
}

#[derive(Clone, Default)]
struct RecordingAuditSink {
    events: Arc<Mutex<Vec<KernelEvent>>>,
}

impl RecordingAuditSink {
    fn new() -> (Self, Arc<Mutex<Vec<KernelEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                events: events.clone(),
            },
            events,
        )
    }
}

impl AgentAuditSink for RecordingAuditSink {
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        self.events.lock().expect("audit lock").push(event);
        Ok(())
    }

    fn list_events(
        &self,
        _query: &AuditEventListQuery,
    ) -> KernelResult<PaginatedResult<KernelEvent>> {
        let items = self.events.lock().expect("audit lock").clone();
        Ok(PaginatedResult {
            items,
            next_page_token: None,
            total_count: None,
            has_more: false,
        })
    }
}

#[derive(Clone)]
struct RecordingTurnStreamSink {
    begin: Arc<Mutex<usize>>,
    deltas: Arc<Mutex<Vec<String>>>,
    events: Arc<Mutex<Vec<KernelEvent>>>,
}

impl RecordingTurnStreamSink {
    fn new() -> Self {
        Self {
            begin: Arc::new(Mutex::new(0)),
            deltas: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn begin_count(&self) -> usize {
        *self.begin.lock().expect("sink lock")
    }

    fn deltas(&self) -> Vec<String> {
        self.deltas.lock().expect("sink lock").clone()
    }

    fn events(&self) -> Vec<KernelEvent> {
        self.events.lock().expect("sink lock").clone()
    }
}

impl TurnExecutionStreamSink for RecordingTurnStreamSink {
    fn begin_turn(&self, _session_id: &str, _turn_id: &str) {
        *self.begin.lock().expect("sink lock") += 1;
    }

    fn push_delta(&self, delta: &str) {
        self.deltas
            .lock()
            .expect("sink lock")
            .push(delta.to_string());
    }

    fn push_event(&self, event: &KernelEvent) -> KernelResult<()> {
        self.events.lock().expect("sink lock").push(event.clone());
        Ok(())
    }
}

fn sample_manifest(agent_id: &str) -> AgentManifest {
    AgentManifest {
        schema_version: "1.0.0".to_string(),
        manifest_type: "agent".to_string(),
        agent_id: agent_id.to_string(),
        name: "live-provider".to_string(),
        display_name: "Live Provider".to_string(),
        description: "live provider e2e".to_string(),
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

fn sample_subject() -> PolicySubject {
    PolicySubject::new("u-1", "100001").with_role("ai.agents.manage")
}

fn create_agent_cmd(
    agent_id: &str,
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    code: &str,
    display_name: &str,
    requested_at: &str,
) -> CreateAgentCommand {
    CreateAgentCommand {
        agent_id: agent_id.to_string(),
        tenant_id,
        organization_id,
        owner_user_id,
        code: code.to_string(),
        display_name: display_name.to_string(),
        description: Some("live provider e2e".to_string()),
        manifest: sample_manifest(agent_id),
        visibility: AgentVisibility::Organization,
        tags: vec!["starter".to_string()],
        default_code_task_intent: Some(CodeTaskIntent::new("Refactor runtime")),
        implementation_provider_id: None,
        implementation_kind: None,
        implementation_type: None,
        requested_by: sample_subject(),
        requested_at: requested_at.to_string(),
    }
}
