use std::sync::{Arc, Mutex};

use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelErrorKind, KernelEvent, KernelResult, PolicySubject,
};
use sdkwork_code_kernel::CodeTaskIntent;
use sdkwork_intelligence_agents_service::{
    extract_event_context, offset_paginated_result, ActivateAgentProviderBindingCommand,
    AgentAuditSink, AgentBusinessStatus, AgentImplementationKind, AgentImplementationType,
    AgentInteractionKind, AgentItemDriveRefInput, AgentItemResourceRole, AgentListQuery,
    AgentPreviewResponseCommand, AgentPromptOptimizationCommand, AgentProviderBindingCommand,
    AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionKind, AgentTurnMode,
    AgentTurnStatus, AgentVisibility, AgentsService, ApproveInteractionCommand,
    AuditEventListQuery, ChangeAgentStatusCommand, ClaimInteractionCommand, CreateAgentCommand,
    CreateInteractionCommand, CreateSessionCommand, CreateSessionRuntimeBindingCommand,
    CreateTurnCommand, DeleteAgentCommand, DenyAllPolicyProvider, GetAgentCommand,
    GetInteractionCommand, GetSessionCommand, GetTurnByIdempotencyCommand, GetTurnCommand,
    IamGatedPolicyProvider, InMemoryAgentRepository, InteractionListQuery,
    ListAgentAuditEventsCommand, ListAgentsCommand, ListInteractionsCommand, PaginatedResult,
    PaginationParams, ProviderBindingListCommand, ProviderBindingListQuery,
    ResolveInteractionCommand, RestoreAgentCommand, UpdateAgentCommand,
    DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY, MAX_PAGE_SIZE,
};
use sdkwork_utils_rust::http_api::offset_limit_page_from_iter;

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
        self.events
            .lock()
            .expect("recording audit mutex poisoned")
            .push(event);
        Ok(())
    }

    fn list_events(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<PaginatedResult<KernelEvent>> {
        let mut matched: Vec<KernelEvent> = self
            .events
            .lock()
            .expect("recording audit mutex poisoned")
            .iter()
            .filter(|event| {
                extract_event_context(event.payload.as_str(), "tenant_id")
                    .and_then(|value| value.parse::<u64>().ok())
                    == Some(query.tenant_id)
                    && extract_event_context(event.payload.as_str(), "agent_id").as_deref()
                        == Some(query.agent_id.as_str())
            })
            .cloned()
            .collect();
        matched.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
        let total_count = matched.len() as u64;
        let page = offset_limit_page_from_iter(
            matched.into_iter(),
            query.pagination.page_size,
            query.pagination.offset,
        )
        .items;
        Ok(offset_paginated_result(
            page,
            &query.pagination,
            total_count,
        ))
    }
}

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

fn sample_subject() -> PolicySubject {
    PolicySubject::new("u-1", "100001").with_role("ai.agents.manage")
}

fn test_policy_provider() -> IamGatedPolicyProvider {
    IamGatedPolicyProvider::new("policy.agents.test.iam-gated")
}

fn test_deny_policy_provider() -> DenyAllPolicyProvider {
    DenyAllPolicyProvider::new("policy.agents.test.deny", "agent.business.denied")
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
        description: Some("sample".to_string()),
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

fn assert_structured_kind(error: KernelError, expected_kind: &str) {
    match error {
        KernelError::Structured { info } => {
            assert_eq!(info.kind.as_str(), expected_kind);
        }
        _ => panic!("expected structured error"),
    }
}

fn assert_agent_id_validation(error: KernelError) {
    match error {
        KernelError::Validation { message } => {
            assert_eq!(message, "agentId must start with agent.");
        }
        KernelError::Structured { info } => {
            panic!(
                "expected agentId validation error, got structured {}",
                info.kind.as_str()
            )
        }
        KernelError::Internal { .. }
        | KernelError::CapabilityMissing { .. }
        | KernelError::ProviderUnavailable { .. }
        | KernelError::PolicyDenied { .. } => panic!("expected agentId validation error"),
    }
}

#[test]
fn create_update_status_delete_restore_and_list_agents() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let create = service
        .create_agent(create_agent_cmd(
            "agent.alpha",
            100_001,
            0,
            100,
            "alpha",
            "Alpha",
            "2026-06-01T00:00:00Z",
        ))
        .expect("create should succeed");
    assert_eq!(create.status, AgentBusinessStatus::Draft);
    assert_eq!(create.visibility, AgentVisibility::Organization);
    assert!(create.id > (1_u64 << 22));
    assert!(create.id <= i64::MAX as u64);

    let updated = service
        .update_agent(UpdateAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            expected_version: Some(create.version),
            display_name: Some("Alpha v2".to_string()),
            description: Some("updated".to_string()),
            manifest: Some(AgentManifest {
                display_name: "Alpha Manifest v2".to_string(),
                optional_capabilities: vec![
                    "tool.invoke".to_string(),
                    "memory.retrieve".to_string(),
                ],
                ..sample_manifest("agent.alpha")
            }),
            visibility: Some(AgentVisibility::Tenant),
            tags: Some(vec!["starter".to_string(), "v2".to_string()]),
            default_code_task_intent: Some(CodeTaskIntent::new("Write tests first")),
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T00:10:00Z".to_string(),
        })
        .expect("update should succeed");
    assert_eq!(updated.display_name, "Alpha v2");
    assert_eq!(updated.manifest.display_name, "Alpha Manifest v2");
    assert_eq!(
        updated.manifest.optional_capabilities,
        vec!["tool.invoke".to_string(), "memory.retrieve".to_string()]
    );
    assert_eq!(updated.visibility, AgentVisibility::Tenant);

    let activated = service
        .change_status(ChangeAgentStatusCommand {
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            expected_version: Some(updated.version),
            target_status: AgentBusinessStatus::Active,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T00:15:00Z".to_string(),
        })
        .expect("status transition should succeed");
    assert_eq!(activated.status, AgentBusinessStatus::Active);

    let deleted = service
        .delete_agent(DeleteAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            expected_version: Some(activated.version),
            requested_by: sample_subject(),
            requested_at: "2026-06-01T00:16:00Z".to_string(),
        })
        .expect("delete should succeed");
    assert_eq!(deleted.status, AgentBusinessStatus::Deleted);
    assert!(deleted.deleted_at.is_some());

    let restored = service
        .restore_agent(RestoreAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            expected_version: Some(deleted.version),
            requested_by: sample_subject(),
            requested_at: "2026-06-01T00:17:00Z".to_string(),
        })
        .expect("restore should succeed");
    assert_eq!(restored.status, AgentBusinessStatus::Active);
    assert!(restored.deleted_at.is_none());

    let got = service
        .get_agent(GetAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            requested_by: sample_subject(),
        })
        .expect("retrieve should succeed");
    assert_eq!(got.agent_id, "agent.alpha");
    assert_eq!(got.version, restored.version);

    let listed = service
        .list_agents(ListAgentsCommand {
            query: AgentListQuery::for_tenant(100_001).for_organization(0),
            requested_by: sample_subject(),
        })
        .expect("list should succeed");
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].status, AgentBusinessStatus::Active);
}

