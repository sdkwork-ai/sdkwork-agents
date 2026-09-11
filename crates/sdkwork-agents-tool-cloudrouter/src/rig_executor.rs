//! Cloud Router account-pool Rig executor — the default RIG model backend.
//!
//! Every RIG agent model invocation is routed through
//! [`cloudrouter_open_sdk::SdkworkAiClient`] to the Cloud Router open-api
//! chat completions gateway. The account-pool routing pipeline (default group
//! → accounts → upstream suppliers) selects the supplier for the requested
//! model, so no local provider binding or API key configuration is required.
//!
//! Credential precedence per invocation:
//! 1. caller dual tokens (`ModelRequest.auth_token` + `access_token`) — the
//!    default for authenticated product calls (API_SPEC §819/§824);
//! 2. the configured `llm.rig.api_key` host secret — API-key mode for
//!    worker/backend flows without caller tokens;
//! 3. fail-closed [`KernelError::provider_error`] when neither is available.

use std::sync::Arc;

use cloudrouter_open_sdk::models::{OpenAiChatCompletionRequest, OpenAiChatMessage};
use cloudrouter_open_sdk::{SdkworkAiClient, SdkworkConfig};
use serde_json::Value;
use sdkwork_agent_kernel::{
    HostProvider, KernelError, KernelResult, ModelDescriptor, ModelProvider, ModelRequest,
    ModelResponse, ModelStreamChunk, ModelStreamSink, ProviderHealth, ProviderManifest, SecretRef,
};
use sdkwork_agent_provider_rig::{ids, RigBackendConfig, RigBackendExecutor, RigModelProvider};

use crate::chat_stream::stream_chat_completion_blocking;
use crate::chat_stream::CloudRouterStreamDelta;

/// Fallback model key sent when the turn carries no model id; the gateway
/// account-pool router resolves it to the tenant's default account.
const DEFAULT_MODEL_KEY: &str = "default";

/// Minimum gateway request timeout (1s) so a misconfigured budget cannot
/// abort the call before the HTTP exchange begins.
const MIN_REQUEST_TIMEOUT_MS: u64 = 1_000;

/// Maximum gateway request timeout (30min) — a hard ceiling aligned with the
/// turn execution budget for long coding completions, not a user-facing
/// setting. Stalled upstreams fail fast via read/idle timeouts long before
/// this ceiling.
const MAX_REQUEST_TIMEOUT_MS: u64 = 1_800_000;

/// Cloud Router account-pool executor for the RIG agent engine.
///
/// The executor is cheap to construct per engine-host bootstrap and stateless
/// between invocations: credentials are applied per request from the caller
/// tokens carried by the [`ModelRequest`] or the configured host secret, so
/// tokens never leak between sessions.
pub struct RigCloudRouterExecutor {
    config: RigBackendConfig,
    host: Arc<dyn HostProvider + Send + Sync>,
    base_url: String,
}

impl RigCloudRouterExecutor {
    /// Builds the executor for the configured backend and secret host,
    /// resolving the gateway base URL from the shared environment defaults.
    pub fn new(config: RigBackendConfig, host: Arc<dyn HostProvider + Send + Sync>) -> Self {
        Self {
            config,
            host,
            base_url: crate::cloudrouter_base_url(),
        }
    }

