-- sdkwork:migration
-- id: 0011_expert_market
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Reverse 0011_expert_market. Drops the expert market's editorial
--   layer (collection items, collections, per-expert profiles). Expert agents
--   and their categories are ordinary `ai_agent` / `ai_agent_category` rows
--   owned by the catalog seed, so they are not removed here (dropping them
--   would delete rows this migration does not own).
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

DROP TABLE IF EXISTS ai_expert_collection_item;
DROP TABLE IF EXISTS ai_expert_collection;
DROP TABLE IF EXISTS ai_expert_profile;

COMMIT;