#[test]
fn duplicate_agent_id_and_code_are_rejected() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    service
        .create_agent(create_agent_cmd(
            "agent.alpha",
            100_001,
            0,
            100,
            "alpha",
            "Alpha",
            "2026-06-01T01:00:00Z",
        ))
        .expect("first create should succeed");

    let duplicate_agent_id = service
        .create_agent(create_agent_cmd(
            "agent.alpha",
            100_001,
            0,
            100,
            "alpha-v2",
            "Alpha Dup",
            "2026-06-01T01:10:00Z",
        ))
        .expect_err("same agent_id in tenant must fail");
    assert_structured_kind(duplicate_agent_id, "conflict");

    let duplicate_code = service
        .create_agent(create_agent_cmd(
            "agent.beta",
            100_001,
            0,
            101,
            "alpha",
            "Beta",
            "2026-06-01T01:20:00Z",
        ))
        .expect_err("same code in tenant must fail");
    assert_structured_kind(duplicate_code, "conflict");
}

#[test]
fn create_agent_rejects_non_standard_agent_id() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let error = service
        .create_agent(create_agent_cmd(
            "pc.agent.invalid",
            100_001,
            0,
            100,
            "invalid",
            "Invalid",
            "2026-06-01T01:30:00Z",
        ))
        .expect_err("non-standard agent_id must fail");

    match error {
        KernelError::Validation { message } => {
            assert_eq!(message, "agentId must start with agent.");
        }
        _ => panic!("expected validation error"),
    }
}

#[test]
fn agent_resource_entry_points_validate_standard_agent_id_before_authorization() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_deny_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);
    let invalid_agent_id = "pc.agent.invalid";

    assert_agent_id_validation(
        service
            .update_agent(UpdateAgentCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                expected_version: Some(1),
                display_name: Some("Invalid".to_string()),
                description: None,
                manifest: None,
                visibility: None,
                tags: None,
                default_code_task_intent: None,
                implementation_provider_id: None,
                implementation_kind: None,
                implementation_type: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:31:00Z".to_string(),
            })
            .expect_err("invalid agent_id must be rejected before update authorization"),
    );
    assert_agent_id_validation(
        service
            .change_status(ChangeAgentStatusCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                expected_version: Some(1),
                target_status: AgentBusinessStatus::Active,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:32:00Z".to_string(),
            })
            .expect_err("invalid agent_id must be rejected before status authorization"),
    );
    assert_agent_id_validation(
        service
            .delete_agent(DeleteAgentCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                expected_version: Some(1),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:33:00Z".to_string(),
            })
            .expect_err("invalid agent_id must be rejected before delete authorization"),
    );
    assert_agent_id_validation(
        service
            .restore_agent(RestoreAgentCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                expected_version: Some(1),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:34:00Z".to_string(),
            })
            .expect_err("invalid agent_id must be rejected before restore authorization"),
    );
    assert_agent_id_validation(
        service
            .get_agent(GetAgentCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                requested_by: sample_subject(),
            })
            .expect_err("invalid agent_id must be rejected before retrieve authorization"),
    );
    assert_agent_id_validation(
        service
            .add_provider_binding(AgentProviderBindingCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                binding_id: "binding.invalid.default".to_string(),
                provider_id: "provider.agent.manifest".to_string(),
                implementation_kind: AgentImplementationKind::ManifestOnly,
                configuration_profile_id: "profile.agent.manifest.default".to_string(),
                capabilities: vec!["model.chat".to_string()],
                make_default: true,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:35:00Z".to_string(),
            })
            .expect_err("invalid agent_id must be rejected before binding authorization"),
    );
    assert_agent_id_validation(
        service
            .activate_provider_binding(ActivateAgentProviderBindingCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                binding_id: "binding.invalid.default".to_string(),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:36:00Z".to_string(),
            })
            .expect_err(
                "invalid agent_id must be rejected before binding activation authorization",
            ),
    );
    assert_agent_id_validation(
        service
            .list_provider_bindings(ProviderBindingListCommand {
                query: ProviderBindingListQuery::for_agent(100_001, invalid_agent_id),
                requested_by: sample_subject(),
            })
            .expect_err("invalid agent_id must be rejected before binding list authorization"),
    );
    assert_agent_id_validation(
        service
            .create_preview_response(AgentPreviewResponseCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                execution_id: "execution.invalid.preview".to_string(),
                content: "preview".to_string(),
                debug_mode: false,
                model: None,
                temperature: None,
                input_payload_json: "{}".to_string(),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:38:00Z".to_string(),
            })
            .expect_err("invalid agent_id must be rejected before preview authorization"),
    );
    assert_agent_id_validation(
        service
            .create_prompt_optimization(AgentPromptOptimizationCommand {
                tenant_id: 100_001,
                agent_id: invalid_agent_id.to_string(),
                execution_id: "execution.invalid.prompt".to_string(),
                prompt: "answer".to_string(),
                input_payload_json: "{}".to_string(),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T01:39:00Z".to_string(),
            })
            .expect_err("invalid agent_id must be rejected before prompt authorization"),
    );
    assert_agent_id_validation(
        service
            .list_agent_audit_events(ListAgentAuditEventsCommand {
                query: AuditEventListQuery::for_agent(100_001, invalid_agent_id),
                requested_by: sample_subject(),
            })
            .expect_err("invalid agent_id must be rejected before audit authorization"),
    );
}

