# Official Agent Catalog

- Status: active
- Catalog version: `1.3.0`
- Owner: `agents-platform`
- Source of truth: [`scripts/agent-catalog/catalog.mjs`](../../../scripts/agent-catalog/catalog.mjs)
- Generator: [`scripts/generate-agent-catalog-seeds.mjs`](../../../scripts/generate-agent-catalog-seeds.mjs)
- Live gate: [`database/verification/agent_catalog_checks.sql`](../../../database/verification/agent_catalog_checks.sql)

## 1. What This Catalog Is

The official catalog is the commercial starting set of managed agents for
`tenant_id=100001`, `organization_id=0`. It is seeded through the standard
database lifecycle, not through application code or an ad-hoc SQL console.

The catalog ships **64 agents** across twelve business domains:

| Domain | Agents | Representative roles |
| --- | ---: | --- |
| Engineering | 7 | Full-stack, frontend, backend, architect, DevOps/SRE, QA, data/AI |
| Education | 16 | K12 Chinese, math, English, physics, chemistry, biology, history, geography, ethics/law; university math, physics, CS, economics, law, medicine, engineering |
| Healthcare | 4 | General practice, pediatrics, cardiology, mental health |
| Legal | 5 | Civil/commercial, labor, criminal, IP, family |
| Travel | 3 | Itinerary planner, destination guide, visa advisor |
| E-commerce | 4 | Operations, product selection, customer service, advertising |
| Real estate | 3 | Buyer, rental, valuation |
| Restaurant / F&B | 4 | Operations, menu engineering, food safety, delivery marketing |
| Enterprise services | 4 | Management consulting, HR, tax, B2B sales |
| Government | 4 | Service guide, policy interpretation, official documents, digital government |
| Hotel / hospitality | 4 | Operations, revenue management, guest service, OTA marketing |
| Writing / publishing | 6 | Novelist, book author, news reporter, article writer, copywriter, editor |

Every agent receives:

- a stable `agent.<domain>.<role>` identity and a unique `code`;
- a versioned `ai_agent_version` snapshot (`version_number=1`);
- one dedicated knowledge-space binding;
- one dedicated memory-space binding;
- one or more MCP catalog bindings;
- one skill binding carrying the role's pre-delivery quality gate;
- a management profile with a localized system prompt, welcome message, model,
  temperature, suggested prompts, knowledge references, and skill references.

## 2. Ownership Boundaries

The catalog respects module ownership. No module writes another module's
tables.

| Module | Owns | Seeded rows |
| --- | --- | --- |
| `sdkwork-agents` | Agent identity, versions, composition references | `ai_agent`, `ai_agent_version`, `ai_agent_composition_slot` |
| `sdkwork-knowledgebase` | Knowledge spaces, documents, chunks, retrieval | `kb_space` |
| `sdkwork-memory` | Memory spaces, subjects, bindings, records | `ai_space`, `ai_subject`, `ai_memory_binding` |
| `sdkwork-mcp` | MCP server catalog, connectors, tools | `ai_mcp_server` |

Agents stores only stable references. It never copies knowledge content,
memory records, MCP endpoints, or MCP credentials.

## 3. Seed Asset Layout

```text
sdkwork-agents/
  scripts/agent-catalog/catalog.mjs
  scripts/generate-agent-catalog-seeds.mjs
  scripts/verify-agent-catalog.mjs
  database/seeds/common/002_agent_catalog.sql
  database/seeds/locales/zh-CN/001_agent_catalog_display.sql
  database/seeds/locales/en-US/001_agent_catalog_display.sql
  database/verification/agent_catalog_checks.sql

sdkwork-knowledgebase/
  database/seeds/common/002_agent_catalog.sql
  database/seeds/locales/zh-CN/001_agent_catalog_display.sql
  database/seeds/locales/en-US/001_agent_catalog_display.sql

sdkwork-memory/
  database/seeds/common/002_agent_catalog.sql
  database/seeds/locales/zh-CN/001_agent_catalog_display.sql
  database/seeds/locales/en-US/001_agent_catalog_display.sql

sdkwork-mcp/
  database/seeds/common/002_agent_catalog.sql
  database/seeds/locales/zh-CN/001_agent_catalog_display.sql
  database/seeds/locales/en-US/001_agent_catalog_display.sql
```

