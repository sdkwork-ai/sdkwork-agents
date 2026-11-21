-- sdkwork:migration
-- id: 0005_agent_category
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Materialize the managed agent category catalog
--   (ai_agent_category). Agents reference a category by `categoryCode` in
--   their management profile; this table is the administrative source of
--   truth for the category list managed from the admin console. The catalog is
--   a tenant-scoped, at-most-two-level taxonomy with an optional parent
--   (same-tenant composite foreign key), an optional icon reference, an
--   explicit lifecycle status and optimistic-concurrency version.
--   The table was added to the agents baseline after existing deployments were
--   bootstrapped, so previously initialized databases never received the DDL
--   and the agents schema drift gate fails with a missing table. Fresh
--   baseline installs (which already contain the same DDL) no-op here via
--   IF NOT EXISTS.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

CREATE TABLE IF NOT EXISTS ai_agent_category (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    category_code VARCHAR(128) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id BIGINT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    icon_ref VARCHAR(512),
    status SMALLINT NOT NULL DEFAULT 1,
    created_by BIGINT NOT NULL DEFAULT 0,
    updated_by BIGINT NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    CONSTRAINT uk_ai_agent_category_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_category_scope UNIQUE (
        tenant_id, organization_id, category_code
    ),
    CONSTRAINT uk_ai_agent_category_scope_id UNIQUE (
        tenant_id, organization_id, id
    ),
    CONSTRAINT ck_ai_agent_category_code CHECK (
        category_code ~ '^[a-z0-9][a-z0-9_-]{1,127}$'
    ),
    CONSTRAINT ck_ai_agent_category_name CHECK (
        char_length(BTRIM(name)) > 0 AND char_length(name) <= 255
    ),
    CONSTRAINT ck_ai_agent_category_description CHECK (
        description IS NULL OR char_length(description) <= 2000
    ),
    CONSTRAINT ck_ai_agent_category_icon_ref CHECK (
        icon_ref IS NULL
        OR (char_length(BTRIM(icon_ref)) > 0 AND char_length(icon_ref) <= 512)
    ),
    CONSTRAINT ck_ai_agent_category_sort_order CHECK (
        sort_order >= 0 AND sort_order <= 9999
    ),
    CONSTRAINT ck_ai_agent_category_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_category_parent_self CHECK (
        parent_id IS NULL OR parent_id <> id
    ),
    CONSTRAINT ck_ai_agent_category_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_category_parent FOREIGN KEY (
        tenant_id, organization_id, parent_id
    ) REFERENCES ai_agent_category (
        tenant_id, organization_id, id
    ) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_category_tenant_org_status
    ON ai_agent_category (tenant_id, organization_id, status, sort_order, id)
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_category_tenant_org_parent
    ON ai_agent_category (tenant_id, organization_id, parent_id, sort_order, id)
    WHERE deleted_at IS NULL;

COMMIT;
