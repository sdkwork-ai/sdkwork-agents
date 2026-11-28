# Agents Database Module

Owner: `agents-platform`

Canonical contract: `database/contract/schema.yaml` (`7.9.0`)

Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`

## Scope

This module is the single system of record for managed agents and durable agent
execution:

```text
AgentWorkspace -> AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
                                      ^
AgentTask -> AgentTaskRun -> AgentTaskRunAttempt
```

The Session aggregate also owns one current runtime binding, one current
execution placement (requested/effective execution target, pinned execution
host, placement lifecycle and lease evidence), the execution host registry
(docker, micro VM, bare metal, kernel cloud-sandbox pools), resumable
checkpoint references, typed Drive relations, durable Turn input queues,
Task/Run/Attempt scheduling state, retry/lease/fencing state, audit facts, and
transactional outbox facts. External outbox delivery is owned by the platform's
approved generic publisher SPI and is not part of this module; the module hosts
no local Kafka producer, raw HTTP relay, or downstream table writer. IM
conversations, IM delivery state,
runtime-location details, provider catalogs, capability content, and Drive bytes
remain owned by their respective modules.

The module also persists the SDKWork expert catalog: each expert is an
ordinary managed agent (`ai_agent`) grouped by an expert category
(`ai_agent_category`), and the expert aggregates carry what an agent cannot:
`ai_expert_team` (a named team with both display locales and a coordinator
template), `ai_expert_team_member` (ordered, localized duties and instructions),
`ai_expert_persona` (the localized persona body with its content hash),
`ai_expert_profile` (the editorial usage count, badge, catalog order and publish
time), and `ai_expert_collection` / `ai_expert_collection_item` (the market's
curated shelves with ordered, bilingual member labels). The catalog is authored
in `sdkwork-specs/specs/expert-catalog` and rendered into the
`003_expert_catalog` / `002_expert_catalog` seeds by
`scripts/generate-expert-seeds.mjs`; the editorial market layer is seeded by the
hand-maintained `004_expert_market` seed. See
`sdkwork-specs/EXPERTS_CATALOG_SPEC.md`.

## Engine And Lifecycle

PostgreSQL is the only managed-store engine. The `7.9.0` greenfield baseline
contains the complete 39-table Session execution, Task scheduling,
execution-placement and expert catalog model. It is the only database state
supported before the first release. `baseline-plus-migrations` remains the
lifecycle strategy. New installations use the complete baseline; the
post-baseline migration set brings already-initialized shared development
schemas up to the folded baseline (`0001` tool configuration/asset tables,
`0002` `organization_id` standardization, `0003` execution host registry and
session execution placement, `0004` agent role assignment, `0005` managed agent
category catalog, `0006`-`0008` manifest-derived delivery projections and
invariants, `0009` built-in chat agent tenant visibility, `0010` expert team and
persona aggregates, `0011` expert market profiles and collections), so shared
development schemas converge on the baseline without replaying from an empty
schema.

Lifecycle `init` atomically materializes the consolidated baseline only when the
completion anchor is absent. Automatic pending-migration execution defaults to
disabled (`lifecycle.autoMigrate=false`); release and operator workflows run
`pnpm db:migrate` explicitly before service readiness. An existing partial
schema that already contains the completion anchor is never treated as an empty
database: startup drift validation fails closed instead of replaying the
greenfield baseline.

All business `id` columns are application-allocated signed 64-bit values.
PostgreSQL sequences and identity columns are not used for business IDs.

## Role assignment

`ai_agent.allowed_role_codes` is a JSONB array of IAM role codes allowed to see
and run the agent. An empty array is the default and means every role. Role
codes follow `^[a-z][a-z0-9_.-]{0,127}$`, are validated by the application, and
are indexed with GIN for role-membership filtering.

## Initialization state

This module is in **initialization state** for greenfield deployments:

1. **Baseline** — `database/ddl/baseline/{engine}/0001_agents_baseline.sql` contains the full DDL snapshot.
2. **Migrations** — `database/migrations/{engine}/` holds the ordered catch-up and catalog migrations (`0001`..`0011`); fresh baseline installs no-op them through `IF NOT EXISTS`.
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
