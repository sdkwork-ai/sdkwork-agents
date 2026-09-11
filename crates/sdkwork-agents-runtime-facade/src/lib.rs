//! Application-owned facade for bootstrapping and executing kernel agent-engine providers.
//!
//! Product repositories (BirdCoder, IM PC) must depend on this crate instead of
//! importing `sdkwork-agent-provider-*` or `sdkwork-agent-kernel` types directly.

mod agent_engine_catalog;
mod agent_engine_config;
mod agent_engines;
mod error;
mod live_interaction;
mod provider_sessions;
mod runtime_host;
mod sessions;
mod turn;

pub use agent_engine_config::{read_agent_engine_config_file, AgentEngineConfigFile};

pub use agent_engine_catalog::{
    bootstrap_bootstrappable_agent_engine_catalog, bootstrap_canonical_agent_engine_catalog,
    build_agent_engine_catalog, list_slot_catalog_entries, model_descriptor_to_catalog_entry,
    AgentEngineAccessModeCatalogEntry, AgentEngineCatalog, AgentEngineCatalogEngine,
    AgentEngineModelCatalogEntry,
};
pub use agent_engines::{
    agent_engine_agent_id, agent_engine_binding_id, agent_engine_provider_scope,
    apply_agent_engine_model_configuration, apply_agent_engine_model_selection,
    bootstrap_agent_engine, bootstrap_rig_agent_engine, bootstrappable_engine_keys,
    canonical_agent_engine_keys, codex_engine_enabled, dematerialize_agent_engine_model_configuration,
    is_canonical_agent_engine, plan_agent_engine_configuration_upgrade,
    read_agent_engine_model_configuration, resolve_agent_engine_runtime_identity,
    AgentEngineBootstrapError, AgentEngineInteractionResolution, AgentEngineRuntimeIdentity,
    AgentEngineSlot, CANONICAL_AGENT_ENGINE_KEYS,
};
pub use error::{RuntimeFacadeError, RuntimeFacadeResult};
pub use live_interaction::{
    ApprovalDecision, EngineLiveInteraction, LiveInteractionRegistry, UserQuestionAnswer,
};
pub use provider_sessions::{
    ProviderSessionDirectoryEntry, ProviderSessionInventoryIssue, ProviderSessionInventoryItem,
    ProviderSessionInventorySelector, ProviderSessionInventorySnapshot,
    ProviderSessionProjectCwdResolver, ProviderSessionProjectCwdSelector,
};
pub use runtime_host::AgentsAgentEngineHost;
pub use sdkwork_agent_kernel::{
    AgentConfigurationProfile, AgentConfigurationStore, AgentModelConfigurationApplication,
    AgentModelConfigurationRequest, AgentModelSelectionRequest, InMemoryAgentConfigurationStore,
    KernelError, KernelResult, ModelDescriptor, ModelResponseFormat, ModelStreamChunk,
    ModelStreamSink, ToolCall,
};
// Media tool family (cloudrouter open-api backed): the facade is the only
// allowed import surface for product repositories.
pub use sdkwork_agents_tool_audio::{
    audio_tool_definitions, AudioMediaToolProvider, AUDIO_PROVIDER_ID,
};
pub use sdkwork_agents_tool_cloudrouter::{
    map_cloudrouter_error, run_sync, CloudRouterMediaClient, DEFAULT_CLOUDROUTER_BASE_URL,
    ENV_CLOUDROUTER_BASE_URL,
};
pub use sdkwork_agents_tool_contract::{
    MediaAuthTokenResolver, MediaResource, MediaToolCall, MediaToolDefinition, MediaToolError,
    MediaToolProvider, MediaToolResult, StaticMediaAuthTokenResolver, ToolAvailability,
    ToolCategory,
};
pub use sdkwork_agents_tool_file::{
    file_tool_definitions, FileMediaToolProvider, FILE_PROVIDER_ID,
};
pub use sdkwork_agents_tool_image::{
    image_tool_definitions, ImageMediaToolProvider, IMAGE_PROVIDER_ID,
};
pub use sdkwork_agents_tool_intelligence::{
    intelligence_tool_definitions, IntelligenceMediaToolProvider, INTELLIGENCE_PROVIDER_ID,
};
pub use sdkwork_agents_tool_music::{
    music_tool_definitions, MusicMediaToolProvider, MUSIC_PROVIDER_ID,
};
pub use sdkwork_agents_tool_sound_effect::{
    sound_effect_tool_definitions, SoundEffectMediaToolProvider, SOUND_EFFECT_PROVIDER_ID,
};
pub use sdkwork_agents_tool_video::{
    video_tool_definitions, VideoMediaToolProvider, VIDEO_PROVIDER_ID,
};
pub use sessions::*;
pub use turn::{
    agent_engine_model_request_id, cancel_agent_engine_turn, execute_agent_engine_turn,
    execute_agent_engine_turn_with_stream, execute_agent_engine_turn_with_stream_sink,
    AgentEngineTurnCancellation, AgentEngineTurnInput, AgentEngineTurnOutput,
    AgentEngineTurnStreamCompletion, MAX_AGENT_ENGINE_MODEL_REQUEST_ID_BYTES,
};
