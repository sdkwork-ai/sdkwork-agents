//! Memory grounding adapter for SDKWork Agents turns.
//!
//! The Agents service owns conversation orchestration; durable user memory stays
//! in `sdkwork-memory`. This crate is the composition-root adapter that turns
//! the service's `MemoryGroundingResolver` port into bounded
//! `memories.list?q=...` calls against the Memory App API using the
//! generator-owned Rust transport SDK.
//!
//! Safety and scope:
//!
//! - The caller's dual tokens are forwarded unchanged; the Memory App API
//!   derives actor, tenant, and owner scope from the credential, so this adapter
//!   never widens access.
//! - Every call runs on a scoped thread with its own current-thread runtime and a
//!   hard deadline, matching the knowledge adapter, so a slow memory backend can
//!   never pin the turn.
//! - The memory slot's `memoryPolicyRef` addresses a Memory space. Accepted forms
//!   are a bare numeric id or `space:<id>`. Invalid refs skip the binding with a
//!   warning instead of failing the turn.
//! - Failures are typed provider errors; the turn degrades to an answer without
//!   user memory and records an operator warning.

use std::time::Duration;

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_intelligence_agents_service::{
    GroundingRequest, MemoryGroundingResolver, MemoryGroundingSnippet, MAX_MEMORY_SNIPPETS,
};
use sdkwork_memory_app_sdk_generated_rust::SdkworkMemoryAppClient;

/// Environment variable holding the Memory App API origin.
pub const MEMORY_BASE_URL_ENV: &str = "SDKWORK_AGENTS_MEMORY_BASE_URL";
/// Default per-turn memory deadline.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);
/// Default retrieval depth when the slot does not set `topK`.
pub const DEFAULT_MEMORY_TOP_K: u32 = 8;
/// Hard ceiling for one memory query.
pub const MAX_MEMORY_TOP_K: u32 = 32;

/// Per-turn Memory grounding resolver.
pub struct MemoryGroundingResolverAdapter {
    base_url: String,
    timeout: Duration,
}

