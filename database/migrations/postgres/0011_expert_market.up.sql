-- sdkwork:migration
-- id: 0011_expert_market
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Materialize the expert market's editorial layer: the per-expert
--   display profile (`ai_expert_profile`) and the curated expert collections
--   (`ai_expert_collection`, `ai_expert_collection_item`).
--
--   An expert is an ordinary managed agent (`ai_agent` row) grouped by an
--   `ai_agent_category` row, so its identity, name and persona live on those
--   aggregates. The market additionally needs editorial data that neither the
--   agent manifest nor the persona body can express: a curated usage statistic
--   (an editorial figure, never telemetry), a short badge code, the catalog
--   order inside the division, and the publish timestamp used for ordering.
--   Collections are the market's featured shelf: a named, bilingual,
--   cover-keyed set of ordered experts with a bilingual per-collection label.
--
--   Member references use the stable `expert_agent_id` business key
--   (`agent.expert.<slug>`) rather than a cross-table foreign key:
--   `ai_agent` is written by the same service, but the catalog may be seeded
--   before the agent rows and the reference must stay a stable business key
--   like every other sibling reference in this schema. Collection items do
--   reference their collection because both rows are owned by this migration
--   and a collection cannot outlive its shelf.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

CREATE TABLE IF NOT EXISTS ai_expert_profile (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    expert_agent_id VARCHAR(128) NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    badge_code VARCHAR(32),
    sort_order INTEGER NOT NULL DEFAULT 0,
    published_at TIMESTAMPTZ,
    created_by BIGINT NOT NULL DEFAULT 0,
    updated_by BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    CONSTRAINT uk_ai_expert_profile_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_expert_profile_expert UNIQUE (
        tenant_id, organization_id, expert_agent_id
    ),
    CONSTRAINT ck_ai_expert_profile_expert_id CHECK (
        expert_agent_id ~ '^agent\.expert\.[a-z0-9][a-z0-9.-]{0,120}$'
    ),
    CONSTRAINT ck_ai_expert_profile_usage_count CHECK (usage_count >= 0),
    CONSTRAINT ck_ai_expert_profile_badge_code CHECK (
        badge_code IS NULL OR badge_code ~ '^[a-z][a-z0-9-]{1,31}$'
    ),
    CONSTRAINT ck_ai_expert_profile_sort_order CHECK (
        sort_order >= 0 AND sort_order <= 9999
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_expert_profile_scope_order
    ON ai_expert_profile (tenant_id, organization_id, sort_order, expert_agent_id)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_expert_collection (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    collection_code VARCHAR(128) NOT NULL,
    name VARCHAR(255) NOT NULL,
    name_en VARCHAR(255) NOT NULL,
    description VARCHAR(1000) NOT NULL,
    description_en VARCHAR(1000) NOT NULL,
    cover_key VARCHAR(64) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL DEFAULT 0,
    updated_by BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    CONSTRAINT uk_ai_expert_collection_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_expert_collection_scope UNIQUE (
        tenant_id, organization_id, collection_code
    ),
    CONSTRAINT uk_ai_expert_collection_scope_id UNIQUE (tenant_id, organization_id, id),
    CONSTRAINT ck_ai_expert_collection_code CHECK (
        collection_code ~ '^collection-[a-z][a-z0-9-]{0,63}$'
    ),
    CONSTRAINT ck_ai_expert_collection_name CHECK (
        char_length(BTRIM(name)) > 0 AND char_length(name) <= 255
        AND char_length(BTRIM(name_en)) > 0 AND char_length(name_en) <= 255
    ),
    CONSTRAINT ck_ai_expert_collection_description CHECK (
        char_length(BTRIM(description)) > 0 AND char_length(description) <= 1000
        AND char_length(BTRIM(description_en)) > 0 AND char_length(description_en) <= 1000
    ),
    CONSTRAINT ck_ai_expert_collection_cover_key CHECK (
        cover_key ~ '^[a-z][a-z0-9-]{0,63}$'
    ),
    CONSTRAINT ck_ai_expert_collection_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_expert_collection_sort_order CHECK (
        sort_order >= 0 AND sort_order <= 9999
    ),
    CONSTRAINT ck_ai_expert_collection_version CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS idx_ai_expert_collection_tenant_org_status
    ON ai_expert_collection (tenant_id, organization_id, status, sort_order, id)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_expert_collection_item (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    collection_code VARCHAR(128) NOT NULL,
    expert_agent_id VARCHAR(128) NOT NULL,
    position INTEGER NOT NULL,
    label VARCHAR(255) NOT NULL,
    label_en VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_expert_collection_item_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_expert_collection_item_expert UNIQUE (
        tenant_id, organization_id, collection_code, expert_agent_id
    ),
    CONSTRAINT uk_ai_expert_collection_item_position UNIQUE (
        tenant_id, organization_id, collection_code, position
    ),
    CONSTRAINT ck_ai_expert_collection_item_expert_id CHECK (
        expert_agent_id ~ '^agent\.expert\.[a-z0-9][a-z0-9.-]{0,120}$'
    ),
    CONSTRAINT ck_ai_expert_collection_item_position CHECK (
        position >= 0 AND position <= 63
    ),
    CONSTRAINT ck_ai_expert_collection_item_label CHECK (
        char_length(BTRIM(label)) > 0 AND char_length(label) <= 255
        AND char_length(BTRIM(label_en)) > 0 AND char_length(label_en) <= 255
    ),
    CONSTRAINT fk_ai_expert_collection_item_collection FOREIGN KEY (
        tenant_id, organization_id, collection_code
    ) REFERENCES ai_expert_collection (
        tenant_id, organization_id, collection_code
    ) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_expert_collection_item_collection_order
    ON ai_expert_collection_item (tenant_id, organization_id, collection_code, position, id)
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_expert_collection_item_expert
    ON ai_expert_collection_item (tenant_id, organization_id, expert_agent_id)
    WHERE deleted_at IS NULL;

COMMIT;
