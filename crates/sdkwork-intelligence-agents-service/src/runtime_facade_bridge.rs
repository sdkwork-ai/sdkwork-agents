//! Bridges managed-agent runtime executions to `sdkwork-agents-runtime-facade`.
//!
//! Preview responses and prompt optimizations must not use deterministic local
//! contract stubs when a canonical agent-engine binding is active.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sdkwork_agent_kernel::{
    AgentConfiguration, FilesystemRequest, FilesystemResult, HostProvider, KernelError,
    KernelResult, NetworkRequest, NetworkResult, ProcessRequest, ProcessResult, ProviderHealth,
    ProviderManifest, ProviderSecretValue, SecretRef,
};
use sdkwork_agents_runtime_facade::{
    bootstrap_agent_engine, bootstrappable_engine_keys, agent_engine_binding_id,
    execute_agent_engine_turn, AgentsAgentEngineHost, AgentEngineTurnInput, LiveInteractionRegistry,
};
use sdkwork_utils_rust::string::is_blank;

use crate::domain::AgentProviderBindingRecord;

pub const RUNTIME_MODE_FACADE: &str = "agents-runtime-facade";
pub const RUNTIME_MODE_CONTRACT_FALLBACK: &str = "agents-contract-fallback";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewExecutionOutput {
    pub content: String,
    pub model_id: Option<String>,
    pub runtime_mode: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptOptimizationOutput {
    pub optimized_prompt: String,
    pub runtime_mode: &'static str,
}

pub fn engine_key_for_binding_id(binding_id: &str) -> Option<&'static str> {
    bootstrappable_engine_keys()
        .iter()
        .find(|&engine_key| agent_engine_binding_id(engine_key) == Some(binding_id))
        .map(|v| v as _)
}

/// Static provider-id -> engine-key map, computed once per process.
///
/// Provider ids are engine-owned constants; bootstrapping every engine on
/// every call (session activity enrichment calls this per row per page) is
/// wasteful. Engines whose bootstrap fails in this build are simply absent
/// from the map.
static PROVIDER_ID_ENGINE_KEYS: std::sync::LazyLock<
    std::collections::HashMap<String, &'static str>,
> = std::sync::LazyLock::new(|| {
    let mut map = std::collections::HashMap::new();
    for engine_key in bootstrappable_engine_keys() {
        if let Ok(slot) = bootstrap_agent_engine(engine_key) {
            for descriptor in slot.list_model_descriptors() {
                map.entry(descriptor.provider_id.clone())
                    .or_insert(engine_key);
            }
        }
    }
    map
});

pub fn engine_key_for_provider_identity(
    binding_id: Option<&str>,
    provider_id: Option<&str>,
) -> Option<&'static str> {
    binding_id.and_then(engine_key_for_binding_id).or_else(|| {
        let provider_id = provider_id?;
        PROVIDER_ID_ENGINE_KEYS.get(provider_id).copied()
    })
}

fn resolve_engine_and_model(
    active_binding: Option<&AgentProviderBindingRecord>,
    requested_model: Option<&str>,
) -> Option<(String, String)> {
    let binding = active_binding?;
    let engine_key = engine_key_for_binding_id(binding.binding_id.as_str())?.to_string();
    let model_id = if is_blank(requested_model) {
        bootstrap_agent_engine(engine_key.as_str())
            .ok()?
            .list_model_ids()
            .into_iter()
            .next()?
    } else {
        requested_model.unwrap_or("").to_string()
    };
    Some((engine_key, model_id))
}

pub fn execute_preview_response(
    active_binding: Option<&AgentProviderBindingRecord>,
    content: &str,
    requested_model: Option<&str>,
) -> PreviewExecutionOutput {
    if let Some((engine_key, model_id)) = resolve_engine_and_model(active_binding, requested_model)
    {
        if let Ok(slot) = bootstrap_agent_engine(engine_key.as_str()) {
            if let Ok(output) = execute_agent_engine_turn(
                &slot,
                &AgentEngineTurnInput {
                    engine_key: engine_key.clone(),
                    model_id: model_id.clone(),
                    prompt: content.to_string(),
                    ..Default::default()
                },
            ) {
                return PreviewExecutionOutput {
                    content: output.assistant_content,
                    model_id: Some(model_id),
                    runtime_mode: RUNTIME_MODE_FACADE,
                };
            }
        }
    }

    PreviewExecutionOutput {
        content: content.to_string(),
        model_id: requested_model.map(str::to_string),
        runtime_mode: RUNTIME_MODE_CONTRACT_FALLBACK,
    }
}

