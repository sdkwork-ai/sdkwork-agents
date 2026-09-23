use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use sdkwork_agents_contract::env_test_lock;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_agents_service::{
    AgentHttpState, IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
};
use sdkwork_routes_agents_app_api::{
    app_route_manifest, build_router, wrap_router_with_web_framework, APP_ROUTES,
};
use sdkwork_web_contract::{HttpMethod, RouteAuth};
use sdkwork_web_core::{auth_token_jwt, encode_unsigned_test_jwt};
use serde_json::json;
use tower::util::ServiceExt;

const AGENTS_APP_ID: &str = "sdkwork-agents";
const DEFAULT_TENANT_ID: &str = "100001";
const DEFAULT_SESSION_ID: &str = "s-1";

fn http_method_name(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Delete => "DELETE",
        HttpMethod::Get => "GET",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
    }
}

fn test_state() -> AgentHttpState {
    AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.test.iam-gated"),
    )
}

fn test_app() -> axum::Router {
    wrap_router_with_web_framework(
        IamWebRequestContextResolver::new(None),
        app_route_manifest(),
        build_router().with_state(test_state()),
    )
}

fn agents_auth_token_bearer(user_id: &str) -> String {
    format!(
        "Bearer {}",
        auth_token_jwt(
            DEFAULT_TENANT_ID,
            user_id,
            DEFAULT_SESSION_ID,
            AGENTS_APP_ID
        )
    )
}

fn agents_access_token(user_id: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "access",
        "tenant_id": DEFAULT_TENANT_ID,
        "user_id": user_id,
        "session_id": DEFAULT_SESSION_ID,
        "app_id": AGENTS_APP_ID,
        "environment": "dev",
        "deployment_mode": "saas",
        "login_scope": "TENANT",
        // token-claims-gate: legacy-fixture — constructs a pre-slimming credential so this
        // test can assert the claim is no longer an authorization source.
        "permission_scope": ["ai.agents.manage"]
    }))
}

#[test]
fn app_route_manifest_covers_all_openapi_operations() {
    let manifest = app_route_manifest();
    assert!(!APP_ROUTES.is_empty());
    for entry in APP_ROUTES {
        let matched = manifest
            .match_route(http_method_name(entry.method), entry.path)
            .unwrap_or_else(|| {
                panic!(
                    "missing http route manifest for {:?} {}",
                    entry.method, entry.path
                );
            });
        assert_eq!(matched.auth, RouteAuth::DualToken);
        assert_eq!(matched.operation_id, entry.operation_id);
    }
}

#[tokio::test]
async fn app_router_web_framework_rejects_unauthenticated_requests() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/app/v3/api/ai/agents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn app_router_web_framework_accepts_dev_jwt_dual_tokens() {
    let _guard = env_test_lock();
    std::env::set_var("SDKWORK_ENV", "dev");
    std::env::set_var("SDKWORK_IAM_ALLOW_DEV_AUTH_FALLBACK", "true");
    std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "development");
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/app/v3/api/ai/agents")
                    .header("Authorization", agents_auth_token_bearer("30001"))
                    .header("Access-Token", agents_access_token("30001"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn app_router_web_framework_accepts_browser_origin_in_development() {
    let _guard = env_test_lock();
    std::env::set_var("SDKWORK_ENV", "dev");
    std::env::set_var("SDKWORK_IAM_ALLOW_DEV_AUTH_FALLBACK", "true");
    std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "development");
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/app/v3/api/ai/agents")
                    .header("origin", "http://localhost:4176")
                    .header("Authorization", agents_auth_token_bearer("30001"))
                    .header("Access-Token", agents_access_token("30001"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::FORBIDDEN);
    });
}

#[test]
fn app_router_web_framework_enforces_exact_production_cors_origins() {
    let _guard = env_test_lock();
    std::env::set_var("SDKWORK_ENVIRONMENT", "production");
    std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "production");
    std::env::set_var("SDKWORK_CORS_ALLOWED_ORIGINS", "https://agents.sdkwork.com");
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let allowlisted_response = test_app()
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/app/v3/api/ai/agents")
                    .header("origin", "https://agents.sdkwork.com")
                    .header("access-control-request-method", "POST")
                    .header("access-control-request-headers", "content-type")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(allowlisted_response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            allowlisted_response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok()),
            Some("https://agents.sdkwork.com")
        );
        assert_ne!(
            allowlisted_response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok()),
            Some("*")
        );
        assert!(allowlisted_response
            .headers()
            .get(header::VARY)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case("origin"))));

        let rejected_response = test_app()
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/app/v3/api/ai/agents")
                    .header("origin", "https://evil.example")
                    .header("access-control-request-method", "POST")
                    .header("access-control-request-headers", "content-type")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(rejected_response.status(), StatusCode::FORBIDDEN);
    });
    std::env::remove_var("SDKWORK_ENVIRONMENT");
    std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
    std::env::remove_var("SDKWORK_CORS_ALLOWED_ORIGINS");
}
