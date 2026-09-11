//! Invocation behaviour for the generations provider.
//!
//! Dispatches each `generations.*` tool call to the matching cloudrouter
//! open-api surface. Synchronous generation tools (image, voice) return
//! normalized media resources; async task tools (video, music) return a
//! `taskId` for polling. SFX is reserved pending upstream capability.

use cloudrouter_open_sdk::models::{
    OpenAiAudioTranscriptionRequest, OpenAiAudioTranslationRequest, OpenAiFileReferenceInput,
    OpenAiImageEditRequest, OpenAiImageReferenceInput, OpenAiImageReferenceInputList,
    OpenAiImageGenerationRequest, OpenAiSpeechCreateRequest, OpenAiVideoCreateRequest,
    OpenAiVideoExtendRequest, SunoMusicGenerationRequest,
};
use sdkwork_agents_tool_cloudrouter::{model_arg, run_sync, CloudRouterMediaClient};
use sdkwork_agents_tool_contract::{MediaResource, MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::tool_ids;

/// Executes one generations tool call against the cloudrouter gateway.
pub fn invoke_generations_tool(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    match call.tool_id.as_str() {
        tool_ids::IMAGE_TEXT_TO_IMAGE => invoke_image_text_to_image(call, auth_token),
        tool_ids::IMAGE_EDIT => invoke_image_edit(call, auth_token),
        tool_ids::VIDEO_TEXT_TO_VIDEO => invoke_video_text_to_video(call, auth_token),
        tool_ids::VIDEO_IMAGE_TO_VIDEO => invoke_video_image_to_video(call, auth_token),
        tool_ids::VIDEO_EXTEND => invoke_video_extend(call, auth_token),
        tool_ids::MUSIC_TEXT_TO_MUSIC => invoke_music_text_to_music(call, auth_token),
        tool_ids::MUSIC_LYRICS_TO_MUSIC => invoke_music_lyrics_to_music(call, auth_token),
        tool_ids::SFX_CREATE => invoke_sfx_create(call, auth_token),
        tool_ids::VOICE_SPEECH => invoke_voice_speech(call, auth_token),
        tool_ids::VOICE_TRANSCRIPTION => invoke_voice_transcription(call, auth_token),
        tool_ids::VOICE_TRANSLATION => invoke_voice_translation(call, auth_token),
        other => Err(MediaToolError::CapabilityMissing(format!(
            "generations provider has no tool `{other}`"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Image tools
// ---------------------------------------------------------------------------

fn invoke_image_text_to_image(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiImageGenerationRequest {
        prompt: call.string_arg("prompt")?,
        model: model_arg(call, "model"),
        n: call.optional_number_arg("n").map(|value| value as i64),
        size: call.optional_string_arg("size"),
        quality: call.optional_string_arg("quality"),
        response_format: call.optional_string_arg("response_format"),
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let images = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images().create_generation(&request))
    })?;

    Ok(normalized_image_items(&call.tool_call_id, images.data))
}

fn invoke_image_edit(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let image = image_reference_arg(call, "image")?;
    let mask = optional_image_reference_arg(call, "mask")?;

    let request = OpenAiImageEditRequest {
        image: Some(OpenAiImageReferenceInputList {
            additional_properties: image.additional_properties,
        }),
        mask,
        model: model_arg(call, "model"),
        prompt: call.string_arg("prompt")?,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let images = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images().create_edit(&request))
    })?;

    Ok(normalized_image_items(&call.tool_call_id, images.data))
}

// ---------------------------------------------------------------------------
// Video tools
// ---------------------------------------------------------------------------

fn invoke_video_text_to_video(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiVideoCreateRequest {
        prompt: call.string_arg("prompt")?,
        model: model_arg(call, "model"),
        seconds: call.optional_number_arg("seconds").map(|value| value as i64),
        size: call.optional_string_arg("size"),
        image: None,
        video: None,
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create(&request))
    })?;

    Ok(task_submission_result(
        &call.tool_call_id,
        &video.id,
        &video.status,
    ))
}

fn invoke_video_image_to_video(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiVideoCreateRequest {
        prompt: call.string_arg("prompt")?,
        model: model_arg(call, "model"),
        image: Some(call.string_arg("image")?),
        seconds: call.optional_number_arg("seconds").map(|value| value as i64),
        size: call.optional_string_arg("size"),
        video: None,
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create(&request))
    })?;

    Ok(task_submission_result(
        &call.tool_call_id,
        &video.id,
        &video.status,
    ))
}

fn invoke_video_extend(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiVideoExtendRequest {
        prompt: Some(call.string_arg("prompt")?),
        model: Some(model_arg(call, "model")),
        video: Some(call.string_arg("video")?),
        seconds: call.optional_number_arg("seconds").map(|value| value as i64),
        size: call.optional_string_arg("size"),
        image: None,
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let video = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.video().create_extension(&request))
    })?;

    Ok(task_submission_result(
        &call.tool_call_id,
        &video.id,
        &video.status,
    ))
}

