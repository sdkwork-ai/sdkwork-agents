# SDKWork Agents Database Specification

- Version: `7.2.0`
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Compliance: L2 for business state, L3 for audit/security event state

## 1. Authorities

The database contract has one machine authority and one physical authority:

| Concern | Authority |
| --- | --- |
| Owned-table inventory and version | `database/contract/schema.yaml` |
| Table ownership registry | `database/contract/table-registry.json` |
| PostgreSQL schema | `database/ddl/baseline/postgres/0001_agents_baseline.sql` |
| Database lifecycle | `database/database.manifest.json` |
| Domain vocabulary | `specs/AGENTS_DOMAIN_SPEC.md` |
| Session aggregate | `specs/AGENTS_SESSION_MODEL_SPEC.md` |
| Platform rules | `../sdkwork-specs/DATABASE_SPEC.md` and `../sdkwork-specs/DATABASE_FRAMEWORK_SPEC.md` |

This document explains those executable contracts. It must not introduce a
table, column, relationship, or lifecycle that is absent from the authorities.

## 2. Bounded Context

`sdkwork-agents` is the only system of record for managed-agent business state
and durable agent execution:

```text
AgentWorkspace -> AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
```

The managed store uses PostgreSQL and owns exactly 30 tables. It has no
derived read tables, shadow tables, compatibility tables, dual-write path, or
second session aggregate. A consumer may render an Agent Session as a dialog,
but that presentation does not create another persistence vocabulary.

SQLite is not a supported engine for Agents persistence (DATABASE_SPEC:
authoritative-server persistence is PostgreSQL only). The database manifest
declares `engines: [postgres]` and no SQLite baseline exists.

## 3. Ownership Boundary

Agents owns orchestration state and stable external references only. The
following data remains authoritative in its source module:

| Capability | Owner | Agents may persist |
| --- | --- | --- |
| Runtime mechanics and provider SPI | `sdkwork-kernel` | Provider/runtime binding identifiers and bounded execution results |
| Skill definitions, packages and installations | `sdkwork-skills` | Composition `target_ref` and optional `target_version_ref` |
| Structured document content and versions | `sdkwork-documents` | `document/documents` composition references only |
| Prompt content and versions | `sdkwork-prompts` | Composition references |
| Memory content | `sdkwork-memory` | Composition references |
| Knowledge and indexes | `sdkwork-knowledgebase` | Composition references |
| Model/provider catalog and credentials | `sdkwork-llm` | Selected model/provider identifiers; never credentials |
| MCP server definitions | `sdkwork-mcp` | Composition references |
| File bytes, versions and access grants | `sdkwork-drive` | Typed Drive node relations |
| IM delivery, membership and correlation | `sdkwork-im` | Nothing; IM stores opaque Agents identifiers on its side |
| Product runtime locations and device-local mount state | Product application | Opaque `runtime_location_id` only |

There are no cross-module foreign keys, cross-module SQL joins, copied catalog
rows, serialized external snapshots, or writes to another module's tables.
External existence and authorization are checked through module contracts or
SDKs before a reference is accepted.

## 4. Owned Table Inventory

### 4.1 Managed-agent control plane

| Table | Profile | Responsibility |
| --- | --- | --- |
| `ai_agent` | tenant entity | Managed agent identity, manifest, visibility and lifecycle |
| `ai_agent_runtime_binding` | tenant entity | Agent-level provider configuration; at most one active binding per agent |
| `ai_agent_composition_slot` | relation entity | Ordered agent-level references to independent capabilities |
| `ai_agent_audit_event` | audit event | Immutable and sanitized aggregate audit facts |

`ai_agent` is the tenant-scoped root. Runtime and composition rows use scoped
foreign keys to it. Composition slot values are allow-listed by `slot_kind` and
`target_module`; `policy_json` is bounded, non-secret orchestration policy.

### 4.2 Project and access plane

| Table | Profile | Responsibility |
| --- | --- | --- |
| `ai_agent_workspace` | tenant entity | User-owned project container; exactly one active default per owner scope |
| `ai_agent_project` | tenant entity | Workspace-scoped orchestration project and project access policy |
| `ai_agent_project_composition_slot` | relation entity | Ordered project-level independent-capability references |
| `ai_agent_project_member` | relation entity | Project owner/editor/viewer collaboration ACL |
| `ai_agent_share_link` | tenant entity, L3 | Revocable and expiring project/session grant using token hashes |
| `ai_agent_resource_user_state` | user entity | Per-user project/session pin, hide, open and last-read item state |