    /// Builds the executor for an explicit gateway base URL (tests).
    pub fn with_base_url(
        config: RigBackendConfig,
        host: Arc<dyn HostProvider + Send + Sync>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            config,
            host,
            base_url: base_url.into(),
        }
    }

    /// The resolved gateway base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn build_client(&self, request: &ModelRequest) -> Result<SdkworkAiClient, KernelError> {
        let mut config = SdkworkConfig::new(self.base_url.clone());
        config.timeout_ms = request
            .timeout_ms
            .map(|timeout_ms| timeout_ms.clamp(MIN_REQUEST_TIMEOUT_MS, MAX_REQUEST_TIMEOUT_MS))
            .unwrap_or(MAX_REQUEST_TIMEOUT_MS);
        SdkworkAiClient::new(config).map_err(|error| {
            KernelError::provider_error(
                "rig_cloudrouter_client_unavailable",
                format!("cloud router SDK client unavailable: {error}"),
            )
        })
    }

    fn apply_credentials(
        &self,
        client: &SdkworkAiClient,
        request: &ModelRequest,
    ) -> KernelResult<()> {
        if let Some(auth_token) = request.auth_token.as_deref().filter(|token| !token.trim().is_empty())
        {
            // Dual-token access per API_SPEC §819/§824: the gateway resolves
            // the account route context from the auth token and carries the
            // access token as the session access context. `set_access_token`
            // runs first so the `Authorization` bearer set by `set_auth_token`
            // below is never dropped by SDK header hygiene.
            if let Some(access_token) = request
                .access_token
                .as_deref()
                .filter(|token| !token.trim().is_empty())
            {
                client.set_access_token(access_token);
            }
            client.set_auth_token(auth_token);
            return Ok(());
        }
        if let Some(secret_ref) = self
            .config
            .api_key_secret_ref
            .as_deref()
            .filter(|secret_ref| !secret_ref.trim().is_empty())
        {
            let secret = self
                .host
                .resolve_secret(SecretRef::new(secret_ref, "Rig cloud router API key"))?;
            client.set_api_key(secret.expose_value());
            return Ok(());
        }
        Err(KernelError::provider_error(
            "rig_cloudrouter_credentials_unavailable",
            "RIG cloud router executor requires the caller auth token or a configured \
             llm.rig.api_key secret; neither was supplied",
        ))
    }

    /// Resolves bearer credentials for the blocking stream client.
    fn resolve_stream_credentials(
        &self,
        request: &ModelRequest,
    ) -> KernelResult<(String, Option<String>)> {
        if let Some(auth_token) = request
            .auth_token
            .as_deref()
            .filter(|token| !token.trim().is_empty())
        {
            let access_token = request
                .access_token
                .as_deref()
                .filter(|token| !token.trim().is_empty())
                .map(str::to_string);
            return Ok((auth_token.to_string(), access_token));
        }
        if let Some(secret_ref) = self
            .config
            .api_key_secret_ref
            .as_deref()
            .filter(|secret_ref| !secret_ref.trim().is_empty())
        {
            let secret = self
                .host
                .resolve_secret(SecretRef::new(secret_ref, "Rig cloud router API key"))?;
            return Ok((secret.expose_value().to_string(), None));
        }
        Err(KernelError::provider_error(
            "rig_cloudrouter_credentials_unavailable",
            "RIG cloud router executor requires the caller auth token or a configured \
             llm.rig.api_key secret; neither was supplied",
        ))
    }

    fn stream_completion_request(
        &self,
        request: &ModelRequest,
    ) -> OpenAiChatCompletionRequest {
        let mut completion_request = build_chat_completion_request(request);
        completion_request.stream = Some(true);
        completion_request
    }
}

impl RigBackendExecutor for RigCloudRouterExecutor {
    fn invoke_model(&self, request: ModelRequest) -> KernelResult<ModelResponse> {
        let request_id = request.model_request_id.clone();
        let client = self.build_client(&request)?;
        self.apply_credentials(&client, &request)?;

        let completion_request = build_chat_completion_request(&request);
        let completion = crate::client::blocking_runtime()
            .block_on(client.chat().create(&completion_request))
            .map_err(map_cloudrouter_kernel_error)?;

        let choice = completion.choices.first().ok_or_else(|| {
            KernelError::provider_error(
                "rig_cloudrouter_empty_response",
                "cloud router returned no completion choices",
            )
        })?;
        let content = choice
            .message
            .content
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                KernelError::provider_error(
                    "rig_cloudrouter_empty_response",
                    "cloud router returned no assistant content",
                )
            })?;

        let finish_reason = choice
            .finish_reason
            .clone()
            .unwrap_or_else(|| "stop".to_string());
        Ok(ModelResponse::text(request_id, ids::MODEL_PROVIDER_ID, content)
            .with_model_id(completion.model.clone())
            .with_finish_reason(finish_reason))
    }
}

/// RIG model provider that adds live Cloud Router streaming on top of the
/// standard invoke-only [`RigModelProvider`].
#[derive(Clone)]
pub struct RigCloudRouterModelProvider {
    inner: RigModelProvider,
    executor: Arc<RigCloudRouterExecutor>,
}

impl RigCloudRouterModelProvider {
    pub fn new(
        config: RigBackendConfig,
        host: Arc<dyn HostProvider + Send + Sync>,
    ) -> Self {
        let executor = Arc::new(RigCloudRouterExecutor::new(config.clone(), host));
        Self {
            inner: RigModelProvider::with_executor(config, executor.clone()),
            executor,
        }
    }

    pub fn with_base_url(
        config: RigBackendConfig,
        host: Arc<dyn HostProvider + Send + Sync>,
        base_url: impl Into<String>,
    ) -> Self {
        let executor = Arc::new(RigCloudRouterExecutor::with_base_url(
            config.clone(),
            host,
            base_url,
        ));
        Self {
            inner: RigModelProvider::with_executor(config, executor.clone()),
            executor,
        }
    }
}

