use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use sdkwork_agent_kernel::{
    AgentConfigValue, AgentConfiguration, AgentConfigurationProvider,
    AgentConfigurationUpgradePlan, AgentConfigurationUpgradeRequest,
    AgentExecutionSettingsRequest, AgentExecutionSettingsResolution, AgentExecutionSettingsSpec,
    AgentMessage, AgentMessageRole, AgentModelConfigurationApplication,
    AgentModelConfigurationRequest, AgentModelSelectionRequest, AgentSession, EnvFileSecretHostProvider,
    HostProvider, KernelError, KernelResult, ModelDescriptor, ModelProvider, ModelRequest,
    ModelResponse, ModelStreamChunk, ModelStreamSink, ProviderModelConfigurationStatus,
    ProviderSessionActivityProvider, SessionActivitySnapshot, SessionKind, SessionSource,
    SessionState,
};
use sdkwork_agent_provider_claude_code::{
    ClaudeCodeConfigurationProvider, ClaudeCodeSdkIntegration,
};
#[cfg(feature = "codex-provider")]
use sdkwork_agent_provider_codex::{
    CodexConfigurationProvider, CodexSdkIntegration, CodexSortDirection, ThreadListCwdFilter,
    ThreadListParams, ThreadTurnsListParams, TurnItemsView,
};
use sdkwork_agent_provider_core::{
    finalize_provider_session_snapshot, SessionLifecycleProvider, SessionListQuery,
};
use sdkwork_agent_provider_gemini_cli::{GeminiCliConfigurationProvider, GeminiCliSdkIntegration};
use sdkwork_agent_provider_hermes::{HermesConfigurationProvider, HermesSdkIntegration};
use sdkwork_agent_provider_mimo_code::{
    MiMoCodeConfigurationProvider, MiMoCodeSdkIntegration,
};
use sdkwork_agent_provider_openclaw::{OpenClawConfigurationProvider, OpenClawSdkIntegration};
use sdkwork_agent_provider_opencode::{OpenCodeConfigurationProvider, OpenCodeSdkIntegration};
use sdkwork_agent_provider_rig::{
    ids, RigBackendConfig, RigBackendMode, RigConfigurationProvider, RigModelProvider,
    RigSdkIntegration,
};
use sdkwork_agents_tool_cloudrouter::RigCloudRouterModelProvider;
use sdkwork_agent_provider_spi::{
    SdkRuntimeInteractionResolution, SdkRuntimeMessageRecord, SdkRuntimeSessionRecord,
    SdkRuntimeStreamCompletion, CLAUDE_CODE_BINDING_ID, CODEX_BINDING_ID, GEMINI_CLI_BINDING_ID,
    HERMES_BINDING_ID, MIMO_CODE_BINDING_ID, OPENCLAW_BINDING_ID, OPENCODE_BINDING_ID,
    RIG_BINDING_ID,
};

/// Canonical T1 agent-engine keys bootstrapped by default in production hosts.
pub const CANONICAL_AGENT_ENGINE_KEYS: [&str; 4] = ["codex", "claude-code", "gemini", "opencode"];

/// T2 autonomous agent engines plus additional kernel-wrapped providers
/// (bootstrap on demand; included in full catalog).
pub const EXTENDED_AUTONOMOUS_ENGINE_KEYS: [&str; 4] =
    ["openclaw", "hermes", "mimo-code", "rig"];

const MAX_PROVIDER_SESSION_COLLECTION_ITEMS: usize = 10_000;

pub fn canonical_agent_engine_keys() -> &'static [&'static str] {
    &CANONICAL_AGENT_ENGINE_KEYS
}

/// Whether the Codex provider is compiled into this build.
///
/// Codex carries an embedded client-local SQLite runtime store and is an
/// optional per-application integration; builds without the `codex-provider`
/// feature reject codex model configuration with `UnsupportedEngine`.
/// Catalog and lifecycle surfaces should skip or document codex operations
/// when this returns `false` instead of failing.
pub fn codex_engine_enabled() -> bool {
    cfg!(feature = "codex-provider")
}

pub fn bootstrappable_engine_keys() -> [&'static str; 8] {
    [
        CANONICAL_AGENT_ENGINE_KEYS[0],
        CANONICAL_AGENT_ENGINE_KEYS[1],
        CANONICAL_AGENT_ENGINE_KEYS[2],
        CANONICAL_AGENT_ENGINE_KEYS[3],
        EXTENDED_AUTONOMOUS_ENGINE_KEYS[0],
        EXTENDED_AUTONOMOUS_ENGINE_KEYS[1],
        EXTENDED_AUTONOMOUS_ENGINE_KEYS[2],
        EXTENDED_AUTONOMOUS_ENGINE_KEYS[3],
    ]
}

pub fn engine_catalog_tier(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" | "claude-code" | "gemini" | "opencode" => Some("t1-code"),
        "openclaw" | "hermes" | "mimo-code" | "rig" => Some("t2-agent"),
        _ => None,
    }
}

/// User-facing engine kind surfaced in the settings catalog so clients can
/// distinguish code agents from work agents and simple runtimes.
pub fn engine_catalog_kind(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" | "claude-code" | "gemini" | "opencode" | "mimo-code" => Some("code"),
        "openclaw" | "hermes" => Some("work"),
        "rig" => Some("simple"),
        _ => None,
    }
}

pub fn is_canonical_agent_engine(engine_key: &str) -> bool {
    CANONICAL_AGENT_ENGINE_KEYS.contains(&engine_key)
}

pub fn apply_agent_engine_model_configuration(
    engine_key: &str,
    request: &AgentModelConfigurationRequest,
) -> crate::RuntimeFacadeResult<AgentModelConfigurationApplication> {
    let _ = agent_engine_agent_id(engine_key).ok_or_else(|| {
        crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        }
    })?;
    // The engine key selects the provider implementation; the configuration
    // subject may be any non-blank agent id (a user-created agent such as
    // `agent.chat.default` or the canonical engine agent).
    if request.agent_id.trim().is_empty() {
        return Err(crate::RuntimeFacadeError::InvalidInput(
            "model configuration agentId must not be blank".to_string(),
        ));
    }

    let result = match engine_key {
        #[cfg(feature = "codex-provider")]
        "codex" => CodexConfigurationProvider::new().apply_model_configuration(request),
        #[cfg(not(feature = "codex-provider"))]
        "codex" => {
            return Err(crate::RuntimeFacadeError::UnsupportedEngine {
                engine_key: engine_key.to_string(),
            });
        }
        "claude-code" => ClaudeCodeConfigurationProvider::new().apply_model_configuration(request),
        "gemini" => GeminiCliConfigurationProvider::new().apply_model_configuration(request),
        "opencode" => OpenCodeConfigurationProvider::new().apply_model_configuration(request),
        "openclaw" => OpenClawConfigurationProvider::new().apply_model_configuration(request),
        "hermes" => HermesConfigurationProvider::new().apply_model_configuration(request),
        "mimo-code" => MiMoCodeConfigurationProvider::new().apply_model_configuration(request),
        "rig" => RigConfigurationProvider::new().apply_model_configuration(request),
        _ => unreachable!("validated agent engine"),
    };
    result.map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))
}

