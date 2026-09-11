//! Kernel-compatible projections from the media tool contract.
//!
//! Every category crate projects its [`MediaToolDefinition`] onto
//! `sdkwork_agent_kernel::ToolDescriptor` and its results/errors onto the
//! kernel types so the same provider can be registered into an `AgentRuntime`
//! later without change.

use sdkwork_agent_kernel::{
    KernelError, KernelResult, SideEffectLevel, ToolCall, ToolDescriptor, ToolResult, ToolSchema,
};

use crate::call::MediaToolResult;
use crate::definition::MediaToolDefinition;
use crate::error::MediaToolError;

/// JSON Schema dialect declared on every projected tool schema.
pub const JSON_SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Maps a media tool side-effect level string to the kernel enum.
pub fn side_effect_level(value: &str) -> SideEffectLevel {
    match value {
        "read_only" => SideEffectLevel::ReadOnly,
        "destructive" => SideEffectLevel::Destructive,
        "external_send" => SideEffectLevel::ExternalSend,
        "privileged" => SideEffectLevel::Privileged,
        _ => SideEffectLevel::SideEffectful,
    }
}

/// Projects a media tool definition onto a kernel `ToolDescriptor`.
pub fn project_tool_descriptor(
    definition: &MediaToolDefinition,
    provider_id: &str,
) -> ToolDescriptor {
    let input_schema = ToolSchema::json_schema(format!("{}.input", definition.tool_id))
        .with_document(definition.input_schema.clone())
        .with_dialect(JSON_SCHEMA_DIALECT);
    let output_schema = ToolSchema::json_schema(format!("{}.output", definition.tool_id))
        .with_document(definition.output_schema.clone())
        .with_dialect(JSON_SCHEMA_DIALECT);

    ToolDescriptor::new(
        definition.tool_id.clone(),
        provider_id,
        definition.display_name.clone(),
        side_effect_level(&definition.side_effect_level),
    )
    .with_name(definition.name.clone())
    .with_version(definition.version.clone())
    .with_description(definition.description.clone())
    .with_input_schema(input_schema)
    .with_output_schema(output_schema)
    .with_policy_categories(definition.policy_categories.clone())
    .with_timeout_ms(definition.timeout_ms)
}

/// Maps a media tool error onto the kernel error taxonomy.
pub fn project_kernel_error(error: MediaToolError) -> KernelError {
    match error {
        MediaToolError::InvalidInput(message) => KernelError::validation(message),
        MediaToolError::CapabilityMissing(message) => KernelError::CapabilityMissing {
            capability_id: message,
        },
        MediaToolError::AuthRequired(message) => KernelError::permission_required(message),
        MediaToolError::ProviderUnavailable(message) => KernelError::ProviderUnavailable {
            provider_id: message,
        },
        MediaToolError::ProviderError(message) => {
            KernelError::provider_error("provider_error", message)
        }
        MediaToolError::Timeout(message) => KernelError::timeout(message),
        MediaToolError::RateLimited(message) => KernelError::rate_limited(message),
    }
}

/// Converts a kernel `ToolCall` into a media tool call, parsing the JSON
/// argument text and validating the tool id.
pub fn media_tool_call(call: &ToolCall) -> Result<crate::MediaToolCall, KernelError> {
    let arguments: serde_json::Value = serde_json::from_str(&call.arguments).map_err(|error| {
        KernelError::validation(format!(
            "tool call `{}` arguments are not valid JSON: {error}",
            call.tool_id
        ))
    })?;
    Ok(crate::MediaToolCall {
        tool_call_id: call.tool_call_id.clone(),
        tool_id: call.tool_id.clone(),
        arguments,
        session_id: call.session_id.clone(),
        trace_id: call
            .trace_context
            .as_ref()
            .map(|context| context.trace_id.clone()),
    })
}

/// Converts a media tool result into a kernel `ToolResult`.
pub fn project_tool_result(
    tool_call_id: impl Into<String>,
    result: &MediaToolResult,
) -> ToolResult {
    let tool_call_id = tool_call_id.into();
    match result.status.as_str() {
        "failed" => ToolResult::failed(tool_call_id, result.error.clone().unwrap_or_default()),
        _ => ToolResult::succeeded(tool_call_id, result.output.to_string()),
    }
}

