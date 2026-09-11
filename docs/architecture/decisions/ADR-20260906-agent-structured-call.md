# ADR-20260906 — Agent Structured Call (Agent Call)

- Status: accepted
- Date: 2026-09-06
- Authority: `specs/AGENTS_STRUCTURED_CALL_SPEC.md`
- Machine contract: `specs/agent-structured-call.contract.json`
- Verification: `node scripts/check-agent-call-contract.mjs`

## Context

Products and external services need managed agents to behave as programmable
data processors: submit a prompt or typed parameters, receive validated
structured output (JSON, XML, or plain text) — with no human interaction and
no chat surface. The existing `preview_responses` and `prompt_optimizations`
operations return free-text content and cannot guarantee schema conformance.

## Decision

1. **One capability, two entries.** `POST /app/v3/api/ai/agents/{agentId}/calls`
   (`agents.calls.create`, 201, `SdkWorkResourceData.item`) is the programmatic
   entry. The same pipeline is projected as the `agent_call` tool for host
   agent turn loops (nesting depth 1, recursion rejected fail-closed).
2. **Pipeline mechanics live in `sdkwork-agents-runtime-facade`
   (`structured_call`).** Model execution is injected through the
   `StructuredTurnExecutor` trait, so the pipeline stays engine-agnostic and
   unit-testable; binding resolution and authorization stay in
   `sdkwork-intelligence-agents-service`.
3. **Fail-closed binding.** Structured calls require an active agent-engine
   provider binding; the `agents-contract-fallback` mode is forbidden because
   a deterministic stub cannot guarantee schema conformance.
4. **Validate before and after.** `params` payloads are validated against the
   caller's `paramSchema` (JSON Schema draft 2020-12) before any model
   invocation; JSON output is validated against `output.schema` after parsing,
   with exactly one repair retry that appends the validation errors.
5. **No new persistence surface.** Calls are recorded as
   `AgentRuntimeExecutionRecord` rows with operation `agent_call`; the typed
   result (status/output/validation/usage/correlation) is the output payload.
6. **Wire authority.** `agents-app-api.openapi.yaml` is the single source for
   route constants and generated SDK types; `check-agent-call-contract.mjs`
   keeps spec, contract, OpenAPI, Rust, and the app SDK aligned.

## Consequences

- Business outcomes (`validation_failed`, `agent_failed`, `timeout`) resolve
  as HTTP 201 items, not `ProblemDetail`; transport and authorization failures
  remain problem responses.
- `jsonschema` (0.54) and `roxmltree` (0.21) join the workspace dependency
  set for schema validation and XML well-formedness checks.
- SDK consumers use `createAgentCall` from `@sdkwork/agents-app-sdk`; the
  generated TypeScript layer carries the wire types.
