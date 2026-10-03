# Agents Database Schema

Status: active

Contract: `7.3.0`

Managed engine: PostgreSQL

## Ownership

All 32 tables are authored and written by `sdkwork-intelligence-agents-service`.
There are no imported tables, derived read stores, shadow copies, compatibility tables,
or cross-module foreign keys. External capabilities are represented only by
stable reference columns.

## Inventory

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Managed agent identity and lifecycle |
| `ai_agent_runtime_binding` | Agent-level provider configuration |
| `ai_agent_composition_slot` | Agent references to sibling-owned capabilities |
| `ai_agent_audit_event` | Immutable sanitized aggregate audit facts |
| `ai_agent_workspace` | Owner-scoped project container with one active default |
| `ai_agent_project` | Reusable orchestration project and access policy |
| `ai_agent_project_composition_slot` | Project references to sibling-owned capabilities |
| `ai_agent_session` | Single durable execution session authority |
| `ai_agent_session_runtime_binding` | Session runtime selection, provider Session lineage, and provider directory metadata |
| `ai_agent_execution_host` | Dispatchable execution host registry (docker, micro VM, bare metal, cloud-sandbox pool) |
| `ai_agent_session_execution_placement` | Durable per-session execution placement: requested/effective target, pinned host, lifecycle state, lease evidence |
| `ai_agent_turn` | Idempotent turn, retry, lease, fencing, usage, and terminal state |
| `ai_agent_turn_input_queue_entry` | Durable owner-scoped FIFO input awaiting Turn execution |
| `ai_agent_session_item` | Ordered typed transcript or execution item |
| `ai_agent_item_drive_ref` | Typed relation to Drive-owned resources |
| `ai_agent_item_feedback` | Per-user assistant-output quality feedback |
| `ai_agent_interaction` | Typed approval, user-question, elicitation or setup pause point with claim fencing |
| `ai_agent_session_checkpoint` | Provider or Drive-backed resumable checkpoint reference |
| `ai_agent_task` | Session-bound one-time or cron schedule definition |
| `ai_agent_task_run` | One logical scheduled, manual, or business-retry occurrence |
| `ai_agent_task_run_attempt` | One leased and fenced infrastructure delivery attempt |
| `ai_agent_resource_user_state` | Per-user session/project view preferences |
| `ai_agent_project_member` | Project collaboration ACL |
| `ai_agent_share_link` | Hashed, revocable, expiring grant |
| `ai_agent_outbox_event` | Transactional aggregate event facts awaiting an approved relay |
| `ai_agent_model_configuration_profile` | Owner-scoped applied model configuration persisted for the runtime facade |
| `ai_agent_tool_configuration` | Admin-managed per-tenant media tool configuration |
| `ai_agent_tool_asset` | Generated media asset persisted to Drive outside session items |

## Composition References

`ai_agent_composition_slot` and `ai_agent_project_composition_slot` share the
single mapping declared by `specs/AGENTS_DOMAIN_SPEC.md` section 3. PostgreSQL
enforces both the enum allow-lists and the exact pair. A document binding is
therefore `slot_kind=document`, `target_module=documents`; the row contains only
stable external references and bounded orchestration policy. Document content,
versions and lifecycle remain in `sdkwork-documents`.

## Session Aggregate

`ai_agent_session` stores only stable business scope, lineage, lifecycle, and
counters. Runtime/provider/model selection belongs to
`ai_agent_session_runtime_binding`. Ordered results belong to
`ai_agent_session_item`; artifacts are Drive references rather than copied
resource snapshots. Turns own idempotency and worker concurrency state.

Checkpoints store exactly one opaque provider checkpoint reference or one
Drive space/node pair. They never store plaintext resume tokens or unconstrained
provider state documents.

Interactions keep legacy options as a bounded array and store typed requests in
the separate bounded `request_json` object. Provider request ids, methods and
callbacks remain outside Agents persistence.

## Isolation And Concurrency

Every session-aggregate unique key and foreign key includes tenant and
organization scope. Application-allocated `BIGINT` primary keys are used.
Mutating aggregates use optimistic versions. Turn workers and interaction
claimers use bounded attempts, leases, opaque lease tokens, expirations, and
monotonic fencing tokens. The outbox schema is relay-ready, but external
publication is not claimed until the platform publisher SPI is integrated and
verified.

Task materializers and Run workers use bounded `FOR UPDATE SKIP LOCKED` scans.
The unique Task generation and scheduled instant prevents duplicate logical
occurrences. Raw Run lease tokens are returned once and only their SHA-256
hashes are persisted. A Run references one canonical Session and one
idempotent Turn; delivery retries never synthesize a Session or a second Turn.

Production PostgreSQL adapters allocate one process-level Snowflake node lease
through `sdkwork-database`; repository and audit writers share the same
generator sequence. Kubernetes supplies Pod UID as `SDKWORK_NODE_INSTANCE_ID`,
and readiness fails when the lease becomes unhealthy. Fixed production node IDs
are forbidden.

PostgreSQL list paths use tenant-leading indexes with stable `id` tie-breakers.
Retention scans use partial indexes. There are no IM delivery/read columns in
the execution aggregate; `last_read_item_sequence` is only a local Agents
user-view preference.
