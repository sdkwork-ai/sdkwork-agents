# SDKWork Agents Kernel Boundary Specification

- Version: `1.1.0`
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Related:
  - [`../../sdkwork-kernel/specs/AGENT_KERNEL_SPEC.md`](../../sdkwork-kernel/specs/AGENT_KERNEL_SPEC.md)
  - [`../../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md`](../../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md)
  - [`AGENTS_SESSION_MODEL_SPEC.md`](./AGENTS_SESSION_MODEL_SPEC.md)
  - [`AGENTS_PROVIDER_TAXONOMY_SPEC.md`](./AGENTS_PROVIDER_TAXONOMY_SPEC.md)

## 1. Boundary

```text
product applications and sdkwork-im
                |
                v
         sdkwork-agents
         - managed business domain
         - durable Project/Session/Turn/SessionItem/Interaction
         - open/app/backend APIs and SDKs
                |
                v
         sdkwork-kernel
         - runtime SPI and provider mechanisms
```

Products consume Agents HTTP/SDK surfaces or the public runtime facade. They do
not depend directly on provider crates or kernel operational APIs.

## 2. Kernel Ownership

Kernel owns:

- runtime SPI for models, tools, MCP, skills, context, knowledge, policy and
  telemetry;
- provider plugins and typed protocol adapters;
- transient invocation state, provider event streams and runtime-local state;
- internal operational health, metrics and internal APIs.

Kernel does not own managed-agent CRUD, tenant policy, Agents `ai_*` tables,
public Agents APIs, product workspaces or durable product execution history.

## 3. Agents Ownership

Agents owns:

- managed agents, provider bindings, composition slots and audit;
- Agent Workspaces, Workspace-scoped Projects, Sessions, Turns, Session Items
  and Interactions;
- runtime bindings, checkpoints, tasks, per-user state, sharing and outbox;
- the public Open, App and Backend API authorities;
- generated SDK families and the product-facing runtime facade.

The current authored API inventories are:

| Surface | Prefix | Operations | SDK |
| --- | --- | ---: | --- |
| Open API | `/agent/v3/api` | 56 | `@sdkwork/agents-sdk` |
| App API | `/app/v3/api` | 102 | `@sdkwork/agents-app-sdk`, `sdkwork_agents_app_sdk` |
| Backend API | `/backend/v3/api` | 58 | `@sdkwork/agents-backend-sdk` |

Counts are derived from the three authored OpenAPI documents. The Open API uses
`X-API-Key`; App and Backend APIs use the SDKWork dual-token request context.

## 4. Persistence Boundary

The Agents managed PostgreSQL store contains 32 Agents-owned `ai_*` tables. Its
authority is
[`AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`](../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md).

The Session aggregate uses `ai_agent_session`, `ai_agent_turn`,
`ai_agent_session_item`, `ai_agent_interaction`, runtime binding and checkpoint
tables. Kernel runtime persistence is private implementation state and is not a
product read authority.

There are no cross-module foreign keys, shared write ownership, shadow tables,
or alternate product session stores.

## 5. Runtime Facade

`sdkwork-agents-runtime-facade` is the supported in-process boundary for:

| Capability | Facade module | Kernel mechanism |
| --- | --- | --- |
| Runtime host | `runtime_host.rs` | provider registry and runtime slots |
| Turn execution | `turn.rs` | model provider invocation |
| Engine catalog | `engine_catalog.rs` | registered provider capabilities |
| Human interaction | `live_interaction.rs` | approval and question callbacks |

`sdkwork-agents-kernel-bridge` adapts this facade into the application service.
Products do not import `sdkwork-agent-kernel` or provider crates.

## 6. Independent Capabilities

| Capability | Owner | Agents relation |
| --- | --- | --- |
| Memory | `sdkwork-memory` | composition reference |
| Knowledge | `sdkwork-knowledgebase` | composition reference |
| Skills | `sdkwork-skills` | stable skill/version reference |
| Prompts | `sdkwork-prompts` | project instruction reference |
| MCP | `sdkwork-mcp` | composition reference and public SDK integration |
| LLM profiles/catalog | `sdkwork-llm` | provider/model/profile reference |
| Files and artifacts | `sdkwork-drive` | Drive resource reference and uploader SDK |

The dependency direction is `sdkwork-agents -> independent capability module`.
Agents does not reproduce their tables, package entities or business APIs.

## 7. Verification

```powershell
cargo test -p sdkwork-agents-runtime-facade
cargo check -p sdkwork-agents-kernel-bridge
cargo check -p sdkwork-api-agents-assembly
cargo test -p sdkwork-intelligence-agents-service --features http-axum
pnpm check:agent-sdk-workspace
pnpm check:architecture-alignment
```