pub fn apply_agent_engine_model_selection(
    engine_key: &str,
    request: &AgentModelSelectionRequest,
) -> crate::RuntimeFacadeResult<AgentModelConfigurationApplication> {
    let _ = agent_engine_agent_id(engine_key).ok_or_else(|| {
        crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        }
    })?;
    // Same subject rule as model configuration: any non-blank agent id.
    if request.agent_id.trim().is_empty() {
        return Err(crate::RuntimeFacadeError::InvalidInput(
            "model selection agentId must not be blank".to_string(),
        ));
    }

    let result = match engine_key {
        #[cfg(feature = "codex-provider")]
        "codex" => CodexConfigurationProvider::new().apply_model_selection(request),
        #[cfg(not(feature = "codex-provider"))]
        "codex" => {
            return Err(crate::RuntimeFacadeError::UnsupportedEngine {
                engine_key: engine_key.to_string(),
            });
        }
        "claude-code" => ClaudeCodeConfigurationProvider::new().apply_model_selection(request),
        "gemini" => GeminiCliConfigurationProvider::new().apply_model_selection(request),
        "opencode" => OpenCodeConfigurationProvider::new().apply_model_selection(request),
        "openclaw" => OpenClawConfigurationProvider::new().apply_model_selection(request),
        "hermes" => HermesConfigurationProvider::new().apply_model_selection(request),
        "mimo-code" => MiMoCodeConfigurationProvider::new().apply_model_selection(request),
        "rig" => RigConfigurationProvider::new().apply_model_selection(request),
        _ => unreachable!("validated agent engine"),
    };
    result.map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))
}

/// Reads the currently effective model configuration back from the engine's
/// provider native config surface, so callers can detect drift and stale CLI
/// state relative to the stored profile.
pub fn read_agent_engine_model_configuration(
    engine_key: &str,
    agent_id: &str,
    profile_id: &str,
) -> crate::RuntimeFacadeResult<ProviderModelConfigurationStatus> {
    let expected_agent_id = agent_engine_agent_id(engine_key).ok_or_else(|| {
        crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        }
    })?;
    if agent_id != expected_agent_id {
        return Err(crate::RuntimeFacadeError::InvalidInput(format!(
            "model configuration agentId does not match engineId {engine_key}"
        )));
    }
    let result = match engine_key {
        #[cfg(feature = "codex-provider")]
        "codex" => CodexConfigurationProvider::new().read_model_configuration(agent_id, profile_id),
        #[cfg(not(feature = "codex-provider"))]
        "codex" => {
            return Err(crate::RuntimeFacadeError::UnsupportedEngine {
                engine_key: engine_key.to_string(),
            });
        }
        "claude-code" => {
            ClaudeCodeConfigurationProvider::new().read_model_configuration(agent_id, profile_id)
        }
        "gemini" => {
            GeminiCliConfigurationProvider::new().read_model_configuration(agent_id, profile_id)
        }
        "opencode" => {
            OpenCodeConfigurationProvider::new().read_model_configuration(agent_id, profile_id)
        }
        "openclaw" => {
            OpenClawConfigurationProvider::new().read_model_configuration(agent_id, profile_id)
        }
        "hermes" => {
            HermesConfigurationProvider::new().read_model_configuration(agent_id, profile_id)
        }
        "mimo-code" => {
            MiMoCodeConfigurationProvider::new().read_model_configuration(agent_id, profile_id)
        }
        "rig" => RigConfigurationProvider::new().read_model_configuration(agent_id, profile_id),
        _ => unreachable!("validated agent engine"),
    };
    result.map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))
}

/// Reverts the engine's materialized provider configuration (restoring the
/// pre-apply backup) when a profile is archived or removed.
pub fn dematerialize_agent_engine_model_configuration(
    engine_key: &str,
    agent_id: &str,
    profile_id: &str,
) -> crate::RuntimeFacadeResult<()> {
    let expected_agent_id = agent_engine_agent_id(engine_key).ok_or_else(|| {
        crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        }
    })?;
    if agent_id != expected_agent_id {
        return Err(crate::RuntimeFacadeError::InvalidInput(format!(
            "model configuration agentId does not match engineId {engine_key}"
        )));
    }
    let result = match engine_key {
        #[cfg(feature = "codex-provider")]
        "codex" => CodexConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        #[cfg(not(feature = "codex-provider"))]
        "codex" => {
            return Err(crate::RuntimeFacadeError::UnsupportedEngine {
                engine_key: engine_key.to_string(),
            });
        }
        "claude-code" => ClaudeCodeConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        "gemini" => GeminiCliConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        "opencode" => OpenCodeConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        "openclaw" => OpenClawConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        "hermes" => HermesConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        "mimo-code" => MiMoCodeConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        "rig" => RigConfigurationProvider::new()
            .dematerialize_model_configuration(agent_id, profile_id),
        _ => unreachable!("validated agent engine"),
    };
    result.map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))
}

/// Plans a configuration profile upgrade for the engine's provider. Providers
/// without a migration plan report the capability as missing.
pub fn plan_agent_engine_configuration_upgrade(
    engine_key: &str,
    request: &AgentConfigurationUpgradeRequest,
) -> crate::RuntimeFacadeResult<AgentConfigurationUpgradePlan> {
    let expected_agent_id = agent_engine_agent_id(engine_key).ok_or_else(|| {
        crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        }
    })?;
    if request.agent_id != expected_agent_id {
        return Err(crate::RuntimeFacadeError::InvalidInput(format!(
            "model configuration agentId does not match engineId {engine_key}"
        )));
    }
    let result = match engine_key {
        #[cfg(feature = "codex-provider")]
        "codex" => CodexConfigurationProvider::new().plan_configuration_upgrade(request),
        #[cfg(not(feature = "codex-provider"))]
        "codex" => {
            return Err(crate::RuntimeFacadeError::UnsupportedEngine {
                engine_key: engine_key.to_string(),
            });
        }
        "claude-code" => ClaudeCodeConfigurationProvider::new().plan_configuration_upgrade(request),
        "gemini" => GeminiCliConfigurationProvider::new().plan_configuration_upgrade(request),
        "opencode" => OpenCodeConfigurationProvider::new().plan_configuration_upgrade(request),
        "openclaw" => OpenClawConfigurationProvider::new().plan_configuration_upgrade(request),
        "hermes" => HermesConfigurationProvider::new().plan_configuration_upgrade(request),
        "mimo-code" => MiMoCodeConfigurationProvider::new().plan_configuration_upgrade(request),
        "rig" => RigConfigurationProvider::new().plan_configuration_upgrade(request),
        _ => unreachable!("validated agent engine"),
    };
    result.map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))
}

pub fn agent_engine_agent_id(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" => Some("agent.codex"),
        "claude-code" => Some("agent.claude-code"),
        "gemini" => Some("agent.gemini"),
        "opencode" => Some("agent.opencode"),
        "openclaw" => Some("agent.openclaw"),
        "hermes" => Some("agent.hermes"),
        "mimo-code" => Some("agent.mimo-code"),
        "rig" => Some("agent.rig-general"),
        _ => None,
    }
}

/// Provider configuration scope materialized into profile entry keys for an
/// engine (mirrors the provider `with_model_configuration_scope` values).
///
/// Scopes live in the profile-entry key namespace, which uses underscore
/// separators by design (`claude_code`, `gemini_cli`) — they are config keys,
/// not durable ids, and intentionally differ from the engine key spelling.
pub fn agent_engine_provider_scope(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" => Some("codex"),
        "claude-code" => Some("claude_code"),
        "gemini" => Some("gemini_cli"),
        "opencode" => Some("opencode"),
        "openclaw" => Some("openclaw"),
        "hermes" => Some("hermes"),
        "mimo-code" => Some("mimo_code"),
        "rig" => Some("rig"),
        _ => None,
    }
}

