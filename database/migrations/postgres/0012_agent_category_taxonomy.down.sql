-- sdkwork:migration
-- id: 0012_agent_category_taxonomy
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Preserve operator-curated category tags and attributes on reversal.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

-- Intentionally no-op: tags_json and attributes_json are retained.
SELECT 1;

COMMIT;
