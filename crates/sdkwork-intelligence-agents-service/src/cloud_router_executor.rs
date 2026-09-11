//! Cloud Router account-pool routing turn executor.
//!
//! Executes durable turns through the sdkwork-cloudrouter open-api chat
//! completions gateway, authenticating with the caller's auth token. The
//! cloudrouter account-pool routing pipeline (Default group → accounts →
//! upstream suppliers) selects the supplier for the requested model — no
//! local provider binding or API key configuration is required.

use std::sync::{Arc, OnceLock};

use cloudrouter_open_sdk::models::{OpenAiChatCompletionRequest, OpenAiChatMessage};
use cloudrouter_open_sdk::SdkworkAiClient;
use sdkwork_agent_kernel::KernelError;

use crate::domain::AgentSessionItemKind;
use crate::runtime_facade_bridge::engine_key_for_binding_id;
use crate::turn_runtime::{
    TurnExecutionInput, TurnExecutionOutput, TurnExecutor, TurnExecutionStreamSink,
};

/// Runtime mode label recorded on turns executed through the cloudrouter gateway.
pub const RUNTIME_MODE_CLOUDROUTER: &str = "cloudrouter-account-pool";

/// Environment variable for the cloudrouter gateway base URL.
pub const ENV_CLOUDROUTER_BASE_URL: &str = "SDKWORK_AGENTS_CLOUDROUTER_BASE_URL";

/// Fallback model key sent when the turn carries no model id.
const DEFAULT_MODEL_KEY: &str = "default";

fn cloudrouter_base_url() -> String {
    // Shared resolver: env override -> the gateway's own ingress bind (the
    // federated topology hosts this surface inside the cloudrouter gateway,
    // whose port varies per deployment profile) -> the platform proxy default.
    sdkwork_agents_tool_cloudrouter::cloudrouter_base_url()
}

fn blocking_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("cloud router executor tokio runtime")
    })
}

/// Turn executor that routes non-rig chat turns through the cloudrouter
/// gateway using the caller's auth token (account-pool routing, no API key
/// required).
///
/// Rig-bound sessions are delegated to the injected local executor: the RIG
/// agent engine's default model provider routes through the cloud router SDK
/// itself with the caller's dual tokens. Other engines (or unbound sessions)
/// carry the auth token and fall back to the injected local executor when the
/// turn carries none (e.g. worker/backend flows), keeping the durable turn
/// pipeline uniform for every path.
#[derive(Debug, Clone, Copy, Default)]
pub struct CloudRouterFirstTurnExecutor<T> {
    fallback: T,
}

impl<T> CloudRouterFirstTurnExecutor<T> {
    pub fn new(fallback: T) -> Self {
        Self { fallback }
    }
}

/// Returns `true` when the turn must be routed directly through the
/// cloudrouter gateway instead of the agent engine host.
///
/// Rig-bound sessions execute inside the RIG agent engine, whose default
/// model provider (`RigCloudRouterExecutor`) already routes every model call
/// through the cloud router SDK with the caller's dual tokens — intercepting
/// them here would bypass the engine. Only engines without that capability
/// (or unbound sessions) keep the direct account-pool shortcut.
fn should_route_through_cloud_router(input: &TurnExecutionInput) -> bool {
    let has_auth_token = input
        .auth_token
        .as_deref()
        .is_some_and(|token| !token.trim().is_empty());
    if !has_auth_token {
        return false;
    }
    let binding_id = input.binding_id.as_deref().unwrap_or("");
    engine_key_for_binding_id(binding_id) != Some("rig")
}

impl<T> TurnExecutor for CloudRouterFirstTurnExecutor<T>
where
    T: TurnExecutor,
{
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        if should_route_through_cloud_router(input) {
            complete_cloud_router_turn(input)
        } else {
            self.fallback.complete(input)
        }
    }

    fn cancel(
        &self,
        input: &crate::turn_runtime::TurnCancellationInput,
    ) -> sdkwork_agent_kernel::KernelResult<crate::turn_runtime::TurnCancellationOutput> {
        self.fallback.cancel(input)
    }

    fn complete_with_stream_preference(
        &self,
        input: &TurnExecutionInput,
        prefer_stream: bool,
    ) -> TurnExecutionOutput {
        if should_route_through_cloud_router(input) {
            complete_cloud_router_turn(input)
        } else {
            self.fallback.complete_with_stream_preference(input, prefer_stream)
        }
    }

    fn complete_with_stream_sink(
        &self,
        input: &TurnExecutionInput,
        sink: Arc<dyn TurnExecutionStreamSink>,
    ) -> TurnExecutionOutput {
        if should_route_through_cloud_router(input) {
            complete_cloud_router_turn(input)
        } else {
            self.fallback.complete_with_stream_sink(input, sink)
        }
    }
}

fn complete_cloud_router_turn(input: &TurnExecutionInput) -> TurnExecutionOutput {
    match execute_cloud_router_turn(input) {
        Ok(output) => output,
        Err(error) => {
            tracing::warn!(
                session_id = %input.session.session_id,
                turn_id = %input.turn_id,
                error = %error,
                "cloud router turn execution failed"
            );
            inference_error_output(format!("cloud router turn failed: {error}"))
        }
    }
}

