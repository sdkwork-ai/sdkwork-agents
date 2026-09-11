//! Cloud Router account-pool routing turn executor.
//!
//! Executes durable turns through the sdkwork-cloudrouter open-api chat
//! completions gateway, authenticating with the caller's auth token. The
//! cloudrouter account-pool routing pipeline (Default group → accounts →
//! upstream suppliers) selects the supplier for the requested model — no
//! local provider binding or API key configuration is required.

use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use cloudrouter_open_sdk::models::{
    OpenAiChatCompletion, OpenAiChatCompletionRequest, OpenAiChatMessage,
};
use cloudrouter_open_sdk::SdkworkAiClient;
use futures_util::StreamExt;
use sdkwork_agent_kernel::{KernelError, KernelResult};

use crate::domain::AgentSessionItemKind;
use crate::runtime_facade_bridge::engine_key_for_binding_id;
use crate::turn_runtime::{
    is_inference_error, TurnExecutionInput, TurnExecutionOutput, TurnExecutor,
    TurnExecutionStreamSink,
};

/// Runtime mode label recorded on turns executed through the cloudrouter gateway.
pub const RUNTIME_MODE_CLOUDROUTER: &str = "cloudrouter-account-pool";

/// Environment variable for the cloudrouter gateway base URL.
pub const ENV_CLOUDROUTER_BASE_URL: &str = "SDKWORK_AGENTS_CLOUDROUTER_BASE_URL";

/// Fallback model key sent when the turn carries no model id.
const DEFAULT_MODEL_KEY: &str = "default";

/// Canonical open-api chat completions path on the cloudrouter gateway.
const CHAT_COMPLETIONS_PATH: &str = "/v1/chat/completions";

/// Upper bound for the provider error body echoed back into the kernel error.
const UPSTREAM_ERROR_BODY_MAX_CHARS: usize = 512;

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
            complete_cloud_router_turn_with_sink(input, sink)
        } else {
            self.fallback.complete_with_stream_sink(input, sink)
        }
    }
}

fn complete_cloud_router_turn(input: &TurnExecutionInput) -> TurnExecutionOutput {
    match execute_cloud_router_turn(input) {
        Ok(output) => output,
        Err(error) => cloud_router_failure_output(input, &error),
    }
}

fn cloud_router_failure_output(
    input: &TurnExecutionInput,
    error: &KernelError,
) -> TurnExecutionOutput {
    tracing::warn!(
        session_id = %input.session.session_id,
        turn_id = %input.turn_id,
        error = %error,
        "cloud router turn execution failed"
    );
    inference_error_output(format!("cloud router turn failed: {error}"))
}

fn require_auth_token(input: &TurnExecutionInput) -> Result<&str, KernelError> {
    input
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| {
            KernelError::validation("cloud router execution requires an auth token")
        })
}

fn execute_cloud_router_turn(
    input: &TurnExecutionInput,
) -> Result<TurnExecutionOutput, KernelError> {
    let auth_token = require_auth_token(input)?;

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

    turn_output_from_buffered_completion(input, &completion)
}