// ---------------------------------------------------------------------------
// Music tools
// ---------------------------------------------------------------------------

fn invoke_music_text_to_music(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = SunoMusicGenerationRequest {
        prompt: call.string_arg("prompt")?,
        model: call.optional_string_arg("model"),
        title: call.optional_string_arg("title"),
        duration: call.optional_number_arg("duration"),
        tags: call.optional_string_arg("tags"),
        negative_tags: call.optional_string_arg("negative_tags"),
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let response = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio_suno().create_v1_music_generation(&request))
    })?;

    let task_id = response
        .task_id
        .or(response.id)
        .ok_or_else(|| MediaToolError::ProviderError("suno returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_music_lyrics_to_music(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = SunoMusicGenerationRequest {
        prompt: call.string_arg("lyrics")?,
        model: call.optional_string_arg("model"),
        title: call.optional_string_arg("title"),
        duration: call.optional_number_arg("duration"),
        tags: call.optional_string_arg("tags"),
        negative_tags: None,
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let response = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio_suno().create_v1_music_generation(&request))
    })?;

    let task_id = response
        .task_id
        .or(response.id)
        .ok_or_else(|| MediaToolError::ProviderError("suno returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

// ---------------------------------------------------------------------------
// SFX tool
// ---------------------------------------------------------------------------

fn invoke_sfx_create(
    call: &MediaToolCall,
    _auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    // SFX generation is reserved pending upstream cloudrouter capability.
    let _ = call.string_arg("prompt")?;
    Err(MediaToolError::pending_capability(
        "sdkwork-generations sound-effect generation is not yet available on the \
         cloudrouter open-api surface",
    ))
}

// ---------------------------------------------------------------------------
// Voice tools
// ---------------------------------------------------------------------------

fn invoke_voice_speech(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiSpeechCreateRequest {
        input: call.string_arg("input")?,
        voice: call.string_arg("voice")?,
        model: model_arg(call, "model"),
        response_format: call.optional_string_arg("response_format"),
        speed: call.optional_number_arg("speed"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let audio_bytes = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_speech(&request))
    })?;

    // The cloudrouter speech endpoint returns raw audio bytes as a string;
    // surface as a data-URL-free opaque payload. The application layer is
    // responsible for persisting the bytes to a retrievable asset.
    let resource = MediaResource::provider_asset("audio", "data:audio/mpeg;base64,...");
    let mut value = serde_json::to_value(&resource).unwrap_or(serde_json::Value::Null);
    value["audioBase64"] = serde_json::json!(audio_bytes);
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "items": [value] }),
    ))
}

fn invoke_voice_transcription(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let file = file_reference_arg(call, "file")?;
    let request = OpenAiAudioTranscriptionRequest {
        file,
        model: model_arg(call, "model"),
        language: call.optional_string_arg("language"),
        prompt: call.optional_string_arg("prompt"),
        response_format: call.optional_string_arg("response_format"),
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let transcription = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_transcription(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "text": transcription.text }),
    ))
}

fn invoke_voice_translation(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let file = file_reference_arg(call, "file")?;
    let request = OpenAiAudioTranslationRequest {
        file,
        model: model_arg(call, "model"),
        prompt: call.optional_string_arg("prompt"),
        response_format: call.optional_string_arg("response_format"),
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let translation = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_translation(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "text": translation.text }),
    ))
}

// ---------------------------------------------------------------------------
// Normalization helpers
// ---------------------------------------------------------------------------

/// Normalizes the vendor image list into `MediaResource` shape, dropping items
/// without a delivery URL.
fn normalized_image_items(
    tool_call_id: &str,
    images: Vec<cloudrouter_open_sdk::models::OpenAiImage>,
) -> MediaToolResult {
    let items: Vec<serde_json::Value> = images
        .iter()
        .filter_map(|image| {
            let url = image.url.as_ref()?.trim();
            if url.is_empty() {
                return None;
            }
            let resource = MediaResource::provider_asset("image", url);
            let mut value = serde_json::to_value(&resource).unwrap_or(serde_json::Value::Null);
            if let Some(revised_prompt) = image.revised_prompt.as_deref() {
                value["revisedPrompt"] = serde_json::json!(revised_prompt);
            }
            Some(value)
        })
        .collect();

    MediaToolResult::succeeded(tool_call_id.to_string(), serde_json::json!({ "items": items }))
}

/// Builds the async task submission result for video/music tools.
fn task_submission_result(tool_call_id: &str, task_id: &str, status: &str) -> MediaToolResult {
    MediaToolResult::succeeded(
        tool_call_id.to_string(),
        serde_json::json!({ "taskId": task_id, "status": status }),
    )
}

/// Builds the required image reference from the `image` argument object.
fn image_reference_arg(
    call: &MediaToolCall,
    name: &str,
) -> Result<OpenAiImageReferenceInput, MediaToolError> {
    let reference = call.arguments.get(name).ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "missing `{name}` argument for tool `{}`",
            call.tool_id
        ))
    })?;
    let object = reference.as_object().ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "`{name}` must be an object (url or asset reference) for tool `{}`",
            call.tool_id
        ))
    })?;
    Ok(OpenAiImageReferenceInput {
        additional_properties: object.clone().into_iter().collect(),
    })
}

