#![cfg(feature = "postgres-sync")]

use sdkwork_intelligence_agents_service::{
    SQL_COMPLETE_AGENT_TURN_STATE, SQL_COUNT_AGENT, SQL_COUNT_AGENT_INTERACTIONS,
    SQL_COUNT_AGENT_SESSIONS, SQL_COUNT_AGENT_SESSION_ITEMS, SQL_INSERT_AGENT,
    SQL_INSERT_AGENT_COMPOSITION_SLOT, SQL_INSERT_AGENT_INTERACTION,
    SQL_INSERT_AGENT_PROVIDER_BINDING, SQL_INSERT_AGENT_SESSION, SQL_INSERT_AGENT_SESSION_ITEM,
    SQL_INSERT_AGENT_SESSION_RUNTIME_BINDING, SQL_INSERT_AGENT_TASK, SQL_INSERT_AGENT_TURN,
    SQL_INSERT_AUDIT_EVENT, SQL_LIST_AGENT, SQL_LIST_AGENT_COMPOSITION_SLOTS,
    SQL_LIST_AGENT_INTERACTIONS, SQL_LIST_AGENT_PROVIDER_BINDINGS, SQL_LIST_AGENT_SESSIONS,
    SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS, SQL_LIST_AGENT_SESSION_ITEMS,
    SQL_LIST_AGENT_SESSION_ITEMS_CURSOR_ASC, SQL_LIST_AGENT_SESSION_ITEMS_CURSOR_DESC,
    SQL_LIST_AGENT_SESSION_ITEMS_DESC, SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT,
    SQL_LIST_AGENT_TASKS, SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_RECORD_AGENT_SESSION_ITEM, SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
    SQL_SELECT_AGENT_COMPOSITION_SLOT, SQL_SELECT_AGENT_INTERACTION,
    SQL_SELECT_AGENT_PROVIDER_BINDING, SQL_SELECT_AGENT_SESSION, SQL_SELECT_AGENT_SESSION_ITEM,
    SQL_SELECT_AGENT_SESSION_RUNTIME_BINDING, SQL_SELECT_AGENT_TASK,
    SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY, SQL_SELECT_TASK_SCHEDULER_METRICS_SNAPSHOT,
    SQL_UPDATE_AGENT, SQL_UPDATE_AGENT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_INTERACTION,
    SQL_UPDATE_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT_SESSION, SQL_UPDATE_AGENT_SESSION_ITEM,
    SQL_UPDATE_AGENT_TASK, TASK_SCHEDULER_METRICS_COUNT_CAP,
};

#[test]
fn session_activity_snapshot_is_one_bounded_projection_query() {
    let sql = SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS;
    assert!(sql.contains("ORDER BY activity_at DESC, id DESC"));
    assert!(sql.contains("(s.activity_at, s.id) <"));
    assert!(sql.contains("LIMIT $9"));
    assert!(sql.contains("row_to_json(latest_turn)"));
    assert!(sql.contains("ORDER BY turn_row.id DESC"));
    assert!(sql.contains(") turn_activity ON TRUE"));
    assert!(sql.contains("ORDER BY turn_row.updated_at DESC, turn_row.id DESC"));
    assert!(sql.contains("row_to_json(pending_interaction)"));
    assert!(sql.contains("row_to_json(current_runtime_binding)"));
    assert!(sql.contains("row_to_json(binding_activity)"));
    assert!(sql.contains("row_to_json(session_user_state)"));
    assert!(sql.contains("interaction_row.status = 0"));
    assert!(sql.contains("interaction_row.kind ASC"));
    assert!(sql.contains("binding_row.is_current = TRUE"));
    assert!(sql.contains("AND binding_row.status = 0"));
    assert!(sql.contains("user_state_row.user_id = page.owner_user_id"));
    assert!(sql.contains("user_state_row.resource_type = 0"));
    assert!(sql.contains("user_state_row.resource_id = page.session_id"));
    assert!(sql.contains("THEN 'user_state'"));
    assert!(sql.contains("interaction_activity.interaction_id AS latest_interaction_id"));
    assert!(sql.contains("project_scope.workspace_id = $6"));
    assert!(sql.contains("activity_at, activity_source"));
    assert!(!sql.contains("activity_at::text AS activity_at"));
}

