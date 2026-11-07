use std::sync::Arc;

use sdkwork_intelligence_agents_service::AgentHttpState;
use sdkwork_routes_agents_http_shared::{
    agent_request_context_injector, app_route_manifest, backend_route_manifest,
};
pub use sdkwork_web_bootstrap::ApiAssemblyContribution;

use crate::readiness::AgentHttpReadinessCheck;

/// App-host runtime ports backed by the same Agents repository state.
pub struct AppRuntimeContribution {
    pub api: ApiAssemblyContribution,
    pub session_facade: Arc<dyn sdkwork_agents_runtime_facade::AgentsSessionFacade>,
}

/// Builds the unwrapped Agents App API for a gateway that owns the single Web Framework layer.
pub async fn assemble_app_api_contribution() -> anyhow::Result<ApiAssemblyContribution> {
    let state = build_agent_http_state().await?;
    contribution_from_state(state)
}

pub async fn assemble_app_api_contribution_with_provider_session_cwd_resolver(
    resolver: Arc<dyn sdkwork_agents_runtime_facade::ProviderSessionProjectCwdResolver>,
) -> anyhow::Result<ApiAssemblyContribution> {
    let state = build_agent_http_state()
        .await?
        .with_provider_session_cwd_resolver(resolver);
    contribution_from_state(state)
}

/// Builds the App API contribution and approved in-process facade from one state.
pub async fn assemble_app_runtime_contribution() -> anyhow::Result<AppRuntimeContribution> {
    let state = build_agent_http_state().await?;
    let session_facade = state.session_facade();
    let api = contribution_from_state(state)?;
    Ok(AppRuntimeContribution {
        api,
        session_facade,
    })
}

/// Builds the unwrapped Agents Backend API for a gateway that owns the single
/// Web Framework layer.
///
/// The backend composition boots its own `AgentHttpState` (repository, audit
/// sink, policy provider) so it stays independent of the App API composition
/// order. Background workers keep App-API ownership: every host that serves
/// Agents composes the App API, so a backend-only merge never double-spawns
/// the turn reconciliation worker.
pub async fn assemble_backend_api_contribution() -> anyhow::Result<ApiAssemblyContribution> {
    let state = build_agent_http_state().await?;
    backend_contribution_from_state(state)
}

/// App-api surface route manifest owned by the Agents dependency assembly.
pub fn app_api_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    sdkwork_routes_agents_http_shared::app_route_manifest()
}

/// Backend-api surface route manifest owned by the Agents dependency assembly.
pub fn backend_api_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    backend_route_manifest()
}

async fn build_agent_http_state() -> anyhow::Result<AgentHttpState> {
    tokio::task::spawn_blocking(sdkwork_agents_kernel_bridge::build_agent_http_state)
        .await
        .map_err(|error| anyhow::anyhow!("agents state bootstrap worker failed: {error}"))?
}

fn contribution_from_state(state: AgentHttpState) -> anyhow::Result<ApiAssemblyContribution> {
    let route_manifest = app_route_manifest();
    let readiness_check = Arc::new(AgentHttpReadinessCheck::new(state.clone()));
    start_owner_background_tasks(&state);
    let router = sdkwork_routes_agents_app_api::build_router().with_state(state);

    ApiAssemblyContribution::from_manifest(
        "sdkwork-agents",
        "SDKWork Agents App API",
        router,
        route_manifest,
        vec![agent_request_context_injector()],
        readiness_check,
    )
    .map_err(anyhow::Error::msg)
}

fn start_owner_background_tasks(state: &AgentHttpState) {
    // Dropping a Tokio JoinHandle detaches the process-lifetime task. Runtime shutdown still
    // terminates it, while the composing gateway remains independent of Agents worker internals.
    drop(state.spawn_turn_reconciliation_worker());
}

fn backend_contribution_from_state(
    state: AgentHttpState,
) -> anyhow::Result<ApiAssemblyContribution> {
    let route_manifest = backend_api_route_manifest();
    let readiness_check = Arc::new(AgentHttpReadinessCheck::new(state.clone()));
    let router = sdkwork_routes_agents_backend_api::build_router().with_state(state);

    ApiAssemblyContribution::from_manifest(
        "sdkwork-agents",
        "SDKWork Agents Backend API",
        router,
        route_manifest,
        vec![agent_request_context_injector()],
        readiness_check,
    )
    .map_err(anyhow::Error::msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_web_contract::{route_inventory_from_openapi, route_inventory_from_routes};

    #[test]
    fn app_api_manifest_openapi_and_auth_inventories_match() {
        let manifest = app_route_manifest();
        let openapi = sdkwork_web_contract::build_openapi_document(
            "SDKWork Agents App API",
            manifest.routes(),
        );

        assert_eq!(
            route_inventory_from_routes(manifest.routes()),
            route_inventory_from_openapi(&openapi).expect("valid Agents App API OpenAPI inventory")
        );
    }

    #[test]
    fn app_api_permission_catalog_is_the_manifest_permission_union() {
        let manifest = app_route_manifest();
        let catalog = sdkwork_web_bootstrap::permission_catalog(manifest.routes());
        let mut expected = manifest
            .routes()
            .iter()
            .flat_map(|route| {
                route
                    .required_permission
                    .into_iter()
                    .chain(route.alternate_permissions.into_iter().flatten().copied())
            })
            .collect::<Vec<_>>();
        expected.sort_unstable();
        expected.dedup();

        assert_eq!(expected, catalog);
    }

    #[test]
    fn backend_api_manifest_and_openapi_inventories_match() {
        let manifest = backend_api_route_manifest();
        let openapi = sdkwork_web_contract::build_openapi_document(
            "SDKWork Agents Backend API",
            manifest.routes(),
        );

        assert_eq!(
            route_inventory_from_routes(manifest.routes()),
            route_inventory_from_openapi(&openapi)
                .expect("valid Agents Backend API OpenAPI inventory")
        );
    }

    #[test]
    fn backend_api_manifest_exposes_the_agent_category_catalog() {
        let manifest = backend_api_route_manifest();
        for (method, path, permission) in [
            (
                "GET",
                "/backend/v3/api/ai/agent_categories",
                "agent.business.category.read",
            ),
            (
                "PUT",
                "/backend/v3/api/ai/agent_categories/{categoryCode}",
                "agent.business.category.manage",
            ),
        ] {
            let route = manifest
                .match_route(method, path)
                .unwrap_or_else(|| panic!("{method} {path} must be served by the backend API"));
            assert_eq!(
                route.auth,
                sdkwork_web_contract::RouteAuth::DualToken,
                "{method} {path} must require the dual-token auth boundary"
            );
            assert_eq!(
                route.required_permission,
                Some(permission),
                "{method} {path} must require its category permission"
            );
        }
    }
}