pub fn execute_prompt_optimization(
    active_binding: Option<&AgentProviderBindingRecord>,
    prompt: &str,
) -> PromptOptimizationOutput {
    let optimization_prompt = format!(
        "Optimize the following agent prompt for clarity and effectiveness. \
Return only the optimized prompt text with no preamble.\n\n{prompt}"
    );

    if let Some((engine_key, model_id)) = resolve_engine_and_model(active_binding, None) {
        if let Ok(slot) = bootstrap_agent_engine(engine_key.as_str()) {
            if let Ok(output) = execute_agent_engine_turn(
                &slot,
                &AgentEngineTurnInput {
                    engine_key: engine_key.clone(),
                    model_id,
                    prompt: optimization_prompt,
                    ..Default::default()
                },
            ) {
                let optimized = output.assistant_content.trim().to_string();
                if !optimized.is_empty() {
                    return PromptOptimizationOutput {
                        optimized_prompt: optimized,
                        runtime_mode: RUNTIME_MODE_FACADE,
                    };
                }
            }
        }
    }

    PromptOptimizationOutput {
        optimized_prompt: normalize_prompt_text(prompt),
        runtime_mode: RUNTIME_MODE_CONTRACT_FALLBACK,
    }
}

static AGENT_ENGINE_HOSTS: std::sync::LazyLock<Mutex<HashMap<String, Arc<AgentsAgentEngineHost>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Host key for the shared default engine host: the product-default
/// cloudrouter dual-token rig backend used by agents without a per-agent
/// model configuration.
const DEFAULT_AGENT_ENGINE_HOST_KEY: &str = "default";

fn agent_engine_host_key(tenant_id: u64, agent_id: &str) -> String {
    format!("{tenant_id}:{agent_id}")
}

fn build_default_agent_engine_host() -> Option<Arc<AgentsAgentEngineHost>> {
    let host = AgentsAgentEngineHost::bootstrap_selected(
        &bootstrappable_engine_keys(),
        LiveInteractionRegistry::new(),
    );
    if host.engine_keys().next().is_some() {
        Some(Arc::new(host))
    } else {
        None
    }
}

pub fn shared_agent_engine_host() -> Option<Arc<AgentsAgentEngineHost>> {
    // A failed bootstrap must not be cached: engine availability can recover
    // (e.g. the provider directory becomes readable again) and a permanently
    // cached None would force a process restart to ever synchronize again.
    let mut guard = AGENT_ENGINE_HOSTS
        .lock()
        .expect("provider engine host mutex poisoned");
    if let Some(host) = guard.get(DEFAULT_AGENT_ENGINE_HOST_KEY) {
        return Some(host.clone());
    }
    drop(guard);
    let host = build_default_agent_engine_host()?;
    let mut guard = AGENT_ENGINE_HOSTS
        .lock()
        .expect("provider engine host mutex poisoned");
    guard
        .entry(DEFAULT_AGENT_ENGINE_HOST_KEY.to_string())
        .or_insert_with(|| host.clone());
    Some(host)
}

/// Engine host for one `(tenant_id, agent_id)` scope.
///
/// A per-agent host installed by [`refresh_rig_agent_engine_for`] is preferred
/// so a custom LLM provider configuration only affects its own agent scope;
/// otherwise the shared default host (cloudrouter dual-token rig backend) is
/// lazily cached under this scope so every scope keeps a stable `Arc` across
/// turns.
pub fn agent_engine_host_for(
    tenant_id: u64,
    agent_id: &str,
) -> Option<Arc<AgentsAgentEngineHost>> {
    let key = agent_engine_host_key(tenant_id, agent_id);
    let mut guard = AGENT_ENGINE_HOSTS
        .lock()
        .expect("provider engine host mutex poisoned");
    if let Some(host) = guard.get(&key) {
        return Some(host.clone());
    }
    drop(guard);
    let shared = shared_agent_engine_host()?;
    let mut guard = AGENT_ENGINE_HOSTS
        .lock()
        .expect("provider engine host mutex poisoned");
    guard.entry(key).or_insert_with(|| shared.clone());
    Some(shared)
}

