# SDKWork Agents Source Configuration

`sdkwork.deployment.config.json` is the source-controlled deployment index for
SDKWork Agents. It selects one typed profile from `topology/`; each profile
defines its deployment profile, lifecycle environment, runtime target, public
ingress URLs, platform gateway URL, and shared Web Framework settings.

## Supported Profiles

| Deployment profile | Environments |
| --- | --- |
| `standalone` | `development`, `test`, `staging`, `production` |
| `cloud` | `development`, `test`, `staging`, `production` |

The profile contract is declared by
[`../specs/sdkwork.deployment.config.schema.json`](../specs/sdkwork.deployment.config.schema.json)
and [`../specs/topology.spec.json`](../specs/topology.spec.json). Runtime
materializers select a profile by its `deploymentProfile`, `environment`, and
`runtimeTarget`; process environment and CLI arguments are explicit overrides,
not a second checked-in configuration authority.

## CORS Ownership

The profile that owns an Agents application ingress declares its complete CORS
allowlist. Production-like profiles project the same exact comma-separated
origin set to `SDKWORK_CORS_ALLOWED_ORIGINS`, so the outer gateway and every
dependency Web Framework router embedded by that ingress apply one policy. The
common/IAM gateway is a separately deployed owner and must be materialized with
the same real origin set before it serves `/app/v3/api/oauth/**`.

- `development` relies on the Web Framework loopback/private-network policy.
- `standalone.development` uses the SDKWork database config PostgreSQL profile for
  the Agents managed store and never enables inline auth bypass. It intentionally
  leaves the connection URL/password out of tracked source config; local SDKWork
  PostgreSQL credentials come from ignored `.env.postgres`, bootstrapped from
  the tracked `.env.postgres.example` template. The
  `cloud.development` profile consumes deployed APIs and declares no local database.
- `test` and `staging` use reserved `.invalid` template origins. They are
  intentionally non-routable and must be replaced, before any browser-facing
  rollout, with every real browser, desktop WebView, and H5 origin that calls
  the ingress directly.
- `cloud.production` contains the only currently confirmed Agents browser
  origin: `https://agents.sdkwork.com`. Every standalone non-development
  profile uses a separate `.invalid` operator template until its application
  and common/IAM ingress origins are registered together.

Every replacement origin must be an exact `http` or `https` origin without a
wildcard, path, query string, fragment, or credentials. Apply the same real
origin set to the owning IAM/common gateway when browser clients call
`/app/v3/api/oauth/**`; that gateway is owned outside this repository.

## Local Overrides And Secrets

Committed files under `etc/` contain safe templates only. Local overrides use
ignored `*.local.*` files; secrets are injected by the deployment platform or
mounted from an ignored `etc/secrets/` path. Do not commit tokens, passwords,
private keys, generated runtime state, or machine-specific paths here.

## Materialization And Verification

Deployments materialize these source profiles into the target runtime
configuration location, then inject secrets separately. The Docker entrypoint
requires `SDKWORK_AGENTS_PROFILE_ID` and fills missing values from
`/app/etc/topology/<profile>.env`; it does not provide a second CORS or
environment default. Kubernetes ConfigMaps must likewise be generated from the
selected source profile, with deployment secrets injected separately. Validate
source configuration before packaging or rollout:

```powershell
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
pnpm topology:validate
pnpm gateway:validate:cloud
node --test tests/contract/dev-runtime-access.contract.test.mjs
```

<!-- SDKWORK-DEPLOY-LAYOUT: v1 -->
## Installed Runtime Paths

Authority: `APPLICATION_DEPLOY_LAYOUT_SPEC.md` (`../sdkwork-specs/`).

| Item | Value |
| --- | --- |
| `appId` | `sdkwork-agents` |
| `runtimeCode` | `agents` |
| Config root | `/etc/sdkwork/agents/` |
| Runtime TOML | `/etc/sdkwork/agents/config.toml` |
| Secrets | `/etc/sdkwork/agents/secrets/` |
| Override | `SDKWORK_AGENTS_CONFIG_FILE` |

Source profiles live under `etc/` (`sdkwork.deployment.config.json` index). Deploy manifest: `deployments/deploy.yaml`. Web data-plane source: `deployments/webserver/` (`SDKWORK_WEBSERVER_SPEC.md` layout v2).

```bash
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
node ../sdkwork-specs/tools/check-application-deploy-layout.mjs --root .
node ../sdkwork-specs/tools/check-webserver-toml-standard.mjs --root deployments/webserver
```
<!-- /SDKWORK-DEPLOY-LAYOUT -->


