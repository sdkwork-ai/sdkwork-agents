//! Cloud Router account-pool routing turn executor.
//!
//! Executes durable turns through the sdkwork-cloudrouter open-api chat
//! completions gateway, authenticating with the caller's auth token. The
//! cloudrouter account-pool routing pipeline (Default group → accounts →
//! upstream suppliers) selects the supplier for the requested model — no
//! local provider binding or API key configuration is required.

use std::sync::Arc;

use cloudrouter_open_sdk::models::{
    OpenAiChatCompletionRequest, OpenAiChatMessage, OpenAiFunctionCall, OpenAiToolCall,
};
use sdkwork_agent_kernel::{AgentStreamEvent, KernelError, ModelStreamChunk};
use sdkwork_agents_tool_cloudrouter::{
    stream_chat_completion_with_tools_blocking, stream_llm_completion_blocking, WireProtocol,
};

use crate::domain::AgentSessionItemKind;
use crate::runtime_facade_bridge::engine_key_for_binding_id;
use crate::tool_calling::{
    cap_tool_content, TurnToolCall, TurnToolDescriptor, TurnToolDispatcher, TurnToolEvent,
    TurnToolEventKind, TurnToolExecution, TurnToolExecutionContext,
};
use crate::turn_runtime::{
    TurnExecutionInput, TurnExecutionOutput, TurnExecutionStreamSink, TurnExecutor,
};

/// Maximum completion rounds per turn when the model keeps issuing tool calls.
/// Guards against runaway loops; the final round still executes its tools and
/// the loop then terminates with the accumulated answer.
const MAX_TOOL_ROUNDS: usize = 8;

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

/// Turn executor that routes non-rig chat turns through the cloudrouter
/// gateway using the caller's auth token (account-pool routing, no API key
/// required).
///
/// When the turn carries an effective tool set, the executor runs the
/// function-calling loop: the model may issue tool calls, the dispatcher
/// executes them, results are fed back, and the loop iterates until the model
/// answers (bounded by [`MAX_TOOL_ROUNDS`]).
///
/// Rig-bound sessions are delegated to the injected local executor: the RIG
/// agent engine's default model provider routes through the cloud router SDK
/// itself with the caller's dual tokens. Other engines (or unbound sessions)
/// carry the auth token and fall back to the injected local executor when the
/// turn carries none (e.g. worker/backend flows), keeping the durable turn
/// pipeline uniform for every path.
pub struct CloudRouterFirstTurnExecutor<T> {
    fallback: T,
    dispatcher: Arc<TurnToolDispatcher>,
}

impl<T> CloudRouterFirstTurnExecutor<T> {
    pub fn new(fallback: T) -> Self {
        Self {
            fallback,
            dispatcher: Arc::new(TurnToolDispatcher::new()),
        }
    }