fn tenant_scoped_select_sql(sql: &str, table: &str) {
    assert!(
        sql.contains("WHERE tenant_id = $1"),
        "{table} select SQL must filter by tenant_id"
    );
    assert!(
        sql.contains("LIMIT 1"),
        "{table} get-by-id select SQL should be bounded"
    );
}

fn tenant_scoped_list_sql(sql: &str, table: &str) {
    assert!(
        sql.contains("WHERE tenant_id = $1") || sql.contains("WHERE s.tenant_id = $1"),
        "{table} list SQL must filter by tenant_id"
    );
}

fn tenant_scoped_update_sql(sql: &str, table: &str) {
    assert!(
        sql.contains("WHERE tenant_id ="),
        "{table} update SQL must filter by tenant_id"
    );
    assert!(
        sql.contains("version ="),
        "{table} update SQL must enforce optimistic concurrency"
    );
}

#[test]
fn postgres_composition_slot_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(
        SQL_SELECT_AGENT_COMPOSITION_SLOT,
        "ai_agent_composition_slot",
    );
    tenant_scoped_list_sql(
        SQL_LIST_AGENT_COMPOSITION_SLOTS,
        "ai_agent_composition_slot",
    );
    tenant_scoped_update_sql(
        SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        "ai_agent_composition_slot",
    );
}

// ---------------------------------------------------------------------------
// SQL Injection Security Tests (T-01)
// ---------------------------------------------------------------------------

/// Verifies that all SQL queries use parameterized queries ($1, $2, etc.)
/// and do NOT use string concatenation or format strings for user input.
/// This prevents SQL injection attacks per SECURITY_SPEC requirements.
fn assert_parameterized_query(sql: &str, query_name: &str) {
    // Check that query uses parameterized placeholders
    let has_params = sql.contains("$1") || sql.contains("?");
    assert!(
        has_params,
        "{query_name} must use parameterized queries ($1, $2, etc.) to prevent SQL injection"
    );

    // Check for dangerous string concatenation patterns
    let dangerous_patterns = [
        "format!(",
        "concat(",
        "String::from(",
        ".to_owned() +",
        "\" + ",
        "\" +",
        "+ \"",
        "f\"{",
        "println!(",
        "eprintln!(",
    ];

    for pattern in dangerous_patterns {
        assert!(
            !sql.contains(pattern),
            "{query_name} must not use string concatenation pattern '{pattern}' - use parameterized queries"
        );
    }

    // Verify LIKE clauses use parameterized input, not concatenated strings
    if sql.contains("LIKE") {
        assert!(
            sql.contains("LIKE LOWER($") || sql.contains("LIKE $") || sql.contains("LIKE '%$"),
            "{query_name} LIKE clause must use parameterized input, not string concatenation"
        );
    }
}

/// Verifies that INSERT statements use parameterized values.
fn assert_safe_insert(sql: &str, query_name: &str) {
    assert_parameterized_query(sql, query_name);

    // INSERT may use VALUES or INSERT ... SELECT when parent scope is resolved in SQL.
    assert!(
        sql.contains("VALUES") || sql.contains("values") || sql.contains(" SELECT "),
        "{query_name} must be a valid parameterized INSERT statement"
    );
}

/// Verifies that UPDATE statements use parameterized SET clauses.
fn assert_safe_update(sql: &str, query_name: &str) {
    assert_parameterized_query(sql, query_name);

    // UPDATE should have WHERE clause with tenant_id filter
    assert!(
        sql.contains("WHERE"),
        "{query_name} must have WHERE clause for security filtering"
    );
}