/// Rebuilds the engine host for one `(tenant_id, agent_id)` scope after a rig
/// (simple agent) model configuration was applied, so the custom provider
/// backend takes effect without a process restart and without affecting other
/// agents' scopes.
///
/// The host is rebuilt with [`AgentsAgentEngineHost::bootstrap_selected_with_rig`]
/// (per-engine failures are tolerated like the lazy bootstrap path). This is a
/// low-frequency management operation, so a fresh live-interaction registry is
/// acceptable; in-flight interactions on the previous host keep their Arc.
pub fn refresh_rig_agent_engine_for(
    tenant_id: u64,
    agent_id: &str,
    configuration: &AgentConfiguration,
    host: Arc<dyn HostProvider + Send + Sync>,
) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<()> {
    let rebuilt = AgentsAgentEngineHost::bootstrap_selected_with_rig(
        &bootstrappable_engine_keys(),
        Some(configuration),
        host,
        LiveInteractionRegistry::new(),
    );
    if rebuilt.engine_keys().next().is_none() {
        return Ok(());
    }
    let key = agent_engine_host_key(tenant_id, agent_id);
    // Swap the scoped host from a dedicated thread so the retired host (whose
    // rig slot may own a tokio runtime) is dropped outside async contexts —
    // dropping a runtime from within an asynchronous context panics.
    std::thread::spawn(move || {
        let mut guard = AGENT_ENGINE_HOSTS
            .lock()
            .expect("provider engine host mutex poisoned");
        guard.insert(key, Arc::new(rebuilt));
    })
    .join()
    .map_err(|_| {
        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
            "rig agent engine host refresh worker panicked".to_string(),
        )
    })?;
    Ok(())
}

/// Backward-compatible refresh replacing the shared default engine host
/// (used by scopes without a per-agent configuration).
pub fn refresh_rig_agent_engine(
    configuration: &AgentConfiguration,
    host: Arc<dyn HostProvider + Send + Sync>,
) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<()> {
    refresh_rig_agent_engine_for(0, DEFAULT_AGENT_ENGINE_HOST_KEY, configuration, host)
}

/// Kernel host surface backed by the model configuration runtime secret store.
///
/// Lets agent engines resolve their configured credential at inference time
/// through `HostProvider::resolve_secret` (kernel contract: providers resolve
/// plaintext secrets through the host secret surface; raw keys are never
/// stored inside kernel profiles).
#[derive(Clone)]
pub struct ModelConfigurationRuntimeHostProvider {
    runtime: Arc<crate::http::AgentModelConfigurationRuntime>,
}

impl ModelConfigurationRuntimeHostProvider {
    pub fn new(runtime: Arc<crate::http::AgentModelConfigurationRuntime>) -> Self {
        Self { runtime }
    }
}

impl HostProvider for ModelConfigurationRuntimeHostProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            "provider.host.agents-model-configuration",
            "host",
            "Agents Model Configuration Secret Host",
            "1.0.0",
            vec!["host.secrets".to_string()],
        )
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }

    fn filesystem(&self, _request: FilesystemRequest) -> KernelResult<FilesystemResult> {
        Err(KernelError::CapabilityMissing {
            capability_id: "host.filesystem".to_string(),
        })
    }

    fn process(&self, _request: ProcessRequest) -> KernelResult<ProcessResult> {
        Err(KernelError::CapabilityMissing {
            capability_id: "host.process".to_string(),
        })
    }

    fn network(&self, _request: NetworkRequest) -> KernelResult<NetworkResult> {
        Err(KernelError::CapabilityMissing {
            capability_id: "host.network".to_string(),
        })
    }

    fn resolve_secret(&self, secret_ref: SecretRef) -> KernelResult<ProviderSecretValue> {
        self.runtime
            .resolve_secret_value(&secret_ref.secret_ref_id, "agent-engine.rig")
    }
}

