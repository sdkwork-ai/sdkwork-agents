//! Knowledgebase grounding adapter for SDKWork Agents turns.
//!
//! The Agents service owns conversation orchestration; knowledge content and
//! indexes stay in `sdkwork-knowledgebase`. This crate is the composition-root
//! adapter that turns the service's `KnowledgeGroundingResolver` port into one
//! bounded retrieval call, and it models the transport exactly as
//! `MODULE_SPEC.md` §4.2 requires:
//!
//! - **Default: in-process** ([`InProcessKnowledgebaseGroundingResolver`]).
//!   The Knowledgebase retrieval capability is addressed inside this process
//!   over the host database, with no sibling HTTP endpoint. The Knowledgebase
//!   app-api retrieval port is `pub(crate)` today, so this resolver fails closed
//!   with a typed provider error and [`in_process::REQUIRED_SIBLING_API`]
//!   records the additive sibling API that unblocks it. Rebuilding
//!   Knowledgebase's RLS-scoped runtime here — a second pool plus its drive
//!   storage — is not a narrow in-process port and is deliberately not done.
//! - **Opt-in split deployment: HTTP** (`SDKWORK_AGENTS_KNOWLEDGEBASE_MODE=http`
//!   plus [`KNOWLEDGEBASE_BASE_URL_ENV`]). The origin is then mandatory:
//!   bootstrap fails when it is missing, blank, or not an http(s) origin —
//!   never at the first business request. The transport lives in
//!   [`http_adapter`] and is the only place that reads the origin variable.
//!
//! Both transports share one request builder and one result mapping, so the
//! grounding contract (`hybrid` methods, citation capture, binding order, the
//! service's snippet ceiling) cannot drift between them.

use std::time::Duration;

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_intelligence_agents_service::{
    GroundingRequest, KnowledgeGroundingSnippet, MAX_KNOWLEDGE_SNIPPETS,
};
use sdkwork_knowledgebase_app_sdk_generated_rust::models::{
    KnowledgeRetrievalBinding, KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
};

pub mod http_adapter;
pub mod in_process;

pub use http_adapter::{KnowledgebaseGroundingResolver, KNOWLEDGEBASE_BASE_URL_ENV};
pub use in_process::{InProcessKnowledgebaseGroundingResolver, REQUIRED_SIBLING_API};

/// Environment variable selecting the Knowledgebase transport.
///
/// `in_process` (default) or `http` (explicit split-deployment opt-in).
pub const KNOWLEDGEBASE_MODE_ENV: &str = "SDKWORK_AGENTS_KNOWLEDGEBASE_MODE";
/// In-process tool call transport (default).
pub const MODE_IN_PROCESS: &str = "in_process";
/// Sibling HTTP surface, allowed only as an explicit opt-in.
pub const MODE_HTTP: &str = "http";
/// Default per-turn retrieval deadline.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);
/// Retrieval methods requested from the Knowledgebase hybrid profile.
pub const RETRIEVAL_METHODS: [&str; 1] = ["hybrid"];

/// Maps Agents grounding bindings to the Knowledgebase retrieval contract.
///
/// The binding order is preserved (highest-priority slot first as resolved by
/// the service). `includeCitations` is always on so the renderer can carry a
/// stable source URI; actor and tenant are never sent - the App API derives
/// both from the forwarded credential and the in-process port derives them from
/// its request context.
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

/// Runs one blocking transport call on a scoped worker thread.
pub(crate) fn run_sdk_call<T>(work: impl FnOnce() -> KernelResult<T> + Send) -> KernelResult<T>
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

/// Builds the typed provider error shared by both transports.
pub(crate) fn provider_error(code: &str, detail: impl ToString) -> KernelError {
    KernelError::provider_error(code, detail.to_string())
        .with_provider("provider.knowledge.sdkwork-knowledgebase")
        .from_source(sdkwork_agent_kernel::KernelErrorSource::Provider)
}

#[cfg(test)]
pub(crate) mod tests_support {
    use sdkwork_intelligence_agents_service::{GroundingRequest, KnowledgeGroundingBinding};

    pub(crate) fn binding(space_id: u64, top_k: Option<u32>) -> KnowledgeGroundingBinding {
        KnowledgeGroundingBinding {
            slot_id: format!("slot.knowledge.{space_id}"),
            space_id,
            top_k,
            min_score: Some(0.2),
            title: None,
        }
    }

    pub(crate) fn request<'a>(knowledge: &'a [KnowledgeGroundingBinding]) -> GroundingRequest<'a> {
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
}

#[cfg(test)]
mod tests {
    use super::tests_support::{binding, request};
    use super::*;
    use sdkwork_intelligence_agents_service::KnowledgeGroundingResolver;
    use sdkwork_knowledgebase_app_sdk_generated_rust::models::{
        KnowledgeCitation, KnowledgeContextFragment,
    };

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
    fn in_process_resolver_needs_no_configuration_and_skips_empty_slots() {
        let resolver = InProcessKnowledgebaseGroundingResolver::from_env()
            .expect("in-process bootstrap")
            .expect("in-process resolver is always mounted");
        let bindings: [sdkwork_intelligence_agents_service::KnowledgeGroundingBinding; 0] = [];
        let snippets = resolver
            .resolve_knowledge(&request(&bindings))
            .expect("no knowledge slots is not an error");
        assert!(snippets.is_empty());
    }

    #[test]
    fn in_process_resolver_fails_closed_with_the_missing_port() {
        let resolver = InProcessKnowledgebaseGroundingResolver::new();
        let bindings = [binding(4242, Some(4))];
        let error = resolver
            .resolve_knowledge(&request(&bindings))
            .expect_err("blocked in-process port must fail closed");
        let detail = error.to_string();
        assert!(
            detail.contains("KnowledgebaseRuntime::retrieval_app_service"),
            "missing-port error must name the required sibling API: {detail}"
        );
        assert!(detail.contains("sdkwork-knowledgebase"));
    }
}
