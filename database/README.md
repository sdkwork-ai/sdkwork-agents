# Agents Database Module

Owner: `agents-platform`

Canonical contract: `database/contract/schema.yaml` (`7.2.0`)

Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`

## Scope

This module is the single system of record for managed agents and durable agent
execution:

```text
AgentWorkspace -> AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
                                      ^
AgentTask -> AgentTaskRun -> AgentTaskRunAttempt
```

The Session aggregate also owns one current runtime binding, resumable
checkpoint references, typed Drive relations, durable Turn input queues,
Task/Run/Attempt scheduling state, retry/lease/fencing state, audit facts, and
transactional outbox facts. External outbox delivery remains a release gate
until the platform provides an approved generic publisher SPI; this module does
not implement a local Kafka producer, raw HTTP relay, or downstream table
writer. IM conversations, IM delivery state,
runtime-location details, provider catalogs, capability content, and Drive bytes
remain owned by their respective modules.

## Engine And Lifecycle

PostgreSQL is the only managed-store engine. The `7.2.0` greenfield baseline
contains the complete 30-table Session execution and Task scheduling model. It
is the only database state supported before the first release.
`baseline-plus-migrations` remains the lifecycle strategy. New installations
use the complete baseline; the post-baseline migration set is empty while the
app is pre-launch (the baseline is folded to the full current contract), so
shared development schemas converge by resetting the agents state to the
baseline instead of replaying forward-only migrations.

Lifecycle `init` atomically materializes the consolidated baseline only when the
completion anchor is absent. Automatic pending-migration execution defaults to
disabled (`lifecycle.autoMigrate=false`); release and operator workflows run
`pnpm db:migrate` explicitly before service readiness. An existing partial
schema that already contains the completion anchor is never treated as an empty
database: startup drift validation fails closed instead of replaying the
greenfield baseline.

All business `id` columns are application-allocated signed 64-bit values.
PostgreSQL sequences and identity columns are not used for business IDs.

## Initialization state

This module is in **initialization state** for greenfield deployments:

1. **Baseline** — `database/ddl/baseline/{engine}/0001_agents_baseline.sql` contains the full DDL snapshot.
2. **Migrations** — `database/migrations/{engine}/` is reserved for incremental schema changes after the application has shipped. It is intentionally empty while the application is pre-launch.
3. **Drift** — run `pnpm db:drift:check` before release.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
