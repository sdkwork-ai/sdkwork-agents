use sdkwork_agent_kernel::{AgentManifest, PolicySubject};
use sdkwork_intelligence_agents_service::{
    AgentItemFeedbackRating, AgentRepository, AgentResourceType, AgentResourceUserStateRecord,
    AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionKind, AgentSessionRecord,
    AgentSessionStatus, AgentSessionTitleSource, AgentVisibility, AgentsService,
    CreateAgentCommand, CreateSessionCommand, CreateSessionItemCommand, GetSessionUserStateCommand,
    IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository, ItemFeedbackListQuery,
    ListItemFeedbackCommand, ListSessionUserStatesCommand, PaginationParams,
    ResourceUserStateListQuery, UpdateItemFeedbackCommand, UpdateSessionUserStateCommand,
    SQL_COUNT_AGENT_ITEM_FEEDBACK, SQL_COUNT_AGENT_RESOURCE_USER_STATES,
    SQL_LIST_AGENT_ITEM_FEEDBACK, SQL_LIST_AGENT_RESOURCE_USER_STATES,
    SQL_SELECT_AGENT_ITEM_FEEDBACK, SQL_SELECT_AGENT_RESOURCE_USER_STATE,
    SQL_UPSERT_AGENT_ITEM_FEEDBACK, SQL_UPSERT_AGENT_RESOURCE_USER_STATE,
};

fn subject() -> PolicySubject {
    PolicySubject::new("user.100", "100001").with_role("ai.agents.manage")
}

