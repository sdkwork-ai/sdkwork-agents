# Agents Agent Kind And Runtime Specification

- Version: `1.0.0`
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Related:
  - [`AGENTS_PROVIDER_TAXONOMY_SPEC.md`](./AGENTS_PROVIDER_TAXONOMY_SPEC.md)
  - [`AGENTS_DOMAIN_SPEC.md`](./AGENTS_DOMAIN_SPEC.md)

## 1. Purpose

This specification defines the two delivery shapes of a managed agent — a
hosted **conversation** agent and a locally installed **local-execution** agent —
the runtime descriptor each one carries, the professional category taxonomy that
classifies them, and the manifest fields the owning modules project into indexed
columns.

The catalog, the console creation flow, the admin surface, and the seed verifier
all consume this contract instead of inventing per-surface fields.

## 2. Agent Kinds

| Kind | Delivery | Execution | Editor |
| --- | --- | --- | --- |
| `conversation` | Managed catalog record | Managed runtime (RIG by default) | Full editor (`/console/agents/editor/<id>`) |
| `local-execution` | Zip archive the user installs | Local Harness runtime | Basic information + archive upload only |

`agent_kind` is not `implementation_kind`. Kind is the user-facing delivery
shape; implementation kind is how the platform integrates the engine.

Creation flow:

1. The host opens one basic-information dialog (name, description, avatar,
   professional category, kind).
2. `local-execution` continues to the archive upload step and finishes there.
3. `conversation` continues to the single agent editor page, which owns prompt,
   model, knowledge, memory, skill, MCP, voice, and publishing.

The flow is a reusable component (`@sdkwork/agents-pc-agents/editor`) so the
console, the admin surface, and the standalone app mount the same steps.

## 3. Runtime Descriptor

Every agent manifest carries `runtime`:

```json
{
  "family": "rig",
  "engine": "rig",
  "implementation_kind": "typed-local-provider",
  "implementation_type": "rig-rust",
  "provider_id": "provider.rig.rust",
  "capabilities": ["model.chat", "tool.invoke"]
}
```

Rules:

1. Hosted conversation agents default to `family: "rig"` and
   `implementation_type: "rig-rust"`. A catalog entry that omits a runtime is
   normalized to this default.
2. Local-execution agents use `family: "harness"`, declare the protocol
   `harness.agent.runtime.v1`, and name the executable engine
   (`openclaw`, `workbuddy`, `codex`, `claude-code`, `gemini-cli`, `opencode`,
   `hermes`). `provider_id` is `provider.harness.local`.
3. A runtime whose provider manifest is not registered fails closed at execution
   time. The catalog never claims an executable provider that is not
   conformant.
4. `runtime.engine` is the concrete product; `runtime.family` is the integration
   model. Never overload one with the other.

## 4. Archive Distribution

A local-execution agent manifest carries `distribution`:

```json
{
  "kind": "archive",
  "format": "zip",
  "archiveRef": "archive.sdkwork.openclaw",
  "version": "1.0.0",
  "entrypoint": "openclaw/manifest.json",
  "installMode": "extract",
  "checksum": null
}
```

- `archiveRef` is a catalog package reference. The Harness package registry
  resolves it to a downloadable archive; the repository never commits binaries.
- `entrypoint` names the manifest inside the extracted archive.
- `installMode` is `extract`; installers must extract into an isolated directory
  and refuse path traversal.
- `checksum` is `null` in the catalog seed. When a user uploads an archive the
  console computes SHA-256 in the browser and replaces the descriptor with the
  Drive locator (`driveUri`, `fileName`, `sizeBytes`, `checksum`).
- Uploaded archives use the declared upload purpose `agent.package_archive`
  (`agent-package`), Drive upload profile `archive`, retention `long_term`, and a
  maximum size of 512 MiB. The renamed archive is stored in Drive; the agent
  manifest keeps only the locator, integrity evidence, and archive identity.

## 5. Professional Category Taxonomy

`ai_agent_category` is seeded from `AGENT_CATEGORY_TREE` in the catalog source.
The taxonomy has two levels:

| Level | Assignable | Example |
| --- | --- | --- |
| Domain (parent) | no | `technology` 技术与工程 |
| Specialty (leaf) | yes | `engineering` 工程技术 |

Rules:

1. Nine professional domains and twenty-five specialties are seeded for
   `tenant_id=100001`, `organization_id=0`.
2. Every agent references exactly one leaf through `manifest.category_code` (and
   the management profile `categoryId`).
3. Parents never carry agents; filtering by a domain is an aggregate query over
   its leaves.
4. Codes are stable and locale-neutral; display names and descriptions are
   localized by the locale seed.
5. The previous single-level category names are preserved as leaf codes wherever
   they already existed (`engineering`, `legal`, `travel`, ...), except education,
   which is split into `education-k12`, `education-higher`, and
   `education-training`.

## 6. Database Projection

The manifest is the authored authority. `ai_agent` exposes five STORED generated
columns so list filters, drift checks, and analytics never depend on a second
write path:

| Column | Source | Default |
| --- | --- | --- |
| `agent_kind` | `manifest.agent_kind` | `conversation` |
| `runtime_family` | `manifest.runtime.family`, else `rig` when `implementation_type = rig-rust` | `NULL` |
| `category_code` | `manifest.category_code` | `NULL` |
| `distribution_kind` | `manifest.distribution.kind` | `manifest` |
| `distribution_json` | `manifest.distribution` | `NULL` |

Writers must not insert or update these columns directly; PostgreSQL rejects
that. Update the manifest and the projection follows in the same statement.
A manifest without a runtime descriptor is never labeled `rig` unless its
implementation type is `rig-rust`.

Migration: `database/migrations/postgres/0006_agent_kind_runtime_and_category`,
the `0007_agent_runtime_family_from_implementation` correction, and
`0008_agent_kind_distribution_constraints` (local agents require a bounded zip
archive and the harness protocol; conversation agents reject an archive).

## 7. Catalog Initialization

The seeded system catalog contains:

- 64 conversation agents across twelve historical business domains, all
  professional leaves in the modern taxonomy, all defaulting to RIG;
- 7 local-execution agents: OpenClaw, WorkBuddy, Codex CLI, Claude Code, Gemini
  CLI, OpenCode, and Hermes;
- 34 category rows (9 domains + 25 specialties).

Local agents carry identity, version, category, runtime, and archive
distribution only. They intentionally have no knowledge, memory, MCP, skill, or
prompt composition because those belong to the archive and the local harness.

## 8. Verification

```powershell
node scripts/verify-agent-catalog.mjs
node --test tests/contract/agent-catalog.contract.test.mjs
pnpm db:migrate
pnpm db:seed
pnpm db:drift:check
node ../tmp-.../agent_catalog_checks.sql  # live SQL verification
```