/// Verifies that SELECT statements have proper bounds (LIMIT).
fn assert_safe_select(sql: &str, query_name: &str) {
    assert_parameterized_query(sql, query_name);

    // SELECT should have LIMIT for pagination/safety
    assert!(
        sql.contains("LIMIT"),
        "{query_name} must have LIMIT clause to prevent unbounded result sets"
    );
}

#[test]
fn sql_injection_prevention_all_queries() {
    // Verify all SQL constants use parameterized queries

    // Agent queries
    assert_safe_select(
        SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
        "SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID",
    );
    assert_parameterized_query(SQL_LIST_AGENT, "SQL_LIST_AGENT");
    assert_safe_insert(SQL_INSERT_AGENT, "SQL_INSERT_AGENT");
    assert_safe_update(SQL_UPDATE_AGENT, "SQL_UPDATE_AGENT");

    // Provider binding queries
    assert_safe_select(
        SQL_SELECT_AGENT_PROVIDER_BINDING,
        "SQL_SELECT_AGENT_PROVIDER_BINDING",
    );
    assert_parameterized_query(
        SQL_LIST_AGENT_PROVIDER_BINDINGS,
        "SQL_LIST_AGENT_PROVIDER_BINDINGS",
    );
    assert_safe_insert(
        SQL_INSERT_AGENT_PROVIDER_BINDING,
        "SQL_INSERT_AGENT_PROVIDER_BINDING",
    );
    assert_safe_update(
        SQL_UPDATE_AGENT_PROVIDER_BINDING,
        "SQL_UPDATE_AGENT_PROVIDER_BINDING",
    );

    // Composition slot queries
    assert_safe_select(
        SQL_SELECT_AGENT_COMPOSITION_SLOT,
        "SQL_SELECT_AGENT_COMPOSITION_SLOT",
    );
    assert_parameterized_query(
        SQL_LIST_AGENT_COMPOSITION_SLOTS,
        "SQL_LIST_AGENT_COMPOSITION_SLOTS",
    );
    assert_safe_insert(
        SQL_INSERT_AGENT_COMPOSITION_SLOT,
        "SQL_INSERT_AGENT_COMPOSITION_SLOT",
    );
    assert_safe_update(
        SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        "SQL_UPDATE_AGENT_COMPOSITION_SLOT",
    );

    // Audit event queries
    assert_safe_insert(SQL_INSERT_AUDIT_EVENT, "SQL_INSERT_AUDIT_EVENT");

    // Session queries
    assert_safe_select(SQL_SELECT_AGENT_SESSION, "SQL_SELECT_AGENT_SESSION");
    assert_parameterized_query(SQL_LIST_AGENT_SESSIONS, "SQL_LIST_AGENT_SESSIONS");
    assert_safe_insert(SQL_INSERT_AGENT_SESSION, "SQL_INSERT_AGENT_SESSION");
    assert_safe_update(SQL_UPDATE_AGENT_SESSION, "SQL_UPDATE_AGENT_SESSION");

    // Message queries
    assert_safe_select(
        SQL_SELECT_AGENT_SESSION_ITEM,
        "SQL_SELECT_AGENT_SESSION_ITEM",
    );
    assert_parameterized_query(SQL_LIST_AGENT_SESSION_ITEMS, "SQL_LIST_AGENT_SESSION_ITEMS");
    assert_safe_insert(
        SQL_INSERT_AGENT_SESSION_ITEM,
        "SQL_INSERT_AGENT_SESSION_ITEM",
    );
    assert_safe_update(
        SQL_UPDATE_AGENT_SESSION_ITEM,
        "SQL_UPDATE_AGENT_SESSION_ITEM",
    );
    assert_safe_update(
        SQL_RECORD_AGENT_SESSION_ITEM,
        "SQL_RECORD_AGENT_SESSION_ITEM",
    );
}