pub fn agent_engine_binding_id(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" => Some(CODEX_BINDING_ID),
        "claude-code" => Some(CLAUDE_CODE_BINDING_ID),
        "gemini" => Some(GEMINI_CLI_BINDING_ID),
        "opencode" => Some(OPENCODE_BINDING_ID),
        "openclaw" => Some(OPENCLAW_BINDING_ID),
        "hermes" => Some(HERMES_BINDING_ID),
        "mimo-code" => Some(MIMO_CODE_BINDING_ID),
        "rig" => Some(RIG_BINDING_ID),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentEngineRuntimeIdentity {
    pub engine_key: String,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentEngineInteractionResolution {
    pub model_request_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub provider_session_id: String,
    pub provider_turn_id: String,
    pub provider_request_id: serde_json::Value,
    pub resolution: serde_json::Value,
}

pub fn resolve_agent_engine_runtime_identity(
    agent_id: &str,
) -> Result<Option<AgentEngineRuntimeIdentity>, AgentEngineBootstrapError> {
    let Some(engine_key) = bootstrappable_engine_keys()
        .into_iter()
        .find(|engine_key| agent_engine_agent_id(engine_key) == Some(agent_id))
    else {
        return Ok(None);
    };
    let slot = bootstrap_agent_engine(engine_key)?;
    let provider_id = slot
        .list_model_descriptors()
        .into_iter()
        .next()
        .map(|descriptor| descriptor.provider_id)
        .ok_or_else(|| {
            AgentEngineBootstrapError::Bootstrap(format!(
                "agent engine {engine_key} did not publish a model provider"
            ))
        })?;
    Ok(Some(AgentEngineRuntimeIdentity {
        engine_key: engine_key.to_string(),
        agent_id: agent_id.to_string(),
        binding_id: slot.binding_id().to_string(),
        provider_id,
    }))
}

#[derive(Debug)]
pub enum AgentEngineBootstrapError {
    UnsupportedEngine(String),
    Bootstrap(String),
}

impl std::fmt::Display for AgentEngineBootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedEngine(engine) => {
                write!(f, "unsupported agent engine for bootstrap: {engine}")
            }
            Self::Bootstrap(message) => write!(f, "agent engine bootstrap failed: {message}"),
        }
    }
}

impl std::error::Error for AgentEngineBootstrapError {}

/// Bootstrapped kernel provider slot for one canonical agent engine.
pub enum AgentEngineSlot {
    #[cfg(feature = "codex-provider")]
    Codex(CodexSdkIntegration),
    ClaudeCode(ClaudeCodeSdkIntegration),
    Gemini(GeminiCliSdkIntegration),
    OpenCode(OpenCodeSdkIntegration),
    OpenClaw(OpenClawSdkIntegration),
    Hermes(HermesSdkIntegration),
    MiMoCode(MiMoCodeSdkIntegration),
    Rig(RigSdkIntegration),
}

