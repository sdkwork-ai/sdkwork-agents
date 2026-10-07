//! SDKWork Agents kernel composition boundary.

mod agent_http_state;
mod boundaries;
mod chat_default_toolkit;

pub use agent_http_state::build_agent_http_state;
pub use boundaries::{AGENTS_OWNED_CAPABILITIES, KERNEL_OWNED_CAPABILITIES};
pub use chat_default_toolkit::{curated_chat_tools, CHAT_DEFAULT_TOOL_IDS};

use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::Router;
use sdkwork_agent_server::{
    api::internal_runtime, app, config::ServerConfig, health, persistence::PersistenceState,
    preflight,
};
use sdkwork_routes_agents_http_shared::build_served_combined_router;

/// Build the served HTTP router for SDKWork Agents by composing kernel surfaces.
pub async fn build_agents_served_router(config: Arc<ServerConfig>) -> Result<Router> {
    let preflight_result = preflight::validate(config.as_ref());
    if !preflight_result.passed {
        bail!("kernel preflight checks failed for sdkwork-agents bootstrap");
    }

    let health_state = Arc::new(health::HealthState::new());
    let persistence = Arc::new(
        PersistenceState::open_from_config_async(config.as_ref())
            .await
            .context("open kernel persistence for agents application")?,
    );
    let runtime_state = Arc::new(
        internal_runtime::InternalRuntimeApiState::new(persistence.clone(), config.clone())
            .map_err(|error| anyhow::anyhow!("agent runtime bootstrap failed: {error}"))?,
    );

    let operational_router =
        app::build_app(config.clone(), health_state, persistence, runtime_state);

    let agent_http_state =
        build_agent_http_state().context("agents managed store HTTP state bootstrap")?;
    let _reconciliation_worker = agent_http_state.spawn_turn_reconciliation_worker();
    let business_router = build_served_combined_router(agent_http_state).await;

    Ok(operational_router.merge(business_router))
}
