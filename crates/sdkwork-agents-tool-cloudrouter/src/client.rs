//! Cloud Router open-api client adapter for the media tool family.
//!
//! Provides a thin, shared wrapper over `cloudrouter_open_sdk::SdkworkAiClient`
//! with auth-token account-pool routing, a dedicated blocking runtime for
//! synchronous kernel tool invocation, and stable error mapping.

use std::sync::OnceLock;

use cloudrouter_open_sdk::SdkworkAiClient;
use sdkwork_agents_tool_contract::MediaToolError;
use sdkwork_utils_rust::service_base_url::{
    mode_of_deployment_profile, resolve, ResolvedBaseUrl, ServiceBaseUrlRequest,
};

/// Environment variable for the cloudrouter gateway base URL. Shared with the
/// chat turn executor so one configuration governs every cloudrouter path.
pub const ENV_CLOUDROUTER_BASE_URL: &str = "SDKWORK_AGENTS_CLOUDROUTER_BASE_URL";

/// Environment variable carrying the gateway's **authored** public HTTP URL
/// (`SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL`).
///
/// This is the topology value the deployment profile declares: an absolute
/// domain in cloud (`https://router.sdkwork.com`) and the standalone origin in
/// a split deployment. It is the only value this resolver may treat as the
/// gateway base URL — see [`cloudrouter_base_url`].
pub const ENV_CLOUDROUTER_PUBLIC_HTTP_URL: &str =
    "SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL";

/// Environment variable carrying the gateway's own public ingress bind.
///
/// **Not read by the resolver.** It is retained only so the topology's bind
/// value has a named home for diagnostics; reading it to build a base URL is
/// prohibited (see [`cloudrouter_base_url_optional`]). The embedded topology is
/// detected from the deployment profile, never from this bind.
pub const ENV_CLOUDROUTER_INGRESS_BIND: &str =
    "SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_INGRESS_BIND";

/// Default cloudrouter gateway base URL (split-deployment default: the standard
/// platform listener port). Only reached when no authored topology value and no
/// explicit override is present — never in the embedded topology, which returns
/// `None` from [`cloudrouter_base_url_optional`] instead.
pub const DEFAULT_CLOUDROUTER_BASE_URL: &str = "http://127.0.0.1:3900";

/// Cloud Router media client bound to one gateway base URL.
///
/// The client is lightweight and cheap to construct per call; authentication
/// is applied per invocation via [`CloudRouterMediaClient::with_auth_token`]
/// so caller tokens never leak between requests or sessions.
#[derive(Debug, Clone)]
pub struct CloudRouterMediaClient {
    base_url: String,
}

impl CloudRouterMediaClient {
    /// Builds a client for the gateway base URL from the environment
    /// (`SDKWORK_AGENTS_CLOUDROUTER_BASE_URL`) with the shared default.
    pub fn from_env() -> Self {
        Self::with_base_url(cloudrouter_base_url())
    }

    /// Builds a client for an explicit gateway base URL.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// The resolved gateway base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Creates an `SdkworkAiClient` configured with the caller's auth token
    /// (account-pool routing; no API key required).
    pub fn with_auth_token(&self, auth_token: &str) -> Result<SdkworkAiClient, MediaToolError> {
        let client =
            SdkworkAiClient::new_with_base_url(self.base_url.clone()).map_err(|error| {
                MediaToolError::ProviderUnavailable(format!(
                    "cloudrouter client unavailable: {error}"
                ))
            })?;
        client.set_auth_token(auth_token);
        Ok(client)
    }

    /// Attaches the inbound request trace context to the generated SDK client
    /// so the cloudrouter gateway sees the same trace id as the agents turn:
    /// `x-trace-id` carries the id, and a W3C `traceparent` is synthesized for
    /// gateway-side span correlation.
    pub fn with_trace_id(&self, sdk: &SdkworkAiClient, trace_id: Option<&str>) -> &Self {
        let Some(trace_id) = trace_id.map(str::trim).filter(|value| !value.is_empty()) else {
            return self;
        };
        sdk.set_header("x-trace-id", trace_id);
        if trace_id.len() == 32 {
            sdk.set_header("traceparent", format!("00-{trace_id}-0000000000000000-01"));
        }
        self
    }

