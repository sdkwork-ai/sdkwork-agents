//! Invocation behaviour for the video category tools.

use cloudrouter_open_sdk::models::{
    KlingVideoGenerationRequest, OpenAiVideoCreateRequest, OpenAiVideoEditRequest,
    OpenAiVideoExtendRequest, OpenAiVideoRemixRequest, ViduImageToVideoRequest,
    ViduReferenceToVideoRequest, ViduStartEndToVideoRequest, ViduTextToVideoRequest,
    VolcengineContentGenerationTaskCreateRequest, VolcengineContentPart,
};
use sdkwork_agents_tool_cloudrouter::{
    model_arg, normalize_vendor_status, normalized_vendor_media, optional_i64_arg, run_sync,
    string_array_arg, CloudRouterMediaClient,
};
use sdkwork_agents_tool_contract::{MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::tool_ids;

/// Executes one video tool call against the cloudrouter gateway.
pub fn invoke_video_tool(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    match call.tool_id.as_str() {
        tool_ids::CREATE => invoke_create(call, auth_token),
        tool_ids::RETRIEVE => invoke_retrieve(call, auth_token),
        tool_ids::LIST => invoke_list(call, auth_token),
        tool_ids::EDITS_CREATE => invoke_edits_create(call, auth_token),
        tool_ids::EXTENSIONS_CREATE => invoke_extensions_create(call, auth_token),
        tool_ids::REMIX_CREATE => invoke_remix_create(call, auth_token),
        tool_ids::CHARACTERS_CREATE => invoke_characters_create(call, auth_token),
        tool_ids::CHARACTERS_LIST => invoke_characters_list(call, auth_token),
        tool_ids::KLING_GENERATIONS_CREATE => invoke_kling_generations_create(call, auth_token),
        tool_ids::KLING_GENERATIONS_RETRIEVE => invoke_kling_generations_retrieve(call, auth_token),
        tool_ids::VIDU_TEXT2VIDEO => invoke_vidu_text2video(call, auth_token),
        tool_ids::VIDU_IMG2VIDEO => invoke_vidu_img2video(call, auth_token),
        tool_ids::VIDU_REFERENCE2VIDEO => invoke_vidu_reference2video(call, auth_token),
        tool_ids::VIDU_START_END2VIDEO => invoke_vidu_start_end2video(call, auth_token),
        tool_ids::VIDU_TASKS_CREATIONS => invoke_vidu_tasks_creations(call, auth_token),
        tool_ids::VOLCENGINE_GENERATIONS_CREATE => {
            invoke_volcengine_generations_create(call, auth_token)
        }
        tool_ids::VOLCENGINE_GENERATIONS_RETRIEVE => {
            invoke_volcengine_generations_retrieve(call, auth_token)
        }
        other => Err(MediaToolError::CapabilityMissing(format!(
            "video provider has no tool `{other}`"
        ))),
    }
}

fn invoke_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiVideoCreateRequest {
        prompt: call.string_arg("prompt")?,
        model: call
            .optional_string_arg("model")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "default".to_string()),
        image: call.optional_string_arg("image"),
        video: call.optional_string_arg("video"),
        seconds: call
            .optional_number_arg("seconds")
            .map(|value| value as i64),
        size: call.optional_string_arg("size"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": video.id }),
    ))
}

fn invoke_retrieve(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let video_id = call.string_arg("videoId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().retrieve(&video_id))
    })?;

    let status = normalize_video_status(&video.status);
    let mut output = serde_json::json!({
        "taskId": video.id,
        "status": status,
    });
    if let Some(url) = video.url.filter(|value| !value.trim().is_empty()) {
        output["url"] = serde_json::json!(url);
        output["kind"] = serde_json::json!("video");
        output["source"] = serde_json::json!("provider_asset");
    }
    if let Some(content_url) = video.content_url.filter(|value| !value.trim().is_empty()) {
        output["contentUrl"] = serde_json::json!(content_url);
    }

    Ok(MediaToolResult::succeeded(&call.tool_call_id, output))
}

fn invoke_list(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let limit = call.optional_number_arg("limit").map(|value| value as i64);

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let videos = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().list(limit, None, None, None))
    })?;

    let items: Vec<serde_json::Value> = videos
        .data
        .iter()
        .map(|video| {
            serde_json::json!({
                "id": video.id,
                "status": normalize_video_status(&video.status),
                "url": video.url,
            })
        })
        .collect();

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "videos": items }),
    ))
}

fn invoke_edits_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiVideoEditRequest {
        prompt: Some(call.string_arg("prompt")?),
        model: call.optional_string_arg("model"),
        image: call.optional_string_arg("image"),
        video: call.optional_string_arg("video"),
        seconds: call
            .optional_number_arg("seconds")
            .map(|value| value as i64),
        size: call.optional_string_arg("size"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create_edit(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": video.id }),
    ))
}

