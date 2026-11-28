-- sdkwork:migration
-- id: 0012_agent_category_taxonomy
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Align agent categories with connector/skill/plugin category
--   management: free-form tags and attributes alongside the existing
--   parent/sort/icon/lifecycle columns. Existing rows receive empty metadata.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE ai_agent_category
    ADD COLUMN IF NOT EXISTS tags_json JSONB NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE ai_agent_category
    ADD COLUMN IF NOT EXISTS attributes_json JSONB NOT NULL DEFAULT '{}'::jsonb;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'ck_ai_agent_category_tags'
          AND conrelid = 'ai_agent_category'::regclass
    ) THEN
        ALTER TABLE ai_agent_category
            ADD CONSTRAINT ck_ai_agent_category_tags
            CHECK (jsonb_typeof(tags_json) = 'array');
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'ck_ai_agent_category_attributes'
          AND conrelid = 'ai_agent_category'::regclass
    ) THEN
        ALTER TABLE ai_agent_category
            ADD CONSTRAINT ck_ai_agent_category_attributes
            CHECK (jsonb_typeof(attributes_json) = 'object');
    END IF;
END
$$;

COMMIT;
