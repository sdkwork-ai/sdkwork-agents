-- sdkwork:migration
-- id: 0013_expert_team_coordinator_prompt
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Restore the pre-0012 team definition: drop the custom coordinator
--   prompt storage and relax the shared-constraints bound back to 4000.
-- reversible: false
-- rollback: forward-only
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE ai_expert_team
    DROP CONSTRAINT IF EXISTS ck_ai_expert_team_coordinator_prompt;

ALTER TABLE ai_expert_team
    DROP CONSTRAINT IF EXISTS ck_ai_expert_team_constraints;

ALTER TABLE ai_expert_team
    ADD CONSTRAINT ck_ai_expert_team_constraints CHECK (
        char_length(constraints_text) <= 4000
    );

ALTER TABLE ai_expert_team
    DROP COLUMN IF EXISTS coordinator_prompt;

COMMIT;
