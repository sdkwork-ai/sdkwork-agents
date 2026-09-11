-- SDKWork Agents PostgreSQL greenfield baseline.
-- Contract: database/contract/schema.yaml (7.2.0)
-- PostgreSQL is the only managed-store authority for this contract.
-- Sibling modules own memory, knowledge, skills, prompts, MCP, model catalogs,
-- runtime-location details, and Drive bytes. Agents stores stable references only.

CREATE OR REPLACE FUNCTION sdkwork_agents_capabilities_json_is_standard(input JSONB)
RETURNS BOOLEAN
LANGUAGE plpgsql
IMMUTABLE
AS $$
BEGIN
    IF jsonb_typeof(input) <> 'array' THEN
        RETURN FALSE;
    END IF;

    RETURN NOT EXISTS (
        SELECT 1
        FROM jsonb_array_elements(input) AS capability_values(value)
        WHERE NOT (
            jsonb_typeof(capability_values.value) = 'string'
            AND char_length(capability_values.value #>> '{}') BETWEEN 1 AND 128
            AND (capability_values.value #>> '{}') ~ '^[a-z0-9_-]+(\.[a-z0-9_-]+)+$'
        )
    )
    AND (
        SELECT COUNT(*) FROM jsonb_array_elements(input)
    ) = (
        SELECT COUNT(DISTINCT capability_values.value #>> '{}')
        FROM jsonb_array_elements(input) AS capability_values(value)
    );
EXCEPTION WHEN others THEN
    RETURN FALSE;
END;
$$;

CREATE TABLE IF NOT EXISTS ai_agent (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    code VARCHAR(128) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    manifest_json JSONB NOT NULL,
    manifest_schema_version VARCHAR(32),
    default_code_task_intent_json JSONB,
    implementation_provider_id VARCHAR(128),
    implementation_kind VARCHAR(64),
    implementation_type VARCHAR(64) NOT NULL DEFAULT 'sdkwork-native',
    status SMALLINT NOT NULL,
    visibility SMALLINT NOT NULL,
    tags_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT uk_ai_agent_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_tenant_agent_id UNIQUE (tenant_id, agent_id),
    CONSTRAINT uk_ai_agent_tenant_code UNIQUE (tenant_id, code),
    CONSTRAINT uk_ai_agent_tenant_id UNIQUE (tenant_id, id),
    CONSTRAINT ck_ai_agent_manifest_json CHECK (jsonb_typeof(manifest_json) = 'object'),
    CONSTRAINT ck_ai_agent_default_intent_json CHECK (
        default_code_task_intent_json IS NULL
        OR jsonb_typeof(default_code_task_intent_json) = 'object'
    ),
    CONSTRAINT ck_ai_agent_tags_json CHECK (jsonb_typeof(tags_json) = 'array'),
    CONSTRAINT ck_ai_agent_implementation_kind CHECK (
        implementation_kind IS NULL OR implementation_kind IN (
            'manifest-only', 'typed-local-provider', 'process-adapter', 'protocol-adapter'
        )
    ),
    CONSTRAINT ck_ai_agent_implementation_type CHECK (
        implementation_type IN (
            'sdkwork-native', 'rig-rust', 'openai-agents', 'langchain', 'langgraph',
            'crewai', 'autogen', 'semantic-kernel', 'custom'
        )
    ),
    CONSTRAINT ck_ai_agent_provider_id CHECK (
        implementation_provider_id IS NULL
        OR implementation_provider_id ~ '^provider\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_visibility CHECK (visibility IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_version CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_org_status_updated
    ON ai_agent (tenant_id, organization_id, status, updated_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_owner_status
    ON ai_agent (tenant_id, owner_user_id, status, updated_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_list_filters
    ON ai_agent (tenant_id, organization_id, owner_user_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_runtime_binding (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    binding_id VARCHAR(128) NOT NULL,
    provider_id VARCHAR(128) NOT NULL,
    implementation_kind VARCHAR(64) NOT NULL,
    configuration_profile_id VARCHAR(128) NOT NULL,
    capabilities_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_runtime_binding_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_runtime_binding_scope UNIQUE (tenant_id, agent_id, binding_id),
    CONSTRAINT ck_ai_agent_runtime_binding_implementation_kind CHECK (
        implementation_kind IN (
            'manifest-only', 'typed-local-provider', 'process-adapter', 'protocol-adapter'
        )
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_id CHECK (
        binding_id ~ '^binding\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_provider_id CHECK (
        provider_id ~ '^provider\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_profile_id CHECK (
        configuration_profile_id ~ '^profile\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_capabilities CHECK (
        sdkwork_agents_capabilities_json_is_standard(capabilities_json)
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_version CHECK (version >= 1),
    CONSTRAINT fk_ai_agent_runtime_binding_agent FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_runtime_binding_active
    ON ai_agent_runtime_binding (tenant_id, agent_id) WHERE active = TRUE;
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_agent_updated
    ON ai_agent_runtime_binding (
        tenant_id, agent_id, active DESC, updated_at DESC, binding_id ASC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_provider
    ON ai_agent_runtime_binding (tenant_id, provider_id, updated_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS ai_agent_composition_slot (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    slot_id VARCHAR(128) NOT NULL,
    slot_kind VARCHAR(64) NOT NULL,
    target_module VARCHAR(64) NOT NULL,
    target_ref VARCHAR(256) NOT NULL,
    target_version_ref VARCHAR(128),
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    policy_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    status SMALLINT NOT NULL DEFAULT 1,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_composition_slot_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_composition_slot_scope UNIQUE (tenant_id, agent_id, slot_id),
    CONSTRAINT ck_ai_agent_composition_slot_kind CHECK (
        slot_kind IN (
            'memory', 'knowledge', 'skill', 'prompt', 'drive', 'document', 'tool', 'mcp'
        )
    ),
    CONSTRAINT ck_ai_agent_composition_slot_module CHECK (
        target_module IN (
            'memory', 'knowledgebase', 'skills', 'prompts', 'drive', 'documents',
            'tools', 'mcp'
        )
    ),
    CONSTRAINT ck_ai_agent_composition_slot_pair CHECK (
        (slot_kind, target_module) IN (
            ('memory', 'memory'),
            ('knowledge', 'knowledgebase'),
            ('skill', 'skills'),
            ('prompt', 'prompts'),
            ('drive', 'drive'),
            ('document', 'documents'),
            ('tool', 'tools'),
            ('mcp', 'mcp')
        )
    ),
    CONSTRAINT ck_ai_agent_composition_slot_id CHECK (
        slot_id ~ '^slot\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_composition_slot_policy CHECK (
        jsonb_typeof(policy_json) = 'object' AND octet_length(policy_json::text) <= 65536
    ),
    CONSTRAINT ck_ai_agent_composition_slot_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_composition_slot_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_composition_slot_agent FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_composition_slot_lookup
    ON ai_agent_composition_slot (
        tenant_id, agent_id, slot_kind, enabled, priority, slot_id
    ) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_audit_event (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    aggregate_type VARCHAR(64) NOT NULL,
    aggregate_id VARCHAR(128) NOT NULL,
    agent_internal_id BIGINT,
    agent_id VARCHAR(128),
    action VARCHAR(64) NOT NULL,
    actor_type SMALLINT NOT NULL,
    actor_id BIGINT NOT NULL,
    request_id VARCHAR(128),
    trace_id VARCHAR(128),
    payload_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_audit_event_uuid UNIQUE (uuid),
    CONSTRAINT ck_ai_agent_audit_aggregate_type CHECK (
        aggregate_type IN (
            'agent', 'runtime_binding', 'composition_slot', 'workspace', 'project', 'project_member',
            'session', 'turn', 'session_item', 'item_feedback', 'interaction',
            'checkpoint', 'task', 'task_run', 'share_link'
        )
    ),
    CONSTRAINT ck_ai_agent_audit_agent_scope CHECK (
        (aggregate_type = 'agent' AND agent_internal_id IS NOT NULL AND agent_id IS NOT NULL)
        OR (aggregate_type <> 'agent' AND agent_internal_id IS NULL AND agent_id IS NULL)
    ),
    CONSTRAINT ck_ai_agent_audit_action CHECK (
        action IN (
            'created', 'updated', 'deleted', 'restored', 'status_changed',
            'started', 'completed', 'failed', 'cancelled', 'runtime_executed',
            'runtime_queued', 'runtime_failed',
            'provider_binding_changed', 'composition_slot_created',
            'composition_slot_updated', 'composition_slot_deleted',
            'workspace_created', 'workspace_updated', 'workspace_archived', 'workspace_deleted',
            'project_created', 'project_updated', 'project_archived', 'project_deleted',
            'project_member_added', 'project_member_role_changed', 'project_member_removed',
            'project_composition_slot_created', 'project_composition_slot_updated',
            'project_composition_slot_deleted', 'session_created', 'session_closed',
            'session_renamed', 'session_moved', 'session_archived', 'session_deleted',
            'session_runtime_binding_created', 'session_runtime_binding_updated',
            'session_runtime_binding_activated', 'session_runtime_binding_deactivated',
            'turn_requested', 'turn_cancel_requested',
            'turn_completed', 'turn_failed', 'turn_cancelled', 'session_item_created',
            'session_item_failed', 'session_item_redacted', 'item_feedback_changed',
            'interaction_created', 'interaction_claimed', 'interaction_resolved', 'interaction_rejected',
            'interaction_expired', 'interaction_cancelled', 'session_checkpoint_created',
            'session_checkpoint_restored', 'session_checkpoint_invalidated', 'task_created',
            'task_updated', 'task_paused', 'task_resumed', 'task_completed', 'task_failed',
            'task_cancelled', 'task_run_created', 'task_run_cancel_requested',
            'task_run_reconciled', 'share_link_created',
            'share_link_revoked', 'share_link_expired'
        )
    ),
    CONSTRAINT ck_ai_agent_audit_actor_type CHECK (actor_type IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_audit_payload CHECK (
        jsonb_typeof(payload_json) = 'object' AND octet_length(payload_json::text) <= 65536
    ),
    CONSTRAINT fk_ai_agent_audit_event_agent FOREIGN KEY (tenant_id, agent_internal_id)
        REFERENCES ai_agent (tenant_id, id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_aggregate_created
    ON ai_agent_audit_event (
        tenant_id, organization_id, aggregate_type, aggregate_id, created_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_action_created
    ON ai_agent_audit_event (
        tenant_id, organization_id, action, created_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_retention
    ON ai_agent_audit_event (tenant_id, retention_until, id)
    WHERE retention_until IS NOT NULL;
-- Keyset coverage for the tenant+agent audit feed: the list query orders on
-- (created_at DESC, uuid DESC) with row-value predicates; without this index
-- every page would trigger a scan plus sort.
CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_agent_created
    ON ai_agent_audit_event (tenant_id, agent_id, created_at DESC, uuid DESC);

-- Durable operational state store for managed-agent runtime executions
-- (structured agent calls). Backs `agents.calls.create` (executionMode
-- async), `agents.calls.list`, and `agents.calls.retrieve`. Distinct from
-- `ai_agent_audit_event` (append-only audit feed): runtime executions are
-- mutable operational records that transition queued -> running ->
-- completed/failed.
CREATE TABLE IF NOT EXISTS ai_agent_runtime_execution (
    id BIGINT NOT NULL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    execution_id VARCHAR(128) NOT NULL,
    operation VARCHAR(32) NOT NULL,
    status VARCHAR(16) NOT NULL,
    input_payload_json JSONB NOT NULL,
    output_payload_json JSONB NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_runtime_execution UNIQUE (tenant_id, agent_id, execution_id),
    CONSTRAINT ck_ai_agent_runtime_execution_operation CHECK (
        operation IN ('preview_response', 'prompt_optimization', 'agent_call')
    ),
    CONSTRAINT ck_ai_agent_runtime_execution_status CHECK (
        status IN ('queued', 'running', 'completed', 'failed')
    )
);

-- Keyset coverage for the tenant+agent execution list: the list query orders
-- on (requested_at DESC, execution_id DESC) with row-value predicates and an
-- optional status filter; without this index every page would trigger a scan
-- plus sort.
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_execution_list
    ON ai_agent_runtime_execution (tenant_id, agent_id, status, requested_at DESC, execution_id DESC);

-- Crash-recovery sweep: locates executions stuck in queued/running whose
-- owning process died before reaching a terminal state.
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_execution_stale
    ON ai_agent_runtime_execution (tenant_id, status, completed_at);

-- Immutable published snapshots of agent definitions (`agents.versions.*`).
-- Version rows are write-once: manifest and metadata never change after
-- creation. Activation (publish/rollback) is the single `activated_at`
-- marker per agent; re-activating an older version is the rollback path.
CREATE TABLE IF NOT EXISTS ai_agent_version (
    id BIGINT NOT NULL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    version_id VARCHAR(128) NOT NULL,
    version_number BIGINT NOT NULL,
    manifest_json JSONB NOT NULL,
    default_code_task_intent_json JSONB,
    implementation_provider_id VARCHAR(128),
    implementation_kind VARCHAR(32),
    implementation_type VARCHAR(32),
    description VARCHAR(512),
    created_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    activated_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_version_id UNIQUE (tenant_id, organization_id, agent_id, version_id),
    CONSTRAINT uk_ai_agent_version_number UNIQUE (tenant_id, organization_id, agent_id, version_number),
    CONSTRAINT ck_ai_agent_version_number CHECK (version_number >= 1)
);

-- Version history feed: newest first with keyset predicates.
CREATE INDEX IF NOT EXISTS idx_ai_agent_version_list
    ON ai_agent_version (tenant_id, organization_id, agent_id, version_number DESC);

-- Outbound webhook subscription (`agents.webhooks.*`). A subscription binds
-- an HTTPS endpoint to a set of agent event types and carries the signing
-- secret used for HMAC payload signatures. The secret is write-once from the
-- API surface: it is returned exactly once at creation time and never echoed
-- by read/list operations.
CREATE TABLE IF NOT EXISTS ai_agent_webhook_subscription (
    id BIGINT NOT NULL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    webhook_id VARCHAR(128) NOT NULL,
    url VARCHAR(2048) NOT NULL,
    event_types_json JSONB NOT NULL,
    status SMALLINT NOT NULL,
    secret VARCHAR(128) NOT NULL,
    description VARCHAR(512),
    created_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_webhook_subscription UNIQUE (tenant_id, organization_id, webhook_id),
    CONSTRAINT ck_ai_agent_webhook_subscription_status CHECK (status IN (0, 1))
);

-- Delivery ledger for webhook attempts (`agents.webhooks.test` and future
-- event dispatches). A row is created in `queued` state before the outbound
-- HTTP attempt and completed with the response code / error detail once the
-- attempt reaches a terminal state.
CREATE TABLE IF NOT EXISTS ai_agent_webhook_delivery (
    id BIGINT NOT NULL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    webhook_id VARCHAR(128) NOT NULL,
    delivery_id VARCHAR(128) NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    payload_json JSONB NOT NULL,
    signature VARCHAR(256) NOT NULL,
    status VARCHAR(16) NOT NULL,
    response_code INTEGER,
    error_detail VARCHAR(512),
    created_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_webhook_delivery UNIQUE (tenant_id, organization_id, webhook_id, delivery_id),
    CONSTRAINT ck_ai_agent_webhook_delivery_status CHECK (
        status IN ('queued', 'succeeded', 'failed')
    )
);

-- Delivery history feed per subscription: newest first.
CREATE INDEX IF NOT EXISTS idx_ai_agent_webhook_delivery_list
    ON ai_agent_webhook_delivery (tenant_id, organization_id, webhook_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ai_agent_workspace (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    workspace_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    status SMALLINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    archived_at TIMESTAMPTZ,
    archived_by BIGINT,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_workspace_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_workspace_scope UNIQUE (
        tenant_id, organization_id, workspace_id
    ),
    CONSTRAINT ck_ai_agent_workspace_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_workspace_version CHECK (version >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_workspace_active_default
    ON ai_agent_workspace (tenant_id, organization_id, owner_user_id)
    WHERE is_default = TRUE AND status = 0 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_workspace_owner_list
    ON ai_agent_workspace (
        tenant_id, organization_id, owner_user_id, status,
        is_default DESC, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_workspace_retention
    ON ai_agent_workspace (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE IF NOT EXISTS ai_agent_project (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    project_id VARCHAR(128) NOT NULL,
    workspace_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    visibility SMALLINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    drive_access_mode SMALLINT NOT NULL DEFAULT 0,
    default_agent_id VARCHAR(128),
    default_model_id VARCHAR(128),
    import_source_kind VARCHAR(64),
    import_source_ref VARCHAR(512),
    drive_space_id VARCHAR(128),
    drive_root_entry_id VARCHAR(128),
    drive_logical_path VARCHAR(1024),
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    archived_at TIMESTAMPTZ,
    archived_by BIGINT,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_project_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_project_scope UNIQUE (tenant_id, organization_id, project_id),
    CONSTRAINT ck_ai_agent_project_visibility CHECK (visibility IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_drive_access CHECK (drive_access_mode IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_shared_drive CHECK (
        visibility <> 2 OR drive_access_mode <> 1
    ),
    CONSTRAINT ck_ai_agent_project_version CHECK (version >= 0),
    CONSTRAINT ck_ai_agent_project_import_source CHECK (
        (import_source_kind IS NULL AND import_source_ref IS NULL)
        OR (import_source_kind IS NOT NULL AND import_source_ref IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_project_drive_source CHECK (
        import_source_kind IS DISTINCT FROM 'drive_sandbox'
        OR (drive_space_id IS NOT NULL AND drive_root_entry_id IS NOT NULL)
    ),
    CONSTRAINT fk_ai_agent_project_workspace FOREIGN KEY (
        tenant_id, organization_id, workspace_id
    ) REFERENCES ai_agent_workspace (tenant_id, organization_id, workspace_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_project_default_agent FOREIGN KEY (tenant_id, default_agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_project_owner_list
    ON ai_agent_project (
        tenant_id, organization_id, owner_user_id, workspace_id, status,
        updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_project_workspace_list
    ON ai_agent_project (
        tenant_id, organization_id, workspace_id, status, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_project_org_list
    ON ai_agent_project (
        tenant_id, organization_id, visibility, status, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_project_retention
    ON ai_agent_project (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_project_active_import_source
    ON ai_agent_project (
        tenant_id, organization_id, owner_user_id, import_source_kind, import_source_ref
    ) WHERE import_source_kind IS NOT NULL
        AND import_source_ref IS NOT NULL
        AND deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_project_composition_slot (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    project_id VARCHAR(128) NOT NULL,
    slot_id VARCHAR(128) NOT NULL,
    slot_kind VARCHAR(64) NOT NULL,
    target_module VARCHAR(64) NOT NULL,
    target_ref VARCHAR(256) NOT NULL,
    target_version_ref VARCHAR(128),
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    policy_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_project_slot_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_project_slot_scope UNIQUE (
        tenant_id, organization_id, project_id, slot_id
    ),
    CONSTRAINT ck_ai_agent_project_slot_kind CHECK (
        slot_kind IN (
            'prompt', 'memory', 'knowledge', 'skill', 'mcp', 'drive', 'document', 'tool'
        )
    ),
    CONSTRAINT ck_ai_agent_project_slot_module CHECK (
        target_module IN (
            'prompts', 'memory', 'knowledgebase', 'skills', 'mcp', 'drive', 'documents',
            'tools'
        )
    ),
    CONSTRAINT ck_ai_agent_project_slot_pair CHECK (
        (slot_kind, target_module) IN (
            ('prompt', 'prompts'),
            ('memory', 'memory'),
            ('knowledge', 'knowledgebase'),
            ('skill', 'skills'),
            ('mcp', 'mcp'),
            ('drive', 'drive'),
            ('document', 'documents'),
            ('tool', 'tools')
        )
    ),
    CONSTRAINT ck_ai_agent_project_slot_policy CHECK (
        jsonb_typeof(policy_json) = 'object' AND octet_length(policy_json::text) <= 65536
    ),
    CONSTRAINT ck_ai_agent_project_slot_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_project_slot_project FOREIGN KEY (
        tenant_id, organization_id, project_id
    ) REFERENCES ai_agent_project (tenant_id, organization_id, project_id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_project_slot_lookup
    ON ai_agent_project_composition_slot (
        tenant_id, organization_id, project_id, slot_kind, enabled, priority, id
    ) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_session (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    project_id VARCHAR(128),
    session_kind SMALLINT NOT NULL,
    entry_surface SMALLINT NOT NULL,
    source_module VARCHAR(64),
    source_context_kind VARCHAR(64),
    source_context_id VARCHAR(256),
    parent_session_id VARCHAR(128),
    forked_from_turn_id VARCHAR(128),
    title VARCHAR(512),
    title_source SMALLINT NOT NULL DEFAULT 1,
    status SMALLINT NOT NULL DEFAULT 0,
    item_count BIGINT NOT NULL DEFAULT 0,
    last_item_sequence BIGINT NOT NULL DEFAULT 0,
    total_input_tokens BIGINT NOT NULL DEFAULT 0,
    total_output_tokens BIGINT NOT NULL DEFAULT 0,
    idempotency_key VARCHAR(256),
    payload_hash VARCHAR(128),
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    activity_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_item_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    archived_by BIGINT,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_session_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_session_scope UNIQUE (tenant_id, organization_id, session_id),
    CONSTRAINT uk_ai_agent_session_scope_owner UNIQUE (
        tenant_id, organization_id, session_id, owner_user_id
    ),
    CONSTRAINT uk_ai_agent_session_scope_agent_owner UNIQUE (
        tenant_id, organization_id, session_id, agent_id, owner_user_id
    ),
    CONSTRAINT ck_ai_agent_session_kind CHECK (session_kind IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_session_entry_surface CHECK (entry_surface IN (0, 1, 2, 3, 4, 5, 6)),
    CONSTRAINT ck_ai_agent_session_status CHECK (status IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_session_title_source CHECK (title_source IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_session_counts CHECK (
        item_count >= 0 AND last_item_sequence >= 0
        AND total_input_tokens >= 0 AND total_output_tokens >= 0
        AND item_count <= last_item_sequence
    ),
    CONSTRAINT ck_ai_agent_session_source_context CHECK (
        (source_module IS NULL AND source_context_kind IS NULL AND source_context_id IS NULL)
        OR (source_module IS NOT NULL AND source_context_kind IS NOT NULL AND source_context_id IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_session_fork_lineage CHECK (
        (parent_session_id IS NULL AND forked_from_turn_id IS NULL)
        OR (
            parent_session_id IS NOT NULL AND forked_from_turn_id IS NOT NULL
            AND parent_session_id <> session_id
        )
    ),
    CONSTRAINT ck_ai_agent_session_idempotency CHECK (
        (idempotency_key IS NULL AND payload_hash IS NULL)
        OR (idempotency_key IS NOT NULL AND payload_hash IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_session_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_session_agent FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_session_project FOREIGN KEY (
        tenant_id, organization_id, project_id
    ) REFERENCES ai_agent_project (tenant_id, organization_id, project_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_session_parent FOREIGN KEY (
        tenant_id, organization_id, parent_session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
        ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_session_create_idempotency
    ON ai_agent_session (tenant_id, organization_id, owner_user_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_owner_list
    ON ai_agent_session (
        tenant_id, organization_id, owner_user_id, status, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_owner_activity
    ON ai_agent_session (
        tenant_id, organization_id, owner_user_id, activity_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_project_list
    ON ai_agent_session (
        tenant_id, organization_id, project_id, status, updated_at DESC, id DESC
    ) WHERE project_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_agent_list
    ON ai_agent_session (
        tenant_id, organization_id, agent_id, status, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_source_context
    ON ai_agent_session (
        tenant_id, organization_id, source_module, source_context_kind,
        source_context_id, updated_at DESC, id DESC
    ) WHERE source_context_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_retention
    ON ai_agent_session (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE IF NOT EXISTS ai_agent_session_runtime_binding (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    runtime_binding_id VARCHAR(128) NOT NULL,
    runtime_location_id VARCHAR(256),
    host_mode VARCHAR(32) NOT NULL,
    transport_kind VARCHAR(64) NOT NULL,
    provider_binding_id VARCHAR(128) NOT NULL,
    model_id VARCHAR(128) NOT NULL,
    provider_id VARCHAR(128) NOT NULL,
    provider_session_id VARCHAR(256),
    provider_session_tree_id VARCHAR(256),
    provider_parent_session_id VARCHAR(256),
    provider_forked_from_session_id VARCHAR(256),
    provider_title VARCHAR(512),
    provider_title_source VARCHAR(64),
    provider_preview VARCHAR(4096),
    provider_created_at TIMESTAMPTZ,
    provider_updated_at TIMESTAMPTZ,
    provider_recency_at TIMESTAMPTZ,
    provider_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    provider_archived BOOLEAN NOT NULL DEFAULT FALSE,
    provider_visible BOOLEAN NOT NULL DEFAULT TRUE,
    provider_sort_key VARCHAR(512),
    provider_source VARCHAR(256),
    status SMALLINT NOT NULL DEFAULT 0,
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    activated_at TIMESTAMPTZ,
    deactivated_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_session_runtime_binding_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_session_runtime_binding_scope UNIQUE (
        tenant_id, organization_id, session_id, runtime_binding_id
    ),
    CONSTRAINT ck_ai_agent_session_runtime_binding_host_mode CHECK (
        host_mode ~ '^[a-z][a-z0-9_-]{0,31}$'
    ),
    CONSTRAINT ck_ai_agent_session_runtime_binding_transport CHECK (
        transport_kind ~ '^[a-z][a-z0-9_-]{0,63}$'
    ),
    CONSTRAINT ck_ai_agent_session_runtime_binding_provider CHECK (
        provider_id ~ '^provider\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_session_runtime_binding_provider_session CHECK (
        provider_session_id IS NULL OR (
            provider_session_id <> '' AND provider_session_id = BTRIM(provider_session_id)
        )
    ),
    CONSTRAINT ck_ai_agent_session_runtime_binding_status CHECK (status IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_session_runtime_binding_current CHECK (
        (is_current = TRUE AND status = 0 AND deactivated_at IS NULL)
        OR (is_current = FALSE AND status <> 0)
    ),
    CONSTRAINT ck_ai_agent_session_runtime_binding_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_session_runtime_binding_session FOREIGN KEY (
        tenant_id, organization_id, session_id, owner_user_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id, owner_user_id)
        ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_session_runtime_binding_current
    ON ai_agent_session_runtime_binding (tenant_id, organization_id, session_id)
    WHERE is_current = TRUE;
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_session_runtime_binding_provider_session
    ON ai_agent_session_runtime_binding (
        tenant_id, organization_id, owner_user_id, provider_binding_id, provider_id, provider_session_id
    ) WHERE provider_session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_runtime_binding_list
    ON ai_agent_session_runtime_binding (
        tenant_id, organization_id, session_id, is_current DESC, updated_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_runtime_location
    ON ai_agent_session_runtime_binding (
        tenant_id, organization_id, runtime_location_id, updated_at DESC, id DESC
    ) WHERE runtime_location_id IS NOT NULL AND is_current = TRUE;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_runtime_binding_provider_directory
    ON ai_agent_session_runtime_binding (
        tenant_id, organization_id, owner_user_id, provider_binding_id,
        provider_visible, provider_archived, provider_pinned DESC,
        provider_recency_at DESC, provider_sort_key, id DESC
    ) WHERE provider_session_id IS NOT NULL AND is_current = TRUE;

CREATE TABLE IF NOT EXISTS ai_agent_turn (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    turn_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    runtime_binding_id VARCHAR(128),
    client_request_id VARCHAR(128),
    idempotency_key VARCHAR(256) NOT NULL,
    payload_hash VARCHAR(128) NOT NULL,
    request_item_id VARCHAR(128) NOT NULL,
    response_item_id VARCHAR(128),
    turn_mode SMALLINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    requested_model_id VARCHAR(128),
    provider_binding_id VARCHAR(128),
    model_id VARCHAR(128),
    provider_id VARCHAR(128),
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cached_tokens BIGINT NOT NULL DEFAULT 0,
    finish_reason VARCHAR(64),
    error_code VARCHAR(128),
    error_detail VARCHAR(2048),
    trace_id VARCHAR(128),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    available_at TIMESTAMPTZ NOT NULL,
    lease_owner VARCHAR(128),
    lease_token VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    fencing_token BIGINT NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancel_requested_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    streaming_content TEXT,
    CONSTRAINT uk_ai_agent_turn_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_turn_scope UNIQUE (tenant_id, organization_id, turn_id),
    CONSTRAINT uk_ai_agent_turn_session_scope UNIQUE (
        tenant_id, organization_id, session_id, turn_id
    ),
    CONSTRAINT uk_ai_agent_turn_idempotency UNIQUE (
        tenant_id, organization_id, owner_user_id, idempotency_key
    ),
    CONSTRAINT ck_ai_agent_turn_mode CHECK (turn_mode IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_turn_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_turn_idempotency CHECK (
        char_length(BTRIM(idempotency_key)) > 0
        AND char_length(BTRIM(payload_hash)) > 0
    ),
    CONSTRAINT ck_ai_agent_turn_response_item CHECK (
        response_item_id IS NULL OR response_item_id <> request_item_id
    ),
    CONSTRAINT ck_ai_agent_turn_tokens CHECK (
        input_tokens >= 0 AND output_tokens >= 0 AND cached_tokens >= 0
    ),
    CONSTRAINT ck_ai_agent_turn_attempts CHECK (
        attempt_count >= 0 AND max_attempts > 0 AND attempt_count <= max_attempts
    ),
    CONSTRAINT ck_ai_agent_turn_lease CHECK (
        (lease_owner IS NULL AND lease_token IS NULL AND lease_expires_at IS NULL)
        OR (lease_owner IS NOT NULL AND lease_token IS NOT NULL AND lease_expires_at IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_turn_fencing CHECK (fencing_token >= 0),
    CONSTRAINT ck_ai_agent_turn_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_turn_session FOREIGN KEY (
        tenant_id, organization_id, session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_turn_agent FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_turn_runtime_binding FOREIGN KEY (
        tenant_id, organization_id, session_id, runtime_binding_id
    ) REFERENCES ai_agent_session_runtime_binding (
        tenant_id, organization_id, session_id, runtime_binding_id
    ) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_turn_client_request
    ON ai_agent_turn (tenant_id, organization_id, owner_user_id, client_request_id)
    WHERE client_request_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_session_timeline
    ON ai_agent_turn (
        tenant_id, organization_id, session_id, created_at ASC, id ASC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_session_activity
    ON ai_agent_turn (tenant_id, organization_id, session_id, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_worker
    ON ai_agent_turn (status, available_at, lease_expires_at, id)
    WHERE status IN (0, 1);
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_retry
    ON ai_agent_turn (status, next_retry_at, id)
    WHERE next_retry_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_trace
    ON ai_agent_turn (tenant_id, organization_id, trace_id)
    WHERE trace_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_retention
    ON ai_agent_turn (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

-- Usage metering feeds (`agents.usage.summary` / `agents.usage.records`):
-- tenant-wide and per-agent keyset/aggregation scans over the token facts.
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_usage_timeline
    ON ai_agent_turn (tenant_id, organization_id, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_usage_agent_timeline
    ON ai_agent_turn (tenant_id, organization_id, agent_id, created_at DESC, id DESC);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_turn_active_session
    ON ai_agent_turn (tenant_id, organization_id, session_id)
    WHERE status IN (0, 1);

CREATE TABLE IF NOT EXISTS ai_agent_turn_input_queue_entry (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    queue_entry_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    display_text TEXT NOT NULL DEFAULT '',
    content_type VARCHAR(128) NOT NULL DEFAULT 'text/plain',
    attachment_names_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    drive_refs_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    turn_mode SMALLINT NOT NULL DEFAULT 0,
    runtime_binding_id VARCHAR(128),
    requested_model_id VARCHAR(128),
    access_mode_id VARCHAR(64),
    idempotency_key VARCHAR(256) NOT NULL,
    payload_hash VARCHAR(128) NOT NULL,
    client_request_id VARCHAR(128) NOT NULL,
    position BIGINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    claim_owner VARCHAR(128),
    claim_token_hash VARCHAR(128),
    claim_expires_at TIMESTAMPTZ,
    fencing_token BIGINT NOT NULL DEFAULT 0,
    error_code VARCHAR(128),
    error_detail VARCHAR(1024),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    claimed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_turn_input_queue_entry_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_turn_input_queue_entry_scope UNIQUE (
        tenant_id, organization_id, session_id, queue_entry_id
    ),
    CONSTRAINT uk_ai_agent_turn_input_queue_entry_idempotency UNIQUE (
        tenant_id, organization_id, owner_user_id, idempotency_key
    ),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_mode CHECK (turn_mode IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_content CHECK (
        octet_length(BTRIM(content)) > 0 AND octet_length(content) <= 262144
    ),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_position CHECK (position > 0),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_fencing CHECK (fencing_token >= 0),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_version CHECK (version >= 0),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_claim CHECK (
        (status = 1 AND claim_owner IS NOT NULL AND claim_token_hash IS NOT NULL AND claim_expires_at IS NOT NULL)
        OR (status <> 1 AND claim_owner IS NULL AND claim_token_hash IS NULL AND claim_expires_at IS NULL)
    ),
    CONSTRAINT fk_ai_agent_turn_input_queue_entry_session FOREIGN KEY (
        tenant_id, organization_id, session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id) ON DELETE CASCADE,
    CONSTRAINT fk_ai_agent_turn_input_queue_entry_agent FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_input_queue_session_order
    ON ai_agent_turn_input_queue_entry (
        tenant_id, organization_id, session_id, owner_user_id, position, id
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_input_queue_owner_quota
    ON ai_agent_turn_input_queue_entry (tenant_id, organization_id, owner_user_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_turn_input_queue_executing_session
    ON ai_agent_turn_input_queue_entry (tenant_id, organization_id, session_id)
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS ai_agent_session_item (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    session_id VARCHAR(128) NOT NULL,
    item_id VARCHAR(128) NOT NULL,
    kind SMALLINT NOT NULL,
    content TEXT,
    content_type VARCHAR(64) NOT NULL DEFAULT 'text/plain',
    status SMALLINT NOT NULL DEFAULT 0,
    sequence BIGINT NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    model_id VARCHAR(128),
    provider_id VARCHAR(128),
    tool_name VARCHAR(128),
    tool_call_id VARCHAR(256),
    tool_arguments_json JSONB,
    tool_result_json JSONB,
    provider_payload_json JSONB,
    parent_item_id VARCHAR(128),
    turn_id VARCHAR(128),
    created_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    redacted_at TIMESTAMPTZ,
    redacted_by BIGINT,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_session_item_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_session_item_scope UNIQUE (
        tenant_id, organization_id, item_id
    ),
    CONSTRAINT uk_ai_agent_session_item_session_scope UNIQUE (
        tenant_id, organization_id, session_id, item_id
    ),
    CONSTRAINT uk_ai_agent_session_item_sequence UNIQUE (
        tenant_id, organization_id, session_id, sequence
    ),
    CONSTRAINT ck_ai_agent_session_item_kind CHECK (kind IN (0, 1, 2, 3, 4, 5, 6, 7, 8)),
    CONSTRAINT ck_ai_agent_session_item_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_session_item_sequence CHECK (sequence >= 1),
    CONSTRAINT ck_ai_agent_session_item_parent CHECK (
        parent_item_id IS NULL OR parent_item_id <> item_id
    ),
    CONSTRAINT ck_ai_agent_session_item_tokens CHECK (
        input_tokens >= 0 AND output_tokens >= 0
    ),
    CONSTRAINT ck_ai_agent_session_item_content CHECK (
        (kind IN (0, 1, 2, 3, 7, 8) AND content IS NOT NULL)
        OR kind IN (4, 5, 6)
    ),
    CONSTRAINT ck_ai_agent_session_item_tool_payload CHECK (
        (
            kind = 4 AND tool_name IS NOT NULL AND tool_call_id IS NOT NULL
            AND tool_arguments_json IS NOT NULL AND tool_result_json IS NULL
        ) OR (
            kind = 5 AND tool_call_id IS NOT NULL
            AND tool_arguments_json IS NULL AND tool_result_json IS NOT NULL
        ) OR (
            kind NOT IN (4, 5) AND tool_name IS NULL AND tool_call_id IS NULL
            AND tool_arguments_json IS NULL AND tool_result_json IS NULL
        )
    ),
    CONSTRAINT ck_ai_agent_session_item_tool_arguments_size CHECK (
        tool_arguments_json IS NULL OR octet_length(tool_arguments_json::text) <= 262144
    ),
    CONSTRAINT ck_ai_agent_session_item_tool_result_size CHECK (
        tool_result_json IS NULL OR octet_length(tool_result_json::text) <= 1048576
    ),
    CONSTRAINT ck_ai_agent_session_item_redaction CHECK (
        (status = 4 AND redacted_at IS NOT NULL AND redacted_by IS NOT NULL)
        OR status <> 4
    ),
    CONSTRAINT ck_ai_agent_session_item_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_session_item_session FOREIGN KEY (
        tenant_id, organization_id, session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_session_item_parent FOREIGN KEY (
        tenant_id, organization_id, session_id, parent_item_id
    ) REFERENCES ai_agent_session_item (
        tenant_id, organization_id, session_id, item_id
    ) ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT fk_ai_agent_session_item_turn FOREIGN KEY (
        tenant_id, organization_id, session_id, turn_id
    ) REFERENCES ai_agent_turn (
        tenant_id, organization_id, session_id, turn_id
    ) ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_session_item_timeline
    ON ai_agent_session_item (
        tenant_id, organization_id, session_id, sequence ASC, id ASC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_item_turn
    ON ai_agent_session_item (
        tenant_id, organization_id, session_id, turn_id, sequence ASC, id ASC
    ) WHERE turn_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_item_kind
    ON ai_agent_session_item (
        tenant_id, organization_id, session_id, kind, sequence ASC, id ASC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_item_retention
    ON ai_agent_session_item (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

DO $sdkwork_agents_baseline$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'fk_ai_agent_turn_request_item'
          AND conrelid = 'ai_agent_turn'::regclass
    ) THEN
        ALTER TABLE ai_agent_turn
            ADD CONSTRAINT fk_ai_agent_turn_request_item FOREIGN KEY (
                tenant_id, organization_id, session_id, request_item_id
            ) REFERENCES ai_agent_session_item (
                tenant_id, organization_id, session_id, item_id
            ) ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'fk_ai_agent_turn_response_item'
          AND conrelid = 'ai_agent_turn'::regclass
    ) THEN
        ALTER TABLE ai_agent_turn
            ADD CONSTRAINT fk_ai_agent_turn_response_item FOREIGN KEY (
                tenant_id, organization_id, session_id, response_item_id
            ) REFERENCES ai_agent_session_item (
                tenant_id, organization_id, session_id, item_id
            ) ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'fk_ai_agent_session_fork_turn'
          AND conrelid = 'ai_agent_session'::regclass
    ) THEN
        ALTER TABLE ai_agent_session
            ADD CONSTRAINT fk_ai_agent_session_fork_turn FOREIGN KEY (
                tenant_id, organization_id, parent_session_id, forked_from_turn_id
            ) REFERENCES ai_agent_turn (
                tenant_id, organization_id, session_id, turn_id
            ) ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
    END IF;
END
$sdkwork_agents_baseline$;

CREATE TABLE IF NOT EXISTS ai_agent_item_drive_ref (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    item_id VARCHAR(128) NOT NULL,
    resource_role VARCHAR(64) NOT NULL,
    drive_space_id VARCHAR(128) NOT NULL,
    drive_node_id VARCHAR(128) NOT NULL,
    media_resource_id VARCHAR(128),
    object_blob_id VARCHAR(128),
    resource_hash VARCHAR(128),
    alt_text VARCHAR(512),
    sort_order INTEGER NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_item_drive_ref_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_item_drive_ref_resource UNIQUE (
        tenant_id, organization_id, item_id, drive_space_id, drive_node_id, resource_role
    ),
    CONSTRAINT ck_ai_agent_item_drive_ref_role CHECK (
        resource_role IN ('attachment', 'image', 'audio', 'generated_output', 'artifact')
    ),
    CONSTRAINT ck_ai_agent_item_drive_ref_order CHECK (sort_order >= 0),
    CONSTRAINT ck_ai_agent_item_drive_ref_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT fk_ai_agent_item_drive_ref_item FOREIGN KEY (
        tenant_id, organization_id, item_id
    ) REFERENCES ai_agent_session_item (tenant_id, organization_id, item_id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_item_drive_ref_list
    ON ai_agent_item_drive_ref (
        tenant_id, organization_id, item_id, status, sort_order, id
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_item_drive_ref_drive
    ON ai_agent_item_drive_ref (
        tenant_id, organization_id, drive_space_id, drive_node_id
    );

CREATE TABLE IF NOT EXISTS ai_agent_item_feedback (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    item_id VARCHAR(128) NOT NULL,
    user_id BIGINT NOT NULL,
    rating SMALLINT NOT NULL,
    reason_code VARCHAR(64),
    comment VARCHAR(1024),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_item_feedback_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_item_feedback_user UNIQUE (
        tenant_id, organization_id, item_id, user_id
    ),
    CONSTRAINT ck_ai_agent_item_feedback_rating CHECK (rating IN (1, -1)),
    CONSTRAINT ck_ai_agent_item_feedback_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_item_feedback_item FOREIGN KEY (
        tenant_id, organization_id, item_id
    ) REFERENCES ai_agent_session_item (tenant_id, organization_id, item_id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_item_feedback_analytics
    ON ai_agent_item_feedback (
        tenant_id, organization_id, rating, created_at DESC, id DESC
    ) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_interaction (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    session_id VARCHAR(128) NOT NULL,
    turn_id VARCHAR(128),
    runtime_binding_id VARCHAR(128),
    interaction_id VARCHAR(128) NOT NULL,
    provider_interaction_id VARCHAR(256),
    kind SMALLINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    prompt TEXT NOT NULL,
    options_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    request_json JSONB,
    resolution_json JSONB,
    claim_owner VARCHAR(128),
    claim_token_hash VARCHAR(128),
    claim_expires_at TIMESTAMPTZ,
    fencing_token BIGINT NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_interaction_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_interaction_scope UNIQUE (
        tenant_id, organization_id, session_id, interaction_id
    ),
    CONSTRAINT ck_ai_agent_interaction_kind CHECK (kind IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_interaction_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_interaction_options CHECK (
        jsonb_typeof(options_json) = 'array' AND octet_length(options_json::text) <= 65536
    ),
    CONSTRAINT ck_ai_agent_interaction_request CHECK (
        request_json IS NULL
        OR (jsonb_typeof(request_json) = 'object' AND octet_length(request_json::text) <= 65536)
    ),
    CONSTRAINT ck_ai_agent_interaction_resolution CHECK (
        resolution_json IS NULL
        OR (jsonb_typeof(resolution_json) = 'object' AND octet_length(resolution_json::text) <= 65536)
    ),
    CONSTRAINT ck_ai_agent_interaction_claim CHECK (
        (claim_owner IS NULL AND claim_token_hash IS NULL AND claim_expires_at IS NULL)
        OR (claim_owner IS NOT NULL AND claim_token_hash IS NOT NULL AND claim_expires_at IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_interaction_provider_scope CHECK (
        provider_interaction_id IS NULL OR runtime_binding_id IS NOT NULL
    ),
    CONSTRAINT ck_ai_agent_interaction_resolution_state CHECK (
        (status = 0 AND resolved_at IS NULL AND resolution_json IS NULL)
        OR (status <> 0 AND resolved_at IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_interaction_fencing CHECK (fencing_token >= 0),
    CONSTRAINT ck_ai_agent_interaction_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_interaction_session FOREIGN KEY (
        tenant_id, organization_id, session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_interaction_turn FOREIGN KEY (
        tenant_id, organization_id, session_id, turn_id
    ) REFERENCES ai_agent_turn (
        tenant_id, organization_id, session_id, turn_id
    ) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_interaction_runtime_binding FOREIGN KEY (
        tenant_id, organization_id, session_id, runtime_binding_id
    ) REFERENCES ai_agent_session_runtime_binding (
        tenant_id, organization_id, session_id, runtime_binding_id
    ) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_interaction_provider
    ON ai_agent_interaction (
        tenant_id, organization_id, session_id, runtime_binding_id, provider_interaction_id
    ) WHERE provider_interaction_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_pending
    ON ai_agent_interaction (
        tenant_id, organization_id, session_id, status, created_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_session_activity
    ON ai_agent_interaction (tenant_id, organization_id, session_id, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_session_kind_activity
    ON ai_agent_interaction (tenant_id, organization_id, session_id, kind ASC, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_claim
    ON ai_agent_interaction (status, claim_expires_at, id) WHERE status = 0;

CREATE TABLE IF NOT EXISTS ai_agent_session_checkpoint (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    session_id VARCHAR(128) NOT NULL,
    checkpoint_id VARCHAR(128) NOT NULL,
    turn_id VARCHAR(128),
    runtime_binding_id VARCHAR(128),
    checkpoint_kind VARCHAR(64) NOT NULL,
    provider_checkpoint_ref VARCHAR(256),
    drive_space_id VARCHAR(128),
    drive_node_id VARCHAR(128),
    resumable BOOLEAN NOT NULL DEFAULT TRUE,
    status SMALLINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    restored_at TIMESTAMPTZ,
    invalidated_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_session_checkpoint_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_session_checkpoint_scope UNIQUE (
        tenant_id, organization_id, session_id, checkpoint_id
    ),
    CONSTRAINT ck_ai_agent_session_checkpoint_kind CHECK (
        checkpoint_kind ~ '^[a-z][a-z0-9_-]{0,63}$'
    ),
    CONSTRAINT ck_ai_agent_session_checkpoint_backing CHECK (
        (
            provider_checkpoint_ref IS NOT NULL AND runtime_binding_id IS NOT NULL
            AND drive_space_id IS NULL AND drive_node_id IS NULL
        ) OR (
            provider_checkpoint_ref IS NULL
            AND drive_space_id IS NOT NULL AND drive_node_id IS NOT NULL
        )
    ),
    CONSTRAINT ck_ai_agent_session_checkpoint_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_session_checkpoint_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_session_checkpoint_session FOREIGN KEY (
        tenant_id, organization_id, session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_session_checkpoint_turn FOREIGN KEY (
        tenant_id, organization_id, session_id, turn_id
    ) REFERENCES ai_agent_turn (
        tenant_id, organization_id, session_id, turn_id
    ) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_session_checkpoint_runtime_binding FOREIGN KEY (
        tenant_id, organization_id, session_id, runtime_binding_id
    ) REFERENCES ai_agent_session_runtime_binding (
        tenant_id, organization_id, session_id, runtime_binding_id
    ) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_session_checkpoint_provider
    ON ai_agent_session_checkpoint (
        tenant_id, organization_id, session_id, runtime_binding_id, provider_checkpoint_ref
    ) WHERE provider_checkpoint_ref IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_checkpoint_list
    ON ai_agent_session_checkpoint (
        tenant_id, organization_id, session_id, status, created_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_checkpoint_retention
    ON ai_agent_session_checkpoint (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE IF NOT EXISTS ai_agent_task (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    task_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    title VARCHAR(512),
    prompt TEXT NOT NULL,
    schedule_kind SMALLINT NOT NULL,
    cron_expression VARCHAR(256),
    timezone VARCHAR(128) NOT NULL,
    scheduled_at TIMESTAMPTZ,
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    next_fire_at TIMESTAMPTZ,
    misfire_policy SMALLINT NOT NULL DEFAULT 0,
    overlap_policy SMALLINT NOT NULL DEFAULT 0,
    max_concurrent_runs SMALLINT NOT NULL DEFAULT 1,
    max_catch_up_runs SMALLINT NOT NULL DEFAULT 1,
    max_attempts SMALLINT NOT NULL DEFAULT 3,
    retry_initial_delay_seconds INTEGER NOT NULL DEFAULT 5,
    retry_max_delay_seconds INTEGER NOT NULL DEFAULT 300,
    timeout_seconds INTEGER NOT NULL DEFAULT 900,
    priority SMALLINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    generation BIGINT NOT NULL DEFAULT 1,
    external_ref VARCHAR(256),
    metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    paused_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_task_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_task_scope UNIQUE (tenant_id, organization_id, task_id),
    CONSTRAINT ck_ai_agent_task_schedule_kind CHECK (schedule_kind IN (0, 1)),
    CONSTRAINT ck_ai_agent_task_schedule_shape CHECK (
        (
            schedule_kind = 0 AND scheduled_at IS NOT NULL
            AND cron_expression IS NULL
        ) OR (
            schedule_kind = 1 AND scheduled_at IS NULL
            AND cron_expression IS NOT NULL
            AND octet_length(BTRIM(cron_expression)) BETWEEN 9 AND 256
        )
    ),
    CONSTRAINT ck_ai_agent_task_timezone CHECK (
        octet_length(BTRIM(timezone)) BETWEEN 1 AND 128
    ),
    CONSTRAINT ck_ai_agent_task_window CHECK (
        starts_at IS NULL OR ends_at IS NULL OR starts_at < ends_at
    ),
    CONSTRAINT ck_ai_agent_task_misfire CHECK (misfire_policy IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_task_overlap CHECK (overlap_policy IN (0, 1)),
    CONSTRAINT ck_ai_agent_task_limits CHECK (
        max_concurrent_runs BETWEEN 1 AND 32
        AND max_catch_up_runs BETWEEN 1 AND 100
        AND max_attempts BETWEEN 1 AND 20
        AND retry_initial_delay_seconds BETWEEN 1 AND 86400
        AND retry_max_delay_seconds BETWEEN retry_initial_delay_seconds AND 604800
        AND timeout_seconds BETWEEN 1 AND 86400
        AND priority BETWEEN -100 AND 100
    ),
    CONSTRAINT ck_ai_agent_task_status CHECK (status IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_task_generation CHECK (generation > 0),
    CONSTRAINT ck_ai_agent_task_lifecycle CHECK (
        (status = 0 AND next_fire_at IS NOT NULL AND paused_at IS NULL AND cancelled_at IS NULL)
        OR (status = 1 AND next_fire_at IS NULL AND paused_at IS NOT NULL AND cancelled_at IS NULL)
        OR (status = 2 AND next_fire_at IS NULL AND completed_at IS NOT NULL AND cancelled_at IS NULL)
        OR (status = 3 AND next_fire_at IS NULL AND cancelled_at IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_task_metadata CHECK (
        jsonb_typeof(metadata_json) = 'object' AND octet_length(metadata_json::text) <= 65536
    ),
    CONSTRAINT ck_ai_agent_task_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_task_agent FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_task_session FOREIGN KEY (
        tenant_id, organization_id, session_id, agent_id, owner_user_id
    ) REFERENCES ai_agent_session (
        tenant_id, organization_id, session_id, agent_id, owner_user_id
    ) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_task_due
    ON ai_agent_task (next_fire_at, priority DESC, id)
    WHERE status = 0 AND next_fire_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_agent_status
    ON ai_agent_task (
        tenant_id, organization_id, agent_id, status, updated_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_agent_owner_list
    ON ai_agent_task (
        tenant_id, organization_id, agent_id, owner_user_id, updated_at DESC, id DESC
    ) INCLUDE (status);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_owner_status
    ON ai_agent_task (
        tenant_id, organization_id, owner_user_id, status, updated_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_session
    ON ai_agent_task (
        tenant_id, organization_id, session_id, status, updated_at DESC, id DESC
    );

CREATE TABLE IF NOT EXISTS ai_agent_task_run (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    run_id VARCHAR(128) NOT NULL,
    task_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    trigger_kind SMALLINT NOT NULL,
    schedule_generation BIGINT NOT NULL,
    scheduled_for TIMESTAMPTZ NOT NULL,
    retry_of_run_id VARCHAR(128),
    priority SMALLINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    idempotency_key VARCHAR(256) NOT NULL,
    payload_hash VARCHAR(128) NOT NULL,
    turn_id VARCHAR(128),
    attempt_count SMALLINT NOT NULL DEFAULT 0,
    max_attempts SMALLINT NOT NULL,
    available_at TIMESTAMPTZ NOT NULL,
    lease_owner VARCHAR(128),
    lease_token_hash VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    fencing_token BIGINT NOT NULL DEFAULT 0,
    timeout_at TIMESTAMPTZ,
    failure_class VARCHAR(64),
    error_code VARCHAR(128),
    error_detail VARCHAR(1024),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    claimed_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_task_run_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_task_run_scope UNIQUE (
        tenant_id, organization_id, run_id
    ),
    CONSTRAINT uk_ai_agent_task_run_idempotency UNIQUE (
        tenant_id, organization_id, owner_user_id, idempotency_key
    ),
    CONSTRAINT ck_ai_agent_task_run_trigger CHECK (trigger_kind IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_task_run_status CHECK (status IN (0, 1, 2, 3, 4, 5, 6, 7)),
    CONSTRAINT ck_ai_agent_task_run_generation CHECK (schedule_generation > 0),
    CONSTRAINT ck_ai_agent_task_run_priority CHECK (priority BETWEEN -100 AND 100),
    CONSTRAINT ck_ai_agent_task_run_attempts CHECK (
        max_attempts BETWEEN 1 AND 20
        AND attempt_count BETWEEN 0 AND max_attempts
    ),
    CONSTRAINT ck_ai_agent_task_run_fencing CHECK (fencing_token >= 0),
    CONSTRAINT ck_ai_agent_task_run_version CHECK (version >= 0),
    CONSTRAINT ck_ai_agent_task_run_retry CHECK (
        (trigger_kind = 2 AND retry_of_run_id IS NOT NULL)
        OR (trigger_kind <> 2 AND retry_of_run_id IS NULL)
    ),
    CONSTRAINT ck_ai_agent_task_run_lease CHECK (
        (status IN (1, 2) AND lease_owner IS NOT NULL AND lease_token_hash IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR (status NOT IN (1, 2) AND lease_owner IS NULL AND lease_token_hash IS NULL AND lease_expires_at IS NULL)
    ),
    CONSTRAINT ck_ai_agent_task_run_terminal CHECK (
        (status IN (3, 4, 5, 7) AND finished_at IS NOT NULL)
        OR (status NOT IN (3, 4, 5, 7) AND finished_at IS NULL)
    ),
    CONSTRAINT fk_ai_agent_task_run_task FOREIGN KEY (
        tenant_id, organization_id, task_id
    ) REFERENCES ai_agent_task (tenant_id, organization_id, task_id) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_task_run_session FOREIGN KEY (
        tenant_id, organization_id, session_id, agent_id, owner_user_id
    ) REFERENCES ai_agent_session (
        tenant_id, organization_id, session_id, agent_id, owner_user_id
    ) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_task_run_retry FOREIGN KEY (
        tenant_id, organization_id, retry_of_run_id
    ) REFERENCES ai_agent_task_run (tenant_id, organization_id, run_id)
        ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_task_run_occurrence
    ON ai_agent_task_run (
        tenant_id, organization_id, task_id, schedule_generation, scheduled_for
    ) WHERE trigger_kind = 0;
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_claim
    ON ai_agent_task_run (priority DESC, available_at, scheduled_for, id)
    WHERE status = 0;
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_active_task
    ON ai_agent_task_run (
        tenant_id, organization_id, task_id, status, scheduled_for, id
    ) WHERE status IN (1, 2, 6);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_list
    ON ai_agent_task_run (
        tenant_id, organization_id, task_id, id DESC
    ) INCLUDE (owner_user_id, status, trigger_kind);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_expired_lease
    ON ai_agent_task_run (lease_expires_at, id)
    WHERE status IN (1, 2);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_reconcile
    ON ai_agent_task_run (updated_at, id) WHERE status = 6;
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_retention
    ON ai_agent_task_run (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE IF NOT EXISTS ai_agent_task_run_attempt (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    attempt_id VARCHAR(128) NOT NULL,
    run_id VARCHAR(128) NOT NULL,
    attempt_no SMALLINT NOT NULL,
    worker_id VARCHAR(128) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    lease_token_hash VARCHAR(128) NOT NULL,
    fencing_token BIGINT NOT NULL,
    failure_class VARCHAR(64),
    error_code VARCHAR(128),
    error_detail VARCHAR(1024),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    heartbeat_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_task_run_attempt_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_task_run_attempt_scope UNIQUE (
        tenant_id, organization_id, attempt_id
    ),
    CONSTRAINT uk_ai_agent_task_run_attempt_number UNIQUE (
        tenant_id, organization_id, run_id, attempt_no
    ),
    CONSTRAINT ck_ai_agent_task_run_attempt_number CHECK (attempt_no BETWEEN 1 AND 20),
    CONSTRAINT ck_ai_agent_task_run_attempt_status CHECK (status IN (0, 1, 2, 3, 4, 5)),
    CONSTRAINT ck_ai_agent_task_run_attempt_fencing CHECK (fencing_token > 0),
    CONSTRAINT ck_ai_agent_task_run_attempt_terminal CHECK (
        (status IN (2, 3, 4, 5) AND finished_at IS NOT NULL)
        OR (status IN (0, 1) AND finished_at IS NULL)
    ),
    CONSTRAINT fk_ai_agent_task_run_attempt_run FOREIGN KEY (
        tenant_id, organization_id, run_id
    ) REFERENCES ai_agent_task_run (tenant_id, organization_id, run_id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_attempt_run
    ON ai_agent_task_run_attempt (
        tenant_id, organization_id, run_id, attempt_no DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_attempt_worker
    ON ai_agent_task_run_attempt (worker_id, status, heartbeat_at, id)
    WHERE status IN (0, 1);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_attempt_retention
    ON ai_agent_task_run_attempt (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE IF NOT EXISTS ai_agent_resource_user_state (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    user_id BIGINT NOT NULL,
    resource_type SMALLINT NOT NULL,
    resource_id VARCHAR(128) NOT NULL,
    pinned_at TIMESTAMPTZ,
    hidden_at TIMESTAMPTZ,
    last_opened_at TIMESTAMPTZ,
    last_read_item_sequence BIGINT,
    custom_title VARCHAR(512),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_resource_user_state_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_resource_user_state_scope UNIQUE (
        tenant_id, organization_id, user_id, resource_type, resource_id
    ),
    CONSTRAINT ck_ai_agent_resource_user_state_type CHECK (resource_type IN (0, 1)),
    CONSTRAINT ck_ai_agent_resource_user_state_sequence CHECK (
        last_read_item_sequence IS NULL OR last_read_item_sequence >= 0
    ),
    CONSTRAINT ck_ai_agent_resource_user_state_version CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_resource_user_state_recent
    ON ai_agent_resource_user_state (
        tenant_id, organization_id, user_id, resource_type,
        pinned_at DESC, last_opened_at DESC, id DESC
    ) WHERE hidden_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_resource_user_state_session_activity
    ON ai_agent_resource_user_state (tenant_id, organization_id, user_id, resource_type, resource_id, updated_at DESC, id DESC);


CREATE TABLE IF NOT EXISTS ai_agent_project_member (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    project_id VARCHAR(128) NOT NULL,
    member_user_id BIGINT NOT NULL,
    role SMALLINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    invited_by BIGINT,
    joined_at TIMESTAMPTZ,
    removed_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_project_member_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_project_member_user UNIQUE (
        tenant_id, organization_id, project_id, member_user_id
    ),
    CONSTRAINT ck_ai_agent_project_member_role CHECK (role IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_member_status CHECK (status IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_project_member_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_project_member_project FOREIGN KEY (
        tenant_id, organization_id, project_id
    ) REFERENCES ai_agent_project (tenant_id, organization_id, project_id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_project_member_user_list
    ON ai_agent_project_member (
        tenant_id, organization_id, member_user_id, status, updated_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_project_member_project_list
    ON ai_agent_project_member (
        tenant_id, organization_id, project_id, status, updated_at DESC, id DESC
    );

CREATE TABLE IF NOT EXISTS ai_agent_share_link (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    link_id VARCHAR(128) NOT NULL,
    target_type SMALLINT NOT NULL,
    target_id VARCHAR(128) NOT NULL,
    permission SMALLINT NOT NULL,
    token_hash VARCHAR(128) NOT NULL,
    token_prefix VARCHAR(16) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    revoked_by BIGINT,
    max_uses BIGINT,
    use_count BIGINT NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_share_link_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_share_link_scope UNIQUE (
        tenant_id, organization_id, link_id
    ),
    CONSTRAINT uk_ai_agent_share_link_token_hash UNIQUE (token_hash),
    CONSTRAINT ck_ai_agent_share_link_target CHECK (target_type IN (0, 1)),
    CONSTRAINT ck_ai_agent_share_link_permission CHECK (permission IN (0, 1)),
    CONSTRAINT ck_ai_agent_share_link_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_share_link_usage CHECK (
        use_count >= 0 AND (max_uses IS NULL OR (max_uses > 0 AND use_count <= max_uses))
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_share_link_target
    ON ai_agent_share_link (
        tenant_id, organization_id, target_type, target_id, status, expires_at, id
    );

CREATE TABLE IF NOT EXISTS ai_agent_outbox_event (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    event_id VARCHAR(128) NOT NULL,
    aggregate_type VARCHAR(64) NOT NULL,
    aggregate_id VARCHAR(128) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    payload_json JSONB NOT NULL,
    headers_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    dedupe_key VARCHAR(256) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 10,
    available_at TIMESTAMPTZ NOT NULL,
    lease_owner VARCHAR(128),
    lease_token VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    fencing_token BIGINT NOT NULL DEFAULT 0,
    published_at TIMESTAMPTZ,
    last_error_code VARCHAR(128),
    last_error_detail VARCHAR(2048),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_outbox_event_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_outbox_event_scope UNIQUE (
        tenant_id, organization_id, event_id
    ),
    CONSTRAINT uk_ai_agent_outbox_event_dedupe UNIQUE (
        tenant_id, organization_id, dedupe_key
    ),
    CONSTRAINT ck_ai_agent_outbox_event_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_outbox_event_attempts CHECK (
        attempt_count >= 0 AND max_attempts > 0 AND attempt_count <= max_attempts
    ),
    CONSTRAINT ck_ai_agent_outbox_event_payload CHECK (
        jsonb_typeof(payload_json) = 'object'
        AND jsonb_typeof(headers_json) = 'object'
    ),
    CONSTRAINT ck_ai_agent_outbox_event_lease CHECK (
        (lease_owner IS NULL AND lease_token IS NULL AND lease_expires_at IS NULL)
        OR (lease_owner IS NOT NULL AND lease_token IS NOT NULL AND lease_expires_at IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_outbox_event_fencing CHECK (fencing_token >= 0)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_outbox_event_worker
    ON ai_agent_outbox_event (status, available_at, lease_expires_at, id)
    WHERE status IN (0, 1, 3);
CREATE INDEX IF NOT EXISTS idx_ai_agent_outbox_event_retention
    ON ai_agent_outbox_event (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

-- ---------------------------------------------------------------------------
-- Session activity materialization
-- ---------------------------------------------------------------------------
-- `ai_agent_session.activity_at` is the materialized recency key for the
-- session-activity feed (DATABASE_SPEC §20.5 keyset ordering). The column is
-- maintained by triggers so every child write (Turn, Interaction, runtime
-- binding, user state) atomically refreshes the owning Session in the same
-- transaction as the child write; the feed query then orders and keysets on
-- an indexed column instead of per-row lateral scans.

CREATE OR REPLACE FUNCTION sdkwork_agents_session_activity_self()
RETURNS TRIGGER
LANGUAGE plpgsql
IMMUTABLE
AS $$
BEGIN
    NEW.activity_at := GREATEST(
        COALESCE(NEW.activity_at, NEW.updated_at),
        NEW.updated_at
    );
    RETURN NEW;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_ai_agent_session_activity_self') THEN
        CREATE TRIGGER trg_ai_agent_session_activity_self BEFORE INSERT OR UPDATE OF updated_at ON ai_agent_session
FOR EACH ROW EXECUTE FUNCTION sdkwork_agents_session_activity_self();
    END IF;
END $$;

CREATE OR REPLACE FUNCTION sdkwork_agents_bump_session_activity()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    event_at TIMESTAMPTZ;
BEGIN
    event_at := GREATEST(
        COALESCE(NEW.updated_at, NEW.created_at),
        NEW.created_at
    );
    UPDATE ai_agent_session
       SET activity_at = event_at
     WHERE tenant_id = NEW.tenant_id
       AND organization_id = NEW.organization_id
       AND session_id = NEW.session_id
       AND activity_at < event_at;
    RETURN NEW;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_ai_agent_turn_bump_session_activity') THEN
        CREATE TRIGGER trg_ai_agent_turn_bump_session_activity AFTER INSERT OR UPDATE OF updated_at, created_at ON ai_agent_turn
FOR EACH ROW EXECUTE FUNCTION sdkwork_agents_bump_session_activity();
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_ai_agent_interaction_bump_session_activity') THEN
        CREATE TRIGGER trg_ai_agent_interaction_bump_session_activity AFTER INSERT OR UPDATE OF updated_at, created_at ON ai_agent_interaction
FOR EACH ROW EXECUTE FUNCTION sdkwork_agents_bump_session_activity();
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_ai_agent_runtime_binding_bump_session_activity') THEN
        CREATE TRIGGER trg_ai_agent_runtime_binding_bump_session_activity AFTER INSERT OR UPDATE OF updated_at, created_at ON ai_agent_session_runtime_binding
FOR EACH ROW EXECUTE FUNCTION sdkwork_agents_bump_session_activity();
    END IF;
END $$;

-- User-state activity is session-scoped only (resource_type = 0); other
-- resource user states must not touch the Session feed.
CREATE OR REPLACE FUNCTION sdkwork_agents_bump_session_activity_from_user_state()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    event_at TIMESTAMPTZ;
BEGIN
    IF NEW.resource_type <> 0 THEN
        RETURN NEW;
    END IF;
    event_at := GREATEST(
        COALESCE(NEW.updated_at, NEW.created_at),
        NEW.created_at
    );
    UPDATE ai_agent_session
       SET activity_at = event_at
     WHERE tenant_id = NEW.tenant_id
       AND organization_id = NEW.organization_id
       AND session_id = NEW.resource_id
       AND activity_at < event_at;
    RETURN NEW;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_ai_agent_user_state_bump_session_activity') THEN
        CREATE TRIGGER trg_ai_agent_user_state_bump_session_activity AFTER INSERT OR UPDATE OF updated_at, created_at ON ai_agent_resource_user_state
FOR EACH ROW EXECUTE FUNCTION sdkwork_agents_bump_session_activity_from_user_state();
    END IF;
END $$;

-- Agent model configuration runtime profiles (server-authoritative
-- PostgreSQL persistence; DATABASE_SPEC: authoritative-server is PostgreSQL
-- only). Applied model configurations survive process restarts in the
-- canonical Agents database.
-- Every row is owner scoped (tenant/organization/owner user); HTTP access
-- always filters on these columns so profiles can never cross tenants.
CREATE TABLE IF NOT EXISTS ai_agent_model_configuration_profile (
    profile_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL DEFAULT 0,
    configuration_version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    configuration_json TEXT NOT NULL DEFAULT '{}',
    secret_bindings_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT ck_ai_agent_model_configuration_profile_version CHECK (version >= 0),
    CONSTRAINT ck_ai_agent_model_configuration_profile_status CHECK (
        status IN ('draft', 'active', 'archived')
    )
);

-- Upgrade compatibility: pre-scope baseline rows created before the scoped
-- columns were introduced are backfilled to the synthetic tenant 0 so the
-- baseline stays safe to re-run against an existing development schema.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.tables
        WHERE table_schema = current_schema()
          AND table_name = 'ai_agent_model_configuration_profile'
    ) THEN
        ALTER TABLE ai_agent_model_configuration_profile
            ADD COLUMN IF NOT EXISTS tenant_id BIGINT NOT NULL DEFAULT 0,
            ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0,
            ADD COLUMN IF NOT EXISTS owner_user_id BIGINT NOT NULL DEFAULT 0;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_ai_agent_model_configuration_profile_scope
    ON ai_agent_model_configuration_profile (tenant_id, organization_id, agent_id, status);

DROP INDEX IF EXISTS idx_ai_agent_model_configuration_profile_agent;

-- Agent media tool per-tenant configuration (admin-managed: enabled state,
-- default save-to-drive behaviour, and default arguments merged at invoke).
CREATE TABLE IF NOT EXISTS ai_agent_tool_configuration (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    tool_id VARCHAR(160) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    save_to_drive_default BOOLEAN NOT NULL DEFAULT FALSE,
    default_arguments_json TEXT NOT NULL DEFAULT '{}',
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL DEFAULT 0,
    updated_by BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_tool_configuration_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_tool_configuration_scope UNIQUE (
        tenant_id, organization_id, tool_id
    ),
    CONSTRAINT ck_ai_agent_tool_configuration_version CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_tool_configuration_tenant
    ON ai_agent_tool_configuration (tenant_id, organization_id, tool_id);

-- Generated media assets persisted to Drive outside a session-item context
-- (direct tool invocation with saveToDrive). Independent of session items so
-- front-end-driven generation saves are registered as assets even without a
-- turn; the drive asset centre (app /assets) remains the storage authority.
CREATE TABLE IF NOT EXISTS ai_agent_tool_asset (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    user_id BIGINT NOT NULL DEFAULT 0,
    tool_id VARCHAR(160) NOT NULL,
    tool_call_id VARCHAR(128) NOT NULL,
    media_kind VARCHAR(64) NOT NULL,
    drive_space_id VARCHAR(128) NOT NULL,
    drive_node_id VARCHAR(128) NOT NULL,
    drive_uri VARCHAR(512) NOT NULL,
    source_url TEXT,
    created_by BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_tool_asset_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_tool_asset_node UNIQUE (
        tenant_id, organization_id, drive_space_id, drive_node_id
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_tool_asset_tenant_user
    ON ai_agent_tool_asset (tenant_id, organization_id, user_id, created_at DESC);
