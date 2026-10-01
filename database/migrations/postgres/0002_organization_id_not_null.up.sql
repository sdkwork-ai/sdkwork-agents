-- sdkwork:migration
-- id: 0002_organization_id_not_null
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Enforce organization_id NOT NULL DEFAULT on all tables in the
--   consolidated baseline. NULL rows (pre-standard data anomalies) are
--   backfilled with the platform sentinel before NOT NULL is set, and
--   NOT NULL columns without an explicit default receive the sentinel
--   default, keeping existing deployments consistent with fresh baseline
--   installs.
-- reversible: false
-- rollback: forward-fix (sentinel backfill is the canonical fix; NULL
--   organization rows are data anomalies)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE ai_agent ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_runtime_binding ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_runtime_binding SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_runtime_binding ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_runtime_binding ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_composition_slot ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_composition_slot SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_composition_slot ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_composition_slot ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_audit_event ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_audit_event SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_audit_event ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_audit_event ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_workspace ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_workspace SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_workspace ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_workspace ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_project ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_project SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_project ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_project ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_project_composition_slot ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_project_composition_slot SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_project_composition_slot ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_project_composition_slot ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_session ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_session SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_session ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_session ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_session_runtime_binding ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_session_runtime_binding SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_session_runtime_binding ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_session_runtime_binding ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_turn ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_turn SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_turn ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_turn ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_turn_input_queue_entry ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_turn_input_queue_entry SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_turn_input_queue_entry ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_turn_input_queue_entry ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_session_item ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_session_item SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_session_item ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_session_item ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_item_drive_ref ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_item_drive_ref SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_item_drive_ref ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_item_drive_ref ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_item_feedback ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_item_feedback SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_item_feedback ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_item_feedback ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_interaction ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_interaction SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_interaction ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_interaction ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_session_checkpoint ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_session_checkpoint SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_session_checkpoint ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_session_checkpoint ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_task ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_task SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_task ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_task ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_task_run ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_task_run SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_task_run ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_task_run ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_task_run_attempt ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_task_run_attempt SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_task_run_attempt ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_task_run_attempt ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_resource_user_state ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_resource_user_state SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_resource_user_state ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_resource_user_state ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_project_member ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_project_member SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_project_member ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_project_member ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_share_link ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_share_link SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_share_link ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_share_link ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_outbox_event ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_outbox_event SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_outbox_event ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_outbox_event ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_model_configuration_profile ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_model_configuration_profile SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_model_configuration_profile ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_model_configuration_profile ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_tool_configuration ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_tool_configuration SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_tool_configuration ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_tool_configuration ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE ai_agent_tool_asset ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE ai_agent_tool_asset SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE ai_agent_tool_asset ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE ai_agent_tool_asset ALTER COLUMN organization_id SET NOT NULL;

COMMIT;
