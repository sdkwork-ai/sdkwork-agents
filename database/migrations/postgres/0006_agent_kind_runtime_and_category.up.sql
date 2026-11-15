-- sdkwork:migration
-- id: 0006_agent_kind_runtime_and_category
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Materialize the agent delivery shape (`agent_kind`), runtime family,
--   professional category, and archive distribution as manifest-derived
--   projection columns on ai_agent. The agent manifest is the authored
--   authority; these columns are STORED generated projections so every writer
--   (seed, console create, admin update) keeps them consistent without a
--   second write path, while list filters and drift verification can index
--   them. Existing rows are backfilled by PostgreSQL when the columns are
--   added because the expression is evaluated from the stored manifest.
--
--   Values follow `specs/AGENTS_PROVIDER_TAXONOMY_SPEC.md` (runtime families,
--   Rig default) and `specs/AGENTS_AGENT_KIND_AND_RUNTIME_SPEC.md` (kinds and
--   archive distribution contract).
-- reversible: true
-- rollback: down-migration (drops derived columns only; the manifest keeps the
--   authoritative values and a re-apply recomputes them)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE ai_agent
    ADD COLUMN IF NOT EXISTS agent_kind VARCHAR(32)
        GENERATED ALWAYS AS (
            COALESCE(NULLIF(BTRIM(manifest_json ->> 'agent_kind'), ''), 'conversation')
        ) STORED;

ALTER TABLE ai_agent
    ADD COLUMN IF NOT EXISTS runtime_family VARCHAR(64)
        GENERATED ALWAYS AS (
            COALESCE(NULLIF(BTRIM(manifest_json #>> '{runtime,family}'), ''), 'rig')
        ) STORED;

ALTER TABLE ai_agent
    ADD COLUMN IF NOT EXISTS category_code VARCHAR(128)
        GENERATED ALWAYS AS (
            NULLIF(BTRIM(manifest_json ->> 'category_code'), '')
        ) STORED;

ALTER TABLE ai_agent
    ADD COLUMN IF NOT EXISTS distribution_kind VARCHAR(32)
        GENERATED ALWAYS AS (
            COALESCE(NULLIF(BTRIM(manifest_json #>> '{distribution,kind}'), ''), 'manifest')
        ) STORED;

ALTER TABLE ai_agent
    ADD COLUMN IF NOT EXISTS distribution_json JSONB
        GENERATED ALWAYS AS (
            CASE
                WHEN jsonb_typeof(manifest_json -> 'distribution') = 'object'
                    THEN manifest_json -> 'distribution'
                ELSE NULL
            END
        ) STORED;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_agent_kind'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_agent_kind
            CHECK (agent_kind IN ('conversation', 'local-execution'));
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_runtime_family'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_runtime_family
            CHECK (runtime_family ~ '^[a-z][a-z0-9_-]{1,63}$');
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_category_code'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_category_code
            CHECK (
                category_code IS NULL
                OR category_code ~ '^[a-z0-9][a-z0-9_-]{1,127}$'
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_distribution_kind'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_distribution_kind
            CHECK (distribution_kind IN ('manifest', 'archive'));
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_distribution_json'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_distribution_json
            CHECK (
                distribution_json IS NULL
                OR jsonb_typeof(distribution_json) = 'object'
            );
    END IF;
END
$$;

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_kind_status
    ON ai_agent (tenant_id, organization_id, agent_kind, status, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_category
    ON ai_agent (tenant_id, organization_id, category_code, status, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_runtime
    ON ai_agent (tenant_id, organization_id, runtime_family, status)
    WHERE deleted_at IS NULL;

COMMIT;
