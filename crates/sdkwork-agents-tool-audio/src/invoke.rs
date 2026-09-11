//! Invocation behaviour for the audio category tools.

use cloudrouter_open_sdk::models::{
    OpenAiAudioTranscriptionRequest, OpenAiFileReferenceInput, OpenAiSpeechCreateRequest,
};
use sdkwork_agents_tool_cloudrouter::{run_sync, CloudRouterMediaClient};
use sdkwork_agents_tool_contract::{MediaResource, MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::tool_ids;

/// Executes one audio tool call against the cloudrouter gateway.
pub fn invoke_audio_tool(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    match call.tool_id.as_str() {
        tool_ids::SPEECH_CREATE => invoke_speech_create(call, auth_token),
        tool_ids::TRANSCRIPTIONS_CREATE => invoke_transcriptions_create(call, auth_token),
        tool_ids::TRANSLATIONS_CREATE => invoke_translations_create(call, auth_token),
        tool_ids::VOICES_LIST => invoke_voices_list(call, auth_token),
        tool_ids::VOICES_CREATE => invoke_voices_create(call, auth_token),
        tool_ids::VOICE_CONSENTS_CREATE => invoke_voice_consents_create(call, auth_token),
        tool_ids::VOICE_CONSENTS_LIST => invoke_voice_consents_list(call, auth_token),
        other => Err(MediaToolError::CapabilityMissing(format!(
            "audio provider has no tool `{other}`"
        ))),
    }
}

fn invoke_speech_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let input = call.string_arg("input")?;
    let model = call
        .optional_string_arg("model")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "default".to_string());
    let voice = call
        .optional_string_arg("voice")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "alloy".to_string());
    let speed = call.optional_number_arg("speed");
    let response_format = call.optional_string_arg("response_format");

    let request = OpenAiSpeechCreateRequest {
        input,
        model,
        voice,
        speed,
        response_format,
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let audio_bytes = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_speech(&request))
    })?;
    if audio_bytes.is_empty() {
        return Err(MediaToolError::ProviderError(
            "speech synthesis returned no audio bytes".to_string(),
        ));
    }
    use base64::Engine as _;
    let audio_url = format!(
        "data:audio/mpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&audio_bytes)
    );

    let resource = MediaResource::provider_asset("audio", audio_url);
    Ok(MediaToolResult::succeeded_with_resource(
        &call.tool_call_id,
        &resource,
    ))
}

fn invoke_transcriptions_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let file = file_reference_arg(call)?;
    let model = call
        .optional_string_arg("model")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "default".to_string());
    let language = call.optional_string_arg("language");
    let prompt = call.optional_string_arg("prompt");
    let response_format = call.optional_string_arg("response_format");

    let request = OpenAiAudioTranscriptionRequest {
        file,
        model,
        language,
        prompt,
        response_format,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let transcription = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_transcription(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "text": transcription.text,
            "language": transcription.language,
            "duration": transcription.duration,
        }),
    ))
}

fn invoke_translations_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let file = file_reference_arg(call)?;
    let model = call
        .optional_string_arg("model")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "default".to_string());
    let prompt = call.optional_string_arg("prompt");
    let response_format = call.optional_string_arg("response_format");

    let request = cloudrouter_open_sdk::models::OpenAiAudioTranslationRequest {
        file,
        model,
        prompt,
        response_format,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let translation = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_translation(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "text": translation.text,
            "duration": translation.duration,
        }),
    ))
}

fn invoke_voices_list(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let limit = call.optional_number_arg("limit").map(|value| value as i64);

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let voices = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().list_voices(limit, None, None, None))
    })?;

    let items: Vec<serde_json::Value> = voices
        .data
        .iter()
        .map(|voice| {
            serde_json::json!({
                "id": voice.id,
                "name": voice.name,
                "status": voice.status,
            })
        })
        .collect();

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "voices": items }),
    ))
}

/// Builds the OpenAI file reference from the `file` argument object.
///
/// Accepts either a URL reference (`{ "url": "..." }`), a provider file id
/// (`{ "file_id": "..." }`), or a provider-specific payload object, mirroring
/// the cloudrouter open-api file input contract.
fn file_reference_arg(call: &MediaToolCall) -> Result<OpenAiFileReferenceInput, MediaToolError> {
    let file = call.arguments.get("file").ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "missing `file` argument for tool `{}`",
            call.tool_id
        ))
    })?;
    let object = file.as_object().ok_or_else(|| {
        MediaToolError::invalid_argument(format!(
            "`file` must be an object (url or file_id reference) for tool `{}`",
            call.tool_id
        ))
    })?;
    Ok(OpenAiFileReferenceInput {
        additional_properties: object.clone().into_iter().collect(),
    })
}

