//! Media tool call and result shapes.
//!
//! The call carries the caller auth token separately from `arguments` so the
//! token never enters model-visible tool arguments or telemetry redaction
//! surfaces.

use serde::{Deserialize, Serialize};

use crate::definition::MediaResource;

/// One media tool invocation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolCall {
    /// Stable call id used for idempotency and audit linkage.
    pub tool_call_id: String,
    /// Tool id resolved against the registry, e.g. `audio.speech.create`.
    pub tool_id: String,
    /// Tool arguments (JSON Schema draft 2020-12 validated by the provider).
    pub arguments: serde_json::Value,
    /// Owning session id when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl MediaToolCall {
    /// Reads a required string argument, returning `Err` for missing or
    /// non-string values.
    pub fn string_arg(&self, name: &str) -> Result<String, crate::MediaToolError> {
        self.arguments
            .get(name)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                crate::MediaToolError::invalid_argument(format!(
                    "missing or non-string argument `{name}` for tool `{}`",
                    self.tool_id
                ))
            })
    }

    /// Reads an optional string argument.
    pub fn optional_string_arg(&self, name: &str) -> Option<String> {
        self.arguments
            .get(name)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }

    /// Reads an optional f64 argument.
    pub fn optional_number_arg(&self, name: &str) -> Option<f64> {
        self.arguments.get(name).and_then(serde_json::Value::as_f64)
    }
}

/// Result of one media tool invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolResult {
    /// Matches the originating `MediaToolCall::tool_call_id`.
    pub tool_call_id: String,
    /// Normalized status (`succeeded` | `failed` | `pending`).
    pub status: String,
    /// Normalized JSON output. For async task tools this is
    /// `{ "taskId": "..." }` until the poll tool completes the task.
    pub output: serde_json::Value,
    /// Optional error detail when `status == "failed"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl MediaToolResult {
    pub fn succeeded(tool_call_id: impl Into<String>, output: serde_json::Value) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            status: "succeeded".to_string(),
            output,
            error: None,
        }
    }

    pub fn failed(tool_call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            status: "failed".to_string(),
            output: serde_json::Value::Null,
            error: Some(error.into()),
        }
    }

    pub fn succeeded_with_resource(
        tool_call_id: impl Into<String>,
        resource: &MediaResource,
    ) -> Self {
        Self::succeeded(
            tool_call_id,
            serde_json::to_value(resource).unwrap_or(serde_json::Value::Null),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_arg_extracts_required_argument() {
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "audio.speech.create".to_string(),
            arguments: serde_json::json!({ "input": "hello", "voice": "alloy" }),
            session_id: None,
        };
        assert_eq!(call.string_arg("input").unwrap(), "hello");
        assert_eq!(call.optional_string_arg("voice").unwrap(), "alloy");
        assert!(call.optional_string_arg("missing").is_none());
        assert!(call.string_arg("missing").is_err());
    }

    #[test]
    fn optional_number_arg_extracts_numbers() {
        let call = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: "audio.speech.create".to_string(),
            arguments: serde_json::json!({ "speed": 1.25 }),
            session_id: None,
        };
        assert_eq!(call.optional_number_arg("speed"), Some(1.25));
        assert!(call.optional_number_arg("missing").is_none());
    }

    #[test]
    fn failed_result_carries_error_detail() {
        let result = MediaToolResult::failed("call.3", "upstream rejected");
        assert_eq!(result.status, "failed");
        assert_eq!(result.error.as_deref(), Some("upstream rejected"));
        assert_eq!(result.output, serde_json::Value::Null);
    }

    #[test]
    fn succeeded_with_resource_normalizes_output() {
        let resource = MediaResource::provider_asset("audio", "https://cdn.example/a.mp3");
        let result = MediaToolResult::succeeded_with_resource("call.4", &resource);
        assert_eq!(result.status, "succeeded");
        assert_eq!(result.output["url"], "https://cdn.example/a.mp3");
        assert_eq!(result.output["source"], "provider_asset");
    }
}
