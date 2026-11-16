-- sdkwork:migration
-- id: 0008_agent_kind_distribution_constraints
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Remove the delivery invariant constraints. The rows themselves are
--   unchanged; only the enforcement is removed.
-- reversible: true
-- rollback: forward-fix
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_local_harness_runtime;
ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_archive_distribution;
ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS ck_ai_agent_kind_distribution;

COMMIT;