fn invoke_voices_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = cloudrouter_open_sdk::models::OpenAiVoiceCreateRequest {
        name: Some(call.string_arg("name")?),
        description: call.optional_string_arg("description"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let voice = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_voice(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "id": voice.id,
            "name": voice.name,
            "status": voice.status,
        }),
    ))
}

fn invoke_voice_consents_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = cloudrouter_open_sdk::models::OpenAiVoiceConsentCreateRequest {
        name: Some(call.string_arg("name")?),
        consent_document: call.optional_string_arg("consentDocument"),
        metadata: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let consent = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().create_voice_consent(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "id": consent.id,
            "name": consent.name,
            "status": consent.status,
        }),
    ))
}

fn invoke_voice_consents_list(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let limit = call.optional_number_arg("limit").map(|value| value as i64);

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let consents = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.audio().list_voice_consents(limit, None, None, None))
    })?;

    let items: Vec<serde_json::Value> = consents
        .data
        .iter()
        .map(|consent| {
            serde_json::json!({
                "id": consent.id,
                "name": consent.name,
                "status": consent.status,
            })
        })
        .collect();

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "consents": items }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_reference_accepts_url_and_file_id_objects() {
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: tool_ids::TRANSCRIPTIONS_CREATE.to_string(),
            arguments: serde_json::json!({ "file": { "url": "https://cdn.example/a.mp3" } }),
            session_id: None,
            trace_id: None,
        };
        let reference = file_reference_arg(&call).expect("url reference accepted");
        assert_eq!(
            reference.additional_properties.get("url"),
            Some(&serde_json::json!("https://cdn.example/a.mp3"))
        );

        let call = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: tool_ids::TRANSCRIPTIONS_CREATE.to_string(),
            arguments: serde_json::json!({ "file": { "file_id": "file.123" } }),
            session_id: None,
            trace_id: None,
        };
        let reference = file_reference_arg(&call).expect("file_id reference accepted");
        assert_eq!(
            reference.additional_properties.get("file_id"),
            Some(&serde_json::json!("file.123"))
        );
    }

    #[test]
    fn file_reference_rejects_missing_or_non_object() {
        let missing = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::TRANSCRIPTIONS_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        assert!(file_reference_arg(&missing).is_err());

        let scalar = MediaToolCall {
            tool_call_id: "call.4".to_string(),
            tool_id: tool_ids::TRANSCRIPTIONS_CREATE.to_string(),
            arguments: serde_json::json!({ "file": "https://cdn.example/a.mp3" }),
            session_id: None,
            trace_id: None,
        };
        assert!(file_reference_arg(&scalar).is_err());
    }

    #[test]
    fn unknown_tool_id_returns_capability_missing() {
        let call = MediaToolCall {
            tool_call_id: "call.5".to_string(),
            tool_id: "audio.not.a.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_audio_tool(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn speech_create_requires_auth_token_before_network() {
        let call = MediaToolCall {
            tool_call_id: "call.6".to_string(),
            tool_id: tool_ids::SPEECH_CREATE.to_string(),
            arguments: serde_json::json!({ "input": "hello" }),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_audio_tool(&call, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");
    }

    #[test]
    fn speech_create_validates_input_argument() {
        let call = MediaToolCall {
            tool_call_id: "call.7".to_string(),
            tool_id: tool_ids::SPEECH_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_audio_tool(&call, Some("token")).expect_err("input required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn voices_create_and_consents_validate_arguments() {
        let no_name = MediaToolCall {
            tool_call_id: "call.8".to_string(),
            tool_id: tool_ids::VOICES_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_audio_tool(&no_name, Some("token")).expect_err("name required");
        assert_eq!(error.code(), "invalid_input");

        let no_consent_name = MediaToolCall {
            tool_call_id: "call.9".to_string(),
            tool_id: tool_ids::VOICE_CONSENTS_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_audio_tool(&no_consent_name, Some("token")).expect_err("name required");
        assert_eq!(error.code(), "invalid_input");

        let no_token = MediaToolCall {
            tool_call_id: "call.10".to_string(),
            tool_id: tool_ids::VOICE_CONSENTS_LIST.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_audio_tool(&no_token, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");
    }
}
