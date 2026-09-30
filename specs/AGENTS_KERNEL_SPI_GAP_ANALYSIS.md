# SDKWork Agents Kernel Capability Closure

- Version: `1.1.0`
- Status: active
- Updated: `2026-07-22`
- Owner: `agents-platform`
- Related:
  - [`AGENTS_KERNEL_BOUNDARY_SPEC.md`](./AGENTS_KERNEL_BOUNDARY_SPEC.md)
  - [`AGENTS_PROVIDER_TAXONOMY_SPEC.md`](./AGENTS_PROVIDER_TAXONOMY_SPEC.md)
  - [`AGENTS_SESSION_MODEL_SPEC.md`](./AGENTS_SESSION_MODEL_SPEC.md)

## 1. Current Closure

The Agents business layer and kernel runtime mechanism have one integration
boundary: `sdkwork-agents-runtime-facade`. Durable product execution uses the
Agents Project, Session, Turn, SessionItem and Interaction model. Provider
events remain kernel/runtime mechanisms and never become a parallel product
model.

| Area | Current authority | Status |
| --- | --- | --- |
| Runtime invocation | `sdkwork-agents-runtime-facade` over kernel provider SPI | closed |
| Durable execution | Agents Session aggregate | closed |
| Public API | 56 Open, 102 App, 58 Backend operations | closed |
| TypeScript SDKs | Open, App and Backend family package roots | closed |
| Flutter SDK | `sdkwork_agents_app_sdk` package root | closed |
| Typed streaming | `AgentTurnStreamEvent` delta and completion events | closed |
| Human pause points | `AgentInteraction` claim and resolution | closed |
| Persistence | 30-table PostgreSQL Agents module, including Task/Run/Attempt scheduling | closed |
| IM boundary | IM-owned opaque Session/Turn correlation | closed |

## 2. Provider Capability Policy

Provider availability is governed by manifest registration and conformance, not
by alternative APIs or persistence:

| Tier | Runtime policy | Product behavior |
| --- | --- | --- |
| T1 agent engines | bootstrapped and exposed in the default catalog | selectable |
| T2 autonomous engines | registered on demand after conformance | opt-in |
| T3 frameworks | selected by an approved implementation binding | fail closed when absent |
| T4 orchestration frameworks | require an approved provider manifest | unavailable until registered |

An unavailable provider produces a typed runtime error. Agents does not add raw
HTTP fallback code, provider-specific public resources or copied provider
configuration.

## 3. Streaming Contract

The public Turn command supports JSON completion and SSE. SSE emits:

```text
AgentTurnStreamEvent(eventType=delta, index, delta)
AgentTurnStreamEvent(eventType=completion, response=AgentTurnExecutionResponse)
```

`AgentTurnExecutionResponse.data.item` contains the canonical Session, Turn and
ordered Session Items. Provider-specific chunk granularity may vary, but the
public event schema and terminal completion semantics do not.

## 4. Independent Capability Integration

Memory, knowledge, skills, prompts, MCP, LLM profiles and Drive remain
independent modules. Agents uses stable identifiers or their public SDKs. A
deployment may enable additional capability modules without changing the
Session schema or SDK resource model.

This is an open-closed extension point: new provider or composition bindings are
registered through manifests and slots; core Session/Turn contracts stay
unchanged.

## 5. Commercial Readiness Gates

| Gate | Required evidence |
| --- | --- |
| API contract | envelope, operation-pattern, collision and pagination checks |
| SDK generation | TypeScript and Flutter outputs are idempotent and compile |
| Persistence | database framework validation and Postgres contract tests |
| Security | trusted request context, dual-token/App-Backend and API-key/Open checks |
| Runtime | facade, bridge and HTTP contract tests |
| Integration | no direct provider dependency and no IM reverse dependency |
| Operations | health, metrics, release, deployment and rollback runbooks |

No gate is satisfied by a compatibility layer, alternate session store,
duplicated DTO, generated-output edit or raw transport fallback.

## 6. Verification Loop

```powershell
cargo test -p sdkwork-agents-runtime-facade
cargo check -p sdkwork-agents-kernel-bridge
cargo test -p sdkwork-intelligence-agents-service --features http-axum
node sdks/workspace-agent-sdkgen.mjs --mode dry-run
node scripts/check-agent-sdk-workspace.mjs
pnpm check:agents-im-boundary
pnpm db:validate
```
