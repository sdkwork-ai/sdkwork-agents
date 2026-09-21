//! Turn transport selection for the Cloud Router chat path
//! (`APPLICATION_GATEWAY_SPEC.md` §2.3, `APP_SDK_INTEGRATION_SPEC.md` §5.2).
//!
//! A chat turn needs one thing from its dependency: a way to reach the
//! CloudRouter open-api surface. Which way depends **only** on the deployment
//! profile, never on a bind address or a reachability probe:
//!
//! - **split** (`cloud`, or a separately deployed application) — CloudRouter
//!   runs in another process, so the turn dials it over HTTP. The base URL comes
//!   from the shared resolver: explicit override, then the authored topology
//!   URL, then the compile-time split default.
//! - **embedded** (`standalone`, the composition root that mounts the
//!   CloudRouter assembly) — the surface is linked into *this* process, so the
//!   turn dispatches into the assembly router directly. There is no HTTP base
//!   URL, and deriving one from this process' own listener is the prohibited
//!   self-loop that produced the production `Bad gateway` incident.
//!
//! Before this module the turn path had only the HTTP arm, so an embedded
//! profile could not produce a base URL and every turn failed with
//! `provider_error: service cloudrouter has no HTTP base URL` (result code
//! `50301`). The missing piece was never configuration — it was the second
//! transport arm, implemented here.

use std::sync::Arc;

use cloudrouter_open_sdk::SdkworkError;
use sdkwork_utils_rust::service_base_url::{mode_of_deployment_profile, ServiceMode};

use crate::chat_stream::{
    stream_chat_completion_with_tools_blocking, stream_llm_completion_blocking,
    CloudRouterChatStreamResult, CloudRouterStreamDelta,
};
use crate::client::ENV_CLOUDROUTER_BASE_URL;
use crate::credentials::{
    embedded_open_api_key, HopCredentials, ACCESS_TOKEN_HEADER, API_KEY_HEADER,
};
use crate::wire_protocol::WireProtocol;

/// The two credential headers the CloudRouter open-api contract declares
/// (`API_SPEC.md` §10).
///
/// CloudRouter's `/v1/*` operations are `api-key-or-dual-token`: `X-API-Key`
/// alone, or the complete `Authorization` + `Access-Token` pair. The exact
/// spelling matters because CloudRouter matches these names literally
/// (`sdkwork_cloudrouter_http::auth` constants `x-api-key` / `Access-Token`),
/// and a mismatch would silently drop the credential instead of failing.
/// The `api-key` spelling is owned by [`crate::credentials`].

/// Receives one fully-built HTTP request and returns its response.
///
/// A boxed closure rather than `axum::Router` keeps this crate free of an axum
/// dependency while still accepting the composition root's
/// `router.oneshot(request)`. Tests install a stub surface through the same
/// seam the production composition root uses.
pub type EmbeddedSurfaceDispatcher = Arc<
    dyn Fn(
            axum::extract::Request,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = SurfaceOutcome> + Send>>
        + Send
        + Sync,
>;

/// Outcome of one in-process dispatch. `Infallible` on the error side, because
/// `tower::ServiceExt::oneshot` on an `axum::Router` never fails.
pub type SurfaceOutcome = Result<axum::response::Response, std::convert::Infallible>;

fn dispatcher_slot() -> &'static std::sync::OnceLock<EmbeddedSurfaceDispatcher> {
    static SLOT: std::sync::OnceLock<EmbeddedSurfaceDispatcher> = std::sync::OnceLock::new();
    &SLOT
}

/// Installs the composition root's in-process CloudRouter surface dispatcher.
///
/// Called once by the process that mounts the CloudRouter assembly
/// (`APPLICATION_GATEWAY_SPEC.md` §2.3). A process has one CloudRouter surface
/// and one composition root, so the first installation wins.
///
/// A second installation is **not** silently dropped: it is reported as an
/// error-level event and recorded, because the only ways to reach it are a
/// duplicated wiring path or a reordered composition root — both of which would
/// otherwise mean the process quietly runs on the *first* surface while the
/// caller believes it installed the second. Returns `true` when this call won
/// the slot, `false` when an earlier installation was kept.
///
/// The return value is `#[must_use]`: an installer that ignores it would be
/// exactly the silent case this signature exists to prevent.
#[must_use = "a `false` result means an earlier installation was kept and this \
              surface is not serving turns — check the composition order"]