#[test]
fn create_flow_casts_rfc3339_text_to_postgres_timestamptz() {
    assert!(SQL_INSERT_AGENT.contains("$18::timestamptz"));
    assert!(SQL_INSERT_AGENT.contains("$19::timestamptz"));
    assert!(SQL_INSERT_AGENT.contains("$20::timestamptz"));
    assert!(SQL_INSERT_AUDIT_EVENT.contains("$15::timestamptz"));
    assert!(SQL_INSERT_AGENT_COMPOSITION_SLOT.contains("$16::timestamptz"));
    assert!(SQL_INSERT_AGENT_COMPOSITION_SLOT.contains("$17::timestamptz"));
    assert!(SQL_INSERT_AGENT_COMPOSITION_SLOT.contains("$18::timestamptz"));
}

#[test]
fn audit_event_sql_uses_authoritative_actor_columns() {
    for sql in [
        SQL_INSERT_AUDIT_EVENT,
        SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    ] {
        assert!(sql.contains("actor_type"));
        assert!(sql.contains("actor_id"));
        assert!(!sql.contains("subject_id"));
        assert!(!sql.contains("subject_tenant_id"));
    }
}

#[test]
fn postgres_session_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_SESSION, "ai_agent_session");
    tenant_scoped_list_sql(SQL_LIST_AGENT_SESSIONS, "ai_agent_session");
    tenant_scoped_update_sql(SQL_UPDATE_AGENT_SESSION, "ai_agent_session");
}

#[test]
fn provider_session_identity_is_owner_scoped_in_database_contract() {
    const BASELINE: &str =
        include_str!("../../../database/ddl/baseline/postgres/0001_agents_baseline.sql");

    assert!(SQL_INSERT_AGENT_SESSION_RUNTIME_BINDING.contains("owner_user_id"));
    assert!(SQL_SELECT_AGENT_SESSION_RUNTIME_BINDING.contains("owner_user_id"));
    assert!(BASELINE.contains(
        "tenant_id, organization_id, session_id, owner_user_id\n    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id, owner_user_id)"
    ));
    assert!(BASELINE.contains(
        "tenant_id, organization_id, owner_user_id, provider_binding_id, provider_id, provider_session_id"
    ));
}

#[test]
fn postgres_session_item_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_SESSION_ITEM, "ai_agent_session_item");
    tenant_scoped_list_sql(SQL_LIST_AGENT_SESSION_ITEMS, "ai_agent_session_item");
    assert!(
        SQL_UPDATE_AGENT_SESSION_ITEM.contains("WHERE tenant_id ="),
        "ai_agent_session_item update SQL must filter by tenant_id"
    );
}

#[test]
fn postgres_session_item_selects_expose_raw_timestamptz_columns() {
    // Provider-history reconciliation parses stored item instants as RFC3339
    // (reconcile_provider_session_history_session_item compares
    // "existing.updatedAt"); the row mapper normalizes raw timestamptz columns
    // into RFC3339, so the projections must not text-cast the timestamp fields
    // into PostgreSQL's display format (e.g. "2026-07-26 08:36:21+00"), which
    // is not RFC3339.
    let session_item_selects = [
        SQL_SELECT_AGENT_SESSION_ITEM,
        SQL_LIST_AGENT_SESSION_ITEMS,
        SQL_LIST_AGENT_SESSION_ITEMS_DESC,
        SQL_LIST_AGENT_SESSION_ITEMS_CURSOR_ASC,
        SQL_LIST_AGENT_SESSION_ITEMS_CURSOR_DESC,
        SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT,
    ];
    for sql in session_item_selects {
        for column in [
            "created_at",
            "updated_at",
            "completed_at",
            "redacted_at",
            "retention_until",
        ] {
            assert!(
                !sql.contains(&format!("{column}::text AS {column}")),
                "session item SELECT must expose {column} as raw timestamptz so the row mapper can format RFC3339"
            );
            assert!(
                sql.contains(column),
                "session item SELECT must still project the {column} column"
            );
        }
    }
}

