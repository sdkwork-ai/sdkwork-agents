//! Proves the media tool family (image/audio/video/music) resolves its
//! CloudRouter base URL through the database registry, not the old loopback
//! default.

use std::sync::Arc;

use sdkwork_agents_tool_cloudrouter::{cloudrouter_base_url, CloudRouterMediaClient};
use sdkwork_utils_rust::service_base_url::{
    install_http_service_registry, HttpServiceRegistry, RegistryHttpTarget,
};

struct FakeRegistry;

impl HttpServiceRegistry for FakeRegistry {
    fn resolve_http(&self, service: &str) -> Option<RegistryHttpTarget> {
        (service == "cloudrouter").then(|| RegistryHttpTarget {
            service: "cloudrouter".to_owned(),
            instance_id: "registry-media-1".to_owned(),
            base_url: "https://registry.example.com".to_owned(),
        })
    }

    fn report_failure(&self, _target: &RegistryHttpTarget, _reason: &str) {}

    fn report_success(&self, _target: &RegistryHttpTarget) {}
}

#[test]
fn media_tools_use_the_registry_target_over_the_loopback_default() {
    assert!(
        install_http_service_registry(Arc::new(FakeRegistry)),
        "the registry slot is process-global and this test binary owns it"
    );

    assert_eq!(cloudrouter_base_url(), "https://registry.example.com");
    assert_eq!(
        CloudRouterMediaClient::from_env().base_url(),
        "https://registry.example.com"
    );
}
