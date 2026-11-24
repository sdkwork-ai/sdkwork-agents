-- sdkwork:migration
-- id: 0010_expert_catalog
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Reverse 0010_expert_catalog. Drops the expert team, team member and
--   persona aggregates. Expert agents and their categories are ordinary
--   `ai_agent` / `ai_agent_category` rows owned by the catalog seed, so they are
--   not removed here (dropping them would delete rows this migration does not
--   own).
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

DROP TABLE IF EXISTS ai_expert_persona;
DROP TABLE IF EXISTS ai_expert_team_member;
DROP TABLE IF EXISTS ai_expert_team;

COMMIT;
