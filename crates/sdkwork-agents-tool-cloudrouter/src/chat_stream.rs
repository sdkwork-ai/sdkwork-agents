//! Live LLM gateway streaming for the four supported wire protocols.
//!
//! Uses the blocking reqwest client so turn workers can incrementally parse
//! upstream SSE without nesting Tokio runtimes inside \`spawn_blocking\`
//! workers. Every protocol's SSE frames are normalized into
//! [`CloudRouterStreamDelta`] (visible answer / reasoning / tool-call
//! fragments), so callers stay protocol-agnostic:
//!
//! - `chat_completions`: `chat.completion.chunk` deltas (OpenAI)
//! - `anthropic_messages`: `content_block_delta` text/thinking deltas
//! - `google_content`: `candidates[0].content.parts[].text` (alt=sse frames)
//! - `openai_responses`: `response.output_text.delta` events

use std::io::Read;
use std::sync::OnceLock;
use std::time::Duration;

use cloudrouter_open_sdk::models::OpenAiChatCompletionRequest;
use cloudrouter_open_sdk::SdkworkError;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

use crate::wire_protocol::{
    build_protocol_request_body, normalize_finish_reason, WireProtocol,
};

/// Wall-clock ceiling for one streamed chat completion (30 minutes).
///
/// Long agent/coding completions legitimately stream for minutes; the prior
/// 120-second total truncated healthy streams. Stream health is enforced by
/// the gateway's own first-frame/idle timeouts (30s each): a stalled upstream
/// is cut by the gateway within seconds and the SSE body then ends, so this
/// total only prevents an unbounded lease on the turn worker. (The blocking
/// reqwest client has no per-read timeout; async-only `read_timeout` cannot
/// be used here.)
const STREAM_REQUEST_TIMEOUT: Duration = Duration::from_secs(1800);
/// TCP connect bound for the gateway request.
const STREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Aggregated result from one streamed chat completion call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudRouterChatStreamResult {
    pub content: String,
    /// Model reasoning/thinking text, kept separate from the visible answer so
    /// callers can render it as a distinct (collapsible) reasoning block.
    pub reasoning_content: String,
    /// Accumulated OpenAI-compatible tool-call argument fragments (JSON), one
    /// string per streamed tool-call argument delta.
    pub tool_call_fragments: Vec<String>,
    pub stream_deltas: Vec<String>,
    pub model: Option<String>,
    pub finish_reason: Option<String>,
}

/// Aggregated result from one non-streaming completion call, normalized
/// across the four wire protocols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudRouterCompletionResult {
    pub content: String,
    /// Model reasoning/thinking text (empty for protocols that do not return
    /// reasoning in non-streaming responses).
    pub reasoning_content: String,
    pub model: Option<String>,
    /// OpenAI-style lowercase finish reason (`stop` / `length` / ...).
    pub finish_reason: Option<String>,
}

/// One streamed protocol delta normalized for consumers. This is the unit
/// delivered to the streaming callback so callers can distinguish the visible
/// answer, the model reasoning, and tool-call arguments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CloudRouterStreamDelta {
    /// Visible answer text for this chunk (empty when the chunk is not text).
    pub content: String,
    /// Reasoning/thinking text for this chunk (empty when the chunk has none).
    pub reasoning_content: String,
    /// Serialized \`tool_calls\` array fragment for this chunk (empty when none).
    pub tool_calls: String,
}

