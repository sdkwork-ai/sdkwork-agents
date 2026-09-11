use crate::generated::{APP_ROUTES, BACKEND_ROUTES, COMBINED_ROUTES, OPEN_ROUTES};
use std::sync::Arc;

use axum::http::{Request, Uri};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_agents_service::AgentRequestContext;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DomainContextInjector, HttpRouteManifest, SecurityPolicy, WebEnvironment, WebRequestContext,
    WebRequestContextProfile,
};

fn canonical_agents_lifecycle_environment(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "dev" | "development" | "local" => "development".to_owned(),
        "test" | "testing" => "test".to_owned(),
        "stage" | "staging" => "staging".to_owned(),
        "prod" | "production" | "live" => "production".to_owned(),
        other => other.to_owned(),
    }
}

fn parse_agents_web_environment(value: Option<String>) -> WebEnvironment {
    match canonical_agents_lifecycle_environment(value.as_deref().unwrap_or("")).as_str() {
        "development" => WebEnvironment::Dev,
        "test" => WebEnvironment::Test,
        // Demo is an isolated showcase tier, not production-like: it gets the
        // relaxed showcase posture instead of production assembly validation.
        "demo" => WebEnvironment::Test,
        // Staging/prod keep the strict fail-closed production posture.
        "staging" | "production" => WebEnvironment::Prod,
        _ => WebEnvironment::Prod,
    }
}

fn first_nonempty_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        std::env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn resolve_agents_web_environment_from_process_env() -> WebEnvironment {
    let shared_environment = first_nonempty_env(&["SDKWORK_ENVIRONMENT"]);
    let agents_environment = first_nonempty_env(&["SDKWORK_AGENTS_ENVIRONMENT"]);
    if let (Some(shared_environment), Some(agents_environment)) =
        (&shared_environment, &agents_environment)
    {
        if canonical_agents_lifecycle_environment(shared_environment)
            != canonical_agents_lifecycle_environment(agents_environment)
        {
            panic!(
                "SDKWORK_ENVIRONMENT and SDKWORK_AGENTS_ENVIRONMENT resolve to different lifecycle environments"
            );
        }
    }
    parse_agents_web_environment(
        shared_environment
            .or(agents_environment)
            .or_else(|| first_nonempty_env(&["SDKWORK_ENV", "ENVIRONMENT"])),
    )
}

fn configured_agents_cors_origins_from_process_env() -> Vec<String> {
    sdkwork_web_bootstrap::cors_allowed_origins_from_env(&["SDKWORK_CORS_ALLOWED_ORIGINS"])
}

fn is_exact_http_origin(origin: &str) -> bool {
    let Ok(uri) = origin.parse::<Uri>() else {
        return false;
    };
    let Some(scheme) = uri.scheme_str() else {
        return false;
    };
    let Some(authority) = uri.authority() else {
        return false;
    };
    matches!(scheme, "http" | "https")
        && !authority.as_str().contains('@')
        && !origin.contains('*')
        && origin == format!("{scheme}://{authority}")
}

fn ensure_production_cors_configuration(
    requires_exact_origins: bool,
    has_configured_origins: bool,
    security_policy: &SecurityPolicy,
) {
    if !requires_exact_origins {
        return;
    }

    // Registered SDKWork client origins are always merged into the policy, so
    // emptiness must be judged against the explicitly configured allowlist:
    // production-like runtimes still demand SDKWORK_CORS_ALLOWED_ORIGINS.
    if !has_configured_origins {
        panic!(
            "production-like Agents HTTP runtime requires SDKWORK_CORS_ALLOWED_ORIGINS"
        );
    }

    // Registered SDKWork desktop WebView and mini program custom-scheme origins
    // (app://dsh, app://birdcoder, tauri://localhost, https://servicewechat.com,
    // ...) are canonical allowlist entries merged by the shared web bootstrap;
    // they are exempt from the exact HTTP(S) origin shape check.
    if let Some(origin) = security_policy
        .cors
        .allowed_origins
        .iter()
        .find(|origin| {
            !is_exact_http_origin(origin)
                && !sdkwork_web_core::is_registered_sdkwork_client_origin(origin)
        })
    {
        panic!("invalid exact HTTP(S) origin in production-like Agents HTTP CORS configuration: {origin}");
    }

    if let Err(error) = security_policy.cors.validate_for_production() {
        panic!("invalid production-like Agents HTTP CORS configuration: {error}");
    }
}

