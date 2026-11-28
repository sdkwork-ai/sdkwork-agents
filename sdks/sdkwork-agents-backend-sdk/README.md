# SDKWork Agent Backend SDK

`sdkwork-agents-backend-sdk` is the backend/admin SDK family for the agent
domain.

| Field | Value |
| --- | --- |
| SDK family | `sdkwork-agents-backend-sdk` |
| API authority | `sdkwork-agents-backend-api` |
| API prefix | `/backend/v3/api` |
| TypeScript package | `@sdkwork/agents-backend-sdk` |
| SDK generator type | `backend` |
| Audience | Backend console, operators, automation, and control-plane integrations |

The authority OpenAPI is `openapi/sdkwork-agents-backend-api.openapi.yaml`.
The derived generator input is `openapi/sdkwork-agents-backend-api.sdkgen.yaml`.

Generated TypeScript transport output belongs under
`sdkwork-agents-backend-sdk-typescript/generated/server-openapi`.

## Generate

Run from repository root:

```powershell
node .\sdks\workspace-agent-sdkgen.mjs --family backend --mode dry-run
node .\sdks\workspace-agent-sdkgen.mjs --family backend --mode apply
```

The generator command uses:

```text
--standard-profile sdkwork-v3
--api-prefix /backend/v3/api
--package-name @sdkwork/agents-backend-sdk
```

## Consume

Backend console, operator tools, and control-plane integrations should consume
`@sdkwork/agents-backend-sdk`. User-facing clients must not consume this family.

## SDKWork Documentation Contract

Domain: intelligence
Capability: agent-backend-sdk
Package type: sdk-family
Status: standardized

### Public API

Public exports are declared in `specs/component.spec.json` under `contracts.publicExports`.

### Required SDK Surface

- `SdkworkBackendClient`

### Configuration

Configuration keys and runtime entrypoints are declared in `specs/component.spec.json`.

### SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`, including runtime configuration rules where applicable.

### Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

### Extension Points

Extension points are limited to declared public exports, runtime entrypoints, SDK clients, events, and config keys.

### Verification

- `node sdks/materialize-agent-v3-openapi-boundaries.mjs`
- `node sdks/sdkwork-agents-backend-sdk/bin/verify-sdk.mjs`
- `node sdks/test/verify-agent-sdk-ownership-boundaries.test.mjs`
- `node scripts/check-agent-sdk-workspace.mjs`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.

## Agent category taxonomy

`agentCategories` exposes list, create/update (upsert by `categoryCode`), soft
delete and sibling move. Categories form a tenant-scoped tree of at most five
levels; `parentId` is nullable, `iconRef` stores a Drive logo reference, and
`tags` / `attributes` carry free-form taxonomy metadata. Delete is refused while
the category owns a subtree or a live agent references its code.

