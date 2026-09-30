//! API assembly for sdkwork-agents.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM

mod bootstrap;
mod contribution;
mod generated;
mod readiness;

pub use bootstrap::{assemble_api_router, ApiAssembly, assemble_api_router_with_pool, web_module, web_module_with_pool};
pub use contribution::{
    assemble_app_api_contribution,
    assemble_app_api_contribution_with_provider_session_cwd_resolver,
    assemble_app_runtime_contribution, app_api_route_manifest, ApiAssemblyContribution,
    AppRuntimeContribution,
};

/// Apply the Agents managed-store lifecycle from the canonical environment profile.
pub async fn bootstrap_application_database_from_env() -> anyhow::Result<()> {
    sdkwork_agents_database_host::bootstrap_agents_database_from_env()
        .await
        .map(|_| ())
        .map_err(anyhow::Error::msg)
}

/// Open the Agents kernel runtime persistence store from the canonical environment profile.
pub async fn bootstrap_kernel_database_from_env() -> anyhow::Result<()> {
    let config = sdkwork_agent_server::config::ServerConfig::from_env().map_err(|error| {
        anyhow::anyhow!("load kernel server config for sdkwork-agents: {error}")
    })?;
    sdkwork_agent_server::persistence::PersistenceState::open_from_config(&config)
        .map(|_| ())
        .map_err(|error| anyhow::anyhow!("kernel persistence migrate/bootstrap failed: {error}"))
}

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}

/// Forwards the process-level stop signal to Agents background workers
/// (turn reconciliation) on behalf of HTTP entrypoints that cannot depend on
/// the service crate directly.
pub fn signal_agents_background_shutdown() {
    sdkwork_intelligence_agents_service::signal_turn_reconciliation_shutdown();
}
