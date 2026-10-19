//! Knowledgebase grounding adapter for SDKWork Agents turns.
//!
//! The Agents service owns conversation orchestration; knowledge content and
//! indexes stay in `sdkwork-knowledgebase`. This crate is the composition-root
//! adapter that turns the service's `KnowledgeGroundingResolver` port into one
//! bounded call to the Knowledgebase App API using the generator-owned Rust
//! transport SDK.
//!
//! Scope and safety:
//!
//! - The caller's dual tokens (`Authorization` + `Access-Token`) are forwarded
//!   unchanged, so the Knowledgebase App API resolves the actor and enforces
//!   space ACLs exactly as it does for the caller's own requests. This adapter
//!   never selects a tenant, actor, or space outside the authenticated token.
//! - The transport call runs on a purpose-built scoped thread with its own
//!   current-thread Tokio runtime, so a turn executing inside a Tokio blocking
//!   worker can never nest runtimes, and a slow backend cannot pin the caller
//!   beyond [`DEFAULT_TIMEOUT`] (overridable per resolver).
//! - Transport failures are typed kernel provider errors. The Agents turn path
//!   degrades to an ungrounded answer and records an operator warning; it does
//!   not fail the turn.
//!
//! Configuration: set `SDKWORK_AGENTS_KNOWLEDGEBASE_BASE_URL` to the
//! Knowledgebase App API origin. When unset,
//! [`KnowledgebaseGroundingResolver::from_env`] returns `Ok(None)` and the
//! composition root simply does not register the adapter; knowledge slots then
//! degrade with a warning instead of failing.

use std::time::Duration;

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_intelligence_agents_service::{
    GroundingRequest, KnowledgeGroundingResolver, KnowledgeGroundingSnippet, MAX_KNOWLEDGE_SNIPPETS,
};
use sdkwork_knowledgebase_app_sdk_generated_rust::models::{
    KnowledgeRetrievalBinding, KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
};
use sdkwork_knowledgebase_app_sdk_generated_rust::SdkworkAppClient;

/// Environment variable holding the Knowledgebase App API origin.
pub const KNOWLEDGEBASE_BASE_URL_ENV: &str = "SDKWORK_AGENTS_KNOWLEDGEBASE_BASE_URL";
/// Default per-turn retrieval deadline.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);
/// Retrieval methods requested from the Knowledgebase hybrid profile.
pub const RETRIEVAL_METHODS: [&str; 1] = ["hybrid"];

/// Per-turn Knowledgebase grounding resolver.
pub struct KnowledgebaseGroundingResolver {
    base_url: String,
    timeout: Duration,
}

impl KnowledgebaseGroundingResolver {
    /// Builds the resolver from `SDKWORK_AGENTS_KNOWLEDGEBASE_BASE_URL`.
    ///
    /// Returns `Ok(None)` when the variable is absent or blank: knowledge
    /// grounding is an optional capability module, not a boot requirement.
    pub fn from_env() -> Result<Option<Self>, String> {
        let base_url = std::env::var(KNOWLEDGEBASE_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        match base_url {
            Some(base_url) => Self::new(base_url).map(Some),
            None => Ok(None),
        }
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
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// Overrides the per-turn retrieval deadline.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }
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
        let client = SdkworkAppClient::new_with_base_url(self.base_url.clone())
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
        })?;

        Ok(map_retrieval_result(result))
    }
}

