# PostgreSQL Migrations

Pre-launch the agents schema is consolidated on the single greenfield baseline:
`database/ddl/baseline/postgres/0001_agents_baseline.sql`. It contains the
complete `7.7.0` schema (canonical Agent, Session, Turn, Task Run/Attempt,
Turn input queue, provider Session directory, typed Interaction envelope,
materialized Session activity keyset (`ai_agent_session.activity_at` with
trigger-maintained recency), turn streaming-content checkpoint, execution
host registry, durable session execution placement, and
simplified `agent.`/`binding.`/`provider.` identity namespaces).

The ordered catch-up migrations align previously initialized shared
development schemas with the folded baseline; fresh baseline installs no-op
them via `IF NOT EXISTS`:

- `0001` materializes the agent tool configuration and generated media asset
  tables.
- `0002` enforces the `organization_id NOT NULL DEFAULT 0` standard on every
  table.
- `0003` materializes the execution host registry
  (`ai_agent_execution_host`) and the durable session execution placement
  (`ai_agent_session_execution_placement`).
- `0004` adds `ai_agent.allowed_role_codes`, the IAM role-code assignment list
  (`[]` = every role, the default).
- `0005` materializes the managed agent category catalog
  (`ai_agent_category`).
- `0006` materializes the manifest-derived agent delivery projection columns
  (`agent_kind`, `runtime_family`, `category_code`, `distribution_kind`,
  `distribution_json`) on `ai_agent`; the manifest stays the authored
  authority and a re-apply recomputes every value.
- `0007` corrects `runtime_family` so a manifest without a runtime descriptor is
  not labeled `rig`: the family derives from the declared runtime or from the
  `rig-rust` implementation type, and stays NULL otherwise.
- `0008` enforces the delivery invariants: local-execution agents require a
  bounded zip archive (entrypoint plus archive reference or Drive locator) and
  the harness runtime protocol; conversation agents reject an archive.

The lifecycle orchestrator applies the baseline once on an empty schema
(`baseline-plus-migrations`, `lifecycle.autoMigrate=false`). The drift gate
then verifies the live schema against `database/contract/`.

After first production release, add ordered expand/contract migrations without
rewriting the released baseline.
