-- sdkwork:migration
-- id: 0004_agent_allowed_role_codes
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Preserve agent role-assignment data on reversal. Dropping the
--   allowed_role_codes column would destroy every tenant's agent visibility
--   policy, so this down migration deliberately performs no DDL.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

-- Intentionally no-op: allowed_role_codes and its index/constraint are retained.
SELECT 1;

COMMIT;
