//! Application-level media tool registry.
//!
//! Aggregates the five category providers (audio/video/music/sound-effect/
//! image) into one lookup surface and routes tool calls by `tool_id`. The
//! registry is the application-layer execution boundary for the media tool
//! family: callers (turn pipeline, future HTTP surfaces) resolve a tool,
//! validate its definition, and invoke it with the caller auth token — the
//! token never enters model-visible arguments.

use std::collections::HashMap;

use sdkwork_agents_tool_audio::AudioMediaToolProvider;
use sdkwork_agents_tool_contract::{
    MediaAuthTokenResolver, MediaToolCall, MediaToolDefinition, MediaToolError, MediaToolProvider,
    MediaToolResult, ToolCategory,
};
use sdkwork_agents_tool_file::FileMediaToolProvider;
use sdkwork_agents_tool_image::ImageMediaToolProvider;
use sdkwork_agents_tool_intelligence::IntelligenceMediaToolProvider;
use sdkwork_agents_tool_music::MusicMediaToolProvider;
use sdkwork_agents_tool_sound_effect::SoundEffectMediaToolProvider;
use sdkwork_agents_tool_video::VideoMediaToolProvider;

/// Aggregated media tool registry for the SDKWork Agents application.
///
/// Providers are registered by category; new categories extend the family by
/// adding a provider here (and its sub-crate dependency) without changing any
/// existing category.
#[derive(Debug, Default)]
pub struct MediaToolRegistry {
    providers: Vec<Box<dyn MediaToolProvider>>,
    by_tool_id: HashMap<String, usize>,
}

impl MediaToolRegistry {
    /// Registry with every cloudrouter-backed category provider registered.
    pub fn new() -> Self {
        Self::default()
            .with_provider(Box::new(AudioMediaToolProvider::without_auth_resolver()))
            .with_provider(Box::new(VideoMediaToolProvider::without_auth_resolver()))
            .with_provider(Box::new(MusicMediaToolProvider::without_auth_resolver()))
            .with_provider(Box::new(SoundEffectMediaToolProvider::new()))
            .with_provider(Box::new(ImageMediaToolProvider::without_auth_resolver()))
            .with_provider(Box::new(FileMediaToolProvider::without_auth_resolver()))
            .with_provider(Box::new(
                IntelligenceMediaToolProvider::without_auth_resolver(),
            ))
    }

    /// Registers one category provider (builder style).
    pub fn with_provider(mut self, provider: Box<dyn MediaToolProvider>) -> Self {
        let index = self.providers.len();
        for definition in provider.definitions() {
            self.by_tool_id.insert(definition.tool_id.clone(), index);
        }
        self.providers.push(provider);
        self
    }

    /// All registered category ids.
    pub fn categories(&self) -> Vec<ToolCategory> {
        self.providers
            .iter()
            .map(|provider| provider.category())
            .collect()
    }

    /// Static definitions for every registered tool.
    pub fn list_tools(&self) -> Vec<MediaToolDefinition> {
        self.providers
            .iter()
            .flat_map(|provider| provider.definitions())
            .collect()
    }

    /// Resolves one tool definition by stable tool id.
    pub fn describe_tool(&self, tool_id: &str) -> Option<MediaToolDefinition> {
        self.by_tool_id.get(tool_id).and_then(|&index| {
            self.providers.get(index).and_then(|provider| {
                provider
                    .definitions()
                    .into_iter()
                    .find(|definition| definition.tool_id == tool_id)
            })
        })
    }

    /// Executes one tool call, routing by `tool_id` to its category provider.
    ///
    /// `auth_token` is the caller's cloudrouter auth token; providers require
    /// it for every upstream call.
    pub fn invoke(
        &self,
        call: &MediaToolCall,
        auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError> {
        let &index = self.by_tool_id.get(&call.tool_id).ok_or_else(|| {
            MediaToolError::CapabilityMissing(format!(
                "media tool registry has no tool `{}`; registered categories: {}",
                call.tool_id,
                self.categories()
                    .iter()
                    .map(|category| category.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;
        let provider = &self.providers[index];
        provider.invoke(call, auth_token)
    }
}

/// Session-scoped auth token resolver backed by an in-memory map.
///
/// The application layer populates the store when a turn/request carries an
/// auth token; kernel `ToolProvider` projections on the category providers
/// resolve tokens through this store by session id.
#[derive(Debug, Default)]
pub struct SessionMediaAuthTokenStore {
    tokens: std::sync::Mutex<HashMap<String, String>>,
}

impl SessionMediaAuthTokenStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the auth token for a session.
    pub fn set(&self, session_id: impl Into<String>, token: impl Into<String>) {
        self.tokens
            .lock()
            .unwrap()
            .insert(session_id.into(), token.into());
    }

    /// Removes the auth token for a session.
    pub fn remove(&self, session_id: &str) {
        self.tokens.lock().unwrap().remove(session_id);
    }
}

impl MediaAuthTokenResolver for SessionMediaAuthTokenStore {
    fn resolve(&self, session_id: Option<&str>) -> Option<String> {
        session_id.and_then(|session_id| self.tokens.lock().unwrap().get(session_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agents_tool_contract::MediaToolCall;

    #[test]
    fn registry_registers_all_categories() {
        let registry = MediaToolRegistry::new();
        let categories = registry.categories();
        assert_eq!(
            categories,
            vec![
                ToolCategory::Audio,
                ToolCategory::Video,
                ToolCategory::Music,
                ToolCategory::SoundEffect,
                ToolCategory::Image,
                ToolCategory::File,
                ToolCategory::Intelligence,
            ]
        );
    }

    #[test]
    fn registry_lists_and_describes_all_tools() {
        let registry = MediaToolRegistry::new();
        let tools = registry.list_tools();
        assert!(!tools.is_empty());
        assert!(
            tools.len() >= 25,
            "expected full family, got {}",
            tools.len()
        );

        for tool in &tools {
            let described = registry.describe_tool(&tool.tool_id);
            assert_eq!(described.as_ref().map(|d| &d.tool_id), Some(&tool.tool_id));
        }
        assert!(registry.describe_tool("unknown.tool").is_none());
    }

    #[test]
    fn registry_routes_calls_by_tool_id() {
        let registry = MediaToolRegistry::new();

        // Missing auth token -> auth_required from the audio provider.
        let call = MediaToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "audio.speech.create".to_string(),
            arguments: serde_json::json!({ "input": "hello" }),
            session_id: None,
            trace_id: None,
        };
        let error = registry.invoke(&call, None).expect_err("auth required");
        assert_eq!(error.code(), "auth_required");

        // Unknown tool id -> capability_missing from the registry itself.
        let unknown = MediaToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: "unknown.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
        };
        let error = registry
            .invoke(&unknown, Some("token"))
            .expect_err("unknown tool");
        assert_eq!(error.code(), "capability_missing");
        assert!(error.to_string().contains("audio"));
    }

    #[test]
    fn session_store_resolves_by_session_id() {
        let store = SessionMediaAuthTokenStore::new();
        assert_eq!(store.resolve(None), None);
        store.set("session.1", "token-1");
        assert_eq!(store.resolve(Some("session.1")).as_deref(), Some("token-1"));
        assert_eq!(store.resolve(Some("session.2")), None);
        store.remove("session.1");
        assert_eq!(store.resolve(Some("session.1")), None);
    }
}