impl ModelProvider for RigCloudRouterModelProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        self.inner.provider_manifest()
    }

    fn health(&self) -> ProviderHealth {
        self.inner.health()
    }

    fn list_models(&self) -> Vec<ModelDescriptor> {
        self.inner.list_models()
    }

    fn invoke(&self, request: ModelRequest) -> KernelResult<ModelResponse> {
        self.inner.invoke(request)
    }

    fn stream(&self, request: ModelRequest) -> KernelResult<Vec<ModelStreamChunk>> {
        let mut chunks = Vec::new();
        self.stream_into(
            request,
            &mut CollectingModelStreamSink {
                chunks: &mut chunks,
            },
        )?;
        Ok(chunks)
    }

    fn stream_into(
        &self,
        request: ModelRequest,
        sink: &mut dyn ModelStreamSink,
    ) -> KernelResult<()> {
        let model_request_id = request.model_request_id.clone();
        let (auth_token, access_token) = self.executor.resolve_stream_credentials(&request)?;
        let completion_request = self.executor.stream_completion_request(&request);
        let mut sequence = 0u64;
        let mut sink_error: Option<KernelError> = None;
        let mut emit_delta = |delta: CloudRouterStreamDelta| -> KernelResult<()> {
            if !delta.reasoning_content.is_empty() {
                sequence += 1;
                let chunk = ModelStreamChunk::reasoning(
                    &model_request_id,
                    sequence,
                    &delta.reasoning_content,
                );
                sink.push_chunk(chunk)?;
            }
            if !delta.content.is_empty() {
                sequence += 1;
                let chunk = ModelStreamChunk::output(&model_request_id, sequence, &delta.content);
                sink.push_chunk(chunk)?;
            }
            if !delta.tool_calls.is_empty() {
                for (tool_call_id, arguments) in
                    extract_tool_call_argument_fragments(&delta.tool_calls)
                {
                    sequence += 1;
                    let chunk = ModelStreamChunk::tool_arguments(
                        &model_request_id,
                        sequence,
                        tool_call_id,
                        arguments,
                    );
                    sink.push_chunk(chunk)?;
                }
            }
            Ok(())
        };
        stream_chat_completion_blocking(
            self.executor.base_url(),
            &auth_token,
            access_token.as_deref(),
            completion_request,
            &mut |delta: CloudRouterStreamDelta| {
                if sink_error.is_some() {
                    return;
                }
                if let Err(error) = emit_delta(delta) {
                    sink_error = Some(error);
                }
            },
        )
        .map_err(map_cloudrouter_kernel_error)?;
        if let Some(error) = sink_error {
            return Err(error);
        }
        Ok(())
    }

    fn cancel(&self, model_request_id: &str) -> KernelResult<ModelResponse> {
        self.inner.cancel(model_request_id)
    }
}

struct CollectingModelStreamSink<'a> {
    chunks: &'a mut Vec<ModelStreamChunk>,
}

impl ModelStreamSink for CollectingModelStreamSink<'_> {
    fn push_chunk(&mut self, chunk: ModelStreamChunk) -> KernelResult<()> {
        self.chunks.push(chunk);
        Ok(())
    }

    fn push_event(
        &mut self,
        _event: sdkwork_agent_kernel::KernelEvent,
    ) -> KernelResult<()> {
        Ok(())
    }
}

/// Builds the OpenAI chat completion request from the kernel model request.
///
/// The turn runtime encodes the transcript as role-prefixed text items
/// (`system: …`, `user: …`, `assistant: …`); those items are mapped back to
/// typed OpenAI messages. Unprefixed items (legacy single-prompt calls) fall
/// back to one `user` message carrying the whole prompt.
fn build_chat_completion_request(request: &ModelRequest) -> OpenAiChatCompletionRequest {
    OpenAiChatCompletionRequest {
        model: request
            .model_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL_KEY.to_string()),
        messages: chat_messages_from_model_request(request),
        stream: Some(false),
        ..Default::default()
    }
}

fn chat_messages_from_model_request(request: &ModelRequest) -> Vec<OpenAiChatMessage> {
    let items = &request.messages;
    if items.is_empty() {
        return Vec::new();
    }
    let mut messages: Vec<OpenAiChatMessage> = Vec::with_capacity(items.len());
    for item in items {
        match split_role_item(item) {
            Some((role, content)) => messages.push(OpenAiChatMessage {
                role: role.to_string(),
                content: Some(content.trim().to_string()),
                ..Default::default()
            }),
            None => {
                // Legacy/unprefixed prompt: one user message with the raw text.
                return vec![OpenAiChatMessage {
                    role: "user".to_string(),
                    content: Some(items.join("\n")),
                    ..Default::default()
                }];
            }
        }
    }
    messages.retain(|message| {
        message
            .content
            .as_deref()
            .is_some_and(|content| !content.trim().is_empty())
    });
    messages
}

