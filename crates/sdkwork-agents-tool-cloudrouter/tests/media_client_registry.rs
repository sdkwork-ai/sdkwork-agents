//! Proves the media tool family (image/audio/video/music) resolves its
//! CloudRouter base URL through the database registry **per invocation**, so a
//! multi-process cluster load-balances across replicas instead of pinning the
//! first target for the process lifetime.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sdkwork_agents_tool_cloudrouter::CloudRouterMediaClient;
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
            ("registry-media-a", "https://registry-a.example.com")
        } else {
            ("registry-media-b", "https://registry-b.example.com")
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
fn media_tools_alternate_registry_targets_per_call() {
    assert!(
        install_http_service_registry(Arc::new(AlternatingRegistry {
            next: AtomicUsize::new(0),
        })),
        "the registry slot is process-global and this test binary owns it"
    );

    let first_client = CloudRouterMediaClient::from_env();
    let second_client = CloudRouterMediaClient::from_env();
    let first = first_client.base_url().to_owned();
    let second = second_client.base_url().to_owned();

    assert_eq!(first, "https://registry-a.example.com");
    assert_eq!(second, "https://registry-b.example.com");
}
