# SDKWork Agents And Appstore Consumer Boundary Specification

- Version: `1.0.0`
- Status: active architecture constraint
- Owner: `agents-platform`
- Consumer: `sdkwork-appstore`
- Related:
  - `AGENTS_KERNEL_BOUNDARY_SPEC.md`
  - `AGENTS_DOMAIN_SPEC.md`
  - `AGENTS_SESSION_MODEL_SPEC.md`
  - `AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md`
  - `../../sdkwork-appstore/specs/AGENTS_DEPENDENCY_BOUNDARY_SPEC.md` (consumer-side contract)

## 1. Dependency Direction

```text
sdkwork-appstore -> sdkwork-agents -> sdkwork-kernel
```

`sdkwork-appstore` is an Agents consumer, in the same class as `sdkwork-im`.
Agents never depends on the appstore through Cargo, pnpm, generated SDKs, HTTP
clients, runtime mounting, source aliases, repositories, or database access. The
appstore consumes Agents only through the public app API surface and the
generated `@sdkwork/agents-app-sdk` client.

## 2. Consumption Surfaces

The appstore PC and H5 applications bind the Agents client inside their `*-core`
packages (for example `sdkwork-appstore-pc-core/src/sdk/clients.ts` calls
`createAgentsAppClient` with the configured `agentsAppApiBaseUrl`) and expose
Agents capabilities to storefront pages only through appstore-owned service
ports.

Allowed:

- Public generated `@sdkwork/agents-app-sdk` operations (`/app/v3/api` surface).
- Agents domain resources Project, Session, Turn, Session Item, Interaction,
  Task, and model-provider profiles, as exposed by the app API.
- Storefront presentation layers that render Agents-backed state through the
  appstore service layer.

Forbidden:

- Raw HTTP, manual auth headers, local SDK proxies, or DTO forks replacing the
  generated client.
- Deep imports into `@sdkwork/agents-app-sdk/generated/**` transport internals.
- Reverse imports from Agents sources into appstore sources.
- Writing `agents_*` tables or any Agents-owned store from appstore services.

## 3. Storefront Catalog Versus Agents Runtime

The appstore owns its marketplace composition: the AI Hub sidebar group
(experts, plugins, skills, MCP, templates) and the independent storefront
expert catalog page (`/experts`) are appstore presentation surfaces.

Agents owns the durable runtime domain. Storefront curated expert content is
NOT an Agents contract; when an expert catalog API is introduced in Agents, it
must be added to the Agents API authority, the generated SDK must be
regenerated, and the appstore must consume it through the approved SDK client.
Agent execution that a storefront expert participates in (sessions, turns,
items, interactions) always flows through the Agents SDK; the appstore must not
persist Agents runtime state in its own stores.

## 4. Data Ownership

Agents owns managed-agent identity, execution sessions, turns, session items,
interactions, checkpoints, usage, audit, and Drive references. The appstore owns
its store catalog, publisher, listing, release, library, moderation, and store
analytics tables.

- No cross-module foreign keys or cross-module SQL.
- Cross-module references stay opaque identifiers (Agents ids are int64 wire
  strings per `API_SPEC.md` section 13.6; never coerce them to `number`).
- Deletion never cascades across module stores.
- Credentials, session tokens, and signed URLs are never copied into either
  module's metadata.

## 5. Workspace Federation

The appstore declares the Agents TypeScript SDK through the dual-track model:

- Local development: `../sdkwork-agents/sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript`
  in the appstore root `pnpm-workspace.yaml`, consumed with `workspace:*`.
- CI packaging: a matching `sdkwork-agents` entry in the appstore
  `sdkwork.workflow.json` `dependencies[]`.

Agents-side changes that alter the generated SDK surface require regenerating
`sdkwork-agents-app-sdk` through the standard generator before appstore
consumers are updated.

## 6. Verification

```powershell
# in sdkwork-agents
pnpm check:app-sdk-consumer-imports
pnpm check:architecture-alignment
pnpm check:rust-backend-composition

# in sdkwork-appstore
node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .
node ../sdkwork-specs/tools/check-workspace-member-protocol.mjs --root .
node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --workspace .
```
