pub const SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json::text AS manifest_json, default_code_task_intent_json::text AS default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json::text AS tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 AND agent_id = $2 LIMIT 1";
pub const SQL_INSERT_AGENT: &str =
    "INSERT INTO ai_agent (id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at, updated_at, deleted_at, version) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, $11::jsonb, $12, $13, $14, $15, $16, $17::jsonb, $18::timestamptz, $19::timestamptz, $20::timestamptz, $21)";
pub const SQL_UPDATE_AGENT: &str =
    "UPDATE ai_agent SET organization_id = $1, owner_user_id = $2, code = $3, display_name = $4, description = $5, manifest_json = $6::jsonb, default_code_task_intent_json = $7::jsonb, implementation_provider_id = $8, implementation_kind = $9, implementation_type = $10, status = $11, visibility = $12, tags_json = $13::jsonb, updated_at = $14::timestamptz, deleted_at = $15::timestamptz, version = $16 WHERE tenant_id = $17 AND agent_id = $18 AND version = $19";
pub const SQL_LIST_AGENT: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json::text AS manifest_json, default_code_task_intent_json::text AS default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json::text AS tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 AND ($2 IS NULL OR organization_id = $2) AND ($3 IS NULL OR owner_user_id = $3) AND (deleted_at IS NULL OR $4::bool = true) AND ($5::text IS NULL OR LOWER(agent_id) LIKE LOWER($5::text) OR LOWER(code) LIKE LOWER($5::text) OR LOWER(display_name) LIKE LOWER($5::text) OR LOWER(COALESCE(description, '')) LIKE LOWER($5::text)) AND ($6::smallint IS NULL OR visibility = $6) ORDER BY updated_at DESC, id DESC LIMIT $7 OFFSET $8";

// All filtering (organization_id, owner_user_id, include_deleted, search_query, visibility)
// is pushed to SQL WHERE clause. Parameters: $1=tenant_id, $2=org_filter(NULL=any),
// $3=owner_filter(NULL=any), $4=include_deleted_flag, $5=search_query(NULL=none, wrapped %...% for LIKE),
// $6=visibility(NULL=any), $7=page_size, $8=offset.
pub const SQL_COUNT_AGENT: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent WHERE tenant_id = $1 AND ($2 IS NULL OR organization_id = $2) AND ($3 IS NULL OR owner_user_id = $3) AND (deleted_at IS NULL OR $4::bool = true) AND ($5::text IS NULL OR LOWER(agent_id) LIKE LOWER($5::text) OR LOWER(code) LIKE LOWER($5::text) OR LOWER(display_name) LIKE LOWER($5::text) OR LOWER(COALESCE(description, '')) LIKE LOWER($5::text)) AND ($6::smallint IS NULL OR visibility = $6)";

#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_WORKSPACE: &str =
    "INSERT INTO ai_agent_workspace (id, uuid, tenant_id, organization_id, workspace_id, owner_user_id, name, description, is_default, status, created_by, updated_by, version, created_at, updated_at, archived_at, archived_by, deleted_at, deleted_by, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14::timestamptz, $15::timestamptz, $16::timestamptz, $17, $18::timestamptz, $19, $20::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_WORKSPACE: &str =
    "UPDATE ai_agent_workspace SET name = $1, description = $2, status = $3, updated_by = $4, version = $5, updated_at = $6::timestamptz, archived_at = $7::timestamptz, archived_by = $8, deleted_at = $9::timestamptz, deleted_by = $10, retention_until = $11::timestamptz WHERE tenant_id = $12 AND organization_id = $13 AND workspace_id = $14 AND version = $15";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_WORKSPACE: &str =
    "SELECT id, uuid, tenant_id, organization_id, workspace_id, owner_user_id, name, description, is_default, status, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_workspace WHERE tenant_id = $1 AND organization_id = $2 AND workspace_id = $3 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_DEFAULT_AGENT_WORKSPACE: &str =
    "SELECT id, uuid, tenant_id, organization_id, workspace_id, owner_user_id, name, description, is_default, status, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_workspace WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND is_default = TRUE AND status = 0 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_WORKSPACES: &str =
    "SELECT id, uuid, tenant_id, organization_id, workspace_id, owner_user_id, name, description, is_default, status, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_workspace WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND ($4::smallint IS NULL OR status = $4) AND ($5::bool = TRUE OR deleted_at IS NULL) ORDER BY is_default DESC, updated_at DESC, id DESC LIMIT $6 OFFSET $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_WORKSPACES: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_workspace WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND ($4::smallint IS NULL OR status = $4) AND ($5::bool = TRUE OR deleted_at IS NULL)";

#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_PROJECT: &str =
    "INSERT INTO ai_agent_project (id, uuid, tenant_id, organization_id, project_id, workspace_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, import_source_kind, import_source_ref, drive_space_id, drive_root_entry_id, drive_logical_path, created_by, updated_by, version, created_at, updated_at, archived_at, archived_by, deleted_at, deleted_by, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23::timestamptz, $24::timestamptz, $25::timestamptz, $26, $27::timestamptz, $28, $29::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_PROJECT: &str =
    "UPDATE ai_agent_project SET workspace_id = $1, name = $2, description = $3, visibility = $4, status = $5, drive_access_mode = $6, default_agent_id = $7, default_model_id = $8, import_source_kind = $9, import_source_ref = $10, drive_space_id = $11, drive_root_entry_id = $12, drive_logical_path = $13, updated_by = $14, version = $15, updated_at = $16::timestamptz, archived_at = $17::timestamptz, archived_by = $18, deleted_at = $19::timestamptz, deleted_by = $20, retention_until = $21::timestamptz WHERE tenant_id = $22 AND organization_id = $23 AND project_id = $24 AND version = $25";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_PROJECT: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, workspace_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, import_source_kind, import_source_ref, drive_space_id, drive_root_entry_id, drive_logical_path, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LOCK_AGENT_PROJECT_WORKSPACE_NAME: &str =
    "SELECT pg_advisory_xact_lock(hashtextextended($1::bigint::text || chr(31) || $2::bigint::text || chr(31) || $3 || chr(31) || LOWER(BTRIM($4)), 0))";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, workspace_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, import_source_kind, import_source_ref, drive_space_id, drive_root_entry_id, drive_logical_path, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND workspace_id = $3 AND LOWER(BTRIM(name)) = LOWER(BTRIM($4)) AND deleted_at IS NULL ORDER BY CASE WHEN status = 0 THEN 0 ELSE 1 END ASC, created_at ASC, id ASC LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_PROJECT_BY_IMPORT_SOURCE: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, workspace_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, import_source_kind, import_source_ref, drive_space_id, drive_root_entry_id, drive_logical_path, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND import_source_kind = $4 AND import_source_ref = $5 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_PROJECTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, workspace_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, import_source_kind, import_source_ref, drive_space_id, drive_root_entry_id, drive_logical_path, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::text IS NULL OR workspace_id = $4) AND ($5::text IS NULL OR LOWER(BTRIM(name)) = LOWER(BTRIM($5))) AND ($6::smallint IS NULL OR status = $6) AND ($7::text IS NULL OR name ILIKE $7 OR COALESCE(description, '') ILIKE $7) AND ($8::bool = TRUE OR deleted_at IS NULL) ORDER BY updated_at DESC, id DESC LIMIT $9 OFFSET $10";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_PROJECTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::text IS NULL OR workspace_id = $4) AND ($5::text IS NULL OR LOWER(BTRIM(name)) = LOWER(BTRIM($5))) AND ($6::smallint IS NULL OR status = $6) AND ($7::text IS NULL OR name ILIKE $7 OR COALESCE(description, '') ILIKE $7) AND ($8::bool = TRUE OR deleted_at IS NULL)";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT: &str =
    "INSERT INTO ai_agent_project_composition_slot (id, uuid, tenant_id, organization_id, project_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, created_by, updated_by, version, created_at, updated_at, deleted_at, deleted_by, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb, $14, $15, $16, $17::timestamptz, $18::timestamptz, $19::timestamptz, $20, $21::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT: &str =
    "UPDATE ai_agent_project_composition_slot SET slot_kind = $1, target_module = $2, target_ref = $3, target_version_ref = $4, priority = $5, enabled = $6, policy_json = $7::jsonb, updated_by = $8, version = $9, updated_at = $10::timestamptz, deleted_at = $11::timestamptz, deleted_by = $12, retention_until = $13::timestamptz WHERE tenant_id = $14 AND organization_id = $15 AND project_id = $16 AND slot_id = $17 AND version = $18";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project_composition_slot WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND slot_id = $4 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project_composition_slot WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND deleted_at IS NULL AND ($4::text IS NULL OR slot_kind = $4) AND ($5::bool IS NULL OR enabled = $5) ORDER BY priority ASC, id ASC LIMIT $6 OFFSET $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_project_composition_slot WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND deleted_at IS NULL AND ($4::text IS NULL OR slot_kind = $4) AND ($5::bool IS NULL OR enabled = $5)";
pub const SQL_INSERT_AGENT_PROVIDER_BINDING: &str =
    "INSERT INTO ai_agent_runtime_binding (id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10, $11, $12::timestamptz, $13::timestamptz)";