#[test]
fn create_agent_records_implementation_type() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let record = service
        .create_agent(CreateAgentCommand {
            implementation_provider_id: Some("provider.agent.langgraph".to_string()),
            implementation_kind: Some(AgentImplementationKind::ProtocolAdapter),
            implementation_type: Some(AgentImplementationType::LangGraph),
            ..create_agent_cmd(
                "agent.implementation.langgraph",
                100_001,
                0,
                100,
                "implementation-langgraph",
                "Implementation LangGraph",
                "2026-06-01T01:10:00Z",
            )
        })
        .expect("create should preserve implementation type");

    assert_eq!(
        record.implementation_type,
        AgentImplementationType::LangGraph
    );
    assert_eq!(
        record.implementation_provider_id.as_deref(),
        Some("provider.agent.langgraph")
    );
    assert_eq!(
        record.implementation_kind,
        Some(AgentImplementationKind::ProtocolAdapter)
    );
}

#[test]
fn create_agent_defaults_implementation_type_to_sdkwork_native() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let record = service
        .create_agent(create_agent_cmd(
            "agent.implementation.default",
            100_001,
            0,
            100,
            "implementation-default",
            "Implementation Default",
            "2026-06-01T01:11:00Z",
        ))
        .expect("create should default implementation type");

    assert_eq!(
        record.implementation_type,
        AgentImplementationType::SdkworkNative
    );
}

#[test]
fn update_agent_changes_implementation_contract() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let created = service
        .create_agent(create_agent_cmd(
            "agent.implementation.update",
            100_001,
            0,
            100,
            "implementation-update",
            "Implementation Update",
            "2026-06-01T01:12:00Z",
        ))
        .expect("create should succeed");

    let updated = service
        .update_agent(UpdateAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.implementation.update".to_string(),
            expected_version: Some(created.version),
            display_name: None,
            description: None,
            manifest: None,
            visibility: None,
            tags: None,
            default_code_task_intent: None,
            implementation_provider_id: Some(Some("provider.agent.crewai".to_string())),
            implementation_kind: Some(Some(AgentImplementationKind::ProcessAdapter)),
            implementation_type: Some(AgentImplementationType::CrewAi),
            requested_by: sample_subject(),
            requested_at: "2026-06-01T01:13:00Z".to_string(),
        })
        .expect("update should change implementation contract");

    assert_eq!(updated.implementation_type, AgentImplementationType::CrewAi);
    assert_eq!(
        updated.implementation_provider_id.as_deref(),
        Some("provider.agent.crewai")
    );
    assert_eq!(
        updated.implementation_kind,
        Some(AgentImplementationKind::ProcessAdapter)
    );
}

#[test]
fn stale_expected_version_is_rejected_for_update() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let created = service
        .create_agent(create_agent_cmd(
            "agent.versioned",
            100_001,
            0,
            100,
            "versioned",
            "Versioned",
            "2026-06-01T01:00:00Z",
        ))
        .expect("create should succeed");

    let updated = service
        .update_agent(UpdateAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.versioned".to_string(),
            expected_version: Some(created.version),
            display_name: Some("Versioned v2".to_string()),
            description: None,
            manifest: None,
            visibility: None,
            tags: None,
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T01:01:00Z".to_string(),
        })
        .expect("update with matching expected version should succeed");
    assert_eq!(updated.version, created.version + 1);

    let stale_update = service
        .update_agent(UpdateAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.versioned".to_string(),
            expected_version: Some(created.version),
            display_name: Some("Versioned v3".to_string()),
            description: None,
            manifest: None,
            visibility: None,
            tags: None,
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T01:02:00Z".to_string(),
        })
        .expect_err("stale expected version should fail");

    assert_structured_kind(stale_update, "conflict");
}

