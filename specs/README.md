# SDKWork Agents Local Specifications

Application narrowing rules for the hosted Agents product. Agent kernel SPI contracts
remain authoritative under `../sdkwork-kernel/specs/`.

## Architecture Authority

| Document | Purpose |
| --- | --- |
| [AGENTS_DOMAIN_SPEC.md](./AGENTS_DOMAIN_SPEC.md) | Canonical Agents bounded context and Project/Session/Turn/Item/Interaction vocabulary |
| [AGENTS_SESSION_MODEL_SPEC.md](./AGENTS_SESSION_MODEL_SPEC.md) | Durable session aggregate, runtime binding, item, interaction, and checkpoint contract |
| [AGENTS_TASK_SCHEDULING_SPEC.md](./AGENTS_TASK_SCHEDULING_SPEC.md) | Durable Task, Run, Attempt, cron, lease, fencing, retry, and reconciliation contract |
| [AGENTS_STRUCTURED_CALL_SPEC.md](./AGENTS_STRUCTURED_CALL_SPEC.md) | Structured agent call contract: prompt/params in, validated JSON/XML/text out, agent-as-tool projection |
| [agent-structured-call.contract.json](./agent-structured-call.contract.json) | Machine-readable structured-call invariants and wire-authority alignment (`node scripts/check-agent-call-contract.mjs`) |
| [AGENTS_KERNEL_BOUNDARY_SPEC.md](./AGENTS_KERNEL_BOUNDARY_SPEC.md) | Kernel vs agents vs product boundary (frozen) |
| [AGENTS_PROVIDER_TAXONOMY_SPEC.md](./AGENTS_PROVIDER_TAXONOMY_SPEC.md) | Code / autonomous / framework agent taxonomy |
| [AGENTS_KERNEL_SPI_GAP_ANALYSIS.md](./AGENTS_KERNEL_SPI_GAP_ANALYSIS.md) | Kernel capability closure and commercial readiness gates |
| [AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md](./AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md) | Mandatory `sdkwork-im -> sdkwork-agents` dependency direction and database ownership boundary |
| [AGENTS_APPSTORE_CONSUMER_BOUNDARY_SPEC.md](./AGENTS_APPSTORE_CONSUMER_BOUNDARY_SPEC.md) | Mandatory `sdkwork-appstore -> sdkwork-agents` consumer direction, SDK consumption surface, and storefront/runtime ownership boundary |
| [AGENTS_AI_COMPOSITION_DATABASE_SPEC.md](../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md) | Canonical 23-table Agents PostgreSQL contract |
| [agent-task-scheduling.contract.json](./agent-task-scheduling.contract.json) | Machine-readable Task scheduling invariants and review authority |
| [agent-interaction-envelope.contract.json](./agent-interaction-envelope.contract.json) | Machine-readable typed Interaction request, resolution, compatibility, and review authority |
| [agents-birdcoder-alignment.spec.json](./agents-birdcoder-alignment.spec.json) | Machine-readable cross-repo alignment tracker |
| [agent-execution-placement-orchestration.contract.json](./agent-execution-placement-orchestration.contract.json) | Draft machine contract for explicit local/cloud intent, Agents orchestration, and Kernel-owned execution placement |
| [docs/architecture/AGENTS_LAYERING.md](../docs/architecture/AGENTS_LAYERING.md) | Crate and SDK layering |
| [docs/product/prd/PRD.md](../docs/product/prd/PRD.md) | Product requirements |
| [docs/architecture/tech/TECH_ARCHITECTURE.md](../docs/architecture/tech/TECH_ARCHITECTURE.md) | Technical architecture |

## Platform Framework Alignment

| Framework | Status | Integration point |
| --- | --- | --- |
| `sdkwork-web-framework` | **Integrated** | `sdkwork-routes-agents-*` + `build_served_combined_router` in kernel-bridge |
| `sdkwork-database` | **Integrated** | `database/` assets, `sdkwork-agents-database-host`, managed-store postgres path |
| `sdkwork-utils` | **Integrated** | `SdkWorkApiResponse`, `parse_bool`, `is_blank`, `trim`, `uuid` across contract/service/facade |
| `sdkwork-drive` | **Integrated for PC upload scope** | PC core uses `@sdkwork/drive-app-sdk` Drive Uploader; Agents session items and slots retain canonical Drive references only |
| `sdkwork-discovery` | **Inactive** | No first-party RPC services yet |