pub const SQL_UPDATE_AGENT_PROVIDER_BINDING: &str =
    "UPDATE ai_agent_runtime_binding SET provider_id = $1, implementation_kind = $2, configuration_profile_id = $3, capabilities_json = $4::jsonb, active = $5, version = $6, updated_at = $7::timestamptz WHERE tenant_id = $8 AND agent_id = $9 AND binding_id = $10 AND version = $11";
pub const SQL_DEACTIVATE_ACTIVE_AGENT_PROVIDER_BINDINGS: &str =
    "UPDATE ai_agent_runtime_binding SET active = false, version = version + 1, updated_at = $3::timestamptz WHERE tenant_id = $1 AND agent_id = $2 AND active = true";
pub const SQL_ACTIVATE_AGENT_PROVIDER_BINDING: &str =
    "UPDATE ai_agent_runtime_binding SET active = true, version = $4, updated_at = $5::timestamptz WHERE tenant_id = $1 AND agent_id = $2 AND binding_id = $3 AND version = $6";
pub const SQL_SELECT_AGENT_PROVIDER_BINDING: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json::text AS capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 AND binding_id = $3 LIMIT 1";
/// Load the single active provider binding for an agent without paginated scans.
/// Used on hot paths (chat, task, preview) where only the active binding matters.
pub const SQL_SELECT_ACTIVE_AGENT_PROVIDER_BINDING: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json::text AS capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 AND active = TRUE LIMIT 1";
pub const SQL_LIST_AGENT_PROVIDER_BINDINGS: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json::text AS capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 ORDER BY active DESC, updated_at DESC, binding_id ASC LIMIT $3 OFFSET $4";
pub const SQL_COUNT_AGENT_PROVIDER_BINDINGS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2";
pub const SQL_INSERT_AUDIT_EVENT: &str =
    "INSERT INTO ai_agent_audit_event (id, uuid, tenant_id, organization_id, aggregate_type, aggregate_id, agent_internal_id, agent_id, action, actor_type, actor_id, request_id, trace_id, payload_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7::bigint, (SELECT id FROM ai_agent WHERE tenant_id = $3 AND agent_id = $8)), $8, $9, $10, $11, $12, $13, $14::jsonb, $15::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, aggregate_type, aggregate_id, agent_internal_id, agent_id, action, actor_type, actor_id, request_id, trace_id, payload_json::text AS payload_json, created_at::text AS created_at FROM ai_agent_audit_event WHERE tenant_id = $1 AND agent_id = $2 AND ($3::text IS NULL OR action = $3) AND ($4::text IS NULL OR created_at >= $4::timestamptz) AND ($5::text IS NULL OR created_at <= $5::timestamptz) AND ($6::timestamptz IS NULL OR (created_at, uuid) < ($6::timestamptz, $7::text)) ORDER BY created_at DESC, uuid DESC LIMIT $8";