/// Maps one buffered chat completion onto the durable-turn output contract.
fn turn_output_from_buffered_completion(
    input: &TurnExecutionInput,
    completion: &OpenAiChatCompletion,
) -> Result<TurnExecutionOutput, KernelError> {
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

/// One streamed cloudrouter turn attempt.
struct CloudRouterStreamAttempt {
    result: Result<TurnExecutionOutput, KernelError>,
    /// True once at least one delta reached the sink. A partially delivered
    /// stream must never be followed by a buffered re-run: the client would
    /// render the assistant answer twice.
    delivered_deltas: bool,
}

/// One decoded upstream SSE frame.
enum CloudRouterStreamFrame {
    Chunk {
        content: Option<String>,
        model: Option<String>,
        finish_reason: Option<String>,
    },
    Done,
}

/// Delta handed from the async SSE reader to the blocking sink drainer thread.
#[derive(Debug)]
enum CloudRouterStreamMessage {
    Delta(String),
}

/// Executes one account-pool turn while forwarding provider deltas live.
///
/// The durable-turn sink is the only incremental delivery contract the HTTP
/// layer has, so a cloudrouter-routed turn must publish through it exactly like
/// an agent-engine turn does — otherwise `?stream=true` degrades into a single
/// terminal `completion` frame and delta-only clients render nothing.
fn complete_cloud_router_turn_with_sink(
    input: &TurnExecutionInput,
    sink: Arc<dyn TurnExecutionStreamSink>,
) -> TurnExecutionOutput {
    let attempt = execute_cloud_router_turn_streaming(input, Arc::clone(&sink));

    if attempt.delivered_deltas {
        return match attempt.result {
            Ok(output) => output,
            Err(error) => cloud_router_failure_output(input, &error),
        };
    }

    match attempt.result {
        Ok(output) => forward_buffered_answer(output, &sink),
        Err(error) => {
            // Streaming could not produce anything (the model is unmapped in the
            // account pool, the supplier rejected `stream: true`, or the stream
            // closed before the first chunk). Re-run the canonical buffered
            // operation so the caller keeps the established provider error
            // contract, and publish its answer as one delta.
            tracing::warn!(
                session_id = %input.session.session_id,
                turn_id = %input.turn_id,
                error = %error,
                "cloud router streaming turn fell back to the buffered operation"
            );
            forward_buffered_answer(complete_cloud_router_turn(input), &sink)
        }
    }
}

/// Publishes an already-complete answer as a single delta.
///
/// The sink contract is delta-shaped, so a runtime that answers in one shot
/// must still emit its text through `push_delta`; delta-only renderers would
/// otherwise show an empty assistant message until the session is refreshed.
fn forward_buffered_answer(
    output: TurnExecutionOutput,
    sink: &Arc<dyn TurnExecutionStreamSink>,
) -> TurnExecutionOutput {
    if output.content.trim().is_empty() || is_inference_error(output.runtime_mode) {
        return output;
    }
    sink.push_delta(&output.content);
    TurnExecutionOutput {
        stream_deltas: vec![output.content.clone()],
        ..output
    }
}

fn execute_cloud_router_turn_streaming(
    input: &TurnExecutionInput,
    sink: Arc<dyn TurnExecutionStreamSink>,
) -> CloudRouterStreamAttempt {
    let auth_token = match require_auth_token(input) {
        Ok(auth_token) => auth_token.to_owned(),
        Err(error) => {
            return CloudRouterStreamAttempt {
                result: Err(error),
                delivered_deltas: false,
            };
        }
    };

    let mut request = build_chat_completion_request(input);
    request.stream = Some(true);
    let url = format!(
        "{}{CHAT_COMPLETIONS_PATH}",
        cloudrouter_base_url().trim_end_matches('/')
    );

    // `TurnExecutionStreamSink::push_delta` is synchronous and forwards through
    // `mpsc::blocking_send`, which panics inside a tokio runtime context. The
    // deltas therefore cross an unbounded std channel into a dedicated drainer
    // thread that owns the sink and never runs inside a runtime.
    let (sender, receiver) = std_mpsc::channel::<CloudRouterStreamMessage>();
    let drainer = {
        let sink = Arc::clone(&sink);
        thread::spawn(move || {
            while let Ok(CloudRouterStreamMessage::Delta(delta)) = receiver.recv() {
                sink.push_delta(&delta);
            }
        })
    };

    // `blocking_runtime()` is a dedicated runtime instance, so `block_on` is
    // safe from the provider worker thread that drives this synchronous
    // executor trait method (same pattern as the buffered path above).
    let attempt = blocking_runtime().block_on(consume_chat_completion_stream(
        streaming_http_client(),
        &url,
        &auth_token,
        input,
        &request,
        sender,
    ));
    // The async consumer owns `sender`, so the drainer sees the disconnect as
    // soon as the future completes and every delivered delta is flushed before
    // the turn result is observed.
    let _ = drainer.join();
    attempt
}

/// Shared client for streamed provider calls.
///
/// Deliberately built without a total request timeout: a streaming turn is
/// bounded by `TURN_EXECUTION_TIMEOUT` at the caller, while the generated SDK
/// client applies a whole-request timeout that would abort mid-stream.
fn streaming_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("cloud router streaming http client")
    })
}