    /// Registers the turn-scoped tool dispatcher (generations MCP, media
    /// tools, external MCP).
    pub fn with_tool_dispatcher(mut self, dispatcher: Arc<TurnToolDispatcher>) -> Self {
        self.dispatcher = dispatcher;
        self
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
fn has_cloud_router_auth_token(input: &TurnExecutionInput) -> bool {
    input
        .auth_token
        .as_deref()
        .is_some_and(|token| !token.trim().is_empty())
}

fn should_route_through_cloud_router(input: &TurnExecutionInput) -> bool {
    if !has_cloud_router_auth_token(input) {
        return false;
    }
    let binding_id = input.binding_id.as_deref().unwrap_or("");
    engine_key_for_binding_id(binding_id) != Some("rig")
}

/// Rig-bound sessions normally execute inside the RIG engine, but its
/// in-process rust runtime buffers the full provider response before
/// replaying stream frames. For live HTTP SSE turns we route through the
/// blocking Cloud Router stream client instead so deltas reach the client
/// as upstream chunks arrive.
fn should_use_cloud_router_live_stream(input: &TurnExecutionInput) -> bool {
    if !has_cloud_router_auth_token(input) {
        return false;
    }
    let binding_id = input.binding_id.as_deref().unwrap_or("");
    engine_key_for_binding_id(binding_id) == Some("rig")
}

impl<T> TurnExecutor for CloudRouterFirstTurnExecutor<T>
where
    T: TurnExecutor,
{
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        if should_route_through_cloud_router(input) {
            complete_cloud_router_turn(input, &self.dispatcher)
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
            if prefer_stream {
                complete_cloud_router_streaming_turn(input, &self.dispatcher, None)
            } else {
                complete_cloud_router_turn(input, &self.dispatcher)
            }
        } else if prefer_stream && should_use_cloud_router_live_stream(input) {
            complete_cloud_router_streaming_turn(input, &self.dispatcher, None)
        } else {
            self.fallback
                .complete_with_stream_preference(input, prefer_stream)
        }
    }

    fn complete_with_stream_sink(
        &self,
        input: &TurnExecutionInput,
        sink: Arc<dyn TurnExecutionStreamSink>,
    ) -> TurnExecutionOutput {
        if should_route_through_cloud_router(input) || should_use_cloud_router_live_stream(input) {
            complete_cloud_router_streaming_turn(input, &self.dispatcher, Some(sink.as_ref()))
        } else {
            self.fallback.complete_with_stream_sink(input, sink)
        }
    }
}

fn complete_cloud_router_streaming_turn(
    input: &TurnExecutionInput,
    dispatcher: &TurnToolDispatcher,
    sink: Option<&dyn TurnExecutionStreamSink>,
) -> TurnExecutionOutput {
    match run_cloud_router_turn(input, dispatcher, sink) {
        Ok(output) => output,
        Err(error) => {
            tracing::warn!(
                session_id = %input.session.session_id,
                turn_id = %input.turn_id,
                error = %error,
                "cloud router streaming turn execution failed"
            );
            inference_error_output(input, format!("cloud router turn failed: {error}"))
        }
    }
}

fn complete_cloud_router_turn(
    input: &TurnExecutionInput,
    dispatcher: &TurnToolDispatcher,
) -> TurnExecutionOutput {
    match run_cloud_router_turn(input, dispatcher, None) {
        Ok(output) => output,
        Err(error) => {
            tracing::warn!(
                session_id = %input.session.session_id,
                turn_id = %input.turn_id,
                error = %error,
                "cloud router turn execution failed"
            );
            inference_error_output(input, format!("cloud router turn failed: {error}"))
        }
    }
}

/// Executes one turn through the cloudrouter gateway, running the
/// function-calling loop when the input carries an effective tool set.
///
/// Tool calls are executed serially (the transport pins
/// `parallel_tool_calls: false`); each result is fed back as a `role: tool`
/// message and the loop iterates until the model answers or
/// [`MAX_TOOL_ROUNDS`] rounds elapse. `requires_approval` tools are never
/// executed inline — the loop reports `approval_required` back to the model so
/// it can ask the user to confirm in conversation.
fn run_cloud_router_turn(
    input: &TurnExecutionInput,
    dispatcher: &TurnToolDispatcher,
    sink: Option<&dyn TurnExecutionStreamSink>,
) -> Result<TurnExecutionOutput, KernelError> {
    let auth_token = input
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| KernelError::validation("cloud router execution requires an auth token"))?;
    let access_token = input
        .access_token
        .as_deref()
        .filter(|token| !token.trim().is_empty());
    let protocol = resolve_wire_protocol(input)?;
    // Function calling is carried on the chat_completions wire for P1; other
    // protocols stream text-only until their tool mapping lands.
    let tool_loop_enabled =
        !input.effective_tools.is_empty() && protocol == WireProtocol::ChatCompletions;

    let mut messages = build_chat_messages(input);
    let mut tool_events: Vec<TurnToolEvent> = Vec::new();
    let mut reasoning_events: Vec<sdkwork_agent_kernel::KernelEvent> = Vec::new();
    let mut content_rounds: Vec<String> = Vec::new();
    let mut stream_deltas: Vec<String> = Vec::new();
    let mut model_id: Option<String> = None;
    let mut finish_reason: Option<String> = None;