pub const SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_audit_event WHERE tenant_id = $1 AND agent_id = $2 AND ($3::text IS NULL OR action = $3) AND ($4::text IS NULL OR created_at >= $4::timestamptz) AND ($5::text IS NULL OR created_at <= $5::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_RUNTIME_EXECUTION: &str =
    "INSERT INTO ai_agent_runtime_execution (id, tenant_id, agent_id, execution_id, operation, status, input_payload_json, output_payload_json, requested_at, completed_at) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8::jsonb, $9::timestamptz, $10::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_RUNTIME_EXECUTION: &str =
    "UPDATE ai_agent_runtime_execution SET status = $1, output_payload_json = $2::jsonb, completed_at = $3::timestamptz WHERE tenant_id = $4 AND agent_id = $5 AND execution_id = $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_RUNTIME_EXECUTION: &str =
    "SELECT id, tenant_id, agent_id, execution_id, operation, status, input_payload_json::text AS input_payload_json, output_payload_json::text AS output_payload_json, requested_at::text AS requested_at, completed_at::text AS completed_at FROM ai_agent_runtime_execution WHERE tenant_id = $1 AND agent_id = $2 AND execution_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_RUNTIME_EXECUTIONS: &str =
    "SELECT id, tenant_id, agent_id, execution_id, operation, status, input_payload_json::text AS input_payload_json, output_payload_json::text AS output_payload_json, requested_at::text AS requested_at, completed_at::text AS completed_at FROM ai_agent_runtime_execution WHERE tenant_id = $1 AND agent_id = $2 AND ($3::text IS NULL OR status = $3) AND ($4::timestamptz IS NULL OR (requested_at, execution_id) < ($4::timestamptz, $5)) ORDER BY requested_at DESC, execution_id DESC LIMIT $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_STALE_RUNTIME_EXECUTIONS: &str =
    "SELECT id, tenant_id, agent_id, execution_id, operation, status, input_payload_json::text AS input_payload_json, output_payload_json::text AS output_payload_json, requested_at::text AS requested_at, completed_at::text AS completed_at FROM ai_agent_runtime_execution WHERE tenant_id = $1 AND status IN ('queued', 'running') AND completed_at < $2::timestamptz ORDER BY completed_at ASC LIMIT $3";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_VERSION: &str =
    "INSERT INTO ai_agent_version (id, tenant_id, organization_id, agent_id, version_id, version_number, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, description, created_by, created_at, activated_at) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8::jsonb, $9, $10, $11, $12, $13, $14::timestamptz, $15::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_VERSION: &str =
    "SELECT id, tenant_id, organization_id, agent_id, version_id, version_number, manifest_json::text AS manifest_json, default_code_task_intent_json::text AS default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, description, created_by, created_at::text AS created_at, activated_at::text AS activated_at FROM ai_agent_version WHERE tenant_id = $1 AND organization_id = $2 AND agent_id = $3 AND version_id = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_VERSIONS: &str =
    "SELECT id, tenant_id, organization_id, agent_id, version_id, version_number, manifest_json::text AS manifest_json, default_code_task_intent_json::text AS default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, description, created_by, created_at::text AS created_at, activated_at::text AS activated_at FROM ai_agent_version WHERE tenant_id = $1 AND organization_id = $2 AND agent_id = $3 AND ($4::bigint IS NULL OR version_number < $4::bigint) ORDER BY version_number DESC LIMIT $5";
#[cfg(feature = "postgres-sync")]
pub const SQL_ACTIVATE_AGENT_VERSION: &str =
    "UPDATE ai_agent_version SET activated_at = CASE WHEN version_id = $3 THEN $4::timestamptz ELSE NULL END WHERE tenant_id = $1 AND organization_id = $2 AND agent_id = $5 AND (activated_at IS NOT NULL OR version_id = $3)";
#[cfg(feature = "postgres-sync")]
pub const SQL_SUMMARIZE_AGENT_USAGE: &str =
    "SELECT COUNT(*)::bigint AS turn_count, COUNT(DISTINCT session_id)::bigint AS session_count, COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens, COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, COALESCE(SUM(cached_tokens), 0)::bigint AS cached_tokens FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND ($3::text IS NULL OR agent_id = $3) AND ($4::text IS NULL OR session_id = $4) AND ($5::text IS NULL OR model_id = $5) AND ($6::timestamptz IS NULL OR created_at >= $6::timestamptz) AND ($7::timestamptz IS NULL OR created_at < $7::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_USAGE_RECORDS: &str =
    "SELECT id, turn_id, session_id, agent_id, owner_user_id, status, model_id, provider_id, input_tokens, output_tokens, cached_tokens, created_at::text AS created_at, completed_at::text AS completed_at FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND ($3::text IS NULL OR agent_id = $3) AND ($4::text IS NULL OR session_id = $4) AND ($5::text IS NULL OR model_id = $5) AND ($6::timestamptz IS NULL OR created_at >= $6::timestamptz) AND ($7::timestamptz IS NULL OR created_at < $7::timestamptz) AND ($8::timestamptz IS NULL OR (created_at, id) < ($8::timestamptz, $9::bigint)) ORDER BY created_at DESC, id DESC LIMIT $10";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_WEBHOOK_SUBSCRIPTION: &str =
    "INSERT INTO ai_agent_webhook_subscription (id, tenant_id, organization_id, webhook_id, url, event_types_json, status, secret, description, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11::timestamptz, $12::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_WEBHOOK_SUBSCRIPTION: &str =
    "SELECT id, tenant_id, organization_id, webhook_id, url, event_types_json::text AS event_types_json, status, secret, description, created_by, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_webhook_subscription WHERE tenant_id = $1 AND organization_id = $2 AND webhook_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_WEBHOOK_SUBSCRIPTIONS: &str =
    "SELECT id, tenant_id, organization_id, webhook_id, url, event_types_json::text AS event_types_json, status, secret, description, created_by, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_webhook_subscription WHERE tenant_id = $1 AND organization_id = $2 ORDER BY created_at DESC, webhook_id ASC LIMIT $3 OFFSET $4";
#[cfg(feature = "postgres-sync")]
pub const SQL_DELETE_WEBHOOK_SUBSCRIPTION: &str =
    "DELETE FROM ai_agent_webhook_subscription WHERE tenant_id = $1 AND organization_id = $2 AND webhook_id = $3";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_WEBHOOK_DELIVERY: &str =
    "INSERT INTO ai_agent_webhook_delivery (id, tenant_id, organization_id, webhook_id, delivery_id, event_type, payload_json, signature, status, response_code, error_detail, created_at, completed_at) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8, $9, $10, $11, $12::timestamptz, $13::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_COMPLETE_WEBHOOK_DELIVERY: &str =
    "UPDATE ai_agent_webhook_delivery SET status = $4, response_code = $5, error_detail = $6, completed_at = $7::timestamptz WHERE tenant_id = $1 AND organization_id = $2 AND webhook_id = $3 AND delivery_id = $8";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_WEBHOOK_DELIVERY: &str =
    "SELECT id, tenant_id, organization_id, webhook_id, delivery_id, event_type, payload_json::text AS payload_json, signature, status, response_code, error_detail, created_at::text AS created_at, completed_at::text AS completed_at FROM ai_agent_webhook_delivery WHERE tenant_id = $1 AND organization_id = $2 AND webhook_id = $3 AND delivery_id = $4 LIMIT 1";
pub const SQL_INSERT_AGENT_COMPOSITION_SLOT: &str =
    "INSERT INTO ai_agent_composition_slot (id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, status, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb, $14, $15, $16::timestamptz, $17::timestamptz, $18::timestamptz)";
pub const SQL_UPDATE_AGENT_COMPOSITION_SLOT: &str =
    "UPDATE ai_agent_composition_slot SET organization_id = $1, slot_kind = $2, target_module = $3, target_ref = $4, target_version_ref = $5, priority = $6, enabled = $7, policy_json = $8::jsonb, status = $9, version = $10, updated_at = $11::timestamptz, deleted_at = $12::timestamptz WHERE tenant_id = $13 AND agent_id = $14 AND slot_id = $15 AND version = $16";
pub const SQL_SELECT_AGENT_COMPOSITION_SLOT: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND slot_id = $3 LIMIT 1";
pub const SQL_LIST_AGENT_COMPOSITION_SLOTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND deleted_at IS NULL ORDER BY priority ASC, slot_id ASC LIMIT $3 OFFSET $4";
pub const SQL_COUNT_AGENT_COMPOSITION_SLOTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND deleted_at IS NULL";
pub const SQL_LIST_MCP_MARKETPLACE_SLOTS: &str =
    "SELECT s.id, s.uuid, s.tenant_id, s.organization_id, s.agent_id, s.slot_id, s.slot_kind, s.target_module, s.target_ref, s.target_version_ref, s.priority, s.enabled, s.policy_json::text AS policy_json, s.status, s.version, s.created_at::text AS created_at, s.updated_at::text AS updated_at, s.deleted_at::text AS deleted_at FROM ai_agent_composition_slot s INNER JOIN ai_agent a ON a.tenant_id = s.tenant_id AND a.agent_id = s.agent_id WHERE s.tenant_id = $1 AND s.slot_kind = 'mcp' AND s.deleted_at IS NULL AND a.deleted_at IS NULL AND ($2::text IS NULL OR (s.target_ref ILIKE $2 OR s.slot_id ILIKE $2 OR s.agent_id ILIKE $2)) ORDER BY s.priority ASC, s.slot_id ASC LIMIT $3 OFFSET $4";
pub const SQL_COUNT_MCP_MARKETPLACE_SLOTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_composition_slot s INNER JOIN ai_agent a ON a.tenant_id = s.tenant_id AND a.agent_id = s.agent_id WHERE s.tenant_id = $1 AND s.slot_kind = 'mcp' AND s.deleted_at IS NULL AND a.deleted_at IS NULL AND ($2::text IS NULL OR (s.target_ref ILIKE $2 OR s.slot_id ILIKE $2 OR s.agent_id ILIKE $2))";

// Session SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_SESSION: &str =
    "INSERT INTO ai_agent_session (id, uuid, tenant_id, organization_id, session_id, agent_id, owner_user_id, project_id, session_kind, entry_surface, source_module, source_context_kind, source_context_id, parent_session_id, forked_from_turn_id, title, title_source, status, item_count, last_item_sequence, total_input_tokens, total_output_tokens, idempotency_key, payload_hash, created_by, updated_by, version, created_at, updated_at, last_item_at, closed_at, archived_at, archived_by, deleted_at, deleted_by, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NULLIF(BTRIM($16), ''), $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28::timestamptz, $29::timestamptz, $30::timestamptz, $31::timestamptz, $32::timestamptz, $33, $34::timestamptz, $35, $36::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_SESSION: &str =
    "UPDATE ai_agent_session SET project_id = $1, title = NULLIF(BTRIM($2), ''), title_source = $3, status = $4, item_count = $5, last_item_sequence = GREATEST(last_item_sequence, $6), total_input_tokens = $7, total_output_tokens = $8, updated_by = $9, version = $10, updated_at = $11::timestamptz, last_item_at = $12::timestamptz, closed_at = $13::timestamptz, archived_at = $14::timestamptz, archived_by = $15, deleted_at = $16::timestamptz, deleted_by = $17, retention_until = $18::timestamptz WHERE tenant_id = $19 AND organization_id = $20 AND session_id = $21 AND deleted_at IS NULL AND version = $22";
#[cfg(feature = "postgres-sync")]
pub const SQL_RECORD_AGENT_SESSION_ITEM: &str =
    "UPDATE ai_agent_session SET item_count = item_count + 1, last_item_sequence = last_item_sequence + 1, total_input_tokens = total_input_tokens + $4, total_output_tokens = total_output_tokens + $5, updated_by = $6, version = version + 1, updated_at = $7::timestamptz, last_item_at = $7::timestamptz WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND deleted_at IS NULL AND ($8::bool = FALSE OR status = 0) RETURNING id, uuid, tenant_id, organization_id, session_id, agent_id, owner_user_id, project_id, session_kind, entry_surface, source_module, source_context_kind, source_context_id, parent_session_id, forked_from_turn_id, title, title_source, status, item_count, last_item_sequence, total_input_tokens, total_output_tokens, idempotency_key, payload_hash, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, last_item_at::text AS last_item_at, closed_at::text AS closed_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, agent_id, owner_user_id, project_id, session_kind, entry_surface, source_module, source_context_kind, source_context_id, parent_session_id, forked_from_turn_id, title, title_source, status, item_count, last_item_sequence, total_input_tokens, total_output_tokens, idempotency_key, payload_hash, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, last_item_at::text AS last_item_at, closed_at::text AS closed_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_session WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION_BY_CREATE_IDEMPOTENCY: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, agent_id, owner_user_id, project_id, session_kind, entry_surface, source_module, source_context_kind, source_context_id, parent_session_id, forked_from_turn_id, title, title_source, status, item_count, last_item_sequence, total_input_tokens, total_output_tokens, idempotency_key, payload_hash, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, last_item_at::text AS last_item_at, closed_at::text AS closed_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_session WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND idempotency_key = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSIONS: &str =
    "SELECT s.id, s.uuid, s.tenant_id, s.organization_id, s.session_id, s.agent_id, s.owner_user_id, s.project_id, s.session_kind, s.entry_surface, s.source_module, s.source_context_kind, s.source_context_id, s.parent_session_id, s.forked_from_turn_id, s.title, s.title_source, s.status, s.item_count, s.last_item_sequence, s.total_input_tokens, s.total_output_tokens, s.idempotency_key, s.payload_hash, s.created_by, s.updated_by, s.version, s.created_at::text AS created_at, s.updated_at::text AS updated_at, s.last_item_at::text AS last_item_at, s.closed_at::text AS closed_at, s.archived_at::text AS archived_at, s.archived_by, s.deleted_at::text AS deleted_at, s.deleted_by, s.retention_until::text AS retention_until FROM ai_agent_session s WHERE s.tenant_id = $1 AND s.deleted_at IS NULL AND ($2::bigint IS NULL OR s.organization_id = $2) AND ($3::text IS NULL OR s.agent_id = $3) AND ($4::text IS NULL OR s.project_id = $4) AND ($5::text IS NULL OR EXISTS (SELECT 1 FROM ai_agent_project p WHERE p.tenant_id = s.tenant_id AND p.organization_id = s.organization_id AND p.project_id = s.project_id AND p.workspace_id = $5 AND p.deleted_at IS NULL)) AND ($6::bigint IS NULL OR s.owner_user_id = $6) AND ($7::smallint IS NULL OR s.status = $7) AND ($8::bool = true OR s.status != 3) AND ($9::timestamptz IS NULL OR (s.updated_at, s.id) < ($9::timestamptz, $10::bigint)) ORDER BY s.updated_at DESC, s.id DESC LIMIT $11";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS: &str = r#"WITH page AS (
        SELECT s.*
        FROM ai_agent_session s
        WHERE s.tenant_id = $1
          AND s.organization_id = $2
          AND s.owner_user_id = $3
          AND s.deleted_at IS NULL
          AND ($4::text IS NULL OR s.agent_id = $4)
          AND ($5::text IS NULL OR s.project_id = $5)
          AND ($6::text IS NULL OR EXISTS (
              SELECT 1
              FROM ai_agent_project project_scope
              WHERE project_scope.tenant_id = s.tenant_id
                AND project_scope.organization_id = s.organization_id
                AND project_scope.project_id = s.project_id
                AND project_scope.workspace_id = $6
                AND project_scope.deleted_at IS NULL
          ))
          AND ($7::timestamptz IS NULL
               OR (s.activity_at, s.id) < ($7::timestamptz, $8::bigint))
        ORDER BY s.activity_at DESC, s.id DESC
        LIMIT $9
    ),
    activity AS (
        SELECT page.*,
               CASE
                   WHEN session_user_state.updated_at IS NOT NULL
                        AND session_user_state.updated_at >= page.updated_at
                        AND session_user_state.updated_at >= COALESCE(turn_activity.updated_at, page.updated_at)
                        AND session_user_state.updated_at >= COALESCE(interaction_activity.updated_at, page.updated_at)
                        AND session_user_state.updated_at >= COALESCE(binding_activity.updated_at, page.updated_at)
                       THEN 'user_state'
                   WHEN interaction_activity.updated_at IS NOT NULL
                        AND interaction_activity.updated_at >= page.updated_at
                        AND interaction_activity.updated_at >= COALESCE(turn_activity.updated_at, page.updated_at)
                        AND interaction_activity.updated_at >= COALESCE(binding_activity.updated_at, page.updated_at)
                        AND interaction_activity.updated_at >= COALESCE(session_user_state.updated_at, page.updated_at)
                       THEN 'interaction'
                   WHEN turn_activity.updated_at IS NOT NULL
                        AND turn_activity.updated_at >= page.updated_at
                        AND turn_activity.updated_at >= COALESCE(binding_activity.updated_at, page.updated_at)
                        AND turn_activity.updated_at >= COALESCE(session_user_state.updated_at, page.updated_at)
                       THEN 'turn'
                   WHEN binding_activity.updated_at IS NOT NULL
                        AND binding_activity.updated_at >= page.updated_at
                        AND binding_activity.updated_at >= COALESCE(session_user_state.updated_at, page.updated_at)
                       THEN 'runtime_binding'
                   ELSE 'session'
               END AS activity_source,
               row_to_json(latest_turn)::text AS latest_turn_json,
               row_to_json(pending_interaction)::text AS pending_interaction_json,
               row_to_json(current_runtime_binding)::text AS current_runtime_binding_json,
               row_to_json(binding_activity)::text AS latest_runtime_binding_json,
               row_to_json(session_user_state)::text AS user_state_json,
               interaction_activity.interaction_id AS latest_interaction_id,
               interaction_activity.version AS latest_interaction_version
        FROM page
        LEFT JOIN LATERAL (
            SELECT turn_row.*
            FROM ai_agent_turn turn_row
            WHERE turn_row.tenant_id = page.tenant_id
              AND turn_row.organization_id = page.organization_id
              AND turn_row.session_id = page.session_id
            ORDER BY turn_row.id DESC
            LIMIT 1
        ) latest_turn ON TRUE
        LEFT JOIN LATERAL (
            SELECT turn_row.updated_at
            FROM ai_agent_turn turn_row
            WHERE turn_row.tenant_id = page.tenant_id
              AND turn_row.organization_id = page.organization_id
              AND turn_row.session_id = page.session_id
            ORDER BY turn_row.updated_at DESC, turn_row.id DESC
            LIMIT 1
        ) turn_activity ON TRUE
        LEFT JOIN LATERAL (
            SELECT interaction_row.interaction_id, interaction_row.updated_at, interaction_row.version
            FROM ai_agent_interaction interaction_row
            WHERE interaction_row.tenant_id = page.tenant_id
              AND interaction_row.organization_id = page.organization_id
              AND interaction_row.session_id = page.session_id
            ORDER BY interaction_row.updated_at DESC, interaction_row.id DESC
            LIMIT 1
        ) interaction_activity ON TRUE
        LEFT JOIN LATERAL (
            SELECT interaction_row.*
            FROM ai_agent_interaction interaction_row
            WHERE interaction_row.tenant_id = page.tenant_id
              AND interaction_row.organization_id = page.organization_id
              AND interaction_row.session_id = page.session_id
              AND interaction_row.status = 0
            ORDER BY interaction_row.kind ASC, interaction_row.updated_at DESC, interaction_row.id DESC
            LIMIT 1
        ) pending_interaction ON TRUE
        LEFT JOIN LATERAL (
            SELECT binding_row.*
            FROM ai_agent_session_runtime_binding binding_row
            WHERE binding_row.tenant_id = page.tenant_id
              AND binding_row.organization_id = page.organization_id
              AND binding_row.session_id = page.session_id
            ORDER BY binding_row.updated_at DESC, binding_row.id DESC
            LIMIT 1
        ) binding_activity ON TRUE
        LEFT JOIN LATERAL (
            SELECT binding_row.*
            FROM ai_agent_session_runtime_binding binding_row
            WHERE binding_row.tenant_id = page.tenant_id
              AND binding_row.organization_id = page.organization_id
              AND binding_row.session_id = page.session_id
              AND binding_row.is_current = TRUE
              AND binding_row.status = 0
            ORDER BY binding_row.updated_at DESC, binding_row.id DESC
            LIMIT 1
        ) current_runtime_binding ON TRUE
        LEFT JOIN LATERAL (
            SELECT user_state_row.*
            FROM ai_agent_resource_user_state user_state_row
            WHERE user_state_row.tenant_id = page.tenant_id
              AND user_state_row.organization_id = page.organization_id
              AND user_state_row.user_id = page.owner_user_id
              AND user_state_row.resource_type = 0
              AND user_state_row.resource_id = page.session_id
            ORDER BY user_state_row.updated_at DESC, user_state_row.id DESC
            LIMIT 1
        ) session_user_state ON TRUE
    )
    SELECT id, uuid, tenant_id, organization_id, session_id, agent_id, owner_user_id,
           project_id, session_kind, entry_surface, source_module, source_context_kind,
           source_context_id, parent_session_id, forked_from_turn_id, title, title_source, status,
           item_count, last_item_sequence, total_input_tokens, total_output_tokens,
           idempotency_key, payload_hash, created_by, updated_by, version,
           created_at::text AS created_at, updated_at::text AS updated_at,
           last_item_at::text AS last_item_at, closed_at::text AS closed_at,
           archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at,
           deleted_by, retention_until::text AS retention_until,
           activity_at, activity_source, latest_turn_json,
           pending_interaction_json, current_runtime_binding_json,
           latest_runtime_binding_json, user_state_json,
           latest_interaction_id, latest_interaction_version
    FROM activity
    ORDER BY activity_at DESC, id DESC"#;
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_SESSIONS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_session s WHERE s.tenant_id = $1 AND s.deleted_at IS NULL AND ($2::bigint IS NULL OR s.organization_id = $2) AND ($3::text IS NULL OR s.agent_id = $3) AND ($4::text IS NULL OR s.project_id = $4) AND ($5::text IS NULL OR EXISTS (SELECT 1 FROM ai_agent_project p WHERE p.tenant_id = s.tenant_id AND p.organization_id = s.organization_id AND p.project_id = s.project_id AND p.workspace_id = $5 AND p.deleted_at IS NULL)) AND ($6::bigint IS NULL OR s.owner_user_id = $6) AND ($7::smallint IS NULL OR s.status = $7) AND ($8::bool = true OR s.status != 3)";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_SESSION_RUNTIME_BINDING: &str =
    "INSERT INTO ai_agent_session_runtime_binding (id, uuid, tenant_id, organization_id, owner_user_id, session_id, runtime_binding_id, runtime_location_id, host_mode, transport_kind, provider_binding_id, model_id, provider_id, provider_session_id, provider_session_tree_id, provider_parent_session_id, provider_forked_from_session_id, provider_title, provider_title_source, provider_preview, provider_created_at, provider_updated_at, provider_recency_at, provider_pinned, provider_archived, provider_visible, provider_sort_key, provider_source, status, is_current, version, created_at, updated_at, activated_at, deactivated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21::timestamptz, $22::timestamptz, $23::timestamptz, $24, $25, $26, $27, $28, $29, $30, $31, $32::timestamptz, $33::timestamptz, $34::timestamptz, $35::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_SESSION_RUNTIME_BINDING: &str =
    "UPDATE ai_agent_session_runtime_binding SET runtime_location_id = $1, host_mode = $2, transport_kind = $3, provider_binding_id = $4, model_id = $5, provider_id = $6, provider_session_id = $7, provider_session_tree_id = $8, provider_parent_session_id = $9, provider_forked_from_session_id = $10, provider_title = $11, provider_title_source = $12, provider_preview = $13, provider_created_at = $14::timestamptz, provider_updated_at = $15::timestamptz, provider_recency_at = $16::timestamptz, provider_pinned = $17, provider_archived = $18, provider_visible = $19, provider_sort_key = $20, provider_source = $21, status = $22, is_current = $23, version = $24, updated_at = $25::timestamptz, activated_at = $26::timestamptz, deactivated_at = $27::timestamptz WHERE tenant_id = $28 AND organization_id = $29 AND session_id = $30 AND runtime_binding_id = $31 AND version = $32";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION_RUNTIME_BINDING: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, session_id, runtime_binding_id, runtime_location_id, host_mode, transport_kind, provider_binding_id, model_id, provider_id, provider_session_id, provider_session_tree_id, provider_parent_session_id, provider_forked_from_session_id, provider_title, provider_title_source, provider_preview, provider_created_at, provider_updated_at, provider_recency_at, provider_pinned, provider_archived, provider_visible, provider_sort_key, provider_source, status, is_current, version, created_at::text AS created_at, updated_at::text AS updated_at, activated_at::text AS activated_at, deactivated_at::text AS deactivated_at FROM ai_agent_session_runtime_binding WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND runtime_binding_id = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_CURRENT_AGENT_SESSION_RUNTIME_BINDING: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, session_id, runtime_binding_id, runtime_location_id, host_mode, transport_kind, provider_binding_id, model_id, provider_id, provider_session_id, provider_session_tree_id, provider_parent_session_id, provider_forked_from_session_id, provider_title, provider_title_source, provider_preview, provider_created_at, provider_updated_at, provider_recency_at, provider_pinned, provider_archived, provider_visible, provider_sort_key, provider_source, status, is_current, version, created_at::text AS created_at, updated_at::text AS updated_at, activated_at::text AS activated_at, deactivated_at::text AS deactivated_at FROM ai_agent_session_runtime_binding WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND is_current = TRUE AND status = 0 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION_RUNTIME_BINDING_BY_PROVIDER_SESSION: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, session_id, runtime_binding_id, runtime_location_id, host_mode, transport_kind, provider_binding_id, model_id, provider_id, provider_session_id, provider_session_tree_id, provider_parent_session_id, provider_forked_from_session_id, provider_title, provider_title_source, provider_preview, provider_created_at, provider_updated_at, provider_recency_at, provider_pinned, provider_archived, provider_visible, provider_sort_key, provider_source, status, is_current, version, created_at::text AS created_at, updated_at::text AS updated_at, activated_at::text AS activated_at, deactivated_at::text AS deactivated_at FROM ai_agent_session_runtime_binding WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND provider_binding_id = $4 AND provider_session_id = $5 ORDER BY is_current DESC, updated_at DESC, id DESC LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_RUNTIME_BINDINGS: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, session_id, runtime_binding_id, runtime_location_id, host_mode, transport_kind, provider_binding_id, model_id, provider_id, provider_session_id, provider_session_tree_id, provider_parent_session_id, provider_forked_from_session_id, provider_title, provider_title_source, provider_preview, provider_created_at, provider_updated_at, provider_recency_at, provider_pinned, provider_archived, provider_visible, provider_sort_key, provider_source, status, is_current, version, created_at::text AS created_at, updated_at::text AS updated_at, activated_at::text AS activated_at, deactivated_at::text AS deactivated_at FROM ai_agent_session_runtime_binding WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4) AND ($5::bool = FALSE OR is_current = TRUE) ORDER BY is_current DESC, updated_at DESC, id DESC LIMIT $6 OFFSET $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_SESSION_RUNTIME_BINDINGS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_session_runtime_binding WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4) AND ($5::bool = FALSE OR is_current = TRUE)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LOCK_AGENT_SESSION_RUNTIME_BINDING: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, session_id, runtime_binding_id, runtime_location_id, host_mode, transport_kind, provider_binding_id, model_id, provider_id, provider_session_id, provider_session_tree_id, provider_parent_session_id, provider_forked_from_session_id, provider_title, provider_title_source, provider_preview, provider_created_at, provider_updated_at, provider_recency_at, provider_pinned, provider_archived, provider_visible, provider_sort_key, provider_source, status, is_current, version, created_at::text AS created_at, updated_at::text AS updated_at, activated_at::text AS activated_at, deactivated_at::text AS deactivated_at FROM ai_agent_session_runtime_binding WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND runtime_binding_id = $4 FOR UPDATE";