#[test]
fn session_aggregate_columns_are_populated_without_item_owner_duplication() {
    for required in ["created_by", "updated_by", "last_item_sequence"] {
        assert!(
            SQL_INSERT_AGENT_SESSION.contains(required),
            "session insert must populate {required}"
        );
    }
    for required in [
        "organization_id",
        "session_id",
        "item_id",
        "kind",
        "turn_id",
        "created_by",
    ] {
        assert!(
            SQL_INSERT_AGENT_SESSION_ITEM.contains(required),
            "session-item insert must populate {required}"
        );
    }
    assert!(
        !SQL_INSERT_AGENT_SESSION_ITEM.contains("owner_user_id"),
        "session items must inherit owner scope through the session aggregate"
    );
    assert!(SQL_RECORD_AGENT_SESSION_ITEM.contains("item_count = item_count + 1"));
    assert!(SQL_RECORD_AGENT_SESSION_ITEM.contains("last_item_sequence = last_item_sequence + 1"));
    assert!(SQL_RECORD_AGENT_SESSION_ITEM.contains("RETURNING id, uuid"));
    assert!(SQL_UPDATE_AGENT_SESSION.contains("last_item_sequence = GREATEST"));
    for sql in [SQL_SELECT_AGENT_SESSION, SQL_LIST_AGENT_SESSIONS] {
        assert!(sql.contains("deleted_at IS NULL"));
    }
    assert!(SQL_SELECT_AGENT_SESSION_ITEM.contains("redacted_at"));
    assert!(SQL_LIST_AGENT_SESSION_ITEMS.contains("redacted_at"));
}

#[test]
fn turn_sql_uses_scoped_idempotency_and_links_items() {
    assert!(SQL_INSERT_AGENT_TURN.contains("idempotency_key"));
    assert!(SQL_INSERT_AGENT_TURN.contains("response_item_id"));
    assert!(SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY.contains("tenant_id = $1"));
    assert!(SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY.contains("organization_id = $2"));
    assert!(SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY.contains("owner_user_id = $3"));
    assert!(SQL_INSERT_AGENT_SESSION_ITEM.contains("turn_id"));
    assert!(SQL_COMPLETE_AGENT_TURN_STATE.contains("version = $34"));
    assert!(SQL_COMPLETE_AGENT_TURN_STATE.contains("status = 1"));
    assert!(SQL_COMPLETE_AGENT_TURN_STATE.contains("response_item_id IS NULL"));
    assert!(SQL_COMPLETE_AGENT_TURN_STATE.contains("fencing_token = $35"));
    assert!(SQL_COMPLETE_AGENT_TURN_STATE.contains("lease_token IS NOT DISTINCT FROM $36"));
}

#[test]
fn postgres_session_list_has_mandatory_pagination() {
    assert!(
        SQL_LIST_AGENT_SESSIONS.contains("LIMIT $11"),
        "SQL_LIST_AGENT_SESSIONS must have LIMIT parameter for mandatory pagination"
    );
    assert!(
        SQL_LIST_AGENT_SESSIONS.contains("(s.updated_at, s.id) < ($9::timestamptz, $10::bigint)"),
        "SQL_LIST_AGENT_SESSIONS must keyset-paginate at the store"
    );
    assert!(SQL_LIST_AGENT_SESSIONS.contains("organization_id = $2"));
    assert!(SQL_LIST_AGENT_SESSIONS.contains("project_id = $4"));
    assert!(SQL_LIST_AGENT_SESSIONS.contains("p.workspace_id = $5"));
    assert!(SQL_COUNT_AGENT_SESSIONS.contains("p.workspace_id = $5"));
}

#[test]
fn postgres_session_item_list_has_mandatory_pagination() {
    assert!(
        SQL_LIST_AGENT_SESSION_ITEMS.contains("LIMIT $6"),
        "SQL_LIST_AGENT_SESSION_ITEMS must have LIMIT parameter for mandatory pagination"
    );
    assert!(
        SQL_LIST_AGENT_SESSION_ITEMS.contains("OFFSET $7"),
        "SQL_LIST_AGENT_SESSION_ITEMS must have OFFSET parameter for page navigation"
    );
    assert_parameterized_query(
        SQL_COUNT_AGENT_SESSION_ITEMS,
        "SQL_COUNT_AGENT_SESSION_ITEMS",
    );
    assert!(
        SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT.contains("ORDER BY sequence DESC"),
        "SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT must fetch the most recent context window first"
    );
    assert!(
        SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT.contains("LIMIT $6"),
        "SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT must bound the turn context window"
    );
}