#[test]
fn deleted_agent_cannot_be_updated() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let created = service
        .create_agent(create_agent_cmd(
            "agent.delta",
            100_001,
            0,
            100,
            "delta",
            "Delta",
            "2026-06-01T02:00:00Z",
        ))
        .expect("create should succeed");

    let deleted = service
        .delete_agent(DeleteAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.delta".to_string(),
            expected_version: Some(created.version),
            requested_by: sample_subject(),
            requested_at: "2026-06-01T02:05:00Z".to_string(),
        })
        .expect("delete should succeed");

    let result = service.update_agent(UpdateAgentCommand {
        tenant_id: 100_001,
        agent_id: "agent.delta".to_string(),
        expected_version: Some(deleted.version),
        display_name: Some("Delta v2".to_string()),
        description: None,
        manifest: None,
        visibility: None,
        tags: None,
        default_code_task_intent: None,
        implementation_provider_id: None,
        implementation_kind: None,
        implementation_type: None,
        requested_by: sample_subject(),
        requested_at: "2026-06-01T02:06:00Z".to_string(),
    });

    let error = result.expect_err("deleted agent should not allow updates");
    match error {
        KernelError::Validation { message } => {
            assert!(message.contains("deleted agent cannot be updated"));
        }
        _ => panic!("expected validation error"),
    }
}

#[test]
fn restore_requires_deleted_status() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    service
        .create_agent(create_agent_cmd(
            "agent.epsilon",
            100_001,
            0,
            100,
            "epsilon",
            "Epsilon",
            "2026-06-01T03:00:00Z",
        ))
        .expect("create should succeed");

    let result = service.restore_agent(RestoreAgentCommand {
        tenant_id: 100_001,
        agent_id: "agent.epsilon".to_string(),
        expected_version: Some(1),
        requested_by: sample_subject(),
        requested_at: "2026-06-01T03:01:00Z".to_string(),
    });

    let error = result.expect_err("restore without delete should fail");
    match error {
        KernelError::Validation { message } => {
            assert!(message.contains("agent is not deleted"));
        }
        _ => panic!("expected validation error"),
    }
}

#[test]
fn list_filters_by_owner_organization_and_deleted_flag() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    service
        .create_agent(create_agent_cmd(
            "agent.owner.a",
            100_001,
            0,
            100,
            "owner-a",
            "Owner A",
            "2026-06-01T04:00:00Z",
        ))
        .expect("create owner a should succeed");
    let owner_b = service
        .create_agent(create_agent_cmd(
            "agent.owner.b",
            100_001,
            11,
            101,
            "owner-b",
            "Owner B",
            "2026-06-01T04:01:00Z",
        ))
        .expect("create owner b should succeed");

    service
        .delete_agent(DeleteAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.owner.b".to_string(),
            expected_version: Some(owner_b.version),
            requested_by: sample_subject(),
            requested_at: "2026-06-01T04:02:00Z".to_string(),
        })
        .expect("delete owner b should succeed");

    let by_org = service
        .list_agents(ListAgentsCommand {
            query: AgentListQuery::for_tenant(100_001).for_organization(0),
            requested_by: sample_subject(),
        })
        .expect("list by org should succeed");
    assert_eq!(by_org.items.len(), 1);
    assert_eq!(by_org.items[0].agent_id, "agent.owner.a");

    let by_owner_without_deleted = service
        .list_agents(ListAgentsCommand {
            query: AgentListQuery::for_tenant(100_001).for_owner(101),
            requested_by: sample_subject(),
        })
        .expect("list by owner should succeed");
    assert!(by_owner_without_deleted.items.is_empty());

    let by_owner_with_deleted = service
        .list_agents(ListAgentsCommand {
            query: AgentListQuery::for_tenant(100_001)
                .for_owner(101)
                .with_deleted(),
            requested_by: sample_subject(),
        })
        .expect("list by owner with deleted should succeed");
    assert_eq!(by_owner_with_deleted.items.len(), 1);
    assert_eq!(
        by_owner_with_deleted.items[0].status,
        AgentBusinessStatus::Deleted
    );
}

#[test]
fn list_filters_by_search_query_across_code_name_and_description() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    service
        .create_agent(create_agent_cmd(
            "agent.search.alpha",
            100_001,
            0,
            100,
            "alpha-code",
            "Alpha Worker",
            "2026-06-01T06:00:00Z",
        ))
        .expect("create alpha should succeed");

    service
        .create_agent(CreateAgentCommand {
            description: Some("handles retrieval workloads".to_string()),
            ..create_agent_cmd(
                "agent.search.beta",
                100_001,
                0,
                101,
                "beta-code",
                "Beta Agent",
                "2026-06-01T06:01:00Z",
            )
        })
        .expect("create beta should succeed");

    let by_code = service
        .list_agents(ListAgentsCommand {
            query: AgentListQuery::for_tenant(100_001).with_search("alpha-code"),
            requested_by: sample_subject(),
        })
        .expect("list by code search should succeed");
    assert_eq!(by_code.items.len(), 1);
    assert_eq!(by_code.items[0].agent_id, "agent.search.alpha");

    let by_name = service
        .list_agents(ListAgentsCommand {
            query: AgentListQuery::for_tenant(100_001).with_search("beta"),
            requested_by: sample_subject(),
        })
        .expect("list by display name search should succeed");
    assert_eq!(by_name.items.len(), 1);
    assert_eq!(by_name.items[0].agent_id, "agent.search.beta");

    let by_description = service
        .list_agents(ListAgentsCommand {
            query: AgentListQuery::for_tenant(100_001).with_search("retrieval"),
            requested_by: sample_subject(),
        })
        .expect("list by description search should succeed");
    assert_eq!(by_description.items.len(), 1);
    assert_eq!(by_description.items[0].agent_id, "agent.search.beta");
}