    for _round in 0..MAX_TOOL_ROUNDS {
        let mut request = build_chat_completion_request(input, true);
        request.messages = std::mem::take(&mut messages);
        let tools_json = if tool_loop_enabled {
            build_openai_tools_json(&input.effective_tools)
        } else {
            Vec::new()
        };
        // Reasoning/thinking deltas follow the industry-standard separate
        // channel (`agent.stream.message.delta` rich events with
        // `kind: "reasoning"`); the visible answer keeps flowing through the
        // plain `delta` frames. All four wire protocols normalize reasoning
        // into the same delta channel.
        let mut reasoning_sequence = reasoning_events.len() as u64;
        let mut on_delta = |delta: sdkwork_agents_tool_cloudrouter::CloudRouterStreamDelta| {
            if !delta.reasoning_content.is_empty() {
                let chunk = ModelStreamChunk::reasoning(
                    input.model_request_id.clone(),
                    reasoning_sequence,
                    delta.reasoning_content.clone(),
                );
                reasoning_sequence += 1;
                let event = AgentStreamEvent::from(&chunk).to_kernel_event();
                if let Some(sink) = sink {
                    let _ = sink.push_event(&event);
                }
                reasoning_events.push(event);
            }
            if !delta.content.is_empty() {
                if let Some(sink) = sink {
                    sink.push_delta(&delta.content);
                }
            }
        };
        let mut streamed = if tool_loop_enabled {
            stream_chat_completion_with_tools_blocking(
                &cloudrouter_base_url(),
                auth_token,
                access_token,
                request,
                tools_json,
                &mut on_delta,
            )
        } else {
            stream_llm_completion_blocking(
                protocol,
                &cloudrouter_base_url(),
                auth_token,
                access_token,
                request,
                &mut on_delta,
            )
        }
        .map_err(cloud_router_error)?;

        model_id = streamed.model.clone().or(model_id);
        finish_reason = streamed.finish_reason.clone().or(finish_reason);
        stream_deltas.extend(streamed.stream_deltas.clone());
        if !streamed.content.is_empty() {
            content_rounds.push(streamed.content.clone());
        }

        let tool_round = !streamed.tool_calls.is_empty()
            || streamed.finish_reason.as_deref() == Some("tool_calls");
        if !tool_round {
            break;
        }
        // `finish_reason: tool_calls` without any reconstructed call means the
        // upstream streamed a shape we cannot execute — terminate instead of
        // looping forever on an unparsable completion.
        if streamed.tool_calls.is_empty() {
            break;
        }
        // Upstreams that omit the tool-call id on the wire still need a
        // stable identifier echoed on the assistant message and the matching
        // `role: tool` result — synthesize one and use it everywhere.
        for (index, streamed_call) in streamed.tool_calls.iter_mut().enumerate() {
            if streamed_call.id.trim().is_empty() {
                streamed_call.id = format!("call_{}_{}", input.model_request_id, index);
            }
        }
        let assistant_tool_calls: Vec<OpenAiToolCall> = streamed
            .tool_calls
            .iter()
            .map(|call| OpenAiToolCall {
                id: call.id.clone(),
                r#type: "function".to_string(),
                function: Some(OpenAiFunctionCall {
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                }),
            })
            .collect();
        messages.push(OpenAiChatMessage {
            content: None,
            role: "assistant".to_string(),
            tool_calls: Some(assistant_tool_calls),
            ..Default::default()
        });
        for streamed_call in &streamed.tool_calls {
            let descriptor = input.effective_tools.iter().find(|descriptor| {
                descriptor.name == streamed_call.name || descriptor.tool_id == streamed_call.name
            });
            let (status, result_content) = match descriptor {
                None => {
                    let message = format!(
                        "unknown tool `{}` in this agent's toolkit",
                        streamed_call.name
                    );
                    tool_events.push(TurnToolEvent {
                        tool_call_id: streamed_call.id.clone(),
                        tool_id: streamed_call.name.clone(),
                        kind: TurnToolEventKind::ToolUse,
                        status: "failed".to_string(),
                        arguments_json: Some(cap_tool_content(&streamed_call.arguments)),
                        content: None,
                    });
                    ("failed".to_string(), message)
                }
                Some(descriptor) => execute_turn_tool(
                    input,
                    dispatcher,
                    sink,
                    streamed_call,
                    descriptor,
                    auth_token,
                    access_token,
                    &mut tool_events,
                ),
            };
            let capped = cap_tool_content(&result_content);
            let tool_id = descriptor
                .map(|descriptor| descriptor.tool_id.clone())
                .unwrap_or_else(|| streamed_call.name.clone());
            tool_events.push(TurnToolEvent {
                tool_call_id: streamed_call.id.clone(),
                tool_id: tool_id.clone(),
                kind: TurnToolEventKind::ToolResult,
                status: status.clone(),
                arguments_json: None,
                content: Some(capped.clone()),
            });
            emit_tool_call_result(sink, streamed_call, &tool_id, &status, &capped);
            messages.push(OpenAiChatMessage {
                content: Some(capped),
                role: "tool".to_string(),
                tool_call_id: Some(streamed_call.id.clone()),
                ..Default::default()
            });
        }
    }