/// Agents HTTP security policy aligned with the shared Web Framework bootstrap behavior.
fn agents_service_security_policy(environment: &WebEnvironment) -> SecurityPolicy {
    let configured_origins = configured_agents_cors_origins_from_process_env();
    let has_configured_origins = !configured_origins.is_empty();
    let requires_exact_origins = matches!(environment, WebEnvironment::Prod)
        || matches!(environment, WebEnvironment::Test) && has_configured_origins;
    let cors_environment = if matches!(environment, WebEnvironment::Test) && has_configured_origins
    {
        WebEnvironment::Prod
    } else {
        environment.clone()
    };
    let cors = sdkwork_web_bootstrap::security_policy_for_environment(
        &cors_environment,
        configured_origins,
    )
    .cors;
    let use_development_security_policy = matches!(environment, WebEnvironment::Dev)
        || matches!(environment, WebEnvironment::Test) && !has_configured_origins;
    let mut security_policy = if use_development_security_policy {
        SecurityPolicy::default()
    } else {
        SecurityPolicy::production()
    };
    security_policy.cors = cors;
    if use_development_security_policy {
        security_policy
            .cross_site
            .reject_untrusted_state_changing_origins = false;
        security_policy.cross_site.reject_cookie_auth_without_origin = false;
    }
    ensure_production_cors_configuration(
        requires_exact_origins,
        has_configured_origins,
        &security_policy,
    );
    security_policy
}

pub fn app_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(APP_ROUTES)
}

pub fn backend_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(BACKEND_ROUTES)
}

pub fn open_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(OPEN_ROUTES)
}

pub fn combined_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(COMBINED_ROUTES)
}

fn agent_web_request_context_profile(environment: WebEnvironment) -> WebRequestContextProfile {
    WebRequestContextProfile {
        open_api_prefixes: vec![
            "/agent/v3/api".to_owned(),
            sdkwork_web_core::OPEN_API_PREFIX.to_owned(),
        ],
        environment,
        ..WebRequestContextProfile::default()
    }
}

#[derive(Clone, Default)]
struct AgentRequestContextInjector;

impl DomainContextInjector for AgentRequestContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(agent_context) = agent_request_context_from_web_request(context) {
            request.extensions_mut().insert(agent_context);
        }
    }
}

/// Returns the Agents domain-context contribution for a host-owned Web Framework pipeline.
pub fn agent_request_context_injector() -> Arc<dyn DomainContextInjector> {
    Arc::new(AgentRequestContextInjector)
}

fn agent_request_context_from_web_request(
    context: &WebRequestContext,
) -> Option<AgentRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().to_string();
    let subject_id = principal.user_id().to_string();
    let organization_id = principal.organization_id().map(str::to_owned);
    let roles = principal.scopes.permission_scope.clone();
    let mut agent_context = AgentRequestContext::new(tenant_id, subject_id.clone())
        .with_subject_id(subject_id)
        .with_roles(roles);
    if let Some(organization_id) = organization_id {
        agent_context = agent_context.with_organization_id(organization_id);
    }
    Some(agent_context)
}

pub fn wrap_router_with_web_framework(
    resolver: IamWebRequestContextResolver,
    route_manifest: HttpRouteManifest,
    router: Router,
) -> Router {
    let environment = resolve_agents_web_environment_from_process_env();
    let security_policy = agents_service_security_policy(&environment);
    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(agent_web_request_context_profile(environment))
        .with_security_policy(security_policy)
        .with_route_manifest(route_manifest)
        .with_domain_injector(agent_request_context_injector());
    with_web_request_context(router, layer)
}

pub async fn wrap_router_with_web_framework_from_env(
    route_manifest: HttpRouteManifest,
    router: Router,
) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_router_with_web_framework(resolver, route_manifest, router)
}

async fn record_agents_request_metrics(request: Request<axum::body::Body>, next: Next) -> Response {
    let response = next.run(request).await;
    sdkwork_intelligence_agents_service::AgentMetricsRegistry::global()
        .record_http_request(response.status().as_u16());
    response
}

/// Builds a combined agents managed store router with mandatory sdkwork-web-framework middleware.
pub async fn build_served_combined_router(
    state: sdkwork_intelligence_agents_service::AgentHttpState,
) -> Router {
    let router = sdkwork_intelligence_agents_service::build_combined_routes().with_state(state);
    let router = router.layer(middleware::from_fn(record_agents_request_metrics));
    wrap_router_with_web_framework_from_env(combined_route_manifest(), router).await
}

#[cfg(test)]
mod tests {
    use super::{
        agents_service_security_policy, parse_agents_web_environment,
        resolve_agents_web_environment_from_process_env,
    };
    use sdkwork_agents_contract::env_test_lock;
    use sdkwork_web_core::WebEnvironment;