#[test]
fn audit_events_are_recorded_for_state_mutations() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, arc_events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    service
        .create_agent(create_agent_cmd(
            "agent.audit",
            100_001,
            0,
            100,
            "audit",
            "Audit",
            "2026-06-01T05:00:00Z",
        ))
        .expect("create should succeed");
    let created = service
        .get_agent(GetAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.audit".to_string(),
            requested_by: sample_subject(),
        })
        .expect("retrieve should succeed");
    let updated = service
        .update_agent(UpdateAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.audit".to_string(),
            expected_version: Some(created.version),
            display_name: Some("Audit v2".to_string()),
            description: None,
            manifest: None,
            visibility: None,
            tags: None,
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:01:00Z".to_string(),
        })
        .expect("update should succeed");
    let activated = service
        .change_status(ChangeAgentStatusCommand {
            tenant_id: 100_001,
            agent_id: "agent.audit".to_string(),
            expected_version: Some(updated.version),
            target_status: AgentBusinessStatus::Active,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:02:00Z".to_string(),
        })
        .expect("status update should succeed");
    let deleted = service
        .delete_agent(DeleteAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.audit".to_string(),
            expected_version: Some(activated.version),
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:03:00Z".to_string(),
        })
        .expect("delete should succeed");
    service
        .restore_agent(RestoreAgentCommand {
            tenant_id: 100_001,
            agent_id: "agent.audit".to_string(),
            expected_version: Some(deleted.version),
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:04:00Z".to_string(),
        })
        .expect("restore should succeed");

    let events_list = arc_events.lock().expect("events mutex poisoned");
    assert_eq!(events_list.len(), 5);
    let event_types: Vec<&str> = events_list
        .iter()
        .map(|event| event.event_type.as_str())
        .collect();
    assert_eq!(
        event_types,
        vec![
            "agent.business.created",
            "agent.business.updated",
            "agent.business.status_changed",
            "agent.business.deleted",
            "agent.business.restored",
        ]
    );
}

#[test]
fn policy_deny_blocks_management_operations() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_deny_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let result = service.create_agent(create_agent_cmd(
        "agent.beta",
        100_001,
        0,
        100,
        "beta",
        "Beta",
        "2026-06-01T01:00:00Z",
    ));

    let error = result.expect_err("denied policy should block create");
    match error {
        KernelError::Structured { info } => {
            assert_eq!(info.kind.as_str(), "permission_required");
        }
        KernelError::Internal { .. }
        | KernelError::Validation { .. }
        | KernelError::CapabilityMissing { .. }
        | KernelError::ProviderUnavailable { .. }
        | KernelError::PolicyDenied { .. } => {
            panic!("expected permission_required structured error")
        }
    }
}

#[test]
fn invalid_status_transition_is_rejected() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let created = service
        .create_agent(create_agent_cmd(
            "agent.gamma",
            100_001,
            0,
            100,
            "gamma",
            "Gamma",
            "2026-06-01T02:00:00Z",
        ))
        .expect("create should succeed");

    let result = service.change_status(ChangeAgentStatusCommand {
        tenant_id: 100_001,
        agent_id: "agent.gamma".to_string(),
        expected_version: Some(created.version),
        target_status: AgentBusinessStatus::Disabled,
        requested_by: sample_subject(),
        requested_at: "2026-06-01T02:10:00Z".to_string(),
    });
    let error = result.expect_err("draft -> disabled should fail");
    match error {
        KernelError::Validation { message } => {
            assert!(message.contains("invalid agent status transition"));
        }
        _ => panic!("expected validation error"),
    }
}

#[test]
fn policy_category_constant_is_sdkwork_intelligence_agents_service_manage() {
    assert_eq!(
        DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
        "agent.business.manage"
    );
}

#[test]
fn list_agent_audit_events_returns_events_for_agent() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let created = service
        .create_agent(create_agent_cmd(
            "agent.audit.list",
            100_001,
            0,
            100,
            "audit-list",
            "Audit List",
            "2026-06-01T04:00:00Z",
        ))
        .expect("create should succeed");

    service
        .change_status(ChangeAgentStatusCommand {
            tenant_id: 100_001,
            agent_id: "agent.audit.list".to_string(),
            expected_version: Some(created.version),
            target_status: AgentBusinessStatus::Active,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T04:05:00Z".to_string(),
        })
        .expect("status transition should succeed");

    let events = service
        .list_agent_audit_events(ListAgentAuditEventsCommand {
            query: AuditEventListQuery::for_agent(100_001, "agent.audit.list")
                .with_pagination(PaginationParams::default().with_page_size(MAX_PAGE_SIZE)),
            requested_by: sample_subject(),
        })
        .expect("list audit events should succeed");
    assert_eq!(events.items.len(), 2);
    assert_eq!(events.items[0].event_type, "agent.business.status_changed");
    assert_eq!(events.items[1].event_type, "agent.business.created");
}