    let content = content_rounds.join("\n");
    let content = if content.trim().is_empty() {
        "本次任务已执行工具调用，但模型未生成最终回答，请补充要求后重试。".to_string()
    } else {
        content
    };
    let output_tokens = estimate_tokens(&content);
    Ok(TurnExecutionOutput {
        tool_events,
        model_request_id: Some(input.model_request_id.clone()),
        finish_reason: finish_reason.or_else(|| Some("stop".to_string())),
        content,
        model_id,
        provider_id: None,
        provider_session_id: None,
        input_tokens: estimate_tokens(&input.user_content),
        output_tokens,
        runtime_mode: RUNTIME_MODE_CLOUDROUTER,
        stream_deltas,
        stream_events: reasoning_events,
    })
}

/// Executes one model-selected tool call and returns the `(status, content)`
/// pair fed back to the model. Invalid-JSON arguments surface as an explicit
/// tool failure (so the model can re-issue the call) instead of silently
/// executing with null arguments; `requires_approval` tools are never
/// executed inline.
#[allow(clippy::too_many_arguments)]
fn execute_turn_tool(
    input: &TurnExecutionInput,
    dispatcher: &TurnToolDispatcher,
    sink: Option<&dyn TurnExecutionStreamSink>,
    streamed_call: &sdkwork_agents_tool_cloudrouter::StreamedToolCall,
    descriptor: &TurnToolDescriptor,
    auth_token: &str,
    access_token: Option<&str>,
    tool_events: &mut Vec<TurnToolEvent>,
) -> (String, String) {
    let arguments = match serde_json::from_str::<serde_json::Value>(&streamed_call.arguments) {
        Ok(value) if value.is_object() => value,
        _ => {
            let message = format!(
                "tool `{}` arguments are not a valid JSON object; re-issue the call with valid JSON arguments",
                descriptor.tool_id
            );
            tool_events.push(TurnToolEvent {
                tool_call_id: streamed_call.id.clone(),
                tool_id: descriptor.tool_id.clone(),
                kind: TurnToolEventKind::ToolUse,
                status: "failed".to_string(),
                arguments_json: Some(cap_tool_content(&streamed_call.arguments)),
                content: None,
            });
            return ("failed".to_string(), message);
        }
    };
    let call = TurnToolCall {
        tool_call_id: streamed_call.id.clone(),
        tool_id: descriptor.tool_id.clone(),
        arguments: arguments.clone(),
        session_id: Some(input.session.session_id.clone()),
        trace_id: Some(input.turn_id.clone()),
        tenant_id: Some(input.session.tenant_id),
    };
    tool_events.push(TurnToolEvent {
        tool_call_id: call.tool_call_id.clone(),
        tool_id: descriptor.tool_id.clone(),
        kind: TurnToolEventKind::ToolUse,
        status: "running".to_string(),
        arguments_json: Some(cap_tool_content(
            &serde_json::to_string(&arguments).unwrap_or_default(),
        )),
        content: None,
    });
    emit_tool_call_start(sink, &call, descriptor);
    let execution = if descriptor.requires_approval {
        TurnToolExecution::ApprovalRequired {
            detail: format!(
                "tool `{}` requires user approval before execution; ask the user to confirm",
                descriptor.tool_id
            ),
        }
    } else {
        let context = TurnToolExecutionContext {
            auth_token: Some(auth_token),
            access_token,
            mcp_connections: &input.mcp_connections,
        };
        match dispatcher.execute(&call, &context) {
            Ok(execution) => execution,
            Err(message) => TurnToolExecution::Failed {
                code: "tool_dispatch_failed".to_string(),
                message,
            },
        }
    };
    match execution {
        TurnToolExecution::Completed { content } => ("succeeded".to_string(), content),
        TurnToolExecution::ApprovalRequired { detail } => ("approval_required".to_string(), detail),
        TurnToolExecution::Failed { code, message } => (
            "failed".to_string(),
            format!("tool {} failed ({code}): {message}", descriptor.tool_id),
        ),
    }
}