async fn consume_chat_completion_stream(
    client: &'static reqwest::Client,
    url: &str,
    auth_token: &str,
    input: &TurnExecutionInput,
    request: &OpenAiChatCompletionRequest,
    sender: std_mpsc::Sender<CloudRouterStreamMessage>,
) -> CloudRouterStreamAttempt {
    let mut builder = client
        .post(url)
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {auth_token}"),
        )
        .json(request);
    // Dual-token access per API_SPEC §819/§824: the gateway resolves the account
    // route context from the auth token and carries the access token as the
    // session access context.
    if let Some(access_token) = input
        .access_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    {
        builder = builder.header("Access-Token", access_token);
    }

    let response = match builder.send().await {
        Ok(response) => response,
        Err(error) => {
            return CloudRouterStreamAttempt {
                result: Err(KernelError::provider_error(
                    "cloudrouter_stream_unavailable",
                    format!("cloud router streaming request failed: {error}"),
                )),
                delivered_deltas: false,
            };
        }
    };

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return CloudRouterStreamAttempt {
            result: Err(cloud_router_status_error(status.as_u16(), &body)),
            delivered_deltas: false,
        };
    }

    if !is_event_stream_response(&response) {
        // The supplier answered with a buffered completion despite
        // `stream: true`; decode it so the caller can publish it as one delta.
        let body = response.text().await.unwrap_or_default();
        return CloudRouterStreamAttempt {
            result: decode_buffered_chat_completion(input, body.as_str()),
            delivered_deltas: false,
        };
    }

    let mut buffer = String::new();
    let mut content = String::new();
    let mut deltas: Vec<String> = Vec::new();
    let mut model_id: Option<String> = None;
    let mut finish_reason: Option<String> = None;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                return CloudRouterStreamAttempt {
                    result: Err(KernelError::provider_error(
                        "cloudrouter_stream_interrupted",
                        format!("cloud router stream interrupted: {error}"),
                    )),
                    delivered_deltas: !deltas.is_empty(),
                };
            }
        };
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(newline) = buffer.find('\n') {
            let line: String = buffer.drain(..=newline).collect();
            let line = line.trim_end_matches(|character| character == '\r' || character == '\n');
            match decode_chat_completion_stream_line(line) {
                Some(CloudRouterStreamFrame::Chunk {
                    content: delta,
                    model,
                    finish_reason: reason,
                }) => {
                    if let Some(model) = model {
                        model_id = Some(model);
                    }
                    if let Some(reason) = reason {
                        finish_reason = Some(reason);
                    }
                    if let Some(delta) = delta {
                        content.push_str(&delta);
                        deltas.push(delta.clone());
                        // A closed sink (client disconnected) must not abort the
                        // durable turn: persistence stays authoritative.
                        let _ = sender.send(CloudRouterStreamMessage::Delta(delta));
                    }
                }
                Some(CloudRouterStreamFrame::Done) | None => {}
            }
        }
    }

    if content.trim().is_empty() {
        return CloudRouterStreamAttempt {
            result: Err(KernelError::provider_error(
                "cloudrouter_empty_response",
                "cloud router streamed no assistant content",
            )),
            delivered_deltas: !deltas.is_empty(),
        };
    }

    let delivered_deltas = !deltas.is_empty();
    CloudRouterStreamAttempt {
        result: Ok(TurnExecutionOutput {
            model_request_id: Some(input.model_request_id.clone()),
            finish_reason: finish_reason.or_else(|| Some("stop".to_string())),
            content: content.clone(),
            model_id: model_id.or_else(|| Some(request.model.clone())),
            provider_id: None,
            provider_session_id: None,
            input_tokens: estimate_tokens(&input.user_content),
            output_tokens: estimate_tokens(&content),
            runtime_mode: RUNTIME_MODE_CLOUDROUTER,
            stream_deltas: deltas,
            stream_events: Vec::new(),
        }),
        delivered_deltas,
    }
}

