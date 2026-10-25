# Operator Guide

Deployment and runtime validation for SDKWork Agents. The public gateway and
Task Worker are separate processes built from the same release source. They
scale, probe, expose metrics, drain and roll back independently; PostgreSQL is
the sole scheduling correctness authority shared by both.

## Pre-flight

From the application root:

```powershell
pnpm verify
pnpm topology:validate
pnpm db:validate
```

These gates cover API and SDK standards, operation semantics, route collision checks, pagination, SDK import boundaries, composition boundaries, deploy manifest (`deployments/deploy.yaml`), topology spec, and database framework alignment.

The gateway serves application APIs. The Worker materializes due Tasks, claims
Runs, executes fenced Attempts, recovers expired leases and reconciles unknown
outcomes. Do not run scheduler loops inside gateway replicas or use Redis/Kafka
as the scheduling authority.

**Staging/production cutover:** [runbooks/pre-launch-verification.md](../../runbooks/pre-launch-verification.md)

**Ongoing operations:** [runbooks/monitoring.md](../../runbooks/monitoring.md), [runbooks/smoke-test.md](../../runbooks/smoke-test.md), [runbooks/incident-rollback.md](../../runbooks/incident-rollback.md)

## Deployment profiles

Authoritative manifest: [deployments/deploy.yaml](../../../deployments/deploy.yaml)

| Profile | Use case |
| --- | --- |
| `cloud.production` | Cloud web + API domains through application public ingress |
| `standalone.production` | Single application public ingress binary package |
| `standalone.development` | Local source-tree dev |

Topology and env keys: [specs/topology.spec.json](../../../specs/topology.spec.json), `etc/topology/*.env`.

In Kubernetes, deploy `sdkwork-api-agents-standalone-gateway` and
`sdkwork-intelligence-agents-worker` separately. Both use Pod UID as
`SDKWORK_NODE_INSTANCE_ID` for the PostgreSQL-backed Snowflake node lease.
The Worker's `SDKWORK_AGENTS_TASK_WORKER_ID` is a distinct scheduling identity.
Treat either node-lease acquisition failure as a readiness failure. Use each
process's own `/healthz`, `/readyz` and `/metrics`, respect the Worker drain
timeout, and rely on lease expiry plus fencing during restart or rollback.

## Database

```powershell
pnpm db:status
pnpm db:migrate
pnpm db:drift:check
```

Production migrations run as a release phase before gateway or Worker rollout.
Never edit Task Run leases or fencing tokens manually. Pause an affected Task
through its governed API for scoped containment; scale the Worker only for
scheduler-wide containment.

## Packaging (CI)

GitHub workflow: `.github/workflows/package.yml` → reusable `sdkwork-package.yml` driven by `agent.workflow.json`.

Manual dispatch inputs include sibling dependency refs (`SDKWORK_KERNEL_REF`, `SDKWORK_KNOWLEDGEBASE_REF`, etc.).

## Non-GA Platform Scope

Rate limiting, CORS, and dashboard wiring are owned by `sdkwork-web-framework` and ops. `sdkwork-agents` must consume those platform capabilities instead of reimplementing them locally. See [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md) section 10.

## Canon

- [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md)
- [runbooks/pre-launch-verification.md](../../runbooks/pre-launch-verification.md)
- [runbooks/monitoring.md](../../runbooks/monitoring.md)
- [runbooks/smoke-test.md](../../runbooks/smoke-test.md)
- [runbooks/incident-rollback.md](../../runbooks/incident-rollback.md)
