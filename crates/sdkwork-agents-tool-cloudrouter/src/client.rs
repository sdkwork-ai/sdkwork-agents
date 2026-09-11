//! Cloud Router open-api client adapter for the media tool family.
//!
//! Provides a thin, shared wrapper over `cloudrouter_open_sdk::SdkworkAiClient`
//! with auth-token account-pool routing, a dedicated blocking runtime for
//! synchronous kernel tool invocation, and stable error mapping.

use std::sync::OnceLock;

use cloudrouter_open_sdk::SdkworkAiClient;
use sdkwork_agents_tool_contract::MediaToolError;

/// Environment variable for the cloudrouter gateway base URL. Shared with the
/// chat turn executor so one configuration governs every cloudrouter path.
pub const ENV_CLOUDROUTER_BASE_URL: &str = "SDKWORK_AGENTS_CLOUDROUTER_BASE_URL";

/// Environment variable carrying the gateway's own public ingress bind
/// (`SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_INGRESS_BIND`). In the
/// federated topology the agents surface runs inside the cloudrouter gateway,
/// so the client targets the gateway's own ingress port instead of a
/// hard-coded one (the standalone development profile binds e.g. :3905).
pub const ENV_CLOUDROUTER_INGRESS_BIND: &str =
    "SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_INGRESS_BIND";

/// Default cloudrouter gateway base URL (shared platform proxy port).
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
        let Some(trace_id) = trace_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return self;
        };
        sdk.set_header("x-trace-id", trace_id);
        if trace_id.len() == 32 {
            sdk.set_header(
                "traceparent",
                format!("00-{trace_id}-0000000000000000-01"),
            );
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

/// Maps a cloudrouter HTTP error body (SDKWork problem JSON whose `detail`
/// the gateway preserves from `x-sdkwork-route-reason`) to an actionable
/// operator hint. Kept in one place so tool, rig, and turn-executor wrappers
/// surface the same message for the same root cause.
pub fn cloudrouter_http_error_hint(status: u16, body: &str) -> &'static str {
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
        if body.contains("pricing") || body.contains("balance") {
            return "; 账号池余额或定价不足：请检查账号池余额与上游成本价配置";
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

/// Resolves the gateway base URL from the environment with the shared default.
///
/// Resolution order:
/// 1. `SDKWORK_AGENTS_CLOUDROUTER_BASE_URL` — explicit override for split
///    (separately deployed) topologies;
/// 2. the gateway's own public ingress bind
///    (`SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_INGRESS_BIND`) mapped to
///    a loopback URL — the federated topology hosts the agents surface inside
///    the cloudrouter gateway, so the executor must target the same port the
///    gateway actually binds (the standalone development profile binds e.g.
///    `0.0.0.0:3905` instead of the default `:3900`);
/// 3. the shared platform proxy port default (`http://127.0.0.1:3900`).
pub fn cloudrouter_base_url() -> String {
    if let Some(value) = std::env::var(ENV_CLOUDROUTER_BASE_URL)
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        return value;
    }
    if let Some(bind) = std::env::var(ENV_CLOUDROUTER_INGRESS_BIND)
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        if let Some((_, port)) = bind.rsplit_once(':') {
            if !port.is_empty() && port.chars().all(|ch| ch.is_ascii_digit()) {
                return format!("http://127.0.0.1:{port}");
            }
        }
    }
    DEFAULT_CLOUDROUTER_BASE_URL.to_string()
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

    #[test]
    fn client_base_url_from_env_with_default() {
        let _guard = env_guard().lock().expect("env test lock");
        let previous = std::env::var(ENV_CLOUDROUTER_BASE_URL).ok();
        let previous_bind = std::env::var(ENV_CLOUDROUTER_INGRESS_BIND).ok();
        std::env::remove_var(ENV_CLOUDROUTER_BASE_URL);
        std::env::remove_var(ENV_CLOUDROUTER_INGRESS_BIND);
        assert_eq!(cloudrouter_base_url(), DEFAULT_CLOUDROUTER_BASE_URL);
        std::env::set_var(ENV_CLOUDROUTER_BASE_URL, "http://example.test:4000");
        assert_eq!(cloudrouter_base_url(), "http://example.test:4000");
        restore_env(ENV_CLOUDROUTER_BASE_URL, previous);
        restore_env(ENV_CLOUDROUTER_INGRESS_BIND, previous_bind);
    }

    #[test]
    fn client_base_url_falls_back_to_gateway_ingress_bind() {
        let _guard = env_guard().lock().expect("env test lock");
        let previous = std::env::var(ENV_CLOUDROUTER_BASE_URL).ok();
        let previous_bind = std::env::var(ENV_CLOUDROUTER_INGRESS_BIND).ok();
        std::env::remove_var(ENV_CLOUDROUTER_BASE_URL);
        std::env::set_var(ENV_CLOUDROUTER_INGRESS_BIND, "0.0.0.0:3905");
        assert_eq!(cloudrouter_base_url(), "http://127.0.0.1:3905");
        std::env::set_var(ENV_CLOUDROUTER_INGRESS_BIND, "127.0.0.1:3900");
        assert_eq!(cloudrouter_base_url(), "http://127.0.0.1:3900");
        // An explicit override always wins over the ingress bind.
        std::env::set_var(ENV_CLOUDROUTER_BASE_URL, "http://example.test:4000");
        assert_eq!(cloudrouter_base_url(), "http://example.test:4000");
        restore_env(ENV_CLOUDROUTER_BASE_URL, previous);
        restore_env(ENV_CLOUDROUTER_INGRESS_BIND, previous_bind);
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

    fn restore_env(key: &str, value: Option<String>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
