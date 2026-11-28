//! Shared HTTP route manifests and sdkwork-web-framework bootstrap for agents managed store surfaces.

mod generated {
    include!(concat!(env!("OUT_DIR"), "/agent_app_routes.rs"));
    include!(concat!(env!("OUT_DIR"), "/agent_backend_routes.rs"));
    include!(concat!(env!("OUT_DIR"), "/agent_open_routes.rs"));
    include!(concat!(env!("OUT_DIR"), "/agent_combined_routes.rs"));
}

mod web_bootstrap;

pub use generated::{APP_ROUTES, BACKEND_ROUTES, COMBINED_ROUTES, OPEN_ROUTES};

pub use web_bootstrap::{
    agent_request_context_injector, app_route_manifest, backend_route_manifest,
    build_served_combined_router, combined_route_manifest, open_route_manifest,
    wrap_router_with_web_framework, wrap_router_with_web_framework_from_env,
};

pub use sdkwork_intelligence_agents_service::{AgentHttpState, AgentRequestContext};


#[cfg(test)]
mod route_manifest_contracts {
    use super::*;

    #[test]
    fn generated_route_manifests_are_non_empty() {
        assert!(!APP_ROUTES.is_empty());
        assert!(!BACKEND_ROUTES.is_empty());
        assert!(!OPEN_ROUTES.is_empty());
        assert!(!COMBINED_ROUTES.is_empty());
    }

    #[test]
    fn route_manifest_helpers_build_from_generated_slices() {
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/agents")
            .is_some());
        assert!(backend_route_manifest()
            .match_route("GET", "/backend/v3/api/ai/agents")
            .is_some());
        assert!(open_route_manifest()
            .match_route("GET", "/agent/v3/api/ai/agents")
            .is_some());
        assert!(combined_route_manifest()
            .match_route("GET", "/app/v3/api/ai/agents")
            .is_some());
    }

    #[test]
    fn app_route_manifest_carries_the_expert_catalog_surface() {
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/expert_divisions")
            .is_some());
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/experts")
            .is_some());
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/experts/{slug}")
            .is_some());
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/expert_teams")
            .is_some());
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/expert_teams/{teamId}")
            .is_some());
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/expert_personas")
            .is_some());
        assert!(app_route_manifest()
            .match_route("GET", "/app/v3/api/ai/expert_collections")
            .is_some());
    }
}
