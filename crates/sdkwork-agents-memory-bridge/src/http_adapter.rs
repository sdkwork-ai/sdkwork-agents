//! Explicit split-deployment transport for Memory grounding
//! (MODULE_SPEC.md §4.2.3).
//!
//! This module is the **only** place that reads [`MEMORY_BASE_URL_ENV`]; it is
//! reached only when the deployment sets `SDKWORK_AGENTS_MEMORY_MODE=http`, and
//! the origin is then required — [`MemoryGroundingResolverAdapter::from_env`]
//! fails bootstrap instead of degrading at the first turn. The default mode is
//! in-process ([`crate::InProcessMemoryGroundingResolver`]).
//!
//! The caller's dual tokens are forwarded unchanged; the Memory App API derives
//! actor, tenant, and owner scope from the credential, so this adapter never
//! widens access. Every call runs on a scoped thread with its own
//! current-thread runtime and a hard deadline, matching the knowledge adapter,
//! so a slow memory backend can never pin the turn.

use std::time::Duration;

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_intelligence_agents_service::{
    GroundingRequest, MemoryGroundingResolver, MemoryGroundingSnippet,
};
use sdkwork_memory_app_sdk_generated_rust::SdkworkMemoryAppClient;
use sdkwork_utils_rust::service_base_url::{registry_base_url_or, report_registry_result};

use crate::{
    map_memory_retrieval_result, parse_memory_space_id, provider_error, run_sdk_call,
    DEFAULT_MEMORY_TOP_K, DEFAULT_TIMEOUT, MAX_MEMORY_TOP_K,
};

/// Environment variable holding the Memory App API origin.
///
/// Read only by the explicit HTTP opt-in (`SDKWORK_AGENTS_MEMORY_MODE=http`).
pub const MEMORY_BASE_URL_ENV: &str = "SDKWORK_AGENTS_MEMORY_BASE_URL";

/// Memory App API transport (split-deployment opt-in only).
pub struct MemoryGroundingResolverAdapter {
    base_url: String,
    timeout: Duration,
}

impl MemoryGroundingResolverAdapter {
    /// Builds the transport from `SDKWORK_AGENTS_MEMORY_BASE_URL`.
    ///
    /// This adapter is only reachable behind the explicit HTTP opt-in, so a
    /// missing, blank, or non-http origin is a deployment misconfiguration: it
    /// fails bootstrap rather than silently disabling retrieval after the host
    /// has started serving (MODULE_SPEC.md §4.2.3).
    pub fn from_env() -> Result<Self, String> {
        Self::new(required_origin(
            std::env::var(MEMORY_BASE_URL_ENV).ok().as_deref(),
        )?)
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

/// Requires the opt-in origin at bootstrap.
fn required_origin(value: Option<&str>) -> Result<String, String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "{MEMORY_BASE_URL_ENV} is required when {}={}",
                crate::MEMORY_MODE_ENV,
                crate::MODE_HTTP
            )
        })
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

        // DISCOVERY_SPEC §12.1: resolve through the database registry first and
        // report the call outcome so a failing instance is demoted.
        let base_url = registry_base_url_or("sdkwork-memory", &self.base_url);
        let client = SdkworkMemoryAppClient::new_with_base_url(base_url.clone())
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
        let result = run_sdk_call(move || {
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
                        .map(map_memory_retrieval_result)
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
        });
        report_registry_result(
            "sdkwork-memory",
            &base_url,
            result.is_ok(),
            "memory retrieval failed",
        );
        let snippets = result?;

        Ok(crate::deduplicate(snippets))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_blank_and_non_http_origins() {
        assert!(MemoryGroundingResolverAdapter::new("ftp://memory").is_err());
        assert!(MemoryGroundingResolverAdapter::new("   ").is_err());
        assert!(MemoryGroundingResolverAdapter::new("https://memory.example.test").is_ok());
    }

    #[test]
    fn http_opt_in_requires_an_origin_at_bootstrap() {
        let error = required_origin(None).expect_err("missing origin must fail bootstrap");
        assert!(error.contains("SDKWORK_AGENTS_MEMORY_BASE_URL"));
        assert!(error.contains("SDKWORK_AGENTS_MEMORY_MODE"));
        assert!(required_origin(Some("   ")).is_err());
        assert_eq!(
            required_origin(Some(" https://memory.example.test ")).expect("origin"),
            "https://memory.example.test"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn resolver_calls_retrievals_create_with_dual_tokens() {
        use crate::tests_support::{binding, request};
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