impl AgentEngineSlot {
    pub fn engine_key(&self) -> &'static str {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(_) => "codex",
            Self::ClaudeCode(_) => "claude-code",
            Self::Gemini(_) => "gemini",
            Self::OpenCode(_) => "opencode",
            Self::OpenClaw(_) => "openclaw",
            Self::Hermes(_) => "hermes",
            Self::MiMoCode(_) => "mimo-code",
            Self::Rig(_) => "rig",
        }
    }

    pub fn binding_id(&self) -> &str {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => integration.binding_id(),
            Self::ClaudeCode(integration) => integration.binding_id(),
            Self::Gemini(integration) => integration.binding_id(),
            Self::OpenCode(integration) => integration.binding_id(),
            Self::OpenClaw(integration) => integration.binding_id(),
            Self::Hermes(integration) => integration.binding_id(),
            Self::MiMoCode(integration) => integration.binding_id(),
            Self::Rig(integration) => integration.binding_id(),
        }
    }

    pub fn list_model_ids(&self) -> Vec<String> {
        self.list_model_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.model_id)
            .collect()
    }

    pub fn list_model_descriptors(&self) -> Vec<ModelDescriptor> {
        self.model_provider().list_models()
    }

    pub fn execution_settings_spec(&self) -> KernelResult<AgentExecutionSettingsSpec> {
        let agent_id = agent_engine_agent_id(self.engine_key()).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: format!("agent.configure.execution.{}", self.engine_key()),
            }
        })?;
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(_) => CodexConfigurationProvider::new().execution_settings_spec(agent_id),
            Self::ClaudeCode(_) => {
                ClaudeCodeConfigurationProvider::new().execution_settings_spec(agent_id)
            }
            Self::Gemini(_) => {
                GeminiCliConfigurationProvider::new().execution_settings_spec(agent_id)
            }
            Self::OpenCode(_) => {
                OpenCodeConfigurationProvider::new().execution_settings_spec(agent_id)
            }
            Self::OpenClaw(_)
            | Self::Hermes(_)
            | Self::MiMoCode(_)
            | Self::Rig(_) => {
                Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                    capability_id: format!("agent.configure.execution.{}", self.engine_key()),
                })
            }
        }
    }

    pub fn resolve_execution_settings(
        &self,
        access_mode_id: &str,
    ) -> KernelResult<AgentExecutionSettingsResolution> {
        let agent_id = agent_engine_agent_id(self.engine_key()).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: format!("agent.configure.execution.{}", self.engine_key()),
            }
        })?;
        let request = AgentExecutionSettingsRequest::new(agent_id).with_access_mode(access_mode_id);
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(_) => {
                CodexConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::ClaudeCode(_) => {
                ClaudeCodeConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::Gemini(_) => {
                GeminiCliConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::OpenCode(_) => {
                OpenCodeConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::OpenClaw(_)
            | Self::Hermes(_)
            | Self::MiMoCode(_)
            | Self::Rig(_) => {
                Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                    capability_id: format!("agent.configure.execution.{}", self.engine_key()),
                })
            }
        }
    }

    pub fn invoke_model(&self, request: ModelRequest) -> KernelResult<ModelResponse> {
        self.model_provider().invoke(request)
    }

    pub fn cancel_model(&self, model_request_id: &str) -> KernelResult<ModelResponse> {
        self.model_provider().cancel(model_request_id)
    }

    pub fn stream_model(&self, request: ModelRequest) -> KernelResult<Vec<ModelStreamChunk>> {
        self.model_provider().stream(request)
    }

    /// Streams provider-neutral model chunks through the kernel SPI.
    ///
    /// Product callers must consume this method through the runtime facade
    /// instead of importing individual provider SDKs or transport crates.
    pub fn stream_model_into(
        &self,
        request: ModelRequest,
        sink: &mut dyn ModelStreamSink,
    ) -> KernelResult<()> {
        self.model_provider().stream_into(request, sink)
    }

    pub fn resolve_interaction(
        &self,
        resolution: &AgentEngineInteractionResolution,
    ) -> crate::RuntimeFacadeResult<serde_json::Value> {
        let runtime_resolution = SdkRuntimeInteractionResolution {
            model_request_id: resolution.model_request_id.clone(),
            session_id: resolution.session_id.clone(),
            turn_id: resolution.turn_id.clone(),
            provider_session_id: resolution.provider_session_id.clone(),
            provider_turn_id: resolution.provider_turn_id.clone(),
            provider_request_id: resolution.provider_request_id.clone(),
            resolution: resolution.resolution.clone(),
        };
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => integration
                .resolve_interaction(&runtime_resolution)
                .map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string())),
            _ => Err(crate::RuntimeFacadeError::InvalidInput(format!(
                "agent engine {} does not support typed interaction resolution",
                self.engine_key()
            ))),
        }
    }

    pub fn get_provider_session_activity(
        &self,
        provider_session_id: &str,
    ) -> KernelResult<SessionActivitySnapshot> {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::ClaudeCode(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::Gemini(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::OpenCode(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::OpenClaw(_) | Self::Hermes(_) | Self::MiMoCode(_) | Self::Rig(_) => {
                Ok(SessionActivitySnapshot::unsupported(provider_session_id))
            }
        }
    }

    pub fn list_provider_sessions(&self) -> KernelResult<Vec<AgentSession>> {
        self.list_provider_sessions_for_directory(None)
    }

    pub fn list_provider_sessions_for_directory(
        &self,
        working_directory: Option<&str>,
    ) -> KernelResult<Vec<AgentSession>> {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => {
                collect_codex_provider_sessions(integration, working_directory)
            }
            Self::ClaudeCode(integration) => adapt_sdk_provider_sessions(
                "claude-code",
                integration.list_provider_sessions_for_directory(working_directory)?,
            ),
            Self::Gemini(integration) => integration.list_provider_sessions(),
            Self::OpenCode(integration) => adapt_sdk_provider_sessions(
                "opencode",
                integration.list_provider_sessions_for_directory(working_directory)?,
            ),
            Self::OpenClaw(integration) => integration
                .lifecycle
                .list_sessions(&SessionListQuery::default()),
            Self::Hermes(integration) => adapt_sdk_provider_sessions(
                "hermes",
                integration
                    .list_provider_sessions(working_directory.map(str::to_string))
                    .map_err(|error| {
                        sdkwork_agent_kernel::KernelError::provider_error(
                            "hermes_session_list_failed",
                            error.message,
                        )
                    })?,
            ),
            Self::MiMoCode(integration) => integration
                .lifecycle
                .list_sessions(&SessionListQuery::default()),
            Self::Rig(integration) => integration
                .lifecycle
                .list_sessions(&SessionListQuery::default()),
        }
    }

    pub fn get_provider_session_history(
        &self,
        provider_session_id: &str,
    ) -> KernelResult<Vec<AgentMessage>> {
        self.get_provider_session_history_for_directory(provider_session_id, None)
    }

    pub fn get_provider_session_history_for_directory(
        &self,
        provider_session_id: &str,
        working_directory: Option<&str>,
    ) -> KernelResult<Vec<AgentMessage>> {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => {
                collect_codex_provider_messages(integration, provider_session_id)
            }
            Self::ClaudeCode(integration) => adapt_sdk_provider_messages(
                "claude-code",
                integration.get_provider_session_history_for_directory(
                    provider_session_id,
                    working_directory,
                )?,
            ),
            Self::Gemini(integration) => {
                integration.get_provider_session_history(provider_session_id)
            }
            Self::OpenCode(integration) => adapt_sdk_provider_messages(
                "opencode",
                integration.get_provider_session_history_for_directory(
                    provider_session_id,
                    working_directory,
                )?,
            ),
            Self::OpenClaw(integration) => integration
                .lifecycle
                .get_conversation_history(provider_session_id),
            Self::Hermes(integration) => adapt_sdk_provider_messages(
                "hermes",
                integration
                    .get_provider_session_history(provider_session_id)
                    .map_err(|error| {
                        sdkwork_agent_kernel::KernelError::provider_error(
                            "hermes_session_history_failed",
                            error.message,
                        )
                    })?,
            ),
            Self::MiMoCode(integration) => integration
                .lifecycle
                .get_conversation_history(provider_session_id),
            Self::Rig(integration) => integration
                .lifecycle
                .get_conversation_history(provider_session_id),
        }
    }

    /// Lists the direct child provider sessions (sub-agents) of the given
    /// provider session. Engines without a sub-agent topology return an empty
    /// list. Used to synchronize the full sub-agent execution context.
    pub fn list_provider_session_children(
        &self,
        provider_session_id: &str,
        working_directory: Option<&str>,
    ) -> KernelResult<Vec<String>> {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => {
                collect_codex_provider_children(integration, provider_session_id, working_directory)
            }
            Self::ClaudeCode(_)
            | Self::Gemini(_)
            | Self::OpenCode(_)
            | Self::OpenClaw(_)
            | Self::Hermes(_)
            | Self::MiMoCode(_)
            | Self::Rig(_) => Ok(Vec::new()),
        }
    }

    /// Whether this engine can establish a new provider session from a verified
    /// runtime stream completion.
    pub(crate) fn supports_streaming_completion(&self) -> bool {
        #[cfg(feature = "codex-provider")]
        {
            matches!(
                self,
                Self::Codex(_)
                    | Self::ClaudeCode(_)
                    | Self::OpenCode(_)
                    | Self::MiMoCode(_)
                    | Self::Rig(_)
            )
        }
        #[cfg(not(feature = "codex-provider"))]
        {
            matches!(
                self,
                Self::ClaudeCode(_) | Self::OpenCode(_) | Self::MiMoCode(_) | Self::Rig(_)
            )
        }
    }

    /// Streams an initial turn through the runtime-backed completion boundary.
    ///
    /// This intentionally remains crate-private: callers consume the
    /// provider-neutral facade completion rather than transport metadata.
    pub(crate) fn stream_model_into_with_completion(
        &self,
        request: ModelRequest,
        sink: &mut dyn ModelStreamSink,
    ) -> KernelResult<SdkRuntimeStreamCompletion> {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => {
                integration.model.stream_into_with_completion(request, sink)
            }
            Self::ClaudeCode(integration) => {
                integration.model.stream_into_with_completion(request, sink)
            }
            Self::OpenCode(integration) => {
                integration.model.stream_into_with_completion(request, sink)
            }
            Self::MiMoCode(integration) => {
                integration.model.stream_into_with_completion(request, sink)
            }
            Self::Rig(integration) => {
                integration.model.stream_into_with_completion(request, sink)
            }
            _ => Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: "model.streaming.initial_session_completion".to_string(),
            }),
        }
    }

    pub(crate) fn model_provider(&self) -> &dyn ModelProvider {
        match self {
            #[cfg(feature = "codex-provider")]
            Self::Codex(integration) => &integration.model,
            Self::ClaudeCode(integration) => &integration.model,
            Self::Gemini(integration) => &integration.model,
            Self::OpenCode(integration) => &integration.model,
            Self::OpenClaw(integration) => &integration.model,
            Self::Hermes(integration) => &integration.model,
            Self::MiMoCode(integration) => &integration.model,
            Self::Rig(integration) => &integration.model,
        }
    }
}

fn adapt_sdk_provider_sessions(
    provider_id: &str,
    records: Vec<SdkRuntimeSessionRecord>,
) -> KernelResult<Vec<AgentSession>> {
    records
        .into_iter()
        .map(|record| adapt_sdk_provider_session(provider_id, record))
        .collect()
}

