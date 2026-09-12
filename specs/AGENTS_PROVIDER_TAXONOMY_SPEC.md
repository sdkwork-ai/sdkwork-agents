# SDKWork Agents Provider Taxonomy Specification

- Version: `1.1.0`
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Related:
  - [`../../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md`](../../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md)
  - [`AGENTS_KERNEL_BOUNDARY_SPEC.md`](./AGENTS_KERNEL_BOUNDARY_SPEC.md)

## 1. Purpose

This specification defines stable provider-family vocabulary. Kernel implements
provider mechanisms; Agents owns managed-agent policy, durable execution and
public APIs; product applications consume Agents.

## 2. Provider Family Tiers

| Tier | Family | Description | Examples |
| --- | --- | --- | --- |
| T1 | Code agents | IDE and CLI coding engines with an official SDK or typed protocol | Codex, Claude Code, Gemini CLI, OpenCode |
| T2 | Autonomous agents | Self-directed engines with tool and MCP loops | OpenClaw, Hermes Agent |
| T3 | Framework agents | In-process frameworks exposed through kernel plugins | Rig |
| T4 | Orchestration frameworks | Manifest-selected frameworks enabled only by an approved provider binding | LangGraph, CrewAI, AutoGen, Semantic Kernel, OpenAI Agents |

Tier describes integration and conformance policy. It does not create a second
session, turn or persistence model.

## 3. Kernel Provider Matrix

| Engine key | Kernel provider crate | Binding id | Integration | Default catalog |
| --- | --- | --- | --- | --- |
| `codex` | `sdkwork-agent-provider-codex` | `binding.codex` | official SDK / typed native adapter | yes |
| `claude-code` | `sdkwork-agent-provider-claude-code` | `binding.claude-code` | official SDK | yes |
| `gemini` | `sdkwork-agent-provider-gemini-cli` | `binding.gemini-cli` | official CLI/SDK adapter | yes |
| `opencode` | `sdkwork-agent-provider-opencode` | `binding.opencode` | official SDK and OpenAPI adapter | yes |
| `openclaw` | `sdkwork-agent-provider-openclaw` | `binding.openclaw` | plugin binding | opt-in |
| `hermes` | `sdkwork-agent-provider-hermes` | `binding.hermes` | plugin binding | opt-in |
| `rig` | `sdkwork-agent-provider-rig` | provider manifest binding | Rust crate plugin | manifest-selected |

The app catalog endpoint is `GET /app/v3/api/ai/agent_engines`. Catalog entries
are runtime capabilities, not copied provider configuration.

## 4. Capability Ownership

| Capability | Runtime authority | Agents integration |
| --- | --- | --- |
| Model invocation and streaming | kernel model provider SPI | `AgentTurn` execution |
| Tool invocation | kernel tool provider SPI | typed session items |
| MCP execution | kernel MCP provider SPI and `sdkwork-mcp` | stable composition reference |
| Memory context | kernel context SPI and `sdkwork-memory` | stable composition reference |
| Knowledge | kernel knowledge SPI and `sdkwork-knowledgebase` | stable composition reference |
| Skills | kernel skill SPI and `sdkwork-skills` | stable skill/version reference |
| Approval and user question | kernel/provider callback plus Agents command | `AgentInteraction` |
| Durable product history | Agents | Session, Turn and SessionItem aggregate |

Independent modules own their entities, APIs and tables. Agents stores bounded
references and orchestration policy only.

## 5. Product Model Mapping

| Product concept | Agents authority |
| --- | --- |
| Managed definition | `Agent` and provider bindings |
| Reusable orchestration context | `AgentProject` |
| Durable execution context | `AgentSession` |
| Idempotent invocation | `AgentTurn` |
| Ordered input/output/tool/artifact fact | `AgentSessionItem` |
| Human pause and resolution | `AgentInteraction` |
| Scheduled command | `AgentTask` |

Provider Session identifiers remain inside runtime bindings, turns, checkpoints
or interactions as opaque values.

## 6. Implementation Type

`ai_agent.implementation_type` may select `sdkwork-native`, `rig-rust`,
`openai-agents`, `langchain`, `langgraph`, `crewai`, `autogen`,
`semantic-kernel` or `custom`. A value is executable only when an approved
provider manifest and conformance-tested binding are registered. Unknown or
unavailable bindings fail closed.

## 7. Integration Policy

1. Use the official SDK or typed protocol when one exists.
2. Do not add a raw HTTP bypass for a missing provider binding.
3. Product apps use the Agents App SDK for projects, sessions, turns, session
   items, interactions and catalogs.
4. Provider adapters do not own product authorization, product persistence or
   public Agents resource identities.
5. Skill package identity, installation and content remain owned by
   `sdkwork-skills`; Agents stores stable references only.

## 8. Built-in MCP Toolkit Contract (turn tool-calling loop)

Chat agents execute model-driven tool calls inside the durable Turn loop
(`sdkwork-intelligence-agents-service`, cloudrouter-account-pool executor):

- **Default toolkit.** Every chat agent receives the built-in generations MCP
  tools (`mcp__generations__image.create|retrieve`, `video.create|retrieve`,
  `speech.create`, `music.create|retrieve`) plus the synchronous media family
  by default; no composition configuration is required.
- **Model-visible names equal dispatch ids.** Tools are advertised with their
  namespaced ids (`mcp__<server>__<tool>`, `sound-effect.generate`, ...) so a
  model echo round-trips without a lookup table. Unknown tools fail closed.
- **Loop bounds.** At most 8 completion rounds per turn; tool calls execute
  serially (`parallel_tool_calls: false`); tool results are length-capped
  (`MAX_TOOL_RESULT_CONTENT_CHARS`) before being fed back.
- **Tool namespaces.** `mcp__generations__*` executes on the federated
  generations app API with the caller's dual tokens (tenant scope resolves
  server-side); the media family executes synchronously through the
  cloudrouter gateway; `mcp__<server>__<tool>` (user-registered servers)
  executes over JSON-RPC 2.0 HTTP. Connections resolve per turn from the
  agent's `slotKind: mcp` composition-slot policy (`endpointUrl`, `authType`,
  `secretRef`, `timeoutMs`, `tools`, `allowedTools`/`deniedTools`); secret
  material resolves from `SDKWORK_AGENTS_MCP_SECRET__<REF>` environment
  variables at call time and unresolved credentials fail the call closed.
  The authoritative MCP server registry remains `sdkwork-mcp`; agents slots
  carry the resolved orchestration snapshot only.
- **Approval.** `requires_approval` tools are never executed inline; the loop
  reports `approval_required` back to the model so it asks the user in
  conversation (interaction-object approval is a follow-up surface).
- **Configuration.** `slotKind: tool` (enabled=false) trims default tools;
  `slotKind: mcp` (enabled=true) expands a bound server's tools from its
  policy; `slotKind: skill` contributes an instructions section to the
  assembled system prompt (the request-level `systemPrompt` still wins).
  `GET /app/v3/api/ai/agents/{agentId}/toolkit` exposes the effective
  toolkit. A broken or endpoint-less slot policy is skipped, never failing
  the turn.

## 9. Verification

```powershell
cargo test -p sdkwork-agents-runtime-facade
cargo check -p sdkwork-agents-kernel-bridge
pnpm check:architecture-alignment
```