fn manifest(agent_id: &str) -> AgentManifest {
    AgentManifest {
        schema_version: "1.0.0".to_string(),
        manifest_type: "agent".to_string(),
        agent_id: agent_id.to_string(),
        name: "user-state-agent".to_string(),
        display_name: "User State Agent".to_string(),
        description: "contract fixture".to_string(),
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

fn create_agent(service: &TestService, agent_id: &str) {
    service
        .create_agent(CreateAgentCommand {
            agent_id: agent_id.to_string(),
            tenant_id: 100_001,
            organization_id: 7,
            owner_user_id: 100,
            code: agent_id.replace('.', "-"),
            display_name: "User State Agent".to_string(),
            description: None,
            manifest: manifest(agent_id),
            visibility: AgentVisibility::Private,
            tags: Vec::new(),
            allowed_role_codes: Vec::new(),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:00Z".to_string(),
        })
        .expect("agent fixture should be created");
}

fn create_session(service: &TestService, agent_id: &str, session_id: &str) {
    service
        .create_session(CreateSessionCommand {
            tenant_id: 100_001,
            organization_id: 7,
            agent_id: agent_id.to_string(),
            owner_user_id: 100,
            session_id: session_id.to_string(),
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Contract session".to_string()),
            idempotency_key: None,
            payload_hash: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:01:00Z".to_string(),
        })
        .expect("session fixture should be created");
}

type TestService =
    AgentsService<InMemoryAgentRepository, InMemoryAgentAuditSink, IamGatedPolicyProvider>;

fn service() -> TestService {
    AgentsService::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.user-state.contract"),
    )
}

#[test]
fn session_user_state_service_enforces_scope_versions_and_read_sequence() {
    let service = service();
    create_agent(&service, "agent.alpha");
    create_agent(&service, "agent.beta");
    create_session(&service, "agent.alpha", "session.alpha");

    let created = service
        .update_session_user_state(UpdateSessionUserStateCommand {
            tenant_id: 100_001,
            organization_id: 7,
            user_id: 100,
            path_agent_id: "agent.alpha".to_string(),
            session_id: "session.alpha".to_string(),
            pinned: Some(true),
            hidden: None,
            mark_opened: true,
            last_read_item_sequence: Some(0),
            custom_title: Some(Some("Pinned contract".to_string())),
            expected_version: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:00Z".to_string(),
        })
        .expect("first user state mutation should create version zero");
    assert_eq!(created.version, 0);
    assert!(created.pinned_at.is_some());

    let pinned = service
        .list_session_user_states(ListSessionUserStatesCommand {
            query: ResourceUserStateListQuery::for_user_sessions(100_001, 7, 100).pinned_only(),
            path_agent_id: "agent.alpha".to_string(),
            requested_by: subject(),
        })
        .expect("pinned list should succeed");
    assert_eq!(pinned.items.len(), 1);

    let wrong_agent = service.get_session_user_state(GetSessionUserStateCommand {
        tenant_id: 100_001,
        organization_id: 7,
        user_id: 100,
        path_agent_id: "agent.beta".to_string(),
        session_id: "session.alpha".to_string(),
        requested_by: subject(),
    });
    assert!(
        wrong_agent.is_err(),
        "nested agent mismatch must be rejected"
    );

    let foreign_owner = service.get_session_user_state(GetSessionUserStateCommand {
        tenant_id: 100_001,
        organization_id: 7,
        user_id: 101,
        path_agent_id: "agent.alpha".to_string(),
        session_id: "session.alpha".to_string(),
        requested_by: subject(),
    });
    assert!(foreign_owner.is_err(), "foreign owner must be rejected");

    let beyond_tail = service.update_session_user_state(UpdateSessionUserStateCommand {
        tenant_id: 100_001,
        organization_id: 7,
        user_id: 100,
        path_agent_id: "agent.alpha".to_string(),
        session_id: "session.alpha".to_string(),
        pinned: None,
        hidden: None,
        mark_opened: false,
        last_read_item_sequence: Some(1),
        custom_title: None,
        expected_version: Some(0),
        requested_by: subject(),
        requested_at: "2026-07-19T00:03:00Z".to_string(),
    });
    assert!(
        beyond_tail.is_err(),
        "read cursor cannot exceed the session tail"
    );

    let updated = service
        .update_session_user_state(UpdateSessionUserStateCommand {
            tenant_id: 100_001,
            organization_id: 7,
            user_id: 100,
            path_agent_id: "agent.alpha".to_string(),
            session_id: "session.alpha".to_string(),
            pinned: Some(false),
            hidden: None,
            mark_opened: false,
            last_read_item_sequence: None,
            custom_title: None,
            expected_version: Some(0),
            requested_by: subject(),
            requested_at: "2026-07-19T00:04:00Z".to_string(),
        })
        .expect("matching expected version should update");
    assert_eq!(updated.version, 1);
    assert!(updated.pinned_at.is_none());

    let stale = service.update_session_user_state(UpdateSessionUserStateCommand {
        expected_version: Some(0),
        pinned: Some(true),
        requested_at: "2026-07-19T00:05:00Z".to_string(),
        tenant_id: 100_001,
        organization_id: 7,
        user_id: 100,
        path_agent_id: "agent.alpha".to_string(),
        session_id: "session.alpha".to_string(),
        hidden: None,
        mark_opened: false,
        last_read_item_sequence: None,
        custom_title: None,
        requested_by: subject(),
    });
    assert!(stale.is_err(), "stale expected version must be rejected");
}

#[test]
fn item_feedback_service_persists_changes_and_soft_delete() {
    let service = service();
    create_agent(&service, "agent.feedback");
    create_session(&service, "agent.feedback", "session.feedback");
    service
        .create_session_item(CreateSessionItemCommand {
            tenant_id: 100_001,
            organization_id: 7,
            session_id: "session.feedback".to_string(),
            item_id: "item.assistant".to_string(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: "Answer".to_string(),
            content_type: "text/plain".to_string(),
            input_tokens: 0,
            output_tokens: 1,
            model_id: None,
            provider_id: None,
            provider_payload_json: None,
            parent_item_id: None,
            requested_by: subject(),
            requested_at: "2026-07-19T01:00:00Z".to_string(),
        })
        .unwrap();

    let created = service
        .update_item_feedback(UpdateItemFeedbackCommand {
            tenant_id: 100_001,
            organization_id: 7,
            user_id: 100,
            path_agent_id: "agent.feedback".to_string(),
            session_id: "session.feedback".to_string(),
            item_id: "item.assistant".to_string(),
            rating: Some(AgentItemFeedbackRating::Up),
            reason_code: Some("helpful".to_string()),
            comment: None,
            expected_version: None,
            requested_by: subject(),
            requested_at: "2026-07-19T01:01:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(created.version, 0);

    let listed = service
        .list_item_feedback(ListItemFeedbackCommand {
            query: ItemFeedbackListQuery::for_user_session(100_001, 7, 100, "session.feedback"),
            path_agent_id: "agent.feedback".to_string(),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].rating, AgentItemFeedbackRating::Up);

    let removed = service
        .update_item_feedback(UpdateItemFeedbackCommand {
            tenant_id: 100_001,
            organization_id: 7,
            user_id: 100,
            path_agent_id: "agent.feedback".to_string(),
            session_id: "session.feedback".to_string(),
            item_id: "item.assistant".to_string(),
            rating: None,
            reason_code: None,
            comment: None,
            expected_version: Some(0),
            requested_by: subject(),
            requested_at: "2026-07-19T01:02:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(removed.version, 1);
    assert!(removed.deleted_at.is_some());

    let after_remove = service
        .list_item_feedback(ListItemFeedbackCommand {
            query: ItemFeedbackListQuery::for_user_session(100_001, 7, 100, "session.feedback"),
            path_agent_id: "agent.feedback".to_string(),
            requested_by: subject(),
        })
        .unwrap();
    assert!(after_remove.items.is_empty());

    let revived = service
        .update_item_feedback(UpdateItemFeedbackCommand {
            tenant_id: 100_001,
            organization_id: 7,
            user_id: 100,
            path_agent_id: "agent.feedback".to_string(),
            session_id: "session.feedback".to_string(),
            item_id: "item.assistant".to_string(),
            rating: Some(AgentItemFeedbackRating::Down),
            reason_code: None,
            comment: Some("Incorrect".to_string()),
            expected_version: None,
            requested_by: subject(),
            requested_at: "2026-07-19T01:03:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(revived.version, 2);
    assert_eq!(revived.rating, AgentItemFeedbackRating::Down);
}

fn session(id: u64, agent_id: &str, session_id: &str, owner_user_id: u64) -> AgentSessionRecord {
    AgentSessionRecord {
        id,
        session_id: session_id.to_string(),
        tenant_id: 10,
        organization_id: 20,
        agent_id: agent_id.to_string(),
        owner_user_id,
        project_id: None,
        session_kind: AgentSessionKind::Assistant,
        entry_surface: AgentSessionEntrySurface::Api,
        source_module: None,
        source_context_kind: None,
        source_context_id: None,
        parent_session_id: None,
        forked_from_turn_id: None,
        title: None,
        title_source: AgentSessionTitleSource::System,
        status: AgentSessionStatus::Active,
        item_count: 0,
        last_item_sequence: 0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        idempotency_key: None,
        payload_hash: None,
        created_by: owner_user_id,
        updated_by: owner_user_id,
        version: 0,
        created_at: "2026-07-19T00:00:00Z".to_string(),
        updated_at: "2026-07-19T00:00:00Z".to_string(),
        last_item_at: None,
        closed_at: None,
        archived_at: None,
        archived_by: None,
        deleted_at: None,
        deleted_by: None,
        retention_until: None,
    }
}

fn user_state(id: u64, session_id: &str, user_id: u64) -> AgentResourceUserStateRecord {
    AgentResourceUserStateRecord {
        id,
        tenant_id: 10,
        organization_id: 20,
        user_id,
        resource_type: AgentResourceType::Session,
        resource_id: session_id.to_string(),
        pinned_at: Some("2026-07-19T00:01:00Z".to_string()),
        hidden_at: None,
        last_opened_at: None,
        last_read_item_sequence: None,
        custom_title: None,
        version: 0,
        created_at: "2026-07-19T00:01:00Z".to_string(),
        updated_at: "2026-07-19T00:01:00Z".to_string(),
    }
}

#[test]
fn resource_user_state_repository_filters_before_pagination() {
    let repository = InMemoryAgentRepository::new();
    repository
        .insert_session(session(1, "agent.alpha", "session.alpha", 30))
        .unwrap();
    repository
        .insert_session(session(2, "agent.beta", "session.beta", 30))
        .unwrap();
    repository
        .insert_session(session(3, "agent.alpha", "session.foreign", 31))
        .unwrap();
    repository
        .upsert_resource_user_state(user_state(11, "session.alpha", 30), None)
        .unwrap();
    repository
        .upsert_resource_user_state(user_state(12, "session.beta", 30), None)
        .unwrap();
    repository
        .upsert_resource_user_state(user_state(13, "session.foreign", 30), None)
        .unwrap();

    let query = ResourceUserStateListQuery::for_user_sessions(10, 20, 30)
        .for_agent("agent.alpha")
        .for_resource_ids(vec![
            "session.alpha".to_string(),
            "session.missing".to_string(),
        ])
        .pinned_only()
        .with_pagination(PaginationParams::default().with_page_size(1));
    let records = repository.list_resource_user_states(&query).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].resource_id, "session.alpha");
    assert_eq!(repository.count_resource_user_states(&query).unwrap(), 1);
}

#[test]
fn postgres_user_state_sql_is_scoped_parameterized_and_versioned() {
    assert!(SQL_UPSERT_AGENT_RESOURCE_USER_STATE.contains("ON CONFLICT"));
    assert!(SQL_UPSERT_AGENT_RESOURCE_USER_STATE.contains("version = $15"));
    assert!(SQL_SELECT_AGENT_RESOURCE_USER_STATE.contains("tenant_id = $1"));
    assert!(SQL_SELECT_AGENT_RESOURCE_USER_STATE.contains("organization_id = $2"));
    assert!(SQL_SELECT_AGENT_RESOURCE_USER_STATE.contains("user_id = $3"));
    for sql in [
        SQL_LIST_AGENT_RESOURCE_USER_STATES,
        SQL_COUNT_AGENT_RESOURCE_USER_STATES,
    ] {
        assert!(sql.contains("session.owner_user_id = state.user_id"));
        assert!(sql.contains("session.agent_id = $5"));
        assert!(sql.contains("session.deleted_at IS NULL"));
        assert!(sql.contains("state.resource_id = ANY($8::text[])"));
    }
    assert!(SQL_LIST_AGENT_RESOURCE_USER_STATES.contains("LIMIT $9 OFFSET $10"));
}

#[test]
fn postgres_item_feedback_sql_is_scoped_parameterized_and_versioned() {
    assert!(SQL_UPSERT_AGENT_ITEM_FEEDBACK.contains("ON CONFLICT"));
    assert!(SQL_UPSERT_AGENT_ITEM_FEEDBACK.contains("version = $13"));
    assert!(SQL_UPSERT_AGENT_ITEM_FEEDBACK.contains("deleted_at IS NOT NULL"));
    assert!(SQL_SELECT_AGENT_ITEM_FEEDBACK.contains("tenant_id = $1"));
    assert!(SQL_SELECT_AGENT_ITEM_FEEDBACK.contains("organization_id = $2"));
    assert!(SQL_LIST_AGENT_ITEM_FEEDBACK.contains("item.session_id = $4"));
    assert!(SQL_LIST_AGENT_ITEM_FEEDBACK.contains("LIMIT $5 OFFSET $6"));
    assert!(SQL_COUNT_AGENT_ITEM_FEEDBACK.contains("feedback.user_id = $3"));
    for sql in [SQL_LIST_AGENT_ITEM_FEEDBACK, SQL_COUNT_AGENT_ITEM_FEEDBACK] {
        assert!(sql.contains("feedback.deleted_at IS NULL"));
        assert!(!sql.contains("item.deleted_at"));
    }
}
