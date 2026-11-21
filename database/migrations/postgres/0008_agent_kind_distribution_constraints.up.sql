-- sdkwork:migration
-- id: 0008_agent_kind_distribution_constraints
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Enforce the agent delivery invariants at the database boundary.
--   A local-execution agent must carry a zip archive distribution and the
--   harness runtime protocol; a conversation agent must not carry an archive;
--   an archive descriptor must be bounded in size and name its entrypoint plus
--   its archive reference or uploaded Drive locator. The constraints read the
--   manifest-derived projection columns, so they apply to every writer without
--   a second validation path.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_kind_distribution'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_kind_distribution
            CHECK (
                (agent_kind = 'local-execution' AND distribution_kind = 'archive')
                OR (agent_kind = 'conversation' AND distribution_kind = 'manifest')
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_archive_distribution'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_archive_distribution
            CHECK (
                distribution_kind <> 'archive'
                OR (
                    distribution_json->>'format' = 'zip'
                    AND NULLIF(BTRIM(distribution_json->>'entrypoint'), '') IS NOT NULL
                    AND (
                        NULLIF(BTRIM(distribution_json->>'archiveRef'), '') IS NOT NULL
                        OR NULLIF(BTRIM(distribution_json->>'driveUri'), '') IS NOT NULL
                    )
                    AND octet_length(distribution_json::text) <= 16384
                )
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_ai_agent_local_harness_runtime'
          AND conrelid = 'ai_agent'::regclass
    ) THEN
        ALTER TABLE ai_agent
            ADD CONSTRAINT ck_ai_agent_local_harness_runtime
            CHECK (
                agent_kind <> 'local-execution'
                OR (
                    runtime_family = 'harness'
                    AND manifest_json #>> '{runtime,protocol}' = 'harness.agent.runtime.v1'
                )
            );
    END IF;
END
$$;

COMMIT;