/// Emits the kernel `agent.stream.tool.call.start` rich event for live clients.
fn emit_tool_call_start(
    sink: Option<&dyn TurnExecutionStreamSink>,
    call: &TurnToolCall,
    descriptor: &TurnToolDescriptor,
) {
    let Some(sink) = sink else {
        return;
    };
    let event = sdkwork_agent_kernel::AgentStreamEvent::ToolCallStart(
        sdkwork_agent_kernel::ToolCallStartEvent::new(
            &call.tool_call_id,
            &call.tool_call_id,
            descriptor.name.clone(),
        ),
    )
    .to_kernel_event();
    let _ = sink.push_event(&event);
}

/// Emits the kernel `agent.stream.tool.result` rich event for live clients.
///
/// The event is built directly (instead of through
/// `AgentStreamEvent::ToolResult`) because the kernel's compact envelope omits
/// the result content — the chat UI needs the payload (media URLs, error
/// text) to replace the running placeholder with the generated asset.
fn emit_tool_call_result(
    sink: Option<&dyn TurnExecutionStreamSink>,
    call: &sdkwork_agents_tool_cloudrouter::StreamedToolCall,
    tool_id: &str,
    status: &str,
    content: &str,
) {
    let Some(sink) = sink else {
        return;
    };
    let payload = serde_json::json!({
        "event_type": "agent.stream.tool.result",
        "tool_call_id": call.id,
        "tool_name": tool_id,
        "content": content,
        "is_error": status == "failed",
        "status": status,
    })
    .to_string();
    let event = sdkwork_agent_kernel::KernelEvent::new(
        &call.id,
        "agent.stream.tool.result",
        sdkwork_agent_kernel::KernelEventSeverity::Info,
        payload,
    )
    .from_source(sdkwork_agent_kernel::KernelEventSource::Tool)
    .with_redaction(sdkwork_agent_kernel::KernelEventRedaction::Public);
    let _ = sink.push_event(&event);
}

/// Maps the effective tool set onto the OpenAI `tools` array (JSON Schema
/// documents passed through verbatim for full fidelity).
fn build_openai_tools_json(tools: &[TurnToolDescriptor]) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|descriptor| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": descriptor.name,
                    "description": descriptor.description,
                    "parameters": descriptor.input_schema,
                }
            })
        })
        .collect()
}

/// Resolves the requested wire protocol for one turn. Blank/absent values
/// fall back to the default chat completions protocol; unknown identifiers
/// fail closed with a validation error instead of silently switching APIs.
fn resolve_wire_protocol(input: &TurnExecutionInput) -> Result<WireProtocol, KernelError> {
    let requested = input
        .wire_protocol
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    WireProtocol::parse(requested).ok_or_else(|| {
        KernelError::validation(format!(
            "unknown wire protocol: {requested} (expected chat_completions, \
             anthropic_messages, google_content, or openai_responses)"
        ))
    })
}