fn split_role_item(item: &str) -> Option<(&'static str, &str)> {
    for role in ["system", "user", "assistant"] {
        if let Some(content) = item.strip_prefix(&format!("{role}: ")) {
            return Some((role, content));
        }
    }
    None
}

/// Maps a cloudrouter SDK failure to a kernel provider error with an
/// actionable hint for the common account-pool routing failures.
pub fn map_cloudrouter_kernel_error(error: cloudrouter_open_sdk::SdkworkError) -> KernelError {
    use cloudrouter_open_sdk::SdkworkError;
    let hint = match &error {
        SdkworkError::HttpStatus { status, body }
            if *status == 404 && body.contains("model_not_found") =>
        {
            "; 所选模型在账号池路由中不可用：请在 Cloud Router 中为该供应商配置模型映射规则（ai_model_mapping_rule）或供应商支持模型"
        }
        SdkworkError::HttpStatus { status, body }
            if *status == 401 && body.contains("invalid_auth_token") =>
        {
            "; 登录 auth token 无效或已过期，请重新登录后重试"
        }
        SdkworkError::HttpStatus { status, body }
            if *status == 401 && body.contains("missing api key credential") =>
        {
            "; Cloud Router 未收到调用凭据：请检查 Agents 部署的 cloudrouter base URL 与 SDK 版本（请求必须同时携带 Authorization 与 Access-Token）"
        }
        SdkworkError::HttpStatus { status, body }
            if *status == 401 && body.contains("account_group_unavailable") =>
        {
            "; 当前租户在账号池中未配置默认分组（Default）或分组下无可用账号"
        }
        SdkworkError::HttpStatus { status, body } if *status >= 500 => {
            crate::cloudrouter_http_error_hint(*status, body)
        }
        _ => "",
    };
    KernelError::provider_error(
        "rig_cloudrouter_chat_completion_failed",
        format!("cloud router chat completion failed: {error}{hint}"),
    )
}

