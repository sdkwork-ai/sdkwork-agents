# SDKWork Agents Structured Call Specification

- Version: `1.4.0`
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Machine contract: [`agent-structured-call.contract.json`](./agent-structured-call.contract.json)
- Checker: `node scripts/check-agent-call-contract.mjs`
- Related:
  - [`../sdkwork-specs/API_SPEC.md`](../../sdkwork-specs/API_SPEC.md) (§14 v3 wire rules, §15 envelopes, §16 resource verb table)
  - [`../sdkwork-specs/CODE_STYLE_SPEC.md`](../../sdkwork-specs/CODE_STYLE_SPEC.md)
  - [`AGENTS_PROVIDER_TAXONOMY_SPEC.md`](./AGENTS_PROVIDER_TAXONOMY_SPEC.md)
  - [`AGENTS_KERNEL_BOUNDARY_SPEC.md`](./AGENTS_KERNEL_BOUNDARY_SPEC.md)
  - [`../sdkwork-intelligence-agents-service/specs/openapi/agents-app-api.openapi.yaml`](../crates/sdkwork-intelligence-agents-service/specs/openapi/agents-app-api.openapi.yaml) (wire authority)

## 1. Purpose

A managed agent must be invocable as a programmable data processor: the caller
submits a prompt or typed parameters, the agent executes, and the result is
returned as validated structured data (JSON, XML, or plain text) — with no
human interaction. This specification defines that capability as **Agent Call**.

Two entries exist:

| Entry | Surface | Consumer |
| --- | --- | --- |
| A. Programmatic call | `POST /app/v3/api/ai/agents/{agentId}/calls` (`agents.calls.create`, sync + async) | Host applications via `@sdkwork/agents-app-sdk`, external services |
| A'. Call observation | `GET .../calls` + `GET .../calls/{executionId}` (`agents.calls.list` / `agents.calls.retrieve`) | Same consumers; poll async executions |
| B. Agent-as-tool | Tool `agent_call` projected into a host agent turn tool loop | Managed agents orchestrating sub-agents |
| C. Usage metering | `GET /app/v3/api/ai/usage/summary` + `GET /app/v3/api/ai/usage/records` (`agents.usage.summary.retrieve` / `agents.usage.records.list`) | Commercial metering: token/turn totals and per-turn facts (§2.4) |
| D. Version governance | `POST`/`GET .../versions`, `GET .../versions/{versionId}`, `POST .../versions/{versionId}/activate` (`agents.versions.*`) | Publish, inspect, and roll back agent definitions (§2.5) |
| E. Event webhooks | `POST`/`GET /app/v3/api/ai/webhooks`, `GET`/`DELETE .../webhooks/{webhookId}`, `POST .../webhooks/{webhookId}/test` (`agents.webhooks.*`) | HMAC-signed outbound event delivery with a durable attempt ledger (§2.6) |

## 2. Call Contract

### 2.1 Request (`CreateAgentCallRequest`)

| Field | Type | Rules |
| --- | --- | --- |
| `executionId` | string | Caller-generated idempotency key, pattern `^execution\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$`, validated by `validate_standard_id` |
| `mode` | string enum | `prompt` \| `params`; required |
| `prompt` | string | Required when `mode=prompt`; non-empty |
| `params` | object | Required when `mode=params`; must validate against `paramSchema` before any model invocation |
| `paramSchema` | object | JSON Schema draft 2020-12; required when `mode=params`; caller-supplied schema wins but must be compatible with the agent's declared input contract |
| `output.format` | string enum | `json` \| `xml` \| `text`; default `json` |
| `output.schema` | object | JSON Schema draft 2020-12; optional; only valid when `format=json` |
| `output.rootElement` | string | Optional; only valid when `format=xml`; XML result must use this root element name |
| `output.strict` | boolean | Default `true`; strict mode fails the call when validation cannot be satisfied |
| `policy.timeoutMs` | integer | Default `60000`, maximum `300000` |
| `policy.maxTurns` | integer | Default `1`; the call is a single structured execution |
| `executionMode` | string enum | `sync` (default) \| `async`; see §2.3 |

Request bodies are `camelCase` with `deny_unknown_fields`. Int64 values are
strings on the wire (`sdkwork-specs/API_SPEC.md` §13.6).

### 2.2 Response (`AgentCallResponse`)

HTTP `201` with the `SdkWorkApiResponse` envelope (`data.item`):