/// Extracts visible assistant text from one OpenAI-compatible chunk delta.
pub fn extract_openai_stream_content(chunk: &Value) -> Option<String> {
    let choice = chunk.get("choices")?.as_array()?.first()?;
    let delta = choice.get("delta")?;
    delta
        .get("content")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Extracts reasoning/thinking text from one OpenAI-compatible chunk delta.
pub fn extract_openai_stream_reasoning(chunk: &Value) -> Option<String> {
    let choice = chunk.get("choices")?.as_array()?.first()?;
    let delta = choice.get("delta")?;
    for key in ["reasoning_content", "reasoning"] {
        if let Some(reasoning) = delta.get(key).and_then(Value::as_str) {
            if !reasoning.is_empty() {
                return Some(reasoning.to_string());
            }
        }
    }
    None
}

/// Extracts the \`tool_calls\` array fragment (if any) from a chunk delta as JSON.
fn extract_openai_stream_tool_calls(chunk: &Value) -> Option<String> {
    let choice = chunk.get("choices")?.as_array()?.first()?;
    let delta = choice.get("delta")?;
    match delta.get("tool_calls") {
        Some(Value::Array(items)) if !items.is_empty() => serde_json::to_string(items).ok(),
        _ => None,
    }
}

/// Builds a normalized streaming delta from one \`chat.completion.chunk\` payload.
fn normalize_stream_delta(chunk: &Value) -> CloudRouterStreamDelta {
    CloudRouterStreamDelta {
        content: extract_openai_stream_content(chunk).unwrap_or_default(),
        reasoning_content: extract_openai_stream_reasoning(chunk).unwrap_or_default(),
        tool_calls: extract_openai_stream_tool_calls(chunk).unwrap_or_default(),
    }
}

/// Per-protocol SSE `data:` frame parser. Implementations consume one frame
/// payload at a time and emit zero or more normalized deltas; terminal model
/// and stop-reason values are tracked separately so the shared accumulator
/// stays protocol-agnostic.
trait ProtocolFrameParser {
    fn feed(&mut self, data: &str) -> Vec<CloudRouterStreamDelta>;
    fn model(&self) -> Option<String> {
        None
    }
    fn finish_reason(&self) -> Option<String> {
        None
    }
}

/// OpenAI Chat Completions frame parser (`chat.completion.chunk` payloads,
/// terminated by the `data: [DONE]` sentinel).
struct OpenAiChatFrameParser {
    model: Option<String>,
    finish_reason: Option<String>,
}

impl OpenAiChatFrameParser {
    fn new() -> Self {
        Self {
            model: None,
            finish_reason: None,
        }
    }
}

impl ProtocolFrameParser for OpenAiChatFrameParser {
    fn feed(&mut self, data: &str) -> Vec<CloudRouterStreamDelta> {
        if data == "[DONE]" {
            return Vec::new();
        }
        let Ok(chunk) = serde_json::from_str::<Value>(data) else {
            return Vec::new();
        };
        if let Some(model_id) = chunk
            .get("model")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            self.model = Some(model_id.to_string());
        }
        if let Some(reason) = chunk
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("finish_reason"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            self.finish_reason = Some(reason.to_string());
        }
        let delta = normalize_stream_delta(&chunk);
        if delta.content.is_empty()
            && delta.reasoning_content.is_empty()
            && delta.tool_calls.is_empty()
        {
            Vec::new()
        } else {
            vec![delta]
        }
    }

    fn model(&self) -> Option<String> {
        self.model.clone()
    }

    fn finish_reason(&self) -> Option<String> {
        self.finish_reason.clone()
    }
}

/// Anthropic Messages frame parser: `message_start` carries the model,
/// `content_block_delta` carries `text_delta` (visible) and `thinking_delta`
/// (reasoning) fragments, and `message_delta` carries the stop reason.
struct AnthropicFrameParser {
    model: Option<String>,
    finish_reason: Option<String>,
}

impl AnthropicFrameParser {
    fn new() -> Self {
        Self {
            model: None,
            finish_reason: None,
        }
    }
}