pub fn install_embedded_cloudrouter_surface(dispatcher: EmbeddedSurfaceDispatcher) -> bool {
    match dispatcher_slot().set(dispatcher) {
        Ok(()) => true,
        Err(_) => {
            tracing::error!(
                target: "sdkwork_agents_tool_cloudrouter",
                "the embedded cloudrouter surface was already installed; keeping the first \
                 installation and ignoring this one. A process has exactly one cloudrouter \
                 surface and one composition root (APPLICATION_GATEWAY_SPEC §2.3) — this \
                 second install means the wiring path is duplicated or the composition \
                 order changed, and turns will keep using the FIRST surface."
            );
            false
        }
    }
}

/// The process-installed dispatcher, when the composition root wired one.
pub fn embedded_cloudrouter_surface() -> Option<EmbeddedSurfaceDispatcher> {
    dispatcher_slot().get().cloned()
}

/// The deployment profile the process runs under, read from the canonical
/// profile variable and then the application-scoped alias.
///
/// Mirrors `client::deployment_profile` deliberately: one process has exactly
/// one profile, and both CloudRouter consumers must read the same inputs so
/// they can never disagree about their own mode.
fn deployment_profile() -> Option<String> {
    [
        "SDKWORK_DEPLOYMENT_PROFILE",
        "SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE",
    ]
    .into_iter()
    .find_map(|key| {
        std::env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

/// The deployment mode this process resolves turn transport under.
pub fn turn_service_mode() -> ServiceMode {
    mode_of_deployment_profile(deployment_profile().as_deref())
}

/// One chat turn invocation, independent of transport.
pub struct CloudRouterTurnRequest<'a> {
    /// Wire protocol selected by the caller's session configuration.
    pub protocol: WireProtocol,
    /// The caller's bearer auth token (account-pool identity).
    pub auth_token: &'a str,
    /// The caller's access token, when the inbound request carried one.
    pub access_token: Option<&'a str>,
    /// The completion request body.
    pub request: cloudrouter_open_sdk::models::OpenAiChatCompletionRequest,
    /// Serialized OpenAI `tools` array; empty disables the function-calling loop.
    pub tools: Vec<serde_json::Value>,
}

/// Transport selected for one process, per the deployment profile.
pub enum CloudRouterTurnTransport {
    /// CloudRouter is composed into this process: dispatch into its surface.
    InProcess {
        dispatcher: EmbeddedSurfaceDispatcher,
        api_key: Option<String>,
    },
    /// CloudRouter runs in another process: dial the resolved HTTP base URL.
    Http { base_url: String },
}

impl CloudRouterTurnTransport {
    /// Selects the transport for this process.
    ///
    /// Resolution is driven by the deployment profile alone
    /// (`APP_SDK_INTEGRATION_SPEC.md` §5.2). In the embedded profile the
    /// in-process surface is required: an HTTP base URL is deliberately *not*
    /// consulted, because a standalone profile authors its public URL as a
    /// loopback bind and adopting it while the surface is composed in-process
    /// is the self-loop the specs prohibit.
    pub fn resolve() -> Result<Self, CloudRouterTransportError> {
        // An explicit external override is the sanctioned escape hatch for a
        // split deployment that hosts an otherwise-embedded surface
        // separately; it wins in either mode, matching the shared resolver.
        if let Some(base_url) = explicit_override() {
            return Ok(Self::Http { base_url });
        }

        if turn_service_mode() == ServiceMode::Embedded {
            let Some(dispatcher) = embedded_cloudrouter_surface() else {
                return Err(CloudRouterTransportError::EmbeddedSurfaceNotWired);
            };
            return Ok(Self::InProcess {
                dispatcher,
                api_key: embedded_open_api_key(),
            });
        }

        crate::client::resolve_cloudrouter_base_url()
            .into_required()
            .map(|base_url| Self::Http { base_url })
            .map_err(|error| CloudRouterTransportError::NoBaseUrl(error.to_string()))
    }

    /// Streams one chat completion over the selected transport.
    ///
    /// Both arms preserve the open-api contract: the same protocol endpoints,
    /// the same dual-token headers, the same SSE normalization and tool-call
    /// reconstruction. Only the hop differs.
    pub fn stream_chat_completion(
        &self,
        input: CloudRouterTurnRequest<'_>,
        on_delta: &mut dyn FnMut(CloudRouterStreamDelta),
    ) -> Result<CloudRouterChatStreamResult, SdkworkError> {
        match self {
            Self::Http { base_url } => stream_over_http(base_url, input, on_delta),
            Self::InProcess {
                dispatcher,
                api_key,
            } => stream_over_in_process(dispatcher, api_key.as_deref(), input, on_delta),
        }
    }
}

/// Reads the explicit override, normalized (trimmed; blank behaves as absent).
fn explicit_override() -> Option<String> {
    std::env::var(ENV_CLOUDROUTER_BASE_URL)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn stream_over_http(
    base_url: &str,
    input: CloudRouterTurnRequest<'_>,
    on_delta: &mut dyn FnMut(CloudRouterStreamDelta),
) -> Result<CloudRouterChatStreamResult, SdkworkError> {
    if input.tools.is_empty() {
        stream_llm_completion_blocking(
            input.protocol,
            base_url,
            input.auth_token,
            input.access_token,
            input.request,
            on_delta,
        )
    } else {
        stream_chat_completion_with_tools_blocking(
            base_url,
            input.auth_token,
            input.access_token,
            input.request,
            input.tools,
            on_delta,
        )
    }
}

/// Streams one completion by dispatching into the in-process CloudRouter
/// surface instead of an HTTP listener.
///
/// The open-api contract is preserved exactly: the same protocol endpoint
/// (`/v1/chat/completions`, `/anthropic/v1/messages`, ...), the same
/// `Authorization` / `Access-Token` identity headers, and the same SSE body
/// that CloudRouter's edge-runtime pipeline emits. The body is then parsed by
/// the same accumulator the HTTP arm uses, so protocol framing, delta delivery,
/// and tool-call reconstruction have exactly one implementation.
///
/// The turn worker calling this is a synchronous blocking worker, so the
/// dispatch future is driven to completion on a dedicated thread with its own
/// single-threaded runtime; the calling thread keeps its synchronous delta
/// callback and never nests a runtime.
fn stream_over_in_process(
    dispatcher: &EmbeddedSurfaceDispatcher,
    api_key: Option<&str>,
    input: CloudRouterTurnRequest<'_>,
    on_delta: &mut dyn FnMut(CloudRouterStreamDelta),
) -> Result<CloudRouterChatStreamResult, SdkworkError> {
    let (protocol, endpoint, body) = build_in_process_request(&input);
    let request = build_surface_request(&endpoint, &body, &input, api_key)?;
    let dispatcher = Arc::clone(dispatcher);

    let (status, response_body) = dispatch_blocking(dispatcher, request)?;

    if !(200..300).contains(&status) {
        return Err(SdkworkError::HttpStatus {
            status,
            body: String::from_utf8_lossy(&response_body).to_string(),
        });
    }

    crate::chat_stream::accumulate_stream_body(protocol, &response_body, on_delta)
}

/// Builds the protocol endpoint and JSON body for one in-process dispatch.
///
/// The body comes from the **same** assemblers the HTTP arm uses
/// (`chat_stream::build_chat_tools_body` for tool turns,
/// `wire_protocol::build_protocol_request_body` otherwise), so the two
/// transports cannot drift: `both_arms_build_the_same_*` tests pin the
/// equivalence.
///
/// Deliberately **no** post-hoc `body["stream"] = true` here. The assemblers
/// already receive `stream = true` and each expresses it the way its protocol
/// requires: chat/anthropic/responses carry a `stream` boolean, while Google
/// signals streaming through the URL (`:streamGenerateContent?alt=sse`) and has
/// no `stream` field at all. Forcing the key on would make the in-process body
/// differ from the HTTP body for Google — a transport-dependent difference for
/// no benefit.
fn build_in_process_request(
    input: &CloudRouterTurnRequest<'_>,
) -> (WireProtocol, String, serde_json::Value) {
    let protocol = if input.tools.is_empty() {
        input.protocol
    } else {
        // Tool-call fidelity requires the OpenAI chat shape
        // (`stream_chat_completion_with_tools_blocking` makes the same choice).
        WireProtocol::ChatCompletions
    };
    let endpoint = protocol.streaming_endpoint(&input.request.model);
    let body = if input.tools.is_empty() {
        crate::wire_protocol::build_protocol_request_body(protocol, &input.request, true)
    } else {
        crate::chat_stream::build_chat_tools_body(&input.request, &input.tools)
    };
    (protocol, endpoint, body)
}

/// Status carried by an in-process **local** failure (body serialization,
/// request construction, dispatch runtime, response read, dispatch panic).
///
/// Zero is not a valid HTTP status, which is the point: it cannot be confused
/// with anything the CloudRouter surface returned. Downstream error mapping
/// keys on real statuses (402/404/401/5xx) and treats `0` as unclassified,
/// so a local failure never masquerades as an upstream rejection.
const EMBEDDED_LOCAL_FAILURE_STATUS: u16 = 0;

/// Builds the `axum` request for the in-process hop, carrying exactly one of
/// the two credential alternatives the open-api contract declares.
///
/// The **selection** comes from [`HopCredentials::select`], the single rule
/// both transports share (`crate::credentials`); this function only renders
/// the chosen source into headers. Keeping the rule shared is what stops the
/// two arms from authenticating as different principals for the same turn.
///
/// CloudRouter rejects any request carrying more than one non-empty credential
/// header (`ApiKeyIdentity::parse_credential` →
/// `ApiKeyCredentialSourcesAmbiguous`), mapped to `400 invalid_request`, so the
/// branches below are mutually exclusive: sending both would make every
/// embedded turn fail with a 400 that looks unrelated to authentication.
fn build_surface_request(
    endpoint: &str,
    body: &serde_json::Value,
    input: &CloudRouterTurnRequest<'_>,
    api_key: Option<&str>,
) -> Result<axum::extract::Request, SdkworkError> {
    let mut builder = axum::extract::Request::builder()
        .method(axum::http::Method::POST)
        .uri(endpoint)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header(axum::http::header::ACCEPT, "text/event-stream");
    match HopCredentials::select(api_key, input.auth_token, input.access_token) {
        // Alternative 1: `X-API-Key` only. No `Authorization`, no
        // `Access-Token` — the API-key branch owns its own server-side lookup.
        HopCredentials::ApiKey(api_key) => {
            builder = builder.header(API_KEY_HEADER, api_key);
        }
        // Alternative 2: the complete dual-token pair. Carried verbatim from
        // the caller's authenticated session, same as the HTTP hop, so the
        // surface projects the identical principal and access context.
        HopCredentials::CallerPair {
            auth_token,
            access_token,
        } => {
            builder = builder.header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {auth_token}"),
            );
            if let Some(access_token) = access_token {
                builder = builder.header(ACCESS_TOKEN_HEADER, access_token);
            }
        }
    }
    let payload = serde_json::to_vec(body).map_err(|error| SdkworkError::HttpStatus {
        status: EMBEDDED_LOCAL_FAILURE_STATUS,
        body: format!("embedded cloudrouter request body serialization failed: {error}"),
    })?;
    builder
        .body(axum::body::Body::from(payload))
        .map_err(|error| SdkworkError::HttpStatus {
            status: EMBEDDED_LOCAL_FAILURE_STATUS,
            body: format!("embedded cloudrouter request build failed: {error}"),
        })
}

/// Test-only hook that builds the in-process hop's `axum` request from a
/// caller-supplied [`CloudRouterTurnRequest`], using the same
/// [`build_surface_request`] the real transport uses.
///
/// It exists so a parity test can compare the **headers each arm renders**
/// instead of only comparing the shared selection rule. `api_key` is passed
/// explicitly, matching the production call site, where the transport carries
/// the resolved key rather than re-reading the environment mid-request.
#[cfg(test)]
pub(crate) fn build_surface_request_for_test(
    input: &CloudRouterTurnRequest<'_>,
) -> Result<axum::extract::Request, SdkworkError> {
    let (protocol, endpoint, body) = build_in_process_request(input);
    debug_assert_eq!(
        protocol, input.protocol,
        "endpoint must match the caller's wire"
    );
    build_surface_request(&endpoint, &body, input, embedded_open_api_key().as_deref())
}

/// Polls the in-process dispatch future to completion on a dedicated thread.
///
/// The turn worker calling this is a synchronous blocking worker, so the
/// dispatcher future runs on a dedicated Tokio runtime owned here. The
/// CloudRouter surface is `Send` and the future is `Send`, so this is a
/// straight `block_on` rather than a self-join.
fn dispatch_blocking(
    dispatcher: EmbeddedSurfaceDispatcher,
    request: axum::extract::Request,
) -> Result<(u16, Vec<u8>), SdkworkError> {
    let handle = std::thread::Builder::new()
        .name("cloudrouter-embedded-dispatch".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| SdkworkError::HttpStatus {
                    status: EMBEDDED_LOCAL_FAILURE_STATUS,
                    body: format!("embedded cloudrouter dispatch runtime failed: {error}"),
                })?;
            runtime.block_on(async move {
                // `oneshot` on an `axum::Router` is infallible, so the only
                // failure left is reading the response body.
                let response = match (dispatcher)(request).await {
                    Ok(response) => response,
                    Err(error) => match error {},
                };
                let status = response.status().as_u16();
                let body = axum::body::to_bytes(response.into_body(), MAX_EMBEDDED_BODY_BYTES)
                    .await
                    .map_err(|error| SdkworkError::HttpStatus {
                        status,
                        body: format!("embedded cloudrouter response read failed: {error}"),
                    })?;
                Ok((status, body.to_vec()))
            })
        })
        .map_err(|error| SdkworkError::HttpStatus {
            status: EMBEDDED_LOCAL_FAILURE_STATUS,
            body: format!("embedded cloudrouter dispatch thread failed: {error}"),
        })?;

    // A panicking dispatch must not unwind the turn worker: it is reported as a
    // transport failure instead.
    handle.join().map_err(|_| SdkworkError::HttpStatus {
        status: EMBEDDED_LOCAL_FAILURE_STATUS,
        body: "embedded cloudrouter dispatch thread panicked".to_string(),
    })?
}