#[test]
fn execute_turn_persists_user_input_and_assistant_output() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, audit_events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let created = service
        .create_agent(create_agent_cmd(
            "agent.chat.turn",
            100_001,
            0,
            100,
            "chat-turn",
            "Agent Turn",
            "2026-06-01T05:00:00Z",
        ))
        .expect("create should succeed");
    service
        .change_status(ChangeAgentStatusCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            expected_version: Some(created.version),
            target_status: AgentBusinessStatus::Active,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:00:30Z".to_string(),
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
            title: Some("Support session".to_string()),
            idempotency_key: None,
            payload_hash: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:01:00Z".to_string(),
        })
        .expect("create session should succeed");

    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            binding_id: "binding.turn.contract".to_string(),
            provider_id: "provider.turn.contract".to_string(),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: "profile.turn.contract".to_string(),
            capabilities: Vec::new(),
            make_default: true,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:01:10Z".to_string(),
        })
        .expect("provider binding should be created");
    let runtime_binding = service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: created.agent_id.clone(),
            session_id: session.session_id.clone(),
            runtime_binding_id: Some("runtime_binding.turn.contract".to_string()),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: provider_binding.binding_id,
            model_id: "model.turn.contract".to_string(),
            provider_id: provider_binding.provider_id,
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            provider_directory: None,
            owner_scope: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:01:20Z".to_string(),
        })
        .expect("session runtime binding should be created");
    assert!(
        runtime_binding.provider_visible,
        "runtime bindings without a Provider directory must remain visible"
    );

    let turn_command = CreateTurnCommand {
        tenant_id: 100_001,
        organization_id: 0,
        agent_id: created.agent_id.clone(),
        session_id: session.session_id.clone(),
        turn_id: Some("turn.contract.one".to_string()),
        content: "Hello, can you help?".to_string(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id),
        requested_model_id: Some("model.turn.contract".to_string()),
        access_mode_id: None,
        idempotency_key: "turn-test-idempotency-1".to_string(),
        payload_hash: "sha256:turn-test-payload-1".to_string(),
        client_request_id: Some("request.turn.1".to_string()),
        drive_refs: vec![AgentItemDriveRefInput {
            resource_role: AgentItemResourceRole::Image,
            drive_space_id: "space-turn".to_string(),
            drive_node_id: "node-turn-1".to_string(),
        }],
        owner_scope: None,
        requested_by: sample_subject(),
        requested_at: "2026-06-01T05:01:30Z".to_string(),
        prefer_stream: false,
        auth_token: None,
        wire_protocol: None,
    };
    let result = service
        .execute_turn(turn_command.clone())
        .expect("turn execution should succeed");

    assert_eq!(result.user_input_item.kind, AgentSessionItemKind::UserInput);
    assert_eq!(
        result.user_input_item.content.as_deref(),
        Some("Hello, can you help?")
    );
    assert_eq!(
        result.assistant_output_item.kind,
        AgentSessionItemKind::AssistantOutput
    );
    assert!(result
        .assistant_output_item
        .content
        .as_deref()
        .is_some_and(|content| !content.is_empty()));
    assert_eq!(result.session.item_count, 2);
    {
        let events = audit_events.lock().expect("recording audit mutex poisoned");
        assert!(events.iter().any(|event| {
            extract_event_context(event.payload.as_str(), "aggregate_type").as_deref()
                == Some("turn")
                && extract_event_context(event.payload.as_str(), "aggregate_id").as_deref()
                    == Some(result.turn.turn_id.as_str())
        }));
        for item in [&result.user_input_item, &result.assistant_output_item] {
            assert!(events.iter().any(|event| {
                extract_event_context(event.payload.as_str(), "aggregate_type").as_deref()
                    == Some("session_item")
                    && extract_event_context(event.payload.as_str(), "aggregate_id").as_deref()
                        == Some(item.item_id.as_str())
            }));
        }
    }
    let completed_turn = service
        .get_turn(GetTurnCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: result.turn.agent_id.clone(),
            session_id: result.user_input_item.session_id.clone(),
            turn_id: result.user_input_item.turn_id.clone().unwrap(),
            owner_scope: None,
            requested_by: sample_subject(),
        })
        .unwrap();
    assert_eq!(completed_turn.status, AgentTurnStatus::Completed);
    assert_eq!(completed_turn.version, 2);
    assert!(completed_turn.started_at.is_some());
    let turn_by_idempotency = service
        .get_turn_by_idempotency(GetTurnByIdempotencyCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: result.turn.agent_id.clone(),
            session_id: result.user_input_item.session_id.clone(),
            owner_user_id: 100,
            idempotency_key: "turn-test-idempotency-1".to_string(),
            requested_by: sample_subject(),
        })
        .expect("turn idempotency lookup should succeed")
        .expect("completed turn should be found");
    assert_eq!(turn_by_idempotency.turn_id, completed_turn.turn_id);
    assert_eq!(turn_by_idempotency.status, AgentTurnStatus::Completed);
    let hidden_from_foreign_owner = service.get_turn_by_idempotency(GetTurnByIdempotencyCommand {
        tenant_id: 100_001,
        organization_id: 0,
        path_agent_id: result.turn.agent_id.clone(),
        session_id: result.user_input_item.session_id.clone(),
        owner_user_id: 999,
        idempotency_key: "turn-test-idempotency-1".to_string(),
        requested_by: sample_subject(),
    });
    assert!(hidden_from_foreign_owner.is_err());
    assert_eq!(result.user_item_drive_refs.len(), 1);
    assert_eq!(result.user_item_drive_refs[0].drive_space_id, "space-turn");
    assert_eq!(result.user_item_drive_refs[0].drive_node_id, "node-turn-1");

    let replay = service
        .execute_turn(turn_command.clone())
        .expect("same idempotency key and payload should replay");
    assert_eq!(
        replay.user_input_item.item_id,
        result.user_input_item.item_id
    );
    assert_eq!(
        replay.assistant_output_item.item_id,
        result.assistant_output_item.item_id,
    );
    assert_eq!(replay.session.item_count, 2);
    assert_eq!(replay.user_item_drive_refs, result.user_item_drive_refs);

    let mut conflicting = turn_command;
    conflicting.content = "Different payload".to_string();
    conflicting.payload_hash = "sha256:turn-test-payload-2".to_string();
    assert!(matches!(
        service.execute_turn(conflicting),
        Err(KernelError::Structured { info }) if info.kind == KernelErrorKind::Conflict
    ));
}

