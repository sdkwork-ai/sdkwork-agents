-- sdkwork:migration
-- id: 0009_builtin_chat_agent_tenant_visibility
-- engine: postgres
-- module: sdkwork-agents
-- purpose: Repair built-in chat agent visibility for records created before the
--   tenant-sharing invariant existed.
-- reversible: false
-- rollback: forward-fix (a data backfill; the previous per-record visibility is not recorded and restoring it would re-break tenant-shared access)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s
-- The built-in chat conversation agent (`agent.chat.default`) is a
-- tenant-shared record: its id is unique per tenant and every member addresses
-- the same row. Clients used to provision it with `private` visibility, which
-- made the record readable only by its creator (the application retrieve
-- answered 404 for everybody else) while it still occupied the tenant-unique
-- id (their create answered 409). The service now normalizes built-in agent ids
-- to tenant visibility on create and update; this backfill repairs the records
-- created before that invariant existed.
UPDATE ai_agent
SET visibility = 2,
    version = version + 1,
    updated_at = NOW()
WHERE agent_id = 'agent.chat.default'
  AND visibility <> 2
  AND deleted_at IS NULL;
