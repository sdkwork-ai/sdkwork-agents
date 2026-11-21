-- sdkwork:migration
-- id: 0006_agent_kind_runtime_and_category
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Reverse the manifest-derived projection columns. The values stay
--   authoritative in `ai_agent.manifest_json`, so dropping the derived columns
--   loses no authored data and a later up-migration recomputes them.
-- reversible: true
-- rollback: forward-fix
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

DROP INDEX IF EXISTS idx_ai_agent_tenant_runtime;
DROP INDEX IF EXISTS idx_ai_agent_tenant_category;
DROP INDEX IF EXISTS idx_ai_agent_tenant_kind_status;

ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_distribution_json;
ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_distribution_kind;
ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_category_code;
ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_runtime_family;
ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_agent_kind;

ALTER TABLE ai_agent DROP COLUMN IF EXISTS distribution_json;
ALTER TABLE ai_agent DROP COLUMN IF EXISTS distribution_kind;
ALTER TABLE ai_agent DROP COLUMN IF EXISTS category_code;
ALTER TABLE ai_agent DROP COLUMN IF EXISTS runtime_family;
ALTER TABLE ai_agent DROP COLUMN IF EXISTS agent_kind;

COMMIT;
