//! Wire protocol definitions for the four supported LLM gateway APIs.
//!
//! The Cloud Router gateway exposes every provider under a single
//! account-pool-routed, metered open-api namespace:
//!
//! - [`WireProtocol::ChatCompletions`] -> `POST /v1/chat/completions`
//! - [`WireProtocol::AnthropicMessages`] -> `POST /anthropic/v1/messages`
//! - [`WireProtocol::GoogleContent`] -> `POST /google/v1beta/models/{model}:generateContent`
//!   (streaming variant: `:streamGenerateContent?alt=sse`)
//! - [`WireProtocol::OpenAiResponses`] -> `POST /v1/responses`
//!
//! All protocols accept the same normalized turn conversation (system
//! prompts + user/assistant history) and are converted to their native
//! request shape here, so turn execution stays protocol-agnostic above
//! this module.

use cloudrouter_open_sdk::models::OpenAiChatCompletionRequest;
use serde_json::{json, Value};

/// Default ceiling for protocols that require an explicit output budget.
///
/// Anthropic mandates `max_tokens`; Google and OpenAI chat completions use
/// provider defaults when unset. The value mirrors a typical playground
/// conversation budget without silently truncating agent answers.
pub const DEFAULT_MAX_TOKENS: i64 = 8192;

/// The LLM wire protocol used for one gateway invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireProtocol {
    /// OpenAI Chat Completions (`/v1/chat/completions`). Default protocol.
    ChatCompletions,
    /// Anthropic Messages (`/anthropic/v1/messages`).
    AnthropicMessages,
    /// Google Gemini Content (`/google/v1beta/models/{model}:generateContent`).
    GoogleContent,
    /// OpenAI Responses (`/v1/responses`).
    OpenAiResponses,
}

impl WireProtocol {
    /// Parses the canonical protocol identifier. Unknown values fail closed
    /// so a typo can never silently downgrade to a different provider API.
    pub fn parse(value: &str) -> Option<WireProtocol> {
        match value.trim() {
            "" | "chat_completions" => Some(WireProtocol::ChatCompletions),
            "anthropic_messages" => Some(WireProtocol::AnthropicMessages),
            "google_content" => Some(WireProtocol::GoogleContent),
            "openai_responses" => Some(WireProtocol::OpenAiResponses),
            _ => None,
        }
    }

    /// Canonical identifier used on the wire and in durable records.
    pub fn as_str(&self) -> &'static str {
        match self {
            WireProtocol::ChatCompletions => "chat_completions",
            WireProtocol::AnthropicMessages => "anthropic_messages",
            WireProtocol::GoogleContent => "google_content",
            WireProtocol::OpenAiResponses => "openai_responses",
        }
    }

    /// Gateway endpoint path for the non-streaming invocation.
    pub fn endpoint(&self, model: &str) -> String {
        match self {
            WireProtocol::ChatCompletions => "/v1/chat/completions".to_string(),
            WireProtocol::AnthropicMessages => "/anthropic/v1/messages".to_string(),
            // The model key is percent-encoded by the caller-safe callers via
            // serde URL composition; gateway model keys are restricted to
            // URL-safe characters ([a-zA-Z0-9._:-]) so direct interpolation is
            // safe here.
            WireProtocol::GoogleContent => {
                format!("/google/v1beta/models/{model}:generateContent")
            }
            WireProtocol::OpenAiResponses => "/v1/responses".to_string(),
        }
    }

    /// Gateway endpoint path for the streaming invocation. Google requires
    /// `alt=sse` to emit newline-delimited SSE frames instead of a single
    /// JSON array document.
    pub fn streaming_endpoint(&self, model: &str) -> String {
        match self {
            WireProtocol::GoogleContent => {
                format!("/google/v1beta/models/{model}:streamGenerateContent?alt=sse")
            }
            other => other.endpoint(model),
        }
    }
}

impl Default for WireProtocol {
    fn default() -> Self {
        WireProtocol::ChatCompletions
    }
}

/// Splits the OpenAI-style conversation carried by the chat completion
/// request into system prompts and user/assistant history so each protocol
/// converter can map them to their native shape.
fn split_conversation(
    request: &OpenAiChatCompletionRequest,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut system_prompts: Vec<String> = Vec::new();
    let mut messages: Vec<(String, String)> = Vec::with_capacity(request.messages.len());
    for message in &request.messages {
        let Some(content) = message.content.as_deref().map(str::trim) else {
            continue;
        };
        if content.is_empty() {
            continue;
        }
        if message.role == "system" {
            system_prompts.push(content.to_string());
        } else {
            messages.push((message.role.clone(), content.to_string()));
        }
    }
    (system_prompts, messages)
}

fn model_key(request: &OpenAiChatCompletionRequest) -> String {
    request
        .model
        .trim()
        .to_string()
}

fn optional_f64(value: Option<f64>) -> Value {
    match value {
        Some(value) => json!(value),
        None => Value::Null,
    }
}