    /// Requires a non-empty auth token, mapping absence to an actionable error.
    pub fn require_auth_token<'a>(
        auth_token: Option<&'a str>,
        tool_id: &str,
    ) -> Result<&'a str, MediaToolError> {
        auth_token
            .filter(|token| !token.trim().is_empty())
            .ok_or_else(|| {
                MediaToolError::AuthRequired(format!(
                    "tool `{tool_id}` requires the caller auth token for cloudrouter \
                     account-pool routing; the turn or request carried none"
                ))
            })
    }
}

/// SDKWork result code for a wallet/funding shortfall (API_SPEC §15.3).
pub const CLOUDROUTER_INSUFFICIENT_BALANCE_CODE: u16 = 40201;

/// `KernelError` detail key marking a self-healable funding shortfall.
///
/// The kernel taxonomy has no dedicated funding variant, so the cause rides on
/// a structured detail that HTTP boundaries can key off to emit a 402 instead
/// of a 503. Kept here so every cloudrouter caller tags it identically.
pub const FUNDING_SHORTFALL_DETAIL_KEY: &str = "funding_shortfall";

/// Whether a cloudrouter failure is a self-healable funding shortfall.
///
/// The gateway reports a rejected billing hold as HTTP 402 with result code
/// `40201`; older builds stamped the same condition as `50201` with
/// `failedStage: billing_precharge`. Both shapes are recognized here so the
/// caller can offer a recharge entry point regardless of gateway vintage.
pub fn is_cloudrouter_insufficient_balance(status: u16, body: &str) -> bool {
    if status == 402 {
        return true;
    }
    if body.contains("\"code\":40201") || body.contains("\"code\": 40201") {
        return true;
    }
    // Pre-402 shape: the shortfall was reported as a bad gateway with the
    // precharge stage. Guarded by the stage/message so a genuine upstream
    // outage is never mistaken for a funding problem.
    if status == 502 || status == 503 {
        return body.contains("billing_precharge")
            || body.contains("insufficient available balance")
            || body.contains("insufficient spendable points");
    }
    false
}

/// Maps a cloudrouter HTTP error body (SDKWork problem JSON whose `detail`
/// the gateway preserves from `x-sdkwork-route-reason`) to an actionable
/// operator hint. Kept in one place so tool, rig, and turn-executor wrappers
/// surface the same message for the same root cause.
///
/// A funding shortfall returns an empty hint: that case is not an operator
/// problem, it is the *end user's* balance, and the caller must present a
/// recharge affordance instead of appending an account-pool diagnostic.
pub fn cloudrouter_http_error_hint(status: u16, body: &str) -> &'static str {
    if is_cloudrouter_insufficient_balance(status, body) {
        return "";
    }
    if status >= 500 {
        if body.contains("upstream")
            || body.contains("credential")
            || body.contains("api key")
            || body.contains("401")
        {
            return "; 上游账号凭证被拒绝（401）：请在 Cloud Router 后台检查 DeepSeek 账号的 API Key 是否有效";
        }
        if body.contains("circuit") || body.contains("breaker") {
            return "; 账号池路由熔断保护已触发，请稍后重试";
        }
        if body.contains("pricing") {
            return "; 账号池定价配置缺失：请检查上游成本价配置";
        }
        // The gateway reports an opaque cause when the route reason never made
        // it onto the response (`detail: "Bad gateway"`). A generic hint here
        // leaves an operator with nothing to act on, which is exactly how the
        // account-pool outage was first reported. Name the missing evidence and
        // the two checks that distinguish "no route configured" from "the
        // upstream account is unreachable".
        if body.contains("Bad gateway") {
            return "; Cloud Router 未透出路由失败原因（detail=Bad gateway）：请检查账号池是否已为所选模型配置可用账号，以及上游厂商连接是否可达（provider connect timeout / deadline elapsed）";
        }
    }
    "; Cloud Router 账号池网关暂不可用，请稍后重试"
}

