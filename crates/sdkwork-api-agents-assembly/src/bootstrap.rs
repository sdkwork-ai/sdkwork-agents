//! Gateway bootstrap for sdkwork-agents.
//! Kernel-owned operational routes compose with managed-store business routes via kernel-bridge,
//! and dependency-owned app-api surfaces (IAM, Drive) mount same-origin.

mod assets;
mod drive;
mod iam;

use anyhow::Context;
use sdkwork_agent_server::config::ServerConfig;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_web_bootstrap::{ApiAssemblyContribution, CompositeReadinessCheck, DatabasePoolReadinessCheck, ReadinessCheck, WebModule};
use std::sync::Arc;

use crate::readiness::AgentHttpReadinessCheck;

/// Indivisible host-neutral API assembly contribution (web-bootstrap contract).
pub type ApiAssembly = ApiAssemblyContribution;

pub async fn assemble_api_router() -> anyhow::Result<ApiAssembly> {
    let config = Arc::new(ServerConfig::from_env().map_err(|error| {
        anyhow::anyhow!("load kernel server config for sdkwork-agents: {error}")
    })?);
    let iam_router = iam::wire_iam_app_router()
        .await
        .map_err(anyhow::Error::msg)
        .context("compose embedded IAM app router")?;
    let assets_router = assets::wire_assets_app_router()
        .await
        .map_err(anyhow::Error::msg)
        .context("compose embedded Assets app router")?;
    let drive_router = drive::wire_drive_app_router()
        .await
        .map_err(anyhow::Error::msg)
        .context("compose embedded Drive app router")?;
    let agents_router = sdkwork_agents_kernel_bridge::build_agents_served_router(config.clone())
        .await
        .context("compose agents served router")?;
    let router = agents_router
        .merge(iam_router)
        .merge(assets_router)
        .merge(drive_router);
    ApiAssemblyContribution::from_manifest(
        "sdkwork-agents",
        "SDKWork Agents API",
        router,
        sdkwork_routes_agents_http_shared::combined_route_manifest(),
        vec![sdkwork_routes_agents_http_shared::agent_request_context_injector()],
        Arc::new(sdkwork_web_bootstrap::AlwaysReady),
    )
    .map_err(anyhow::Error::msg)
}

/// Assemble the Agents contribution against a caller-provided database pool so
/// the platform cloud gateway can share its process-wide PostgreSQL pool.
///
/// Only agents-owned combined routes are mounted; the cloud gateway hosts the
/// dependency-owned IAM and Drive surfaces as separate contributions.
pub async fn assemble_api_router_with_pool(pool: DatabasePool) -> Result<ApiAssembly, String> {
    let state = tokio::task::spawn_blocking(sdkwork_agents_kernel_bridge::build_agent_http_state)
        .await
        .map_err(|error| format!("agents state bootstrap worker failed: {error}"))?
        .map_err(|error| format!("{error:#}"))?;
    drop(state.spawn_turn_reconciliation_worker());
    let readiness: Arc<dyn ReadinessCheck> = Arc::new(CompositeReadinessCheck::new(vec![
        Arc::new(AgentHttpReadinessCheck::new(state.clone())),
        Arc::new(DatabasePoolReadinessCheck::new(pool)),
    ]));
    let router = sdkwork_intelligence_agents_service::build_combined_routes().with_state(state);
    ApiAssemblyContribution::from_manifest(
        "sdkwork-agents",
        "SDKWork Agents API",
        router,
        sdkwork_routes_agents_http_shared::combined_route_manifest(),
        vec![sdkwork_routes_agents_http_shared::agent_request_context_injector()],
        readiness,
    )
}

/// Canonical Web Module definition for this application
/// (API_ASSEMBLY_SPEC §4.1.1): the complete HTTP surface — every route,
/// manifest, and OpenAPI document of this owner — as one installable module.
pub async fn web_module() -> Result<WebModule, String> {
    Ok(WebModule::from_contribution(assemble_api_router().await.map_err(|error| error.to_string())?))
}

/// Same as [`web_module`] but composed on a process-shared database pool
/// (platform gateways, API_ASSEMBLY_SPEC §4.1.1).
pub async fn web_module_with_pool(pool: DatabasePool) -> Result<WebModule, String> {
    Ok(WebModule::from_contribution(assemble_api_router_with_pool(pool).await?))
}