fn insert_if_present(payload: &mut Value, key: &str, value: Value) {
    if !value.is_null() {
        payload[key] = value;
    }
}

/// Builds the native JSON request body for the given protocol from the
/// normalized chat completion conversation. Streaming only toggles the
/// protocol's own `stream` field; the transport (SSE vs JSON) is chosen by
/// the endpoint.
pub fn build_protocol_request_body(
    protocol: WireProtocol,
    request: &OpenAiChatCompletionRequest,
    stream: bool,
) -> Value {
    let (system_prompts, messages) = split_conversation(request);
    let system_text = system_prompts.join("\n\n");
    let model = model_key(request);
    let max_tokens = request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
    match protocol {
        WireProtocol::ChatCompletions => {
            // System prompts lead the message list for chat completions.
            let mut all_messages = system_prompts
                .iter()
                .map(|prompt| json!({"role": "system", "content": prompt}))
                .collect::<Vec<_>>();
            all_messages.extend(messages.iter().map(|(role, content)| {
                json!({"role": role, "content": content})
            }));
            let mut body = json!({
                "model": model,
                "messages": all_messages,
                "stream": stream,
            });
            insert_if_present(&mut body, "temperature", optional_f64(request.temperature));
            insert_if_present(&mut body, "max_tokens", json!(request.max_tokens));
            insert_if_present(&mut body, "stop", json!(request.stop));
            body
        }
        WireProtocol::AnthropicMessages => {
            let mut body = json!({
                "model": model,
                // Anthropic mandates an explicit output budget.
                "max_tokens": max_tokens,
                "messages": messages
                    .iter()
                    .map(|(role, content)| json!({"role": role, "content": content}))
                    .collect::<Vec<_>>(),
                "stream": stream,
            });
            if !system_text.is_empty() {
                body["system"] = json!(system_text);
            }
            insert_if_present(&mut body, "temperature", optional_f64(request.temperature));
            // The normalized request carries stop as a single string; Anthropic
            // expects an array of stop sequences.
            if let Some(stop) = request.stop.as_deref().filter(|stop| !stop.trim().is_empty()) {
                body["stop_sequences"] = json!([stop]);
            }
            body
        }
        WireProtocol::GoogleContent => {
            let mut body = json!({
                "contents": messages
                    .iter()
                    .map(|(role, content)| {
                        let role = if role == "assistant" { "model" } else { role.as_str() };
                        json!({"role": role, "parts": [{"text": content}]})
                    })
                    .collect::<Vec<_>>(),
            });
            if !system_text.is_empty() {
                body["systemInstruction"] = json!({"parts": [{"text": system_text}]});
            }
            let mut generation_config = serde_json::Map::new();
            if let Some(temperature) = request.temperature {
                generation_config.insert("temperature".to_string(), json!(temperature));
            }
            if let Some(max_tokens) = request.max_tokens {
                generation_config.insert("maxOutputTokens".to_string(), json!(max_tokens));
            }
            if let Some(stop) = request.stop.as_deref().filter(|stop| !stop.trim().is_empty()) {
                generation_config.insert("stopSequences".to_string(), json!([stop]));
            }
            if !generation_config.is_empty() {
                body["generationConfig"] = Value::Object(generation_config);
            }
            body
        }
        WireProtocol::OpenAiResponses => {
            let mut body = json!({
                "model": model,
                "input": messages
                    .iter()
                    .map(|(role, content)| {
                        let content_type = if role == "assistant" {
                            "output_text"
                        } else {
                            "input_text"
                        };
                        json!({
                            "role": role,
                            "content": [{"type": content_type, "text": content}],
                        })
                    })
                    .collect::<Vec<_>>(),
                "stream": stream,
            });
            if !system_text.is_empty() {
                body["instructions"] = json!(system_text);
            }
            insert_if_present(&mut body, "temperature", optional_f64(request.temperature));
            insert_if_present(&mut body, "max_output_tokens", json!(request.max_tokens));
            body
        }
    }
}