## Independent Module Integration

`sdkwork-agents` consumes independent capability modules; those modules do not
depend on `sdkwork-agents` for their core domain behavior.

| Module | Integration mode | SDK / contract |
| --- | --- | --- |
| `sdkwork-memory` | `slot_kind=memory` composition slot | `@sdkwork/memory-app-sdk` (when mounted) |
| `sdkwork-knowledgebase` | `slot_kind=knowledge`, `target_module=knowledgebase` | `@sdkwork/knowledgebase-app-sdk` |
| `sdkwork-skills` | `slot_kind=skill`, `target_module=skills` | `@sdkwork/skills-app-sdk` |
| `sdkwork-prompts` | `slot_kind=prompt`, `target_module=prompts` | `@sdkwork/prompts-app-sdk` |
| `sdkwork-documents` | `slot_kind=document`, `target_module=documents` | `@sdkwork/documents-app-sdk` |
| `sdkwork-mcp` | `slot_kind=mcp`, `target_module=mcp` | public SDK/runtime integration with stable MCP references |
| `sdkwork-llm` | runtime binding / model provider profile | model catalog, provider profile, credential references |
| `sdkwork-drive` | `slot_kind=drive`, `target_module=drive` | `@sdkwork/drive-app-sdk`; Drive Uploader only |

Chat file library: chat-uploaded files are marked in Drive with the `app_public`
node property `agents.chat_file_library` (written by the PC core upload service)
and the library is listed through the Drive app API `propertyNodes.list`
(`GET /app/v3/api/drive/properties/{propertyKey}/nodes`). Agents rows keep only
the canonical Drive references; no raw bytes or markers are stored in Agents
tables.

Search indexing and generated-media workflows remain independent capabilities.
They integrate through approved public contracts and are not copied into the
Agents database or SDK authorities.

Do not reverse the dependency: memory, knowledgebase, skills, prompts, documents,
mcp, llm, and drive own their tables, APIs, SDKs, and runtime contracts. Agents stores only
references and orchestration policy. The root `specs/component.spec.json` declares
these entries as `dependencyMode=independent-capability-module` with
`reverseDependencyPolicy=forbidden`, and `pnpm check:architecture-alignment`
validates that the direction remains unchanged.

## Kernel Dependency

Runtime composition uses sibling checkout `../sdkwork-kernel` per `DEPENDENCY_MANAGEMENT_SPEC.md`.

Products (including `sdkwork-birdcoder`) MUST consume agent runtime through
`sdkwork-agents-runtime-facade` and `@sdkwork/agents-app-sdk`, not `sdkwork-agent-provider-*`.

`sdkwork-im` is an Agents consumer. The mandatory direction is
`sdkwork-im -> sdkwork-agents -> sdkwork-kernel`; Agents MUST NOT import IM SDKs,
read or write `im_*` tables, or persist IM communication ownership.
See `AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md`.

`sdkwork-appstore` is an Agents consumer through `@sdkwork/agents-app-sdk` only
(bound in the appstore `*-core` SDK client inventory). Agents MUST NOT depend on
the appstore, and Agents runtime state never lives in appstore stores. Storefront
catalog pages such as the appstore expert page are appstore presentation surfaces,
not Agents contracts. See `AGENTS_APPSTORE_CONSUMER_BOUNDARY_SPEC.md`.

Client composition authority: `APP_COMPOSITION_SPEC.md` via `pnpm check:app-composition` (`verify-repo.mjs`). Do not add `dependency.composition.json`.

Client SDK boundary: only `*-core` packages may depend on generated app SDKs (`@sdkwork/agents-app-sdk`, `@sdkwork/knowledgebase-app-sdk`). Capability packages and app shells consume types and clients through `*-core/sdk` exports. Enforced by `pnpm check:architecture-alignment`.

## Verification

```powershell
pnpm verify
pnpm check
pnpm check:api-operation-patterns
pnpm check:route-path-collisions
pnpm check:pagination
pnpm check:app-sdk-consumer-imports
pnpm check:agent-sdk-workspace
pnpm check:component-port-bindings
pnpm check:frontend-composition
pnpm check:permission-composition
pnpm check:composition-resolver
pnpm check:rust-backend-composition
pnpm check:production-security
pnpm deploy:validate:cloud
pnpm check:architecture-alignment
```
