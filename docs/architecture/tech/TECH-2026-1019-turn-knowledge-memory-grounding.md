# Agent Turn Knowledge And Memory Grounding

- Status: implemented (knowledge); memory port implemented, adapter pending
- Owner: sdkwork-agents
- Updated: 2026-10-19
- Related:
  - [`TECH_ARCHITECTURE.md`](TECH_ARCHITECTURE.md)
  - [`../../specs/AGENTS_DOMAIN_SPEC.md`](../../specs/AGENTS_DOMAIN_SPEC.md)
  - [`../../specs/AGENTS_KERNEL_BOUNDARY_SPEC.md`](../../specs/AGENTS_KERNEL_BOUNDARY_SPEC.md)
  - [`../../../sdkwork-knowledgebase/docs/architecture/tech/TECH-2026-06-09-knowledgebase-agent-rag-design.md`](../../../sdkwork-knowledgebase/docs/architecture/tech/TECH-2026-06-09-knowledgebase-agent-rag-design.md)

## 1. Problem

An Agent can bind knowledge and memory composition slots, and the management
profile already carries `knowledgeBaseIds` and `memoryEnabled`, but the turn
path never read either. A chat answer therefore never saw uploaded knowledge:
retrieval existed only inside the Knowledgebase product's own agent profile
chat.

## 2. Ownership

Agents owns conversation orchestration. Knowledge content, chunks, indexes, and
retrieval traces stay in `sdkwork-knowledgebase`; memory content stays in
`sdkwork-memory`. Agents stores composition references only
(`ai_agent_composition_slot`).

The turn path therefore consumes two narrow ports declared by the Agents
service (`crates/sdkwork-intelligence-agents-service/src/context_grounding.rs`):

- `KnowledgeGroundingResolver::resolve_knowledge(&GroundingRequest)`
- `MemoryGroundingResolver::resolve_memory(&GroundingRequest)`

Transport is never implemented in the service. The composition root
(`sdkwork-agents-kernel-bridge`) registers an adapter built over the capability
module's generated SDK.

## 3. Flow

1. `execute_committed_turn` loads one live composition-slot snapshot per turn
   (`list_turn_slots`). Toolkit, knowledge, and memory resolution all read that
   same snapshot, so a turn never observes a half-updated agent.
2. `resolve_knowledge_bindings` keeps enabled `knowledge -> knowledgebase` slots
   whose `target_ref` is a numeric space id (optionally `space:<id>`) or whose
   `policy_json` carries `spaceId`. Other slots are skipped with a warning.
3. `resolve_memory_bindings` keeps enabled `memory -> memory` slots with
   `policy_json.memoryPolicyRef` (or `policyRef`, or the `target_ref`).
4. Resolvers run before model execution. The caller's dual tokens are forwarded
   unchanged so the Knowledgebase App API derives the actor and enforces space
   ACLs.
5. `build_grounding_prompt` renders one bounded system block: numbered
   knowledge sources (`[1]`, `[2]`, ...), then user memory (`[M1]`, ...).
   Retrieved content is labelled untrusted reference data, never instructions.
6. Both prompt builders consume the block:
   - cloudrouter path: `build_chat_messages` inserts a `system` message after
     the agent system prompt/welcome and before history;
   - agent-engine path: `build_model_items` inserts a `system:` item in the same
     position.
7. A retrieval failure degrades to an ungrounded turn with a warning. Knowledge
   or memory outages never fail chat.

## 4. Adapter

`crates/sdkwork-agents-knowledgebase-bridge` implements the knowledge port over
`sdkwork-knowledgebase-app-sdk-generated-rust` (generator-owned App API
transport). It:

- maps bindings to `KnowledgeRetrievalRequest` (`hybrid` methods, citations on,
  no actor/tenant fields);
- executes the call on a scoped worker thread with its own current-thread Tokio
  runtime, so a turn already inside a Tokio blocking worker cannot nest
  runtimes;
- applies an 8-second default deadline and returns typed kernel provider errors.

Configuration: `SDKWORK_AGENTS_KNOWLEDGEBASE_BASE_URL` (App API origin).
Unset = capability module not mounted (grounding skipped). Set-but-invalid =
bootstrap failure, never a silent disable.

## 5. Citation Delivery

`TurnExecutionResult.citations` carries the same `KnowledgeCitation` set the
prompt rendered (`index`, `documentId`, `title`, `sourceUri`, `score`). The
service computes it once with `build_grounding_prompt` before the turn moves its
snippet vectors into the execution input, so marker numbers and truncation match
the prompt exactly.

`AgentTurnExecutionDto` exposes `citations` on the item, which means both
delivery modes carry them without a separate channel:

- non-streaming `POST .../turns` returns `data.item.citations`;
- streaming turns include the same item in the terminal `completion` SSE event.

Ungrounded turns and replayed turns serialize `[]`. The field is optional on the
wire so older clients keep decoding responses.

## 6. Memory Adapter

`crates/sdkwork-agents-memory-bridge` provides two transports selected by
`SDKWORK_AGENTS_MEMORY_MODE` (default `in_process`):

- `in_process`: bootstraps the memory data plane from the process database
  profile and calls `MemoryAppApi::create_retrieval` directly. No network hop;
  requires co-deployment and memory database access.
- `http`: calls the Memory App API through the generator-owned Rust SDK using
  `SDKWORK_AGENTS_MEMORY_BASE_URL`.

Both resolve `memoryPolicyRef` as a Memory space id (`4242` or `space:4242`),
pass the authenticated actor explicitly (in-process) or derive it from the
caller's dual tokens (HTTP), accept only active records, bound and de-duplicate
snippets, and render `[Mn]` markers with the existing prompt builder. Missing
optional configuration leaves memory disabled; invalid mode or origin fails
bootstrap.

## 7. Remaining Gates

- Durable per-session-item citations: the turn response carries citations, but
  replayed historical turns do not retain them. Persisting them belongs with the
  session item projection contract.
- Turn-level telemetry for grounding counts and degraded retrievals.

## 6. Verification

```powershell
cargo test -p sdkwork-intelligence-agents-service --lib context_grounding
cargo test -p sdkwork-intelligence-agents-service --lib turn_grounding
cargo test -p sdkwork-intelligence-agents-service --lib model_items_include_bounded
cargo test -p sdkwork-intelligence-agents-service --lib chat_messages_include_knowledge
cargo test -p sdkwork-agents-knowledgebase-bridge
cargo check -p sdkwork-agents-kernel-bridge
cargo check -p sdkwork-api-agents-assembly
```