fn normalize_prompt_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AgentImplementationKind, AgentProviderBindingRecord};

    fn sample_binding(binding_id: &str) -> AgentProviderBindingRecord {
        AgentProviderBindingRecord {
            id: 1,
            tenant_id: 100001,
            agent_id: "agent.test".to_string(),
            binding_id: binding_id.to_string(),
            provider_id: "provider.test".to_string(),
            implementation_kind: AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id: "profile.test".to_string(),
            capabilities: vec!["model.chat".to_string()],
            active: true,
            version: 1,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn maps_canonical_binding_ids_to_engine_keys() {
        assert_eq!(engine_key_for_binding_id("binding.codex"), Some("codex"));
        assert_eq!(
            engine_key_for_binding_id("binding.opencode"),
            Some("opencode")
        );
        assert!(engine_key_for_binding_id("binding.unknown").is_none());
    }

    #[test]
    fn resolves_engine_from_provider_identity_when_binding_id_is_custom() {
        assert_eq!(
            engine_key_for_provider_identity(Some("binding.custom"), Some("provider.codex")),
            Some("codex")
        );
    }

    #[test]
    fn preview_resolves_codex_binding_through_runtime_facade_catalog() {
        let binding = sample_binding("binding.codex");
        let (engine_key, model_id) = resolve_engine_and_model(Some(&binding), None)
            .expect("Codex binding must resolve through the runtime facade catalog");

        assert_eq!(engine_key, "codex");
        assert!(!model_id.trim().is_empty());
    }

    #[test]
    fn preview_falls_back_without_agent_engine_binding() {
        let output = execute_preview_response(
            Some(&sample_binding("binding.custom")),
            "hello fallback",
            None,
        );
        assert_eq!(output.runtime_mode, RUNTIME_MODE_CONTRACT_FALLBACK);
        assert_eq!(output.content, "hello fallback");
    }

    #[test]
    fn host_provider_resolves_applied_secret_from_model_configuration_runtime() {
        use sdkwork_agent_kernel::{
            InMemorySecretProvider, SecretCreateRequest, SecretProvider, SecretType,
        };

        let mut secrets = InMemorySecretProvider::new();
        let metadata = secrets
            .create_secret(SecretCreateRequest::new(
                "test.rig.api_key",
                SecretType::ApiKey,
                "sk-test-value",
            ))
            .expect("create secret");
        let runtime = Arc::new(crate::http::AgentModelConfigurationRuntime::with_providers(
            Box::new(secrets),
            Box::new(
                crate::postgres_model_configuration_store::ScopedInMemoryAgentConfigurationStore::new(),
            ),
        ));
        let provider = ModelConfigurationRuntimeHostProvider::new(runtime);
        let value = provider
            .resolve_secret(SecretRef::new(metadata.secret_id.clone(), "Rig API key"))
            .expect("resolve configured secret");
        assert_eq!(value.expose_value(), "sk-test-value");
        assert!(
            provider
                .resolve_secret(SecretRef::new("missing.rig.api_key", "Missing key"))
                .is_err(),
            "unconfigured secrets must fail closed"
        );
    }

    #[test]
    fn refresh_rig_agent_engine_rebuilds_shared_host_with_live_backend() {
        use sdkwork_agent_kernel::{AgentConfigValue, AgentConfiguration, EnvFileSecretHostProvider};

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
        let host: Arc<dyn HostProvider + Send + Sync> =
            Arc::new(EnvFileSecretHostProvider::new());
        refresh_rig_agent_engine(&configuration, host).expect("rig refresh");

        let shared = shared_agent_engine_host().expect("shared agent engine host");
        let rig = shared.slot("rig").expect("rig slot");
        let descriptor = rig
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
    fn per_agent_engine_hosts_are_isolated_by_tenant_and_agent_scope() {
        use sdkwork_agent_kernel::{AgentConfigValue, AgentConfiguration, EnvFileSecretHostProvider};

        // Agent A (tenant 1) gets a custom provider configuration; agent B
        // (tenant 1, different agent) and tenant 2 keep the shared default.
        let configuration = AgentConfiguration::new("agent.chat.default", "profile.rig.custom")
            .set("llm.rig.provider_id", AgentConfigValue::string("openai"))
            .set(
                "llm.rig.api_key",
                AgentConfigValue::secret_ref("test.rig.api_key"),
            )
            .set(
                "llm.rig.default_model",
                AgentConfigValue::string("custom-chat"),
            )
            .set("runtime.rig.backend_mode", AgentConfigValue::string("live"));
        let host: Arc<dyn HostProvider + Send + Sync> =
            Arc::new(EnvFileSecretHostProvider::new());
        refresh_rig_agent_engine_for(1, "agent.chat.default", &configuration, host)
            .expect("per-agent rig refresh");

        // The configured scope resolves to its own host.
        let configured = agent_engine_host_for(1, "agent.chat.default")
            .expect("configured agent host");
        // A different agent in the same tenant falls back to the shared host
        // (lazily cached under its own scope key).
        let other_agent = agent_engine_host_for(1, "agent.other")
            .expect("other agent host");
        let other_tenant = agent_engine_host_for(2, "agent.chat.default")
            .expect("other tenant host");
        let shared = shared_agent_engine_host().expect("shared agent engine host");

        // Per-agent scopes are distinct instances; the fallback scopes share
        // the default host instance.
        assert!(!Arc::ptr_eq(&configured, &shared));
        assert!(!Arc::ptr_eq(&configured, &other_agent));
        assert!(!Arc::ptr_eq(&configured, &other_tenant));
        assert!(Arc::ptr_eq(&other_agent, &shared));
        assert!(Arc::ptr_eq(&other_tenant, &shared));
    }

    #[test]
    fn prompt_optimization_avoids_deterministic_local_contract_mode() {
        let output = execute_prompt_optimization(
            Some(&sample_binding("binding.codex")),
            "  make   this   better  ",
        );
        assert_ne!(output.runtime_mode, "deterministic-local-contract");
    }
}