/// Runs a cloudrouter SDK async call on a dedicated blocking runtime,
/// returning the mapped media tool error on failure.
///
/// The generated SDK transport enforces its own request timeout
/// (`SdkworkConfig::timeout_ms`, default 30s), so a hung gateway cannot
/// block the worker indefinitely; the media tool handler additionally
/// bounds the whole invocation.
pub fn run_sync<T>(
    tool_id: &str,
    call: impl FnOnce(&tokio::runtime::Runtime) -> Result<T, cloudrouter_open_sdk::SdkworkError>,
) -> Result<T, MediaToolError> {
    let runtime = blocking_runtime();
    call(runtime).map_err(|error| map_cloudrouter_error(tool_id, error))
}

/// Dedicated multi-thread runtime for blocking kernel tool invocation.
pub(crate) fn blocking_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("cloud router media tool tokio runtime")
    })
}

/// The deployment profile the process runs under, read from the canonical
/// profile variables (`SDKWORK_DEPLOYMENT_PROFILE`, then the application-scoped
/// alias). `None` when the process publishes neither.
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

/// Resolves the cloudrouter gateway base URL, or `None` when this process is
/// the gateway that hosts the surface in-process.
///
/// The decision itself lives in the shared component
/// [`sdkwork_utils_rust::service_base_url`], so every backend SDK consumer in
/// the workspace resolves an origin the same way instead of re-deriving one per
/// call. This function only supplies cloudrouter's environment inputs.
///
/// Resolution order (`APP_SDK_INTEGRATION_SPEC.md` §5.2):
/// 1. `SDKWORK_AGENTS_CLOUDROUTER_BASE_URL` — explicit override, the only
///    loopback-free escape hatch and the value a split deployment sets;
/// 2. in the embedded (standalone) topology, stop: the dependency is composed
///    in-process, so there is no HTTP base URL to derive. Reading the gateway's
///    own ingress bind and formatting `http://127.0.0.1:{port}` from it is
///    forbidden — that is the self-loop `APPLICATION_GATEWAY_SPEC.md` §2.3
///    prohibits, and it silently produced the production `Bad gateway`
///    incident;
/// 3. the **authored** gateway public HTTP URL
///    (`SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL`) — an absolute
///    domain in cloud, an authored origin in a split deployment;
/// 4. the split-deployment default (`http://127.0.0.1:3900`).
pub fn cloudrouter_base_url_optional() -> Option<String> {
    resolve_cloudrouter_base_url().base_url().map(str::to_owned)
}

/// The full resolution, including which source supplied the value.
///
/// Prefer this where the outcome is logged or turned into a problem detail:
/// the source distinguishes a configured value from a fallback, which is what
/// makes a misconfiguration diagnosable.
pub fn resolve_cloudrouter_base_url() -> ResolvedBaseUrl {
    let request = ServiceBaseUrlRequest::new(
        "cloudrouter",
        mode_of_deployment_profile(deployment_profile().as_deref()),
    )
    .with_explicit_override(std::env::var(ENV_CLOUDROUTER_BASE_URL).ok())
    .with_authored_public_url(std::env::var(ENV_CLOUDROUTER_PUBLIC_HTTP_URL).ok())
    .with_split_default(DEFAULT_CLOUDROUTER_BASE_URL);
    resolve(&request)
}

/// Resolves the cloudrouter gateway base URL from the environment with the
/// shared default.
///
/// Convenience wrapper over [`cloudrouter_base_url_optional`] that falls back to
/// the split-deployment default for callers that cannot represent "no HTTP
/// transport". Callers on a turn path that can reach a gateway hosted
/// in-process **must** use the optional form so the missing transport is
/// handled explicitly instead of silently dialling a self-loop.
pub fn cloudrouter_base_url() -> String {
    cloudrouter_base_url_optional().unwrap_or_else(|| DEFAULT_CLOUDROUTER_BASE_URL.to_string())
}