fn is_event_stream_response(response: &reqwest::Response) -> bool {
    response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains("text/event-stream"))
        .unwrap_or(false)
}

fn decode_buffered_chat_completion(
    input: &TurnExecutionInput,
    body: &str,
) -> Result<TurnExecutionOutput, KernelError> {
    let completion: OpenAiChatCompletion = serde_json::from_str(body).map_err(|error| {
        KernelError::provider_error(
            "cloudrouter_invalid_response",
            format!("cloud router returned an undecodable completion: {error}"),
        )
    })?;
    turn_output_from_buffered_completion(input, &completion)
}

/// Decodes one upstream SSE line into a provider-neutral frame.
///
/// Comment lines, blank lines, `event:`/`id:` fields and payloads without
/// incremental content all collapse to `None`, so keep-alives from the gateway
/// never perturb delta accounting.
fn decode_chat_completion_stream_line(line: &str) -> Option<CloudRouterStreamFrame> {
    let line = line.trim();
    if line.is_empty() || line.starts_with(':') {
        return None;
    }
    let payload = line.strip_prefix("data:").map(str::trim).unwrap_or(line);
    if payload.is_empty() {
        return None;
    }
    if payload == "[DONE]" {
        return Some(CloudRouterStreamFrame::Done);
    }
    let value: serde_json::Value = serde_json::from_str(payload).ok()?;
    let choice = value
        .get("choices")
        .and_then(serde_json::Value::as_array)
        .and_then(|choices| choices.first());
    let content = choice
        .and_then(|choice| choice.get("delta"))
        .and_then(|delta| delta.get("content"))
        .or_else(|| {
            choice
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
        })
        .and_then(serde_json::Value::as_str)
        .filter(|content| !content.is_empty())
        .map(str::to_owned);
    Some(CloudRouterStreamFrame::Chunk {
        content,
        model: value
            .get("model")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        finish_reason: choice
            .and_then(|choice| choice.get("finish_reason"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
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
/// Actionable hint appended to account-pool gateway failures.
///
/// Shared by the generated-SDK buffered path and the hand-rolled streaming path
/// so both surface the same operator guidance for a given status/body pair.
fn cloud_router_status_hint(status: u16, body: &str) -> &'static str {
    match status {
        404 if body.contains("model_not_found") => {
            "; 所选模型在账号池路由中不可用：请在 Cloud Router 中为该供应商配置模型映射规则（ai_model_mapping_rule）或供应商支持模型"
        }
        401 if body.contains("invalid_auth_token") => {
            "; 登录 auth token 无效或已过期，请重新登录后重试"
        }
        401 if body.contains("missing api key credential") => {
            "; Cloud Router 未收到调用凭据：请检查 Agents 部署的 cloudrouter base URL 与 SDK 版本（请求必须同时携带 Authorization 与 Access-Token）"
        }
        401 if body.contains("account_group_unavailable") => {
            "; 当前租户在账号池中未配置默认分组（Default）或分组下无可用账号"
        }
        status if status >= 500 => "; Cloud Router 账号池网关暂不可用，请稍后重试",
        _ => "",
    }
}

fn cloud_router_error(error: cloudrouter_open_sdk::SdkworkError) -> KernelError {
    use cloudrouter_open_sdk::SdkworkError;
    let hint = match &error {
        SdkworkError::HttpStatus { status, body } => cloud_router_status_hint(*status, body),
        _ => "",
    };
    KernelError::provider_error(
        "cloudrouter_chat_completion_failed",
        format!("cloud router chat completion failed: {error}{hint}"),
    )
}

/// Maps a raw gateway HTTP failure from the streaming transport.
fn cloud_router_status_error(status: u16, body: &str) -> KernelError {
    let hint = cloud_router_status_hint(status, body);
    let detail: String = body.chars().take(UPSTREAM_ERROR_BODY_MAX_CHARS).collect();
    KernelError::provider_error(
        "cloudrouter_chat_completion_failed",
        format!("cloud router chat completion failed: HTTP {status} {detail}{hint}"),
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

    /// Records every delta the runtime publishes through the sink.
    #[derive(Default)]
    struct RecordingStreamSink {
        deltas: std::sync::Mutex<Vec<String>>,
    }

    impl RecordingStreamSink {
        fn deltas(&self) -> Vec<String> {
            self.deltas.lock().expect("delta lock").clone()
        }
    }

    impl TurnExecutionStreamSink for RecordingStreamSink {
        fn push_delta(&self, delta: &str) {
            self.deltas
                .lock()
                .expect("delta lock")
                .push(delta.to_string());
        }

        fn push_event(&self, _event: &sdkwork_agent_kernel::KernelEvent) -> KernelResult<()> {
            Ok(())
        }
    }

    fn cloudrouter_output(content: &str, runtime_mode: &'static str) -> TurnExecutionOutput {
        TurnExecutionOutput {
            model_request_id: None,
            finish_reason: Some("stop".to_string()),
            content: content.to_string(),
            model_id: Some("default".to_string()),
            provider_id: None,
            provider_session_id: None,
            input_tokens: 1,
            output_tokens: 1,
            runtime_mode,
            stream_deltas: Vec::new(),
            stream_events: Vec::new(),
        }
    }

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

    #[test]
    fn streaming_line_decoder_extracts_incremental_content() {
        let frame = decode_chat_completion_stream_line(
            r#"data: {"model":"deepseek-chat","choices":[{"delta":{"content":"你"},"finish_reason":null}]}"#,
        );
        match frame {
            Some(CloudRouterStreamFrame::Chunk {
                content,
                model,
                finish_reason,
            }) => {
                assert_eq!(Some("你".to_string()), content);
                assert_eq!(Some("deepseek-chat".to_string()), model);
                assert_eq!(None, finish_reason);
            }
            _ => panic!("expected an incremental chunk"),
        }

        // A `message.content` payload is also honoured: suppliers that answer
        // with a complete completion despite `stream: true` stay decodable.
        let buffered_style = decode_chat_completion_stream_line(
            r#"data: {"choices":[{"message":{"content":"done"},"finish_reason":"stop"}]}"#,
        );
        match buffered_style {
            Some(CloudRouterStreamFrame::Chunk {
                content,
                finish_reason,
                ..
            }) => {
                assert_eq!(Some("done".to_string()), content);
                assert_eq!(Some("stop".to_string()), finish_reason);
            }
            _ => panic!("expected a buffered-style chunk"),
        }
    }

    #[test]
    fn streaming_line_decoder_ignores_framing_without_content() {
        // SSE comments (the gateway keep-alive), blank lines, other SSE fields
        // and the terminal sentinel must never be mistaken for delta text.
        assert!(decode_chat_completion_stream_line(": keep-alive").is_none());
        assert!(decode_chat_completion_stream_line("").is_none());
        assert!(decode_chat_completion_stream_line("event: completion").is_none());
        assert!(decode_chat_completion_stream_line("id: 42").is_none());
        assert!(decode_chat_completion_stream_line(
            r#"data: {"choices":[],"usage":{"total_tokens":9}}"#
        )
        .is_some_and(|frame| matches!(
            frame,
            CloudRouterStreamFrame::Chunk { content: None, .. }
        )));
        assert!(matches!(
            decode_chat_completion_stream_line("data: [DONE]"),
            Some(CloudRouterStreamFrame::Done)
        ));
        // A supplier that opens the message with an empty role-only delta.
        assert!(decode_chat_completion_stream_line(
            r#"data: {"choices":[{"delta":{"role":"assistant","content":""}}]}"#
        )
        .is_some_and(|frame| matches!(
            frame,
            CloudRouterStreamFrame::Chunk { content: None, .. }
        )));
    }

    #[test]
    fn buffered_answer_is_published_as_a_single_delta() {
        let sink = Arc::new(RecordingStreamSink::default());
        let stream_sink: Arc<dyn TurnExecutionStreamSink> = sink.clone();
        let output = forward_buffered_answer(
            cloudrouter_output("完整回答", RUNTIME_MODE_CLOUDROUTER),
            &stream_sink,
        );

        // Delta-only renderers read `push_delta`; a single-shot runtime must
        // still publish its text or the assistant bubble stays empty.
        assert_eq!(vec!["完整回答".to_string()], sink.deltas());
        assert_eq!(vec!["完整回答".to_string()], output.stream_deltas);
    }

    #[test]
    fn inference_failures_are_not_published_as_assistant_text() {
        let sink = Arc::new(RecordingStreamSink::default());
        let stream_sink: Arc<dyn TurnExecutionStreamSink> = sink.clone();
        let output = forward_buffered_answer(
            cloudrouter_output(
                "cloud router turn failed: upstream unavailable",
                crate::turn_runtime::RUNTIME_MODE_INFERENCE_ERROR,
            ),
            &stream_sink,
        );

        // The error text stays in the terminal completion envelope; forwarding
        // it as a delta would render a provider failure as the answer.
        assert!(sink.deltas().is_empty());
        assert!(output.stream_deltas.is_empty());
    }

    #[test]
    fn streaming_status_errors_reuse_the_actionable_hints() {
        let not_found = cloud_router_status_error(
            404,
            r#"{"error":{"code":"model_not_found","message":"model is not available"}}"#,
        );
        assert!(not_found.to_string().contains("模型映射规则"));

        let unauthorized = cloud_router_status_error(
            401,
            r#"{"error":{"code":"account_group_unavailable"}}"#,
        );
        assert!(unauthorized.to_string().contains("未配置默认分组"));

        let upstream = cloud_router_status_error(503, "service unavailable");
        assert!(upstream.to_string().contains("暂不可用"));
        assert!(upstream.to_string().contains("HTTP 503"));
    }

    /// `SDKWORK_AGENTS_CLOUDROUTER_BASE_URL` is process-global, so the end-to-end
    /// streaming test must not race other tests that resolve the same base URL.
    static CLOUDROUTER_BASE_URL_TEST_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> =
        std::sync::OnceLock::new();

    fn cloudrouter_base_url_test_guard() -> std::sync::MutexGuard<'static, ()> {
        CLOUDROUTER_BASE_URL_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .expect("cloudrouter base url test lock")
    }

    /// Restores an environment variable when the test scope ends, including on
    /// panic, so a failing case cannot leak the stub URL into other tests.
    struct EnvVarRestore {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarRestore {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarRestore {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    /// Stub `/v1/chat/completions` upstream that emits one delta, then holds the
    /// response open until the test releases it.
    ///
    /// The hold is the whole point: the assertion below can only pass if the
    /// first delta reached the sink while the upstream stream was still being
    /// written.
    struct StubStreamingUpstream {
        base_url: String,
        /// Latched rather than a `Notify`: the stub also serves the reachability
        /// probe, and a one-shot notification would be consumed by whichever
        /// request happened to be waiting first.
        released: Arc<std::sync::atomic::AtomicBool>,
        shutdown: Arc<tokio::sync::Notify>,
        server: Option<std::thread::JoinHandle<()>>,
    }

    impl StubStreamingUpstream {
        fn start() -> Self {
            let listener =
                std::net::TcpListener::bind("127.0.0.1:0").expect("stub upstream listener");
            let addr = listener.local_addr().expect("stub upstream addr");
            // `tokio::net::TcpListener::from_std` requires a non-blocking socket;
            // without this the adopting thread panics and the port still accepts
            // connections into the kernel backlog without ever answering them.
            listener
                .set_nonblocking(true)
                .expect("stub upstream nonblocking");
            let released = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let shutdown = Arc::new(tokio::sync::Notify::new());

            let handler_release = Arc::clone(&released);
            let server_shutdown = Arc::clone(&shutdown);
            let server = std::thread::Builder::new()
                .name("stub-cloudrouter-upstream".to_string())
                .spawn(move || {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .worker_threads(2)
                    .build()
                    .expect("stub upstream runtime");
                runtime.block_on(async move {
                    let listener = tokio::net::TcpListener::from_std(listener)
                        .expect("adopt stub upstream listener");
                    let app = axum::Router::new()
                        .route(
                            "/v1/chat/completions",
                            axum::routing::post(stub_chat_completions),
                        )
                        .with_state(handler_release);
                    axum::serve(listener, app)
                        .with_graceful_shutdown(async move { server_shutdown.notified().await })
                        .await
                        .expect("stub upstream serve");
                });
            })
            .expect("spawn stub upstream thread");

            Self {
                base_url: format!("http://{addr}"),
                released,
                shutdown,
                server: Some(server),
            }
        }

        /// Lets the stub emit its remaining frames and finish the response.
        fn release(&self) {
            self.released
                .store(true, std::sync::atomic::Ordering::Release);
        }

        /// Blocks until the stub answers, so a broken fixture fails with an
        /// explicit message instead of an opaque timeout later on.
        ///
        /// A silent timeout here almost always means the `stub-cloudrouter-upstream`
        /// thread panicked: the bound port still completes TCP handshakes from
        /// the kernel backlog even when nothing is accepting them.
        fn assert_reachable(&self, what: &str) {
            use std::io::{Read, Write};

            let mut stream = std::net::TcpStream::connect(
                self.base_url.trim_start_matches("http://"),
            )
            .unwrap_or_else(|error| panic!("{what}: stub upstream is not reachable: {error}"));
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("probe read timeout");
            stream
                .write_all(
                    b"POST /v1/chat/completions HTTP/1.1\r\nHost: stub\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
                )
                .unwrap_or_else(|error| panic!("{what}: probe write failed: {error}"));
            let mut buffer = [0u8; 256];
            let read = stream
                .read(&mut buffer)
                .unwrap_or_else(|error| panic!("{what}: probe read failed: {error}"));
            let head = String::from_utf8_lossy(&buffer[..read]).to_string();
            assert!(
                head.starts_with("HTTP/1.1 200") && head.contains("text/event-stream"),
                "{what}: unexpected stub response head: {head:?}"
            );
        }
    }

    impl Drop for StubStreamingUpstream {
        fn drop(&mut self) {
            self.shutdown.notify_one();
            if let Some(server) = self.server.take() {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                while !server.is_finished() && std::time::Instant::now() < deadline {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                if server.is_finished() {
                    let _ = server.join();
                }
                // Otherwise let the stub thread be reaped at process exit rather
                // than panicking inside `Drop`.
            }
        }
    }

    async fn stub_chat_completions(
        axum::extract::State(released): axum::extract::State<
            Arc<std::sync::atomic::AtomicBool>,
        >,
    ) -> axum::response::Response {
        let (sender, receiver) = tokio::sync::mpsc::channel::<Result<axum::body::Bytes, std::io::Error>>(8);
        tokio::spawn(async move {
            let frame = |content: &str| {
                Ok(axum::body::Bytes::from(format!(
                    "data: {{\"model\":\"stub.model\",\"choices\":[{{\"delta\":{{\"content\":\"{content}\"}}}}]}}\n\n"
                )))
            };
            if sender.send(frame("Hello")).await.is_err() {
                return;
            }
            // Hold the SSE response open until the test has observed `Hello`.
            while !released.load(std::sync::atomic::Ordering::Acquire) {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            if sender.send(frame(" world")).await.is_err() {
                return;
            }
            let _ = sender
                .send(Ok(axum::body::Bytes::from_static(
                    b"data: [DONE]\n\n",
                )))
                .await;
        });

        let body_stream = futures_util::stream::unfold(receiver, |mut receiver| async move {
            receiver.recv().await.map(|item| (item, receiver))
        });
        axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "text/event-stream")
            .header("X-Accel-Buffering", "no")
            .body(axum::body::Body::from_stream(body_stream))
            .expect("stub SSE response")
    }

    /// Waits for a worker thread without ever blocking indefinitely, so a
    /// wiring regression surfaces as a bounded failure instead of a hung suite.
    fn join_bounded<T: Send + 'static>(
        handle: std::thread::JoinHandle<T>,
        timeout: std::time::Duration,
        what: &str,
    ) -> T {
        let deadline = std::time::Instant::now() + timeout;
        while !handle.is_finished() {
            assert!(
                std::time::Instant::now() < deadline,
                "{what} did not finish within {timeout:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        match handle.join() {
            Ok(value) => value,
            Err(_) => panic!("{what} panicked"),
        }
    }

    /// Regression guard for the Playground defect: `complete_with_stream_sink`
    /// used to route account-pool turns into the *buffered* operation and throw
    /// the sink away, so `?stream=true` produced a single terminal `completion`
    /// frame and delta-only clients rendered nothing until a page reload.
    ///
    /// This drives the real HTTP path against a stub gateway and asserts the
    /// first delta is published **while the upstream stream is still open**.
    #[test]
    fn streams_account_pool_deltas_through_the_sink_before_upstream_completes() {
        let _guard = cloudrouter_base_url_test_guard();
        let upstream = StubStreamingUpstream::start();
        upstream.assert_reachable("before the streaming turn");
        // Resolve the name from the owning crate so the stub can never drift
        // away from the variable the executor actually reads.
        let _env = EnvVarRestore::set(
            sdkwork_agents_tool_cloudrouter::ENV_CLOUDROUTER_BASE_URL,
            &upstream.base_url,
        );

        let sink = Arc::new(RecordingStreamSink::default());
        let input = sample_input(Some("auth-token"), Some("access-token"));
        assert!(
            should_route_through_cloud_router(&input),
            "the fixture must exercise the account-pool path"
        );

        let worker = {
            let sink = Arc::clone(&sink);
            let input = input.clone();
            std::thread::spawn(move || {
                let stream_sink: Arc<dyn TurnExecutionStreamSink> = sink;
                let executor = CloudRouterFirstTurnExecutor::new(UnreachableFallback);
                executor.complete_with_stream_sink(&input, stream_sink)
            })
        };

        // Wait for the first delta while the upstream is deliberately held open.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while sink.deltas().is_empty()
            && !worker.is_finished()
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if sink.deltas().is_empty() {
            // Report the turn's own outcome so a wiring regression is
            // diagnosable instead of a bare "got no deltas". Release first so a
            // worker still parked on the held-up stream can finish.
            upstream.release();
            let output =
                join_bounded(worker, std::time::Duration::from_secs(10), "streaming turn worker");
            panic!(
                "no delta reached the sink while the upstream stream was open: \
                 content={:?} runtime_mode={:?} stream_deltas={:?}",
                output.content, output.runtime_mode, output.stream_deltas
            );
        }
        assert_eq!(
            vec!["Hello".to_string()],
            sink.deltas(),
            "the first delta must reach the sink before the upstream stream completes"
        );

        upstream.release();
        let output =
            join_bounded(worker, std::time::Duration::from_secs(10), "streaming turn worker");

        assert_eq!(vec!["Hello".to_string(), " world".to_string()], sink.deltas());
        assert_eq!("Hello world", output.content);
        assert_eq!(
            vec!["Hello".to_string(), " world".to_string()],
            output.stream_deltas
        );
        assert_eq!(RUNTIME_MODE_CLOUDROUTER, output.runtime_mode);
        assert!(!is_inference_error(output.runtime_mode));
    }

    /// The fallback must never be reached on the account-pool path; if the
    /// streaming request silently degraded, this executor would be invoked and
    /// the assertion above would fail with a fallback message instead.
    struct UnreachableFallback;

    impl TurnExecutor for UnreachableFallback {
        fn complete(&self, _input: &TurnExecutionInput) -> TurnExecutionOutput {
            panic!("account-pool turns must not delegate to the injected executor");
        }
    }
}