#[cfg(feature = "postgres-sync")]
pub const SQL_DEACTIVATE_CURRENT_AGENT_SESSION_RUNTIME_BINDINGS: &str =
    "UPDATE ai_agent_session_runtime_binding SET status = 1, is_current = FALSE, version = version + 1, updated_at = $5::timestamptz, deactivated_at = $5::timestamptz WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND runtime_binding_id <> $4 AND is_current = TRUE";
#[cfg(feature = "postgres-sync")]
pub const SQL_ACTIVATE_AGENT_SESSION_RUNTIME_BINDING: &str =
    "UPDATE ai_agent_session_runtime_binding SET status = 0, is_current = TRUE, version = version + 1, updated_at = $6::timestamptz, activated_at = $6::timestamptz, deactivated_at = NULL WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND runtime_binding_id = $4 AND version = $5";

#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_SESSION_CHECKPOINT: &str =
    "INSERT INTO ai_agent_session_checkpoint (id, uuid, tenant_id, organization_id, session_id, checkpoint_id, turn_id, runtime_binding_id, checkpoint_kind, provider_checkpoint_ref, drive_space_id, drive_node_id, resumable, status, created_by, version, created_at, updated_at, restored_at, invalidated_at, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17::timestamptz, $18::timestamptz, $19::timestamptz, $20::timestamptz, $21::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_SESSION_CHECKPOINT: &str =
    "UPDATE ai_agent_session_checkpoint SET resumable = $1, status = $2, version = $3, updated_at = $4::timestamptz, restored_at = $5::timestamptz, invalidated_at = $6::timestamptz, retention_until = $7::timestamptz WHERE tenant_id = $8 AND organization_id = $9 AND session_id = $10 AND checkpoint_id = $11 AND version = $12";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION_CHECKPOINT: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, checkpoint_id, turn_id, runtime_binding_id, checkpoint_kind, provider_checkpoint_ref, drive_space_id, drive_node_id, resumable, status, created_by, version, created_at::text AS created_at, updated_at::text AS updated_at, restored_at::text AS restored_at, invalidated_at::text AS invalidated_at, retention_until::text AS retention_until FROM ai_agent_session_checkpoint WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND checkpoint_id = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_CHECKPOINTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, checkpoint_id, turn_id, runtime_binding_id, checkpoint_kind, provider_checkpoint_ref, drive_space_id, drive_node_id, resumable, status, created_by, version, created_at::text AS created_at, updated_at::text AS updated_at, restored_at::text AS restored_at, invalidated_at::text AS invalidated_at, retention_until::text AS retention_until FROM ai_agent_session_checkpoint WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4) ORDER BY created_at DESC, id DESC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_SESSION_CHECKPOINTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_session_checkpoint WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPSERT_AGENT_RESOURCE_USER_STATE: &str =
    "INSERT INTO ai_agent_resource_user_state (id, uuid, tenant_id, organization_id, user_id, resource_type, resource_id, pinned_at, hidden_at, last_opened_at, last_read_item_sequence, custom_title, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $9::timestamptz, $10::timestamptz, $11, $12, 0, $13::timestamptz, $14::timestamptz) ON CONFLICT (tenant_id, organization_id, user_id, resource_type, resource_id) DO UPDATE SET pinned_at = EXCLUDED.pinned_at, hidden_at = EXCLUDED.hidden_at, last_opened_at = EXCLUDED.last_opened_at, last_read_item_sequence = EXCLUDED.last_read_item_sequence, custom_title = EXCLUDED.custom_title, version = ai_agent_resource_user_state.version + 1, updated_at = EXCLUDED.updated_at WHERE ai_agent_resource_user_state.version = $15 RETURNING id, uuid, tenant_id, organization_id, user_id, resource_type, resource_id, pinned_at::text AS pinned_at, hidden_at::text AS hidden_at, last_opened_at::text AS last_opened_at, last_read_item_sequence, custom_title, version, created_at::text AS created_at, updated_at::text AS updated_at";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_RESOURCE_USER_STATE: &str =
    "SELECT id, uuid, tenant_id, organization_id, user_id, resource_type, resource_id, pinned_at::text AS pinned_at, hidden_at::text AS hidden_at, last_opened_at::text AS last_opened_at, last_read_item_sequence, custom_title, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_resource_user_state WHERE tenant_id = $1 AND organization_id = $2 AND user_id = $3 AND resource_type = $4 AND resource_id = $5 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_RESOURCE_USER_STATES: &str =
    "SELECT state.id, state.uuid, state.tenant_id, state.organization_id, state.user_id, state.resource_type, state.resource_id, state.pinned_at::text AS pinned_at, state.hidden_at::text AS hidden_at, state.last_opened_at::text AS last_opened_at, state.last_read_item_sequence, state.custom_title, state.version, state.created_at::text AS created_at, state.updated_at::text AS updated_at FROM ai_agent_resource_user_state AS state WHERE state.tenant_id = $1 AND state.organization_id = $2 AND state.user_id = $3 AND state.resource_type = $4 AND ($5::text IS NULL OR EXISTS (SELECT 1 FROM ai_agent_session AS session WHERE state.resource_type = 0 AND session.tenant_id = state.tenant_id AND session.organization_id = state.organization_id AND session.owner_user_id = state.user_id AND session.session_id = state.resource_id AND session.agent_id = $5 AND session.deleted_at IS NULL)) AND ($6::bool = false OR state.pinned_at IS NOT NULL) AND ($7::bool = true OR state.hidden_at IS NULL) AND (cardinality($8::text[]) = 0 OR state.resource_id = ANY($8::text[])) ORDER BY state.pinned_at DESC NULLS LAST, state.last_opened_at DESC NULLS LAST, state.id DESC LIMIT $9 OFFSET $10";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_RESOURCE_USER_STATES: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_resource_user_state AS state WHERE state.tenant_id = $1 AND state.organization_id = $2 AND state.user_id = $3 AND state.resource_type = $4 AND ($5::text IS NULL OR EXISTS (SELECT 1 FROM ai_agent_session AS session WHERE state.resource_type = 0 AND session.tenant_id = state.tenant_id AND session.organization_id = state.organization_id AND session.owner_user_id = state.user_id AND session.session_id = state.resource_id AND session.agent_id = $5 AND session.deleted_at IS NULL)) AND ($6::bool = false OR state.pinned_at IS NOT NULL) AND ($7::bool = true OR state.hidden_at IS NULL) AND (cardinality($8::text[]) = 0 OR state.resource_id = ANY($8::text[]))";