Every Project has a non-null `workspace_id`. Default Workspace initialization is
idempotent and uses `workspace.default.<owner_user_id>` as its stable business
identifier. Historical Projects are assigned to their owner's default Workspace
before the foreign key is enabled. Project membership is an Agents collaboration
ACL, not an IM group. Share-link
rows persist only `token_hash` and a safe prefix; a raw token is returned once,
is never logged, and is never included in audit or outbox payloads.

Both composition tables enforce the canonical mapping in
`specs/AGENTS_DOMAIN_SPEC.md` section 3. PostgreSQL allow-lists `document` and
`documents`, requires that pair, and also includes the existing `tool/tools`
pair. The application service applies the same rule before persistence.

### 4.3 Session aggregate

| Table | Profile | Responsibility |
| --- | --- | --- |
| `ai_agent_session` | tenant entity | The sole durable agent execution session |
| `ai_agent_session_runtime_binding` | tenant entity | Current/previous runtime selection and provider Session lineage |
| `ai_agent_turn` | operational state | One idempotent execution command with retry, lease and fencing state |
| `ai_agent_turn_input_queue_entry` | operational state | Durable owner-scoped FIFO input awaiting one Turn execution |
| `ai_agent_session_item` | tenant entity | Ordered typed input, output, tool, artifact or status item |
| `ai_agent_item_drive_ref` | relation entity | Typed relation from an item to Drive-owned resources |
| `ai_agent_item_feedback` | user entity | Per-user positive/negative quality feedback for an item |
| `ai_agent_interaction` | operational state | Typed approval, user-question, elicitation or setup pause point with claimed resolution |
| `ai_agent_session_checkpoint` | operational state | Provider-backed or Drive-backed resumable checkpoint reference |

The aggregate relationships are tenant and organization scoped. Session
lineage uses `parent_session_id` together with `forked_from_turn_id`; both are
present or both are absent. Only one current runtime binding may exist for a
session. Provider Session identifiers are opaque and unique only inside their
provider/runtime scope.

### 4.4 Operational reliability

| Table | Profile | Responsibility |
| --- | --- | --- |
| `ai_agent_task` | operational state | Session-bound one-time or cron schedule definition |
| `ai_agent_task_run` | operational state | One logical occurrence with one idempotent Turn identity |
| `ai_agent_task_run_attempt` | operational state | One leased, fenced infrastructure delivery attempt |
| `ai_agent_outbox_event` | outbox event, L3 | Transactional aggregate facts awaiting an approved external relay |

Outbox rows are written in the same transaction as their aggregate mutation.
The current platform does not expose a generic publisher SPI, so external
delivery and every event-dependent release remain gated. A future relay must
use the approved platform publisher SPI; this module must not add a local Kafka
producer, raw HTTP dispatcher, or direct writer into downstream tables.

Task schedules, Runs and Attempts follow
`specs/AGENTS_TASK_SCHEDULING_SPEC.md`. PostgreSQL atomically materializes each
Task generation and scheduled instant once. A Run references the same tenant,
organization, agent, owner and persisted Session as its Task. Worker claims use
hashed lease tokens and monotonic fencing; infrastructure retries reuse the Run
and Turn, while business retries create a linked Run.

## 5. Common Data Rules

1. Business primary keys are application-allocated signed 64-bit `id` values.
   PostgreSQL sequences, identity columns and database-side business ID
   allocation are forbidden.
2. Public identifiers such as `agent_id`, `project_id`, `session_id`, `turn_id`,
   `item_id` and `interaction_id` have scoped unique constraints.
3. Every tenant-owned lookup and relationship includes `tenant_id`; resources
   with organization scope also include `organization_id`.
4. `tenant_id`, `organization_id`, owner/user identifiers and actor identifiers
   come from trusted IAM request context or an authorized target resource, not
   from client-selected current-tenant fields.
5. Optimistically mutable rows carry `version`; updates compare the expected
   version and increment it atomically.
6. JSON columns are type checked and size bounded. They do not contain secrets,
   raw tokens, provider credentials, signed URLs, file bytes, or unbounded
   provider payloads.
7. Lifecycle deletion/redaction uses explicit timestamps and actors. Physical
   deletion is reserved for approved retention processing.

