//! Proves a failed HTTP turn reports the target to the registry resolver, so
//! turn traffic participates in failure demotion and failover.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use sdkwork_agents_tool_cloudrouter::{
    CloudRouterTurnRequest, CloudRouterTurnTransport, WireProtocol,
};
use sdkwork_utils_rust::service_base_url::{
    install_http_service_registry, HttpServiceRegistry, RegistryHttpTarget,
};

struct CountingRegistry {
    failures: AtomicUsize,
}

impl HttpServiceRegistry for CountingRegistry {
    fn resolve_http(&self, service: &str) -> Option<RegistryHttpTarget> {
        (service == "cloudrouter").then(|| RegistryHttpTarget {
            service: "cloudrouter".to_owned(),
            instance_id: "http-1".to_owned(),
            base_url: "http://127.0.0.1:1".to_owned(),
        })
    }

    fn report_failure(&self, _target: &RegistryHttpTarget, _reason: &str) {
        self.failures.fetch_add(1, Ordering::SeqCst);
    }

    fn report_success(&self, _target: &RegistryHttpTarget) {}
}

#[test]
fn a_failed_http_turn_is_reported_to_the_registry() {
    let registry = Arc::new(CountingRegistry {
        failures: AtomicUsize::new(0),
    });
    assert!(
        install_http_service_registry(registry.clone()),
        "the registry slot is process-global and this test binary owns it"
    );

    let transport = CloudRouterTurnTransport::Http {
        base_url: "http://127.0.0.1:1".to_owned(),
    };
    let mut request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
    request.model = "gpt-4o-mini".to_owned();
    let mut deltas = Vec::new();

    let result = transport.stream_chat_completion(
        CloudRouterTurnRequest {
            protocol: WireProtocol::ChatCompletions,
            auth_token: "test-auth",
            access_token: None,
            request,
            tools: Vec::new(),
        },
        &mut |delta| deltas.push(delta.content),
    );

    assert!(result.is_err(), "an unreachable target must fail the turn");
    assert!(
        registry.failures.load(Ordering::SeqCst) >= 1,
        "the failed turn must report the target to the registry"
    );
}