/// Bounded response budget for one in-process dispatch, mirroring the generated
/// SDK's own cap so a runaway surface cannot exhaust the turn worker.
const MAX_EMBEDDED_BODY_BYTES: usize = 64 * 1024 * 1024;

/// Why a turn could not obtain a transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudRouterTransportError {
    /// The embedded profile is active but the composition root never installed
    /// the CloudRouter surface dispatcher.
    ///
    /// Failing closed is deliberate: falling back to HTTP here would dial this
    /// process' own listener (`APPLICATION_GATEWAY_SPEC.md` §2.3).
    EmbeddedSurfaceNotWired,
    /// The split profile is active but no base URL could be resolved.
    NoBaseUrl(String),
}

impl std::fmt::Display for CloudRouterTransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmbeddedSurfaceNotWired => write!(
                formatter,
                "cloudrouter is composed into this process but its in-process surface was never \
                 installed by the composition root; the composition root must call \
                 install_embedded_cloudrouter_surface with the CloudRouter assembly router \
                 (APPLICATION_GATEWAY_SPEC §2.3)"
            ),
            Self::NoBaseUrl(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for CloudRouterTransportError {}

/// Reads the CloudRouter open-api key used for in-process billing identity.
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes env-mutating tests: `std::env` is process-global, and the
    /// transport resolver reads the profile from it.
    fn env_guard() -> &'static Mutex<()> {
        static GUARD: Mutex<()> = Mutex::new(());
        &GUARD
    }

    /// Snapshot/restore the resolution inputs so the process environment is
    /// left exactly as the test found it.
    struct TransportEnv {
        profile: Option<String>,
        app_profile: Option<String>,
        base_url: Option<String>,
        public_http_url: Option<String>,
    }

    impl TransportEnv {
        fn capture(keys: &[&str]) -> Self {
            let read = |key: &str| std::env::var(key).ok();
            let guard = Self {
                profile: read("SDKWORK_DEPLOYMENT_PROFILE"),
                app_profile: read("SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE"),
                base_url: read(ENV_CLOUDROUTER_BASE_URL),
                public_http_url: read(crate::client::ENV_CLOUDROUTER_PUBLIC_HTTP_URL),
            };
            for key in keys {
                std::env::remove_var(key);
            }
            guard
        }
    }

    impl Drop for TransportEnv {
        fn drop(&mut self) {
            for (key, value) in [
                ("SDKWORK_DEPLOYMENT_PROFILE", &self.profile),
                ("SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE", &self.app_profile),
                (ENV_CLOUDROUTER_BASE_URL, &self.base_url),
                (
                    crate::client::ENV_CLOUDROUTER_PUBLIC_HTTP_URL,
                    &self.public_http_url,
                ),
            ] {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    const RESOLUTION_KEYS: &[&str] = &[
        "SDKWORK_DEPLOYMENT_PROFILE",
        "SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE",
        ENV_CLOUDROUTER_BASE_URL,
        "SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL",
    ];

    #[test]
    fn only_the_standalone_profile_selects_the_in_process_arm() {
        // `mode_of_deployment_profile` is the single authority; this asserts the
        // turn transport consumes it rather than inventing a second rule.
        assert_eq!(
            mode_of_deployment_profile(Some("standalone")),
            ServiceMode::Embedded
        );
        assert_eq!(
            mode_of_deployment_profile(Some("cloud")),
            ServiceMode::Split
        );
        assert_eq!(mode_of_deployment_profile(None), ServiceMode::Split);
    }

    #[test]
    fn embedded_transport_error_names_the_composition_root_obligation() {
        let message = CloudRouterTransportError::EmbeddedSurfaceNotWired.to_string();
        assert!(message.contains("in-process"));
        assert!(message.contains("install_embedded_cloudrouter_surface"));
        assert!(message.contains("APPLICATION_GATEWAY_SPEC"));
    }

    #[test]
    fn blank_open_api_key_is_absent() {
        // Guards the identity input: a blank variable must not become a
        // `Bearer ` header with an empty credential.
        std::env::set_var(crate::credentials::ENV_CLOUDROUTER_OPEN_API_KEY, "   ");
        assert_eq!(embedded_open_api_key(), None);
        std::env::remove_var(crate::credentials::ENV_CLOUDROUTER_OPEN_API_KEY);
    }

    #[test]
    fn embedded_profile_without_a_wired_surface_fails_closed() {
        // The defect this module exists to prevent: an embedded profile that
        // cannot resolve a base URL. It must NOT fall back to loopback — it must
        // report that the composition root never wired the in-process port.
        let _guard = env_guard().lock().expect("env test lock");
        let _env = TransportEnv::capture(RESOLUTION_KEYS);
        std::env::set_var("SDKWORK_DEPLOYMENT_PROFILE", "standalone");
        // Even with an authored loopback present, the embedded arm must not
        // adopt it (that is the self-loop).
        std::env::set_var(
            "SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL",
            "http://127.0.0.1:3900",
        );

        match CloudRouterTurnTransport::resolve() {
            Err(CloudRouterTransportError::EmbeddedSurfaceNotWired) => {}
            other => panic!(
                "expected EmbeddedSurfaceNotWired, got {other:?}",
                other = other.map(|_| "a resolved transport")
            ),
        }
    }

    #[test]
    fn explicit_override_wins_in_both_modes() {
        // The sanctioned escape hatch for a split deployment that hosts an
        // otherwise-embedded surface separately.
        let _guard = env_guard().lock().expect("env test lock");
        let _env = TransportEnv::capture(RESOLUTION_KEYS);
        std::env::set_var("SDKWORK_DEPLOYMENT_PROFILE", "standalone");
        std::env::set_var(ENV_CLOUDROUTER_BASE_URL, "https://router.sdkwork.com");

        match CloudRouterTurnTransport::resolve() {
            Ok(CloudRouterTurnTransport::Http { base_url }) => {
                assert_eq!(base_url, "https://router.sdkwork.com");
            }
            _ => panic!("expected the explicit override to select the HTTP arm"),
        }
    }

    #[test]
    fn split_profile_resolves_the_http_arm() {
        let _guard = env_guard().lock().expect("env test lock");
        let _env = TransportEnv::capture(RESOLUTION_KEYS);
        std::env::set_var("SDKWORK_DEPLOYMENT_PROFILE", "cloud");
        std::env::set_var(
            "SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL",
            "https://router.sdkwork.com",
        );

        match CloudRouterTurnTransport::resolve() {
            Ok(CloudRouterTurnTransport::Http { base_url }) => {
                assert_eq!(base_url, "https://router.sdkwork.com");
            }
            _ => panic!("expected the split profile to select the HTTP arm"),
        }
    }

    #[test]
    fn tool_free_requests_keep_the_selected_protocol_endpoint() {
        let mut request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
        request.model = "gemini-2.5".to_string();
        let input = CloudRouterTurnRequest {
            protocol: WireProtocol::GoogleContent,
            auth_token: "auth",
            access_token: None,
            request,
            tools: Vec::new(),
        };
        let (protocol, endpoint, _body) = build_in_process_request(&input);
        assert_eq!(protocol, WireProtocol::GoogleContent);
        assert_eq!(
            endpoint,
            "/google/v1beta/models/gemini-2.5:streamGenerateContent?alt=sse"
        );
    }

    fn turn_request<'a>(
        request: &'a cloudrouter_open_sdk::models::OpenAiChatCompletionRequest,
    ) -> CloudRouterTurnRequest<'a> {
        CloudRouterTurnRequest {
            protocol: WireProtocol::ChatCompletions,
            auth_token: "login-auth-token",
            access_token: Some("login-access-token"),
            request: request.clone(),
            tools: Vec::new(),
        }
    }

    #[test]
    fn api_key_branch_omits_both_token_headers() {
        // API_SPEC §10: `api-key-or-dual-token` allows `X-API-Key` ALONE. The
        // dual-token headers must be absent, not merely secondary — CloudRouter
        // rejects any request carrying two credential header sources
        // (`ApiKeyCredentialSourcesAmbiguous` → HTTP 400 `invalid_request`).
        let request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
        let input = turn_request(&request);

        let built = build_surface_request(
            "/v1/chat/completions",
            &serde_json::json!({"model": "gpt-4o-mini"}),
            &input,
            Some("sk-embedded-key"),
        )
        .expect("request builds");

        assert_eq!(
            Some("sk-embedded-key"),
            built
                .headers()
                .get(API_KEY_HEADER)
                .map(|v| v.to_str().unwrap())
        );
        assert!(
            !built
                .headers()
                .contains_key(axum::http::header::AUTHORIZATION),
            "the API-key branch must not also carry Authorization: {}",
            "that is the forbidden 'API key mixed with token' combination"
        );
        assert!(
            !built.headers().contains_key(ACCESS_TOKEN_HEADER),
            "the API-key branch must not also carry Access-Token"
        );
    }

    #[test]
    fn dual_token_branch_carries_both_tokens_and_no_api_key() {
        // The other sanctioned alternative: the complete `Authorization` +
        // `Access-Token` pair, with no API key header alongside it.
        let request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
        let input = turn_request(&request);

        let built = build_surface_request(
            "/v1/chat/completions",
            &serde_json::json!({"model": "gpt-4o-mini"}),
            &input,
            None,
        )
        .expect("request builds");

        assert_eq!(
            Some("Bearer login-auth-token"),
            built
                .headers()
                .get(axum::http::header::AUTHORIZATION)
                .map(|v| v.to_str().unwrap())
        );
        assert_eq!(
            Some("login-access-token"),
            built
                .headers()
                .get(ACCESS_TOKEN_HEADER)
                .map(|v| v.to_str().unwrap())
        );
        assert!(
            !built.headers().contains_key(API_KEY_HEADER),
            "the dual-token branch must not also carry an API key"
        );
    }

    #[test]
    fn no_request_ever_carries_two_credential_sources() {
        // The invariant that keeps the in-process hop accepted by CloudRouter.
        // Asserted over both branches and both access-token shapes so a future
        // edit that reintroduces header mixing fails here instead of as an
        // unexplained 400 in production.
        let request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
        let body = serde_json::json!({"model": "gpt-4o-mini"});
        let with_access = turn_request(&request);
        let without_access = CloudRouterTurnRequest {
            access_token: None,
            ..turn_request(&request)
        };

        for (label, input, api_key) in [
            ("api-key", &with_access, Some("sk-embedded-key")),
            ("dual-token", &with_access, None),
            ("dual-token-no-access", &without_access, None),
        ] {
            let built = build_surface_request("/v1/chat/completions", &body, input, api_key)
                .expect("request builds");
            let present = [API_KEY_HEADER, "authorization"]
                .iter()
                .filter(|name| built.headers().contains_key(**name))
                .count();
            assert_eq!(
                1, present,
                "{label} carried {present} credential sources; exactly one is allowed"
            );
        }
    }

    /// A conversation with every message shape the tool loop produces:
    /// system, user, an assistant turn carrying `tool_calls` with `content:
    /// null`, and the `role: tool` result keyed by `tool_call_id`.
    ///
    /// Using an empty `messages` vec (as an earlier version of the end-to-end
    /// test did) hides every body-construction difference, because most
    /// fields are derived from the messages.
    fn tool_loop_conversation() -> cloudrouter_open_sdk::models::OpenAiChatCompletionRequest {
        use cloudrouter_open_sdk::models::{
            OpenAiChatCompletionRequest, OpenAiChatMessage, OpenAiFunctionCall, OpenAiToolCall,
        };
        OpenAiChatCompletionRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                OpenAiChatMessage {
                    content: Some("You are a helpful agent.".to_string()),
                    role: "system".to_string(),
                    ..Default::default()
                },
                OpenAiChatMessage {
                    content: Some("read a.txt".to_string()),
                    role: "user".to_string(),
                    ..Default::default()
                },
                OpenAiChatMessage {
                    content: None,
                    role: "assistant".to_string(),
                    tool_calls: Some(vec![OpenAiToolCall {
                        id: "call_abc123".to_string(),
                        r#type: "function".to_string(),
                        function: Some(OpenAiFunctionCall {
                            name: "read_file".to_string(),
                            arguments: r#"{"path":"a.txt"}"#.to_string(),
                        }),
                    }]),
                    ..Default::default()
                },
                OpenAiChatMessage {
                    content: Some("hello".to_string()),
                    role: "tool".to_string(),
                    tool_call_id: Some("call_abc123".to_string()),
                    ..Default::default()
                },
            ],
            temperature: Some(0.25),
            max_tokens: Some(512),
            stop: Some("STOP".to_string()),
            ..Default::default()
        }
    }

    /// 🔴 The two arms build their chat body through **different code paths**:
    /// the HTTP arm through `stream_chat_completion_with_tools_blocking`, the
    /// in-process arm through `build_in_process_request`. This asserts they
    /// agree field for field, so a future edit to one cannot silently drift the
    /// other — the failure mode would be a turn that behaves differently
    /// depending only on the deployment profile.
    #[test]
    fn both_arms_build_the_same_chat_tools_body() {
        use crate::chat_stream::build_chat_tools_body;

        let request = tool_loop_conversation();
        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {"name": "read_file", "parameters": {"type": "object"}}
        })];
        let input = CloudRouterTurnRequest {
            protocol: WireProtocol::ChatCompletions,
            auth_token: "auth",
            access_token: None,
            request: request.clone(),
            tools: tools.clone(),
        };

        let (protocol, endpoint, in_process_body) = build_in_process_request(&input);
        let http_body = build_chat_tools_body(&request, &tools);

        assert_eq!(WireProtocol::ChatCompletions, protocol);
        assert_eq!("/v1/chat/completions", endpoint);
        assert_eq!(
            http_body, in_process_body,
            "the in-process arm's tool body must be byte-identical to the HTTP arm's"
        );
        // Assert the fidelity the tool loop depends on, rather than trusting
        // the equality alone (both could be equally wrong).
        assert_eq!(Some("auto"), in_process_body["tool_choice"].as_str());
        assert_eq!(
            Some(false),
            in_process_body["parallel_tool_calls"].as_bool()
        );
        assert_eq!(Some(true), in_process_body["stream"].as_bool());
        let messages = in_process_body["messages"]
            .as_array()
            .expect("messages array");
        assert_eq!(4, messages.len(), "every conversation turn must be sent");
        assert_eq!(
            None,
            messages[2]["content"].as_str(),
            "an assistant tool-call turn must keep `content: null` (not an empty string)"
        );
        assert_eq!(
            Some("call_abc123"),
            messages[2]["tool_calls"][0]["id"].as_str(),
            "the tool_call id must survive so the provider can pair the result"
        );
        assert_eq!(
            Some("call_abc123"),
            messages[3]["tool_call_id"].as_str(),
            "the `role: tool` result must stay paired with its call id"
        );
    }

    /// The same equivalence for the **non-tool** path, where both arms route
    /// through `build_protocol_request_body`. Asserted for every protocol
    /// because the in-process arm computes the endpoint itself (including
    /// Google's `?alt=sse`), independently of the HTTP arm.
    #[test]
    fn both_arms_build_the_same_protocol_body_for_every_protocol() {
        for protocol in [
            WireProtocol::ChatCompletions,
            WireProtocol::AnthropicMessages,
            WireProtocol::GoogleContent,
            WireProtocol::OpenAiResponses,
        ] {
            let request = tool_loop_conversation();
            let input = CloudRouterTurnRequest {
                protocol,
                auth_token: "auth",
                access_token: None,
                request: request.clone(),
                tools: Vec::new(),
            };

            let (resolved, endpoint, in_process_body) = build_in_process_request(&input);
            let http_body =
                crate::wire_protocol::build_protocol_request_body(protocol, &request, true);

            assert_eq!(protocol, resolved, "{protocol:?} must keep its own wire");
            assert_eq!(
                protocol.streaming_endpoint(&request.model),
                endpoint,
                "{protocol:?} endpoint must match the shared resolver"
            );
            assert_eq!(
                http_body, in_process_body,
                "{protocol:?} body must be identical across both arms"
            );
            // A streaming request is expressed differently per protocol: the
            // JSON protocols carry a `stream` boolean, Google signals it through
            // the URL and has no such field. Assert the protocol-appropriate
            // form rather than "every body has stream: true".
            match protocol {
                WireProtocol::GoogleContent => assert_eq!(
                    None,
                    in_process_body["stream"].as_bool(),
                    "google streams via the URL and must not gain a `stream` field"
                ),
                other => assert_eq!(
                    Some(true),
                    in_process_body["stream"].as_bool(),
                    "{other:?} must request a stream"
                ),
            }
        }
    }

    /// A tool-bearing request pins the OpenAI chat shape regardless of the
    /// session's protocol.
    ///
    /// Reachability note: the production caller
    /// (`cloud_router_executor::run_cloud_router_turn`) already gates tools on
    /// `protocol == ChatCompletions`, so on today's call path this branch never
    /// sees a foreign protocol. It is asserted anyway because the pin is what
    /// makes the helper safe for a future caller that does not pre-filter — and
    /// because a pin that only exists in a comment is a pin that can be deleted
    /// by accident.
    #[test]
    fn tool_requests_pin_chat_completions_even_from_a_foreign_protocol() {
        let request = tool_loop_conversation();
        let input = CloudRouterTurnRequest {
            protocol: WireProtocol::AnthropicMessages,
            auth_token: "auth",
            access_token: None,
            request,
            tools: vec![serde_json::json!({"type": "function"})],
        };

        let (protocol, endpoint, body) = build_in_process_request(&input);
        assert_eq!(WireProtocol::ChatCompletions, protocol);
        assert_eq!("/v1/chat/completions", endpoint);
        assert_eq!(Some(true), body["stream"].as_bool());
    }
}