    #[test]
    fn dev_security_policy_allows_browser_origins() {
        let policy = agents_service_security_policy(&WebEnvironment::Dev);
        assert!(!policy.cors.allow_all_origins);
        assert!(!policy.cross_site.reject_untrusted_state_changing_origins);
    }

    #[test]
    fn production_security_policy_rejects_permissive_cors() {
        let _guard = env_test_lock();
        std::env::set_var("SDKWORK_CORS_ALLOWED_ORIGINS", "https://agents.sdkwork.com");
        let policy = agents_service_security_policy(&WebEnvironment::Prod);
        std::env::remove_var("SDKWORK_CORS_ALLOWED_ORIGINS");
        assert!(!policy.cors.allow_all_origins);
        policy
            .cors
            .validate_origin_value("https://agents.sdkwork.com")
            .expect("configured production origin");
        policy
            .cors
            .validate_origin_value("https://evil.example")
            .expect_err("unknown production origin");
    }

    #[test]
    fn production_security_policy_allows_registered_client_origins() {
        let _guard = env_test_lock();
        std::env::set_var("SDKWORK_CORS_ALLOWED_ORIGINS", "https://agents.sdkwork.com");
        let policy = agents_service_security_policy(&WebEnvironment::Prod);
        std::env::remove_var("SDKWORK_CORS_ALLOWED_ORIGINS");
        for origin in [
            "app://dsh",
            "app://birdcoder",
            "app://sdkwork",
            "https://servicewechat.com",
        ] {
            policy
                .cors
                .validate_origin_value(origin)
                .expect("registered SDKWork client origin");
        }
        policy
            .cors
            .validate_origin_value("app://unregistered")
            .expect_err("unregistered custom scheme must stay rejected");
    }

    #[test]
    #[should_panic(expected = "requires SDKWORK_CORS_ALLOWED_ORIGINS")]
    fn production_security_policy_requires_explicit_origin_allowlist() {
        let _guard = env_test_lock();
        std::env::remove_var("SDKWORK_CORS_ALLOWED_ORIGINS");
        let _ = agents_service_security_policy(&WebEnvironment::Prod);
    }

    #[test]
    fn test_security_policy_uses_exact_origins_when_configured() {
        let _guard = env_test_lock();
        std::env::set_var(
            "SDKWORK_CORS_ALLOWED_ORIGINS",
            "https://test.agents.sdkwork.com",
        );
        let policy = agents_service_security_policy(&WebEnvironment::Test);
        std::env::remove_var("SDKWORK_CORS_ALLOWED_ORIGINS");
        policy
            .cors
            .validate_origin_value("https://test.agents.sdkwork.com")
            .expect("configured test origin");
        policy
            .cors
            .validate_origin_value("http://127.0.0.1:5173")
            .expect_err("test profile with exact origins must reject development wildcard origins");
    }

    #[test]
    fn parse_environment_from_agents_profile() {
        assert_eq!(
            parse_agents_web_environment(Some("development".to_owned())),
            WebEnvironment::Dev
        );
        assert_eq!(
            parse_agents_web_environment(Some("production".to_owned())),
            WebEnvironment::Prod
        );
        assert_eq!(
            parse_agents_web_environment(Some("staging".to_owned())),
            WebEnvironment::Prod
        );
    }

    #[test]
    fn resolve_environment_from_agents_env_key() {
        let _guard = env_test_lock();
        std::env::remove_var("SDKWORK_ENVIRONMENT");
        unsafe {
            std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "development");
        }
        assert_eq!(
            resolve_agents_web_environment_from_process_env(),
            WebEnvironment::Dev
        );
        unsafe {
            std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        }
    }

    #[test]
    fn resolve_environment_uses_shared_environment_setting() {
        let _guard = env_test_lock();
        std::env::set_var("SDKWORK_ENVIRONMENT", "test");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        assert_eq!(
            resolve_agents_web_environment_from_process_env(),
            WebEnvironment::Test
        );
        std::env::remove_var("SDKWORK_ENVIRONMENT");
    }

    #[test]
    fn shared_and_agents_environment_settings_must_agree() {
        let _guard = env_test_lock();
        std::env::set_var("SDKWORK_ENVIRONMENT", "test");
        std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "production");
        let result = std::panic::catch_unwind(resolve_agents_web_environment_from_process_env);
        std::env::remove_var("SDKWORK_ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        assert!(result.is_err());
    }

    #[test]
    fn shared_and_agents_staging_and_production_settings_must_not_converge_silently() {
        let _guard = env_test_lock();
        std::env::set_var("SDKWORK_ENVIRONMENT", "staging");
        std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "production");
        let result = std::panic::catch_unwind(resolve_agents_web_environment_from_process_env);
        std::env::remove_var("SDKWORK_ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        assert!(result.is_err());
    }
}