#[cfg(feature = "postgres-sync")]
pub const SQL_UPSERT_AGENT_ITEM_FEEDBACK: &str =
    "INSERT INTO ai_agent_item_feedback (id, uuid, tenant_id, organization_id, item_id, user_id, rating, reason_code, comment, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, $10::timestamptz, $11::timestamptz, $12::timestamptz) ON CONFLICT (tenant_id, organization_id, item_id, user_id) DO UPDATE SET rating = EXCLUDED.rating, reason_code = EXCLUDED.reason_code, comment = EXCLUDED.comment, version = ai_agent_item_feedback.version + 1, updated_at = EXCLUDED.updated_at, deleted_at = EXCLUDED.deleted_at WHERE ai_agent_item_feedback.version = $13 OR (ai_agent_item_feedback.deleted_at IS NOT NULL AND $13 = -1 AND EXCLUDED.deleted_at IS NULL) RETURNING id, uuid, tenant_id, organization_id, item_id, user_id, rating, reason_code, comment, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_ITEM_FEEDBACK: &str =
    "SELECT id, uuid, tenant_id, organization_id, item_id, user_id, rating, reason_code, comment, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_item_feedback WHERE tenant_id = $1 AND organization_id = $2 AND item_id = $3 AND user_id = $4 AND ($5::bool = TRUE OR deleted_at IS NULL) LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_ITEM_FEEDBACK: &str =
    "SELECT feedback.id, feedback.uuid, feedback.tenant_id, feedback.organization_id, feedback.item_id, feedback.user_id, feedback.rating, feedback.reason_code, feedback.comment, feedback.version, feedback.created_at::text AS created_at, feedback.updated_at::text AS updated_at, feedback.deleted_at::text AS deleted_at FROM ai_agent_item_feedback AS feedback INNER JOIN ai_agent_session_item AS item ON item.tenant_id = feedback.tenant_id AND item.organization_id = feedback.organization_id AND item.item_id = feedback.item_id WHERE feedback.tenant_id = $1 AND feedback.organization_id = $2 AND feedback.user_id = $3 AND item.session_id = $4 AND feedback.deleted_at IS NULL ORDER BY item.sequence ASC, feedback.id ASC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_ITEM_FEEDBACK: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_item_feedback AS feedback INNER JOIN ai_agent_session_item AS item ON item.tenant_id = feedback.tenant_id AND item.organization_id = feedback.organization_id AND item.item_id = feedback.item_id WHERE feedback.tenant_id = $1 AND feedback.organization_id = $2 AND feedback.user_id = $3 AND item.session_id = $4 AND feedback.deleted_at IS NULL";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_ITEM_DRIVE_REF: &str =
    "INSERT INTO ai_agent_item_drive_ref (id, uuid, tenant_id, organization_id, item_id, resource_role, drive_space_id, drive_node_id, media_resource_id, object_blob_id, resource_hash, alt_text, sort_order, status, created_by, created_at, updated_at, deleted_at, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16::timestamptz, $17::timestamptz, $18::timestamptz, $19::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_ITEM_DRIVE_REFS: &str =
    "SELECT id, uuid, tenant_id, organization_id, item_id, resource_role, drive_space_id, drive_node_id, media_resource_id, object_blob_id, resource_hash, alt_text, sort_order, status, created_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, retention_until::text AS retention_until FROM ai_agent_item_drive_ref WHERE tenant_id = $1 AND organization_id = $2 AND item_id = $3 AND status = 0 AND deleted_at IS NULL ORDER BY sort_order ASC, id ASC LIMIT 200";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_ITEM_DRIVE_REFS_BATCH: &str =
    "SELECT id, uuid, tenant_id, organization_id, item_id, resource_role, drive_space_id, drive_node_id, media_resource_id, object_blob_id, resource_hash, alt_text, sort_order, status, created_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, retention_until::text AS retention_until FROM ai_agent_item_drive_ref WHERE tenant_id = $1 AND organization_id = $2 AND item_id = ANY($3::text[]) AND status = 0 AND deleted_at IS NULL ORDER BY item_id ASC, sort_order ASC, id ASC";