## 6. Session, Turn and Item Invariants

### 6.1 Session

`ai_agent_session` owns counters and the ordered item sequence. The row enforces:

- non-negative item/token counters and `item_count <= last_item_sequence`;
- all-or-none source context (`source_module`, `source_context_kind`,
  `source_context_id`);
- all-or-none fork lineage;
- paired creation `idempotency_key` and `payload_hash`;
- unique creation idempotency per tenant, organization and owner;
- scoped agent, project, parent-session and fork-turn integrity.

The supported session kinds are assistant, coding, automation and IM dispatch.
The latter means an IM-owned caller dispatched work; it does not transfer IM
data ownership to Agents.

### 6.2 Turn

`ai_agent_turn` is the idempotency and worker-concurrency boundary:

- `(tenant_id, organization_id, owner_user_id, idempotency_key)` is unique;
- an optional `client_request_id` is unique in the same owner scope;
- retrying the same key with the same payload hash returns/reuses the existing
  turn; a different hash is a conflict;
- request and response items must belong to the same scoped session;
- token counts and retry counts are non-negative and bounded;
- a lease is all-or-none: owner, opaque lease token and expiry;
- each successful claim advances `fencing_token`; stale workers cannot commit;
- completed, failed and cancelled are terminal and cannot be overwritten by a
  late worker or cancellation request;
- `error_detail` is sanitized and bounded before persistence.

Inference and provider calls run outside a database transaction. Reservation,
state transition and completion are short compare-and-set transactions.

### 6.3 Session item

`ai_agent_session_item` has one unique positive `sequence` per session. Item
kinds are:

```text
user_input, system_instruction, assistant_output, reasoning, tool_call,
tool_result, artifact_reference, status_notice, error_notice
```

Items are immutable after completion except for explicit redaction lifecycle.
A redacted row retains identity, sequence, lineage and auditability, sets the
redacted status/time/actor, and removes sensitive content through the service's
redaction transaction. Silent in-place history rewriting is forbidden.

Tool-call rows require a tool name, call id and bounded arguments. Tool-result
rows require the matching call id and bounded result. Other item kinds cannot
carry tool payload columns. Parent items and turn links are scoped foreign keys.

### 6.4 Drive references and feedback

`ai_agent_item_drive_ref` stores only Drive stable identifiers and bounded
relation metadata. Roles are attachment, image, audio, generated output and
artifact. It does not persist bucket/object keys, provider URLs, credentials,
presigned URLs, or file bytes.

`ai_agent_item_feedback` is unique per item and user. It records a rating of
`1` or `-1` with bounded optional reason/comment fields. It is product quality
feedback, not delivery state or an IM reaction.

## 7. Interaction and Checkpoint Security

An interaction claim returns a random raw claim token once. The database stores
only `claim_token_hash` together with claim owner, expiry, fencing token and
version. Typed interactions store one bounded provider-neutral object in
`request_json`; provider request ids, methods and callbacks remain Kernel-private.
Legacy approval/answer and typed resolve commands must prove the token,
unexpired lease, fencing token and expected version. A successful resolution is
atomic and cannot be repeated by another service instance.

A checkpoint has exactly one backing form:

- provider checkpoint reference plus session runtime binding; or
- Drive space and node identifiers.

Checkpoint rows never store plaintext resume credentials, arbitrary provider
state documents or file bytes. Restore re-authorizes the session, turn/runtime
binding and checkpoint lifecycle before invoking a provider.

## 8. Transaction Boundaries

### 8.1 Create session

Validate referenced agent/project and external context, insert the session,
initialize counters, and write audit/outbox facts in one transaction. A repeated
idempotency key follows the payload-hash rule.

### 8.2 Execute turn

1. Lock the scoped session row.
2. Allocate `last_item_sequence + 1` for the request item.
3. Insert the request item and Drive relations.
4. Insert the requested turn with idempotency, availability and retry state.
5. Update session counters and write audit/outbox facts.
6. Commit before invoking the provider.
7. On completion, lock the session again, allocate ordered result items, update
   the turn terminal state and usage, update session totals, and write
   audit/outbox facts atomically.

Unlocked `MAX(sequence) + 1`, long-running provider calls inside transactions,
and dual writes to consumer stores are forbidden.

### 8.3 Claim workers

