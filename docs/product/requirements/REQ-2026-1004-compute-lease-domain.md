# REQ-2026-1004 Compute Lease And On-Demand Agent Runtime Domain

- Date: 2026-10-04
- Owner: agents-platform (workspace shell)
- Status: draft — owner-directed (workspace shell instruction: greenfield holistic
  design for a compute-rental and on-demand agent runtime environment, polished
  to commercial delivery)
- Follows: REQ-2026-0730 (hybrid execution orchestration), REQ-2026-0731 (task
  scheduling)

## 1. Problem

Today compute-related state is scattered across three module databases:

| Module | Tables | Domain |
| --- | --- | --- |
| sdkwork-sandbox | `sandbox_instance`, `sandbox_runtime_binding`, `sandbox_session_lease`, ... | User-rentable sandbox instances (resources, images, assurance) |
| sdkwork-agents | `ai_agent_execution_host`, `ai_agent_session_execution_placement` | Agent dispatch hosts and session placement bindings |
| sdkwork-webserver | `webserver_cluster_host`, `webserver_cluster_instance` | Web serving machines and process instances |

All three model the same underlying concepts (a compute node, a rented
compute unit, an allocation of work to a node) with three divergent
vocabularies, three lifecycle state machines, and three ownership
boundaries. Nothing is launched, so there is no migration constraint —
the model can be designed once, cohesively.

## 2. Goal

One **compute lease and on-demand allocation domain** (`sdkwork-compute`)
that owns:

- the node inventory (machines and dispatch targets: docker daemons,
  micro-VM pools, bare metal, cloud pools);
- rentable compute offerings and leased instances (a sandbox, an agent
  runtime, a web serving process are all *purposes* of one instance model);
- the allocation of work to nodes with lease, fencing and reconciliation;
- tenancy and capacity accounting for commercial metering.

Consumers (agents, sandbox product, webserver) keep only *references*
(allocation/instance ids) inside their own aggregates and integrate through
the compute App API and SDK. No consumer duplicates node or instance state.

## 3. Requirements

- R1 Node inventory with capacity, region, dispatch endpoint (opaque,
  credential-free), isolation-capability set, lifecycle (register/active/
  drain/disable) and liveness telemetry.
- R2 Rentable offerings (resource shapes + isolation assurance) and leased
  instances bound to nodes with rental expiry and renewal.
- R3 On-demand allocation: a consumer requests capacity with constraints
  (kind, region, assurance, resources); the domain reserves a node/instance
  and returns a durable allocation with lease and fencing evidence.
- R4 Reconciliation: expired leases and lost nodes are reconciled by the
  compute worker; consumers are notified through allocation state.
- R5 Multi-tenancy: platform-shared pool (`tenant_id = 0`) and tenant-pinned
  nodes; every read/write is tenant-scoped.
- R6 Commercial metering hooks: allocation and instance records carry the
  facts metering needs (owner, offering, start/end, duration).
- R7 Agents sessions resolve execution placement through compute allocations;
  the agent-side host table is retired in favor of compute references.

## 4. Non-Goals

- Owning sandbox product semantics (workspace attachment, IM integration) —
  the sandbox module keeps product metadata and references compute instances.
- Replacing the kernel runtime SPI — the kernel keeps executing inside
  instances; compute owns the environment they run in.
- Billing invoicing — metering hooks only.

## 5. Acceptance

- One node table, one instance table, one allocation table serve sandbox,
  agents and webserver consumers through versioned APIs and SDKs.
- Agents no longer stores host inventory; session placement references a
  compute allocation id.
- Full sdkwork-specs verification battery green in every touched module.