/// Maps a cloudrouter SDK failure to the media tool error taxonomy with
/// actionable hints for the common gateway failures.
pub fn map_cloudrouter_error(
    tool_id: &str,
    error: cloudrouter_open_sdk::SdkworkError,
) -> MediaToolError {
    use cloudrouter_open_sdk::SdkworkError;

    let mut message = format!("tool `{tool_id}` cloudrouter call failed: {error}");
    match &error {
        SdkworkError::HttpStatus { status, .. } if *status == 401 => {
            message.push_str("; 登录 auth token 无效或已过期，请重新登录后重试");
            MediaToolError::AuthRequired(message)
        }
        SdkworkError::HttpStatus { status, body }
            if *status == 404 && body.contains("model_not_found") =>
        {
            message.push_str("; 所选模型在账号池路由中不可用：请在 Cloud Router 中为该供应商配置模型映射规则或供应商支持模型");
            MediaToolError::ProviderError(message)
        }
        SdkworkError::HttpStatus { status, body }
            if *status == 404 && body.contains("account_group_unavailable") =>
        {
            message.push_str("; 当前租户在账号池中未配置默认分组（Default）或分组下无可用账号");
            MediaToolError::ProviderUnavailable(message)
        }
        SdkworkError::HttpStatus { status, .. } if *status == 429 => {
            message.push_str("; Cloud Router 配额或限流触发，请稍后重试");
            MediaToolError::RateLimited(message)
        }
        SdkworkError::HttpStatus { status, body } if *status >= 500 => {
            message.push_str(cloudrouter_http_error_hint(*status, body));
            MediaToolError::ProviderUnavailable(message)
        }
        _ => MediaToolError::ProviderError(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudrouter_open_sdk::SdkworkError;
    use std::sync::{Mutex, OnceLock};

    /// Serializes env-mutating base-url tests: `std::env` is process-global,
    /// so parallel tests would race on the same variables.
    fn env_guard() -> &'static Mutex<()> {
        static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        GUARD.get_or_init(|| Mutex::new(()))
    }

    /// Snapshot/restore the base-URL resolution inputs so the process-global
    /// environment is left exactly as the test found it.
    struct BaseUrlEnv {
        base_url: Option<String>,
        public_http_url: Option<String>,
        ingress_bind: Option<String>,
        profile: Option<String>,
        app_profile: Option<String>,
    }

    impl BaseUrlEnv {
        fn capture() -> Self {
            let read = |key: &str| std::env::var(key).ok();
            let guard = Self {
                base_url: read(ENV_CLOUDROUTER_BASE_URL),
                public_http_url: read(ENV_CLOUDROUTER_PUBLIC_HTTP_URL),
                ingress_bind: read(ENV_CLOUDROUTER_INGRESS_BIND),
                profile: read("SDKWORK_DEPLOYMENT_PROFILE"),
                app_profile: read("SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE"),
            };
            for key in [
                ENV_CLOUDROUTER_BASE_URL,
                ENV_CLOUDROUTER_PUBLIC_HTTP_URL,
                ENV_CLOUDROUTER_INGRESS_BIND,
                "SDKWORK_DEPLOYMENT_PROFILE",
                "SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE",
            ] {
                std::env::remove_var(key);
            }
            guard
        }

        fn set(&self, key: &str, value: &str) {
            std::env::set_var(key, value);
        }
    }

    impl Drop for BaseUrlEnv {
        fn drop(&mut self) {
            for (key, value) in [
                (ENV_CLOUDROUTER_BASE_URL, &self.base_url),
                (ENV_CLOUDROUTER_PUBLIC_HTTP_URL, &self.public_http_url),
                (ENV_CLOUDROUTER_INGRESS_BIND, &self.ingress_bind),
                ("SDKWORK_DEPLOYMENT_PROFILE", &self.profile),
                ("SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE", &self.app_profile),
            ] {
                restore_env(key, value.clone());
            }
        }
    }

    #[test]
    fn client_base_url_from_env_with_default() {
        let _guard = env_guard().lock().expect("env test lock");
        let env = BaseUrlEnv::capture();
        assert_eq!(cloudrouter_base_url(), DEFAULT_CLOUDROUTER_BASE_URL);
        env.set(ENV_CLOUDROUTER_BASE_URL, "http://example.test:4000");
        assert_eq!(cloudrouter_base_url(), "http://example.test:4000");
    }

    #[test]
    fn embedded_profile_derives_no_loopback_base_url_from_the_ingress_bind() {
        // The regression that produced the production `Bad gateway` incident:
        // the standalone profile publishes the gateway's own ingress bind, and
        // the surface runs inside that gateway. Deriving
        // `http://127.0.0.1:{port}` from the bind dials the process itself.
        let _guard = env_guard().lock().expect("env test lock");
        let env = BaseUrlEnv::capture();
        env.set("SDKWORK_DEPLOYMENT_PROFILE", "standalone");
        env.set(ENV_CLOUDROUTER_INGRESS_BIND, "0.0.0.0:3905");
        env.set(ENV_CLOUDROUTER_PUBLIC_HTTP_URL, "http://127.0.0.1:3905");

        assert_eq!(cloudrouter_base_url_optional(), None);
        // The non-optional convenience form must not resurrect the loopback
        // either; without an authored value it degrades to the split default.
        assert_eq!(cloudrouter_base_url(), DEFAULT_CLOUDROUTER_BASE_URL);
    }

    #[test]
    fn explicit_override_wins_even_in_the_embedded_profile() {
        // A split deployment that hosts the surface separately sets the
        // override; it is the sanctioned loopback-free escape hatch.
        let _guard = env_guard().lock().expect("env test lock");
        let env = BaseUrlEnv::capture();
        env.set("SDKWORK_DEPLOYMENT_PROFILE", "standalone");
        env.set(ENV_CLOUDROUTER_INGRESS_BIND, "0.0.0.0:3905");
        env.set(ENV_CLOUDROUTER_BASE_URL, "https://router.sdkwork.com");

        assert_eq!(
            cloudrouter_base_url_optional().as_deref(),
            Some("https://router.sdkwork.com")
        );
    }

    #[test]
    fn cloud_profile_resolves_the_authored_absolute_domain() {
        // Cloud publishes an absolute domain, never a loopback port.
        let _guard = env_guard().lock().expect("env test lock");
        let env = BaseUrlEnv::capture();
        env.set("SDKWORK_DEPLOYMENT_PROFILE", "cloud");
        env.set(ENV_CLOUDROUTER_INGRESS_BIND, "0.0.0.0:3900");
        env.set(
            ENV_CLOUDROUTER_PUBLIC_HTTP_URL,
            "https://router.sdkwork.com",
        );

        assert_eq!(
            cloudrouter_base_url_optional().as_deref(),
            Some("https://router.sdkwork.com")
        );
    }

    #[test]
    fn split_deployment_without_authored_url_uses_the_shared_default() {
        let _guard = env_guard().lock().expect("env test lock");
        let _env = BaseUrlEnv::capture();
        assert_eq!(
            cloudrouter_base_url_optional().as_deref(),
            Some(DEFAULT_CLOUDROUTER_BASE_URL)
        );
    }

    #[test]
    fn require_auth_token_rejects_empty() {
        assert!(CloudRouterMediaClient::require_auth_token(None, "audio.speech.create").is_err());
        assert!(
            CloudRouterMediaClient::require_auth_token(Some(""), "audio.speech.create").is_err()
        );
        assert!(
            CloudRouterMediaClient::require_auth_token(Some("  "), "audio.speech.create").is_err()
        );
        assert_eq!(
            CloudRouterMediaClient::require_auth_token(Some("token"), "audio.speech.create")
                .unwrap(),
            "token"
        );
    }

    #[test]
    fn maps_unauthorized_with_login_hint() {
        let error = map_cloudrouter_error(
            "audio.speech.create",
            SdkworkError::HttpStatus {
                status: 401,
                body: r#"{"error":{"code":"invalid_auth_token","message":"invalid"}}"#.to_string(),
            },
        );
        assert_eq!(error.code(), "auth_required");
        assert!(error.to_string().contains("重新登录"));
    }

    #[test]
    fn maps_rate_limit_and_gateway_errors() {
        let rate = map_cloudrouter_error(
            "image.generations.create",
            SdkworkError::HttpStatus {
                status: 429,
                body: "rate limited".to_string(),
            },
        );
        assert_eq!(rate.code(), "rate_limited");

        let gateway = map_cloudrouter_error(
            "video.create",
            SdkworkError::HttpStatus {
                status: 503,
                body: "unavailable".to_string(),
            },
        );
        assert_eq!(gateway.code(), "provider_unavailable");
    }

    #[test]
    fn funding_shortfall_is_recognized_across_gateway_vintages() {
        // Current contract: HTTP 402 with result code 40201.
        assert!(is_cloudrouter_insufficient_balance(
            402,
            r#"{"code":40201,"detail":"insufficient available balance for hold"}"#
        ));
        // Same contract with a spaced JSON encoding.
        assert!(is_cloudrouter_insufficient_balance(
            400,
            r#"{"code": 40201}"#
        ));
        // Pre-402 shape: stamped as a bad gateway with the precharge stage.
        assert!(is_cloudrouter_insufficient_balance(
            502,
            r#"{"code":50201,"failedStage":"billing_precharge","detail":"insufficient available balance for hold"}"#
        ));
        assert!(is_cloudrouter_insufficient_balance(
            503,
            "insufficient spendable points lots for hold"
        ));
    }

    #[test]
    fn genuine_upstream_outages_are_not_funding_shortfalls() {
        // A plain 5xx with no funding marker must stay an infrastructure fault,
        // otherwise every provider outage would be reported as "recharge".
        assert!(!is_cloudrouter_insufficient_balance(
            502,
            r#"{"code":50201,"failedStage":"dispatch_failed","detail":"upstream returned 500"}"#
        ));
        assert!(!is_cloudrouter_insufficient_balance(503, "unavailable"));
        assert!(!is_cloudrouter_insufficient_balance(
            401,
            "invalid auth token"
        ));
    }

    #[test]
    fn funding_shortfall_yields_no_operator_hint() {
        // The account-pool diagnostic tells the *end user* to check server-side
        // pricing config, which they cannot act on; the funding case must not
        // carry it. Every other 5xx keeps its hint.
        assert_eq!(
            cloudrouter_http_error_hint(
                502,
                r#"{"detail":"insufficient available balance for hold"}"#
            ),
            ""
        );
        assert!(
            !cloudrouter_http_error_hint(503, "upstream account credential rejected").is_empty()
        );
    }

    #[test]
    fn opaque_bad_gateway_still_yields_an_actionable_hint() {
        // The gateway reports `detail: "Bad gateway"` when the route reason was
        // never attached. That body matched no branch, so the operator got a
        // generic "temporarily unavailable" and no way to tell "no account
        // configured" from "the upstream account is unreachable".
        let hint = cloudrouter_http_error_hint(
            502,
            r#"{"code":50201,"failedStage":"provider_http_transport_failed","detail":"Bad gateway"}"#,
        );
        assert!(!hint.is_empty());
        assert!(hint.contains("Bad gateway"));
        assert!(hint.contains("账号池"));
        // It must stay a distinct hint, not the generic fallback.
        assert_ne!(
            hint,
            cloudrouter_http_error_hint(502, r#"{"detail":"something else entirely"}"#)
        );
    }

    #[test]
    fn funding_shortfall_still_maps_to_the_media_taxonomy() {
        // The media-tool path has no funding variant; what matters is that it
        // is not reported as "provider unavailable" (which reads as an outage).
        let error = map_cloudrouter_error(
            "image.generations.create",
            SdkworkError::HttpStatus {
                status: 402,
                body: r#"{"code":40201,"detail":"insufficient available balance for hold"}"#
                    .to_string(),
            },
        );
        assert_ne!(error.code(), "provider_unavailable");
    }

    fn restore_env(key: &str, value: Option<String>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