// Turn and session-item SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_TURN: &str =
    "INSERT INTO ai_agent_turn (id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, runtime_binding_id, client_request_id, idempotency_key, payload_hash, request_item_id, response_item_id, turn_mode, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, cached_tokens, finish_reason, error_code, error_detail, trace_id, attempt_count, max_attempts, next_retry_at, available_at, lease_owner, lease_token, lease_expires_at, fencing_token, version, created_at, updated_at, started_at, completed_at, cancel_requested_at, cancelled_at, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30::timestamptz, $31::timestamptz, $32, $33, $34::timestamptz, $35, $36, $37::timestamptz, $38::timestamptz, $39::timestamptz, $40::timestamptz, $41::timestamptz, $42::timestamptz, $43::timestamptz) ON CONFLICT (tenant_id, organization_id, owner_user_id, idempotency_key) DO NOTHING";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, runtime_binding_id, client_request_id, idempotency_key, payload_hash, request_item_id, response_item_id, turn_mode, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, cached_tokens, finish_reason, error_code, error_detail, trace_id, attempt_count, max_attempts, next_retry_at::text AS next_retry_at, available_at::text AS available_at, lease_owner, lease_token, lease_expires_at::text AS lease_expires_at, fencing_token, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND idempotency_key = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_TURN: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, runtime_binding_id, client_request_id, idempotency_key, payload_hash, request_item_id, response_item_id, turn_mode, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, cached_tokens, finish_reason, error_code, error_detail, trace_id, attempt_count, max_attempts, next_retry_at::text AS next_retry_at, available_at::text AS available_at, lease_owner, lease_token, lease_expires_at::text AS lease_expires_at, fencing_token, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND turn_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_TURNS: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, runtime_binding_id, client_request_id, idempotency_key, payload_hash, request_item_id, response_item_id, turn_mode, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, cached_tokens, finish_reason, error_code, error_detail, trace_id, attempt_count, max_attempts, next_retry_at::text AS next_retry_at, available_at::text AS available_at, lease_owner, lease_token, lease_expires_at::text AS lease_expires_at, fencing_token, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4) AND ($5::timestamptz IS NULL OR (created_at, id) < ($5::timestamptz, $6::bigint)) ORDER BY created_at DESC, id DESC LIMIT $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_TURNS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4)";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_TURN_INPUT_QUEUE_ENTRY: &str =
    "SELECT id, tenant_id, organization_id, queue_entry_id, session_id, agent_id, owner_user_id, content, display_text, content_type, attachment_names_json::text AS attachment_names_json, drive_refs_json::text AS drive_refs_json, turn_mode, runtime_binding_id, requested_model_id, access_mode_id, idempotency_key, payload_hash, client_request_id, position, status, claim_owner, claim_token_hash, claim_expires_at::text AS claim_expires_at, fencing_token, error_code, error_detail, version, created_at::text AS created_at, updated_at::text AS updated_at, claimed_at::text AS claimed_at, failed_at::text AS failed_at FROM ai_agent_turn_input_queue_entry WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND owner_user_id = $4 AND queue_entry_id = $5 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_TURN_INPUT_QUEUE_ENTRIES: &str =
    "SELECT id, tenant_id, organization_id, queue_entry_id, session_id, agent_id, owner_user_id, content, display_text, content_type, attachment_names_json::text AS attachment_names_json, drive_refs_json::text AS drive_refs_json, turn_mode, runtime_binding_id, requested_model_id, access_mode_id, idempotency_key, payload_hash, client_request_id, position, status, claim_owner, claim_token_hash, claim_expires_at::text AS claim_expires_at, fencing_token, error_code, error_detail, version, created_at::text AS created_at, updated_at::text AS updated_at, claimed_at::text AS claimed_at, failed_at::text AS failed_at FROM ai_agent_turn_input_queue_entry WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND owner_user_id = $4 ORDER BY position ASC, id ASC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_TURN_INPUT_QUEUE_ENTRIES: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_turn_input_queue_entry WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND owner_user_id = $4";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_TURN_INPUT_QUEUE_ENTRY: &str =
    "UPDATE ai_agent_turn_input_queue_entry SET content = $1, display_text = $2, content_type = $3, attachment_names_json = $4::jsonb, drive_refs_json = $5::jsonb, turn_mode = $6, runtime_binding_id = $7, requested_model_id = $8, access_mode_id = $9, idempotency_key = $10, payload_hash = $11, client_request_id = $12, status = $13, claim_owner = NULL, claim_token_hash = NULL, claim_expires_at = NULL, error_code = $14, error_detail = $15, version = $16, updated_at = $17::timestamptz, failed_at = $18::timestamptz WHERE tenant_id = $19 AND organization_id = $20 AND session_id = $21 AND owner_user_id = $22 AND queue_entry_id = $23 AND version = $24 AND status <> 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_RECONCILABLE_AGENT_TURNS: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, runtime_binding_id, client_request_id, idempotency_key, payload_hash, request_item_id, response_item_id, turn_mode, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, cached_tokens, finish_reason, error_code, error_detail, trace_id, attempt_count, max_attempts, next_retry_at::text AS next_retry_at, available_at::text AS available_at, lease_owner, lease_token, lease_expires_at::text AS lease_expires_at, fencing_token, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until, streaming_content FROM ai_agent_turn WHERE status IN (0, 1) AND updated_at < $1::timestamptz AND (lease_expires_at IS NULL OR lease_expires_at < $1::timestamptz) ORDER BY updated_at ASC, id ASC LIMIT $2";