fn adapt_sdk_provider_session(
    provider_id: &str,
    record: SdkRuntimeSessionRecord,
) -> KernelResult<AgentSession> {
    let archived = record.archived_at.is_some();
    let mut session = AgentSession::new(record.provider_session_id);
    session.source = SessionSource::Cli;
    session.kind = if record.parent_provider_session_id.is_some() {
        SessionKind::Subagent
    } else {
        SessionKind::Main
    };
    session.parent_session_id = record.parent_provider_session_id;
    session.forked_from_id = sdk_metadata_string(&record.metadata, "codex.forked_from_id");
    session.agent_nickname = sdk_metadata_string(&record.metadata, "codex.agent_nickname");
    session.agent_role = sdk_metadata_string(&record.metadata, "codex.agent_role");
    session.title = record.title;
    session.summary = record.summary;
    session.preview = record.preview;
    session.created_at = record.created_at;
    session.updated_at = record.updated_at;
    session.archived_at = record.archived_at;
    session.model = record.model;
    session.model_provider = record.model_provider;
    if let Some(cwd) = record.cwd {
        session.cwd = Some(cwd.clone());
        session.workspace_roots.push(cwd);
    }
    session.message_count = record.message_count;
    session.tool_call_count = record.tool_call_count;
    session.token_usage.input_tokens = record.input_tokens;
    session.token_usage.output_tokens = record.output_tokens;
    session.token_usage.cached_tokens = record.cached_tokens;
    session.token_usage.reasoning_tokens = record.reasoning_tokens;
    session.cost_cents = record.cost_cents;
    session.change_summary.additions = record.additions;
    session.change_summary.deletions = record.deletions;
    session.change_summary.files_changed = record.files_changed;
    session.metadata.extend(sdk_metadata_pairs(record.metadata));
    if archived {
        session.state = SessionState::Archived;
        session.metadata.push((
            "sdkwork.provider.session.directory.archived".to_string(),
            "true".to_string(),
        ));
    }
    finalize_provider_session_snapshot(provider_id, session)
}

fn adapt_sdk_provider_messages(
    provider_id: &str,
    records: Vec<SdkRuntimeMessageRecord>,
) -> KernelResult<Vec<AgentMessage>> {
    records
        .into_iter()
        .map(|record| adapt_sdk_provider_message(provider_id, record))
        .collect()
}

fn adapt_sdk_provider_message(
    provider_id: &str,
    record: SdkRuntimeMessageRecord,
) -> KernelResult<AgentMessage> {
    let role = match record.role.as_str() {
        "user" => AgentMessageRole::User,
        "agent" => AgentMessageRole::Agent,
        "model" => AgentMessageRole::Model,
        "system" => AgentMessageRole::System,
        "tool" => AgentMessageRole::Tool,
        "policy" => AgentMessageRole::Policy,
        "adapter" => AgentMessageRole::Adapter,
        other => {
            return Err(KernelError::provider_error(
                "provider_sdk_message_role_invalid",
                format!("{provider_id} returned unsupported message role: {other}"),
            ))
        }
    };
    let parts = record
        .parts
        .into_iter()
        .map(|part| {
            part.into_agent_part()
                .map(|part| part.from_provider(provider_id))
        })
        .collect::<KernelResult<Vec<_>>>()?;
    let mut message = AgentMessage::new(record.provider_message_id, role, parts)
        .for_session(record.provider_session_id);
    if let Some(parent_provider_message_id) = record.parent_provider_message_id {
        message = message.with_parent_message(parent_provider_message_id);
    }
    if let Some(created_at) = record.created_at {
        message = message.created_at(created_at);
    }
    message.metadata.extend(sdk_metadata_pairs(record.metadata));
    message
        .metadata
        .push(("sdkwork.provider.id".to_string(), provider_id.to_string()));
    message.validate()?;
    Ok(message)
}

fn sdk_metadata_pairs(metadata: BTreeMap<String, serde_json::Value>) -> Vec<(String, String)> {
    metadata
        .into_iter()
        .map(|(key, value)| {
            let value = match value {
                serde_json::Value::String(value) => value,
                value => value.to_string(),
            };
            (key, value)
        })
        .collect()
}

/// Extracts a single string metadata value from a provider session record,
/// returning `None` when the key is absent or not a string.
fn sdk_metadata_string(
    metadata: &BTreeMap<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    metadata
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
}

#[cfg(feature = "codex-provider")]
fn collect_codex_provider_sessions(
    integration: &CodexSdkIntegration,
    working_directory: Option<&str>,
) -> KernelResult<Vec<AgentSession>> {
    let records = collect_provider_pages(
        "codex",
        "session inventory",
        |cursor| {
            let page = futures::executor::block_on(integration.list_provider_sessions(
                ThreadListParams {
                    cursor,
                    limit: Some(sdkwork_utils_rust::DEFAULT_LIST_PAGE_SIZE as u32),
                    sort_key: None,
                    sort_direction: None,
                    model_providers: None,
                    source_kinds: None,
                    archived: None,
                    section_id: None,
                    cwd: working_directory.map(|cwd| ThreadListCwdFilter::One(cwd.to_string())),
                    use_state_db_only: false,
                    search_term: None,
                    parent_thread_id: None,
                    ancestor_thread_id: None,
                },
            ))?;
            Ok((
                page.data.into_iter().map(|record| record.session).collect(),
                page.next_cursor,
            ))
        },
        |session| session.provider_session_id.as_str(),
    )?;
    adapt_sdk_provider_sessions("codex", records)
}

/// Enumerates the direct sub-agent threads spawned under the given Codex
/// thread via `thread/list` with `parentThreadId`. The sub-agent topology is
/// otherwise invisible to the top-level inventory which only lists interactive
/// root threads.
#[cfg(feature = "codex-provider")]
fn collect_codex_provider_children(
    integration: &CodexSdkIntegration,
    provider_session_id: &str,
    working_directory: Option<&str>,
) -> KernelResult<Vec<String>> {
    let records = collect_provider_pages(
        "codex",
        "sub-agent thread inventory",
        |cursor| {
            let page = futures::executor::block_on(integration.list_provider_sessions(
                ThreadListParams {
                    cursor,
                    limit: Some(sdkwork_utils_rust::DEFAULT_LIST_PAGE_SIZE as u32),
                    sort_key: None,
                    sort_direction: None,
                    model_providers: None,
                    source_kinds: None,
                    archived: None,
                    section_id: None,
                    cwd: working_directory.map(|cwd| ThreadListCwdFilter::One(cwd.to_string())),
                    use_state_db_only: false,
                    search_term: None,
                    parent_thread_id: Some(provider_session_id.to_string()),
                    ancestor_thread_id: None,
                },
            ))?;
            Ok((
                page.data.into_iter().map(|record| record.session).collect(),
                page.next_cursor,
            ))
        },
        |session| session.provider_session_id.as_str(),
    )?;
    Ok(records
        .into_iter()
        .map(|session| session.provider_session_id)
        .collect())
}

#[cfg(feature = "codex-provider")]
fn collect_codex_provider_messages(
    integration: &CodexSdkIntegration,
    provider_session_id: &str,
) -> KernelResult<Vec<AgentMessage>> {
    let records = collect_provider_pages(
        "codex",
        "session transcript",
        |cursor| {
            let page = futures::executor::block_on(integration.get_provider_session_history(
                ThreadTurnsListParams {
                    thread_id: provider_session_id.to_owned(),
                    cursor,
                    limit: Some(sdkwork_utils_rust::DEFAULT_LIST_PAGE_SIZE as u32),
                    sort_direction: Some(CodexSortDirection::Asc),
                    items_view: Some(TurnItemsView::Full),
                },
            ))?;
            Ok((
                page.data.into_iter().map(|record| record.message).collect(),
                page.next_cursor,
            ))
        },
        |message| message.provider_message_id.as_str(),
    )?;
    adapt_sdk_provider_messages("codex", records)
}

fn collect_provider_pages<T>(
    provider_id: &str,
    resource: &str,
    mut load_page: impl FnMut(Option<String>) -> KernelResult<(Vec<T>, Option<String>)>,
    item_id: impl Fn(&T) -> &str,
) -> KernelResult<Vec<T>> {
    let mut cursor = None;
    let mut seen_cursors = HashSet::new();
    let mut seen_item_ids = HashSet::new();
    let mut items = Vec::new();
    loop {
        let (page_items, next_cursor) = load_page(cursor.clone())?;
        for item in page_items {
            if seen_item_ids.insert(item_id(&item).to_string()) {
                items.push(item);
            }
        }
        ensure_provider_collection_size(provider_id, resource, items.len())?;
        let Some(next_cursor) = normalized_provider_cursor(next_cursor) else {
            return Ok(items);
        };
        ensure_new_provider_cursor(provider_id, resource, &mut seen_cursors, &next_cursor)?;
        cursor = Some(next_cursor);
    }
}

