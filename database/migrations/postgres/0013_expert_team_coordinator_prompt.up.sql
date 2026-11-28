-- sdkwork:migration
-- id: 0013_expert_team_coordinator_prompt
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Close the gap between `ai_expert_team` and the expert package's team
--   contract, so every catalog team is representable and summonable.
--
--   1. A `coordinator_mode = 'custom'` team needs its own coordinator prompt,
--      but the table had no column to hold one: a custom team could be recorded
--      and then never reach the app that summons it. `coordinator_prompt` adds
--      that storage and requires it for custom coordinators.
--   2. `constraints_text` accepted 4000 characters while the package caps a
--      team's shared constraints at 2000, so an over-long row would be seeded
--      successfully and then rejected at ingest. The bound now matches the
--      consumer contract.
--
--   Template coordinators keep an empty prompt: their text comes from the
--   catalogued template the package resolves from `coordinator_template_id`.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE ai_expert_team
    ADD COLUMN IF NOT EXISTS coordinator_prompt TEXT NOT NULL DEFAULT '';

ALTER TABLE ai_expert_team
    DROP CONSTRAINT IF EXISTS ck_ai_expert_team_constraints;

ALTER TABLE ai_expert_team
    DROP CONSTRAINT IF EXISTS ck_ai_expert_team_coordinator_prompt;

ALTER TABLE ai_expert_team
    ADD CONSTRAINT ck_ai_expert_team_constraints CHECK (
        char_length(constraints_text) <= 2000
    );

ALTER TABLE ai_expert_team
    ADD CONSTRAINT ck_ai_expert_team_coordinator_prompt CHECK (
        char_length(coordinator_prompt) <= 12000
        AND (
            coordinator_mode <> 'custom'
            OR char_length(BTRIM(coordinator_prompt)) > 0
        )
    );

COMMIT;