Turn and interaction workers claim eligible rows with bounded leases,
compare-and-set versions and monotonic fencing tokens. PostgreSQL workers use
indexed eligibility predicates and `FOR UPDATE SKIP LOCKED` where appropriate.
The outbox table and indexes are relay-ready, but no outbox claim or publisher
worker is active until the platform provides the approved publisher SPI.

### 8.4 Materialize and execute scheduled tasks

Due Task selection, occurrence insertion, schedule-cursor advancement and
outbox creation are one short transaction. Run claim and Attempt insertion are
a second short transaction. Provider execution occurs outside both. Run
heartbeat and completion compare the lease hash, fencing token, generation and
non-terminal state. Unknown provider outcomes enter reconciliation.

## 9. Query and Index Rules

All list/search filtering, authorization scope, stable ordering and bounds are
pushed to PostgreSQL. Loading a complete aggregate and slicing in application
memory is forbidden.

| Resource | Stable database ordering |
| --- | --- |
| Agents | `updated_at DESC, id DESC` inside tenant/organization/visibility scope |
| Projects | `updated_at DESC, id DESC` inside owner/member scope |
| Sessions | `updated_at DESC, id DESC` inside owner, project or agent scope |
| Turns | `created_at ASC, id ASC` inside one session |
| Session items | `sequence ASC, id ASC` inside one session |
| Interactions | `created_at DESC, id DESC` inside one session/status scope |
| Checkpoints | `created_at DESC, id DESC` inside one session/status scope |
| Tasks | `updated_at DESC, id DESC` inside agent/owner/status scope |
| Task Runs | `scheduled_for DESC, id DESC` inside one Task/status scope |
| Run Attempts | `attempt_no DESC, id DESC` inside one Run |
| Feedback analytics | `created_at DESC, id DESC` inside rating scope |

Repository list methods use bounded pagination and paired count queries when
the API response requires totals. Long-running worker queries use the dedicated
partial indexes declared by the baseline.

## 10. Audit, Outbox, Retention and Privacy

- Audit rows are append-only, carry trusted actor/request/trace scope and store
  bounded change summaries rather than full item content.
- Agent composition mutations emit the exact audit actions
  `composition_slot_created`, `composition_slot_updated` and
  `composition_slot_deleted`; project-scoped composition uses the corresponding
  `project_composition_slot_*` actions.
- Outbox payloads contain stable Agents identifiers and the minimum bounded
  event data required by a consumer.
- Claim tokens, share tokens, access credentials, provider secrets, signed
  URLs, raw prompts containing secrets and full tool results are excluded from
  audit/outbox/logging.
- Retention workers use `retention_until` indexes and preserve legal/audit
  requirements. Redaction and deletion write explicit lifecycle outbox facts.
- Every read and mutation rechecks tenant, organization, owner/member and
  aggregate lineage; a public identifier alone is never authorization.

## 11. Schema Lifecycle

PostgreSQL `0001_agents_baseline.sql` is the greenfield `7.2.0` authority and
contains the complete 30-table model. The application is pre-launch, so every
development, test, staging and release-candidate database must be created from
that baseline. Importing or reinterpreting databases created from an earlier
application contract is unsupported.

Shared development schemas use the ordered forward migrations in
`database/migrations/postgres/`. After the first production release, changes
continue to use forward migrations and the expand/contract rules from
`MIGRATION_SPEC.md`; baseline rewrites then stop.

## 12. Verification

```powershell
pnpm db:validate
pnpm db:materialize:contract
pnpm db:plan
cargo test -p sdkwork-intelligence-agents-service
pnpm test:database:postgres-live
```

The live PostgreSQL suite requires `SDKWORK_DATABASE_URL` plus
`SDKWORK_DATABASE_ADMIN_HOST`, `SDKWORK_DATABASE_ADMIN_DATABASE`,
`SDKWORK_DATABASE_ADMIN_USERNAME` and `SDKWORK_DATABASE_ADMIN_PASSWORD`.
`SDKWORK_DATABASE_ADMIN_PORT` and `SDKWORK_DATABASE_ADMIN_SSL_MODE` are optional.
The suite provisions and removes an isolated `sdkwork_ai_test_*` database and
schema. Its evidence includes concurrent Task occurrence materialization, Run
claiming, Attempt creation, lease recovery and stale-fence rejection. The
contract is complete only when table inventory, DDL, repository behavior, API
resources and generated SDK vocabulary agree on the full 23-table model.
