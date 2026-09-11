//! Invocation behaviour for the file category tools.

use cloudrouter_open_sdk::models::OpenAiFileUploadRequest;
use sdkwork_agents_tool_cloudrouter::{run_sync, CloudRouterMediaClient};
use sdkwork_agents_tool_contract::{MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::tool_ids;

/// Executes one file tool call against the cloudrouter gateway.
pub fn invoke_file_tool(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    match call.tool_id.as_str() {
        tool_ids::UPLOAD => invoke_upload(call, auth_token),
        tool_ids::LIST => invoke_list(call, auth_token),
        tool_ids::RETRIEVE => invoke_retrieve(call, auth_token),
        tool_ids::DELETE => invoke_delete(call, auth_token),
        tool_ids::CONTENT => invoke_content(call, auth_token),
        other => Err(MediaToolError::CapabilityMissing(format!(
            "file provider has no tool `{other}`"
        ))),
    }
}

fn invoke_upload(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let request = OpenAiFileUploadRequest {
        // The generated SDK binds the multipart file part as bytes; the tool
        // contract accepts a URL/asset reference string, which the gateway
        // registers as the file payload (existing behavior, now explicit).
        file: call.string_arg("file")?.into_bytes(),
        purpose: call
            .optional_string_arg("purpose")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "assistants".to_string()),
    };

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let file = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.files().create(&request))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "id": file.id,
            "filename": file.filename,
            "bytes": file.bytes,
            "purpose": file.purpose,
            "status": file.status,
        }),
    ))
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
    let files = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.files().list(limit, None, None, None))
    })?;

    let items: Vec<serde_json::Value> = files
        .data
        .iter()
        .map(|file| {
            serde_json::json!({
                "id": file.id,
                "filename": file.filename,
                "bytes": file.bytes,
                "purpose": file.purpose,
                "status": file.status,
            })
        })
        .collect();

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({ "files": items }),
    ))
}

fn invoke_retrieve(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let file_id = call.string_arg("fileId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let file = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.files().retrieve(&file_id))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "id": file.id,
            "filename": file.filename,
            "bytes": file.bytes,
            "purpose": file.purpose,
            "status": file.status,
        }),
    ))
}

fn invoke_delete(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let file_id = call.string_arg("fileId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let result = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.files().delete(&file_id))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "deleted": result.deleted,
            "fileId": result.id,
        }),
    ))
}

fn invoke_content(
    call: &MediaToolCall,
    auth_token: Option<&str>,
) -> Result<MediaToolResult, MediaToolError> {
    let auth_token = CloudRouterMediaClient::require_auth_token(auth_token, &call.tool_id)?;
    let file_id = call.string_arg("fileId")?;

    let client = CloudRouterMediaClient::from_env();
    let sdk = client.with_auth_token(auth_token)?;
    client.with_trace_id(&sdk, call.trace_id.as_deref());
    let content = run_sync(&call.tool_id, |runtime| {
        runtime.block_on(sdk.files().content(&file_id))
    })?;

    Ok(MediaToolResult::succeeded(
        &call.tool_call_id,
        serde_json::json!({
            "fileId": file_id,
            "content": content,
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
            tool_id: "file.not.a.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_file_tool(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn upload_requires_auth_token_and_file_argument() {
        let no_token = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: tool_ids::UPLOAD.to_string(),
            arguments: serde_json::json!({ "file": "https://cdn.example/a.mp3", "purpose": "audio" }),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_file_tool(&no_token, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");

        let no_file = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::UPLOAD.to_string(),
            arguments: serde_json::json!({ "purpose": "audio" }),
            session_id: None,
            trace_id: None,
        };
        let error = invoke_file_tool(&no_file, Some("token")).expect_err("file required");
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn id_based_tools_require_file_id_argument() {
        for tool_id in [tool_ids::RETRIEVE, tool_ids::DELETE, tool_ids::CONTENT] {
            let call = MediaToolCall {
                tool_call_id: "call.4".to_string(),
                tool_id: tool_id.to_string(),
                arguments: serde_json::json!({}),
                session_id: None,
                trace_id: None,
            };
            let error = invoke_file_tool(&call, Some("token")).expect_err("fileId required");
            assert_eq!(error.code(), "invalid_input", "{tool_id}");
        }
    }
}