/// Builds an optional image reference from the `mask` argument object.
fn optional_image_reference_arg(
    call: &MediaToolCall,
    name: &str,
) -> Result<Option<OpenAiImageReferenceInput>, MediaToolError> {
    if call.arguments.get(name).is_none() {
        return Ok(None);
    }
    image_reference_arg(call, name).map(Some)
}

/// Builds the required file reference from the `file` argument object.
fn file_reference_arg(
    call: &MediaToolCall,
    name: &str,
) -> Result<OpenAiFileReferenceInput, MediaToolError> {
    let reference = call.arguments.get(name).ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "missing `{name}` argument for tool `{}`",
            call.tool_id
        ))
    })?;
    let object = reference.as_object().ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "`{name}` must be an object (url or asset reference) for tool `{}`",
            call.tool_id
        ))
    })?;
    Ok(OpenAiFileReferenceInput {
        additional_properties: object.clone().into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_tool_id_returns_capability_missing() {
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "generations.not.a.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn image_generation_requires_auth_token_before_network() {
        let call = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: tool_ids::IMAGE_TEXT_TO_IMAGE.to_string(),
            arguments: serde_json::json!({ "prompt": "a red fox" }),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");
    }

    #[test]
    fn image_generation_validates_prompt_argument() {
        let call = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::IMAGE_TEXT_TO_IMAGE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("prompt required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn image_edit_rejects_missing_image_reference() {
        let call = MediaToolCall {
            tool_call_id: "call.4".to_string(),
            tool_id: tool_ids::IMAGE_EDIT.to_string(),
            arguments: serde_json::json!({ "prompt": "add snow" }),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("image required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn video_text_to_video_requires_prompt() {
        let call = MediaToolCall {
            tool_call_id: "call.5".to_string(),
            tool_id: tool_ids::VIDEO_TEXT_TO_VIDEO.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("prompt required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn video_image_to_video_requires_image_argument() {
        let call = MediaToolCall {
            tool_call_id: "call.6".to_string(),
            tool_id: tool_ids::VIDEO_IMAGE_TO_VIDEO.to_string(),
            arguments: serde_json::json!({ "prompt": "make it move" }),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("image required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn music_text_to_music_requires_prompt() {
        let call = MediaToolCall {
            tool_call_id: "call.7".to_string(),
            tool_id: tool_ids::MUSIC_TEXT_TO_MUSIC.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("prompt required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn music_lyrics_to_music_requires_lyrics() {
        let call = MediaToolCall {
            tool_call_id: "call.8".to_string(),
            tool_id: tool_ids::MUSIC_LYRICS_TO_MUSIC.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("lyrics required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn sfx_create_returns_pending_capability() {
        let call = MediaToolCall {
            tool_call_id: "call.9".to_string(),
            tool_id: tool_ids::SFX_CREATE.to_string(),
            arguments: serde_json::json!({ "prompt": "thunder crack" }),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("sfx pending");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn voice_speech_requires_input_and_voice() {
        let call = MediaToolCall {
            tool_call_id: "call.10".to_string(),
            tool_id: tool_ids::VOICE_SPEECH.to_string(),
            arguments: serde_json::json!({ "input": "hello" }),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("voice required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn voice_transcription_requires_file_reference() {
        let call = MediaToolCall {
            tool_call_id: "call.11".to_string(),
            tool_id: tool_ids::VOICE_TRANSCRIPTION.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("file required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn voice_translation_requires_file_reference() {
        let call = MediaToolCall {
            tool_call_id: "call.12".to_string(),
            tool_id: tool_ids::VOICE_TRANSLATION.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_generations_tool(&call, Some("token")).expect_err("file required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn normalized_image_items_drops_images_without_url() {
        use cloudrouter_open_sdk::models::OpenAiImage;
        let images = vec![
            OpenAiImage {
                url: Some("https://cdn.example/a.png".to_string()),
                revised_prompt: Some("better fox".to_string()),
                ..Default::default()
            },
            OpenAiImage {
                url: None,
                revised_prompt: None,
                ..Default::default()
            },
            OpenAiImage {
                url: Some("   ".to_string()),
                revised_prompt: None,
                ..Default::default()
            },
        ];
        let result = normalized_image_items("call.13", images);
        let items = result.output["items"].as_array().expect("array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["url"], "https://cdn.example/a.png");
        assert_eq!(items[0]["kind"], "image");
        assert_eq!(items[0]["source"], "provider_asset");
        assert_eq!(items[0]["revisedPrompt"], "better fox");
    }

    #[test]
    fn task_submission_result_carries_task_id_and_status() {
        let result = task_submission_result("call.14", "task.abc", "queued");
        assert_eq!(result.status, "succeeded");
        assert_eq!(result.output["taskId"], "task.abc");
        assert_eq!(result.output["status"], "queued");
    }
}
