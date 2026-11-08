//! Proves chat turns resolve the CloudRouter target through the registry on
//! every turn, so a multi-process cluster load-balances dialogue traffic.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sdkwork_agents_tool_cloudrouter::CloudRouterTurnTransport;
use sdkwork_utils_rust::service_base_url::{
    install_http_service_registry, HttpServiceRegistry, RegistryHttpTarget,
};

struct AlternatingRegistry {
    next: AtomicUsize,
}

impl HttpServiceRegistry for AlternatingRegistry {
    fn resolve_http(&self, service: &str) -> Option<RegistryHttpTarget> {
        if service != "cloudrouter" {
            return None;
        }
        let index = self.next.fetch_add(1, Ordering::SeqCst);
        let (instance_id, base_url) = if index % 2 == 0 {
            ("registry-chat-a", "https://registry-a.example.com")
        } else {
            ("registry-chat-b", "https://registry-b.example.com")
        };
        Some(RegistryHttpTarget {
            service: "cloudrouter".to_owned(),
            instance_id: instance_id.to_owned(),
            base_url: base_url.to_owned(),
        })
    }

    fn report_failure(&self, _target: &RegistryHttpTarget, _reason: &str) {}

    fn report_success(&self, _target: &RegistryHttpTarget) {}
}

#[test]
fn chat_turns_alternate_registry_targets() {
    assert!(
        install_http_service_registry(Arc::new(AlternatingRegistry {
            next: AtomicUsize::new(0),
        })),
        "the registry slot is process-global and this test binary owns it"
    );
    std::env::set_var("SDKWORK_DEPLOYMENT_PROFILE", "cloud");
    std::env::remove_var("SDKWORK_AGENTS_CLOUDROUTER_BASE_URL");
    std::env::remove_var("SDKWORK_CLOUDROUTER_ROUTER_APPLICATION_PUBLIC_HTTP_URL");

    let first = match CloudRouterTurnTransport::resolve() {
        Ok(CloudRouterTurnTransport::Http { base_url }) => base_url,
        _ => panic!("expected the HTTP arm"),
    };
    let second = match CloudRouterTurnTransport::resolve() {
        Ok(CloudRouterTurnTransport::Http { base_url }) => base_url,
        _ => panic!("expected the HTTP arm"),
    };

    assert_eq!(first, "https://registry-a.example.com");
    assert_eq!(second, "https://registry-b.example.com");
}
