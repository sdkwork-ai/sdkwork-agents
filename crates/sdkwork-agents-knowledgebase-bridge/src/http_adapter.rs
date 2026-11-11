//! Explicit split-deployment transport for Knowledgebase grounding
//! (MODULE_SPEC.md §4.2.3).
//!
//! This module is the **only** place that reads
//! [`KNOWLEDGEBASE_BASE_URL_ENV`]; it is reached only when the deployment sets
//! `SDKWORK_AGENTS_KNOWLEDGEBASE_MODE=http`, and the origin is then required —
//! [`KnowledgebaseGroundingResolver::from_env`] fails bootstrap instead of
//! degrading at the first turn. The default mode is in-process
//! ([`crate::InProcessKnowledgebaseGroundingResolver`]).
//!
//! The transport call forwards the caller's dual tokens (`Authorization` +
//! `Access-Token`) unchanged, so the Knowledgebase App API resolves the actor
//! and enforces space ACLs exactly as it does for the caller's own requests.
//! This adapter never selects a tenant, actor, or space outside the
//! authenticated token.
//!
//! The call runs on a purpose-built scoped thread with its own current-thread
//! Tokio runtime, so a turn executing inside a Tokio blocking worker can never
//! nest runtimes, and a slow backend cannot pin the caller beyond
//! [`crate::DEFAULT_TIMEOUT`] (overridable per resolver).

use std::time::Duration;

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_intelligence_agents_service::{
    GroundingRequest, KnowledgeGroundingResolver, KnowledgeGroundingSnippet,
};
use sdkwork_knowledgebase_app_sdk_generated_rust::SdkworkAppClient;
use sdkwork_utils_rust::service_base_url::{registry_base_url_or, report_registry_result};

use crate::{build_retrieval_request, map_retrieval_result, provider_error, run_sdk_call};

/// Environment variable holding the Knowledgebase App API origin.
///
/// Read only by the explicit HTTP opt-in (`SDKWORK_AGENTS_KNOWLEDGEBASE_MODE=http`).
pub const KNOWLEDGEBASE_BASE_URL_ENV: &str = "SDKWORK_AGENTS_KNOWLEDGEBASE_BASE_URL";

/// Knowledgebase App API transport (split-deployment opt-in only).
pub struct KnowledgebaseGroundingResolver {
    base_url: String,
    timeout: Duration,
}

impl KnowledgebaseGroundingResolver {
    /// Builds the transport from `SDKWORK_AGENTS_KNOWLEDGEBASE_BASE_URL`.
    ///
    /// This adapter is only reachable behind the explicit HTTP opt-in, so a
    /// missing, blank, or non-http origin is a deployment misconfiguration: it
    /// fails bootstrap rather than silently disabling retrieval after the host
    /// has started serving (MODULE_SPEC.md §4.2.3).
    pub fn from_env() -> Result<Self, String> {
        Self::new(required_origin(
            std::env::var(KNOWLEDGEBASE_BASE_URL_ENV).ok().as_deref(),
        )?)
    }

    /// Builds the resolver for an explicit Knowledgebase App API origin.
    pub fn new(base_url: impl Into<String>) -> Result<Self, String> {
        let base_url = base_url.into();
        let trimmed = base_url.trim();
        if trimmed.is_empty() {
            return Err(format!(
                "{KNOWLEDGEBASE_BASE_URL_ENV} must not be blank when knowledge grounding is enabled"
            ));
        }
        if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            return Err(format!(
                "{KNOWLEDGEBASE_BASE_URL_ENV} must be an absolute http(s) origin"
            ));
        }
        Ok(Self {
            base_url: trimmed.trim_end_matches('/').to_string(),
            timeout: crate::DEFAULT_TIMEOUT,
        })
    }

    /// Overrides the per-turn retrieval deadline.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }
}

/// Requires the opt-in origin at bootstrap.
fn required_origin(value: Option<&str>) -> Result<String, String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "{KNOWLEDGEBASE_BASE_URL_ENV} is required when {}={}",
                crate::KNOWLEDGEBASE_MODE_ENV,
                crate::MODE_HTTP
            )
        })
}

