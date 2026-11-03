# Agent Catalog Source

`catalog.mjs` is the single source of truth for the official 58-agent catalog.

- `AGENTS` — identity, taxonomy, knowledge/memory/MCP references, bilingual
  prompts, welcome messages, suggested prompts, and skill quality gates.
- `MCP_SERVERS` — the 16 shared MCP catalog entries referenced by agents.
- `MCP_CATEGORIES` — the 11 industry MCP categories used by the catalog UI.
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
[`docs/guides/operator/agent-catalog.md`](../../docs/guides/operator/agent-catalog.md).