/// Maps the durable turn history into OpenAI chat messages: the agent system
/// prompt (request-level override wins; otherwise the slot-assembled prompt)
/// and welcome message lead as `system` messages (mirroring
/// `build_model_items` on the agent-engine path so both turn paths honor the
/// same agent personality), followed by the `user`/`assistant` history and the
/// current user content.
fn build_chat_messages(input: &TurnExecutionInput) -> Vec<OpenAiChatMessage> {
    let mut messages: Vec<OpenAiChatMessage> = Vec::with_capacity(input.history.len() + 3);
    for (label, content) in [
        (
            "system",
            input
                .system_prompt
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .or(input.assembled_system_prompt.as_deref()),
        ),
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
    messages
}

fn build_chat_completion_request(
    input: &TurnExecutionInput,
    stream: bool,
) -> OpenAiChatCompletionRequest {
    OpenAiChatCompletionRequest {
        model: input
            .model_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL_KEY.to_string()),
        messages: build_chat_messages(input),
        stream: Some(stream),
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
        SdkworkError::HttpStatus { status, body } if *status >= 500 => {
            sdkwork_agents_tool_cloudrouter::cloudrouter_http_error_hint(*status, body)
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

fn inference_error_output(input: &TurnExecutionInput, message: String) -> TurnExecutionOutput {
    TurnExecutionOutput {
        tool_events: Vec::new(),
        model_request_id: Some(input.model_request_id.clone()),
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
    use crate::domain::{
        AgentSessionEntrySurface, AgentSessionKind, AgentSessionRecord, AgentSessionStatus,
    };

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
            effective_tools: Vec::new(),
            assembled_system_prompt: None,
            mcp_connections: Vec::new(),
            turn_id: "turn.test".to_string(),
            model_request_id: "model-request.test".to_string(),
            agent_display_name: "Test Agent".to_string(),
            welcome_message: None,
            session: sample_session(),
            history: vec![
                (AgentSessionItemKind::UserInput, "hello".to_string()),
                (
                    AgentSessionItemKind::AssistantOutput,
                    "hi there".to_string(),
                ),
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
            wire_protocol: None,
        }
    }

    #[test]
    fn resolve_wire_protocol_defaults_and_fails_closed() {
        let mut input = sample_input(Some("token"), None);
        assert_eq!(
            resolve_wire_protocol(&input).unwrap(),
            WireProtocol::ChatCompletions
        );
        input.wire_protocol = Some("anthropic_messages".to_string());
        assert_eq!(
            resolve_wire_protocol(&input).unwrap(),
            WireProtocol::AnthropicMessages
        );
        input.wire_protocol = Some("google_content".to_string());
        assert_eq!(
            resolve_wire_protocol(&input).unwrap(),
            WireProtocol::GoogleContent
        );
        input.wire_protocol = Some("openai_responses".to_string());
        assert_eq!(
            resolve_wire_protocol(&input).unwrap(),
            WireProtocol::OpenAiResponses
        );
        input.wire_protocol = Some("bogus".to_string());
        assert!(resolve_wire_protocol(&input).is_err());
    }

    #[test]
    fn builds_openai_messages_from_turn_history() {
        let input = sample_input(Some("token"), Some("access"));
        let request = build_chat_completion_request(&input, false);
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
        let request = build_chat_completion_request(&input, false);
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
        let request = build_chat_completion_request(&input, false);
        assert_eq!(request.model, "default");
    }

    #[test]
    fn executor_without_auth_token_delegates_to_fallback() {
        struct RecordingFallback(std::sync::Mutex<Vec<String>>);
        impl TurnExecutor for RecordingFallback {
            fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
                self.0.lock().unwrap().push(input.user_content.clone());
                inference_error_output(input, "fallback".to_string())
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
    fn streaming_request_sets_stream_flag() {
        let input = sample_input(Some("token"), Some("access"));
        let request = build_chat_completion_request(&input, true);
        assert_eq!(request.stream, Some(true));
    }

    #[test]
    fn rig_bound_session_uses_live_stream_shortcut_when_auth_present() {
        let mut input = sample_input(Some("token"), Some("access"));
        input.binding_id = Some("binding.rig".to_string());
        assert!(!should_route_through_cloud_router(&input));
        assert!(should_use_cloud_router_live_stream(&input));
    }

    #[test]
    fn rig_bound_session_without_auth_does_not_use_live_stream_shortcut() {
        let mut input = sample_input(None, None);
        input.binding_id = Some("binding.rig".to_string());
        assert!(!should_use_cloud_router_live_stream(&input));
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
    fn tool_result_event_carries_result_content_for_media_rendering() {
        // The chat UI replaces the running placeholder with the generated
        // media by reading `payload.content` off the tool.result event, so
        // the payload must carry the full result text (the kernel's compact
        // ToolResult envelope omits it).
        struct RecordingSink(std::sync::Mutex<Vec<sdkwork_agent_kernel::KernelEvent>>);
        impl TurnExecutionStreamSink for RecordingSink {
            fn push_delta(&self, _delta: &str) {}
            fn push_event(
                &self,
                event: &sdkwork_agent_kernel::KernelEvent,
            ) -> sdkwork_agent_kernel::KernelResult<()> {
                self.0.lock().unwrap().push(event.clone());
                Ok(())
            }
        }
        let sink = RecordingSink(std::sync::Mutex::new(Vec::new()));
        let call = sdkwork_agents_tool_cloudrouter::StreamedToolCall {
            id: "call_1".to_string(),
            name: "mcp__generations__image.create".to_string(),
            arguments: "{}".to_string(),
        };
        let content = r#"{"generation":{"id":"gen-1"},"mediaUrls":["https://cdn/img.png"]}"#;
        emit_tool_call_result(
            Some(&sink),
            &call,
            "mcp__generations__image.create",
            "succeeded",
            content,
        );
        let events = sink.0.lock().unwrap();
        assert_eq!(events.len(), 1);
        let payload: serde_json::Value = serde_json::from_str(&events[0].payload).unwrap();
        assert_eq!(events[0].event_type, "agent.stream.tool.result");
        assert_eq!(payload["tool_call_id"], "call_1");
        assert_eq!(payload["tool_name"], "mcp__generations__image.create");
        assert_eq!(payload["is_error"], false);
        assert!(
            payload["content"]
                .as_str()
                .unwrap()
                .contains("https://cdn/img.png"),
            "payload content must carry the media URL"
        );

        // Failure results mark is_error so the UI renders the error state.
        emit_tool_call_result(
            Some(&sink),
            &call,
            "mcp__generations__image.create",
            "failed",
            "boom",
        );
        let events = sink.0.lock().unwrap();
        let payload: serde_json::Value = serde_json::from_str(&events[1].payload).unwrap();
        assert_eq!(payload["is_error"], true);
    }

    #[test]
    fn streamed_tool_call_ids_are_synthesized_when_upstream_omits_them() {
        // Mirrors the loop's id-synthesis rule: stable ids are required so
        // the assistant message and the tool result stay paired.
        let mut calls = vec![
            sdkwork_agents_tool_cloudrouter::StreamedToolCall::default(),
            sdkwork_agents_tool_cloudrouter::StreamedToolCall {
                id: "call_upstream".to_string(),
                name: "mcp__generations__music.create".to_string(),
                arguments: "{}".to_string(),
            },
        ];
        for (index, call) in calls.iter_mut().enumerate() {
            if call.id.trim().is_empty() {
                call.id = format!("call_{}_{}", "model-request.test", index);
            }
        }
        assert_eq!(calls[0].id, "call_model-request.test_0");
        assert_eq!(calls[1].id, "call_upstream");
    }
}
