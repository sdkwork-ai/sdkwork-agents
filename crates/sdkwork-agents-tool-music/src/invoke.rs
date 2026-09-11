//! Invocation behaviour for the music category tools.

use cloudrouter_open_sdk::models::SunoMusicGenerationRequest;
use sdkwork_agents_tool_cloudrouter::{run_sync, CloudRouterMediaClient};
use sdkwork_agents_tool_contract::{MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::tool_ids;

/// Executes one music tool call against the cloudrouter gateway.
pub fn invoke_music_tool(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    match call.tool_id.as_str() {
        tool_ids::GENERATIONS_CREATE => invoke_generations_create(call, auth_token),
        tool_ids::GENERATIONS_LIST => invoke_generations_list(call, auth_token),
        other => Err(MediaToolError::CapabilityMissing(format!(
            "music provider has no tool `{other}`"
        ))),
    }
}

fn invoke_generations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = SunoMusicGenerationRequest {
        prompt: call.string_arg("prompt")?,
        model: call
            .optional_string_arg("model")
            .filter(|value| !value.trim().is_empty())
            .or_else(|| Some("default".to_string())),
        title: call.optional_string_arg("title"),
        tags: call.optional_string_arg("tags"),
        duration: call.optional_number_arg("duration"),
        negative_tags: call.optional_string_arg("negativeTags"),
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let response = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio_suno().create_v1_music_generation(&request))
    })?;

    let task_id = response.task_id.or(response.id).ok_or_else(|| {
        MediaToolError::ProviderError(
            "cloudrouter music generation returned no task id".to_string(),
        )
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_generations_list(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let task_id = call.string_arg("taskId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio_suno().list_v1_music_generations(&task_id))
    })?;

    let status = normalize_music_status(task.status.as_deref());
    let mut output = serde_json::json!({
        "taskId": task_id,
        "status": status,
    });

    if let Some(tracks) = task.tracks {
        let items: Vec<serde_json::Value> = tracks
            .iter()
            .filter_map(|track| {
                track.audio_url.as_ref().map(|url| {
                    serde_json::json!({
                        "kind": "music",
                        "source": "provider_asset",
                        "url": url,
                        "title": track.title,
                        "duration": track.duration,
                    })
                })
            })
            .collect();
        output["tracks"] = serde_json::json!(items);
    }

    if let Some(error) = task
        .error
        .as_ref()
        .and_then(|provider_error| provider_error.message.clone())
    {
        output["error"] = serde_json::json!(error);
    }

    Ok(MediaToolResult::succeeded(&call.tool_call_id, output))
}

/// Maps provider status strings to the stable task status vocabulary.
fn normalize_music_status(status: Option<&str>) -> &'static str {
    match status.unwrap_or_default().to_ascii_lowercase().as_str() {
        "queued" | "pending" => "queued",
        "processing" | "running" | "in_progress" => "processing",
        "completed" | "succeeded" | "success" => "completed",
        "failed" | "error" | "cancelled" => "failed",
        _ => "processing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_provider_statuses() {
        assert_eq!(normalize_music_status(Some("queued")), "queued");
        assert_eq!(normalize_music_status(Some("running")), "processing");
        assert_eq!(normalize_music_status(Some("completed")), "completed");
        assert_eq!(normalize_music_status(Some("failed")), "failed");
        assert_eq!(normalize_music_status(None), "processing");
    }

    #[test]
    fn unknown_tool_id_returns_capability_missing() {
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "music.not.a.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_music_tool(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn create_requires_auth_token_and_prompt() {
        let no_token = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: tool_ids::GENERATIONS_CREATE.to_string(),
            arguments: serde_json::json!({ "prompt": "lofi beats" }),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_music_tool(&no_token, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");

        let no_prompt = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::GENERATIONS_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_music_tool(&no_prompt, Some("token")).expect_err("prompt required");
        assert_eq!(error.code(), "invalid_input");
    }
}