#[test]
fn get_session_rejects_foreign_owner_scope() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = test_policy_provider();
    let service = AgentsService::new(repository, audit_sink, policy_provider);

    let created = service
        .create_agent(create_agent_cmd(
            "agent.owner.scope",
            100_001,
            0,
            100,
            "owner-scope",
            "Owner Scope",
            "2026-06-01T05:00:00Z",
        ))
        .expect("create should succeed");

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
            title: Some("Private session".to_string()),
            idempotency_key: None,
            payload_hash: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:01:00Z".to_string(),
        })
        .expect("create session should succeed");

    let error = service
        .get_session(GetSessionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: created.agent_id.clone(),
            session_id: session.session_id.clone(),
            owner_scope: Some(999),
            requested_by: sample_subject(),
        })
        .expect_err("foreign owner must not read session");
    assert!(error.to_string().contains("session not found"));
}

#[test]
fn interaction_approval_lifecycle_persists_and_resolves() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let service = AgentsService::new(repository, audit_sink, test_policy_provider());

    let agent = service
        .create_agent(create_agent_cmd(
            "agent.interaction",
            100_001,
            0,
            100,
            "interaction",
            "Interaction Agent",
            "2026-06-01T05:00:00Z",
        ))
        .expect("create agent should succeed");

    let session = service
        .create_session(CreateSessionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            agent_id: agent.agent_id.clone(),
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
            title: Some("Interaction session".to_string()),
            idempotency_key: None,
            payload_hash: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:01:00Z".to_string(),
        })
        .expect("create session should succeed");

    let interaction = service
        .create_interaction(CreateInteractionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: agent.agent_id.clone(),
            session_id: session.session_id.clone(),
            interaction_id: String::new(),
            turn_id: None,
            runtime_binding_id: None,
            provider_interaction_id: None,
            kind: AgentInteractionKind::Approval,
            prompt: "Approve write?".to_string(),
            options_json: "[]".to_string(),
            request_json: None,
            retention_until: None,
            owner_scope: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:02:00Z".to_string(),
        })
        .expect("create interaction should succeed");
    assert_eq!(interaction.status.as_str(), "pending");

    let listed = service
        .list_interactions(ListInteractionsCommand {
            query: InteractionListQuery::for_session(100_001, 0, session.session_id.clone()),
            path_agent_id: agent.agent_id.clone(),
            owner_scope: None,
            requested_by: sample_subject(),
        })
        .expect("list interactions should succeed");
    assert_eq!(listed.items.len(), 1);

    let claim = service
        .claim_interaction(ClaimInteractionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: agent.agent_id.clone(),
            session_id: session.session_id.clone(),
            interaction_id: interaction.interaction_id.clone(),
            claim_owner: "worker.contract".to_string(),
            lease_seconds: 60,
            expected_version: interaction.version,
            owner_scope: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:02:30Z".to_string(),
        })
        .expect("claim interaction should succeed");

    let resolved = service
        .approve_interaction(ApproveInteractionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: agent.agent_id.clone(),
            session_id: session.session_id.clone(),
            interaction_id: interaction.interaction_id.clone(),
            approved: true,
            reason: Some("ok".to_string()),
            claim_token: claim.claim_token,
            fencing_token: claim.fencing_token,
            expected_version: claim.interaction.version,
            owner_scope: None,
            requested_by: sample_subject(),
            requested_at: "2026-06-01T05:03:00Z".to_string(),
        })
        .expect("approve interaction should succeed");
    assert_eq!(resolved.status.as_str(), "resolved");

    let retrieved = service
        .get_interaction(GetInteractionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: agent.agent_id,
            session_id: session.session_id,
            interaction_id: interaction.interaction_id,
            owner_scope: None,
            requested_by: sample_subject(),
        })
        .expect("get interaction should succeed");
    assert_eq!(retrieved.status.as_str(), "resolved");
}

