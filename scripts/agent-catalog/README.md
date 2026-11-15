# Agent Catalog Source

`catalog.mjs` is the single source of truth for the official system catalog: 64
hosted conversation agents, 7 local-execution agents, and the two-level
professional category taxonomy.

- `AGENTS` — hosted identity, taxonomy, knowledge/memory/MCP references, bilingual
  prompts, welcome messages, suggested prompts, and skill quality gates. All
  default to the Rig Rust implementation (`runtime.family: rig`).
- `LOCAL_EXECUTION_AGENTS` — archive-distributed agents executed by the local
  Harness runtime (`runtime.family: harness`, protocol
  `harness.agent.runtime.v1`) with a zip distribution descriptor.
- `AGENT_CATEGORY_TREE` — nine professional domains and twenty-five assignable
  specialties. Agents reference a leaf code through `categoryCode`.
- `MCP_SERVERS` — the 18 shared MCP catalog entries referenced by agents.
- `MCP_CATEGORIES` — the 12 industry MCP categories used by the catalog UI.
  Each server also gets a deterministic draft connector; activation adds the
  endpoint and credentials without changing the catalog seed.
- `COMMON_GUARDRAILS_*` — cross-domain safety invariants appended to every prompt.

Do not edit generated SQL. Edit this source and run:

```powershell
cd sdkwork-agents
node scripts/generate-agent-catalog-seeds.mjs
node scripts/verify-agent-catalog.mjs
```

The operator guide is
[`docs/guides/operator/agent-catalog.md`](../../docs/guides/operator/agent-catalog.md)
and the kind/runtime contract is
[`specs/AGENTS_AGENT_KIND_AND_RUNTIME_SPEC.md`](../../specs/AGENTS_AGENT_KIND_AND_RUNTIME_SPEC.md).
