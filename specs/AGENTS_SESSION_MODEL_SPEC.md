# SDKWork Agents Session Model Specification

- Version: `5.3.0`
- Status: active
- Owner: `agents-platform`
- Authority: `AGENTS_DOMAIN_SPEC.md`

## 1. Aggregate

`AgentSession` is the only durable product session aggregate for agent
execution. Every product and integration uses this aggregate directly through
an Agents API or SDK.

```text
AgentProject
  `- AgentSession
       |- AgentSessionRuntimeBinding
       |- AgentTurn
       |    `- AgentSessionItem (ordered)
       |- AgentInteraction
       `- AgentSessionCheckpoint
```

The aggregate is transactional. It has no alternate session store, transcript
store, read-model table, shadow table, or dual-write path.

## 2. Session

An `AgentSession` carries:

- tenant, organization, owner, agent and optional Agent Project scope;
- `sessionKind`: `assistant`, `coding`, `automation` or `im_dispatch`;
- `entrySurface` and bounded source context identifiers;
- title, lifecycle, item counters, token totals and optimistic version;
- optional parent session and fork-turn lineage.

Creation is idempotent and requires `sessionKind`, `entrySurface`,
`idempotencyKey`, `payloadHash` and `requestedAt`. Tenant, organization and
owner are resolved only from trusted request context.

A replay in the same tenant, organization and owner scope with the same
idempotency key and payload hash returns the existing Session. Reusing the key
with a different payload, agent, project or explicit Session id is a conflict.

Every agent-nested Session lifecycle and child-resource operation validates
that the `{agentId}` path identity matches the persisted Session. Mismatches
fail as not found so callers cannot enumerate resources across agent paths.

T1 agent-engine identities advertised by the default runtime catalog are stable
Kernel manifest identities. Before validating a new Session reference, Agents
idempotently materializes the tenant business-agent projection and its provider
binding. Arbitrary, unavailable and opt-in agent identities still fail closed;
products never create these projections or substitute legacy aliases.

A session does not contain IM resource identities, an embedded Workspace
snapshot, filesystem paths, copied catalogs, raw credentials or unbounded
provider metadata.

## 3. Runtime Binding

`AgentSessionRuntimeBinding` records the selected runtime for one session:

- Agents provider binding and model references;
- an opaque `runtimeLocationId` mapped by Kernel from the active
  `SandboxRuntimeBindingId` when Sandbox execution is used;
- host mode and transport kind;
- provider session, tree, parent and fork identifiers;
- lifecycle and optimistic version.

Only one current binding may be active for a session. Runtime-location details
remain owned by the product or runtime-location module.

### 3.1 Execution Placement

Where a session executes is resolved per turn as an execution target:
`in_process` (the current service process), `cloud` (kernel-coordinated cloud
execution; the legacy `sandbox` route code is an accepted alias), or `host`
(a registered dedicated execution host). Resolution order is request override,
deployment default, built-in `in_process`; an unknown code fails closed. The
vocabulary lives in `sdkwork-agents-runtime-facade` `execution_route`.

The durable placement facts are session-owned rows, never fields on the
runtime binding (the combined runtime binding must not gain physical fields):

- `ai_agent_execution_host` is the dispatchable host registry. One row per
  docker daemon, micro VM, bare-metal box, or kernel cloud-sandbox pool
  member, with host kind, opaque dispatch endpoint, region, capacity and
  lifecycle status. `tenant_id`/`organization_id` use the platform sentinel
  `0` for platform-shared hosts. Host selection is placement-owned and never
  client-writable.
- `ai_agent_session_execution_placement` is the durable placement binding.
  At most one current placement per session records the requested and
  effective target, the pinned host (host target only), the opaque kernel
  placement reference, the placement lifecycle state
  (`requested`/`allocating`/`ready`/`active`/`releasing`/`released`/`failed`/`expired`)
  and bounded lease evidence (`lease_owner`, `lease_expires_at`,
  `fencing_generation`). Lease credentials stay kernel-owned and are never
  persisted.

Scheduling reads these surfaces: the host registry answers which hosts of a
kind are eligible, the placement binding answers which session runs where
(session affinity, per-host drain, reconciliation of stale placements).
Lease and fencing transitions follow
`agent-execution-placement-orchestration.contract.json`; the runtime
lifecycle implementation (kernel placement port, automatic recording at turn
claim) remains gated there and lands as a separate reviewed change.

## 4. Turn

An `AgentTurn` is one idempotent execution command. It owns:

- client request and idempotency keys plus payload hash;
- request and response item references;
- mode, lifecycle, model/provider selection and usage;
- sanitized error and trace information;
- bounded retry, availability, lease and fencing state;
- start, completion, cancellation and retention timestamps.

Creation requires `content`, `turnMode`, `idempotencyKey`, `payloadHash` and
`requestedAt`. A retry uses the same idempotency key. A different payload hash
for that key is rejected. Terminal states are `completed`, `failed` and
`cancelled`.

## 5. Session Item

An `AgentSessionItem` is an ordered, typed execution fact. Its kind is one of:

```text
user_input
system_instruction
assistant_output
reasoning
tool_call
tool_result
artifact_reference
status_notice
error_notice
```

Item state is `pending`, `completed`, `failed`, `cancelled` or `redacted`.
Completed content is immutable except for governed redaction. Text is bounded;
tool arguments and results use typed JSON contracts; artifacts use
`ai_agent_item_drive_ref`. Agents stores no raw bytes, signed URLs, IM delivery
state or read cursors.

## 6. Interaction

`AgentInteraction` is a durable pause point linked to a session and optionally a
turn. Supported kinds are approval and user question. Claim and resolution use
optimistic versioning plus a one-time claim token so concurrent service
instances cannot resolve the same interaction.

Provider-specific interaction identifiers are opaque references. They are not
public resource identities.

## 7. Checkpoint

`AgentSessionCheckpoint` represents a resumable provider or logical execution
point. It stores a stable provider checkpoint reference or Drive-backed state
reference, never an unconstrained state document or plaintext resume token.

Restore validates tenant, organization, owner, agent, session, runtime binding,
checkpoint lifecycle and expected version before invoking the runtime.

## 8. Product Integration

### 8.1 Current-State Activity Snapshot

The App API exposes `GET /app/v3/api/ai/session_activity_summaries` as the
canonical owner-scoped Session list projection. It is a bounded current-state
snapshot, not a second Session authority and not a durable change feed.

Each row composes the durable Session, latest Turn, one deterministic pending
Interaction, current runtime binding, the authenticated owner's Session
resource user-state, provider session identity, revision freshness and an
effective presentation phase. Pending approvals take priority over pending user
questions; the newest Interaction wins within the same kind. User-state changes
such as pin, hidden, unread position and custom title participate in
`activityAt` with the `user_state` source, so cross-application presentation
changes converge through the same head scan. Soft-deleted Sessions remain
visible as `deleted` tombstones when they fall in the requested snapshot page.
`latestInteractionId`/`latestInteractionVersion` and
`latestRuntimeBindingId`/`latestRuntimeBindingVersion` remain present even when
the corresponding `pendingInteraction` or `currentRuntimeBinding` becomes null.
Versions are monotonic only within the matching identity, not across the
Interaction or RuntimeBinding collection. Consumers compare the complete
`activityAt + identity + version` evidence to reject stale snapshots and clear
previous pending/current presentation state.
`currentRuntimeBinding` follows the database invariant and is always active;
`latestRuntimeBinding` retains the latest full binding record. When there is no
active current binding and that latest record is `failed`, the presentation
phase is `failed` only when its `updatedAt` is at least as recent as the
canonical latest Turn. A later Turn may therefore supersede an older binding
failure, while a later binding failure supersedes an older Turn phase. Latest
`deactivated` or `deleted` records are tombstones for clearing current binding
state and do not masquerade as failures.

The canonical `latestTurn` is the Turn with the greatest immutable internal
Turn id (the model has no separate Turn sequence field), independent of later
updates to older Turns. A separate Turn activity projection allows any Turn
update to advance `activityAt` without replacing the canonical latest Turn or
changing the presentation phase to an older execution.

Rows are ordered by `(activityAt DESC, internal Session id DESC)`. A null
cursor starts a new head snapshot. An opaque cursor continues only the current
bounded traversal and is bound to tenant, organization, owner, workspace,
project and agent filters. Consumers must start each refresh cycle at the null
cursor and may then follow `nextCursor`; they must not retain a cursor as a
change-feed watermark. Moving heads converge on the next null-cursor refresh.

Provider Session runtime observations are query-time evidence and are never persisted as
fake Turns, Interactions or shadow-table records. Only `fresh` provider status,
event, lock or process evidence with both `observedAt` and a future bounded
`freshUntil` may change the effective phase. `stale`,
`unsupported` and `unavailable` observations project to `unknown`, never to
`ready` or `idle`. Consumers must fail closed when `freshUntil` expires. Static
history inventory and file modification timestamps are not live activity.
Deleted, archived or closed Sessions, persisted pending Interactions, a latest
failed runtime binding when no active current binding exists, and leased
requested/running Turns outrank provider evidence. For an active Session otherwise projected as `ready`, `idle`,
`completed` or `unknown`, fresh provider `working`, `waiting` or `failed` evidence
overrides the settled phase so another application's live execution remains
visible.

The provider activity interfaces currently define ingestion and lookup, but
external Codex app-server, Claude hook, OpenCode event and Gemini AgentEvent
collectors are separate runtime wiring. Without collector ingestion, the
snapshot reports the provider observation as `unsupported` rather than guessing.

The current PostgreSQL implementation performs one bounded-result projection
query and removes application-layer N+1 reads, but the baseline schema does not
provide an owner-scoped cross-component activity-head index. A large owner scope
may therefore require a broad CTE scan and sort before applying the page limit.
A production store-bounded plan requires human-reviewed schema work for a
materialized/equivalent Session activity-head projection indexed by owner and
`activityAt`; this contract does not authorize that migration.

Provider observations are enriched only after the durable page is selected. A
provider-only activity change therefore cannot promote an otherwise old
Session into the durable head page; Turn ingestion remains the discovery path
until the human-reviewed activity-head projection also ingests provider
activity.

The in-memory activity index normalizes RFC3339 offsets to UTC and retains the
greatest observed activity time, but the persisted model currently stores
client-supplied lifecycle timestamps without a server-owned monotonic activity
revision. A regressing client clock can therefore weaken PostgreSQL ordering;
closing that gap requires a contract and schema decision for a server-owned
revision or strict cross-resource timestamp guards.

Workspace filtering excludes deleted Projects. Because project deletion does
not currently emit a Session-scope tombstone, a consumer caching a filtered
workspace cannot infer removals from this snapshot alone. Project deletion
cascade/tombstone semantics must be defined by the owning Project contract
before that cache-eviction path is complete.

### 8.2 Product Mapping

Product applications map their domain context to stable Agents references:

| Product fact | Agents resource |
| --- | --- |
| Project-level orchestration context | `AgentProject` |
| Durable execution context | `AgentSession` |
| One idempotent execution request | `AgentTurn` |
| Ordered input, output, tool or artifact fact | `AgentSessionItem` |
| Runtime selection | `AgentSessionRuntimeBinding` |
| Resume point | `AgentSessionCheckpoint` |
| Approval or user question | `AgentInteraction` |

Agents owns Workspace identity and Project membership. Products own their local
UI selection and device state, retain only opaque Agents resource identifiers
needed for correlation, and do not persist a second agent
execution transcript.

## 9. Verification

```powershell
cargo test -p sdkwork-intelligence-agents-service --features http-axum --lib
cargo test -p sdkwork-intelligence-agents-service --features http-axum --test http_axum_contracts
pnpm check:agents-im-boundary
pnpm db:validate
```