#[test]
fn postgres_session_list_has_mandatory_count_query() {
    assert_parameterized_query(SQL_COUNT_AGENT_SESSIONS, "SQL_COUNT_AGENT_SESSIONS");
}

#[test]
fn sql_list_agent_has_mandatory_pagination() {
    // DATABASE_SPEC §16: All list queries must have mandatory LIMIT and OFFSET
    assert!(
        SQL_LIST_AGENT.contains("LIMIT $7"),
        "SQL_LIST_AGENT must have LIMIT parameter for mandatory pagination (DATABASE_SPEC §16)"
    );
    assert!(
        SQL_LIST_AGENT.contains("OFFSET $8"),
        "SQL_LIST_AGENT must have OFFSET parameter for page navigation"
    );
}

#[test]
fn sql_list_agent_pushes_category_and_role_scope_filters() {
    // The two optional list filters must narrow the same WHERE clause that
    // already carries tenant/org/owner/visibility scoping, so neither filter can
    // widen the listing; both list and count must stay in sync.
    for (query_name, sql) in [
        ("SQL_LIST_AGENT", SQL_LIST_AGENT),
        ("SQL_COUNT_AGENT", SQL_COUNT_AGENT),
    ] {
        assert!(
            sql.contains("category_code = $"),
            "{query_name} must push the category filter into the WHERE clause"
        );
        assert!(
            sql.contains("jsonb_array_length(allowed_role_codes) = 0"),
            "{query_name} must keep agents whose allowed_role_codes list is empty"
        );
        assert!(
            sql.contains("allowed_role_codes ?| $"),
            "{query_name} must intersect the role filter through the JSONB array operator"
        );
    }
}

#[test]
fn sql_search_query_is_parameterized() {
    // Verify search query uses parameterized LIKE, not string concatenation
    assert!(
        SQL_LIST_AGENT.contains("LIKE LOWER($5::text)") || SQL_LIST_AGENT.contains("LIKE $5"),
        "SQL_LIST_AGENT search query must use parameterized LIKE clause to prevent SQL injection"
    );

    // Verify search query is wrapped in %...% for substring matching
    // (This wrapping should happen in application code, not SQL)
    assert!(
        !SQL_LIST_AGENT.contains("'%{}%'") && !SQL_LIST_AGENT.contains("'%s%'"),
        "SQL_LIST_AGENT must not have hardcoded search patterns - wrapping should be in application code"
    );
}