fn invoke_extensions_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiVideoExtendRequest {
        prompt: Some(call.string_arg("prompt")?),
        model: call.optional_string_arg("model"),
        image: call.optional_string_arg("image"),
        video: call.optional_string_arg("video"),
        seconds: call
            .optional_number_arg("seconds")
            .map(|value| value as i64),
        size: call.optional_string_arg("size"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create_extension(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": video.id }),
    ))
}

fn invoke_remix_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let video_id = call.string_arg("videoId")?;
    let request = OpenAiVideoRemixRequest {
        prompt: Some(call.string_arg("prompt")?),
        model: call.optional_string_arg("model"),
        image: call.optional_string_arg("image"),
        video: call.optional_string_arg("video"),
        seconds: call
            .optional_number_arg("seconds")
            .map(|value| value as i64),
        size: call.optional_string_arg("size"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create_remix(&video_id, &request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": video.id }),
    ))
}

fn invoke_characters_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = cloudrouter_open_sdk::models::OpenAiVideoCharacterCreateRequest {
        name: Some(call.string_arg("name")?),
        description: call.optional_string_arg("description"),
        image: call.optional_string_arg("image"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let character = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create_character(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "characterId": character.id,
        }),
    ))
}

fn invoke_characters_list(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let character_id = call.string_arg("characterId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let character = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().list_characters(&character_id))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "characterId": character.id,
            "name": character.name,
            "description": character.description,
        }),
    ))
}

/// Maps provider status strings to the stable task status vocabulary.
fn normalize_video_status(status: &str) -> &'static str {
    match status.to_ascii_lowercase().as_str() {
        "queued" | "pending" => "queued",
        "processing" | "running" | "in_progress" => "processing",
        "completed" | "succeeded" | "success" => "completed",
        "failed" | "error" | "cancelled" => "failed",
        _ => "processing",
    }
}

fn invoke_kling_generations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = KlingVideoGenerationRequest {
        prompt: call.string_arg("prompt")?,
        model: call.optional_string_arg("model"),
        image: call.optional_string_arg("image"),
        image_tail: call.optional_string_arg("imageTail"),
        duration: optional_i64_arg(call, "duration"),
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        mode: call.optional_string_arg("mode"),
        cfg_scale: call.optional_number_arg("cfgScale"),
        negative_prompt: call.optional_string_arg("negativePrompt"),
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.videos_kling().create_v1_videos_generation(&request))
    })?;

    let task_id = task
        .task_id
        .or(task.id)
        .ok_or_else(|| MediaToolError::ProviderError("kling returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_kling_generations_retrieve(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let task_id = call.string_arg("taskId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.videos_kling().list_v1_videos_generations(&task_id))
    })?;

    let status = normalize_vendor_status(task.status.as_deref().or(task.state.as_deref()));
    let items = task.videos.unwrap_or_default();
    Ok(normalized_vendor_media(
        &task_id,
        status,
        items,
        "video",
        task.error.as_ref(),
    ))
}

