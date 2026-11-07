-- sdkwork:migration
-- id: 0005_agent_category
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Revert the managed agent category catalog created by
--   0005_agent_category.up.sql. Dropping the catalog discards operator-authored
--   category metadata only; agent records keep their `categoryId` string.
-- reversible: true
-- rollback: forward-fix
-- transactional: true

BEGIN;

DROP INDEX IF EXISTS idx_ai_agent_category_tenant_org_status;
DROP TABLE IF EXISTS ai_agent_category;

COMMIT;