impl MemoryGroundingResolverAdapter {
    /// Builds the resolver from `SDKWORK_AGENTS_MEMORY_BASE_URL`.
    ///
    /// Returns `Ok(None)` when the variable is absent or blank: memory grounding
    /// is an optional capability module, not a boot requirement.
    pub fn from_env() -> Result<Option<Self>, String> {
        let base_url = std::env::var(MEMORY_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        match base_url {
            Some(base_url) => Self::new(base_url).map(Some),
            None => Ok(None),
        }
    }

    /// Builds the resolver for an explicit Memory App API origin.
    pub fn new(base_url: impl Into<String>) -> Result<Self, String> {
        let base_url = base_url.into();
        let trimmed = base_url.trim();
        if trimmed.is_empty() {
            return Err(format!(
                "{MEMORY_BASE_URL_ENV} must not be blank when memory grounding is enabled"
            ));
        }
        if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            return Err(format!(
                "{MEMORY_BASE_URL_ENV} must be an absolute http(s) origin"
            ));
        }
        Ok(Self {
            base_url: trimmed.trim_end_matches('/').to_string(),
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// Overrides the per-turn memory deadline.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }
}

impl MemoryGroundingResolver for MemoryGroundingResolverAdapter {
    fn resolve_memory(
        &self,
        request: &GroundingRequest<'_>,
    ) -> KernelResult<Vec<MemoryGroundingSnippet>> {
        if request.memory.is_empty() {
            return Ok(Vec::new());
        }
        let Some(auth_token) = request.auth_token.filter(|value| !value.trim().is_empty()) else {
            return Err(KernelError::validation(
                "memory grounding requires the caller auth token; refusing to read memory without an authenticated actor",
            ));
        };

        let client = SdkworkMemoryAppClient::new_with_base_url(self.base_url.clone())
            .map_err(|error| provider_error("memory.client_init_failed", error))?;
        client.set_auth_token(auth_token);
        if let Some(access_token) = request
            .access_token
            .filter(|value| !value.trim().is_empty())
        {
            client.set_access_token(access_token);
        }

        let top_k = request
            .memory
            .iter()
            .filter_map(|binding| binding.top_k)
            .max()
            .unwrap_or(DEFAULT_MEMORY_TOP_K)
            .clamp(1, MAX_MEMORY_TOP_K);
        let spaces = request
            .memory
            .iter()
            .filter_map(|binding| {
                parse_memory_space_id(&binding.memory_policy_ref).or_else(|| {
                    tracing::warn!(
                        slot_id = %binding.slot_id,
                        memory_policy_ref = %binding.memory_policy_ref,
                        "memory composition slot does not reference a numeric memory space; skipping"
                    );
                    None
                })
            })
            .collect::<Vec<_>>();
        if spaces.is_empty() {
            return Ok(Vec::new());
        }

        let query = request.query.trim().to_string();
        let actor_id = request.actor_id.map(|value| value.to_string());
        let timeout = self.timeout;
        let snippets = run_sdk_call(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| provider_error("memory.runtime_init_failed", error.to_string()))?;
            runtime.block_on(async {
                let body = sdkwork_memory_app_sdk_generated_rust::models::MemoryRetrievalRequest {
                    query,
                    space_ids: spaces,
                    actor_id,
                    top_k: i64::from(top_k),
                    context_budget_tokens: 1_024,
                    show_expired: Some(false),
                    ..Default::default()
                };
                let idempotency_key = format!(
                    "agents-memory-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|duration| duration.as_nanos())
                        .unwrap_or_default()
                );
                match tokio::time::timeout(
                    timeout,
                    client.memory().retrievals_create(&body, &idempotency_key),
                )
                .await
                {
                    Ok(result) => result
                        .map(map_retrieval_hits)
                        .map_err(|error| provider_error("memory.retrieval_failed", error)),
                    Err(_) => Err(KernelError::provider_error(
                        "memory.retrieval_timeout",
                        format!(
                            "memory query did not complete within {} ms",
                            timeout.as_millis()
                        ),
                    )),
                }
            })
        })?;

        Ok(deduplicate(snippets))
    }
}

fn map_retrieval_hits(
    result: sdkwork_memory_app_sdk_generated_rust::models::MemoryRetrievalResult,
) -> Vec<MemoryGroundingSnippet> {
    result
        .hits
        .into_iter()
        .take(MAX_MEMORY_SNIPPETS)
        .filter_map(|hit| {
            let record = hit.memory?;
            if !record.status.eq_ignore_ascii_case("active") {
                return None;
            }
            let content = record
                .summary_text
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(record.canonical_text);
            if content.trim().is_empty() {
                return None;
            }
            Some(MemoryGroundingSnippet {
                memory_id: record.memory_id,
                content,
                score: hit.fused_score.or(Some(record.confidence)),
                source_uri: None,
            })
        })
        .collect()
}

/// In-process memory grounding resolver for co-deployed Agents processes.
pub struct InProcessMemoryGroundingResolver {
    runtime: std::sync::Arc<sdkwork_memory_inprocess::InProcessMemoryRuntime>,
}

impl InProcessMemoryGroundingResolver {
    /// Bootstraps the process-wide in-process runtime from the environment.
    ///
    /// Returns `Ok(None)` when the memory database is not configured, matching
    /// the HTTP mode's optional-module behaviour.
    pub fn from_env() -> Result<Option<Self>, String> {
        let runtime = std::thread::scope(|scope| {
            let handle = std::thread::Builder::new()
                .name("agents-memory-inprocess-bootstrap".to_string())
                .spawn_scoped(scope, || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|error| format!("memory runtime init failed: {error}"))?;
                    runtime.block_on(
                        sdkwork_memory_inprocess::InProcessMemoryRuntime::shared_from_env(),
                    )
                })
                .map_err(|error| format!("memory bootstrap worker spawn failed: {error}"))?;
            handle
                .join()
                .map_err(|_| "memory bootstrap worker panicked".to_string())?
        })?;
        Ok(runtime.map(Self::new))
    }

    /// Builds the resolver over an initialized runtime.
    #[must_use]
    pub fn new(runtime: std::sync::Arc<sdkwork_memory_inprocess::InProcessMemoryRuntime>) -> Self {
        Self { runtime }
    }
}