fn invoke_vidu_text2video(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = ViduTextToVideoRequest {
        prompt: call.string_arg("prompt")?,
        model: model_arg(call, "model"),
        duration: optional_i64_arg(call, "duration"),
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        resolution: call.optional_string_arg("resolution"),
        movement_amplitude: call.optional_string_arg("movementAmplitude"),
        seed: optional_i64_arg(call, "seed"),
        payload: None,
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.videos_vidu().create_ent_v2_text2video(&request))
    })?;

    let task_id = task
        .task_id
        .ok_or_else(|| MediaToolError::ProviderError("vidu returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_vidu_img2video(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let images = string_array_arg(call, "images")?.ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "missing `images` argument for tool `{}`",
            call.tool_id
        ))
    })?;
    let request = ViduImageToVideoRequest {
        images,
        model: model_arg(call, "model"),
        prompt: call.optional_string_arg("prompt"),
        duration: optional_i64_arg(call, "duration"),
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        resolution: call.optional_string_arg("resolution"),
        movement_amplitude: call.optional_string_arg("movementAmplitude"),
        seed: optional_i64_arg(call, "seed"),
        payload: None,
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.videos_vidu().create_ent_v2_img2video(&request))
    })?;

    let task_id = task
        .task_id
        .ok_or_else(|| MediaToolError::ProviderError("vidu returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_vidu_reference2video(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let images = string_array_arg(call, "images")?.ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "missing `images` argument for tool `{}`",
            call.tool_id
        ))
    })?;
    let request = ViduReferenceToVideoRequest {
        images,
        model: model_arg(call, "model"),
        prompt: call.optional_string_arg("prompt"),
        duration: optional_i64_arg(call, "duration"),
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        resolution: call.optional_string_arg("resolution"),
        movement_amplitude: call.optional_string_arg("movementAmplitude"),
        seed: optional_i64_arg(call, "seed"),
        payload: None,
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.videos_vidu().create_ent_v2_reference2video(&request))
    })?;

    let task_id = task
        .task_id
        .ok_or_else(|| MediaToolError::ProviderError("vidu returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_vidu_start_end2video(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let images = string_array_arg(call, "images")?.ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "missing `images` argument for tool `{}`",
            call.tool_id
        ))
    })?;
    let request = ViduStartEndToVideoRequest {
        images,
        model: model_arg(call, "model"),
        prompt: call.optional_string_arg("prompt"),
        duration: optional_i64_arg(call, "duration"),
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        resolution: call.optional_string_arg("resolution"),
        movement_amplitude: call.optional_string_arg("movementAmplitude"),
        seed: optional_i64_arg(call, "seed"),
        payload: None,
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.videos_vidu().create_ent_v2_start_end2video(&request))
    })?;

    let task_id = task
        .task_id
        .ok_or_else(|| MediaToolError::ProviderError("vidu returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_vidu_tasks_creations(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let task_id = call.string_arg("taskId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.videos_vidu().list_ent_v2_tasks_creations(&task_id))
    })?;

    let status = normalize_vendor_status(task.state.as_deref());
    let creations = task.creations.unwrap_or_default();
    // Vidu creations carry video/image/audio URLs; normalize into video items.
    let items: Vec<cloudrouter_open_sdk::models::ProviderGeneratedMedia> = creations
        .into_iter()
        .map(
            |creation| cloudrouter_open_sdk::models::ProviderGeneratedMedia {
                url: creation.video_url.or(creation.url).or(creation.image_url),
                uri: creation.uri,
                duration: creation.duration,
                ..Default::default()
            },
        )
        .collect();

    Ok(normalized_vendor_media(
        &task_id, status, items, "video", None,
    ))
}

fn invoke_volcengine_generations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let mut content = vec![VolcengineContentPart {
        text: Some(call.string_arg("prompt")?),
        image_url: None,
        video_url: None,
        file_id: None,
        r#type: "text".to_string(),
    }];
    if let Some(image_url) = call.optional_string_arg("imageUrl") {
        content.push(VolcengineContentPart {
            text: None,
            image_url: Some(image_url),
            video_url: None,
            file_id: None,
            r#type: "image_url".to_string(),
        });
    }
    let request = VolcengineContentGenerationTaskCreateRequest {
        content,
        model: model_arg(call, "model"),
        callback_url: None,
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let response = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(
            sdk.videos_volcengine()
                .create_api_v3_contents_generations_task(&request),
        )
    })?;

    let task_id = response
        .task_id
        .or(response.id)
        .ok_or_else(|| MediaToolError::ProviderError("volcengine returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_volcengine_generations_retrieve(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let task_id = call.string_arg("taskId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(
            sdk.videos_volcengine()
                .list_api_v3_contents_generations_tasks(&task_id),
        )
    })?;

    let status = normalize_vendor_status(task.status.as_deref().or(task.state.as_deref()));
    let items = task.videos.unwrap_or_default();
    Ok(normalized_vendor_media(
        &task_id,
        status,
        items,
        "video",
        task.error.as_ref(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_provider_statuses() {
        assert_eq!(normalize_video_status("queued"), "queued");
        assert_eq!(normalize_video_status("processing"), "processing");
        assert_eq!(normalize_video_status("completed"), "completed");
        assert_eq!(normalize_video_status("failed"), "failed");
        assert_eq!(normalize_video_status("unknown"), "processing");
    }

    #[test]
    fn unknown_tool_id_returns_capability_missing() {
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "video.not.a.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_video_tool(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn create_requires_auth_token_before_network() {
        let call = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: tool_ids::CREATE.to_string(),
            arguments: serde_json::json!({ "prompt": "a robot walking" }),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_video_tool(&call, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");
    }

    #[test]
    fn retrieve_requires_video_id_argument() {
        let call = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::RETRIEVE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_video_tool(&call, Some("token")).expect_err("videoId required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn remix_requires_both_video_id_and_prompt() {
        let missing_id = MediaToolCall {
            tool_call_id: "call.4".to_string(),
            tool_id: tool_ids::REMIX_CREATE.to_string(),
            arguments: serde_json::json!({ "prompt": "new style" }),
            session_id: None,
            trace_id: None,
        };
        assert!(invoke_video_tool(&missing_id, Some("token")).is_err());
    }
}
