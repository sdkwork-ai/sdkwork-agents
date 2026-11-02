-- sdkwork:migration
-- id: 0004_agent_allowed_role_codes
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Assign agents to IAM roles. `allowed_role_codes` holds the role
--   codes allowed to see and run an agent; an empty array (the default) means
--   every role, which keeps existing rows visible to all.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE ai_agent
    ADD COLUMN IF NOT EXISTS allowed_role_codes JSONB NOT NULL DEFAULT '[]'::jsonb;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'ck_ai_agent_allowed_roles_json'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_allowed_roles_json
            CHECK (jsonb_typeof(allowed_role_codes) = 'array');
    END IF;
END
$$;

UPDATE ai_agent
SET allowed_role_codes = '[]'::jsonb
WHERE allowed_role_codes IS NULL OR jsonb_typeof(allowed_role_codes) <> 'array';

CREATE INDEX IF NOT EXISTS idx_ai_agent_allowed_roles
    ON ai_agent USING GIN (allowed_role_codes)
    WHERE deleted_at IS NULL;

COMMIT;