| Field | Rules |
| --- | --- |
| `executionId` | Echoes the request idempotency key |
| `status` | `queued` \| `succeeded` \| `validation_failed` \| `agent_failed` \| `timeout` \| `failed` |
| `output` | Parsed structured value for `json`; string for `xml`/`text` |
| `rawText` | Model output before parsing; omitted unless `status != succeeded` or debug requested |
| `agentError` | Optional engine diagnostic; present only when `status=agent_failed` |
| `validation` | `{ valid: boolean, errors: string[] }` |
| `usage` | `{ durationMs, attempts, runtimeMode }` |
| `correlation` | `{ executionId, agentId }` — int64-free string identifiers |

Business failure (`validation_failed`, `agent_failed`, `timeout`) is carried in
`data.item.status`, not as a non-success `code` (API_SPEC §15.3: failures use
`ProblemDetail` at the HTTP layer only for transport/authorization errors).

### 2.3 Async Execution Lifecycle

`executionMode: "async"` decouples long calls from the HTTP connection:

| Step | Surface | Behavior |
| --- | --- | --- |
| 1. Queue | `POST .../calls` with `executionMode: "async"` | Input is fully validated first (malformed input never queues); a durable record with status `queued` is persisted in `ai_agent_runtime_execution`; the response is `202` with the queued record |
| 2. Execute | In-process queued worker | The record transitions `queued -> running -> terminal`; transitions are durable so a crash leaves an observable state |
| 3. Observe | `GET .../calls/{executionId}` (`agents.calls.retrieve`) | Returns the current record; terminal `status` uses the §2.2 vocabulary (`failed` is reserved for crash recovery) |
| 4. List | `GET .../calls` (`agents.calls.list`) | Keyset-paginated by `(requestedAt, executionId)` descending, optional `status` filter |
| 5. Recover | `recover_stale_agent_calls` (ops entry point) | Marks non-terminal records not updated since the cutoff as `failed` with an explicit recovery diagnostic; re-execution of terminal records is a conflict |

Uniqueness: `(tenant_id, agent_id, execution_id)` is enforced by the durable
store; a replayed `executionId` for a terminal call is a `409` conflict, never
a silent re-execution.

### 2.4 Usage Metering (`agents.usage.*`)

Commercial metering facts are exposed over the durable turn token columns
(`ai_agent_turn.input_tokens / output_tokens / cached_tokens`):

| Surface | Behavior |
| --- | --- |
| `GET /app/v3/api/ai/usage/summary` (`agents.usage.summary.retrieve`) | Aggregated `turnCount`, `sessionCount`, and token totals for the tenant scope; optional conjunctive `agentId` / `sessionId` / `modelId` filters plus an inclusive-`from` / exclusive-`to` RFC 3339 window |
| `GET /app/v3/api/ai/usage/records` (`agents.usage.records.list`) | Turn-level usage feed ordered by `(createdAt, id)` descending with an opaque scope-bound cursor; same filters |

Both responses carry int64 values as strings (API_SPEC §13.6). Billing,
orders and quota enforcement stay owned by the platform gateway; Agents owns
the metering facts only.

### 2.5 Version Governance (`agents.versions.*`)

Commercial deployment requires publish-visible, rollback-capable definitions.
The durable `ai_agent_version` store holds **immutable snapshots**: a version
row is write-once (manifest and metadata never change; only `activated_at`
transitions).

| Surface | Behavior |
| --- | --- |
| `POST /app/v3/api/ai/agents/{agentId}/versions` (`agents.versions.create`) | Snapshots the live definition as a new version; `versionNumber` increases monotonically, replayed `versionId` conflicts |
| `GET .../versions` (`agents.versions.list`) | Keyset-paginated by versionNumber descending |
| `GET .../versions/{versionId}` (`agents.versions.retrieve`) | Returns one immutable snapshot |
| `POST .../versions/{versionId}/activate` (`agents.versions.activate`) | Marks exactly one version active (single `activated_at` marker per agent) and writes its immutable manifest back onto the live definition — activation IS the rollback path |

`versionId` follows the standard id scheme with the `version.` prefix.
Snapshots are computed from the persisted agent definition and therefore
always reflect the exact payload a caller previously observed.

### 2.6 Event Webhooks (`agents.webhooks.*`)

Outbound event delivery for commercial integrations. A subscription binds an
**HTTPS-only** endpoint to a closed event vocabulary
(`agent_call.completed`, `agent_call.failed`, `task_run.completed`,
`task_run.failed`, `interaction.requested`).