fn normalized_provider_cursor(cursor: Option<String>) -> Option<String> {
    cursor.and_then(|cursor| {
        let cursor = cursor.trim();
        (!cursor.is_empty()).then(|| cursor.to_string())
    })
}

fn ensure_new_provider_cursor(
    provider_id: &str,
    resource: &str,
    seen_cursors: &mut HashSet<String>,
    cursor: &str,
) -> KernelResult<()> {
    if seen_cursors.insert(cursor.to_string()) {
        return Ok(());
    }
    Err(sdkwork_agent_kernel::KernelError::provider_error(
        "provider_session_cursor_cycle",
        format!("{provider_id} repeated an opaque cursor while reading {resource}"),
    ))
}

fn ensure_provider_collection_size(
    provider_id: &str,
    resource: &str,
    size: usize,
) -> KernelResult<()> {
    if size <= MAX_PROVIDER_SESSION_COLLECTION_ITEMS {
        return Ok(());
    }
    Err(sdkwork_agent_kernel::KernelError::provider_error(
        "provider_session_collection_too_large",
        format!("{provider_id} {resource} exceeds {MAX_PROVIDER_SESSION_COLLECTION_ITEMS} items"),
    ))
}

pub fn bootstrap_agent_engine(engine_key: &str) -> Result<AgentEngineSlot, AgentEngineBootstrapError> {
    match engine_key {
        #[cfg(feature = "codex-provider")]
        "codex" => CodexSdkIntegration::bootstrap()
            .map(AgentEngineSlot::Codex)
            .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string())),
        #[cfg(not(feature = "codex-provider"))]
        "codex" => Err(AgentEngineBootstrapError::Bootstrap(
            "codex provider is not enabled in this build (enable the codex-provider feature to integrate it)".to_string(),
        )),
        "claude-code" => ClaudeCodeSdkIntegration::bootstrap()
            .map(AgentEngineSlot::ClaudeCode)
            .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string())),
        "gemini" => GeminiCliSdkIntegration::bootstrap()
            .map(AgentEngineSlot::Gemini)
            .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string())),
        "opencode" => OpenCodeSdkIntegration::bootstrap()
            .map(AgentEngineSlot::OpenCode)
            .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string())),
        "openclaw" => OpenClawSdkIntegration::bootstrap()
            .map(AgentEngineSlot::OpenClaw)
            .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string())),
        "hermes" => HermesSdkIntegration::bootstrap()
            .map(AgentEngineSlot::Hermes)
            .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string())),
        "mimo-code" => MiMoCodeSdkIntegration::bootstrap()
            .map(AgentEngineSlot::MiMoCode)
            .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string())),
        "rig" => bootstrap_rig_agent_engine(None, Arc::new(EnvFileSecretHostProvider::new())),
        other => Err(AgentEngineBootstrapError::UnsupportedEngine(
            other.to_string(),
        )),
    }
}