#[cfg(feature = "postgres-sync")]
pub const SQL_APPEND_TURN_STREAMING_CONTENT: &str =
    "UPDATE ai_agent_turn SET streaming_content = $4, updated_at = $5::timestamptz WHERE tenant_id = $1 AND organization_id = $2 AND turn_id = $3 AND status IN (0, 1)";
#[cfg(feature = "postgres-sync")]
pub const SQL_CLEAR_TURN_STREAMING_CONTENT: &str =
    "UPDATE ai_agent_turn SET streaming_content = NULL WHERE tenant_id = $1 AND organization_id = $2 AND turn_id = $3";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_TURN_STREAMING_CONTENT: &str =
    "SELECT streaming_content FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND turn_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_TURN_FOR_UPDATE: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, runtime_binding_id, client_request_id, idempotency_key, payload_hash, request_item_id, response_item_id, turn_mode, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, cached_tokens, finish_reason, error_code, error_detail, trace_id, attempt_count, max_attempts, next_retry_at::text AS next_retry_at, available_at::text AS available_at, lease_owner, lease_token, lease_expires_at::text AS lease_expires_at, fencing_token, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until, streaming_content FROM ai_agent_turn WHERE tenant_id = $1 AND organization_id = $2 AND turn_id = $3 LIMIT 1 FOR UPDATE";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION_ITEM_ID_BY_TURN_AND_KIND: &str =
    "SELECT item_id FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND turn_id = $4 AND kind = $5 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_TURN_STATE: &str =
    "UPDATE ai_agent_turn SET response_item_id = $1, runtime_binding_id = $2, turn_mode = $3, status = $4, requested_model_id = $5, provider_binding_id = $6, model_id = $7, provider_id = $8, input_tokens = $9, output_tokens = $10, cached_tokens = $11, finish_reason = $12, error_code = $13, error_detail = $14, trace_id = $15, attempt_count = $16, max_attempts = $17, next_retry_at = $18::timestamptz, available_at = $19::timestamptz, lease_owner = $20, lease_token = $21, lease_expires_at = $22::timestamptz, fencing_token = $23, version = $24, updated_at = $25::timestamptz, started_at = $26::timestamptz, completed_at = $27::timestamptz, cancel_requested_at = $28::timestamptz, cancelled_at = $29::timestamptz, retention_until = $30::timestamptz WHERE tenant_id = $31 AND organization_id = $32 AND turn_id = $33 AND version = $34";
#[cfg(feature = "postgres-sync")]
pub const SQL_COMPLETE_AGENT_TURN_STATE: &str =
    "UPDATE ai_agent_turn SET response_item_id = $1, runtime_binding_id = $2, turn_mode = $3, status = $4, requested_model_id = $5, provider_binding_id = $6, model_id = $7, provider_id = $8, input_tokens = $9, output_tokens = $10, cached_tokens = $11, finish_reason = $12, error_code = $13, error_detail = $14, trace_id = $15, attempt_count = $16, max_attempts = $17, next_retry_at = $18::timestamptz, available_at = $19::timestamptz, lease_owner = $20, lease_token = $21, lease_expires_at = $22::timestamptz, fencing_token = $23, version = $24, updated_at = $25::timestamptz, started_at = $26::timestamptz, completed_at = $27::timestamptz, cancel_requested_at = $28::timestamptz, cancelled_at = $29::timestamptz, retention_until = $30::timestamptz WHERE tenant_id = $31 AND organization_id = $32 AND turn_id = $33 AND version = $34 AND status = 1 AND response_item_id IS NULL AND fencing_token = $35 AND lease_token IS NOT DISTINCT FROM $36";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_SESSION_ITEM: &str =
    "INSERT INTO ai_agent_session_item (id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json, tool_result_json, provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18::jsonb, $19::jsonb, $20::jsonb, $21, $22, $23, $24, $25::timestamptz, $26::timestamptz, $27::timestamptz, $28::timestamptz, $29, $30::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_SESSION_ITEM: &str =
    "UPDATE ai_agent_session_item SET content = $1, content_type = $2, status = $3, model_id = $4, provider_id = $5, tool_name = $6, tool_call_id = $7, tool_arguments_json = $8::jsonb, tool_result_json = $9::jsonb, provider_payload_json = $10::jsonb, parent_item_id = $11, turn_id = $12, version = $13, updated_at = $14::timestamptz, completed_at = $15::timestamptz, redacted_at = $16::timestamptz, redacted_by = $17, retention_until = $18::timestamptz WHERE tenant_id = $19 AND organization_id = $20 AND session_id = $21 AND item_id = $22 AND version = $23";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION_ITEM: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json::text AS tool_arguments_json, tool_result_json::text AS tool_result_json, provider_payload_json::text AS provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND item_id = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_ITEMS: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json::text AS tool_arguments_json, tool_result_json::text AS tool_result_json, provider_payload_json::text AS provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR kind = $4) AND ($5::smallint IS NULL OR status = $5) ORDER BY sequence ASC, id ASC LIMIT $6 OFFSET $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_ITEMS_DESC: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json::text AS tool_arguments_json, tool_result_json::text AS tool_result_json, provider_payload_json::text AS provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR kind = $4) AND ($5::smallint IS NULL OR status = $5) ORDER BY sequence DESC, id DESC LIMIT $6 OFFSET $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_ITEMS_CURSOR_ASC: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json::text AS tool_arguments_json, tool_result_json::text AS tool_result_json, provider_payload_json::text AS provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR kind = $4) AND ($5::smallint IS NULL OR status = $5) AND ($6::bigint IS NULL OR (sequence, id) > ($6::bigint, $7::bigint)) ORDER BY sequence ASC, id ASC LIMIT $8";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_ITEMS_CURSOR_DESC: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json::text AS tool_arguments_json, tool_result_json::text AS tool_result_json, provider_payload_json::text AS provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR kind = $4) AND ($5::smallint IS NULL OR status = $5) AND ($6::bigint IS NULL OR (sequence, id) < ($6::bigint, $7::bigint)) ORDER BY sequence DESC, id DESC LIMIT $8";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json::text AS tool_arguments_json, tool_result_json::text AS tool_result_json, provider_payload_json::text AS provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR kind = $4) AND ($5::smallint IS NULL OR status = $5) ORDER BY sequence DESC, id DESC LIMIT $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSION_ITEMS_BY_TURN: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, item_id, kind, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, tool_name, tool_call_id, tool_arguments_json::text AS tool_arguments_json, tool_result_json::text AS tool_result_json, provider_payload_json::text AS provider_payload_json, parent_item_id, turn_id, created_by, version, created_at, updated_at, completed_at, redacted_at, redacted_by, retention_until FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND turn_id = $4 ORDER BY sequence ASC, id ASC LIMIT $5";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_SESSION_ITEMS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_session_item WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR kind = $4) AND ($5::smallint IS NULL OR status = $5)";
// Interaction SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_INTERACTION: &str =
    "INSERT INTO ai_agent_interaction (id, uuid, tenant_id, organization_id, session_id, turn_id, runtime_binding_id, interaction_id, provider_interaction_id, kind, status, prompt, options_json, request_json, resolution_json, claim_owner, claim_token_hash, claim_expires_at, fencing_token, version, created_at, updated_at, resolved_at, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb, $14::jsonb, $15::jsonb, $16, $17, $18::timestamptz, $19, $20, $21::timestamptz, $22::timestamptz, $23::timestamptz, $24::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_INTERACTION: &str =
    "UPDATE ai_agent_interaction SET kind = $1, status = $2, prompt = $3, options_json = $4::jsonb, request_json = $5::jsonb, resolution_json = $6::jsonb, claim_owner = $7, claim_token_hash = $8, claim_expires_at = $9::timestamptz, fencing_token = $10, version = $11, updated_at = $12::timestamptz, resolved_at = $13::timestamptz, retention_until = $14::timestamptz WHERE tenant_id = $15 AND organization_id = $16 AND session_id = $17 AND interaction_id = $18 AND version = $19";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_INTERACTION: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, turn_id, runtime_binding_id, interaction_id, provider_interaction_id, kind, status, prompt, options_json::text AS options_json, request_json::text AS request_json, resolution_json::text AS resolution_json, claim_owner, claim_token_hash, claim_expires_at::text AS claim_expires_at, fencing_token, version, created_at::text AS created_at, updated_at::text AS updated_at, resolved_at::text AS resolved_at, retention_until::text AS retention_until FROM ai_agent_interaction WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND interaction_id = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_INTERACTIONS: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, turn_id, runtime_binding_id, interaction_id, provider_interaction_id, kind, status, prompt, options_json::text AS options_json, request_json::text AS request_json, resolution_json::text AS resolution_json, claim_owner, claim_token_hash, claim_expires_at::text AS claim_expires_at, fencing_token, version, created_at::text AS created_at, updated_at::text AS updated_at, resolved_at::text AS resolved_at, retention_until::text AS retention_until FROM ai_agent_interaction WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4) AND ($5::smallint IS NULL OR kind = $5) AND ($6::timestamptz IS NULL OR (created_at, id) < ($6::timestamptz, $7::bigint)) ORDER BY created_at DESC, id DESC LIMIT $8";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_INTERACTIONS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_interaction WHERE tenant_id = $1 AND organization_id = $2 AND session_id = $3 AND ($4::smallint IS NULL OR status = $4) AND ($5::smallint IS NULL OR kind = $5)";