fn execute_cloud_router_turn(
    input: &TurnExecutionInput,
) -> Result<TurnExecutionOutput, KernelError> {
    let auth_token = input
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| {
            KernelError::validation("cloud router execution requires an auth token")
        })?;

    let request = build_chat_completion_request(input);
    let client = SdkworkAiClient::new_with_base_url(cloudrouter_base_url())
        .map_err(|error| {
            KernelError::provider_error("cloudrouter_client_unavailable", error.to_string())
        })?;
    // Dual-token access per API_SPEC §819/§824: the gateway resolves the
    // account route context from the auth token and carries the access token
    // as the session access context. `set_access_token` runs first so the
    // `Authorization` bearer set by `set_auth_token` below is never dropped
    // by SDK header hygiene, keeping both tokens on the wire.
    if let Some(access_token) = input.access_token.as_deref().filter(|t| !t.trim().is_empty()) {
        client.set_access_token(access_token);
    }
    client.set_auth_token(auth_token);

    let completion = blocking_runtime()
        .block_on(client.chat().create(&request))
        .map_err(cloud_router_error)?;

    let choice = completion.choices.first().ok_or_else(|| {
        KernelError::provider_error(
            "cloudrouter_empty_response",
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
                "cloudrouter_empty_response",
                "cloud router returned no assistant content",
            )
        })?;

    let output_tokens = estimate_tokens(&content);
    Ok(TurnExecutionOutput {
        model_request_id: Some(input.model_request_id.clone()),
        finish_reason: choice
            .finish_reason
            .clone()
            .or_else(|| Some("stop".to_string())),
        content,
        model_id: Some(completion.model.clone()),
        provider_id: None,
        provider_session_id: None,
        input_tokens: estimate_tokens(&input.user_content),
        output_tokens,
        runtime_mode: RUNTIME_MODE_CLOUDROUTER,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    })
}

/// Maps the durable turn history into OpenAI chat messages: the agent system
/// prompt and welcome message lead as `system` messages (mirroring
/// `build_model_items` on the agent-engine path so both turn paths honor the
/// same agent personality), followed by the `user`/`assistant` history and the
/// current user content.
fn build_chat_completion_request(input: &TurnExecutionInput) -> OpenAiChatCompletionRequest {
    let mut messages: Vec<OpenAiChatMessage> = Vec::with_capacity(input.history.len() + 3);
    for (label, content) in [
        ("system", input.system_prompt.as_deref()),
        ("system", input.welcome_message.as_deref()),
    ] {
        let Some(content) = content.map(str::trim).filter(|value| !value.is_empty()) else {
            continue;
        };
        messages.push(OpenAiChatMessage {
            content: Some(content.to_string()),
            role: label.to_string(),
            ..Default::default()
        });
    }
    for (kind, content) in &input.history {
        let role = match kind {
            AgentSessionItemKind::UserInput => "user",
            AgentSessionItemKind::AssistantOutput => "assistant",
            _ => continue,
        };
        let content = content.trim();
        if content.is_empty() {
            continue;
        }
        messages.push(OpenAiChatMessage {
            content: Some(content.to_string()),
            role: role.to_string(),
            ..Default::default()
        });
    }
    messages.push(OpenAiChatMessage {
        content: Some(input.user_content.trim().to_string()),
        role: "user".to_string(),
        ..Default::default()
    });
    OpenAiChatCompletionRequest {
        model: input
            .model_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL_KEY.to_string()),
        messages,
        stream: Some(false),
        ..Default::default()
    }
}