#[test]
fn typed_interaction_envelopes_validate_and_resolve_without_loss() {
    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let service = AgentsService::new(repository, audit_sink, test_policy_provider());
    let agent = service
        .create_agent(create_agent_cmd(
            "agent.typed.interaction",
            100_001,
            0,
            100,
            "typed-interaction",
            "Typed Interaction Agent",
            "2026-08-01T00:00:00Z",
        ))
        .expect("create typed interaction agent");
    let session = service
        .create_session(CreateSessionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            agent_id: agent.agent_id.clone(),
            owner_user_id: 100,
            session_id: "session.typed.interaction".to_string(),
            project_id: None,
            session_kind: AgentSessionKind::Coding,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Typed interactions".to_string()),
            idempotency_key: None,
            payload_hash: None,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:00:01Z".to_string(),
        })
        .expect("create typed interaction session");
    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 100_001,
            agent_id: agent.agent_id.clone(),
            binding_id: "binding.typed.interaction".to_string(),
            provider_id: "provider.codex".to_string(),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: "profile.typed.interaction".to_string(),
            capabilities: Vec::new(),
            make_default: true,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:00:02Z".to_string(),
        })
        .expect("create typed interaction provider binding");
    let runtime_binding = service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: agent.agent_id.clone(),
            session_id: session.session_id.clone(),
            runtime_binding_id: Some("runtime_binding.typed.interaction".to_string()),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "typescript_node".to_string(),
            provider_binding_id: provider_binding.binding_id,
            model_id: "gpt-5.6-codex".to_string(),
            provider_id: provider_binding.provider_id,
            provider_session_id: Some("provider-session-typed".to_string()),
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            provider_directory: None,
            owner_scope: None,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:00:03Z".to_string(),
        })
        .expect("create typed interaction runtime binding");

    let cases = [
        (
            AgentInteractionKind::UserQuestion,
            "question-set",
            serde_json::json!({
                "schemaVersion": 1,
                "category": "user_input",
                "kind": "question_set",
                "allowedActions": ["submit", "cancel"],
                "data": {
                    "autoResolutionMs": 60000,
                    "questions": [
                        {"id": "workspace", "header": "Workspace", "prompt": "Which workspace?", "allowOther": true, "secret": false, "options": null},
                        {"id": "mode", "header": "Mode", "prompt": "Which mode?", "allowOther": false, "secret": false, "options": [{"label": "Local", "description": "Use this device"}]}
                    ]
                }
            }),
            serde_json::json!({
                "action": "submit",
                "answers": {"workspace": ["E:\\workspace"], "mode": ["Local"]}
            }),
        ),
        (
            AgentInteractionKind::Approval,
            "session-command",
            serde_json::json!({
                "schemaVersion": 1,
                "category": "approval",
                "kind": "command_execution",
                "allowedActions": ["accept", "accept_for_session", "accept_with_exec_policy_amendment", "apply_network_policy_amendment", "decline", "cancel"],
                "data": {"command": "pnpm test", "cwd": "E:\\workspace"}
            }),
            serde_json::json!({"action": "accept_for_session"}),
        ),
        (
            AgentInteractionKind::Approval,
            "permission-profile",
            serde_json::json!({
                "schemaVersion": 1,
                "category": "approval",
                "kind": "permission_profile",
                "allowedActions": ["grant", "decline", "cancel"],
                "data": {"cwd": "E:\\workspace", "requestedPermissions": {"network": {"enabled": true}}}
            }),
            serde_json::json!({
                "action": "grant",
                "permissions": {"network": {"enabled": true}},
                "scope": "session",
                "strictAutoReview": true
            }),
        ),
        (
            AgentInteractionKind::Elicitation,
            "mcp-elicitation",
            serde_json::json!({
                "schemaVersion": 1,
                "category": "elicitation",
                "kind": "mcp_elicitation",
                "allowedActions": ["accept", "decline", "cancel"],
                "data": {"mode": "form", "serverName": "deployments", "message": "Provide deployment details", "requestedSchema": {"type": "object"}}
            }),
            serde_json::json!({
                "action": "accept",
                "content": {"environment": "test"},
                "metadata": {"source": "contract-test"}
            }),
        ),
    ];

    for (index, (kind, suffix, request, resolution)) in cases.into_iter().enumerate() {
        let interaction = service
            .create_interaction(CreateInteractionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id.clone(),
                session_id: session.session_id.clone(),
                interaction_id: format!("interaction.typed.{suffix}"),
                turn_id: None,
                runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
                provider_interaction_id: Some(format!("provider-interaction-{suffix}")),
                kind,
                prompt: format!("Resolve {suffix}"),
                options_json: "[]".to_string(),
                request_json: Some(request.to_string()),
                retention_until: None,
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: format!("2026-08-01T00:01:{index:02}Z"),
            })
            .expect("create typed interaction");
        assert_eq!(
            interaction.request_json.as_deref(),
            Some(request.to_string().as_str())
        );

        if suffix == "session-command" {
            let legacy_error = service
                .approve_interaction(ApproveInteractionCommand {
                    tenant_id: 100_001,
                    organization_id: 0,
                    path_agent_id: agent.agent_id.clone(),
                    session_id: session.session_id.clone(),
                    interaction_id: interaction.interaction_id.clone(),
                    approved: true,
                    reason: None,
                    claim_token: "not-used-for-typed-interactions".to_string(),
                    fencing_token: 0,
                    expected_version: interaction.version,
                    owner_scope: None,
                    requested_by: sample_subject(),
                    requested_at: "2026-08-01T00:02:00Z".to_string(),
                })
                .expect_err("typed interaction must reject legacy approve");
            assert!(legacy_error.to_string().contains("typed interaction"));
        }

        let claim = service
            .claim_interaction(ClaimInteractionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id.clone(),
                session_id: session.session_id.clone(),
                interaction_id: interaction.interaction_id.clone(),
                claim_owner: "worker.typed.contract".to_string(),
                lease_seconds: 60,
                expected_version: interaction.version,
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-08-01T00:02:01Z".to_string(),
            })
            .expect("claim typed interaction");

        if suffix == "question-set" {
            for (invalid_resolution, expected_error) in [
                (serde_json::json!({"action": "grant"}), "not allowed"),
                (
                    serde_json::json!({"action": "submit", "answers": {"unknown": ["value"]}}),
                    "unknown question id",
                ),
            ] {
                let error = service
                    .resolve_interaction(ResolveInteractionCommand {
                        tenant_id: 100_001,
                        organization_id: 0,
                        path_agent_id: agent.agent_id.clone(),
                        session_id: session.session_id.clone(),
                        interaction_id: interaction.interaction_id.clone(),
                        resolution_json: invalid_resolution.to_string(),
                        claim_token: claim.claim_token.clone(),
                        fencing_token: claim.fencing_token,
                        expected_version: claim.interaction.version,
                        owner_scope: None,
                        requested_by: sample_subject(),
                        requested_at: "2026-08-01T00:02:02Z".to_string(),
                    })
                    .expect_err("invalid typed resolution must fail");
                assert!(error.to_string().contains(expected_error));
            }
        }

        let resolved = service
            .resolve_interaction(ResolveInteractionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id.clone(),
                session_id: session.session_id.clone(),
                interaction_id: interaction.interaction_id,
                resolution_json: resolution.to_string(),
                claim_token: claim.claim_token,
                fencing_token: claim.fencing_token,
                expected_version: claim.interaction.version,
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-08-01T00:02:03Z".to_string(),
            })
            .expect("resolve typed interaction");
        assert_eq!(resolved.status.as_str(), "resolved");
        assert_eq!(
            resolved.resolution_json.as_deref(),
            Some(resolution.to_string().as_str())
        );
    }
}