// Task SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_TASK: &str =
    "INSERT INTO ai_agent_task (id, uuid, tenant_id, organization_id, agent_id, task_id, owner_user_id, session_id, title, prompt, schedule_kind, cron_expression, timezone, scheduled_at, starts_at, ends_at, next_fire_at, misfire_policy, overlap_policy, max_concurrent_runs, max_catch_up_runs, max_attempts, retry_initial_delay_seconds, retry_max_delay_seconds, timeout_seconds, priority, status, generation, external_ref, metadata_json, version, created_at, updated_at, completed_at, paused_at, cancelled_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14::timestamptz, $15::timestamptz, $16::timestamptz, $17::timestamptz, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30::jsonb, $31, $32::timestamptz, $33::timestamptz, $34::timestamptz, $35::timestamptz, $36::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_TASK: &str =
    "UPDATE ai_agent_task SET title = $1, prompt = $2, schedule_kind = $3, cron_expression = $4, timezone = $5, scheduled_at = $6::timestamptz, starts_at = $7::timestamptz, ends_at = $8::timestamptz, next_fire_at = $9::timestamptz, misfire_policy = $10, overlap_policy = $11, max_concurrent_runs = $12, max_catch_up_runs = $13, max_attempts = $14, retry_initial_delay_seconds = $15, retry_max_delay_seconds = $16, timeout_seconds = $17, priority = $18, status = $19, generation = $20, external_ref = $21, metadata_json = $22::jsonb, version = $23, updated_at = $24::timestamptz, completed_at = $25::timestamptz, paused_at = $26::timestamptz, cancelled_at = $27::timestamptz WHERE tenant_id = $28 AND organization_id = $29 AND task_id = $30 AND version = $31";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_TASK: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, task_id, owner_user_id, session_id, title, prompt, schedule_kind, cron_expression, timezone, scheduled_at::text AS scheduled_at, starts_at::text AS starts_at, ends_at::text AS ends_at, next_fire_at::text AS next_fire_at, misfire_policy, overlap_policy, max_concurrent_runs, max_catch_up_runs, max_attempts, retry_initial_delay_seconds, retry_max_delay_seconds, timeout_seconds, priority, status, generation, external_ref, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, completed_at::text AS completed_at, paused_at::text AS paused_at, cancelled_at::text AS cancelled_at FROM ai_agent_task WHERE tenant_id = $1 AND organization_id = $2 AND task_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_TASKS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, task_id, owner_user_id, session_id, title, prompt, schedule_kind, cron_expression, timezone, scheduled_at::text AS scheduled_at, starts_at::text AS starts_at, ends_at::text AS ends_at, next_fire_at::text AS next_fire_at, misfire_policy, overlap_policy, max_concurrent_runs, max_catch_up_runs, max_attempts, retry_initial_delay_seconds, retry_max_delay_seconds, timeout_seconds, priority, status, generation, external_ref, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, completed_at::text AS completed_at, paused_at::text AS paused_at, cancelled_at::text AS cancelled_at FROM ai_agent_task WHERE tenant_id = $1 AND organization_id = $2 AND ($3::text IS NULL OR agent_id = $3) AND ($4::bigint IS NULL OR owner_user_id = $4) AND ($5::smallint IS NULL OR status = $5) AND ($6::timestamptz IS NULL OR (updated_at, id) < ($6::timestamptz, $7::bigint)) ORDER BY updated_at DESC, id DESC LIMIT $8";

#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_TASK_SCHEDULER_METRICS_SNAPSHOT: &str = r#"
SELECT
    (SELECT COUNT(*)::bigint FROM (
        SELECT 1 FROM ai_agent_task
         WHERE status = 0 AND next_fire_at <= $1::timestamptz
         LIMIT $2
    ) bounded_due_tasks) AS due_tasks,
    COALESCE((SELECT GREATEST(
        FLOOR(EXTRACT(EPOCH FROM ($1::timestamptz - MIN(next_fire_at))))::bigint,
        0
    ) FROM ai_agent_task
      WHERE status = 0 AND next_fire_at <= $1::timestamptz), 0) AS materialization_lag_seconds,
    (SELECT COUNT(*)::bigint FROM (
        SELECT 1 FROM ai_agent_task_run
         WHERE status = 0 AND available_at <= $1::timestamptz
         LIMIT $2
    ) bounded_eligible_runs) AS eligible_runs,
    COALESCE((SELECT GREATEST(
        FLOOR(EXTRACT(EPOCH FROM ($1::timestamptz - MIN(available_at))))::bigint,
        0
    ) FROM ai_agent_task_run
      WHERE status = 0 AND available_at <= $1::timestamptz), 0) AS eligible_run_oldest_age_seconds,
    (SELECT COUNT(*)::bigint FROM (
        SELECT 1 FROM ai_agent_task_run
         WHERE status IN (1, 2) AND lease_expires_at >= $1::timestamptz
         LIMIT $2
    ) bounded_active_leases) AS active_leases,
    (SELECT COUNT(*)::bigint FROM (
        SELECT 1 FROM ai_agent_task_run WHERE status = 6 LIMIT $2
    ) bounded_reconciling_runs) AS reconciling_runs,
    COALESCE((SELECT GREATEST(
        FLOOR(EXTRACT(EPOCH FROM ($1::timestamptz - MIN(updated_at))))::bigint,
        0
    ) FROM ai_agent_task_run WHERE status = 6), 0) AS reconciliation_oldest_age_seconds,
    (SELECT COUNT(*)::bigint FROM (
        SELECT 1 FROM ai_agent_outbox_event WHERE status IN (0, 1, 3) LIMIT $2
    ) bounded_pending_outbox_events) AS pending_outbox_events,
    COALESCE((SELECT GREATEST(
        FLOOR(EXTRACT(EPOCH FROM ($1::timestamptz - MIN(created_at))))::bigint,
        0
    ) FROM ai_agent_outbox_event WHERE status IN (0, 1, 3)), 0) AS outbox_oldest_age_seconds
"#;

// ---------------------------------------------------------------------------
// Media tool configuration (tenant-scoped admin settings)
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres-sync")]
pub const SQL_UPSERT_AGENT_TOOL_CONFIGURATION: &str =
    "INSERT INTO ai_agent_tool_configuration (id, uuid, tenant_id, organization_id, tool_id, enabled, save_to_drive_default, default_arguments_json, version, created_by, updated_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::jsonb, 0, $9, $10, $11::timestamptz, $12::timestamptz) ON CONFLICT (tenant_id, organization_id, tool_id) DO UPDATE SET enabled = EXCLUDED.enabled, save_to_drive_default = EXCLUDED.save_to_drive_default, default_arguments_json = EXCLUDED.default_arguments_json, updated_by = EXCLUDED.updated_by, updated_at = EXCLUDED.updated_at, version = ai_agent_tool_configuration.version + 1 WHERE ai_agent_tool_configuration.version = $13 RETURNING id, uuid, tenant_id, organization_id, tool_id, enabled, save_to_drive_default, default_arguments_json::text AS default_arguments_json, version, created_by, updated_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at";

#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_TOOL_CONFIGURATION: &str =
    "SELECT id, uuid, tenant_id, organization_id, tool_id, enabled, save_to_drive_default, default_arguments_json::text AS default_arguments_json, version, created_by, updated_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_tool_configuration WHERE tenant_id = $1 AND organization_id = $2 AND tool_id = $3 AND deleted_at IS NULL LIMIT 1";

#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_TOOL_CONFIGURATIONS: &str =
    "SELECT id, uuid, tenant_id, organization_id, tool_id, enabled, save_to_drive_default, default_arguments_json::text AS default_arguments_json, version, created_by, updated_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_tool_configuration WHERE tenant_id = $1 AND organization_id = $2 AND deleted_at IS NULL ORDER BY updated_at DESC, id DESC";

// ---------------------------------------------------------------------------
// Generated-media tool assets (saveToDrive registration)
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_TOOL_ASSET: &str =
    "INSERT INTO ai_agent_tool_asset (id, uuid, tenant_id, organization_id, user_id, tool_id, tool_call_id, media_kind, drive_space_id, drive_node_id, drive_uri, source_url, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14::timestamptz, $15::timestamptz) ON CONFLICT (tenant_id, organization_id, drive_space_id, drive_node_id) DO NOTHING";

#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_TOOL_ASSETS: &str =
    "SELECT id, uuid, tenant_id, organization_id, user_id, tool_id, tool_call_id, media_kind, drive_space_id, drive_node_id, drive_uri, source_url, created_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_tool_asset WHERE tenant_id = $1 AND organization_id = $2 AND ($3 = 0 OR user_id = $3) AND deleted_at IS NULL ORDER BY created_at DESC, id DESC LIMIT $4";