#[test]
fn all_queries_enforce_tenant_isolation() {
    // SECURITY_SPEC: All queries must enforce tenant isolation
    let queries = [
        (
            "SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID",
            SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
        ),
        ("SQL_LIST_AGENT", SQL_LIST_AGENT),
        ("SQL_UPDATE_AGENT", SQL_UPDATE_AGENT),
        (
            "SQL_SELECT_AGENT_PROVIDER_BINDING",
            SQL_SELECT_AGENT_PROVIDER_BINDING,
        ),
        (
            "SQL_LIST_AGENT_PROVIDER_BINDINGS",
            SQL_LIST_AGENT_PROVIDER_BINDINGS,
        ),
        (
            "SQL_UPDATE_AGENT_PROVIDER_BINDING",
            SQL_UPDATE_AGENT_PROVIDER_BINDING,
        ),
        (
            "SQL_SELECT_AGENT_COMPOSITION_SLOT",
            SQL_SELECT_AGENT_COMPOSITION_SLOT,
        ),
        (
            "SQL_LIST_AGENT_COMPOSITION_SLOTS",
            SQL_LIST_AGENT_COMPOSITION_SLOTS,
        ),
        (
            "SQL_UPDATE_AGENT_COMPOSITION_SLOT",
            SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        ),
        ("SQL_SELECT_AGENT_SESSION", SQL_SELECT_AGENT_SESSION),
        ("SQL_LIST_AGENT_SESSIONS", SQL_LIST_AGENT_SESSIONS),
        ("SQL_UPDATE_AGENT_SESSION", SQL_UPDATE_AGENT_SESSION),
        (
            "SQL_SELECT_AGENT_SESSION_ITEM",
            SQL_SELECT_AGENT_SESSION_ITEM,
        ),
        ("SQL_LIST_AGENT_SESSION_ITEMS", SQL_LIST_AGENT_SESSION_ITEMS),
        (
            "SQL_UPDATE_AGENT_SESSION_ITEM",
            SQL_UPDATE_AGENT_SESSION_ITEM,
        ),
        ("SQL_SELECT_AGENT_INTERACTION", SQL_SELECT_AGENT_INTERACTION),
        ("SQL_LIST_AGENT_INTERACTIONS", SQL_LIST_AGENT_INTERACTIONS),
        ("SQL_UPDATE_AGENT_INTERACTION", SQL_UPDATE_AGENT_INTERACTION),
        ("SQL_SELECT_AGENT_TASK", SQL_SELECT_AGENT_TASK),
        ("SQL_LIST_AGENT_TASKS", SQL_LIST_AGENT_TASKS),
        ("SQL_UPDATE_AGENT_TASK", SQL_UPDATE_AGENT_TASK),
    ];

    for (name, sql) in queries {
        assert!(
            sql.contains("tenant_id = $") || sql.contains("tenant_id ="),
            "{name} must enforce tenant isolation via tenant_id filter"
        );
    }
}

#[test]
fn update_queries_enforce_optimistic_concurrency() {
    // DATABASE_SPEC: All UPDATE queries must enforce optimistic concurrency via version check
    let update_queries = [
        ("SQL_UPDATE_AGENT", SQL_UPDATE_AGENT),
        (
            "SQL_UPDATE_AGENT_PROVIDER_BINDING",
            SQL_UPDATE_AGENT_PROVIDER_BINDING,
        ),
        (
            "SQL_UPDATE_AGENT_COMPOSITION_SLOT",
            SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        ),
        ("SQL_UPDATE_AGENT_INTERACTION", SQL_UPDATE_AGENT_INTERACTION),
        ("SQL_UPDATE_AGENT_TASK", SQL_UPDATE_AGENT_TASK),
    ];

    for (name, sql) in update_queries {
        assert!(
            sql.contains("version ="),
            "{name} must enforce optimistic concurrency via version check"
        );
    }
}

#[test]
fn postgres_interaction_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_INTERACTION, "ai_agent_interaction");
    tenant_scoped_list_sql(SQL_LIST_AGENT_INTERACTIONS, "ai_agent_interaction");
    tenant_scoped_update_sql(SQL_UPDATE_AGENT_INTERACTION, "ai_agent_interaction");
    assert_parameterized_query(SQL_COUNT_AGENT_INTERACTIONS, "SQL_COUNT_AGENT_INTERACTIONS");
    assert!(
        SQL_COUNT_AGENT_INTERACTIONS.contains("tenant_id = $1"),
        "ai_agent_interaction count SQL must filter by tenant_id"
    );
    assert!(
        SQL_INSERT_AGENT_INTERACTION.contains("tenant_id"),
        "ai_agent_interaction insert SQL must include tenant_id"
    );
    for (name, sql) in [
        ("list", SQL_LIST_AGENT_INTERACTIONS),
        ("count", SQL_COUNT_AGENT_INTERACTIONS),
    ] {
        assert!(
            sql.contains("status = $4") && sql.contains("kind = $5"),
            "ai_agent_interaction {name} SQL must apply the same status and kind filters"
        );
    }
    assert!(
        SQL_LIST_AGENT_INTERACTIONS.contains("(created_at, id) < ($6::timestamptz, $7::bigint)")
            && SQL_LIST_AGENT_INTERACTIONS.contains("LIMIT $8"),
        "ai_agent_interaction list SQL must keyset-paginate at the store"
    );
}