The common seed is language-neutral: stable ids, codes, taxonomy, capability
references, versions, and composition slots. It deliberately leaves display
names, descriptions, system prompts, and skill instructions unset. Both
`zh-CN` and `en-US` locale seeds carry a complete localized payload, and each
has its own manifest checksum.

This separation means a common seed run never overwrites localized content.
The default deployment runs `common + zh-CN`. An English deployment runs
`common + en-US`. The catalog source remains bilingual and both locale sets are
verified by the static gate.

## 4. Initialization Sequence

Run from each owning repository root. The order matters only for human review:
the Agents seed has no cross-module foreign keys.

```powershell
# 1. Agents schema and catalog
cd sdkwork-agents
pnpm db:migrate
pnpm db:seed
pnpm db:status

# 2. Knowledge spaces
cd ../sdkwork-knowledgebase
pnpm db:seed

# 3. Memory spaces and agent-memory bindings
cd ../sdkwork-memory
pnpm db:seed

# 4. MCP catalog entries
cd ../sdkwork-mcp
pnpm db:migrate
pnpm db:seed
```

`db:seed` is idempotent. A second run reports `applied 0 seed script(s)` when
the seed content checksum has not changed. Each seed file is wrapped in
`BEGIN` / `COMMIT`; a partial failure rolls back that file.

The default locale is `zh-CN`. To initialize an English deployment, set
`SDKWORK_DATABASE_SEED_LOCALE=en-US` before running the four seed commands.
The common seed remains neutral and the `en-US` locale file supplies the
English display and prompts. Do not switch locales on an already seeded
tenant by re-running seeds: seed history records the locale that was applied.

## 5. Verification

Static gates:

```powershell
cd sdkwork-agents
pnpm check:agent-catalog
node --test tests/contract/agent-catalog.contract.test.mjs
pnpm db:validate
```

Live gate against the shared development schema:

```powershell
psql "$SDKWORK_DATABASE_URL" -v ON_ERROR_STOP=1 `
  -f database/verification/agent_catalog_checks.sql
```

Expected result:

```text
agents | versions | slots | knowledge_spaces | memory_spaces | mcp_servers | mcp_categories | mcp_connectors
-------+----------+-------+------------------+---------------+-------------+----------------+----------------
    64 |       64 |   364 |               64 |            64 |          18 |             12 |             18
