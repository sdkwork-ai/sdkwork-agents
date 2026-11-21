-- sdkwork:migration
-- id: 0003_agent_execution_host_and_session_placement
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Materialize the execution host registry and the durable session
--   execution placement tables (ai_agent_execution_host,
--   ai_agent_session_execution_placement). These tables were added to the
--   agents baseline after existing deployments were bootstrapped, so
--   previously initialized databases never received the DDL and the agents
--   schema drift gate fails with missing tables. Fresh baseline installs
--   (which already contain the same DDL) no-op here via IF NOT EXISTS.
-- reversible: false
-- rollback: forward-fix (dropping the tables would discard execution host
--   registrations and durable session placement facts; placement state is
--   reconciled by the placement lifecycle, never rewound)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

CREATE TABLE IF NOT EXISTS ai_agent_execution_host (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    host_id VARCHAR(128) NOT NULL,
    display_name VARCHAR(256),
    host_kind VARCHAR(32) NOT NULL,
    endpoint VARCHAR(512) NOT NULL,
    region VARCHAR(64),
    max_concurrent_sessions INTEGER NOT NULL DEFAULT 1,
    capabilities_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    status SMALLINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL DEFAULT 0,
    updated_by BIGINT NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    CONSTRAINT uk_ai_agent_execution_host_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_execution_host_scope UNIQUE (tenant_id, organization_id, host_id),
    CONSTRAINT ck_ai_agent_execution_host_kind CHECK (
        host_kind IN ('docker', 'micro_vm', 'bare_metal', 'cloud_sandbox')
    ),
    CONSTRAINT ck_ai_agent_execution_host_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_execution_host_capacity CHECK (max_concurrent_sessions > 0),
    CONSTRAINT ck_ai_agent_execution_host_endpoint CHECK (char_length(BTRIM(endpoint)) > 0),
    CONSTRAINT ck_ai_agent_execution_host_version CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_execution_host_eligible
    ON ai_agent_execution_host (
        tenant_id, organization_id, host_kind, status, max_concurrent_sessions DESC, id
    )
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_session_execution_placement (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    placement_id VARCHAR(128) NOT NULL,
    execution_kind VARCHAR(32) NOT NULL,
    execution_id VARCHAR(128) NOT NULL,
    requested_target VARCHAR(16),
    effective_target VARCHAR(16) NOT NULL,
    host_id VARCHAR(128),
    host_kind VARCHAR(32),
    kernel_placement_ref VARCHAR(256),
    placement_state SMALLINT NOT NULL DEFAULT 0,
    lease_owner VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    fencing_generation BIGINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    activated_at TIMESTAMPTZ,
    deactivated_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_session_execution_placement_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_session_execution_placement_scope UNIQUE (
        tenant_id, organization_id, session_id, placement_id
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_execution_kind CHECK (
        execution_kind IN ('turn', 'task_run_attempt')
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_requested_target CHECK (
        requested_target IS NULL
        OR requested_target IN ('in_process', 'cloud', 'host')
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_effective_target CHECK (
        effective_target IN ('in_process', 'cloud', 'host')
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_host CHECK (
        (effective_target = 'host' AND host_id IS NOT NULL AND host_kind IS NOT NULL)
        OR (effective_target <> 'host' AND host_id IS NULL AND host_kind IS NULL)
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_state CHECK (
        placement_state IN (0, 1, 2, 3, 4, 5, 6, 7)
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_lease CHECK (
        (lease_owner IS NULL AND lease_expires_at IS NULL)
        OR (lease_owner IS NOT NULL AND lease_expires_at IS NOT NULL)
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_fencing CHECK (fencing_generation >= 0),
    CONSTRAINT ck_ai_agent_session_execution_placement_status CHECK (status IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_session_execution_placement_current CHECK (
        (is_current = TRUE AND status = 0 AND deactivated_at IS NULL)
        OR (is_current = FALSE AND status <> 0)
    ),
    CONSTRAINT ck_ai_agent_session_execution_placement_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_session_execution_placement_session FOREIGN KEY (
        tenant_id, organization_id, session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
        ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_session_execution_placement_current
    ON ai_agent_session_execution_placement (tenant_id, organization_id, session_id)
    WHERE is_current = TRUE;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_execution_placement_list
    ON ai_agent_session_execution_placement (
        tenant_id, organization_id, session_id, is_current DESC, updated_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_execution_placement_host
    ON ai_agent_session_execution_placement (
        tenant_id, organization_id, host_id, placement_state, updated_at DESC, id DESC
    )
    WHERE is_current = TRUE AND host_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_execution_placement_reconcile
    ON ai_agent_session_execution_placement (tenant_id, placement_state, lease_expires_at, id)
    WHERE placement_state IN (0, 1, 2, 3);

COMMIT;
