//! Tool definitions for the sound-effect category.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for the sound-effect category tools.
pub mod tool_ids {
    pub const GENERATE: &str = "sound-effect.generate";
}

const VERSION: &str = "0.1.0";

/// Static definitions for the sound-effect category.
///
/// The tool is fully available: the cloudrouter gateway exposes
/// `POST /app/v3/api/generations/sound_effects` and the ElevenLabs adapter
/// dispatches it (the tool used to be availability-gated as
/// `PendingCapability` before that surface opened).
pub fn sound_effect_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![generate_definition()]
}

fn generate_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::GENERATE.to_string(),
        category: ToolCategory::SoundEffect,
        name: "generate".to_string(),
        display_name: "Generate Sound Effect".to_string(),
        version: VERSION.to_string(),
        description: "Generates a sound effect from a text description (thunder, \
                      rain, café ambience, …) and returns a playable audio asset."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Natural-language description of the sound effect."
                },
                "model": {
                    "type": "string",
                    "description": "Sound-effect model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "duration": {
                    "type": "number",
                    "description": "Requested duration in seconds."
                }
            },
            "required": ["prompt"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "kind": { "type": "string", "const": "audio" },
                "source": { "type": "string", "const": "provider_asset" },
                "url": { "type": "string", "description": "Sound-effect asset delivery URL." }
            },
            "required": ["kind", "source", "url"]
        }),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::SoundEffect.policy_category("generate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_effect_tool_is_defined_but_pending() {
        let definitions = sound_effect_tool_definitions();
        assert_eq!(definitions.len(), 1);
        let definition = &definitions[0];
        assert_eq!(definition.tool_id, tool_ids::GENERATE);
        assert_eq!(definition.category, ToolCategory::SoundEffect);
        assert_eq!(
            definition.input_schema["required"],
            serde_json::json!(["prompt"])
        );
        assert_eq!(definition.availability, ToolAvailability::Available);
        assert_eq!(
            definition.policy_categories,
            vec!["media.sound-effect.generate"]
        );
    }
}
