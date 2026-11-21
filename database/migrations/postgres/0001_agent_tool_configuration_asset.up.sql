-- sdkwork:migration
-- id: 0001_agent_tool_configuration_asset
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Materialize the agent media tool per-tenant configuration and the
--   generated media asset tables (ai_agent_tool_configuration,
--   ai_agent_tool_asset). These tables were added to the agents baseline after
--   existing deployments were bootstrapped, so previously initialized
--   databases never received the DDL and the agents schema drift gate failed
--   with missing tables. Fresh baseline installs (which already contain the
--   same DDL) no-op here via IF NOT EXISTS.
-- reversible: false
-- rollback: forward-fix (dropping the tables would discard per-tenant tool
--   configuration and generated media asset rows)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s
-- contract_version: 7.2.0

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
