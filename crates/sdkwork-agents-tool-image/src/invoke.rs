//! Invocation behaviour for the image category tools.

use cloudrouter_open_sdk::models::{
    MidjourneyImageGenerationRequest, NanoBananaImageGenerationRequest, OpenAiImageEditRequest,
    OpenAiImageGenerationRequest, OpenAiImageReferenceInput, OpenAiImageReferenceInputList,
    OpenAiImageVariationRequest, ViduReferenceToImageRequest,
};
use sdkwork_agents_tool_cloudrouter::{
    model_arg, normalize_vendor_status, normalized_vendor_media, run_sync, string_array_arg,
    CloudRouterMediaClient,
};
use sdkwork_agents_tool_contract::{MediaResource, MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::tool_ids;

/// Executes one image tool call against the cloudrouter gateway.
pub fn invoke_image_tool(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    match call.tool_id.as_str() {
        tool_ids::GENERATIONS_CREATE => invoke_generations_create(call, auth_token),
        tool_ids::EDITS_CREATE => invoke_edits_create(call, auth_token),
        tool_ids::VARIATIONS_CREATE => invoke_variations_create(call, auth_token),
        tool_ids::MIDJOURNEY_GENERATIONS_CREATE => {
            invoke_midjourney_generations_create(call, auth_token)
        }
        tool_ids::MIDJOURNEY_GENERATIONS_LIST => {
            invoke_midjourney_generations_list(call, auth_token)
        }
        tool_ids::NANO_BANANA_GENERATIONS_CREATE => {
            invoke_nano_banana_generations_create(call, auth_token)
        }
        tool_ids::NANO_BANANA_GENERATIONS_RETRIEVE => {
            invoke_nano_banana_generations_retrieve(call, auth_token)
        }
        tool_ids::VIDU_REFERENCE2IMAGE => invoke_vidu_reference2image(call, auth_token),
        other => Err(MediaToolError::CapabilityMissing(format!(
            "image provider has no tool `{other}`"
        ))),
    }
}

fn invoke_generations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiImageGenerationRequest {
        prompt: call.string_arg("prompt")?,
        model: call
            .optional_string_arg("model")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "default".to_string()),
        n: call.optional_number_arg("n").map(|value| value as i64),
        size: call.optional_string_arg("size"),
        quality: call.optional_string_arg("quality"),
        response_format: call.optional_string_arg("response_format"),
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let images = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images().create_generation(&request))
    })?;

    Ok(normalized_images_result(&call.tool_call_id, images.data))
}

fn invoke_edits_create(
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
        model: call
            .optional_string_arg("model")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "default".to_string()),
        prompt: call.string_arg("prompt")?,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let images = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images().create_edit(&request))
    })?;

    Ok(normalized_images_result(&call.tool_call_id, images.data))
}

fn invoke_variations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let image = image_reference_arg(call, "image")?;

    let request = OpenAiImageVariationRequest {
        image,
        model: call
            .optional_string_arg("model")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "default".to_string()),
        size: call.optional_string_arg("size"),
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let images = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images().create_variation(&request))
    })?;

    Ok(normalized_images_result(&call.tool_call_id, images.data))
}

/// Normalizes the vendor image list into `MediaResource` shape, dropping
/// items without a delivery URL.
fn normalized_images_result(
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

    MediaToolResult::succeeded(tool_call_id, serde_json::json!({ "images": items }))
}

fn invoke_midjourney_generations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = MidjourneyImageGenerationRequest {
        prompt: call.string_arg("prompt")?,
        model: call.optional_string_arg("model"),
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        style: call.optional_string_arg("style"),
        seed: call.optional_number_arg("seed").map(|value| value as i64),
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(
            sdk.images_midjourney()
                .create_v1_images_generation(&request),
        )
    })?;

    let task_id = task
        .task_id
        .or(task.id)
        .ok_or_else(|| MediaToolError::ProviderError("midjourney returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_midjourney_generations_list(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let task_id = call.string_arg("taskId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images_midjourney().list_v1_images_generations(&task_id))
    })?;

    let status = normalize_vendor_status(task.status.as_deref().or(task.state.as_deref()));
    let items = task.images.unwrap_or_default();
    Ok(normalized_vendor_media(
        &task_id,
        status,
        items,
        "image",
        task.error.as_ref(),
    ))
}

fn invoke_nano_banana_generations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = NanoBananaImageGenerationRequest {
        prompt: call.string_arg("prompt")?,
        model: call.optional_string_arg("model"),
        images: string_array_arg(call, "images")?,
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        size: call.optional_string_arg("size"),
        seed: call.optional_number_arg("seed").map(|value| value as i64),
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images_nano_banana().create_generations(&request))
    })?;

    let task_id = task
        .task_id
        .or(task.id)
        .ok_or_else(|| MediaToolError::ProviderError("nano-banana returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
}

fn invoke_nano_banana_generations_retrieve(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let task_id = call.string_arg("taskId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images_nano_banana().retrieve_generations(&task_id))
    })?;

    let status = normalize_vendor_status(task.status.as_deref().or(task.state.as_deref()));
    let items = task.images.unwrap_or_default();
    Ok(normalized_vendor_media(
        &task_id,
        status,
        items,
        "image",
        task.error.as_ref(),
    ))
}

fn invoke_vidu_reference2image(
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
    let request = ViduReferenceToImageRequest {
        prompt: call.string_arg("prompt")?,
        model: model_arg(call, "model"),
        images,
        aspect_ratio: call.optional_string_arg("aspectRatio"),
        style: call.optional_string_arg("style"),
        seed: call.optional_number_arg("seed").map(|value| value as i64),
        payload: None,
        callback_url: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let task = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.images_vidu().create_ent_v2_reference2image(&request))
    })?;

    let task_id = task
        .task_id
        .ok_or_else(|| MediaToolError::ProviderError("vidu returned no task id".into()))?;
    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "taskId": task_id }),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_tool_id_returns_capability_missing() {
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "image.not.a.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_image_tool(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn generation_requires_auth_token_before_network() {
        let call = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: tool_ids::GENERATIONS_CREATE.to_string(),
            arguments: serde_json::json!({ "prompt": "a red fox" }),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_image_tool(&call, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");
    }

    #[test]
    fn generation_validates_prompt_argument() {
        let call = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::GENERATIONS_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_image_tool(&call, Some("token")).expect_err("prompt required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn variations_reject_missing_image_reference() {
        let call = MediaToolCall {
            tool_call_id: "call.4".to_string(),
            tool_id: tool_ids::VARIATIONS_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_image_tool(&call, Some("token")).expect_err("image required");
        assert_eq!(error.code(), "invalid_input");

        let scalar = MediaToolCall {
            tool_call_id: "call.5".to_string(),
            tool_id: tool_ids::VARIATIONS_CREATE.to_string(),
            arguments: serde_json::json!({ "image": "https://cdn.example/a.png" }),
            session_id: None,
            trace_id: None,
        };
        assert!(invoke_image_tool(&scalar, Some("token")).is_err());
    }

    #[test]
    fn normalized_result_skips_images_without_url() {
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
        let result = normalized_images_result("call.6", images);
        let items = result.output["images"].as_array().expect("array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["url"], "https://cdn.example/a.png");
        assert_eq!(items[0]["kind"], "image");
        assert_eq!(items[0]["source"], "provider_asset");
        assert_eq!(items[0]["revisedPrompt"], "better fox");
    }
}