/// Maps Agents grounding bindings to the Knowledgebase retrieval contract.
///
/// The binding order is preserved (highest-priority slot first as resolved by
/// the service). `includeCitations` is always on so the renderer can carry a
/// stable source URI; actor and tenant are never sent - the App API derives
/// both from the forwarded credential.
#[must_use]
pub fn build_retrieval_request(request: &GroundingRequest<'_>) -> KnowledgeRetrievalRequest {
    let bindings = request
        .knowledge
        .iter()
        .map(|binding| KnowledgeRetrievalBinding {
            space_id: binding.space_id.to_string(),
            priority: 0,
            top_k: binding.top_k.map(i64::from),
            min_score: binding.min_score,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    let top_k = request
        .knowledge
        .iter()
        .filter_map(|binding| binding.top_k)
        .max()
        .map(i64::from);

    KnowledgeRetrievalRequest {
        query: request.query.trim().to_string(),
        bindings,
        methods: Some(
            RETRIEVAL_METHODS
                .iter()
                .map(|method| (*method).to_string())
                .collect(),
        ),
        top_k,
        include_citations: true,
        include_trace: false,
        ..Default::default()
    }
}

/// Maps one Knowledgebase retrieval result into bounded grounding snippets.
#[must_use]
pub fn map_retrieval_result(result: KnowledgeRetrievalResult) -> Vec<KnowledgeGroundingSnippet> {
    result
        .hits
        .into_iter()
        .take(MAX_KNOWLEDGE_SNIPPETS)
        .map(|hit| {
            let source_uri = hit
                .citation
                .as_ref()
                .and_then(|citation| citation.source_uri.clone());
            KnowledgeGroundingSnippet {
                document_id: hit.document_id,
                title: hit.title,
                snippet: hit.content,
                score: hit.score,
                source_uri,
                retrieval_method: Some(hit.retrieval_method),
            }
        })
        .collect()
}

fn run_sdk_call<T>(work: impl FnOnce() -> KernelResult<T> + Send) -> KernelResult<T>
where
    T: Send,
{
    std::thread::scope(|scope| {
        let handle = std::thread::Builder::new()
            .name("agents-kb-grounding".to_string())
            .spawn_scoped(scope, work)
            .map_err(|error| {
                provider_error("knowledgebase.worker_spawn_failed", error.to_string())
            })?;
        handle.join().map_err(|_| {
            KernelError::provider_error(
                "knowledgebase.worker_panicked",
                "knowledgebase grounding worker panicked",
            )
        })?
    })
}

fn provider_error(code: &str, detail: impl ToString) -> KernelError {
    KernelError::provider_error(code, detail.to_string())
        .with_provider("provider.knowledge.sdkwork-knowledgebase")
        .from_source(sdkwork_agent_kernel::KernelErrorSource::Provider)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_intelligence_agents_service::KnowledgeGroundingBinding;
    use sdkwork_knowledgebase_app_sdk_generated_rust::models::{
        KnowledgeCitation, KnowledgeContextFragment,
    };

    fn binding(space_id: u64, top_k: Option<u32>) -> KnowledgeGroundingBinding {
        KnowledgeGroundingBinding {
            slot_id: format!("slot.knowledge.{space_id}"),
            space_id,
            top_k,
            min_score: Some(0.2),
            title: None,
        }
    }

    fn request<'a>(knowledge: &'a [KnowledgeGroundingBinding]) -> GroundingRequest<'a> {
        GroundingRequest {
            tenant_id: 100_001,
            organization_id: 0,
            actor_id: Some(7),
            agent_id: "agent.chat.default",
            session_id: "session.1",
            query: "  how do refunds work  ",
            knowledge,
            memory: &[],
            auth_token: Some("auth-token"),
            access_token: Some("access-token"),
        }
    }

    fn fragment(document_id: &str, title: &str, content: &str) -> KnowledgeContextFragment {
        KnowledgeContextFragment {
            chunk_id: format!("{document_id}-chunk"),
            document_id: document_id.to_string(),
            space_id: "4242".to_string(),
            title: title.to_string(),
            content: content.to_string(),
            score: Some(0.9),
            rank: 1,
            retrieval_method: "hybrid".to_string(),
            citation: Some(KnowledgeCitation {
                document_id: document_id.to_string(),
                title: title.to_string(),
                source_uri: Some(format!("drive://kb/{document_id}")),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn request_maps_space_bindings_and_trims_the_query() {
        let bindings = [binding(4242, Some(4)), binding(9001, None)];
        let request = request(&bindings);
        let body = build_retrieval_request(&request);
        assert_eq!(body.query, "how do refunds work");
        assert_eq!(body.bindings.len(), 2);
        assert_eq!(body.bindings[0].space_id, "4242");
        assert_eq!(body.bindings[0].top_k, Some(4));
        assert_eq!(body.bindings[1].space_id, "9001");
        assert_eq!(body.top_k, Some(4));
        assert!(body.include_citations);
        assert_eq!(body.methods, Some(vec!["hybrid".to_string()]));
        assert!(body.retrieval_profile_id.is_none());
        assert!(body.actor_id.is_none());
    }

    #[test]
    fn result_maps_hits_to_grounding_snippets_with_source_uri() {
        let result = KnowledgeRetrievalResult {
            retrieval_id: "7001".to_string(),
            hits: vec![fragment("doc-1", "Refund policy", "30 days on refunds")],
            ..Default::default()
        };
        let snippets = map_retrieval_result(result);
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].document_id, "doc-1");
        assert_eq!(snippets[0].source_uri.as_deref(), Some("drive://kb/doc-1"));
        assert_eq!(snippets[0].retrieval_method.as_deref(), Some("hybrid"));
    }

    #[test]
    fn new_rejects_blank_and_non_http_origins() {
        assert!(KnowledgebaseGroundingResolver::new("ftp://knowledgebase").is_err());
        assert!(KnowledgebaseGroundingResolver::new("   ").is_err());
        assert!(KnowledgebaseGroundingResolver::new("https://kb.example.test").is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn resolver_calls_the_app_retrieval_endpoint_with_dual_tokens() {
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
