//! Shared helpers for vendor-direct (non-OpenAI) generation tools.
//!
//! Midjourney, Nano Banana, Kling, Vidu, and Volcengine surface async task
//! APIs with provider-specific status strings and generated-media payloads.
//! These helpers normalize those payloads into the media tool contract shapes
//! so category crates share one implementation.

use cloudrouter_open_sdk::models::{ProviderGeneratedMedia, ProviderTaskError};
use sdkwork_agents_tool_contract::{MediaToolCall, MediaToolError, MediaToolResult};

/// Maps a vendor task status string to the stable task status vocabulary.
pub fn normalize_vendor_status(status: Option<&str>) -> &'static str {
    match status.unwrap_or_default().to_ascii_lowercase().as_str() {
        "queued" | "pending" | "submitted" => "queued",
        "processing" | "running" | "in_progress" | "working" => "processing",
        "completed" | "succeeded" | "success" | "finished" => "completed",
        "failed" | "error" | "cancelled" => "failed",
        _ => "processing",
    }
}

/// Normalizes vendor `ProviderGeneratedMedia` items into `MediaResource`
/// shape, preserving `url`/`uri` delivery fields and the provider error.
pub fn normalized_vendor_media(
    task_id: &str,
    status: &str,
    items: Vec<ProviderGeneratedMedia>,
    kind: &str,
    error: Option<&ProviderTaskError>,
) -> MediaToolResult {
    let normalized: Vec<serde_json::Value> = items
        .iter()
        .filter_map(|item| {
            let url = item.url.as_ref().or(item.uri.as_ref())?.trim().to_string();
            if url.is_empty() {
                return None;
            }
            Some(serde_json::json!({
                "kind": kind,
                "source": "provider_asset",
                "url": url,
                "uri": item.uri,
            }))
        })
        .collect();

    let mut output = serde_json::json!({
        "taskId": task_id,
        "status": status,
        "items": normalized,
    });
    if let Some(error) = error.and_then(|provider_error| provider_error.message.clone()) {
        output["error"] = serde_json::json!(error);
    }
    MediaToolResult::succeeded(task_id.to_string(), output)
}

/// Reads an optional array-of-strings argument.
pub fn string_array_arg(
    call: &MediaToolCall,
    name: &str,
) -> Result<Option<Vec<String>>, MediaToolError> {
    match call.arguments.get(name) {
        None => Ok(None),
        Some(serde_json::Value::Array(items)) => {
            let mut strings = Vec::with_capacity(items.len());
            for item in items {
                let value = item.as_str().ok_or_else(|| {
                    MediaToolError::invalid_argument(format!(
                        "`{name}` must be an array of strings for tool `{}`",
                        call.tool_id
                    ))
                })?;
                strings.push(value.to_string());
            }
            Ok(Some(strings))
        }
        Some(_) => Err(MediaToolError::invalid_argument(format!(
            "`{name}` must be an array of strings for tool `{}`",
            call.tool_id
        ))),
    }
}

/// Reads an optional i64 argument.
pub fn optional_i64_arg(call: &MediaToolCall, name: &str) -> Option<i64> {
    call.optional_number_arg(name).map(|value| value as i64)
}

/// Reads an optional string argument defaulting to `default` when empty.
pub fn model_arg(call: &MediaToolCall, name: &str) -> String {
    call.optional_string_arg(name)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "default".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agents_tool_contract::MediaToolCall;

    fn call(arguments: serde_json::Value) -> MediaToolCall {
        MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "image.midjourney.generations.create".to_string(),
            arguments,
            session_id: None,
            trace_id: None,
        }
    }

    #[test]
    fn normalizes_vendor_statuses() {
        assert_eq!(normalize_vendor_status(Some("submitted")), "queued");
        assert_eq!(normalize_vendor_status(Some("working")), "processing");
        assert_eq!(normalize_vendor_status(Some("finished")), "completed");
        assert_eq!(normalize_vendor_status(Some("error")), "failed");
        assert_eq!(normalize_vendor_status(None), "processing");
    }

    #[test]
    fn vendor_media_normalization_uses_url_or_uri() {
        let items = vec![
            ProviderGeneratedMedia {
                url: Some("https://cdn.example/a.png".to_string()),
                uri: None,
                ..Default::default()
            },
            ProviderGeneratedMedia {
                url: None,
                uri: Some("https://cdn.example/b.png".to_string()),
                ..Default::default()
            },
            ProviderGeneratedMedia {
                url: None,
                uri: None,
                ..Default::default()
            },
        ];
        let result = normalized_vendor_media("task.1", "completed", items, "image", None);
        let items = result.output["items"].as_array().expect("array");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["url"], "https://cdn.example/a.png");
        assert_eq!(items[1]["url"], "https://cdn.example/b.png");
        assert_eq!(result.output["status"], "completed");
    }

    #[test]
    fn string_array_arg_parses_arrays_only() {
        let parsed = string_array_arg(&call(serde_json::json!({"images": ["a", "b"]})), "images")
            .expect("array parsed");
        assert_eq!(parsed, Some(vec!["a".to_string(), "b".to_string()]));

        let missing = string_array_arg(&call(serde_json::json!({})), "images").expect("none");
        assert_eq!(missing, None);

        assert!(string_array_arg(&call(serde_json::json!({"images": "a"})), "images").is_err());
    }

    #[test]
    fn model_arg_defaults_to_default_key() {
        assert_eq!(model_arg(&call(serde_json::json!({})), "model"), "default");
        assert_eq!(
            model_arg(&call(serde_json::json!({"model": "midjourney-6"})), "model"),
            "midjourney-6"
        );
        assert_eq!(
            model_arg(&call(serde_json::json!({"model": "  "})), "model"),
            "default"
        );
    }
}
