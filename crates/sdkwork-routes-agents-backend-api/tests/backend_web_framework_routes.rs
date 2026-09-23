use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_agents_contract::env_test_lock;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_agents_service::{
    AgentHttpState, IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
};
use sdkwork_routes_agents_backend_api::{
    backend_route_manifest, build_router, wrap_router_with_web_framework,
};
use sdkwork_web_core::{auth_token_jwt, encode_unsigned_test_jwt};
use serde_json::json;
use tower::util::ServiceExt;

const AGENTS_APP_ID: &str = "sdkwork-agents";
const DEFAULT_TENANT_ID: &str = "100001";
const DEFAULT_SESSION_ID: &str = "s-1";

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
        backend_route_manifest(),
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

#[tokio::test]
async fn backend_router_web_framework_rejects_unauthenticated_requests() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/backend/v3/api/ai/agents?tenant_id=100001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn backend_router_web_framework_accepts_dev_jwt_dual_tokens() {
    let _guard = env_test_lock();
    std::env::set_var("SDKWORK_ENV", "dev");
    std::env::set_var("SDKWORK_IAM_ALLOW_DEV_AUTH_FALLBACK", "true");
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/backend/v3/api/ai/agents?tenant_id=100001")
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