| Surface | Behavior |
| --- | --- |
| `POST /app/v3/api/ai/webhooks` (`agents.webhooks.create`) | Registers the subscription and generates a signing secret (`whsec_` + 32 random bytes); the secret is echoed **exactly once** in the creation response and never returned again |
| `GET /app/v3/api/ai/webhooks` (`agents.webhooks.list`) | Offset-paginated list (low-volume config set); secrets redacted |
| `GET .../webhooks/{webhookId}` (`agents.webhooks.retrieve`) | Returns one subscription without its secret |
| `DELETE .../webhooks/{webhookId}` (`agents.webhooks.delete`) | Removes the subscription (`204` no content); past deliveries stay in the ledger |
| `POST .../webhooks/{webhookId}/test` (`agents.webhooks.test`) | Builds a signed `agent_call.completed` test payload, POSTs it with a bounded 10s timeout, and records the terminal delivery outcome |

Signature scheme (Stripe-compatible shape): the request carries
`Sdkwork-Signature: t=<unix-seconds>,v1=<hmac-sha256(secret, "<ts>.<payload>")>`.
The timestamp is part of the signed content, so replayed bodies fail
verification for any recipient that checks the skew window; comparison uses a
constant-time equality helper from `sdkwork-utils`.

Deliveries are recorded in the durable `ai_agent_webhook_delivery` ledger as
`queued` before the outbound attempt and completed `succeeded`/`failed` with
the response code (or bounded error detail) after it. Subscriptions and
deliveries use the standard id scheme with the `webhook.` / `delivery.`
prefixes; endpoints must be absolute HTTPS URLs (≤2048 chars).

## 3. Execution Pipeline

Authority: `sdkwork-agents-runtime-facade` `structured_call` module. The
service layer (`sdkwork-intelligence-agents-service`) composes agent,
authorization, persistence, and binding resolution; the facade owns format
mechanics only.

1. **Input validation** — `mode=params` payloads are validated against
   `paramSchema` before any model invocation; invalid input returns
   `validation_failed` without consuming model quota.
2. **Binding resolution** — an active agent-engine provider binding is
   required; structured calls fail closed (no `agents-contract-fallback`
   mode: a deterministic stub cannot guarantee schema conformance).
3. **Prompt composition** — agent system contract + output format directive
   (JSON Schema inline for `json`; root element and markup conventions for
   `xml`).
4. **Turn execution** — one `AgentEngineTurnInput` via
   `execute_agent_engine_turn`; no tools are exposed (pure data processing).
5. **Parse + validate** — `json`: strip code fences, `serde_json` parse,
   validate against `output.schema` (when present) via the `jsonschema`
   crate; `xml`: well-formedness via `roxmltree` plus root element check;
   `text`: returned verbatim.
6. **Repair retry** — on validation failure, exactly one repair turn is
   executed with the validation errors appended; `attempts` reports the
   total. Strict mode marks the call `validation_failed` when repair also
   fails; non-strict returns `rawText` with `validation.valid=false`.
7. **Persistence** — the call is recorded as an
   `AgentRuntimeExecutionRecord` with operation `agent_call` in the durable
   `ai_agent_runtime_execution` store (unique per
   `(tenant_id, agent_id, execution_id)`); input and output payloads are
   persisted JSON for audit and async replay.

## 4. Agent-as-Tool

- Tool id `agent_call`, projected through `MediaToolDefinition` conventions
  (`sdkwork-agents-tool-contract`): JSON Schema 2020-12 input, availability,
  and policy categories.
- The tool executes the same pipeline as entry A with the sub-agent's
  binding.
- **Nesting depth is 1**: a call executed inside a turn loop MUST NOT itself
  expose `agent_call` to its own tool loop. Recursion is rejected fail-closed.
- Tool timeout is sub-allocated from the host turn budget; the tool call
  fails (never blocks the host loop) when the sub-call exceeds it.

## 5. Boundaries

- Kernel SPI and provider adapters are untouched; Agent Call is an Agents
  application capability composed through `sdkwork-agents-runtime-facade`.
- Product consumers integrate only through `@sdkwork/agents-app-sdk`.
- The call pipeline does not read or write IM, appstore, or independent
  module tables; persistence stays in Agents-owned runtime execution storage.
- Generated SDK types and route constants are regenerated from the OpenAPI
  authority; hand-edited generated files are forbidden.

## 6. Verification

```powershell
node scripts/check-agent-call-contract.mjs
cargo test -p sdkwork-agents-runtime-facade structured_call
cargo test -p sdkwork-intelligence-agents-service agent_call
cargo test -p sdkwork-intelligence-agents-service usage
cargo test -p sdkwork-intelligence-agents-service agent_version
cargo test -p sdkwork-intelligence-agents-service webhook
pnpm check
```
