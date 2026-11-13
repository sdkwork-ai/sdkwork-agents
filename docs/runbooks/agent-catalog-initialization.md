# Agent Catalog Initialization Runbook

- When to use: first install, a fresh tenant bootstrap, a catalog version upgrade, or recovery after a failed seed
- Owner: `agents-platform`
- Catalog source: [`scripts/agent-catalog/catalog.mjs`](../../../scripts/agent-catalog/catalog.mjs)
- Operator guide: [`docs/guides/operator/agent-catalog.md`](../guides/operator/agent-catalog.md)

## 1. Preconditions

- PostgreSQL is reachable with the shared `SDKWORK_DATABASE_*` profile.
- The schema is `sdkwork_ai_dev` and the database role is not a superuser.
- The four owning repositories are on `main` and their seed manifests validate.
- No conflicting agent ids or reserved id ranges exist.

Check reserved ranges before the first run:

```sql
SELECT id, agent_id FROM ai_agent
WHERE id BETWEEN 366300000000000001 AND 366300000000000064;
```

The catalog uses these deterministic ranges:

| Range | Owner |
| --- | --- |
| `366300000000000001..064` | `ai_agent` |
| `366500000000000001..064` | `ai_agent_version` |
| `366400000000000001..` | `ai_agent_composition_slot` |
| `533000000000000001..064` | `kb_space` |
| `534000000000000001..064` | `ai_space` |
| `535000000000000001..018` | `ai_mcp_server` |
| `536000000000000001..064` | `ai_subject` |
| `537000000000000001..064` | `ai_memory_binding` |
| `538000000000000001..018` | `ai_mcp_connector` |
| `539000000000000001..012` | `ai_mcp_server_category` |

## 2. Apply In Order

```powershell
cd sdkwork-agents
pnpm db:plan
pnpm db:migrate
pnpm db:status
pnpm db:seed
pnpm db:status
```

Expected: `status=clean pending_migrations=0`, and seed applies
`001_bootstrap.sql`, `002_agent_catalog.sql`, and the `zh-CN` display file on
the first run.

For an English-first deployment, export
`SDKWORK_DATABASE_SEED_LOCALE=en-US` before the four seed commands. The common
seed is neutral, so the `en-US` locale file supplies the English payload.
Choose the seed locale at initialization, not by re-running seeds on a live
tenant.

```powershell
cd ../sdkwork-knowledgebase
pnpm db:plan
pnpm db:seed
pnpm db:status
```

```powershell
cd ../sdkwork-memory
pnpm db:seed
pnpm db:status
```

The memory seed writes `ai_space`, `ai_subject`, and an `agent_space` /
`retriever` binding per agent. The drift policy ignores the trigger-maintained
`ai_record.search_document` column so `db:status` stays clean.

```powershell
cd ../sdkwork-mcp
pnpm db:plan
pnpm db:migrate
pnpm db:seed
pnpm db:status
```

The MCP migration `0002_mcp_server_allowed_role_codes` is required before the
catalog seed can write `allowed_role_codes`.

## 3. Verify The Seed

```powershell
cd sdkwork-agents
pnpm check:agent-catalog
node --test tests/contract/agent-catalog.contract.test.mjs
pnpm db:validate
```

```powershell
psql "$SDKWORK_DATABASE_URL" -v ON_ERROR_STOP=1 `
  -f database/verification/agent_catalog_checks.sql
```

Expected summary:

```text
agents | versions | slots | knowledge_spaces | memory_spaces | mcp_servers | mcp_categories | mcp_connectors
-------+----------+-------+------------------+---------------+-------------+----------------+----------------
    64 |       64 |   364 |               64 |            64 |          18 |             12 |             18
```

## 4. Manual Spot Checks

Run with the tenant context because Knowledgebase uses forced RLS:

```sql
SELECT set_config('app.current_tenant_id', '100001', false);
SELECT set_config('app.current_organization_id', '0', false);

SELECT a.agent_id, a.display_name,
       s_k.target_ref AS knowledge_space,
       s_m.target_ref AS memory_space
FROM ai_agent a
LEFT JOIN ai_agent_composition_slot s_k
  ON s_k.tenant_id = a.tenant_id AND s_k.agent_id = a.agent_id AND s_k.slot_kind = 'knowledge'
LEFT JOIN ai_agent_composition_slot s_m
  ON s_m.tenant_id = a.tenant_id AND s_m.agent_id = a.agent_id AND s_m.slot_kind = 'memory'
WHERE a.agent_id = 'agent.programmer.fullstack';

SELECT target_ref, policy_json->>'spaceId' AS policy_space
FROM ai_agent_composition_slot
WHERE agent_id = 'agent.programmer.fullstack' AND slot_kind = 'knowledge';
```

The two space ids must match exactly. A numeric JSON value would lose the last
digits; the catalog stores `spaceId` as a string.

## 5. Re-Run And Upgrade

Re-running the same seed content is a no-op:

```powershell
pnpm db:seed
# applied 0 seed script(s)
```

After changing `catalog.mjs`:

```powershell
node scripts/generate-agent-catalog-seeds.mjs
node scripts/verify-agent-catalog.mjs
```

Then run `db:seed` in the four modules. The lifecycle records the new file
checksum and applies only the changed files. Never hand-edit generated SQL or
the locale checksum.

## 6. Failure Handling

| Failure | Action |
| --- | --- |
| Seed security validation rejects a file | Remove inline `--` comments after SQL, or any `GRANT`/`TRUNCATE`/dynamic SQL; full-line comments are allowed |
| Transaction rolls back mid-file | Fix the first failing statement, re-run `db:seed`; the transaction boundary prevents partial rows |
| `policy_space_mismatch` | Regenerate; ensure `spaceId` is a string, not a JSON number |
| Missing knowledge/memory/MCP reference | Run the owning module seed, then the live gate again |
| Duplicate agent id | Check the reserved range and `ai_agent` conflicts; do not renumber existing rows |
| RLS hides rows | Set `app.current_tenant_id` and `app.current_organization_id`, or use the module API |

## 7. Activation After Initialization

The seed provisions identity and references, not tenant-private content or
credentials.

1. Upload curated sources into each `kb_space` through the Knowledgebase API.
2. Confirm the memory adapter mode and base URL for the deployment.
3. Create MCP connectors with approved endpoints and credentials, then update
   each MCP slot policy with `endpointUrl` and the discovered tool descriptors.
4. Confirm the referenced model id resolves in the deployment model catalog.
5. Review role visibility for regulated tenants before publishing the catalog.

Until step 3 is complete, MCP slots deliberately carry no endpoint and no tools;
the turn toolkit skips them and never calls an unreachable server.

## 8. Reporting

Record for each initialization:

- catalog version and seed file checksums;
- `db:plan`, `db:seed`, `db:status` output for all four modules;
- static gate and live gate output;
- activation gaps (knowledge ingestion, MCP connectors, model routing);
- any pre-existing module issue, clearly separated from catalog changes.

Authority: [`DATABASE_FRAMEWORK_SPEC.md`](../../../sdkwork-specs/DATABASE_FRAMEWORK_SPEC.md),
[`DATABASE_SPEC.md`](../../../sdkwork-specs/DATABASE_SPEC.md),
[`AGENTS_DOMAIN_SPEC.md`](../../../specs/AGENTS_DOMAIN_SPEC.md).
