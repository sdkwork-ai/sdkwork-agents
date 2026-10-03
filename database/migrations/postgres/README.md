# PostgreSQL Migrations

Pre-launch the agents schema is consolidated on the single greenfield baseline:
`database/ddl/baseline/postgres/0001_agents_baseline.sql`. It contains the
complete `7.3.0` schema (canonical Agent, Session, Turn, Task Run/Attempt,
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

The lifecycle orchestrator applies the baseline once on an empty schema
(`baseline-plus-migrations`, `lifecycle.autoMigrate=false`). The drift gate
then verifies the live schema against `database/contract/`.

After first production release, add ordered expand/contract migrations without
rewriting the released baseline.
