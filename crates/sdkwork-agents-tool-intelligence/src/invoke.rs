//! Invocation behaviour for the intelligence category tools.

use cloudrouter_open_sdk::models::{OpenAiEmbeddingsRequest, OpenAiModerationCreateRequest};
use sdkwork_agents_tool_cloudrouter::{run_sync, CloudRouterMediaClient};
use sdkwork_agents_tool_contract::{MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::tool_ids;

/// Executes one intelligence tool call against the cloudrouter gateway.
pub fn invoke_intelligence_tool(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    match call.tool_id.as_str() {
        tool_ids::MODEL_LIST => invoke_model_list(call, auth_token),
        tool_ids::EMBEDDING_CREATE => invoke_embedding_create(call, auth_token),
        tool_ids::MODERATION_CREATE => invoke_moderation_create(call, auth_token),
        other => Err(MediaToolError::CapabilityMissing(format!(
            "intelligence provider has no tool `{other}`"
        ))),
    }
}

fn invoke_model_list(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let models = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.models().list())
    })?;

    let items: Vec<serde_json::Value> = models
        .data
        .iter()
        .map(|model| {
            serde_json::json!({
                "id": model.id,
                "ownedBy": model.owned_by,
                "created": model.created,
            })
        })
        .collect();

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "models": items }),
    ))
}

fn invoke_embedding_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiEmbeddingsRequest {
        input: call.string_arg("input")?,
        model: call
            .optional_string_arg("model")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "default".to_string()),
        dimensions: call
            .optional_number_arg("dimensions")
            .map(|value| value as i64),
        encoding_format: call.optional_string_arg("encoding_format"),
        user: None,
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let embeddings = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.embeddings().create(&request))
    })?;

    let items: Vec<serde_json::Value> = embeddings
        .data
        .iter()
        .map(|embedding| {
            serde_json::json!({
                "index": embedding.index,
                "embedding": embedding.embedding,
            })
        })
        .collect();

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "embeddings": items,
            "model": embeddings.model,
        }),
    ))
}

fn invoke_moderation_create(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiModerationCreateRequest {
        input: call.string_arg("input")?,
        model: call
            .optional_string_arg("model")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "default".to_string()),
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    let moderation = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.moderations().create(&request))
    })?;

    let items: Vec<serde_json::Value> = moderation
        .results
        .iter()
        .map(|result| {
            serde_json::json!({
                "flagged": result.flagged,
                "categories": result.categories,
                "categoryScores": result.category_scores,
            })
        })
        .collect();

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "id": moderation.id,
            "model": moderation.model,
            "results": items,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_tool_id_returns_capability_missing() {
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "model.not.a.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_intelligence_tool(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn model_list_requires_auth_token_before_network() {
        let call = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: tool_ids::MODEL_LIST.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error = invoke_intelligence_tool(&call, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");
    }

    #[test]
    fn embedding_and_moderation_validate_input() {
        let embedding = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::EMBEDDING_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error =
            invoke_intelligence_tool(&embedding, Some("token")).expect_err("input required");
        assert_eq!(error.code(), "invalid_input");

        let moderation = MediaToolCall {
            tool_call_id: "call.4".to_string(),
            tool_id: tool_ids::MODERATION_CREATE.to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
        };
        let error =
            invoke_intelligence_tool(&moderation, Some("token")).expect_err("input required");
        assert_eq!(error.code(), "invalid_input");
    }
}
