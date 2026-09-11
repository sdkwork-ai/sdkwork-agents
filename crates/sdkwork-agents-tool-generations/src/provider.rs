//! The unified generations tool provider.
//!
//! Wraps the sdkwork-generations service and exposes image/video/music/sfx/voice
//! generation tools. Unlike the single-category providers (image, video, music),
//! this provider spans several categories, so it is identified by a stable
//! provider id (`generations`) rather than a single `ToolCategory`.

use sdkwork_agents_tool_contract::{MediaToolCall, MediaToolError, MediaToolResult};

use crate::definitions::generations_tool_definitions;
use crate::invoke::invoke_generations_tool;

/// Provider id for the unified generations provider.
pub const GENERATIONS_PROVIDER_ID: &str = "generations";

/// Unified generations tool provider (cloudrouter open-api backed).
///
/// Spans the image, video, music, sound-effect, and voice categories under one
/// provider id. Each tool definition carries its own category for dispatch and
/// authorization; the provider itself is category-agnostic.
#[derive(Debug, Clone)]
pub struct GenerationsToolProvider;

impl GenerationsToolProvider {
    /// Builds the provider with default configuration.
    pub fn new() -> Self {
        Self
    }

    /// Stable provider id.
    pub fn id(&self) -> &str {
        GENERATIONS_PROVIDER_ID
    }

    /// Human-readable provider name.
    pub fn name(&self) -> &str {
        "Generations"
    }

    /// Static tool definitions for every generations tool.
    pub fn definitions(&self) -> Vec<sdkwork_agents_tool_contract::MediaToolDefinition> {
        generations_tool_definitions()
    }

    /// Executes one tool call against the cloudrouter gateway.
    ///
    /// `auth_token` is the caller's cloudrouter auth token (login identity);
    /// synchronous tools return `succeeded`/`failed` results, async task tools
    /// return `succeeded` with a `taskId` in the output for the poll tool.
    pub fn invoke(
        &self,
        call: &MediaToolCall,
        auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError> {
        invoke_generations_tool(call, auth_token)
    }
}

impl Default for GenerationsToolProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::tool_ids;

    #[test]
    fn provider_exposes_all_generations_tools() {
        let provider = GenerationsToolProvider::new();
        let definitions = provider.definitions();
        let ids: Vec<&str> = definitions
            .iter()
            .map(|definition| definition.tool_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                tool_ids::IMAGE_TEXT_TO_IMAGE,
                tool_ids::IMAGE_EDIT,
                tool_ids::VIDEO_TEXT_TO_VIDEO,
                tool_ids::VIDEO_IMAGE_TO_VIDEO,
                tool_ids::VIDEO_EXTEND,
                tool_ids::MUSIC_TEXT_TO_MUSIC,
                tool_ids::MUSIC_LYRICS_TO_MUSIC,
                tool_ids::SFX_CREATE,
                tool_ids::VOICE_SPEECH,
                tool_ids::VOICE_TRANSCRIPTION,
                tool_ids::VOICE_TRANSLATION,
            ]
        );
        assert_eq!(provider.id(), GENERATIONS_PROVIDER_ID);
        assert_eq!(provider.name(), "Generations");
    }

    #[test]
    fn invoke_routes_to_image_tool() {
        let provider = GenerationsToolProvider::new();
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: tool_ids::IMAGE_TEXT_TO_IMAGE.to_string(),
            arguments: serde_json::json!({ "prompt": "a red fox" }),
            session_id: None,
            trace_id: None,
        };
        // Without an auth token the call fails closed before any network access.
        let error = provider.invoke(&call, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");
    }

    #[test]
    fn invoke_routes_to_unknown_tool() {
        let provider = GenerationsToolProvider::new();
        let call = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: "generations.unknown".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = provider.invoke(&call, Some("token")).expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn invoke_routes_sfx_to_pending_capability() {
        let provider = GenerationsToolProvider::new();
        let call = MediaToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: tool_ids::SFX_CREATE.to_string(),
            arguments: serde_json::json!({ "prompt": "thunder" }),
            session_id: None,
            trace_id: None,
        };
        let error = provider.invoke(&call, Some("token")).expect_err("sfx pending");
        assert_eq!(error.code(), "capability_missing");
    }

    #[test]
    fn default_provider_matches_new() {
        let default = GenerationsToolProvider::default();
        let built = GenerationsToolProvider::new();
        assert_eq!(default.id(), built.id());
        assert_eq!(default.name(), built.name());
        assert_eq!(default.definitions().len(), built.definitions().len());
    }
}
