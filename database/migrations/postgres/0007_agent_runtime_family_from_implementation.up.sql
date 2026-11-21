-- sdkwork:migration
-- id: 0007_agent_runtime_family_from_implementation
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Make `ai_agent.runtime_family` an honest projection. The earlier
--   projection defaulted every manifest without a runtime descriptor to
--   `rig`, which mislabels legacy `sdkwork-native` rows (for example
--   `agent.chat.default`) as Rig-backed. The column now derives `rig` only
--   when the implementation type is `rig-rust`; otherwise it stays NULL until
--   the manifest declares a runtime family. Seeded catalog agents are
--   unaffected because every seeded manifest declares its runtime.
-- reversible: true
-- rollback: down-migration (restores the 0006 expression)
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
            COALESCE(
                NULLIF(BTRIM(manifest_json #>> '{runtime,family}'), ''),
                CASE WHEN implementation_type = 'rig-rust' THEN 'rig' ELSE NULL END
            )
        ) STORED;

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_runtime
    ON ai_agent (tenant_id, organization_id, runtime_family, status)
    WHERE deleted_at IS NULL;

COMMIT;
