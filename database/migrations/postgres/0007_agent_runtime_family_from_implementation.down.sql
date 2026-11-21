-- sdkwork:migration
-- id: 0007_agent_runtime_family_from_implementation
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Restore the 0006 projection expression (default `rig`) for
--   reversal. Values remain derived from the stored manifest and
--   implementation type, so no authored data is dropped.
-- reversible: true
-- rollback: forward-fix
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

DROP INDEX IF EXISTS idx_ai_agent_tenant_runtime;

ALTER TABLE ai_agent DROP COLUMN IF EXISTS runtime_family;

ALTER TABLE ai_agent
    ADD COLUMN runtime_family VARCHAR(64)
        GENERATED ALWAYS AS (
            COALESCE(NULLIF(BTRIM(manifest_json #>> '{runtime,family}'), ''), 'rig')
        ) STORED;

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_runtime
    ON ai_agent (tenant_id, organization_id, runtime_family, status)
    WHERE deleted_at IS NULL;

COMMIT;
