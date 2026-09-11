# SDKWork Agents Topology Profiles

The files in this directory are the source-controlled runtime inputs for the
eight supported `deploymentProfile.environment` combinations. The machine
contract is [`../../specs/topology.spec.json`](../../specs/topology.spec.json).

| Profile | Use |
| --- | --- |
| `standalone.development` | Local development and smoke checks |
| `standalone.test` | Standalone integration testing |
| `standalone.staging` | Standalone pre-production validation |
| `standalone.production` | Standalone production deployment |
| `cloud.development` | Cloud integration development |
| `cloud.test` | Cloud integration testing |
| `cloud.staging` | Cloud pre-production validation |
| `cloud.production` | Cloud production deployment |
| `standalone.demo` | Standalone independent demo deployment |
| `cloud.demo` | Cloud independent demo deployment |

`development` uses the Web Framework loopback/private-network policy. Every
other profile projects an explicit, exact CORS allowlist through
`SDKWORK_CORS_ALLOWED_ORIGINS` for the Agents host and embedded route policy.
The `test` and `staging` URLs use reserved `.invalid` hostnames deliberately:
they are fail-closed templates, not deployable browser domains. Operators must
replace every `.invalid` value with the complete real browser, WebView, and H5
origin set before exposing either tier.

The Agents standalone ingress handles Agents routes at its application origin,
but it does not own IAM/Appbase OAuth routes. `SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL`
therefore remains the separate common/IAM gateway address for every profile.