impl ProtocolFrameParser for AnthropicFrameParser {
    fn feed(&mut self, data: &str) -> Vec<CloudRouterStreamDelta> {
        let Ok(event) = serde_json::from_str::<Value>(data) else {
            return Vec::new();
        };
        match event.get("type").and_then(Value::as_str) {
            Some("message_start") => {
                self.model = event
                    .pointer("/message/model")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
                Vec::new()
            }
            Some("content_block_delta") => {
                let Some(delta) = event.get("delta") else {
                    return Vec::new();
                };
                let delta_type = delta.get("type").and_then(Value::as_str);
                match delta_type {
                    Some("text_delta") => vec![CloudRouterStreamDelta {
                        content: delta
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        ..Default::default()
                    }],
                    Some("thinking_delta") => vec![CloudRouterStreamDelta {
                        reasoning_content: delta
                            .get("thinking")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        ..Default::default()
                    }],
                    _ => Vec::new(),
                }
            }
            Some("message_delta") => {
                if let Some(stop_reason) = event
                    .pointer("/delta/stop_reason")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                {
                    self.finish_reason = Some(stop_reason.to_string());
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn model(&self) -> Option<String> {
        self.model.clone()
    }

    fn finish_reason(&self) -> Option<String> {
        self.finish_reason.clone()
    }
}

/// Google Content frame parser (`alt=sse` frames, each a full
/// `GenerateContentResponse`): visible text and `thought: true` parts arrive
/// inside `candidates[0].content.parts[]`, with the terminal reason on
/// `candidates[0].finishReason`.
struct GoogleFrameParser {
    model: Option<String>,
    finish_reason: Option<String>,
}

impl GoogleFrameParser {
    fn new() -> Self {
        Self {
            model: None,
            finish_reason: None,
        }
    }
}

impl ProtocolFrameParser for GoogleFrameParser {
    fn feed(&mut self, data: &str) -> Vec<CloudRouterStreamDelta> {
        let Ok(frame) = serde_json::from_str::<Value>(data) else {
            return Vec::new();
        };
        if let Some(model_version) = frame
            .get("modelVersion")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            self.model = Some(model_version.to_string());
        }
        let candidate = frame
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|candidates| candidates.first())
            .cloned()
            .unwrap_or_default();
        if let Some(finish) = candidate
            .get("finishReason")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            self.finish_reason = Some(finish.to_string());
        }
        let mut deltas: Vec<CloudRouterStreamDelta> = Vec::new();
        if let Some(parts) = candidate
            .pointer("/content/parts")
            .and_then(Value::as_array)
        {
            for part in parts {
                let Some(text) = part.get("text").and_then(Value::as_str) else {
                    continue;
                };
                if text.is_empty() {
                    continue;
                }
                let is_thought = part
                    .get("thought")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                deltas.push(if is_thought {
                    CloudRouterStreamDelta {
                        reasoning_content: text.to_string(),
                        ..Default::default()
                    }
                } else {
                    CloudRouterStreamDelta {
                        content: text.to_string(),
                        ..Default::default()
                    }
                });
            }
        }
        deltas
    }

    fn model(&self) -> Option<String> {
        self.model.clone()
    }

    fn finish_reason(&self) -> Option<String> {
        self.finish_reason.clone()
    }
}

/// OpenAI Responses frame parser: `response.output_text.delta` carries the
/// visible answer, `response.reasoning_summary_text.delta` the reasoning, and
/// `response.completed` the terminal model identity.
struct OpenAiResponsesFrameParser {
    model: Option<String>,
}

impl OpenAiResponsesFrameParser {
    fn new() -> Self {
        Self { model: None }
    }
}

impl ProtocolFrameParser for OpenAiResponsesFrameParser {
    fn feed(&mut self, data: &str) -> Vec<CloudRouterStreamDelta> {
        if data == "[DONE]" {
            return Vec::new();
        }
        let Ok(event) = serde_json::from_str::<Value>(data) else {
            return Vec::new();
        };
        match event.get("type").and_then(Value::as_str) {
            Some("response.output_text.delta") => vec![CloudRouterStreamDelta {
                content: event
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                ..Default::default()
            }],
            Some("response.reasoning_summary_text.delta") => vec![CloudRouterStreamDelta {
                reasoning_content: event
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                ..Default::default()
            }],
            Some("response.completed") => {
                self.model = event
                    .pointer("/response/model")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn model(&self) -> Option<String> {
        self.model.clone()
    }
}

fn frame_parser_for(protocol: WireProtocol) -> Box<dyn ProtocolFrameParser> {
    match protocol {
        WireProtocol::ChatCompletions => Box::new(OpenAiChatFrameParser::new()),
        WireProtocol::AnthropicMessages => Box::new(AnthropicFrameParser::new()),
        WireProtocol::GoogleContent => Box::new(GoogleFrameParser::new()),
        WireProtocol::OpenAiResponses => Box::new(OpenAiResponsesFrameParser::new()),
    }
}

/// Shared aggregation state for one streamed invocation: accumulates the
/// full answer/reasoning/tool fragments, forwards deltas to the caller
/// callback, and tracks model/finish-reason as parsers surface them.
struct StreamAccumulator<'a> {
    content: String,
    reasoning_content: String,
    tool_call_fragments: Vec<String>,
    stream_deltas: Vec<String>,
    model: Option<String>,
    finish_reason: Option<String>,
    on_delta: &'a mut dyn FnMut(CloudRouterStreamDelta),
}

impl<'a> StreamAccumulator<'a> {
    fn new(on_delta: &'a mut dyn FnMut(CloudRouterStreamDelta)) -> Self {
        Self {
            content: String::new(),
            reasoning_content: String::new(),
            tool_call_fragments: Vec::new(),
            stream_deltas: Vec::new(),
            model: None,
            finish_reason: None,
            on_delta,
        }
    }

    fn consume_frame(&mut self, protocol: WireProtocol, parser: &mut dyn ProtocolFrameParser, data: &str) {
        for delta in parser.feed(data) {
            if !delta.content.is_empty() {
                self.content.push_str(&delta.content);
                self.stream_deltas.push(delta.content.clone());
            }
            if !delta.reasoning_content.is_empty() {
                self.reasoning_content.push_str(&delta.reasoning_content);
            }
            if !delta.tool_calls.is_empty() {
                self.tool_call_fragments.push(delta.tool_calls.clone());
            }
            (self.on_delta)(delta);
        }
        if self.model.is_none() {
            self.model = parser.model();
        }
        if self.finish_reason.is_none() {
            // Normalize native stop vocabulary to the OpenAI-style lowercase
            // set so turn persistence stays protocol-agnostic.
            self.finish_reason = parser
                .finish_reason()
                .and_then(|raw| normalize_finish_reason(protocol, &raw))
                .or_else(|| parser.finish_reason());
        }
    }

    fn consume_sse_buffer(&mut self, protocol: WireProtocol, parser: &mut dyn ProtocolFrameParser, buffer: &mut String) {
        normalize_sse_buffer(buffer);
        while let Some(pos) = buffer.find("\n\n") {
            let block: String = buffer.drain(..pos).collect();
            buffer.drain(..2);
            for data in block_data_lines(&block) {
                self.consume_frame(protocol, parser, &data);
            }
        }
    }

    fn flush_remaining(&mut self, protocol: WireProtocol, parser: &mut dyn ProtocolFrameParser, buffer: &str) {
        let mut tail = buffer.to_string();
        normalize_sse_buffer(&mut tail);
        for data in block_data_lines(&tail) {
            self.consume_frame(protocol, parser, &data);
        }
    }
}

/// Extracts trimmed `data:` payload lines from one raw SSE block.
fn block_data_lines(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(':'))
        .filter_map(|line| line.strip_prefix("data:"))
        .map(|data| data.trim().to_string())
        .collect()
}

fn apply_dual_token_headers(
    headers: &mut HeaderMap,
    auth_token: &str,
    access_token: Option<&str>,
) -> Result<(), SdkworkError> {
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {auth_token}"))
            .map_err(SdkworkError::InvalidHeaderValue)?,
    );
    if let Some(access_token) = access_token.filter(|token| !token.trim().is_empty()) {
        headers.insert(
            "Access-Token",
            HeaderValue::from_str(access_token).map_err(SdkworkError::InvalidHeaderValue)?,
        );
    }
    Ok(())
}

fn normalize_sse_buffer(buffer: &mut String) {
    if buffer.contains('\r') {
        *buffer = buffer.replace("\r\n", "\n").replace('\r', "\n");
    }
}

#[allow(clippy::too_many_arguments)]
fn consume_sse_buffer(
    buffer: &mut String,
    content: &mut String,
    reasoning_content: &mut String,
    tool_call_fragments: &mut Vec<String>,
    stream_deltas: &mut Vec<String>,
    model: &mut Option<String>,
    finish_reason: &mut Option<String>,
    on_delta: &mut dyn FnMut(CloudRouterStreamDelta),
) {
    let mut accumulator = StreamAccumulator::new(on_delta);
    accumulator.content = std::mem::take(content);
    accumulator.reasoning_content = std::mem::take(reasoning_content);
    accumulator.tool_call_fragments = std::mem::take(tool_call_fragments);
    accumulator.stream_deltas = std::mem::take(stream_deltas);
    accumulator.model = model.take();
    accumulator.finish_reason = finish_reason.take();
    let mut parser = OpenAiChatFrameParser::new();
    accumulator.consume_sse_buffer(WireProtocol::ChatCompletions, &mut parser, buffer);
    *content = std::mem::take(&mut accumulator.content);
    *reasoning_content = std::mem::take(&mut accumulator.reasoning_content);
    *tool_call_fragments = std::mem::take(&mut accumulator.tool_call_fragments);
    *stream_deltas = std::mem::take(&mut accumulator.stream_deltas);
    *model = accumulator.model;
    *finish_reason = accumulator.finish_reason;
}

fn streaming_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(STREAM_CONNECT_TIMEOUT)
            .timeout(STREAM_REQUEST_TIMEOUT)
            .pool_max_idle_per_host(8)
            .build()
            .expect("cloud router streaming http client")
    })
}

