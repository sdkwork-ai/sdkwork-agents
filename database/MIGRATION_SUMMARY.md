# Database Contract State

Status: active pre-launch baseline consolidation

Updated: 2026-10-03

## Current Contract

- Contract version: `7.3.0`
- Managed engine: PostgreSQL
- Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- Lifecycle strategy: `baseline-plus-migrations`
- Development migrations: catch-up set `0001`..`0003` aligning
  previously initialized shared development schemas with the folded baseline
- Active tables: 32

The full current schema is installed from one baseline on an empty schema and
tracked in `ops_database_installation_state`. The original pre-launch forward
development migrations (`0001`..`0007`) were removed when the baseline was
folded to the complete schema; the restored catch-up migrations
(`0001` tool configuration/asset tables, `0002` `organization_id`
standardization, `0003` execution host registry and session execution
placement) re-align databases that were initialized from an older baseline and
no-op on fresh baseline installs via `IF NOT EXISTS`. There is
no dual-write path, derived read store, legacy Session table, or runtime
compatibility branch.

After first production release, add ordered expand/contract migrations without
rewriting the released baseline.

## Operational Checks

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:drift:check
```