/// Parses a serialized OpenAI `tool_calls` array fragment into per-call
/// argument deltas, deriving a stable `tool_call_id` from the provider `id`
/// when present (falling back to `tool_call_{index}` for later deltas that
/// omit the transient id). Emits only non-empty argument fragments so the
/// kernel stream carries actual JSON argument deltas, not name-only notices.
fn extract_tool_call_argument_fragments(serialized: &str) -> Vec<(String, String)> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(serialized) else {
        return Vec::new();
    };
    let mut fragments = Vec::new();
    for item in items {
        let index = item.get("index").and_then(Value::as_u64).unwrap_or(0);
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("tool_call_{index}"));
        let arguments = item
            .get("function")
            .and_then(|function| function.get("arguments"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if !arguments.is_empty() {
            fragments.push((id, arguments));
        }
    }
    fragments
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::{KernelErrorKind, ProviderHealth, ProviderManifest};
    use sdkwork_agent_provider_rig::RigBackendMode;

    fn test_host() -> Arc<TestSecretHost> {
        Arc::new(TestSecretHost)
    }

    struct TestSecretHost;

    impl HostProvider for TestSecretHost {
        fn provider_manifest(&self) -> ProviderManifest {
            ProviderManifest::new(
                "provider.test-host",
                "test",
                "Test secret host",
                "0.1.0",
                Vec::new(),
            )
        }

        fn health(&self) -> ProviderHealth {
            ProviderHealth::available()
        }

        fn filesystem(
            &self,
            _request: sdkwork_agent_kernel::FilesystemRequest,
        ) -> KernelResult<sdkwork_agent_kernel::FilesystemResult> {
            Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: "filesystem".to_string(),
            })
        }

        fn process(
            &self,
            _request: sdkwork_agent_kernel::ProcessRequest,
        ) -> KernelResult<sdkwork_agent_kernel::ProcessResult> {
            Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: "process".to_string(),
            })
        }

        fn network(
            &self,
            _request: sdkwork_agent_kernel::NetworkRequest,
        ) -> KernelResult<sdkwork_agent_kernel::NetworkResult> {
            Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: "network".to_string(),
            })
        }

        fn resolve_secret(
            &self,
            secret_ref: sdkwork_agent_kernel::SecretRef,
        ) -> KernelResult<sdkwork_agent_kernel::ProviderSecretValue> {
            if secret_ref.secret_ref_id == "secret://rig/cloudrouter" {
                Ok(sdkwork_agent_kernel::ProviderSecretValue::new(
                    secret_ref.secret_ref_id,
                    "sk-test-key",
                ))
            } else {
                Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                    capability_id: secret_ref.secret_ref_id,
                })
            }
        }
    }

    fn live_config(api_key: Option<&str>) -> RigBackendConfig {
        RigBackendConfig {
            mode: RigBackendMode::Live,
            provider_id: None,
            api_key_secret_ref: api_key.map(str::to_string),
            base_url: None,
        }
    }

    #[test]
    fn chat_messages_parse_role_prefixed_items() {
        let request = ModelRequest::new(
            "request-1",
            vec![
                "system: You are helpful".to_string(),
                "user: hello".to_string(),
                "assistant: hi".to_string(),
            ],
        );
        let messages = chat_messages_from_model_request(&request);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content.as_deref(), Some("You are helpful"));
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[2].role, "assistant");
    }

    #[test]
    fn chat_messages_fall_back_to_single_user_message_for_unprefixed_items() {
        let request =
            ModelRequest::new("request-1", vec!["plain prompt".to_string(), "more".to_string()]);
        let messages = chat_messages_from_model_request(&request);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content.as_deref(), Some("plain prompt\nmore"));
    }

    #[test]
    fn chat_messages_drop_empty_items() {
        let request = ModelRequest::new(
            "request-1",
            vec!["system: ".to_string(), "user: hello".to_string()],
        );
        let messages = chat_messages_from_model_request(&request);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
    }

    #[test]
    fn chat_messages_return_empty_for_empty_request() {
        let request = ModelRequest::new("request-1", Vec::new());
        assert!(chat_messages_from_model_request(&request).is_empty());
    }

    #[test]
    fn completion_request_uses_model_id_or_default_key() {
        let request = ModelRequest::new("request-1", vec!["user: hi".to_string()])
            .with_model_id("openai-default");
        let completion = build_chat_completion_request(&request);
        assert_eq!(completion.model, "openai-default");
        assert_eq!(completion.stream, Some(false));

        let request = ModelRequest::new("request-2", vec!["user: hi".to_string()]);
        let completion = build_chat_completion_request(&request);
        assert_eq!(completion.model, DEFAULT_MODEL_KEY);
    }

    #[test]
    fn credentials_prefer_caller_dual_tokens_over_api_key() {
        let config = live_config(Some("secret://rig/cloudrouter"));
        let executor = RigCloudRouterExecutor::with_base_url(
            config,
            test_host(),
            "http://127.0.0.1:0",
        );
        let request = ModelRequest::new("request-1", vec!["user: hi".to_string()])
            .for_caller(
                Some("caller-auth-token".to_string()),
                Some("caller-access-token".to_string()),
            );

        // Dual tokens are accepted without touching the secret host; the call
        // itself fails against the dead base URL, not on credentials.
        let error = executor.invoke_model(request).expect_err("gateway call must fail");
        assert_ne!(
            error.kind(),
            KernelErrorKind::ProviderUnavailable,
            "credential decision must be accepted"
        );
        assert!(
            error.to_string().contains("chat completion failed"),
            "expected the gateway call to proceed past credentials, got: {error}"
        );
    }

    #[test]
    fn credentials_fail_closed_without_tokens_or_api_key() {
        let executor =
            RigCloudRouterExecutor::with_base_url(live_config(None), test_host(), "http://127.0.0.1:0");
        let request = ModelRequest::new("request-1", vec!["user: hi".to_string()]);
        let error = executor.invoke_model(request).expect_err("must fail closed");
        assert_eq!(error.kind(), KernelErrorKind::ProviderError);
        assert!(
            error.to_string().contains("requires the caller auth token"),
            "expected an actionable credential hint, got: {error}"
        );
    }

    #[test]
    fn cloudrouter_error_mapping_keeps_actionable_hints() {
        let error = map_cloudrouter_kernel_error(cloudrouter_open_sdk::SdkworkError::HttpStatus {
            status: 401,
            body: r#"{"error":{"code":"invalid_auth_token","message":"invalid"}}"#.to_string(),
        });
        assert!(error.to_string().contains("重新登录"));

        let error = map_cloudrouter_kernel_error(cloudrouter_open_sdk::SdkworkError::HttpStatus {
            status: 503,
            body: "unavailable".to_string(),
        });
        assert!(error.to_string().contains("暂不可用"));
    }
}