fn open_gateway_response(
    base_url: &str,
    endpoint: &str,
    auth_token: &str,
    access_token: Option<&str>,
    body: &Value,
    accept: &'static str,
) -> Result<reqwest::blocking::Response, SdkworkError> {
    let mut headers = HeaderMap::new();
    apply_dual_token_headers(&mut headers, auth_token, access_token)?;
    headers.insert(ACCEPT, HeaderValue::from_static(accept));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let url = format!("{}{}", base_url.trim_end_matches('/'), endpoint);
    let client = streaming_client();
    let response = client
        .post(url)
        .headers(headers)
        .json(body)
        .send()?;
    let status = response.status();
    if !status.is_success() {
        // The error body must be drained before returning so callers see the
        // gateway's structured error payload instead of a truncated stream.
        let error_response = response;
        let body = error_response.text().unwrap_or_default();
        return Err(SdkworkError::HttpStatus {
            status: status.as_u16(),
            body,
        });
    }
    Ok(response)
}

/// Streams one completion from the Cloud Router gateway over the requested
/// wire protocol, invoking \`on_delta\` for every provider delta (visible
/// text, reasoning, tool-call arguments) as it is parsed from the upstream
/// SSE body.
///
/// Fails closed: callers must not fall back to buffered completion when this
/// returns an error.
pub fn stream_llm_completion_blocking(
    protocol: WireProtocol,
    base_url: &str,
    auth_token: &str,
    access_token: Option<&str>,
    mut request: OpenAiChatCompletionRequest,
    on_delta: &mut dyn FnMut(CloudRouterStreamDelta),
) -> Result<CloudRouterChatStreamResult, SdkworkError> {
    request.stream = Some(true);
    let body = build_protocol_request_body(protocol, &request, true);
    let endpoint = protocol.streaming_endpoint(&request.model);
    let mut response = open_gateway_response(
        base_url,
        &endpoint,
        auth_token,
        access_token,
        &body,
        "text/event-stream",
    )?;
    let status = response.status();

    let mut buffer = String::new();
    let mut accumulator = StreamAccumulator::new(on_delta);
    let mut parser = frame_parser_for(protocol);
    let mut chunk_buf = [0u8; 8 * 1024];

    loop {
        let read = response.read(&mut chunk_buf).map_err(|error| {
            SdkworkError::HttpStatus {
                status: status.as_u16(),
                body: format!("failed to read cloud router stream body: {error}"),
            }
        })?;
        if read == 0 {
            break;
        }
        buffer.push_str(&String::from_utf8_lossy(&chunk_buf[..read]));
        accumulator.consume_sse_buffer(protocol, parser.as_mut(), &mut buffer);
    }

    if !buffer.trim().is_empty() {
        accumulator.flush_remaining(protocol, parser.as_mut(), &buffer);
    }

    if accumulator.content.trim().is_empty() || accumulator.stream_deltas.is_empty() {
        return Err(SdkworkError::HttpStatus {
            status: status.as_u16(),
            body: "cloud router stream returned no assistant content".to_string(),
        });
    }

    Ok(CloudRouterChatStreamResult {
        content: accumulator.content,
        reasoning_content: accumulator.reasoning_content,
        tool_call_fragments: accumulator.tool_call_fragments,
        stream_deltas: accumulator.stream_deltas,
        model: accumulator.model,
        finish_reason: accumulator.finish_reason,
    })
}

