-- sdkwork:migration
-- id: 0010_expert_catalog
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Materialize the SDKWork expert catalog's team and persona aggregates
--   (`ai_expert_team`, `ai_expert_team_member`, `ai_expert_persona`).
--
--   An expert is an ordinary managed agent (`ai_agent` row) grouped by an
--   `ai_agent_category` row; this migration adds the two aggregates that agents
--   cannot express: a named expert team with ordered members, and the localized
--   persona body an expert is summoned with. The expert catalog data is
--   initialized from `sdkwork-specs/specs/expert-catalog` (see
--   `sdkwork-specs/EXPERTS_CATALOG_SPEC.md`); these tables are its persistence.
--
--   Members reference an expert by the stable `agent_id` string rather than a
--   cross-table foreign key: `ai_agent` is written by the same service, but the
--   catalog may be seeded before the agent rows and the reference must stay a
--   stable business key like every other sibling reference in this schema.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

CREATE TABLE IF NOT EXISTS ai_expert_team (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    team_id VARCHAR(128) NOT NULL,
    name VARCHAR(255) NOT NULL,
    name_en VARCHAR(255) NOT NULL,
    description VARCHAR(1000) NOT NULL,
    description_en VARCHAR(1000) NOT NULL,
    tags_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    tags_en_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    goal TEXT NOT NULL,
    goal_en TEXT NOT NULL,
    constraints_text TEXT NOT NULL,
    delivery_requirements VARCHAR(1000) NOT NULL,
    delivery_requirements_en VARCHAR(1000) NOT NULL,
    examples_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    examples_en_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    coordinator_mode VARCHAR(16) NOT NULL,
    coordinator_template_id VARCHAR(32) NOT NULL,
    coordinator_template_version INTEGER NOT NULL DEFAULT 1,
    status SMALLINT NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL DEFAULT 0,
    updated_by BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    CONSTRAINT uk_ai_expert_team_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_expert_team_scope UNIQUE (tenant_id, organization_id, team_id),
    CONSTRAINT uk_ai_expert_team_scope_id UNIQUE (tenant_id, organization_id, id),
    CONSTRAINT ck_ai_expert_team_id CHECK (
        team_id ~ '^team-[a-z][a-z0-9-]{0,63}$'
    ),
    CONSTRAINT ck_ai_expert_team_name CHECK (
        char_length(BTRIM(name)) > 0 AND char_length(name) <= 255
        AND char_length(BTRIM(name_en)) > 0 AND char_length(name_en) <= 255
    ),
    CONSTRAINT ck_ai_expert_team_description CHECK (
        char_length(BTRIM(description)) > 0 AND char_length(description) <= 1000
        AND char_length(BTRIM(description_en)) > 0 AND char_length(description_en) <= 1000
    ),
    CONSTRAINT ck_ai_expert_team_tags_json CHECK (jsonb_typeof(tags_json) = 'array'),
    CONSTRAINT ck_ai_expert_team_tags_en_json CHECK (jsonb_typeof(tags_en_json) = 'array'),
    CONSTRAINT ck_ai_expert_team_examples_json CHECK (jsonb_typeof(examples_json) = 'array'),
    CONSTRAINT ck_ai_expert_team_examples_en_json CHECK (jsonb_typeof(examples_en_json) = 'array'),
    CONSTRAINT ck_ai_expert_team_goal CHECK (
        char_length(BTRIM(goal)) > 0 AND char_length(goal) <= 4000
        AND char_length(BTRIM(goal_en)) > 0 AND char_length(goal_en) <= 4000
    ),
    CONSTRAINT ck_ai_expert_team_constraints CHECK (
        char_length(constraints_text) <= 4000
    ),
    CONSTRAINT ck_ai_expert_team_delivery CHECK (
        char_length(BTRIM(delivery_requirements)) > 0
        AND char_length(delivery_requirements) <= 1000
        AND char_length(BTRIM(delivery_requirements_en)) > 0
        AND char_length(delivery_requirements_en) <= 1000
    ),
    CONSTRAINT ck_ai_expert_team_coordinator_mode CHECK (
        coordinator_mode IN ('template', 'custom')
    ),
    CONSTRAINT ck_ai_expert_team_coordinator_template CHECK (
        coordinator_template_id IN ('general', 'product', 'technical', 'content', 'data', 'research')
    ),
    CONSTRAINT ck_ai_expert_team_coordinator_version CHECK (
        coordinator_template_version >= 1
    ),
    CONSTRAINT ck_ai_expert_team_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_expert_team_sort_order CHECK (sort_order >= 0 AND sort_order <= 9999),
    CONSTRAINT ck_ai_expert_team_version CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS idx_ai_expert_team_tenant_org_status
    ON ai_expert_team (tenant_id, organization_id, status, sort_order, id)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_expert_team_member (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    team_id VARCHAR(128) NOT NULL,
    expert_agent_id VARCHAR(128) NOT NULL,
    position INTEGER NOT NULL,
    duty VARCHAR(255) NOT NULL,
    duty_en VARCHAR(255) NOT NULL,
    instructions TEXT NOT NULL,
    instructions_en TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_expert_team_member_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_expert_team_member_expert UNIQUE (
        tenant_id, organization_id, team_id, expert_agent_id
    ),
    CONSTRAINT uk_ai_expert_team_member_position UNIQUE (
        tenant_id, organization_id, team_id, position
    ),
    CONSTRAINT ck_ai_expert_team_member_expert_id CHECK (
        expert_agent_id ~ '^agent\.expert\.[a-z0-9][a-z0-9.-]{0,120}$'
    ),
    CONSTRAINT ck_ai_expert_team_member_position CHECK (
        position >= 0 AND position <= 63
    ),
    CONSTRAINT ck_ai_expert_team_member_duty CHECK (
        char_length(BTRIM(duty)) > 0 AND char_length(duty) <= 255
        AND char_length(BTRIM(duty_en)) > 0 AND char_length(duty_en) <= 255
    ),
    CONSTRAINT ck_ai_expert_team_member_instructions CHECK (
        char_length(BTRIM(instructions)) > 0 AND char_length(instructions) <= 8000
        AND char_length(BTRIM(instructions_en)) > 0 AND char_length(instructions_en) <= 8000
    ),
    CONSTRAINT fk_ai_expert_team_member_team FOREIGN KEY (
        tenant_id, organization_id, team_id
    ) REFERENCES ai_expert_team (
        tenant_id, organization_id, team_id
    ) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_expert_team_member_team_order
    ON ai_expert_team_member (tenant_id, organization_id, team_id, position, id)
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_expert_team_member_expert
    ON ai_expert_team_member (tenant_id, organization_id, expert_agent_id)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_expert_persona (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    organization_id BIGINT NOT NULL DEFAULT 0,
    expert_agent_id VARCHAR(128) NOT NULL,
    locale VARCHAR(16) NOT NULL,
    persona_text TEXT NOT NULL,
    content_sha256 CHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_expert_persona_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_expert_persona_locale UNIQUE (
        tenant_id, organization_id, expert_agent_id, locale
    ),
    CONSTRAINT ck_ai_expert_persona_expert_id CHECK (
        expert_agent_id ~ '^agent\.expert\.[a-z0-9][a-z0-9.-]{0,120}$'
    ),
    CONSTRAINT ck_ai_expert_persona_locale CHECK (
        locale ~ '^[a-z]{2}(-[A-Z]{2})?$'
    ),
    CONSTRAINT ck_ai_expert_persona_text CHECK (
        char_length(BTRIM(persona_text)) > 0 AND char_length(persona_text) <= 65536
    ),
    CONSTRAINT ck_ai_expert_persona_sha CHECK (
        content_sha256 ~ '^[0-9a-f]{64}$'
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_expert_persona_expert
    ON ai_expert_persona (tenant_id, organization_id, expert_agent_id, locale);

COMMIT;