impl MemoryGroundingResolver for InProcessMemoryGroundingResolver {
    fn resolve_memory(
        &self,
        request: &GroundingRequest<'_>,
    ) -> KernelResult<Vec<MemoryGroundingSnippet>> {
        if request.memory.is_empty() {
            return Ok(Vec::new());
        }
        let spaces = request
            .memory
            .iter()
            .filter_map(|binding| {
                parse_memory_space_id(&binding.memory_policy_ref)
                    .and_then(|value| value.parse::<u64>().ok())
                    .or_else(|| {
                        tracing::warn!(
                            slot_id = %binding.slot_id,
                            memory_policy_ref = %binding.memory_policy_ref,
                            "memory composition slot does not reference a numeric memory space; skipping"
                        );
                        None
                    })
            })
            .collect::<Vec<_>>();
        if spaces.is_empty() {
            return Ok(Vec::new());
        }
        let top_k = request
            .memory
            .iter()
            .filter_map(|binding| binding.top_k)
            .max()
            .unwrap_or(DEFAULT_MEMORY_TOP_K)
            .clamp(1, MAX_MEMORY_TOP_K);
        let query = request.query.trim().to_string();
        let runtime = &self.runtime;
        let tenant_id = request.tenant_id;
        let Some(actor_id) = request.actor_id else {
            tracing::warn!(
                agent_id = %request.agent_id,
                "in-process memory grounding requires an authenticated actor; skipping retrieval"
            );
            return Ok(Vec::new());
        };
        let mut snippets = Vec::new();
        for space_id in spaces {
            if snippets.len() >= MAX_MEMORY_SNIPPETS {
                break;
            }
            let result = run_sdk_call(|| {
                let runtime_handle = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| provider_error("memory.runtime_init_failed", error))?;
                runtime_handle
                    .block_on(runtime.retrieve(
                        tenant_id,
                        Some(actor_id),
                        space_id,
                        &query,
                        Some(top_k),
                        Some(DEFAULT_MEMORY_CONTEXT_BUDGET),
                    ))
                    .map_err(|error| {
                        provider_error("memory.retrieval_failed", format!("{error:?}"))
                    })
            })?;
            for hit in result.hits {
                if snippets.len() >= MAX_MEMORY_SNIPPETS {
                    break;
                }
                let Some(record) = hit.memory else {
                    continue;
                };
                let content = record
                    .summary_text
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(record.canonical_text);
                if content.trim().is_empty() {
                    continue;
                }
                snippets.push(MemoryGroundingSnippet {
                    memory_id: record.memory_id.to_string(),
                    content,
                    score: hit.fused_score.or(Some(record.confidence)),
                    source_uri: None,
                });
            }
        }
        Ok(deduplicate(snippets))
    }
}

const DEFAULT_MEMORY_CONTEXT_BUDGET: u32 = 1_024;

/// Parses a memory slot reference into a Memory space id.
///
/// Accepted forms: `4242` and `space:4242`. Any other form returns `None`; the
/// caller logs and skips the slot.
#[must_use]
pub fn parse_memory_space_id(reference: &str) -> Option<String> {
    let trimmed = reference.trim();
    let numeric = trimmed.strip_prefix("space:").unwrap_or(trimmed).trim();
    if numeric.is_empty() || !numeric.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    Some(numeric.to_string())
}

/// Maps one Memory retrieval result into bounded grounding snippets.
#[must_use]
pub fn map_memory_retrieval_result(
    result: sdkwork_memory_app_sdk_generated_rust::models::MemoryRetrievalResult,
) -> Vec<MemoryGroundingSnippet> {
    map_retrieval_hits(result)
}

fn deduplicate(snippets: Vec<MemoryGroundingSnippet>) -> Vec<MemoryGroundingSnippet> {
    let mut seen = std::collections::HashSet::new();
    snippets
        .into_iter()
        .filter(|snippet| seen.insert(snippet.memory_id.clone()))
        .take(MAX_MEMORY_SNIPPETS)
        .collect()
}

fn run_sdk_call<T>(work: impl FnOnce() -> KernelResult<T> + Send) -> KernelResult<T>
where
    T: Send,
{
    std::thread::scope(|scope| {
        let handle = std::thread::Builder::new()
            .name("agents-memory-grounding".to_string())
            .spawn_scoped(scope, work)
            .map_err(|error| provider_error("memory.worker_spawn_failed", error.to_string()))?;
        handle.join().map_err(|_| {
            KernelError::provider_error(
                "memory.worker_panicked",
                "memory grounding worker panicked",
            )
        })?
    })
}

