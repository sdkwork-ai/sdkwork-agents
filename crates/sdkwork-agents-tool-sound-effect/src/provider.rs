//! The sound-effect category media tool provider.
//!
//! The category taxonomy and tool definition are in place; invocation
//! deliberately fails with `capability_missing` because the cloudrouter
//! open-api surface has no sound-effect endpoint yet. When the upstream
//! surface opens, the invocation logic slots into this crate without touching
//! any other category.

use sdkwork_agent_kernel::{
    KernelResult, ProviderHealth, ProviderManifest, ToolCall, ToolDescriptor, ToolProvider,
    ToolResult,
};
use sdkwork_agents_tool_contract::{
    media_tool_call, project_invoke_result, project_kernel_error, project_tool_descriptor,
    MediaToolCall, MediaToolDefinition, MediaToolError, MediaToolProvider, MediaToolResult,
    ToolCategory,
};

use crate::definitions::{sound_effect_tool_definitions, tool_ids};

/// Provider id for the cloudrouter-backed sound-effect category.
pub const SOUND_EFFECT_PROVIDER_ID: &str = "cloudrouter.media.sound-effect";

/// Sound-effect category tool provider (descriptors; execution delegated).
#[derive(Debug, Default)]
pub struct SoundEffectMediaToolProvider;

impl SoundEffectMediaToolProvider {
    pub fn new() -> Self {
        Self
    }
}

impl MediaToolProvider for SoundEffectMediaToolProvider {
    fn category(&self) -> ToolCategory {
        ToolCategory::SoundEffect
    }

    fn definitions(&self) -> Vec<MediaToolDefinition> {
        sound_effect_tool_definitions()
    }

    fn invoke(
        &self,
        call: &MediaToolCall,
        _auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError> {
        match call.tool_id.as_str() {
            tool_ids::GENERATE => Err(MediaToolError::CapabilityMissing(
                "sound-effect.generate executes in the agents media tool family \
                 executor (generations HTTP); this provider carries descriptors only"
                    .to_string(),
            )),
            other => Err(MediaToolError::CapabilityMissing(format!(
                "sound-effect provider has no tool `{other}`"
            ))),
        }
    }
}

impl ToolProvider for SoundEffectMediaToolProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            SOUND_EFFECT_PROVIDER_ID,
            "tool",
            "Sound-effect media tools",
            "0.1.0",
            vec![
                "tool.invoke".to_string(),
                "media.sound-effect.generate".to_string(),
            ],
        )
    }

    fn list_tools(&self) -> Vec<ToolDescriptor> {
        sound_effect_tool_definitions()
            .iter()
            .map(|definition| project_tool_descriptor(definition, SOUND_EFFECT_PROVIDER_ID))
            .collect()
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }

    fn invoke_tool(&self, call: ToolCall) -> KernelResult<ToolResult> {
        let media_call: MediaToolCall = media_tool_call(&call)?;
        let outcome = self.invoke(&media_call, None);
        project_invoke_result(&call.tool_call_id, outcome)
    }

    fn describe_tool(&self, tool_id: &str) -> KernelResult<ToolDescriptor> {
        sound_effect_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_id)
            .map(|definition| project_tool_descriptor(&definition, SOUND_EFFECT_PROVIDER_ID))
            .ok_or_else(|| {
                project_kernel_error(MediaToolError::CapabilityMissing(format!(
                    "sound-effect provider has no tool `{tool_id}`"
                )))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_exposes_the_sound_effect_tool() {
        let provider = SoundEffectMediaToolProvider::new();
        let descriptors = provider.list_tools();
        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].tool_id, tool_ids::GENERATE);
        assert_eq!(descriptors[0].provider_id, SOUND_EFFECT_PROVIDER_ID);
    }

    #[test]
    fn invocation_reports_the_executor_delegation() {
        // Execution lives in the agents media tool family executor (generations
        // HTTP); this provider carries descriptors only.
        let provider = SoundEffectMediaToolProvider::new();
        let call = ToolCall::new(
            "call.1",
            tool_ids::GENERATE,
            r#"{"prompt":"thunder rumble"}"#,
        );
        let error = provider.invoke_tool(call).expect_err("descriptor-only provider");
        assert_eq!(
            error.kind(),
            sdkwork_agent_kernel::KernelErrorKind::CapabilityMissing
        );
        assert!(error.to_string().contains("media tool family"));
    }

    #[test]
    fn describe_known_and_unknown_tools() {
        let provider = SoundEffectMediaToolProvider::new();
        assert!(provider.describe_tool(tool_ids::GENERATE).is_ok());
        assert!(provider.describe_tool("sound-effect.missing").is_err());
    }
}
