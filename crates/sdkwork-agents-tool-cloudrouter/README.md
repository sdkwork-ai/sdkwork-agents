# sdkwork-agents-tool-cloudrouter

Shared sdkwork-cloudrouter open-api client adapter for the SDKWork Agents media
tool family.

Every media tool category crate (audio/video/music/sound-effect/image) calls
the cloudrouter gateway through this adapter instead of constructing raw HTTP
calls. The adapter owns:

- gateway base URL resolution (`APP_SDK_INTEGRATION_SPEC.md` §5.2), shared with
  the chat turn executor:
  1. `SDKWORK_AGENTS_CLOUDROUTER_BASE_URL` — explicit override (split deployment);
  2. the **authored** `SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL`
     (an absolute domain in cloud, e.g. `https://router.sdkwork.com`);
  3. the split-deployment default `http://127.0.0.1:3900`.

  In the embedded (standalone) topology the resolver returns **no** base URL:
  this surface is composed into the gateway process, so it is consumed through
  the in-process port. Deriving `http://127.0.0.1:{port}` from the process' own
  `..._APPLICATION_PUBLIC_INGRESS_BIND` is prohibited
  (`APPLICATION_GATEWAY_SPEC.md` §2.3) — that self-loop is what made every
  embedded SDK call dial a port the process does not own;
- auth-token injection (`Authorization: Bearer <auth token>`) for cloudrouter
  account-pool routing — the caller's login token selects the tenant account
  group upstream, no API key required. User-scoped calls propagate the caller's
  dual tokens (`auth_token` + `access_token`) so the full caller identity
  reaches the gateway (`APP_SDK_INTEGRATION_SPEC.md` §5.2.1); the adapter never
  mints, caches, or widens a credential;
- a dedicated blocking Tokio runtime so synchronous kernel
  `ToolProvider::invoke_tool` calls can drive async SDK calls;
- error mapping from cloudrouter SDK errors to the media tool error taxonomy
  with actionable hints for common gateway failures.