fn provider_error(code: &str, detail: impl ToString) -> KernelError {
    KernelError::provider_error(code, detail.to_string())
        .with_provider("provider.memory.sdkwork-memory")
        .from_source(sdkwork_agent_kernel::KernelErrorSource::Provider)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_intelligence_agents_service::MemoryGroundingBinding;

    fn binding(reference: &str, top_k: Option<u32>) -> MemoryGroundingBinding {
        MemoryGroundingBinding {
            slot_id: "slot.memory.test".to_string(),
            memory_policy_ref: reference.to_string(),
            top_k,
        }
    }

    fn request<'a>(memory: &'a [MemoryGroundingBinding]) -> GroundingRequest<'a> {
        GroundingRequest {
            tenant_id: 100_001,
            organization_id: 0,
            actor_id: Some(7),
            agent_id: "agent.chat.default",
            session_id: "session.1",
            query: "  how do refunds work  ",
            knowledge: &[],
            memory,
            auth_token: Some("auth-token"),
            access_token: Some("access-token"),
        }
    }

    #[test]
    fn space_references_accept_numeric_and_space_prefix() {
        assert_eq!(parse_memory_space_id("4242").as_deref(), Some("4242"));
        assert_eq!(
            parse_memory_space_id(" space:4242 ").as_deref(),
            Some("4242")
        );
        assert!(parse_memory_space_id("memory.space.42").is_none());
        assert!(parse_memory_space_id("space:").is_none());
        assert!(parse_memory_space_id("-1").is_none());
    }

    #[test]
    fn records_map_to_bounded_snippets_and_skip_inactive() {
        use sdkwork_memory_app_sdk_generated_rust::models::{
            MemoryRecord, MemoryRetrievalHit, MemoryRetrievalResult,
        };
        let record = |memory_id: &str, text: &str, status: &str| MemoryRecord {
            memory_id: memory_id.to_string(),
            canonical_text: text.to_string(),
            confidence: 0.9,
            status: status.to_string(),
            ..Default::default()
        };
        let result = MemoryRetrievalResult {
            retrieval_id: "1".to_string(),
            hits: vec![
                MemoryRetrievalHit {
                    hit_id: "h1".to_string(),
                    memory: Some(record("7001", "Prefers refunds within 30 days.", "active")),
                    retriever_name: "hybrid".to_string(),
                    result_rank: 1,
                    fused_score: Some(0.93),
                    status: "active".to_string(),
                    ..Default::default()
                },
                MemoryRetrievalHit {
                    hit_id: "h2".to_string(),
                    memory: Some(record("7002", "Deleted memory", "deleted")),
                    retriever_name: "hybrid".to_string(),
                    result_rank: 2,
                    status: "deleted".to_string(),
                    ..Default::default()
                },
            ],
            degraded: false,
            ..Default::default()
        };
        let snippets = map_memory_retrieval_result(result);
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].memory_id, "7001");
        assert_eq!(snippets[0].content, "Prefers refunds within 30 days.");
    }

    #[test]
    fn new_rejects_blank_and_non_http_origins() {
        assert!(MemoryGroundingResolverAdapter::new("ftp://memory").is_err());
        assert!(MemoryGroundingResolverAdapter::new("   ").is_err());
        assert!(MemoryGroundingResolverAdapter::new("https://memory.example.test").is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn resolver_calls_retrievals_create_with_dual_tokens() {
        use serde_json::json;
        use wiremock::matchers::{body_partial_json, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let body = json!({
            "code": 0,
            "data": {
                "retrievalId": "1",
                "hits": [{
                    "hitId": "h1",
                    "memory": {
                        "memoryId": "7001",
                        "spaceId": "4242",
                        "userId": null,
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "Prefers refunds within 30 days.",
                        "summaryText": null,
                        "confidence": 0.9,
                        "evidenceCount": 1,
                        "contradictionCount": 0,
                        "expiresAt": null,
                        "status": "active",
                        "sensitivityLevel": "normal",
                        "metadata": null,
                        "supersedesMemoryId": null,
                        "supersededByMemoryId": null,
                        "createdAt": "2026-10-01T00:00:00Z",
                        "updatedAt": "2026-10-01T00:00:00Z",
                        "version": "1"
                    },
                    "retrieverName": "hybrid",
                    "resultRank": 1,
                    "fusedScore": 0.93,
                    "status": "active"
                }],
                "degraded": false
            },
            "traceId": "00000000-0000-4000-8000-000000000000"
        });
        Mock::given(method("POST"))
            .and(path("/app/v3/api/memory/retrievals"))
            .and(body_partial_json(json!({
                "query": "how do refunds work",
                "spaceIds": ["4242"],
                "actorId": "7",
                "topK": 4
            })))
            .and(header("Authorization", "Bearer auth-token"))
            .and(header("Access-Token", "access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let resolver = MemoryGroundingResolverAdapter::new(server.uri()).expect("memory resolver");
        let snippets = tokio::task::spawn_blocking(move || {
            let bindings = [binding("space:4242", Some(4))];
            resolver.resolve_memory(&request(&bindings))
        })
        .await
        .expect("grounding task")
        .expect("memory snippets");

        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].memory_id, "7001");
        assert!(snippets[0].content.contains("30 days"));
    }
}