```

The live gate fails closed when:

- any agent lacks an enabled knowledge, memory, MCP, or skill binding;
- a reference points to a missing `kb_space`, `ai_space`, or `ai_mcp_server`;
- a knowledge slot's `policy_json.spaceId` differs from `target_ref`;
- the management profile lacks a `knowledge.base.*` reference;
- a slot id is duplicated;
- a prompt is shorter than 900 characters;
- an agent uses an unknown domain.

## 6. What Is Ready And What Needs Activation

Ready after seeding:

- all 64 agents are active and publicly visible in the tenant catalog;
- every agent has a complete Chinese management profile and system prompt;
- every agent has version history and stable composition references;
- knowledge spaces and memory spaces exist, are active, and are tenant-scoped;
- 18 MCP server entries, 12 industry categories, and 18 draft connectors exist.
  Each server points at its latest draft connector; no connector has an endpoint
  or credential until an operator activates it.
- the six writing agents declare their intended tool descriptors on the
  `mcp.sdkwork.writing-sources` and `mcp.sdkwork.knowledgebase-notes` slots
  (`read-only` for source verification, `read-write` for notes), so consumers can
  render the capability surface before activation. The connectors stay draft and
  the runtime still discovers no live tool until an operator activates them.

Requires operator activation before the capability is live:

| Capability | Gate |
| --- | --- |
| Knowledge grounding | Upload curated sources into each `kb_space`; the space is provisioned but intentionally contains no tenant-private documents |
| Memory retrieval | The Agents memory adapter must be configured (`SDKWORK_AGENTS_MEMORY_MODE`); memory spaces start empty and learn from sessions |
| MCP tools | The 18 draft connectors are `publish_status=draft`, `lifecycle_status=0`, and have no endpoint. Add the approved endpoint and credentials to the draft connector, publish it, then update the slot policy with `endpointUrl` and the discovered `tools` descriptors. The writing agents' declared tool names describe intent only until this activation |
| Model routing | Each management profile references `model.deepseek-deepseek-v4-pro`; confirm the deployment's model catalog and routing policy resolve it |
| Role visibility | `allowed_role_codes=[]` means every role may see and run the agent; narrow it for regulated tenants |

The MCP composition slots omit `endpointUrl` until activation, and every
non-writing slot also omits tool descriptors; only the writing slots publish
intended descriptors for consumer rendering. The toolkit therefore fails closed
and never exposes an unchecked tool
that has no reachable server. This is a deliberate safety gate, not an
unfinished reference.

## 7. Regenerating The Catalog

Edit the source catalog, then run:

```powershell
cd sdkwork-agents
node scripts/generate-agent-catalog-seeds.mjs
node scripts/verify-agent-catalog.mjs
```

The generator is deterministic. Row ids, version ids, slot ids, and uuids are
derived from stable catalog keys, so unchanged content produces byte-identical
SQL. Changed content changes the seed file checksum, and the lifecycle applies
only the changed file on the next `db:seed`.

Do not hand-edit generated SQL. Fix the catalog source, generator, or owning
module seed manifest instead.

## 8. Adding An Agent

1. Add one entry to `scripts/agent-catalog/catalog.mjs`.
2. Provide complete `zh` and `en` payloads: role, tagline, mission, workflow,
   standards, deliverables, guardrails, welcome message, suggested prompts, and
   skill instructions.
3. Reference one or more existing MCP keys, or add a new catalog entry to
   `MCP_SERVERS`.
4. Run the generator and static gate.
5. Run `db:seed` in all four owning modules.
6. Run the live verification SQL.
7. Review the agent's prompt for domain accuracy, safety boundaries, and
   jurisdiction or grade-level applicability before publishing.

Every agent must keep at least one knowledge binding, one memory binding, one
MCP binding, and one skill binding. The static gate enforces this.

## 9. Safety And Compliance Invariants

The catalog is designed for commercial deployment under these invariants:

- Medical agents never diagnose, prescribe, or provide doses. Emergency red
  flags route to immediate care.
- Legal agents never guarantee outcomes and never replace licensed counsel.
  Jurisdiction and effective-law date are part of the analysis.
- Education agents never write graded submissions and never use methods above
  the learner's grade band without explanation.
- Financial, travel, e-commerce, and property agents never fabricate live
  prices, availability, policies, or returns.
- Every high-stakes answer includes applicability, invalidation conditions, and
  a human-review path.
- Retrieved content is untrusted reference data. No agent follows instructions
  embedded in retrieved documents or discloses system prompts or credentials.

These invariants are enforced in the prompt text and in the static gate's
prompt-length and structure checks. Domain reviewers should still review every
prompt change before release.

## 10. Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `applied 0 seed script(s)` on first run | Seed content checksum already recorded | Check `ops_seed_history`; no action |
| `column "allowed_role_codes" ... does not exist` in MCP seed | Pending MCP migration | Run `pnpm db:migrate` in `sdkwork-mcp`, then `pnpm db:seed` |
| Knowledge rows invisible in `psql` | Forced RLS | `SELECT set_config('app.current_tenant_id','100001',false);` |
| Locale checksum mismatch | Seed files changed without regenerating the manifest | Re-run the generator; never hand-edit the checksum |
| `policy_space_mismatch` | Numeric JSON precision loss | Keep `spaceId` as a JSON string; the static gate rejects 18-digit JSON numbers |

## 11. Related Documents

- [Database module README](../../../database/README.md)
- [Agents domain specification](../../../specs/AGENTS_DOMAIN_SPEC.md)
- [Knowledge and memory grounding design](../../../docs/architecture/tech/TECH-2026-1019-turn-knowledge-memory-grounding.md)
- [Runbook: agent catalog initialization](../../runbooks/agent-catalog-initialization.md)