/// Maps a cloudrouter SDK failure to a kernel provider error with an
/// actionable hint for the common account-pool routing failures.
fn cloud_router_error(error: cloudrouter_open_sdk::SdkworkError) -> KernelError {
    use cloudrouter_open_sdk::SdkworkError;
    let hint = match &error {
        SdkworkError::HttpStatus { status, body } if *status == 404 && body.contains("model_not_found") => {
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
        SdkworkError::HttpStatus { status, .. } if *status >= 500 => {
            "; Cloud Router 账号池网关暂不可用，请稍后重试"
        }
        _ => "",
    };
    KernelError::provider_error(
        "cloudrouter_chat_completion_failed",
        format!("cloud router chat completion failed: {error}{hint}"),
    )
}

fn estimate_tokens(text: &str) -> u64 {
    (text.chars().count() / 4) as u64
}

fn inference_error_output(message: String) -> TurnExecutionOutput {
    TurnExecutionOutput {
        model_request_id: None,
        finish_reason: None,
        content: message,
        model_id: None,
        provider_id: None,
        provider_session_id: None,
        input_tokens: 0,
        output_tokens: 0,
        runtime_mode: crate::turn_runtime::RUNTIME_MODE_INFERENCE_ERROR,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AgentSessionEntrySurface, AgentSessionKind, AgentSessionRecord, AgentSessionStatus};

    fn sample_session() -> AgentSessionRecord {
        AgentSessionRecord {
            id: 1,
            session_id: "session.test".to_string(),
            tenant_id: 100001,
            organization_id: 0,
            agent_id: "agent.test".to_string(),
            owner_user_id: 42,
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Test".to_string()),
            title_source: crate::domain::AgentSessionTitleSource::System,
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: None,
            payload_hash: None,
            created_by: 42,
            updated_by: 42,
            version: 0,
            created_at: "2026-06-28T00:00:00Z".to_string(),
            updated_at: "2026-06-28T00:00:00Z".to_string(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        }
    }

    fn sample_input(auth_token: Option<&str>, access_token: Option<&str>) -> TurnExecutionInput {
        TurnExecutionInput {
            turn_id: "turn.test".to_string(),
            model_request_id: "model-request.test".to_string(),
            agent_display_name: "Test Agent".to_string(),
            welcome_message: None,
            session: sample_session(),
            history: vec![
                (AgentSessionItemKind::UserInput, "hello".to_string()),
                (AgentSessionItemKind::AssistantOutput, "hi there".to_string()),
                (AgentSessionItemKind::StatusNotice, "ignored".to_string()),
            ],
            user_content: "latest question".to_string(),
            model_id: Some("rig.default-chat".to_string()),
            provider_id: None,
            provider_session_id: None,
            access_mode_id: None,
            binding_id: None,
            provider_has_model_chat: true,
            system_prompt: None,
            auth_token: auth_token.map(str::to_string),
            access_token: access_token.map(str::to_string),
        }
    }

    #[test]
    fn builds_openai_messages_from_turn_history() {
        let input = sample_input(Some("token"), Some("access"));
        let request = build_chat_completion_request(&input);
        assert_eq!(request.model, "rig.default-chat");
        assert_eq!(request.stream, Some(false));
        let roles: Vec<&str> = request.messages.iter().map(|m| m.role.as_str()).collect();
        assert_eq!(roles, vec!["user", "assistant", "user"]);
        assert_eq!(
            request.messages.last().and_then(|m| m.content.as_deref()),
            Some("latest question")
        );
    }

    #[test]
    fn builds_openai_messages_prepend_system_prompt_and_welcome() {
        let mut input = sample_input(Some("token"), Some("access"));
        input.system_prompt = Some("You are a helpful agent".to_string());
        input.welcome_message = Some("Hi! How can I help?".to_string());
        let request = build_chat_completion_request(&input);
        let roles: Vec<&str> = request.messages.iter().map(|m| m.role.as_str()).collect();
        assert_eq!(roles, vec!["system", "system", "user", "assistant", "user"]);
        assert_eq!(
            request.messages[0].content.as_deref(),
            Some("You are a helpful agent")
        );
        assert_eq!(
            request.messages[1].content.as_deref(),
            Some("Hi! How can I help?")
        );
    }

    #[test]
    fn defaults_model_key_when_unset() {
        let mut input = sample_input(None, None);
        input.model_id = None;
        let request = build_chat_completion_request(&input);
        assert_eq!(request.model, "default");
    }

    #[test]
    fn executor_without_auth_token_delegates_to_fallback() {
        struct RecordingFallback(std::sync::Mutex<Vec<String>>);
        impl TurnExecutor for RecordingFallback {
            fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
                self.0.lock().unwrap().push(input.user_content.clone());
                inference_error_output("fallback".to_string())
            }
        }
        let fallback = RecordingFallback(std::sync::Mutex::new(Vec::new()));
        let executor = CloudRouterFirstTurnExecutor::new(fallback);

        // No auth token -> fallback path.
        let output = executor.complete(&sample_input(None, None));
        assert!(output.content.contains("fallback"));
        assert_eq!(executor.fallback.0.lock().unwrap().len(), 1);
    }

    #[test]
    fn cloud_router_error_adds_actionable_hints() {
        use cloudrouter_open_sdk::SdkworkError;

        let not_found = cloud_router_error(SdkworkError::HttpStatus {
            status: 404,
            body: r#"{"error":{"code":"model_not_found","message":"model is not available: rig.default-chat"}}"#.to_string(),
        });
        assert!(not_found.to_string().contains("模型映射规则"));

        let unauthorized = cloud_router_error(SdkworkError::HttpStatus {
            status: 401,
            body: r#"{"error":{"code":"invalid_auth_token","message":"invalid or expired auth token"}}"#.to_string(),
        });
        assert!(unauthorized.to_string().contains("auth token 无效或已过期"));

        let upstream = cloud_router_error(SdkworkError::HttpStatus {
            status: 502,
            body: "bad gateway".to_string(),
        });
        assert!(upstream.to_string().contains("暂不可用"));

        let serialization = cloud_router_error(SdkworkError::Serialization(
            serde_json::from_str::<serde_json::Value>("not-json").unwrap_err(),
        ));
        assert!(!serialization.to_string().contains("模型映射规则"));
    }
}