/// Bootstraps the Rig (simple agent) engine slot, upgrading its model provider
/// from the default fail-closed backend when a live backend is available.
///
/// Executor selection:
/// - no configuration → the default live cloud router backend: every model
///   call is routed through the `sdkwork-cloudrouter-sdk` account pool with
///   the caller's dual tokens (no local configuration or API key required);
/// - `llm.rig.provider_id=cloudrouter` (with an optional `llm.rig.api_key`) →
///   the cloud router backend in API-key mode for token-less flows;
/// - any other live configuration (provider_id `openai`, `deepseek`, custom
///   vendors, …) with an `llm.rig.api_key` secret → the rig-core
///   OpenAI-compatible adapter: a custom `llm.rig.base_url` endpoint and
///   `llm.rig.default_model` are honored for personalized provider setups.
///
/// `configuration` is the materialized rig profile configuration (from
/// [`apply_agent_engine_model_configuration`]); an invalid configuration keeps
/// the fail-closed behavior so a misconfigured engine never silently answers
/// as a stub.
pub fn bootstrap_rig_agent_engine(
    configuration: Option<&AgentConfiguration>,
    host: Arc<dyn HostProvider + Send + Sync>,
) -> Result<AgentEngineSlot, AgentEngineBootstrapError> {
    let integration = RigSdkIntegration::bootstrap()
        .map_err(|error| AgentEngineBootstrapError::Bootstrap(error.to_string()))?;
    let backend_config = match configuration {
        Some(configuration) => match RigBackendConfig::from_configuration(configuration) {
            Ok(config) => config,
            Err(error) => {
                tracing::debug!(
                    error = %error,
                    "rig backend configuration is not usable; keeping fail-closed backend"
                );
                return Ok(AgentEngineSlot::Rig(integration));
            }
        },
        // Product default: live cloud router account-pool routing with the
        // caller's dual tokens — no local secret is required.
        None => RigBackendConfig {
            mode: RigBackendMode::Live,
            provider_id: None,
            api_key_secret_ref: None,
            base_url: None,
        },
    };
    if !matches!(backend_config.mode, RigBackendMode::Live) {
        return Ok(AgentEngineSlot::Rig(integration));
    }

    let routes_through_cloud_router = backend_config
        .provider_id
        .as_deref()
        .is_some_and(|provider_id| provider_id == "cloudrouter")
        || backend_config.api_key_secret_ref.is_none();
    let model: Arc<dyn ModelProvider + Send + Sync> = if routes_through_cloud_router {
        // Cloud router account-pool executor: caller dual tokens by default,
        // gateway API-key mode when the configuration binds `llm.rig.api_key`.
        Arc::new(RigCloudRouterModelProvider::new(backend_config.clone(), host))
    } else {
        // Custom OpenAI-compatible provider (rig-core direct): vendor id,
        // api key secret, optional custom base url and default model.
        let default_model_id = configuration
            .and_then(|configuration| configuration.value("llm.rig.default_model"))
            .and_then(|value| match value {
                AgentConfigValue::String(value) if !value.trim().is_empty() => Some(value.clone()),
                _ => None,
            })
            .unwrap_or_else(|| ids::DEFAULT_MODEL_ID.to_string());
        match RigModelProvider::with_rig_core_openai(backend_config, host, default_model_id) {
            Ok(model) => Arc::new(model),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "rig live backend upgrade failed; keeping fail-closed backend"
                );
                return Ok(AgentEngineSlot::Rig(integration));
            }
        }
    };
    // Rebuild the in-process rust runtime around the live model provider so
    // runtime-routed model calls reach the live backend (cloudrouter account
    // pool or rig-core adapter) instead of the bootstrap fail-closed stub.
    let upgraded = integration.with_live_model_provider(model);
    tracing::info!("rig agent engine upgraded to live backend");
    Ok(AgentEngineSlot::Rig(upgraded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Mutex, OnceLock};

    /// Redirects every provider config write to a fresh temp home.
    ///
    /// All engine config SPIs resolve their config file through
    /// `provider_user_home()` (`USERPROFILE` on Windows, `HOME` elsewhere), so
    /// pointing `USERPROFILE` at a temp directory keeps config-SPI tests from
    /// ever touching real user configuration (and from racing each other on
    /// the same file).
    fn isolate_provider_user_home() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let temp_home = std::env::temp_dir().join(format!("sdkwork-test-home-{stamp}"));
        std::fs::create_dir_all(&temp_home).expect("create temp user home");
        std::env::set_var("USERPROFILE", &temp_home);
    }

    /// Serializes config-SPI tests: they redirect `USERPROFILE` (process-wide)
    /// to temp homes and write overlapping provider config files, so they must
    /// not run concurrently with each other (or with bootstraps that resolve
    /// the same paths).
    static CONFIG_SPI_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn config_spi_test_guard() -> std::sync::MutexGuard<'static, ()> {
        CONFIG_SPI_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    #[test]
    fn canonical_agent_engines_map_to_binding_ids() {
        for engine in canonical_agent_engine_keys() {
            assert!(agent_engine_agent_id(engine).is_some());
            assert!(agent_engine_binding_id(engine).is_some());
        }
    }

    #[test]
    fn provider_page_collection_drains_opaque_cursors_and_deduplicates_items() {
        let mut pages = VecDeque::from([
            (
                None,
                vec!["session-new".to_string(), "session-shared".to_string()],
                Some("provider-cursor-1".to_string()),
            ),
            (
                Some("provider-cursor-1".to_string()),
                vec!["session-shared".to_string(), "session-old".to_string()],
                None,
            ),
        ]);

        let sessions = collect_provider_pages(
            "provider-under-test",
            "session inventory",
            |cursor| {
                let (expected_cursor, items, next_cursor) =
                    pages.pop_front().expect("expected provider page");
                assert_eq!(cursor, expected_cursor);
                Ok((items, next_cursor))
            },
            String::as_str,
        )
        .expect("complete provider inventory");

        assert_eq!(sessions, ["session-new", "session-shared", "session-old"]);
        assert!(pages.is_empty());
    }

    #[test]
    fn provider_page_collection_rejects_repeated_opaque_cursor() {
        let mut request_count = 0;
        let error = collect_provider_pages(
            "provider-under-test",
            "session transcript",
            |_| {
                request_count += 1;
                Ok((
                    vec![format!("message-{request_count}")],
                    Some("repeated-provider-cursor".to_string()),
                ))
            },
            String::as_str,
        )
        .expect_err("cursor cycle must fail closed");

        assert!(error.to_string().contains("repeated an opaque cursor"));
        assert_eq!(request_count, 2);
    }

    #[test]
    fn sdk_provider_records_are_explicitly_adopted_at_the_runtime_boundary() {
        let session: SdkRuntimeSessionRecord = serde_json::from_value(serde_json::json!({
            "provider_session_id": "provider-session-1",
            "parent_provider_session_id": "provider-session-parent",
            "title": "Provider session",
            "cwd": "E:/workspace/project",
            "created_at": "2026-08-01T00:00:00Z",
            "updated_at": "2026-08-01T01:00:00Z",
            "archived_at": "2026-08-01T02:00:00Z",
            "input_tokens": 7,
            "output_tokens": 11,
            "metadata": {"provider.tag": "history"}
        }))
        .expect("SDK session record");
        let session = adapt_sdk_provider_session("provider-under-test", session)
            .expect("adopt provider session");

        assert_eq!(session.session_id, "provider-session-1");
        assert_eq!(session.kind, SessionKind::Subagent);
        assert_eq!(session.state, SessionState::Archived);
        assert_eq!(session.token_usage.total_tokens, 18);
        assert_eq!(
            session.metadata_value("sdkwork.provider.session.directory.archived"),
            Some("true")
        );

        let message: SdkRuntimeMessageRecord = serde_json::from_value(serde_json::json!({
            "provider_message_id": "provider-message-1",
            "provider_session_id": "provider-session-1",
            "role": "agent",
            "parts": [{
                "part_id": "provider-part-1",
                "kind": "text",
                "text": "done",
                "metadata": {"sdkwork.provider.content_type": "reasoning"}
            }],
            "created_at": "2026-08-01T01:00:00Z"
        }))
        .expect("SDK message record");
        let message = adapt_sdk_provider_message("provider-under-test", message)
            .expect("adopt provider message");

        assert_eq!(message.message_id, "provider-message-1");
        assert_eq!(message.session_id.as_deref(), Some("provider-session-1"));
        assert_eq!(
            message.parts[0].provenance.as_deref(),
            Some("provider-under-test")
        );
        assert_eq!(
            message.parts[0].metadata_value("sdkwork.provider.content_type"),
            Some("reasoning")
        );
    }

    #[test]
    fn all_canonical_agent_engines_bootstrap() {
        for engine in canonical_agent_engine_keys() {
            let slot = bootstrap_agent_engine(engine).unwrap_or_else(|error| {
                panic!("bootstrap failed for {engine}: {error}");
            });
            assert_eq!(slot.engine_key(), *engine);
            assert!(!slot.list_model_ids().is_empty());
        }
    }

    #[test]
    fn rig_engine_defaults_to_live_cloud_router_backend_without_configuration() {
        // The product default: no applied configuration still bootstraps a
        // live cloud router backend (caller dual-token account-pool routing).
        let host = std::sync::Arc::new(EnvFileSecretHostProvider::new());
        let slot = bootstrap_rig_agent_engine(None, host).expect("rig bootstrap");
        assert_eq!(slot.engine_key(), "rig");
        assert_eq!(slot.model_provider().health().status, "available");
        let descriptor = slot
            .list_model_descriptors()
            .into_iter()
            .next()
            .expect("rig model descriptor");
        assert_eq!(
            descriptor.metadata_value("sdkwork.backend.mode"),
            Some("live")
        );
        assert_eq!(
            descriptor.metadata_value("sdkwork.backend.fail_closed"),
            Some("false")
        );
    }

    #[test]
    fn rig_engine_upgrades_to_live_backend_with_applied_configuration() {
        let configuration = AgentConfiguration::new("agent.rig-general", "profile.rig.live")
            .set("llm.rig.provider_id", AgentConfigValue::string("openai"))
            .set(
                "llm.rig.api_key",
                AgentConfigValue::secret_ref("test.rig.api_key"),
            )
            .set(
                "llm.rig.default_model",
                AgentConfigValue::string("example-chat"),
            )
            .set("runtime.rig.backend_mode", AgentConfigValue::string("live"));
        let host = std::sync::Arc::new(EnvFileSecretHostProvider::new());
        let slot = bootstrap_rig_agent_engine(Some(&configuration), host).expect("rig bootstrap");
        assert_eq!(slot.engine_key(), "rig");
        assert_eq!(slot.model_provider().health().status, "available");
        let descriptor = slot
            .list_model_descriptors()
            .into_iter()
            .next()
            .expect("rig model descriptor");
        assert_eq!(
            descriptor.metadata_value("sdkwork.backend.fail_closed"),
            Some("false")
        );
    }

    #[test]
    fn rig_engine_keeps_fail_closed_when_configuration_is_not_live() {
        let configuration = AgentConfiguration::new("agent.rig-general", "profile.rig.failclosed")
            .set(
                "runtime.rig.backend_mode",
                AgentConfigValue::string("fail_closed"),
            );
        let host = std::sync::Arc::new(EnvFileSecretHostProvider::new());
        let slot = bootstrap_rig_agent_engine(Some(&configuration), host).expect("rig bootstrap");
        assert_eq!(slot.model_provider().health().status, "degraded");
    }

    #[test]
    fn rig_engine_uses_custom_openai_compatible_provider_with_base_url() {
        // A personalized provider setup: custom vendor + api key secret +
        // custom base url + default model routes through the rig-core direct
        // adapter, not the cloud router gateway.
        let configuration = AgentConfiguration::new("agent.rig-general", "profile.rig.custom")
            .set("llm.rig.provider_id", AgentConfigValue::string("deepseek"))
            .set(
                "llm.rig.api_key",
                AgentConfigValue::secret_ref("test.rig.deepseek"),
            )
            .set(
                "llm.rig.base_url",
                AgentConfigValue::string("https://api.deepseek.example.com/v1"),
            )
            .set(
                "llm.rig.default_model",
                AgentConfigValue::string("deepseek-chat"),
            )
            .set("runtime.rig.backend_mode", AgentConfigValue::string("live"));
        let host = std::sync::Arc::new(EnvFileSecretHostProvider::new());
        let slot = bootstrap_rig_agent_engine(Some(&configuration), host).expect("rig bootstrap");
        assert_eq!(slot.engine_key(), "rig");
        assert_eq!(slot.model_provider().health().status, "available");
        let descriptor = slot
            .list_model_descriptors()
            .into_iter()
            .next()
            .expect("rig model descriptor");
        assert_eq!(
            descriptor.metadata_value("sdkwork.backend.fail_closed"),
            Some("false")
        );
    }

    #[test]
    fn rig_engine_explicit_cloudrouter_provider_uses_gateway_executor() {
        // provider_id=cloudrouter with an api key routes through the cloud
        // router gateway in API-key mode (the direct adapter rejects it).
        let configuration = AgentConfiguration::new("agent.rig-general", "profile.rig.cr")
            .set(
                "llm.rig.provider_id",
                AgentConfigValue::string("cloudrouter"),
            )
            .set(
                "llm.rig.api_key",
                AgentConfigValue::secret_ref("test.rig.cloudrouter"),
            )
            .set("runtime.rig.backend_mode", AgentConfigValue::string("live"));
        let host = std::sync::Arc::new(EnvFileSecretHostProvider::new());
        let slot = bootstrap_rig_agent_engine(Some(&configuration), host).expect("rig bootstrap");
        assert_eq!(slot.engine_key(), "rig");
        assert_eq!(slot.model_provider().health().status, "available");
    }

    #[test]
    fn session_sdk_engines_enable_verified_first_turn_streaming() {
        #[cfg(feature = "codex-provider")]
        let codex = bootstrap_agent_engine("codex").expect("codex bootstrap");
        let claude = bootstrap_agent_engine("claude-code").expect("claude bootstrap");
        let opencode = bootstrap_agent_engine("opencode").expect("opencode bootstrap");
        let gemini = bootstrap_agent_engine("gemini").expect("gemini bootstrap");

        #[cfg(feature = "codex-provider")]
        assert!(codex.supports_streaming_completion());
        assert!(claude.supports_streaming_completion());
        assert!(opencode.supports_streaming_completion());
        assert!(!gemini.supports_streaming_completion());
    }

    #[test]
    fn runtime_identity_resolves_every_bootstrappable_agent_id() {
        for engine_key in bootstrappable_engine_keys() {
            let agent_id = agent_engine_agent_id(engine_key).expect("agent id");
            let identity = resolve_agent_engine_runtime_identity(agent_id)
                .expect("identity resolution")
                .expect("known identity");
            assert_eq!(identity.engine_key, engine_key);
            assert_eq!(identity.agent_id, agent_id);
            assert_eq!(
                identity.binding_id,
                agent_engine_binding_id(engine_key).unwrap()
            );
            assert!(!identity.provider_id.is_empty());
        }
        assert!(resolve_agent_engine_runtime_identity("agent.unknown")
            .expect("unknown identity resolution")
            .is_none());
    }

    #[test]
    fn model_configuration_dispatches_to_each_agent_engine_config_spi() {
        let _guard = config_spi_test_guard();
        isolate_provider_user_home();
        for engine_key in bootstrappable_engine_keys() {
            let agent_id = agent_engine_agent_id(engine_key).expect("agent id");
            let request = AgentModelConfigurationRequest::new(
                format!("request.model.{engine_key}"),
                agent_id,
                format!("profile.model.{engine_key}"),
                "openai-compatible",
                "https://models.example.test/v1",
                format!("secret-model-{engine_key}"),
                "example-chat",
            );
            let application = apply_agent_engine_model_configuration(engine_key, &request)
                .expect("provider Config SPI applies model configuration");
            assert_eq!(application.profile.agent_id, agent_id);
            assert_eq!(application.profile.profile_id, request.profile_id);
            assert_ne!(application.provider_scope, "process_adapter");
        }
    }

    #[test]
    fn model_selection_dispatches_to_each_agent_engine_config_spi() {
        let _guard = config_spi_test_guard();
        isolate_provider_user_home();
        for engine_key in bootstrappable_engine_keys() {
            let agent_id = agent_engine_agent_id(engine_key).expect("agent id");
            // Selection is fail-closed without a SDKWork-managed provider
            // entry in the config surface (OpenCode/OpenClaw reject silently
            // leaving the CLI on the previous model). Establish the
            // pre-condition exactly like a real apply flow: configure first.
            let setup = AgentModelConfigurationRequest::new(
                format!("request.selection.setup.{engine_key}"),
                agent_id.clone(),
                format!("profile.selection.{engine_key}"),
                "openai-compatible",
                "https://models.example.test/v1",
                format!("secret-selection-{engine_key}"),
                "example-chat",
            );
            apply_agent_engine_model_configuration(engine_key, &setup)
                .expect("provider Config SPI applies model configuration before selection");
            let request = AgentModelSelectionRequest::new(
                format!("request.selection.{engine_key}"),
                agent_id,
                format!("profile.selection.{engine_key}"),
                "catalog-model",
            );
            let application = apply_agent_engine_model_selection(engine_key, &request)
                .expect("provider Config SPI applies model selection");
            assert_eq!(application.profile.agent_id, agent_id);
            assert_eq!(application.profile.profile_id, request.profile_id);
            assert_ne!(application.provider_scope, "process_adapter");
        }
    }

    #[test]
    fn adapts_provider_session_identity_metadata_without_losing_tree_fields() {
        let record = SdkRuntimeSessionRecord {
            provider_session_id: "0198-thread".to_string(),
            parent_provider_session_id: Some("0196-thread".to_string()),
            title: Some("Provider review".to_string()),
            summary: None,
            preview: Some("Review the provider".to_string()),
            cwd: Some("E:/workspace/project".to_string()),
            created_at: Some("2026-07-26T00:00:00Z".to_string()),
            updated_at: None,
            archived_at: None,
            model: None,
            model_provider: Some("openai".to_string()),
            message_count: 2,
            tool_call_count: 1,
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            reasoning_tokens: 0,
            cost_cents: None,
            additions: 1,
            deletions: 0,
            files_changed: 1,
            metadata: BTreeMap::from([
                (
                    "codex.forked_from_id".to_string(),
                    serde_json::Value::String("0197-thread".to_string()),
                ),
                (
                    "codex.agent_nickname".to_string(),
                    serde_json::Value::String("reviewer".to_string()),
                ),
                (
                    "codex.agent_role".to_string(),
                    serde_json::Value::String("code-review".to_string()),
                ),
                (
                    "codex.session_id".to_string(),
                    serde_json::Value::String("0198-session".to_string()),
                ),
            ]),
        };

        let session =
            adapt_sdk_provider_session("codex", record).expect("provider session adaptation");

        assert_eq!(session.kind, SessionKind::Subagent);
        assert_eq!(session.parent_session_id.as_deref(), Some("0196-thread"));
        assert_eq!(session.forked_from_id.as_deref(), Some("0197-thread"));
        assert_eq!(session.agent_nickname.as_deref(), Some("reviewer"));
        assert_eq!(session.agent_role.as_deref(), Some("code-review"));
        assert!(session
            .metadata
            .iter()
            .any(|(key, value)| key == "codex.session_id" && value == "0198-session"));
    }
}