impl KnowledgeGroundingResolver for KnowledgebaseGroundingResolver {
    fn resolve_knowledge(
        &self,
        request: &GroundingRequest<'_>,
    ) -> KernelResult<Vec<KnowledgeGroundingSnippet>> {
        if request.knowledge.is_empty() {
            return Ok(Vec::new());
        }
        let Some(auth_token) = request.auth_token.filter(|value| !value.trim().is_empty()) else {
            return Err(KernelError::validation(
                "knowledge grounding requires the caller auth token; refusing to retrieve without an authenticated actor",
            ));
        };

        let body = build_retrieval_request(request);
        // DISCOVERY_SPEC §12.1: registry-first base URL, configured value as
        // the degradation fallback.
        let base_url = registry_base_url_or("sdkwork-knowledgebase", &self.base_url);
        let client = SdkworkAppClient::new_with_base_url(base_url.clone())
            .map_err(|error| provider_error("knowledgebase.client_init_failed", error))?;
        client.set_auth_token(auth_token);
        if let Some(access_token) = request
            .access_token
            .filter(|value| !value.trim().is_empty())
        {
            client.set_access_token(access_token);
        }

        let timeout = self.timeout;
        let result = run_sdk_call(|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| {
                    provider_error("knowledgebase.runtime_init_failed", error.to_string())
                })?;
            runtime.block_on(async {
                match tokio::time::timeout(timeout, client.knowledge().retrievals_create(&body))
                    .await
                {
                    Ok(result) => result
                        .map_err(|error| provider_error("knowledgebase.retrieval_failed", error)),
                    Err(_) => Err(KernelError::provider_error(
                        "knowledgebase.retrieval_timeout",
                        format!(
                            "knowledgebase retrieval did not complete within {} ms",
                            timeout.as_millis()
                        ),
                    )),
                }
            })
        });
        report_registry_result(
            "sdkwork-knowledgebase",
            &base_url,
            result.is_ok(),
            "knowledgebase retrieval failed",
        );
        let result = result?;

        Ok(map_retrieval_result(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_blank_and_non_http_origins() {
        assert!(KnowledgebaseGroundingResolver::new("ftp://knowledgebase").is_err());
        assert!(KnowledgebaseGroundingResolver::new("   ").is_err());
        assert!(KnowledgebaseGroundingResolver::new("https://kb.example.test").is_ok());
    }

    #[test]
    fn http_opt_in_requires_an_origin_at_bootstrap() {
        let error = required_origin(None).expect_err("missing origin must fail bootstrap");
        assert!(error.contains("SDKWORK_AGENTS_KNOWLEDGEBASE_BASE_URL"));
        assert!(error.contains("SDKWORK_AGENTS_KNOWLEDGEBASE_MODE"));
        assert!(required_origin(Some("   ")).is_err());
        assert_eq!(
            required_origin(Some(" https://kb.example.test ")).expect("origin"),
            "https://kb.example.test"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn resolver_calls_the_app_retrieval_endpoint_with_dual_tokens() {
        use crate::tests_support::{binding, request};
        use serde_json::json;
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let body = json!({
            "code": 0,
            "data": {
                "item": {
                    "retrievalId": "7001",
                    "hits": [{
                        "chunkId": "doc-1-chunk",
                        "documentId": "doc-1",
                        "spaceId": "4242",
                        "title": "Refund policy",
                        "content": "Refunds are accepted within 30 days.",
                        "score": 0.93,
                        "rank": 1,
                        "retrievalMethod": "hybrid",
                        "citation": {
                            "documentId": "doc-1",
                            "title": "Refund policy",
                            "sourceUri": "drive://kb/doc-1"
                        }
                    }]
                }
            },
            "traceId": "00000000-0000-4000-8000-000000000000"
        });
        Mock::given(method("POST"))
            .and(path("/app/v3/api/knowledge/retrievals"))
            .and(header("Authorization", "Bearer auth-token"))
            .and(header("Access-Token", "access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let resolver = KnowledgebaseGroundingResolver::new(server.uri()).expect("resolver");
        // The resolver blocks on its own worker thread; run it off the async
        // workers so the mock server can still answer.
        let snippets = tokio::task::spawn_blocking(move || {
            let bindings = [binding(4242, Some(4))];
            resolver.resolve_knowledge(&request(&bindings))
        })
        .await
        .expect("grounding task")
        .expect("grounding snippets");

        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].document_id, "doc-1");
        assert!(snippets[0].snippet.contains("30 days"));
        assert_eq!(snippets[0].source_uri.as_deref(), Some("drive://kb/doc-1"));
    }
}
