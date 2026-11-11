//! In-process Knowledgebase grounding (MODULE_SPEC.md §4.2, default mode).
//!
//! # Why this is blocked
//!
//! The Knowledgebase retrieval capability agents need is the App API
//! `POST /app/v3/api/knowledge/retrievals` behaviour: tenant/organization scope
//! check, a `reader` ACL check per binding, the tenant retrieval rate limit, and
//! a persisted retrieval trace. That behaviour lives in
//! `sdkwork_knowledgebase`'s app-api route crate only:
//!
//! - `HostedRetrievalService` (the only `KnowledgeRetrievalAppService` impl) is
//!   `pub(crate)` in `sdkwork-routes-knowledgebase-app-api`;
//! - the runtime accessors it composes (`retrieval_store`, `space_store`,
//!   `access_control`, `retrieval_service`) are `pub(crate)` too;
//! - the public persistence stores that could rebuild it are not usable from a
//!   non-Knowledgebase host: `PostgresKnowledgeSpaceStore` /
//!   `PostgresGroupKnowledgeSpaceBindingStore` take a `sqlx::AnyPool` whose
//!   connections must carry Knowledgebase's deployment scope
//!   (`app.current_tenant_id` / `app.current_organization_id`) because the
//!   `kb_*` tables use `FORCE ROW LEVEL SECURITY`; a pool opened by the Agents
//!   process without that scope silently reads zero rows;
//! - `PostgresKnowledgeChunkRetrievalStore` is `PgPool`-backed but performs no
//!   ACL check at all.
//!
//! Rebuilding the app-api runtime (second RLS-scoped pool, drive storage for the
//! OKF initializer, group-space authorizer ordering) inside this adapter would
//! be a second Knowledgebase runtime composition, not a narrow in-process port
//! (MODULE_SPEC.md §4.2.7). The additive sibling API this adapter needs is
//! recorded in [`REQUIRED_SIBLING_API`].
//!
//! Until that API exists this resolver fails closed with a typed provider error:
//! the turn degrades to an ungrounded answer exactly like an unreachable
//! Knowledgebase App API, and the operator sees the missing port instead of a
//! silent capability loss. The HTTP transport remains available as an explicit
//! opt-in (`SDKWORK_AGENTS_KNOWLEDGEBASE_MODE=http` + origin) and fails at
//! startup when its origin is missing.
//!
//! Agents code never reads a Knowledgebase table or SQL statement: no SQL is
//! added here (MODULE_SPEC.md §4.2.3).

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_intelligence_agents_service::{
    GroundingRequest, KnowledgeGroundingResolver, KnowledgeGroundingSnippet,
};

use crate::provider_error;

/// The additive `sdkwork-knowledgebase` API this adapter needs.
///
/// Either signature unblocks the migration; both must be reachable from a
/// co-deployed module without rebuilding Knowledgebase's runtime:
///
/// ```ignore
/// // sdkwork-knowledgebase/crates/sdkwork-routes-knowledgebase-app-api/src/runtime.rs
/// impl KnowledgebaseRuntime {
///     /// `POST /app/v3/api/knowledge/retrievals` as a callable port: tenant and
///     /// organization scope check, per-binding `reader` ACL, tenant retrieval
///     /// rate limit, persisted trace, identical `ApiError` mapping.
///     pub fn retrieval_app_service(self: &std::sync::Arc<Self>)
///         -> std::sync::Arc<dyn KnowledgeRetrievalAppService>;
///
///     /// Builds the runtime over pools the host already owns (no second pool,
///     /// no schema install) with Knowledgebase's deployment RLS scope applied.
///     pub async fn from_host_pools(
///         pool: sqlx::PgPool,
///         knowledgebase_pool: sqlx::AnyPool,
///         tenant_id: u64,
///         organization_id: u64,
///     ) -> Result<Self, sqlx::Error>;
/// }
/// ```
pub const REQUIRED_SIBLING_API: &str = "sdkwork-knowledgebase: add `KnowledgebaseRuntime::retrieval_app_service(&Arc<Self>) -> Arc<dyn KnowledgeRetrievalAppService>` and a host-pool constructor `KnowledgebaseRuntime::from_host_pools(PgPool, AnyPool, tenant_id, organization_id)`; the App API retrieval port is `pub(crate)` today";

/// In-process Knowledgebase grounding resolver.
///
/// Fails closed until Knowledgebase exposes the callable retrieval port (see
/// [`REQUIRED_SIBLING_API`]); the failure is a typed provider error so a turn
/// degrades exactly like an unreachable Knowledgebase App API.
#[derive(Debug, Default, Clone, Copy)]
pub struct InProcessKnowledgebaseGroundingResolver;

impl InProcessKnowledgebaseGroundingResolver {
    /// Builds the resolver.
    ///
    /// In-process grounding needs no deployment configuration of its own: the
    /// Knowledgebase data lives in the host database and the port is expected to
    /// take the host pools. Nothing is read from the environment.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Builds the resolver for the default in-process mode.
    ///
    /// Always available: the capability is mounted whenever the deployment runs
    /// the Agents backend. Returns `Ok(None)` only when a future port requires
    /// optional Knowledgebase configuration that is absent.
    pub fn from_env() -> Result<Option<Self>, String> {
        Ok(Some(Self::new()))
    }
}

impl KnowledgeGroundingResolver for InProcessKnowledgebaseGroundingResolver {
    fn resolve_knowledge(
        &self,
        request: &GroundingRequest<'_>,
    ) -> KernelResult<Vec<KnowledgeGroundingSnippet>> {
        if request.knowledge.is_empty() {
            return Ok(Vec::new());
        }
        Err(missing_port_error())
    }
}

/// Builds the typed error for the missing Knowledgebase in-process port.
#[must_use]
pub fn missing_port_error() -> KernelError {
    provider_error(
        "knowledgebase.in_process_port_unavailable",
        REQUIRED_SIBLING_API,
    )
}
