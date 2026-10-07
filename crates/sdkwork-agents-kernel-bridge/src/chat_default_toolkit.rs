//! Curated default toolkit for the built-in chat conversation agent.
//!
//! The full built-in tool surface unions two families with overlapping
//! capabilities and disjoint task-id spaces:
//!
//! - the federated generations MCP (`mcp__generations__*`), backed by the
//!   generations app API with durable generation records, and
//! - the gateway media mirror family (`image.*` / `video.*` / `audio.*` /
//!   `music.*` / `sound-effect.*`), backed by vendor-shaped passthrough
//!   routes whose task ids live in vendor namespaces.
//!
//! Exposing both to a free-chat model invites cross-family id confusion (for
//! example polling a `mcp__generations__video.create` generation id through a
//! vendor mirror retrieve tool). The chat default set therefore curates one
//! tool per intent, preferring the generations MCP family (durable records,
//! tenant billing, drive persistence) plus the two audio utilities the
//! generations family does not cover. Everything else stays available to
//! opt-in agents through enabled Tool composition slots
//! (`TurnToolkitConfig::all_builtin_tools`).

use sdkwork_intelligence_agents_service::TurnToolDescriptor;

/// Built-in tool ids every chat conversation agent gets by default.
///
/// Keep in sync with [`super::chat_agent_prompt`] on the service side: the
/// canonical chat system prompt references exactly these tools, and the unit
/// tests below plus the service-side prompt tests enforce the pairing.
pub const CHAT_DEFAULT_TOOL_IDS: [&str; 10] = [
    "mcp__generations__image.create",
    "mcp__generations__image.retrieve",
    "mcp__generations__video.create",
    "mcp__generations__video.retrieve",
    "mcp__generations__music.create",
    "mcp__generations__music.retrieve",
    "mcp__generations__speech.create",
    "sound-effect.generate",
    "audio.transcriptions.create",
    "audio.translations.create",
];

/// Filters the full built-in descriptor set down to the curated chat default.
///
/// Descriptor order follows [`CHAT_DEFAULT_TOOL_IDS`] so the model sees a
/// stable, intent-ordered tool list. Ids missing from `all` (an executor not
/// wired in this deployment) are skipped rather than erroring — the toolkit
/// resolution must never fail a turn over a missing optional tool.
pub fn curated_chat_tools(all: &[TurnToolDescriptor]) -> Vec<TurnToolDescriptor> {
    CHAT_DEFAULT_TOOL_IDS
        .iter()
        .filter_map(|tool_id| {
            all.iter()
                .find(|descriptor| descriptor.tool_id == *tool_id)
                .cloned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_intelligence_agents_service::TurnToolOrigin;

    fn descriptor(tool_id: &str) -> TurnToolDescriptor {
        TurnToolDescriptor {
            tool_id: tool_id.to_string(),
            name: tool_id.to_string(),
            description: format!("descriptor for {tool_id}"),
            input_schema: serde_json::json!({ "type": "object" }),
            requires_approval: false,
            policy_category: None,
            timeout_ms: 30_000,
            origin: if tool_id.starts_with("mcp__generations__") {
                TurnToolOrigin::BuiltinGenerations
            } else {
                TurnToolOrigin::BuiltinMedia
            },
        }
    }

    fn full_builtin_fixture() -> Vec<TurnToolDescriptor> {
        vec![
            descriptor("mcp__generations__image.create"),
            descriptor("mcp__generations__image.retrieve"),
            descriptor("mcp__generations__video.create"),
            descriptor("mcp__generations__video.retrieve"),
            descriptor("mcp__generations__music.create"),
            descriptor("mcp__generations__music.retrieve"),
            descriptor("mcp__generations__speech.create"),
            descriptor("sound-effect.generate"),
            descriptor("audio.speech.create"),
            descriptor("audio.transcriptions.create"),
            descriptor("audio.translations.create"),
            descriptor("audio.voices.list"),
            descriptor("video.kling.generations.create"),
            descriptor("video.kling.generations.retrieve"),
            descriptor("image.midjourney.generations.create"),
            descriptor("file.upload"),
            descriptor("model.list"),
        ]
    }

    #[test]
    fn curated_set_matches_chat_default_ids_exactly() {
        let curated = curated_chat_tools(&full_builtin_fixture());
        let ids: Vec<&str> = curated.iter().map(|tool| tool.tool_id.as_str()).collect();
        assert_eq!(ids, CHAT_DEFAULT_TOOL_IDS);
    }

    #[test]
    fn curated_set_excludes_vendor_mirror_and_utility_families() {
        let curated = curated_chat_tools(&full_builtin_fixture());
        for excluded in [
            "video.kling.generations.create",
            "image.midjourney.generations.create",
            "audio.speech.create",
            "audio.voices.list",
            "file.upload",
            "model.list",
        ] {
            assert!(
                !curated.iter().any(|tool| tool.tool_id == excluded),
                "vendor mirror tool `{excluded}` must not be in the chat default set"
            );
        }
    }

    #[test]
    fn curated_set_skips_missing_executors_without_failing() {
        let partial = vec![descriptor("mcp__generations__image.create")];
        let curated = curated_chat_tools(&partial);
        assert_eq!(curated.len(), 1);
        assert_eq!(curated[0].tool_id, "mcp__generations__image.create");
    }

    #[test]
    fn curated_set_is_empty_without_builtin_tools() {
        assert!(curated_chat_tools(&[]).is_empty());
    }
}