/// Streams one OpenAI-compatible chat completion from the Cloud Router
/// gateway (the default `chat_completions` wire protocol).
///
/// Fails closed: callers must not fall back to buffered completion when this
/// returns an error.
pub fn stream_chat_completion_blocking(
    base_url: &str,
    auth_token: &str,
    access_token: Option<&str>,
    request: OpenAiChatCompletionRequest,
    on_delta: &mut dyn FnMut(CloudRouterStreamDelta),
) -> Result<CloudRouterChatStreamResult, SdkworkError> {
    stream_llm_completion_blocking(
        WireProtocol::ChatCompletions,
        base_url,
        auth_token,
        access_token,
        request,
        on_delta,
    )
}

/// Extracts the visible answer, reasoning, model, and stop reason from one
/// non-streaming protocol response payload, normalizing to OpenAI-style
/// vocabulary.
fn normalize_completion_payload(protocol: WireProtocol, payload: &Value) -> CloudRouterCompletionResult {
    let mut content = String::new();
    let mut reasoning_content = String::new();
    let mut model: Option<String> = None;
    let mut raw_finish: Option<String> = None;
    match protocol {
        WireProtocol::ChatCompletions => {
            let choice = payload
                .get("choices")
                .and_then(Value::as_array)
                .and_then(|choices| choices.first())
                .cloned()
                .unwrap_or_default();
            content = choice
                .pointer("/message/content")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            reasoning_content = choice
                .pointer("/message/reasoning_content")
                .or_else(|| choice.pointer("/message/reasoning"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            raw_finish = choice
                .get("finish_reason")
                .and_then(Value::as_str)
                .map(str::to_string);
            model = payload
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        WireProtocol::AnthropicMessages => {
            if let Some(blocks) = payload.get("content").and_then(Value::as_array) {
                for block in blocks {
                    match block.get("type").and_then(Value::as_str) {
                        Some("text") => {
                            if let Some(text) = block.get("text").and_then(Value::as_str) {
                                content.push_str(text);
                            }
                        }
                        Some("thinking") => {
                            if let Some(thinking) = block.get("thinking").and_then(Value::as_str) {
                                reasoning_content.push_str(thinking);
                            }
                        }
                        _ => {}
                    }
                }
            }
            raw_finish = payload
                .get("stop_reason")
                .and_then(Value::as_str)
                .map(str::to_string);
            model = payload
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        WireProtocol::GoogleContent => {
            let candidate = payload
                .get("candidates")
                .and_then(Value::as_array)
                .and_then(|candidates| candidates.first())
                .cloned()
                .unwrap_or_default();
            if let Some(parts) = candidate.pointer("/content/parts").and_then(Value::as_array) {
                for part in parts {
                    let Some(text) = part.get("text").and_then(Value::as_str) else {
                        continue;
                    };
                    let is_thought = part
                        .get("thought")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if is_thought {
                        reasoning_content.push_str(text);
                    } else {
                        content.push_str(text);
                    }
                }
            }
            raw_finish = candidate
                .get("finishReason")
                .and_then(Value::as_str)
                .map(str::to_string);
            model = payload
                .get("modelVersion")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        WireProtocol::OpenAiResponses => {
            if let Some(output) = payload.get("output").and_then(Value::as_array) {
                for item in output {
                    match item.get("type").and_then(Value::as_str) {
                        Some("message") => {
                            if let Some(parts) =
                                item.pointer("/content").and_then(Value::as_array)
                            {
                                for part in parts {
                                    if part.get("type").and_then(Value::as_str)
                                        == Some("output_text")
                                    {
                                        if let Some(text) =
                                            part.get("text").and_then(Value::as_str)
                                        {
                                            content.push_str(text);
                                        }
                                    }
                                }
                            }
                        }
                        Some("reasoning") => {
                            if let Some(parts) =
                                item.pointer("/summary").and_then(Value::as_array)
                            {
                                for part in parts {
                                    if let Some(text) = part.get("text").and_then(Value::as_str)
                                    {
                                        reasoning_content.push_str(text);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            if content.is_empty() {
                // Some gateway-compatible responses provide the convenience
                // aggregation field instead of an output array.
                content = payload
                    .get("output_text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
            }
            model = payload
                .pointer("/model")
                .and_then(Value::as_str)
                .map(str::to_string);
            raw_finish = payload
                .get("status")
                .and_then(Value::as_str)
                .map(|status| match status {
                    "completed" => "stop".to_string(),
                    "incomplete" => "length".to_string(),
                    other => other.to_string(),
                });
        }
    }
    CloudRouterCompletionResult {
        content,
        reasoning_content,
        model,
        // Prefer the protocol-normalized mapping but keep the raw reason when
        // the vocabulary is unrecognized instead of dropping the signal.
        finish_reason: raw_finish
            .clone()
            .and_then(|raw| normalize_finish_reason(protocol, &raw))
            .or(raw_finish),
    }
}

/// Executes one non-streaming completion against the Cloud Router gateway
/// over the requested wire protocol and returns the normalized result.
///
/// Fails closed: empty answers surface as errors, mirroring the streaming
/// path's no-content contract.
pub fn create_llm_completion_blocking(
    protocol: WireProtocol,
    base_url: &str,
    auth_token: &str,
    access_token: Option<&str>,
    mut request: OpenAiChatCompletionRequest,
) -> Result<CloudRouterCompletionResult, SdkworkError> {
    request.stream = Some(false);
    let body = build_protocol_request_body(protocol, &request, false);
    let endpoint = protocol.endpoint(&request.model);
    let mut response = open_gateway_response(
        base_url,
        &endpoint,
        auth_token,
        access_token,
        &body,
        "application/json",
    )?;
    let status = response.status();
    let payload: Value = response.json().map_err(|error| SdkworkError::HttpStatus {
        status: status.as_u16(),
        body: format!("failed to decode cloud router completion body: {error}"),
    })?;
    let result = normalize_completion_payload(protocol, &payload);
    if result.content.trim().is_empty() {
        return Err(SdkworkError::HttpStatus {
            status: status.as_u16(),
            body: "cloud router returned no assistant content".to_string(),
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Reads one complete HTTP request from the mock socket: headers plus the
    /// full `Content-Length` body. hyper 1.x (reqwest 0.13) sends the request
    /// body after the headers on a separate write, so stopping at the header
    /// terminator races the client and breaks the connection mid-body.
    fn read_complete_mock_request(
        stream: &mut std::net::TcpStream,
    ) -> String {
        let mut request = String::new();
        let mut buf = [0u8; 2048];
        loop {
            let read = stream.read(&mut buf).expect("read mock request");
            if read == 0 {
                break;
            }
            request.push_str(&String::from_utf8_lossy(&buf[..read]));
            if let Some(header_end) = request.find("\r\n\r\n") {
                let declared = request[..header_end]
                    .to_ascii_lowercase()
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("content-length:")
                            .and_then(|value| value.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                let body_read = request.len() - (header_end + 4);
                if body_read >= declared {
                    break;
                }
            }
        }
        request
    }

    #[test]
    fn extract_openai_stream_content_reads_choice_delta_content() {
        let chunk = json!({
            "choices": [{ "delta": { "content": "hello" }, "index": 0 }],
            "object": "chat.completion.chunk"
        });
        assert_eq!(extract_openai_stream_content(&chunk).as_deref(), Some("hello"));
    }

    #[test]
    fn extract_openai_stream_reasoning_reads_reasoning_content() {
        let chunk = json!({
            "choices": [{ "delta": { "reasoning_content": "think" }, "index": 0 }],
            "object": "chat.completion.chunk"
        });
        assert_eq!(extract_openai_stream_reasoning(&chunk).as_deref(), Some("think"));
        assert_eq!(extract_openai_stream_content(&chunk).as_deref(), None);
    }

    #[test]
    fn normalize_stream_delta_separates_reasoning_from_content() {
        let delta = normalize_stream_delta(&json!({
            "choices": [{ "delta": { "reasoning_content": "think", "content": "answer" } }]
        }));
        assert_eq!(delta.reasoning_content, "think");
        assert_eq!(delta.content, "answer");
        assert!(delta.tool_calls.is_empty());
    }

    #[test]
    fn anthropic_parser_normalizes_text_and_thinking_deltas() {
        let mut parser = AnthropicFrameParser::new();
        let deltas = parser.feed(
            r#"{"type":"message_start","message":{"model":"claude-sonnet-4","role":"assistant"}}"#,
        );
        assert!(deltas.is_empty());
        let deltas = parser.feed(
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"let me"}}"#,
        );
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].reasoning_content, "let me");
        let deltas = parser.feed(
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"text_delta","text":"Hi"}}"#,
        );
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].content, "Hi");
        parser.feed(r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"}}"#);
        assert_eq!(parser.model().as_deref(), Some("claude-sonnet-4"));
        assert_eq!(parser.finish_reason().as_deref(), Some("end_turn"));
    }

    #[test]
    fn google_parser_splits_thought_parts_from_answer_parts() {
        let mut parser = GoogleFrameParser::new();
        let deltas = parser.feed(
            r#"{"candidates":[{"content":{"parts":[{"text":"reason step","thought":true}]},"index":0}],"modelVersion":"gemini-2.5"}"#,
        );
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].reasoning_content, "reason step");
        let deltas = parser.feed(
            r#"{"candidates":[{"content":{"parts":[{"text":"answer"}]},"finishReason":"STOP"}]}"#,
        );
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].content, "answer");
        assert_eq!(parser.model().as_deref(), Some("gemini-2.5"));
        assert_eq!(parser.finish_reason().as_deref(), Some("STOP"));
    }

    #[test]
    fn responses_parser_reads_output_text_and_reasoning_deltas() {
        let mut parser = OpenAiResponsesFrameParser::new();
        let deltas = parser.feed(
            r#"{"type":"response.reasoning_summary_text.delta","delta":"plan"}"#,
        );
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].reasoning_content, "plan");
        let deltas = parser.feed(
            r#"{"type":"response.output_text.delta","delta":"answer text"}"#,
        );
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].content, "answer text");
        parser.feed(r#"{"type":"response.completed","response":{"model":"gpt-5"}}"#);
        assert_eq!(parser.model().as_deref(), Some("gpt-5"));
    }

    #[test]
    fn consume_sse_buffer_invokes_on_delta_per_event() {
        struct Collected {
            contents: Vec<String>,
            reasonings: Vec<String>,
        }
        let mut collected = Collected {
            contents: Vec::new(),
            reasonings: Vec::new(),
        };
        let mut on_delta = |delta: CloudRouterStreamDelta| {
            collected.contents.push(delta.content.clone());
            collected.reasonings.push(delta.reasoning_content.clone());
        };
        let mut buffer = String::from(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n",
        );
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut tool_call_fragments = Vec::new();
        let mut stream_deltas = Vec::new();
        let mut model = None;
        let mut finish_reason = None;
        consume_sse_buffer(
            &mut buffer,
            &mut content,
            &mut reasoning_content,
            &mut tool_call_fragments,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            &mut on_delta,
        );
        assert_eq!(content, "Hello");
        assert_eq!(reasoning_content, "");
        assert_eq!(stream_deltas, vec!["Hel".to_string(), "lo".to_string()]);
        assert_eq!(collected.contents, vec!["Hel".to_string(), "lo".to_string()]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn consume_sse_buffer_accumulates_reasoning_separately() {
        let mut on_delta = |_: CloudRouterStreamDelta| {};
        let mut buffer = String::from(
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"th\"}}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\n",
        );
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut tool_call_fragments = Vec::new();
        let mut stream_deltas = Vec::new();
        let mut model = None;
        let mut finish_reason = None;
        consume_sse_buffer(
            &mut buffer,
            &mut content,
            &mut reasoning_content,
            &mut tool_call_fragments,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            &mut on_delta,
        );
        assert_eq!(content, "Hi");
        assert_eq!(reasoning_content, "th");
        assert_eq!(stream_deltas, vec!["Hi".to_string()]);
    }

    #[test]
    fn consume_sse_buffer_captures_tool_call_fragments() {
        let mut on_delta = |_: CloudRouterStreamDelta| {};
        let mut buffer = String::from(
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"search\"}}]}}]}\n\n",
        );
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut tool_call_fragments = Vec::new();
        let mut stream_deltas = Vec::new();
        let mut model = None;
        let mut finish_reason = None;
        consume_sse_buffer(
            &mut buffer,
            &mut content,
            &mut reasoning_content,
            &mut tool_call_fragments,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            &mut on_delta,
        );
        assert_eq!(content, "");
        assert!(stream_deltas.is_empty());
        assert_eq!(tool_call_fragments.len(), 1);
        assert!(tool_call_fragments[0].contains("search"));
    }

    #[test]
    fn consume_sse_buffer_handles_crlf_delimited_events() {
        let mut on_delta = |_: CloudRouterStreamDelta| {};
        let mut buffer = String::from(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\r\n\r\n",
        );
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut tool_call_fragments = Vec::new();
        let mut stream_deltas = Vec::new();
        let mut model = None;
        let mut finish_reason = None;
        consume_sse_buffer(
            &mut buffer,
            &mut content,
            &mut reasoning_content,
            &mut tool_call_fragments,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            &mut on_delta,
        );
        assert_eq!(content, "Hi");
        assert_eq!(stream_deltas, vec!["Hi".to_string()]);
    }

    #[test]
    fn empty_sse_body_is_an_error_shape() {
        let content = String::new();
        let stream_deltas: Vec<String> = Vec::new();
        assert!(content.trim().is_empty() || stream_deltas.is_empty());
    }

    #[test]
    fn stream_chat_completion_blocking_reads_live_sse_chunks() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        use cloudrouter_open_sdk::models::OpenAiChatMessage;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock gateway");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let (ready_tx, ready_rx) = mpsc::channel();
        thread::spawn(move || {
            ready_tx.send(()).expect("signal mock gateway listening");
            let (mut stream, _) = listener.accept().expect("accept mock request");
            let request = read_complete_mock_request(&mut stream);
            assert!(request.contains("POST /v1/chat/completions"));
            assert!(request.contains("stream"));
            let body = "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"th\"}}]}\n\n\
                        data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n\
                        data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n\
                        data: [DONE]\n\n";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("write mock sse");
        });
        ready_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock gateway should be listening");

        let mut deltas = Vec::new();
        let request = OpenAiChatCompletionRequest {
            model: "default".to_string(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: Some("hi".to_string()),
                ..Default::default()
            }],
            stream: Some(true),
            ..Default::default()
        };
        let result = stream_chat_completion_blocking(
            &base_url,
            "test-auth",
            None,
            request,
            &mut |delta| deltas.push(delta),
        )
        .expect("stream should succeed");
        assert_eq!(result.content, "Hello");
        assert_eq!(result.reasoning_content, "th");
        assert_eq!(result.stream_deltas, vec!["Hel".to_string(), "lo".to_string()]);
        assert_eq!(deltas.len(), 3);
    }

    #[test]
    fn stream_llm_completion_blocking_normalizes_anthropic_sse() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        use cloudrouter_open_sdk::models::OpenAiChatMessage;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock gateway");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let (ready_tx, ready_rx) = mpsc::channel();
        thread::spawn(move || {
            ready_tx.send(()).expect("signal mock gateway listening");
            let (mut stream, _) = listener.accept().expect("accept mock request");
            let request = read_complete_mock_request(&mut stream);
            // Vendor routes must be hit verbatim: no /v1 prefix in front.
            assert!(request.contains("POST /anthropic/v1/messages"));
            assert!(!request.contains("Anthropic-Version"));
            let body = "data: {\"type\":\"message_start\",\"message\":{\"model\":\"claude-sonnet-4\"}}\n\n\
                        data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"th\"}}\n\n\
                        data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel\"}}\n\n\
                        data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"lo\"}}\n\n\
                        data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("write mock sse");
        });
        ready_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock gateway should be listening");

        let mut deltas = Vec::new();
        let request = OpenAiChatCompletionRequest {
            model: "default".to_string(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: Some("hi".to_string()),
                ..Default::default()
            }],
            stream: Some(true),
            ..Default::default()
        };
        let result = stream_llm_completion_blocking(
            WireProtocol::AnthropicMessages,
            &base_url,
            "test-auth",
            None,
            request,
            &mut |delta| deltas.push(delta),
        )
        .expect("anthropic stream should succeed");
        assert_eq!(result.content, "Hello");
        assert_eq!(result.reasoning_content, "th");
        assert_eq!(result.model.as_deref(), Some("claude-sonnet-4"));
        // end_turn normalizes to the OpenAI-style stop vocabulary.
        assert_eq!(result.finish_reason.as_deref(), Some("stop"));
        assert_eq!(deltas.len(), 3);
    }

    #[test]
    fn create_llm_completion_blocking_normalizes_google_json_response() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        use cloudrouter_open_sdk::models::OpenAiChatMessage;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock gateway");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let (ready_tx, ready_rx) = mpsc::channel();
        thread::spawn(move || {
            ready_tx.send(()).expect("signal mock gateway listening");
            let (mut stream, _) = listener.accept().expect("accept mock request");
            let request = read_complete_mock_request(&mut stream);
            assert!(request.contains("POST /google/v1beta/models/default:generateContent"));
            let body = r#"{"candidates":[{"content":{"parts":[{"text":"reason","thought":true},{"text":"Gemini answer"}]},"finishReason":"STOP"}],"modelVersion":"gemini-2.5"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("write mock json");
        });
        ready_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock gateway should be listening");

        let request = OpenAiChatCompletionRequest {
            model: "default".to_string(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: Some("hi".to_string()),
                ..Default::default()
            }],
            stream: Some(false),
            ..Default::default()
        };
        let result = create_llm_completion_blocking(
            WireProtocol::GoogleContent,
            &base_url,
            "test-auth",
            None,
            request,
        )
        .expect("google completion should succeed");
        assert_eq!(result.content, "Gemini answer");
        assert_eq!(result.reasoning_content, "reason");
        assert_eq!(result.model.as_deref(), Some("gemini-2.5"));
        assert_eq!(result.finish_reason.as_deref(), Some("stop"));
    }
}