/// Convenience: project a media tool invocation into a kernel result,
/// mapping errors to kernel errors.
pub fn project_invoke_result(
    tool_call_id: impl Into<String>,
    outcome: Result<MediaToolResult, MediaToolError>,
) -> KernelResult<ToolResult> {
    match outcome {
        Ok(result) => Ok(project_tool_result(tool_call_id, &result)),
        Err(error) => Err(project_kernel_error(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MediaResource, ToolAvailability, ToolCategory};

    fn sample_definition() -> MediaToolDefinition {
        MediaToolDefinition {
            tool_id: "audio.speech.create".to_string(),
            category: ToolCategory::Audio,
            name: "speech.create".to_string(),
            display_name: "Text to Speech".to_string(),
            version: "0.1.0".to_string(),
            description: "Synthesizes speech.".to_string(),
            input_schema: serde_json::json!({ "type": "object" }),
            output_schema: serde_json::json!({ "type": "object" }),
            side_effect_level: "side_effectful".to_string(),
            policy_categories: vec!["media.audio.generate".to_string()],
            timeout_ms: 60_000,
            availability: ToolAvailability::Available,
        }
    }

    #[test]
    fn projects_descriptor_with_all_fields() {
        let descriptor = project_tool_descriptor(&sample_definition(), "cloudrouter.media.audio");
        assert_eq!(descriptor.tool_id, "audio.speech.create");
        assert_eq!(descriptor.provider_id, "cloudrouter.media.audio");
        assert_eq!(descriptor.side_effect_level, SideEffectLevel::SideEffectful);
        assert_eq!(descriptor.policy_categories, vec!["media.audio.generate"]);
        assert_eq!(descriptor.timeout_ms, Some(60_000));
        let schema = descriptor.input_schema.expect("input schema");
        assert_eq!(schema.dialect.as_deref(), Some(JSON_SCHEMA_DIALECT));
        assert!(schema.document_json().is_some());
    }

    #[test]
    fn maps_side_effect_strings() {
        assert_eq!(side_effect_level("read_only"), SideEffectLevel::ReadOnly);
        assert_eq!(
            side_effect_level("destructive"),
            SideEffectLevel::Destructive
        );
        assert_eq!(
            side_effect_level("external_send"),
            SideEffectLevel::ExternalSend
        );
        assert_eq!(side_effect_level("privileged"), SideEffectLevel::Privileged);
        assert_eq!(side_effect_level("unknown"), SideEffectLevel::SideEffectful);
    }

    #[test]
    fn projects_errors_to_kernel_kinds() {
        use sdkwork_agent_kernel::KernelErrorKind;

        assert_eq!(
            project_kernel_error(MediaToolError::invalid_argument("x")).kind(),
            KernelErrorKind::ValidationError
        );
        assert_eq!(
            project_kernel_error(MediaToolError::pending_capability("x")).kind(),
            KernelErrorKind::CapabilityMissing
        );
        assert_eq!(
            project_kernel_error(MediaToolError::AuthRequired("x".into())).kind(),
            KernelErrorKind::PermissionRequired
        );
    }

    #[test]
    fn parses_kernel_call_arguments() {
        let call = ToolCall::new("call.1", "audio.speech.create", r#"{"input":"hello"}"#);
        let media_call = media_tool_call(&call).expect("parses");
        assert_eq!(media_call.tool_call_id, "call.1");
        assert_eq!(media_call.arguments["input"], "hello");

        let invalid = ToolCall::new("call.2", "audio.speech.create", "not-json");
        assert!(media_tool_call(&invalid).is_err());
    }

    #[test]
    fn projects_results_and_outcomes() {
        let resource = MediaResource::provider_asset("audio", "https://cdn.example/a.mp3");
        let result = MediaToolResult::succeeded_with_resource("call.3", &resource);
        let kernel_result = project_tool_result("call.3", &result);
        assert_eq!(kernel_result.status, "succeeded");
        assert!(kernel_result.output.contains("cdn.example"));

        let outcome =
            project_invoke_result("call.4", Err(MediaToolError::invalid_argument("bad input")));
        assert!(outcome.is_err());
    }
}