/// Normalizes a protocol-native stop reason into the OpenAI-style lowercase
/// vocabulary (`stop` / `length`) so turn persistence stays protocol-agnostic.
pub fn normalize_finish_reason(protocol: WireProtocol, raw: &str) -> Option<String> {
    let normalized = match protocol {
        WireProtocol::ChatCompletions | WireProtocol::OpenAiResponses => match raw {
            "stop" | "end_turn" | "stop_sequence" => Some("stop"),
            "length" | "max_tokens" | "max_output_tokens" => Some("length"),
            "tool_calls" | "tool_use" => Some("tool_calls"),
            _ => None,
        },
        WireProtocol::AnthropicMessages => match raw {
            "end_turn" | "stop_sequence" => Some("stop"),
            "max_tokens" => Some("length"),
            "tool_use" => Some("tool_calls"),
            _ => None,
        },
        WireProtocol::GoogleContent => match raw {
            "STOP" => Some("stop"),
            "MAX_TOKENS" => Some("length"),
            "SAFETY" | "RECITATION" => Some("content_filter"),
            _ => None,
        },
    };
    normalized.map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudrouter_open_sdk::models::OpenAiChatMessage;

    fn sample_request() -> OpenAiChatCompletionRequest {
        OpenAiChatCompletionRequest {
            model: "default".to_string(),
            messages: vec![
                OpenAiChatMessage {
                    role: "system".to_string(),
                    content: Some("You are helpful.".to_string()),
                    ..Default::default()
                },
                OpenAiChatMessage {
                    role: "user".to_string(),
                    content: Some("hi".to_string()),
                    ..Default::default()
                },
                OpenAiChatMessage {
                    role: "assistant".to_string(),
                    content: Some("hello".to_string()),
                    ..Default::default()
                },
                OpenAiChatMessage {
                    role: "user".to_string(),
                    content: Some("how are you".to_string()),
                    ..Default::default()
                },
            ],
            stream: Some(false),
            ..Default::default()
        }
    }

    #[test]
    fn parse_accepts_known_protocol_ids_and_blank_default() {
        assert_eq!(
            WireProtocol::parse("chat_completions"),
            Some(WireProtocol::ChatCompletions)
        );
        assert_eq!(
            WireProtocol::parse("anthropic_messages"),
            Some(WireProtocol::AnthropicMessages)
        );
        assert_eq!(
            WireProtocol::parse("google_content"),
            Some(WireProtocol::GoogleContent)
        );
        assert_eq!(
            WireProtocol::parse("openai_responses"),
            Some(WireProtocol::OpenAiResponses)
        );
        assert_eq!(WireProtocol::parse(""), Some(WireProtocol::ChatCompletions));
        assert_eq!(WireProtocol::parse("bogus"), None);
    }

    #[test]
    fn endpoints_use_verbatim_vendor_paths() {
        assert_eq!(
            WireProtocol::ChatCompletions.endpoint("m"),
            "/v1/chat/completions"
        );
        assert_eq!(
            WireProtocol::AnthropicMessages.endpoint("m"),
            "/anthropic/v1/messages"
        );
        assert_eq!(
            WireProtocol::GoogleContent.endpoint("gemini-2.5"),
            "/google/v1beta/models/gemini-2.5:generateContent"
        );
        assert_eq!(
            WireProtocol::GoogleContent.streaming_endpoint("gemini-2.5"),
            "/google/v1beta/models/gemini-2.5:streamGenerateContent?alt=sse"
        );
        assert_eq!(WireProtocol::OpenAiResponses.endpoint("m"), "/v1/responses");
    }

    #[test]
    fn anthropic_body_carries_system_and_mandatory_max_tokens() {
        let body = build_protocol_request_body(WireProtocol::AnthropicMessages, &sample_request(), true);
        assert_eq!(body["model"], "default");
        assert_eq!(body["max_tokens"], DEFAULT_MAX_TOKENS);
        assert_eq!(body["system"], "You are helpful.");
        assert_eq!(body["stream"], true);
        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[2]["content"], "how are you");
    }

    #[test]
    fn google_body_maps_assistant_to_model_and_system_instruction() {
        let body = build_protocol_request_body(WireProtocol::GoogleContent, &sample_request(), true);
        let contents = body["contents"].as_array().expect("contents array");
        assert_eq!(contents.len(), 3);
        assert_eq!(contents[1]["role"], "model");
        assert_eq!(
            body["systemInstruction"]["parts"][0]["text"],
            "You are helpful."
        );
    }

    #[test]
    fn responses_body_uses_typed_content_parts_and_instructions() {
        let body = build_protocol_request_body(WireProtocol::OpenAiResponses, &sample_request(), false);
        let input = body["input"].as_array().expect("input array");
        assert_eq!(input.len(), 3);
        assert_eq!(input[0]["content"][0]["type"], "input_text");
        assert_eq!(input[1]["content"][0]["type"], "output_text");
        assert_eq!(body["instructions"], "You are helpful.");
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn chat_completions_body_keeps_openai_shape_with_leading_system() {
        let body =
            build_protocol_request_body(WireProtocol::ChatCompletions, &sample_request(), true);
        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn finish_reason_maps_native_vocabulary_to_openai_style() {
        assert_eq!(
            normalize_finish_reason(WireProtocol::AnthropicMessages, "end_turn").as_deref(),
            Some("stop")
        );
        assert_eq!(
            normalize_finish_reason(WireProtocol::AnthropicMessages, "max_tokens").as_deref(),
            Some("length")
        );
        assert_eq!(
            normalize_finish_reason(WireProtocol::GoogleContent, "STOP").as_deref(),
            Some("stop")
        );
        assert_eq!(
            normalize_finish_reason(WireProtocol::GoogleContent, "MAX_TOKENS").as_deref(),
            Some("length")
        );
        assert_eq!(
            normalize_finish_reason(WireProtocol::ChatCompletions, "stop").as_deref(),
            Some("stop")
        );
        assert_eq!(normalize_finish_reason(WireProtocol::GoogleContent, "WEIRD"), None);
    }
}