#[test]
fn postgres_session_item_sql_supports_stable_bidirectional_pages() {
    assert!(
        SQL_LIST_AGENT_SESSION_ITEMS.contains("ORDER BY sequence ASC, id ASC LIMIT $6 OFFSET $7")
    );
    assert!(SQL_LIST_AGENT_SESSION_ITEMS_DESC
        .contains("ORDER BY sequence DESC, id DESC LIMIT $6 OFFSET $7"));
    for sql in [
        SQL_LIST_AGENT_SESSION_ITEMS,
        SQL_LIST_AGENT_SESSION_ITEMS_DESC,
        SQL_COUNT_AGENT_SESSION_ITEMS,
    ] {
        assert!(sql.contains("kind = $4") && sql.contains("status = $5"));
    }
}

#[test]
fn postgres_task_sql_is_tenant_and_organization_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_TASK, "ai_agent_task");
    tenant_scoped_list_sql(SQL_LIST_AGENT_TASKS, "ai_agent_task");
    tenant_scoped_update_sql(SQL_UPDATE_AGENT_TASK, "ai_agent_task");
    assert!(
        SQL_INSERT_AGENT_TASK.contains("tenant_id"),
        "ai_agent_task insert SQL must include tenant_id"
    );
    assert!(
        SQL_SELECT_AGENT_TASK.contains("tenant_id = $1 AND organization_id = $2 AND task_id = $3"),
        "ai_agent_task select SQL must filter by tenant and organization"
    );
    assert!(
        SQL_LIST_AGENT_TASKS.contains("tenant_id = $1 AND organization_id = $2"),
        "ai_agent_task list SQL must filter by tenant and organization"
    );
    assert!(
        SQL_UPDATE_AGENT_TASK.contains(
            "tenant_id = $28 AND organization_id = $29 AND task_id = $30 AND version = $31"
        ),
        "ai_agent_task update SQL must filter by tenant and organization"
    );
    assert!(
        SQL_LIST_AGENT_TASKS.contains("(updated_at, id) < ($6::timestamptz, $7::bigint)")
            && SQL_LIST_AGENT_TASKS.contains("ORDER BY updated_at DESC, id DESC LIMIT $8")
            && !SQL_LIST_AGENT_TASKS.contains("OFFSET"),
        "ai_agent_task list SQL must use store-level updated_at/id keyset pagination"
    );
}

#[test]
fn task_scheduler_metrics_query_is_bounded_indexable_and_sanitized() {
    let sql = SQL_SELECT_TASK_SCHEDULER_METRICS_SNAPSHOT;
    assert_eq!(TASK_SCHEDULER_METRICS_COUNT_CAP, 100_000);
    assert!(sql.contains("next_fire_at <= $1::timestamptz"));
    assert!(sql.contains("available_at <= $1::timestamptz"));
    assert!(sql.contains("status IN (1, 2) AND lease_expires_at >= $1::timestamptz"));
    assert!(sql.contains("WHERE status = 6"));
    assert!(sql.contains("WHERE status IN (0, 1, 3)"));
    assert_eq!(sql.matches("LIMIT $2").count(), 5);
    for required_projection in [
        "materialization_lag_seconds",
        "eligible_run_oldest_age_seconds",
        "reconciliation_oldest_age_seconds",
        "outbox_oldest_age_seconds",
    ] {
        assert!(sql.contains(required_projection));
    }
    for forbidden_dimension in [
        "tenant_id",
        "organization_id",
        "owner_user_id",
        "task_id",
        "run_id",
        "worker_id",
        "lease_token",
        "error_detail",
    ] {
        assert!(!sql.contains(forbidden_dimension));
    }
}
