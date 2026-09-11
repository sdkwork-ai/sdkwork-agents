use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use sdkwork_agent_kernel::{
    AgentMessage, AgentMessageRole, AgentPart, AgentPartKind, KernelError, KernelErrorKind,
    KernelResult, PolicySubject, SessionKind,
};
use sdkwork_agents_runtime_facade::{
    AgentsSessionActor, AgentsSessionEntrySurface, AgentsSessionFacade, AgentsSessionKind,
    AgentsSessionRuntimeBindingDescriptor, ProviderSessionInventoryItem,
    ProviderSessionInventorySelector, ProviderSessionInventorySnapshot,
    ResolveAgentsSessionRequest, RuntimeFacadeError,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::application::{
    ArchiveSessionCommand, CloseSessionCommand, CreateSessionCommand,
    CreateSessionRuntimeBindingCommand, GetProjectCommand, GetSessionCommand,
    ListSessionItemsCommand, ListSessionRuntimeBindingsCommand, ListSessionsCommand,
    ReconcileProviderSessionHistoryItemCommand,
};
use crate::domain::{
    AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionItemStatus, AgentSessionKind,
    AgentSessionStatus,
};
use crate::http::{HttpAgentsSessionFacade, HttpService};
use crate::list_cursors::{decode_session_list_cursor, SessionListCursor};
use crate::ports::{
    PaginationParams, SessionItemListQuery, SessionListQuery, SessionRuntimeBindingListQuery,
};
use crate::project::AgentProjectRecord;
use crate::runtime_facade_bridge::shared_agent_engine_host;
use crate::session_id_scheme::{
    canonical_provider_item_id, canonical_provider_item_id_prefix,
    canonical_provider_runtime_binding_id, canonical_provider_session_id,
    is_provider_session_id_for,
};

const PROVIDER_SESSION_TITLE_MAX_BYTES: usize = 512;
const PROVIDER_SESSION_RECONCILIATION_MAX_ITEMS: usize = 10_000;
const PROVIDER_SESSION_RECONCILIATION_TIMEOUT: Duration = Duration::from_secs(15);
/**
 * How long a completed provider Session inventory synchronization outcome
 * stays reusable in the process cache.
 *
 * The refresh window covers the most expensive part of a repeat
 * synchronization: the provider inventory discovery scan. Codex and Claude
 * Code provider Sessions are JSONL files on disk, and every discovery walks
 * the provider store (the SDKs expose no index API; Codex's private state
 * database is deliberately never inspected). A synchronization inside this
 * window is served from the cached outcome without re-discovering the
 * provider inventory at all. The window must cover the client-side 60s
 * deduplication TTL so the background inbox synchronization loop never
 * re-scans the provider store in steady state; imported Session freshness
 * therefore lags by at most one client cycle, which the activity feed
 * already tolerates.
 *
 * After the window a cold synchronization always re-discovers, and the
 * per-Session reconcile converges provider metadata. The two full inventory
 * sweeps (orphan and missing-directory reconciliation — their outcome is a
 * pure function of the discovered Session identity set) are skipped whenever
 * the set is unchanged, so a cold synchronization whose set did not move
 * costs only the scan.
 */
pub(crate) const PROVIDER_SESSION_SYNC_REFRESH_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderSessionSynchronizationIssueDisposition {
    Skipped,
    Failed,
}

impl ProviderSessionSynchronizationIssueDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderSessionSynchronizationIssue {
    pub(crate) code: &'static str,
    pub(crate) count: usize,
    pub(crate) disposition: ProviderSessionSynchronizationIssueDisposition,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ProviderSessionSynchronizationResult {
    pub(crate) failed_session_count: usize,
    pub(crate) issues: Vec<ProviderSessionSynchronizationIssue>,
    pub(crate) skipped_session_count: usize,
    pub(crate) synchronized_session_count: usize,
}

/// Reports whether one Session Item synchronization window actually imported
/// provider transcript history, or why it could not. The persisted item
/// window remains authoritative in every case, so callers never treat a
/// skipped synchronization as an empty transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderSessionTranscriptSyncOutcome {
    /// The provider transcript was loaded and reconciled; the count can be
    /// zero when the provider Session has no importable messages.
    Imported { imported_item_count: usize },
    /// The requested Session is not a provider-history Session; no provider
    /// transcript synchronization applies to it.
    NotProviderSession,
    /// The Session has no active current provider-history runtime binding,
    /// so its provider identity cannot be resolved. This is the signature of
    /// an orphaned or not-yet-bound provider Session.
    NoActiveBinding,
    /// The provider agent engine could not load the transcript (unavailable
    /// engine or transient read failure); the persisted window is returned.
    EngineUnavailable,
}

impl ProviderSessionTranscriptSyncOutcome {
    pub(crate) const fn status_code(self) -> &'static str {
        match self {
            Self::Imported { .. } => "imported",
            Self::NotProviderSession => "not-provider-session",
            Self::NoActiveBinding => "no-active-binding",
            Self::EngineUnavailable => "engine-unavailable",
        }
    }

    pub(crate) const fn imported_item_count(self) -> usize {
        match self {
            Self::Imported {
                imported_item_count,
            } => imported_item_count,
            Self::NotProviderSession | Self::NoActiveBinding | Self::EngineUnavailable => 0,
        }
    }
}

impl ProviderSessionSynchronizationResult {
    fn record_issue(
        &mut self,
        code: &'static str,
        disposition: ProviderSessionSynchronizationIssueDisposition,
        count: usize,
    ) {
        if count == 0 {
            return;
        }
        if let Some(issue) = self
            .issues
            .iter_mut()
            .find(|issue| issue.code == code && issue.disposition == disposition)
        {
            issue.count += count;
        } else {
            self.issues.push(ProviderSessionSynchronizationIssue {
                code,
                count,
                disposition,
            });
        }
        match disposition {
            ProviderSessionSynchronizationIssueDisposition::Skipped => {
                self.skipped_session_count += count;
            }
            ProviderSessionSynchronizationIssueDisposition::Failed => {
                self.failed_session_count += count;
            }
        }
    }

    fn record_skipped(&mut self, code: &'static str) {
        self.record_issue(
            code,
            ProviderSessionSynchronizationIssueDisposition::Skipped,
            1,
        );
    }

    fn record_failed(&mut self, code: &'static str) {
        self.record_issue(
            code,
            ProviderSessionSynchronizationIssueDisposition::Failed,
            1,
        );
    }
}

/// The process-local outcome of one completed provider Session inventory
/// synchronization, keyed by owner-and-project scope. It makes repeat
/// synchronizations incremental: a synchronization inside the refresh window
/// returns the stored outcome without re-discovering the provider inventory,
/// and a cold synchronization whose discovered Session identity set is
/// unchanged skips the two full inventory sweeps (the fingerprint recorded
/// here is compared inside `synchronize_provider_session_snapshot`).
#[derive(Debug, Clone)]
pub(crate) struct CompletedProviderSessionSync {
    pub(crate) fingerprint: String,
    pub(crate) result: ProviderSessionSynchronizationResult,
    pub(crate) completed_at: Instant,
}

static COMPLETED_PROVIDER_SESSION_SYNCS: OnceLock<
    Mutex<HashMap<String, CompletedProviderSessionSync>>,
> = OnceLock::new();

fn completed_provider_session_syncs(
) -> &'static Mutex<HashMap<String, CompletedProviderSessionSync>> {
    COMPLETED_PROVIDER_SESSION_SYNCS.get_or_init(Default::default)
}

pub(crate) fn provider_session_sync_cache_key(project: &AgentProjectRecord) -> String {
    format!(
        "{}/{}/{}:{}",
        project.tenant_id, project.organization_id, project.owner_user_id, project.project_id
    )
}

pub(crate) fn read_completed_provider_session_sync(
    cache_key: &str,
) -> Option<CompletedProviderSessionSync> {
    completed_provider_session_syncs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(cache_key)
        .cloned()
}

/// Maximum number of completed provider Session synchronization outcomes kept
/// in the process cache. The cache is a pure performance optimization for
/// repeat refreshes, so evicting an arbitrary entry on overflow only costs
/// one cold synchronization for that scope.
const PROVIDER_SESSION_SYNC_MAX_CACHED_PROJECTS: usize = 1_024;

fn record_completed_provider_session_sync(
    cache_key: &str,
    completed: CompletedProviderSessionSync,
) {
    let mut cache = completed_provider_session_syncs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if cache.len() >= PROVIDER_SESSION_SYNC_MAX_CACHED_PROJECTS && !cache.contains_key(cache_key) {
        if let Some(evicted_key) = cache.keys().next().cloned() {
            cache.remove(&evicted_key);
        }
    }
    cache.insert(cache_key.to_string(), completed);
}

static IN_FLIGHT_PROJECT_SYNCS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn in_flight_provider_session_syncs() -> &'static Mutex<HashSet<String>> {
    IN_FLIGHT_PROJECT_SYNCS.get_or_init(Default::default)
}

/// Marks one project's synchronization as in-flight. Returns `false` when a
/// synchronization is already running for the same project scope, so a burst
/// of concurrent requests never duplicates the discovery scan or the two
/// full inventory sweeps.
pub(crate) fn mark_provider_session_sync_in_flight(cache_key: &str) -> bool {
    in_flight_provider_session_syncs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(cache_key.to_string())
}

/// Clears the in-flight marker after a background synchronization settles
/// (success or failure); the next request then starts a fresh attempt.
pub(crate) fn clear_provider_session_sync_in_flight(cache_key: &str) {
    in_flight_provider_session_syncs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(cache_key);
}

/// RAII guard that clears the in-flight marker on drop — including unwinds.
///
/// The background synchronization may panic (a worker bug must never wedge a
/// project into a permanent `202 pending`); holding this guard guarantees the
/// marker is released on every exit path.
pub(crate) struct ProviderSessionSyncGuard {
    cache_key: String,
}

impl ProviderSessionSyncGuard {
    pub(crate) fn new(cache_key: impl Into<String>) -> Self {
        Self {
            cache_key: cache_key.into(),
        }
    }
}

impl Drop for ProviderSessionSyncGuard {
    fn drop(&mut self) {
        clear_provider_session_sync_in_flight(&self.cache_key);
    }
}

#[cfg(test)]
pub(crate) fn reset_provider_session_sync_cache_for_testing() {
    completed_provider_session_syncs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

/// Fingerprints the discovered provider Session identity set
/// (engine, binding, provider, provider Session id). The set determines the
/// outcome of both full inventory sweeps, so an unchanged fingerprint proves
/// their results cannot have changed either.
fn provider_session_inventory_fingerprint(items: &[ProviderSessionInventoryItem]) -> String {
    let mut identities = items
        .iter()
        .map(|item| {
            (
                item.engine_key.trim().to_string(),
                item.binding_id.trim().to_string(),
                item.provider_id.trim().to_string(),
                item.session.session_id.trim().to_string(),
            )
        })
        .collect::<Vec<_>>();
    identities.sort();
    let mut manifest = String::from("sdkwork.provider-session-inventory.v1\n");
    for (engine_key, binding_id, provider_id, session_id) in identities {
        manifest.push_str(&engine_key);
        manifest.push('\0');
        manifest.push_str(&binding_id);
        manifest.push('\0');
        manifest.push_str(&provider_id);
        manifest.push('\0');
        manifest.push_str(&session_id);
        manifest.push('\n');
    }
    format!(
        "sha256:{}",
        sdkwork_utils_rust::sha256_hash(manifest.as_bytes())
    )
}

pub(crate) fn synchronize_project_provider_sessions(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    let exact_cwd = std::env::current_dir()
        .ok()
        .filter(|cwd| {
            cwd.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|basename| basename.eq_ignore_ascii_case(&project.name))
        })
        .map(|cwd| cwd.to_string_lossy().into_owned());
    synchronize_project_provider_sessions_with_selector(
        service,
        project,
        subject,
        exact_cwd,
        Some(project.name.clone()),
        None,
    )
}

pub(crate) fn synchronize_project_provider_sessions_at_cwd(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    exact_cwd: Option<String>,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    synchronize_project_provider_sessions_with_selector(
        service,
        project,
        subject,
        exact_cwd,
        Some(project.name.clone()),
        None,
    )
}

pub(crate) fn synchronize_project_provider_sessions_with_selector(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    exact_cwd: Option<String>,
    unique_basename: Option<String>,
    directory_fingerprint: Option<String>,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    let cache_key = provider_session_sync_cache_key(project);
    // Refresh fast path: a completed synchronization inside the refresh
    // window is returned without re-discovering the provider inventory. The
    // discovery scan is the dominant cost of a synchronization (the provider
    // store is JSONL files on disk), and the client already deduplicates
    // refreshes with a 60-second TTL, so a 60-second backend window keeps
    // the steady-state background load off the provider store while bounding
    // imported Session freshness to at most one client cycle.
    if let Some(cached) = read_completed_provider_session_sync(&cache_key) {
        if cached.completed_at.elapsed() < PROVIDER_SESSION_SYNC_REFRESH_TTL {
            tracing::debug!(
                target: "sdkwork.agents.provider_session_sync",
                project_id = %project.project_id,
                "provider Session inventory synchronization served from the refresh cache"
            );
            return Ok(cached.result);
        }
    }
    let Some(host) = shared_agent_engine_host() else {
        // Without a provider agent engine host the inventory can never be
        // discovered; report the skipped synchronization instead of returning
        // a bare default that is indistinguishable from an empty project. The
        // skipped outcome is a completed outcome: recording it keeps the
        // refresh window effective and stops repeat requests from hammering
        // the no-host path.
        let mut result = ProviderSessionSynchronizationResult::default();
        result.record_issue(
            "provider_engine_unavailable",
            ProviderSessionSynchronizationIssueDisposition::Skipped,
            1,
        );
        record_completed_provider_session_sync(
            &cache_key,
            CompletedProviderSessionSync {
                fingerprint: String::new(),
                result: result.clone(),
                completed_at: Instant::now(),
            },
        );
        return Ok(result);
    };
    let discovery_started_at = Instant::now();
    let inventory = host
        .discover_provider_sessions(&ProviderSessionInventorySelector {
            directory_fingerprint,
            exact_cwd,
            unique_basename,
        })
        .map_err(runtime_facade_error)?;
    let discovery_elapsed_ms = discovery_started_at.elapsed().as_millis() as u64;

    // The fingerprint is recorded with the completed outcome so a later cold
    // synchronization can prove the identity set did not move and skip the
    // two full inventory sweeps inside `synchronize_provider_session_snapshot`.
    let fingerprint = provider_session_inventory_fingerprint(&inventory.items);
    let result = synchronize_provider_session_snapshot(service, project, subject, inventory)?;
    record_completed_provider_session_sync(
        &cache_key,
        CompletedProviderSessionSync {
            fingerprint,
            result: result.clone(),
            completed_at: Instant::now(),
        },
    );
    tracing::info!(
        target: "sdkwork.agents.provider_session_sync",
        project_id = %project.project_id,
        discovery_elapsed_ms,
        synchronized_session_count = result.synchronized_session_count,
        "provider Session inventory discovered and synchronized"
    );
    Ok(result)
}

pub(crate) fn synchronize_provider_session_transcript(
    service: &HttpService,
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    agent_id: String,
    session_id: String,
    subject: PolicySubject,
    provider_session_cwd_resolver: Option<
        &dyn sdkwork_agents_runtime_facade::ProviderSessionProjectCwdResolver,
    >,
) -> KernelResult<ProviderSessionTranscriptSyncOutcome> {
    let Some(engine_key) = agent_id.strip_prefix("agent.") else {
        return Ok(ProviderSessionTranscriptSyncOutcome::NotProviderSession);
    };
    if sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key) != Some(agent_id.as_str())
        || !is_provider_session_id_for(&session_id, engine_key)
    {
        return Ok(ProviderSessionTranscriptSyncOutcome::NotProviderSession);
    }
    let session = service.get_session(GetSessionCommand {
        tenant_id,
        organization_id,
        path_agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        owner_scope: Some(owner_user_id),
        requested_by: subject.clone(),
    })?;
    let exact_cwd = match (provider_session_cwd_resolver, session.project_id.as_deref()) {
        (Some(resolver), Some(project_id)) => {
            let project = match service.get_project(GetProjectCommand {
                tenant_id,
                organization_id,
                project_id: project_id.to_string(),
                owner_scope: Some(owner_user_id),
                requested_by: subject.clone(),
            }) {
                Ok(project) => Some(project),
                // A vanished or inaccessible project must not fail the
                // transcript read; the provider directory hint is best-effort.
                Err(error) => {
                    tracing::warn!(
                        target: "sdkwork.agents.provider_session_sync",
                        agent_id = %agent_id,
                        session_id = %session_id,
                        project_id = %project_id,
                        error_code = %error,
                        "provider Session project lookup failed; continuing without a working directory"
                    );
                    None
                }
            };
            match project {
                Some(project) => match resolver.resolve_project_cwd(
                    &sdkwork_agents_runtime_facade::ProviderSessionProjectCwdSelector {
                        tenant_id: project.tenant_id,
                        organization_id: project.organization_id,
                        owner_user_id: project.owner_user_id,
                        project_id: project.project_id,
                        project_name: project.name,
                    },
                ) {
                    Ok(cwd) => cwd,
                    // The provider directory hint is best-effort: transcripts
                    // are loaded by provider Session id and a failed mount
                    // lookup must not fail the read.
                    Err(error) => {
                        tracing::warn!(
                            target: "sdkwork.agents.provider_session_sync",
                            agent_id = %agent_id,
                            session_id = %session_id,
                            project_id = %project_id,
                            error_code = %error,
                            "provider Session working directory resolution failed; continuing without it"
                        );
                        None
                    }
                },
                None => None,
            }
        }
        _ => None,
    };
    let binding_page =
        service.list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
            query: SessionRuntimeBindingListQuery::for_session(
                tenant_id,
                organization_id,
                session_id.clone(),
            )
            .current_only()
            .with_pagination(PaginationParams::default().with_page_size(20)),
            path_agent_id: agent_id.clone(),
            owner_scope: Some(owner_user_id),
            requested_by: subject.clone(),
        })?;
    let Some(binding) = binding_page.items.into_iter().find(|binding| {
        binding.is_current
            && binding.status.as_str() == "active"
            && binding.transport_kind == "provider-session-history"
            && sdkwork_agents_runtime_facade::agent_engine_binding_id(engine_key)
                == Some(binding.provider_binding_id.as_str())
    }) else {
        // A provider-history Session without its canonical runtime binding
        // cannot resolve its provider identity; the persisted window (which
        // can never grow for such a Session) is still authoritative.
        return Ok(ProviderSessionTranscriptSyncOutcome::NoActiveBinding);
    };
    let Some(provider_session_id) = binding
        .provider_session_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(ProviderSessionTranscriptSyncOutcome::NoActiveBinding);
    };
    if shared_agent_engine_host().is_none() {
        // Without a provider agent engine host the transcript can never be
        // loaded; report the skipped synchronization instead of pretending
        // the provider Session has no messages.
        return Ok(ProviderSessionTranscriptSyncOutcome::EngineUnavailable);
    }
    let mut seen = HashSet::new();
    let worker_result = synchronize_provider_session_transcript_worker(
        service,
        tenant_id,
        organization_id,
        owner_user_id,
        &agent_id,
        engine_key,
        session.project_id.as_deref().unwrap_or_default(),
        &binding.provider_binding_id,
        &binding.provider_id,
        &binding.model_id,
        &session_id,
        provider_session_id,
        None,
        None,
        &subject,
        exact_cwd.as_deref(),
        &mut seen,
        0,
    );
    match worker_result {
        Ok(imported_item_count) => Ok(ProviderSessionTranscriptSyncOutcome::Imported {
            imported_item_count,
        }),
        // A provider engine read failure must not fail the transcript read:
        // report the skipped synchronization so the caller still returns the
        // persisted item window.
        Err(error)
            if error.kind() == sdkwork_agent_kernel::KernelErrorKind::ProviderUnavailable =>
        {
            Ok(ProviderSessionTranscriptSyncOutcome::EngineUnavailable)
        }
        Err(error) => Err(error),
    }
}

const MAX_PROVIDER_SUBAGENT_SYNC_DEPTH: usize = 16;

#[allow(clippy::too_many_arguments)]
fn synchronize_provider_session_transcript_worker(
    service: &HttpService,
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    agent_id: &str,
    engine_key: &str,
    project_id: &str,
    provider_binding_id: &str,
    provider_id: &str,
    model_id: &str,
    session_id: &str,
    provider_session_id: &str,
    canonical_parent_session_id: Option<&str>,
    parent_provider_session_id: Option<&str>,
    subject: &PolicySubject,
    exact_cwd: Option<&str>,
    seen: &mut HashSet<String>,
    depth: usize,
) -> KernelResult<usize> {
    if depth > MAX_PROVIDER_SUBAGENT_SYNC_DEPTH || !seen.insert(provider_session_id.to_string()) {
        return Ok(0);
    }
    // Sub-agent sessions are not part of the top-level inventory; ensure their
    // canonical Session and runtime binding exist before syncing messages so
    // the full sub-agent execution context is durable.
    if canonical_parent_session_id.is_some() {
        let stable_key = stable_provider_session_key(
            tenant_id,
            organization_id,
            owner_user_id,
            engine_key,
            provider_binding_id,
            provider_id,
            provider_session_id,
        );
        let child_session_id = canonical_provider_session_id(engine_key, &stable_key);
        let child_runtime_binding_id =
            canonical_provider_runtime_binding_id(engine_key, &stable_key);
        let requested_at = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| {
                KernelError::validation(format!(
                    "failed to format provider Session synchronization timestamp: {error}"
                ))
            })?;
        // Sub-agent Session reconciliation is best-effort: a single sub-agent
        // whose canonical Session or binding cannot be created must not fail
        // the parent transcript window (mirrors the item-level Conflict skip
        // below). The child is deliberately not archived here — a replay may
        // have hit a pre-existing Session with items — so any Session left
        // without a binding is reclaimed by the project-level orphan sweep
        // instead.
        if let Err(error) =
            service.reconcile_provider_session_history_session(CreateSessionCommand {
                tenant_id,
                organization_id,
                agent_id: agent_id.to_string(),
                owner_user_id,
                session_id: child_session_id.clone(),
                project_id: (!project_id.is_empty()).then(|| project_id.to_string()),
                session_kind: AgentSessionKind::Coding,
                entry_surface: AgentSessionEntrySurface::Pc,
                source_module: Some("birdcoder".to_string()),
                source_context_kind: Some("provider_session".to_string()),
                source_context_id: Some(project_id.to_string()),
                parent_session_id: canonical_parent_session_id.map(str::to_string),
                forked_from_turn_id: None,
                title: Some(provider_session_title(None, engine_key)),
                idempotency_key: Some(format!("provider-session:{engine_key}:{stable_key}")),
                payload_hash: Some(format!("provider-session-v1:{engine_key}:{stable_key}")),
                requested_by: subject.clone(),
                requested_at: requested_at.clone(),
            })
        {
            tracing::warn!(
                target: "sdkwork.agents.provider_session_sync",
                agent_id = %agent_id,
                session_id = %session_id,
                child_session_id = %child_session_id,
                error_kind = error.kind().as_str(),
                "provider Session sub-agent reconciliation failed: {error}"
            );
            return Ok(0);
        }
        if let Err(error) = service.reconcile_provider_session_history_runtime_binding(
            CreateSessionRuntimeBindingCommand {
                tenant_id,
                organization_id,
                path_agent_id: agent_id.to_string(),
                session_id: child_session_id.clone(),
                runtime_binding_id: Some(child_runtime_binding_id),
                runtime_location_id: None,
                host_mode: "server".to_string(),
                transport_kind: "provider-session-history".to_string(),
                provider_binding_id: provider_binding_id.to_string(),
                model_id: model_id.to_string(),
                provider_id: provider_id.to_string(),
                provider_session_id: Some(provider_session_id.to_string()),
                provider_session_tree_id: Some(provider_session_id.to_string()),
                provider_parent_session_id: parent_provider_session_id.map(str::to_string),
                provider_forked_from_session_id: None,
                provider_directory: None,
                owner_scope: Some(owner_user_id),
                requested_by: subject.clone(),
                requested_at,
            },
        ) {
            tracing::warn!(
                target: "sdkwork.agents.provider_session_sync",
                agent_id = %agent_id,
                session_id = %session_id,
                child_session_id = %child_session_id,
                error_kind = error.kind().as_str(),
                "provider Session sub-agent runtime binding reconciliation failed: {error}"
            );
            return Ok(0);
        }
    }
    let session = service.get_session(GetSessionCommand {
        tenant_id,
        organization_id,
        path_agent_id: agent_id.to_string(),
        session_id: session_id.to_string(),
        owner_scope: Some(owner_user_id),
        requested_by: subject.clone(),
    })?;
    let Some(host) = shared_agent_engine_host() else {
        // The top-level synchronization gate reports EngineUnavailable before
        // the worker runs; keep this defensive branch an error so a recursive
        // worker can never silently report an empty transcript.
        return Err(KernelError::ProviderUnavailable {
            provider_id: engine_key.to_string(),
        });
    };
    let messages = host
        .load_provider_session_messages_for_directory(engine_key, provider_session_id, exact_cwd)
        .map_err(runtime_facade_error)?;
    let mut unmatched_local_user_inputs = local_user_inputs_for_provider_reconciliation(
        service,
        tenant_id,
        organization_id,
        owner_user_id,
        agent_id,
        session_id,
        engine_key,
        subject,
    )?;
    let mut synchronized = 0;
    let mut tool_calls = HashMap::<String, (String, Option<String>)>::new();
    for message in messages {
        let requested_at = provider_message_requested_at(&message, session.updated_at.as_str());
        for mut item in provider_session_history_items(engine_key, &message) {
            if item.kind == AgentSessionItemKind::UserInput
                && unmatched_local_user_inputs.front().is_some_and(|content| {
                    content == item.content.as_deref().unwrap_or_default().trim()
                })
            {
                unmatched_local_user_inputs.pop_front();
                synchronized += 1;
                continue;
            }
            let item_id = stable_provider_session_item_id(
                engine_key,
                provider_session_id,
                item.provider_item_key.as_str(),
            );
            if item.kind == AgentSessionItemKind::ToolResult {
                if let Some((parent_item_id, tool_name)) = item
                    .tool_call_id
                    .as_ref()
                    .and_then(|tool_call_id| tool_calls.get(tool_call_id))
                {
                    item.parent_item_id = Some(parent_item_id.clone());
                    if item.tool_name.as_deref().is_none_or(|name| name == "tool") {
                        item.tool_name = tool_name.clone();
                    }
                }
            }
            let reconcile_result = service.reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    tenant_id,
                    organization_id,
                    session_id: session_id.to_string(),
                    item_id: item_id.clone(),
                    kind: item.kind,
                    content: item.content,
                    content_type: item.content_type,
                    status: item.status,
                    model_id: Some(model_id.to_string()),
                    provider_id: Some(provider_id.to_string()),
                    tool_name: item.tool_name.clone(),
                    tool_call_id: item.tool_call_id.clone(),
                    tool_arguments_json: item.tool_arguments_json,
                    tool_result_json: item.tool_result_json,
                    provider_payload_json: item.provider_payload_json,
                    parent_item_id: item.parent_item_id,
                    requested_by: subject.clone(),
                    requested_at: requested_at.clone(),
                },
                engine_key,
            );
            if let Err(error) = reconcile_result {
                if error.kind() == KernelErrorKind::Conflict {
                    // A terminal provider Session history item is immutable; one
                    // item that cannot be reconciled must not fail the whole
                    // synchronization window read. Skip it and keep syncing the
                    // remaining history so the caller still receives the window.
                    tracing::warn!(
                        target: "sdkwork.agents.provider_session_sync",
                        agent_id = %agent_id,
                        session_id = %session_id,
                        item_id = %item_id,
                        error_kind = error.kind().as_str(),
                        "provider Session transcript item skipped during synchronization: {error}"
                    );
                    continue;
                }
                return Err(error);
            }
            if item.kind == AgentSessionItemKind::ToolCall {
                if let Some(tool_call_id) = item.tool_call_id {
                    tool_calls.insert(tool_call_id, (item_id, item.tool_name));
                }
            }
            synchronized += 1;
        }
    }
    // Recurse into the provider sub-agent tree so every spawned session's
    // messages are synchronized with its canonical parent edge intact.
    let children = host
        .load_provider_session_children(engine_key, provider_session_id, exact_cwd)
        .map_err(runtime_facade_error)?;
    let mut total = synchronized;
    for child in children {
        let child_stable_key = stable_provider_session_key(
            tenant_id,
            organization_id,
            owner_user_id,
            engine_key,
            provider_binding_id,
            provider_id,
            &child,
        );
        let child_session_id = canonical_provider_session_id(engine_key, &child_stable_key);
        match synchronize_provider_session_transcript_worker(
            service,
            tenant_id,
            organization_id,
            owner_user_id,
            agent_id,
            engine_key,
            project_id,
            provider_binding_id,
            provider_id,
            model_id,
            &child_session_id,
            &child,
            Some(session_id),
            Some(provider_session_id),
            subject,
            exact_cwd,
            seen,
            depth + 1,
        ) {
            Ok(imported) => total += imported,
            // A failing sub-agent must not fail the parent window; its
            // messages are simply absent from this synchronization pass.
            Err(error) => {
                tracing::warn!(
                    target: "sdkwork.agents.provider_session_sync",
                    agent_id = %agent_id,
                    session_id = %session_id,
                    child_session_id = %child_session_id,
                    error_kind = error.kind().as_str(),
                    "provider Session sub-agent transcript synchronization failed: {error}"
                );
            }
        }
    }
    Ok(total)
}

fn local_user_inputs_for_provider_reconciliation(
    service: &HttpService,
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    agent_id: &str,
    session_id: &str,
    engine_key: &str,
    subject: &PolicySubject,
) -> KernelResult<VecDeque<String>> {
    let mut contents = VecDeque::new();
    let provider_item_prefix = canonical_provider_item_id_prefix(engine_key);
    let mut offset = 0usize;
    loop {
        let pagination = PaginationParams {
            page_size: 200,
            offset,
            page_token: None,
        };
        let page = service.list_session_items(ListSessionItemsCommand {
            query: SessionItemListQuery::for_session(
                tenant_id,
                organization_id,
                session_id.to_string(),
            )
            .with_kind(AgentSessionItemKind::UserInput.as_str())
            .with_pagination(pagination),
            path_agent_id: agent_id.to_string(),
            owner_scope: Some(owner_user_id),
            requested_by: subject.clone(),
        })?;
        let page_len = page.items.len();
        for item in page.items {
            if item.item_id.starts_with(&provider_item_prefix) {
                continue;
            }
            if let Some(content) = item
                .content
                .as_deref()
                .map(str::trim)
                .filter(|content| !content.is_empty())
            {
                contents.push_back(content.to_string());
            }
        }
        if page_len < 200 {
            break;
        }
        offset = offset.checked_add(page_len).ok_or_else(|| {
            KernelError::conflict("provider Session user input pagination overflow")
        })?;
        if offset >= PROVIDER_SESSION_RECONCILIATION_MAX_ITEMS {
            return Err(KernelError::validation(format!(
                "provider Session history exceeds {PROVIDER_SESSION_RECONCILIATION_MAX_ITEMS} local user items"
            )));
        }
    }
    Ok(contents)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderSessionHistoryItem {
    provider_item_key: String,
    kind: AgentSessionItemKind,
    content: Option<String>,
    content_type: String,
    status: AgentSessionItemStatus,
    tool_name: Option<String>,
    tool_call_id: Option<String>,
    tool_arguments_json: Option<String>,
    tool_result_json: Option<String>,
    provider_payload_json: Option<String>,
    parent_item_id: Option<String>,
}

fn provider_session_history_items(
    engine_key: &str,
    message: &AgentMessage,
) -> Vec<ProviderSessionHistoryItem> {
    let raw_provider_payload = message
        .parts
        .iter()
        .find(|part| is_raw_provider_item(engine_key, part))
        .and_then(|part| part.json.clone());
    let has_semantic_parts = message
        .parts
        .iter()
        .any(|part| !is_raw_provider_item(engine_key, part));
    let mut legacy_text_item_available = true;
    message
        .parts
        .iter()
        .flat_map(|part| {
            if has_semantic_parts && is_raw_provider_item(engine_key, part) {
                return Vec::new();
            }
            let content_type = provider_part_content_type(engine_key, part);
            let Some(kind) = provider_session_item_kind(message.role, part.kind, content_type)
            else {
                return Vec::new();
            };
            let uses_legacy_message_id = legacy_text_item_available
                && part.kind == AgentPartKind::Text
                && !matches!(kind, AgentSessionItemKind::Reasoning);
            if uses_legacy_message_id {
                legacy_text_item_available = false;
            }
            let provider_item_key = if uses_legacy_message_id {
                message.message_id.clone()
            } else {
                format!("{}\u{0}{}", message.message_id, part.part_id)
            };
            let tool_call_id = part.tool_call_id.clone().or_else(|| {
                provider_part_metadata(engine_key, part, "tool_call_id").map(str::to_string)
            });
            let has_result = provider_part_metadata(engine_key, part, "has_result") == Some("true");
            let status = provider_session_item_status(engine_key, part, kind, has_result);
            let provider_json = part.json.clone();
            let tool_payload = if kind == AgentSessionItemKind::ToolCall
                || kind == AgentSessionItemKind::ToolResult
            {
                provider_json
                    .or_else(|| raw_provider_payload.clone())
                    .or_else(|| {
                        Some(
                            serde_json::json!({
                                "id": tool_call_id.as_deref(),
                                "type": kind.as_str(),
                                "name": part.name.as_deref(),
                                "output": part.text.as_deref(),
                            })
                            .to_string(),
                        )
                    })
            } else {
                None
            };
            let content = match kind {
                AgentSessionItemKind::ToolCall | AgentSessionItemKind::ToolResult => None,
                AgentSessionItemKind::ArtifactReference | AgentSessionItemKind::StatusNotice => {
                    part.json
                        .clone()
                        .or_else(|| part.content_ref.clone())
                        .or_else(|| part.artifact_id.clone())
                        .or_else(|| part.policy_decision_id.clone())
                        .or_else(|| part.text.clone())
                }
                _ => part.text.clone().or_else(|| part.json.clone()),
            }
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
            let item_content_type = if matches!(
                kind,
                AgentSessionItemKind::ToolCall
                    | AgentSessionItemKind::ToolResult
                    | AgentSessionItemKind::StatusNotice
            ) || part.json.is_some()
            {
                "application/json".to_string()
            } else {
                part.mime_type
                    .clone()
                    .unwrap_or_else(|| "text/plain".to_string())
            };
            let item = ProviderSessionHistoryItem {
                provider_item_key: provider_item_key.clone(),
                kind,
                content,
                content_type: item_content_type,
                status,
                tool_name: part.name.clone(),
                tool_call_id,
                tool_arguments_json: (kind == AgentSessionItemKind::ToolCall)
                    .then(|| tool_payload.clone())
                    .flatten(),
                tool_result_json: (kind == AgentSessionItemKind::ToolResult)
                    .then_some(tool_payload)
                    .flatten(),
                provider_payload_json: raw_provider_payload.clone(),
                parent_item_id: None,
            };
            if kind == AgentSessionItemKind::ToolCall && has_result {
                let result = ProviderSessionHistoryItem {
                    provider_item_key: format!("{provider_item_key}\u{0}result"),
                    kind: AgentSessionItemKind::ToolResult,
                    content: None,
                    content_type: "application/json".to_string(),
                    status,
                    tool_name: part.name.clone(),
                    tool_call_id: item.tool_call_id.clone(),
                    tool_arguments_json: None,
                    tool_result_json: item.tool_arguments_json.clone(),
                    provider_payload_json: raw_provider_payload.clone(),
                    parent_item_id: None,
                };
                vec![item, result]
            } else {
                vec![item]
            }
        })
        .collect()
}

fn is_raw_provider_item(engine_key: &str, part: &AgentPart) -> bool {
    provider_part_content_type(engine_key, part) == Some("raw_provider_item")
}

fn provider_session_item_kind(
    role: AgentMessageRole,
    part_kind: AgentPartKind,
    content_type: Option<&str>,
) -> Option<AgentSessionItemKind> {
    // Codex projects reasoning as `reasoning_summary` / `reasoning_content`
    // part types; other providers use `reasoning` / `thinking`. All of them
    // map to the canonical Reasoning item so the history reconciliation path
    // matches the live turn projection.
    if matches!(
        content_type,
        Some("reasoning" | "reasoning_summary" | "reasoning_content" | "thinking")
    ) {
        return Some(AgentSessionItemKind::Reasoning);
    }
    if content_type.is_some_and(|value| {
        matches!(
            value,
            "advisor_tool_result"
                | "bash_code_execution_tool_result"
                | "code_execution_tool_result"
                | "collab_agent_tool_result"
                | "function_call_output"
                | "custom_tool_call_output"
                | "mcp_tool_result"
                | "text_editor_code_execution_tool_result"
                | "tool_result"
                | "tool_search_tool_result"
                | "web_fetch_tool_result"
                | "web_search_tool_result"
        )
    }) {
        return Some(AgentSessionItemKind::ToolResult);
    }
    if matches!(
        content_type,
        Some(
            "compaction"
                | "context_compacted"
                | "queue-operation"
                | "step-start"
                | "step-finish"
                | "task_complete"
                | "task_started"
        )
    ) {
        return Some(AgentSessionItemKind::StatusNotice);
    }
    if content_type.is_some_and(|value| {
        matches!(
            value,
            "attachment" | "input_image" | "image" | "document" | "file"
        )
    }) {
        return Some(AgentSessionItemKind::ArtifactReference);
    }

    match part_kind {
        AgentPartKind::Text => match role {
            AgentMessageRole::User => Some(AgentSessionItemKind::UserInput),
            AgentMessageRole::Agent | AgentMessageRole::Model => {
                Some(AgentSessionItemKind::AssistantOutput)
            }
            AgentMessageRole::System | AgentMessageRole::Policy => {
                Some(AgentSessionItemKind::SystemInstruction)
            }
            AgentMessageRole::Tool => Some(AgentSessionItemKind::ToolResult),
            AgentMessageRole::Adapter => Some(AgentSessionItemKind::StatusNotice),
        },
        AgentPartKind::ToolCallRef => Some(AgentSessionItemKind::ToolCall),
        AgentPartKind::Error => Some(AgentSessionItemKind::ErrorNotice),
        AgentPartKind::PolicyDecisionRef => Some(AgentSessionItemKind::StatusNotice),
        AgentPartKind::Json => match role {
            AgentMessageRole::Tool => Some(AgentSessionItemKind::ToolResult),
            _ => Some(AgentSessionItemKind::ArtifactReference),
        },
        AgentPartKind::BinaryRef
        | AgentPartKind::FileRef
        | AgentPartKind::ArtifactRef
        | AgentPartKind::ImageRef
        | AgentPartKind::AudioRef
        | AgentPartKind::VideoRef => Some(AgentSessionItemKind::ArtifactReference),
    }
}

fn provider_session_item_status(
    engine_key: &str,
    part: &AgentPart,
    kind: AgentSessionItemKind,
    has_result: bool,
) -> AgentSessionItemStatus {
    match provider_part_metadata(engine_key, part, "status") {
        Some("pending" | "queued" | "running" | "in_progress") => AgentSessionItemStatus::Pending,
        Some("failed" | "error") => AgentSessionItemStatus::Failed,
        Some("cancelled" | "canceled" | "aborted") => AgentSessionItemStatus::Cancelled,
        Some("completed" | "complete" | "success" | "succeeded") => {
            AgentSessionItemStatus::Completed
        }
        _ if kind == AgentSessionItemKind::ToolCall && !has_result => {
            AgentSessionItemStatus::Pending
        }
        _ => AgentSessionItemStatus::Completed,
    }
}

fn provider_part_content_type<'a>(engine_key: &str, part: &'a AgentPart) -> Option<&'a str> {
    provider_part_metadata(engine_key, part, "content_type")
}

fn provider_part_metadata<'a>(
    engine_key: &str,
    part: &'a AgentPart,
    field_name: &str,
) -> Option<&'a str> {
    if let Some(value) = part.metadata_value(format!("sdkwork.provider.{field_name}").as_str()) {
        return Some(value);
    }
    let namespace = match engine_key {
        "claude-code" => "claude",
        other => other,
    };
    part.metadata_value(format!("{namespace}.{field_name}").as_str())
}

fn synchronize_provider_session_inventory(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    inventory: Vec<ProviderSessionInventoryItem>,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    synchronize_provider_session_inventory_with_timeout(
        service,
        project,
        subject,
        inventory,
        PROVIDER_SESSION_RECONCILIATION_TIMEOUT,
    )
}

fn synchronize_provider_session_snapshot(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    snapshot: ProviderSessionInventorySnapshot,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    let ProviderSessionInventorySnapshot {
        directory_resolved,
        items,
        successful_engine_keys,
        issues,
        unattributed_provider_sessions,
    } = snapshot;
    let visible_provider_sessions = items
        .iter()
        .map(|item| {
            (
                item.engine_key.clone(),
                item.binding_id.trim().to_string(),
                item.provider_id.trim().to_string(),
                item.session.session_id.trim().to_string(),
            )
        })
        .collect::<HashSet<_>>();
    // Computed before `items` moves into the per-Session reconciliation; it
    // must match the fingerprint recorded by the synchronization entry point.
    let inventory_fingerprint = provider_session_inventory_fingerprint(&items);
    let mut result =
        synchronize_provider_session_inventory(service.clone(), project, subject.clone(), items)?;
    for _issue in issues {
        result.record_failed("provider_inventory_unavailable");
    }
    if unattributed_provider_sessions {
        // Provider Sessions exist in the runtime inventory but none could be
        // attributed to this project directory. Report it as a skipped issue
        // instead of silently returning an empty project, so consumers can
        // distinguish "no provider Sessions" from "inventory unattributed".
        result.record_skipped("provider_inventory_unattributed");
    }
    // The two full inventory sweeps are a pure function of the discovered
    // Session identity set; an unchanged fingerprint proves their outcome
    // cannot have changed since the last completed synchronization, so they
    // are skipped. Per-Session provider metadata reconciliation still runs so
    // provider renames and directory fields converge even when the set is
    // stable. Tests that call this function directly have no cached entry and
    // therefore always run the full path.
    let inventory_fingerprint_unchanged =
        read_completed_provider_session_sync(&provider_session_sync_cache_key(project))
            .is_some_and(|cached| cached.fingerprint == inventory_fingerprint);
    if directory_resolved && !inventory_fingerprint_unchanged {
        reconcile_missing_provider_session_directories(
            service.clone(),
            project,
            subject.clone(),
            &successful_engine_keys,
            &visible_provider_sessions,
            &mut result,
        )?;
    }
    if !inventory_fingerprint_unchanged {
        reconcile_orphaned_provider_sessions(
            &service,
            project,
            subject,
            &successful_engine_keys,
            &mut result,
        )?;
    }
    Ok(result)
}

/// Archives provider Session rows that can never be synchronized: Sessions
/// attributed to a provider transcript but left without a canonical runtime
/// binding and without items (e.g. created under a retired stable key scheme
/// or whose binding creation failed). Such Sessions have no recoverable
/// provider identity, so they would otherwise linger in the active inventory
/// as permanently empty conversations. Sessions with any binding or with
/// items are never touched.
fn reconcile_orphaned_provider_sessions(
    service: &HttpService,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    successful_engine_keys: &[String],
    result: &mut ProviderSessionSynchronizationResult,
) -> KernelResult<()> {
    let mut archived_count = 0usize;
    for engine_key in successful_engine_keys {
        let Some(agent_id) = sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
        else {
            continue;
        };
        let mut cursor: Option<SessionListCursor> = None;
        loop {
            let sessions = service.list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id.clone())
                    .for_agent(agent_id)
                    .include_archived()
                    .with_cursor_page(200, cursor),
                requested_by: subject.clone(),
            })?;
            for session in sessions.items {
                if session.source_module.as_deref() != Some("birdcoder")
                    || session.source_context_kind.as_deref() != Some("provider_session")
                    || session.deleted_at.is_some()
                    || session.status != AgentSessionStatus::Active
                    || session.item_count != 0
                {
                    continue;
                }
                let bindings =
                    service.list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                        query: SessionRuntimeBindingListQuery::for_session(
                            project.tenant_id,
                            project.organization_id,
                            session.session_id.clone(),
                        )
                        .current_only()
                        .with_pagination(PaginationParams::default().with_page_size(1)),
                        path_agent_id: agent_id.to_string(),
                        owner_scope: Some(project.owner_user_id),
                        requested_by: subject.clone(),
                    })?;
                if !bindings.items.is_empty() {
                    continue;
                }
                // The orphan sweep is a conservative cleanup; a single
                // Session that cannot be closed or archived must not fail the
                // whole project synchronization. Record the issue and keep
                // sweeping the remaining Sessions.
                let closed = if session.status == AgentSessionStatus::Active {
                    match service.close_session(CloseSessionCommand {
                        tenant_id: project.tenant_id,
                        organization_id: project.organization_id,
                        path_agent_id: agent_id.to_string(),
                        session_id: session.session_id.clone(),
                        expected_version: Some(session.version),
                        owner_scope: Some(project.owner_user_id),
                        requested_by: subject.clone(),
                        requested_at: project_synchronization_requested_at(),
                    }) {
                        Ok(closed) => closed,
                        Err(error) => {
                            result.record_issue(
                                "orphaned_provider_session_close_failed",
                                ProviderSessionSynchronizationIssueDisposition::Skipped,
                                1,
                            );
                            tracing::warn!(
                                target: "sdkwork.agents.provider_session_sync",
                                session_id = %session.session_id,
                                error_kind = error.kind().as_str(),
                                "orphaned provider Session close failed: {error}"
                            );
                            continue;
                        }
                    }
                } else {
                    session.clone()
                };
                if let Err(error) = service.archive_session(ArchiveSessionCommand {
                    tenant_id: project.tenant_id,
                    organization_id: project.organization_id,
                    path_agent_id: agent_id.to_string(),
                    session_id: session.session_id.clone(),
                    expected_version: Some(closed.version),
                    owner_scope: Some(project.owner_user_id),
                    requested_by: subject.clone(),
                    requested_at: project_synchronization_requested_at(),
                }) {
                    result.record_issue(
                        "orphaned_provider_session_archive_failed",
                        ProviderSessionSynchronizationIssueDisposition::Skipped,
                        1,
                    );
                    tracing::warn!(
                        target: "sdkwork.agents.provider_session_sync",
                        session_id = %session.session_id,
                        error_kind = error.kind().as_str(),
                        "orphaned provider Session archive failed: {error}"
                    );
                    continue;
                }
                tracing::info!(
                    target: "sdkwork.agents.provider_session_sync",
                    session_id = %session.session_id,
                    item_count = session.item_count,
                    "orphaned provider Session archived (no runtime binding, zero items)"
                );
                archived_count += 1;
            }
            if !sessions.has_more {
                break;
            }
            cursor = sessions
                .next_page_token
                .as_deref()
                .map(decode_session_list_cursor)
                .transpose()?;
        }
    }
    result.record_issue(
        "orphaned_provider_session_archived",
        ProviderSessionSynchronizationIssueDisposition::Skipped,
        archived_count,
    );
    Ok(())
}

fn project_synchronization_requested_at() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 formatting of the current instant cannot fail")
}

fn reconcile_missing_provider_session_directories(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    successful_engine_keys: &[String],
    visible_provider_sessions: &HashSet<(String, String, String, String)>,
    result: &mut ProviderSessionSynchronizationResult,
) -> KernelResult<()> {
    for engine_key in successful_engine_keys {
        let Some(agent_id) = sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
        else {
            continue;
        };
        let mut cursor: Option<SessionListCursor> = None;
        loop {
            let sessions = service.list_sessions(crate::application::ListSessionsCommand {
                query: crate::ports::SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id.clone())
                    .for_agent(agent_id)
                    .include_archived()
                    .with_cursor_page(200, cursor),
                requested_by: subject.clone(),
            })?;
            for session in sessions.items {
                if session.source_module.as_deref() != Some("birdcoder")
                    || session.source_context_kind.as_deref() != Some("provider_session")
                    || session.deleted_at.is_some()
                {
                    continue;
                }
                let bindings =
                    service.list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                        query: SessionRuntimeBindingListQuery::for_session(
                            project.tenant_id,
                            project.organization_id,
                            session.session_id.clone(),
                        )
                        .current_only()
                        .with_pagination(PaginationParams::default().with_page_size(1)),
                        path_agent_id: agent_id.to_string(),
                        owner_scope: Some(project.owner_user_id),
                        requested_by: subject.clone(),
                    })?;
                let Some(binding) = bindings.items.into_iter().next() else {
                    continue;
                };
                let Some(provider_session_id) = binding.provider_session_id.clone() else {
                    continue;
                };
                if visible_provider_sessions.contains(&(
                    engine_key.clone(),
                    binding.provider_binding_id.clone(),
                    binding.provider_id.clone(),
                    provider_session_id,
                )) || (!binding.provider_visible && binding.provider_archived)
                {
                    continue;
                }
                let requested_at = OffsetDateTime::now_utc()
                    .format(&Rfc3339)
                    .map_err(|error| KernelError::Internal {
                        message: error.to_string(),
                    })?;
                let mut directory = sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry {
                    title: binding.provider_title.clone(),
                    title_source: binding.provider_title_source.clone(),
                    preview: binding.provider_preview.clone(),
                    created_at: binding.provider_created_at.clone(),
                    updated_at: binding.provider_updated_at.clone(),
                    recency_at: binding.provider_recency_at.clone(),
                    pinned: binding.provider_pinned,
                    archived: true,
                    visible: false,
                    source: binding.provider_source.clone(),
                    sort_key: binding.provider_sort_key.clone().unwrap_or_default(),
                };
                directory.pinned = false;
                service.reconcile_provider_session_history_runtime_binding_directory(
                    crate::application::ReconcileProviderSessionRuntimeBindingDirectoryCommand {
                        tenant_id: project.tenant_id,
                        organization_id: project.organization_id,
                        path_agent_id: agent_id.to_string(),
                        session_id: session.session_id,
                        runtime_binding_id: binding.runtime_binding_id,
                        expected_version: binding.version,
                        provider_directory: directory,
                        owner_scope: Some(project.owner_user_id),
                        requested_by: subject.clone(),
                        requested_at,
                    },
                )?;
                result.synchronized_session_count += 1;
            }
            if !sessions.has_more {
                break;
            }
            cursor = sessions
                .next_page_token
                .as_deref()
                .map(decode_session_list_cursor)
                .transpose()?;
        }
    }
    Ok(())
}

fn synchronize_provider_session_inventory_with_timeout(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    inventory: Vec<ProviderSessionInventoryItem>,
    timeout: Duration,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    let facade =
        HttpAgentsSessionFacade::for_provider_session_history_reconciliation(service.clone());
    let actor = AgentsSessionActor {
        subject_id: subject.subject_id.clone(),
        roles: subject.roles.clone(),
    };
    let started_at = Instant::now();
    let inventory_len = inventory.len();
    let mut result = ProviderSessionSynchronizationResult::default();
    let mut seen_provider_sessions = HashSet::new();
    for (index, mut item) in inventory.into_iter().enumerate() {
        if index >= PROVIDER_SESSION_RECONCILIATION_MAX_ITEMS {
            result.record_issue(
                "inventory_item_limit_exceeded",
                ProviderSessionSynchronizationIssueDisposition::Failed,
                inventory_len - index,
            );
            break;
        }
        if started_at.elapsed() >= timeout {
            result.record_issue(
                "synchronization_time_budget_exceeded",
                ProviderSessionSynchronizationIssueDisposition::Failed,
                inventory_len - index,
            );
            break;
        }
        if item.session.kind == SessionKind::Subagent
            && item
                .session
                .parent_session_id
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
        {
            // A subagent without a parent provider session has no canonical
            // tree edge; keep it out of the project inventory until its parent
            // can be resolved.
            result.record_skipped("subagent_without_parent");
            continue;
        }
        let provider_id = item.provider_id.trim().to_string();
        let provider_session_id = item.session.session_id.trim().to_string();
        if provider_id.is_empty() || provider_session_id.is_empty() {
            result.record_failed("invalid_provider_session_identity");
            continue;
        }
        let provider_binding_id = item.binding_id.trim().to_string();
        if provider_binding_id.is_empty() {
            result.record_failed("invalid_runtime_binding_identity");
            continue;
        }
        if !seen_provider_sessions.insert((
            provider_binding_id.clone(),
            provider_id.clone(),
            provider_session_id.clone(),
        )) {
            result.record_skipped("duplicate_provider_session");
            continue;
        }
        let requested_at = match provider_session_requested_at(&item, project) {
            Ok(requested_at) => requested_at,
            Err(_) => {
                result.record_failed("invalid_synchronization_timestamp");
                continue;
            }
        };
        if let Err(error) = service.ensure_agent_engine_runtime_identity(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
            &item.engine_key,
            &item.agent_id,
            &provider_binding_id,
            &provider_id,
            subject.clone(),
            &requested_at,
        ) {
            if is_fatal_provider_session_synchronization_error(&error) {
                return Err(error);
            }
            record_provider_session_reconciliation_failure(
                project,
                &item.engine_key,
                "runtime_identity_reconciliation_failed",
                &error,
            );
            result.record_failed("runtime_identity_reconciliation_failed");
            continue;
        }
        let stable_key = stable_provider_session_key(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
            &item.engine_key,
            &provider_binding_id,
            &provider_id,
            &provider_session_id,
        );
        let session_id = canonical_provider_session_id(&item.engine_key, &stable_key);
        let runtime_binding_id =
            canonical_provider_runtime_binding_id(&item.engine_key, &stable_key);
        // Sub-agent sessions persist the canonical parent session edge so the
        // canonical session tree mirrors the provider sub-agent topology. The
        // parent must already be synchronized; otherwise the edge is deferred
        // to a later inventory pass.
        let parent_session_id = item
            .session
            .parent_session_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|parent_provider_session_id| {
                let parent_stable_key = stable_provider_session_key(
                    project.tenant_id,
                    project.organization_id,
                    project.owner_user_id,
                    &item.engine_key,
                    &provider_binding_id,
                    &provider_id,
                    parent_provider_session_id,
                );
                canonical_provider_session_id(&item.engine_key, &parent_stable_key)
            });
        if let Some(parent_session_id) = parent_session_id.as_deref() {
            match service.get_session(GetSessionCommand {
                tenant_id: project.tenant_id,
                organization_id: project.organization_id,
                path_agent_id: item.agent_id.clone(),
                session_id: parent_session_id.to_string(),
                owner_scope: Some(project.owner_user_id),
                requested_by: subject.clone(),
            }) {
                Ok(_) => {}
                Err(error)
                    if error.detail_value("sdkwork.not_found") == Some("true")
                        && error.message() == "session not found" =>
                {
                    result.record_skipped("parent_session_not_synced");
                    continue;
                }
                Err(error) => {
                    record_provider_session_reconciliation_failure(
                        project,
                        &item.engine_key,
                        "parent_session_resolution_failed",
                        &error,
                    );
                    result.record_failed("parent_session_resolution_failed");
                    continue;
                }
            }
        }
        item.directory = clamp_provider_session_directory(item.directory);
        match service.retire_legacy_provider_session_bindings(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
            &item.engine_key,
            &provider_binding_id,
            &provider_session_id,
            &session_id,
            &project.project_id,
            subject.clone(),
            &requested_at,
        ) {
            Ok(crate::application::ProviderSessionBindingClaim::Free)
            | Ok(crate::application::ProviderSessionBindingClaim::AlreadyTarget)
            | Ok(crate::application::ProviderSessionBindingClaim::Retired) => {}
            Ok(crate::application::ProviderSessionBindingClaim::AlreadyBoundByUserSession) => {
                result.record_skipped("provider_session_already_bound");
                continue;
            }
            Err(error) => {
                if is_fatal_provider_session_synchronization_error(&error) {
                    return Err(error);
                }
                record_provider_session_reconciliation_failure(
                    project,
                    &item.engine_key,
                    "legacy_runtime_binding_retirement_failed",
                    &error,
                );
                result.record_failed("legacy_runtime_binding_retirement_failed");
                continue;
            }
        }
        let title = provider_session_title(item.session.title.as_deref(), &item.engine_key);
        let model_id = item
            .session
            .model
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| item.default_model_id.clone());
        let reconciliation = facade.resolve_or_create_session(ResolveAgentsSessionRequest {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            owner_user_id: project.owner_user_id,
            agent_id: item.agent_id.clone(),
            session_id,
            project_id: Some(project.project_id.clone()),
            session_kind: AgentsSessionKind::Coding,
            entry_surface: AgentsSessionEntrySurface::Pc,
            source_module: Some("birdcoder".to_string()),
            source_context_kind: Some("provider_session".to_string()),
            source_context_id: Some(project.project_id.clone()),
            parent_session_id,
            forked_from_turn_id: None,
            title,
            idempotency_key: format!("provider-session:{}:{}", item.engine_key, stable_key),
            payload_hash: format!("provider-session-v1:{}:{}", item.engine_key, stable_key),
            runtime_binding: Some(AgentsSessionRuntimeBindingDescriptor {
                runtime_binding_id,
                runtime_location_id: None,
                host_mode: "server".to_string(),
                transport_kind: "provider-session-history".to_string(),
                provider_binding_id,
                model_id,
                provider_id,
                provider_session_id: Some(provider_session_id.clone()),
                provider_session_tree_id: Some(provider_session_id),
                provider_parent_session_id: item.session.parent_session_id.clone(),
                provider_forked_from_session_id: item.session.forked_from_id.clone(),
                provider_directory: Some(item.directory.clone()),
            }),
            actor: actor.clone(),
            requested_at,
        });
        if let Err(error) = reconciliation {
            let error = runtime_facade_error(error);
            if is_fatal_provider_session_synchronization_error(&error) {
                return Err(error);
            }
            record_provider_session_reconciliation_failure(
                project,
                &item.engine_key,
                "session_reconciliation_failed",
                &error,
            );
            result.record_failed("session_reconciliation_failed");
            continue;
        }
        result.synchronized_session_count += 1;
    }
    Ok(result)
}

fn is_fatal_provider_session_synchronization_error(error: &KernelError) -> bool {
    matches!(
        error.kind(),
        KernelErrorKind::PermissionRequired
            | KernelErrorKind::PolicyDenied
            | KernelErrorKind::SecurityViolation
    )
}

fn record_provider_session_reconciliation_failure(
    project: &AgentProjectRecord,
    engine_key: &str,
    issue_code: &'static str,
    error: &KernelError,
) {
    tracing::warn!(
        target: "sdkwork.agents.provider_session_sync",
        project_id = %project.project_id,
        engine_key = %engine_key,
        issue_code,
        error_code = error.code(),
        error_kind = error.kind().as_str(),
        "provider session inventory item reconciliation failed: {error}"
    );
}

fn provider_session_requested_at(
    item: &ProviderSessionInventoryItem,
    project: &AgentProjectRecord,
) -> KernelResult<String> {
    [
        item.session.updated_at.as_deref(),
        item.session.created_at.as_deref(),
        Some(project.updated_at.as_str()),
        Some(project.created_at.as_str()),
    ]
    .into_iter()
    .flatten()
    .find_map(normalize_provider_session_timestamp)
    .ok_or_else(|| {
        KernelError::validation("provider session inventory has no valid synchronization timestamp")
    })
}

fn normalize_provider_session_timestamp(value: &str) -> Option<String> {
    let value = value.trim();
    if OffsetDateTime::parse(value, &Rfc3339).is_ok() {
        return Some(value.to_string());
    }

    let (date, time) = value.split_once(' ')?;
    let mut candidate = format!("{date}T{time}");
    let offset_index = candidate
        .char_indices()
        .skip(11)
        .filter_map(|(index, character)| matches!(character, '+' | '-').then_some(index))
        .last()?;
    let offset = &candidate[offset_index..];
    if offset.len() == 3
        && offset[1..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        candidate.push_str(":00");
    }

    OffsetDateTime::parse(&candidate, &Rfc3339)
        .ok()
        .map(|_| candidate)
}

/// Derives the RFC3339 item timestamp for one provider transcript message.
/// Provider SDKs are not guaranteed to emit RFC3339 timestamps (space
/// separators, compact offsets, missing offsets), so the value is normalized
/// the same way as provider Session timestamps before it can be persisted as
/// Session item created_at/updated_at. When neither the message nor the
/// Session timestamp is parsable, the current instant is used so an
/// unparsable provider timestamp can never corrupt the transcript row.
fn provider_message_requested_at(message: &AgentMessage, session_updated_at: &str) -> String {
    message
        .created_at
        .as_deref()
        .and_then(normalize_provider_session_timestamp)
        .or_else(|| normalize_provider_session_timestamp(session_updated_at))
        .unwrap_or_else(|| {
            OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .expect("RFC3339 formatting of the current instant cannot fail")
        })
}

fn provider_session_title(value: Option<&str>, engine_key: &str) -> String {
    let compact = value
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let value = if compact.is_empty() {
        format!("{engine_key} session")
    } else {
        compact
    };
    clamp_provider_text(value, PROVIDER_SESSION_TITLE_MAX_BYTES)
}

/// Clamps provider directory fields to the service-side limits so a single
/// oversized provider field can never fail the whole Session reconciliation.
fn clamp_provider_session_directory(
    mut directory: sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry,
) -> sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry {
    directory.title = directory.title.map(|value| clamp_provider_text(value, 512));
    directory.preview = directory
        .preview
        .map(|value| clamp_provider_text(value, 4096));
    directory.source = directory
        .source
        .map(|value| clamp_provider_text(value, 256));
    directory.sort_key = clamp_provider_text(directory.sort_key, 512);
    directory
}

fn clamp_provider_text(value: String, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value;
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].trim_end().to_string()
}

fn stable_provider_session_key(
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    engine_key: &str,
    provider_binding_id: &str,
    provider_id: &str,
    provider_session_id: &str,
) -> String {
    let digest = sdkwork_utils_rust::sha256_hash(
        format!(
            "provider-session-v3\u{0}{tenant_id}\u{0}{organization_id}\u{0}{owner_user_id}\u{0}{engine_key}\u{0}{provider_binding_id}\u{0}{provider_id}\u{0}{provider_session_id}"
        )
        .as_bytes(),
    );
    digest[..32].to_string()
}

pub(crate) fn stable_provider_session_item_id(
    engine_key: &str,
    provider_session_id: &str,
    provider_message_id: &str,
) -> String {
    let digest = sdkwork_utils_rust::sha256_hash(
        format!(
            "provider-session-item-v1\u{0}{engine_key}\u{0}{provider_session_id}\u{0}{provider_message_id}"
        )
        .as_bytes(),
    );
    canonical_provider_item_id(engine_key, &digest[..32])
}

pub(crate) fn runtime_facade_error(error: RuntimeFacadeError) -> KernelError {
    match error {
        RuntimeFacadeError::InvalidInput(message)
        | RuntimeFacadeError::EngineMismatch {
            slot_engine: message,
            ..
        } => KernelError::validation(message),
        RuntimeFacadeError::UnsupportedEngine { engine_key }
        | RuntimeFacadeError::UnsupportedLiveInteraction { engine_key, .. } => {
            KernelError::validation(format!("unsupported engineId \"{engine_key}\""))
        }
        RuntimeFacadeError::UnsupportedCapability { capability_id, .. } => {
            KernelError::CapabilityMissing { capability_id }
        }
        RuntimeFacadeError::BlankPrompt => KernelError::validation("prompt must not be blank"),
        RuntimeFacadeError::EngineUnavailable { engine_key, .. } => {
            KernelError::ProviderUnavailable {
                provider_id: engine_key,
            }
        }
        RuntimeFacadeError::Kernel(message) | RuntimeFacadeError::Handler(message) => {
            KernelError::Internal { message }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{
        CreateProjectCommand, CreateSessionCommand, CreateSessionRuntimeBindingCommand,
        GetSessionCommand, GetSessionRuntimeBindingCommand, ListSessionActivitySummariesCommand,
        ListSessionItemsCommand, ListSessionRuntimeBindingsCommand, ListSessionsCommand,
        UpdateSessionCommand,
    };
    use crate::http::AgentHttpState;
    use crate::infrastructure::{
        IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use crate::ports::{
        PaginationParams, SessionActivitySummaryListQuery, SessionItemListQuery, SessionListQuery,
        SessionRuntimeBindingListQuery,
    };
    use crate::{AgentProjectDriveAccessMode, AgentProjectVisibility};
    use sdkwork_agent_kernel::{AgentSession, SessionKind};
    use sdkwork_agents_runtime_facade::AgentEngineCatalogEngine;

    #[test]
    fn classifies_provider_parts_across_all_agent_engine_content_types() {
        use sdkwork_agent_kernel::{AgentMessageRole, AgentPartKind};

        let kind = |role, part_kind, content_type| {
            provider_session_item_kind(role, part_kind, content_type).expect("classified kind")
        };
        // Codex reasoning projection uses summary/content part types; Claude
        // Code and Gemini use reasoning/thinking. All map to Reasoning so the
        // history reconciliation matches the live turn projection.
        for content_type in [
            "reasoning",
            "reasoning_summary",
            "reasoning_content",
            "thinking",
        ] {
            assert_eq!(
                kind(
                    AgentMessageRole::Agent,
                    AgentPartKind::Text,
                    Some(content_type)
                ),
                AgentSessionItemKind::Reasoning,
                "content type {content_type}"
            );
        }
        // Provider-specific tool result content types all map to ToolResult.
        for content_type in [
            "mcp_tool_result",
            "tool_result",
            "bash_code_execution_tool_result",
            "code_execution_tool_result",
            "function_call_output",
            "custom_tool_call_output",
            "web_search_tool_result",
            "collab_agent_tool_result",
        ] {
            assert_eq!(
                kind(
                    AgentMessageRole::Tool,
                    AgentPartKind::Json,
                    Some(content_type)
                ),
                AgentSessionItemKind::ToolResult,
                "content type {content_type}"
            );
        }
        // Tool call refs are ToolCall; text under tool role is a ToolResult.
        assert_eq!(
            kind(AgentMessageRole::Tool, AgentPartKind::ToolCallRef, None),
            AgentSessionItemKind::ToolCall
        );
        assert_eq!(
            kind(AgentMessageRole::Tool, AgentPartKind::Text, None),
            AgentSessionItemKind::ToolResult
        );
        assert_eq!(
            kind(AgentMessageRole::Agent, AgentPartKind::Error, None),
            AgentSessionItemKind::ErrorNotice
        );
        assert_eq!(
            kind(AgentMessageRole::Adapter, AgentPartKind::Text, None),
            AgentSessionItemKind::StatusNotice
        );
        assert_eq!(
            kind(
                AgentMessageRole::User,
                AgentPartKind::Text,
                Some("input_text")
            ),
            AgentSessionItemKind::UserInput
        );
    }

    #[test]
    fn provider_session_ids_are_stable_and_runtime_scoped() {
        let first = stable_provider_session_key(
            100001,
            0,
            42,
            "codex",
            "binding.codex",
            "provider.codex",
            "provider-1",
        );
        assert_eq!(
            first,
            stable_provider_session_key(
                100001,
                0,
                42,
                "codex",
                "binding.codex",
                "provider.codex",
                "provider-1",
            )
        );
        assert_ne!(
            first,
            stable_provider_session_key(
                100001,
                0,
                42,
                "opencode",
                "binding.opencode",
                "provider.opencode",
                "provider-1",
            )
        );
        assert_ne!(
            first,
            stable_provider_session_key(
                100001,
                0,
                43,
                "codex",
                "binding.codex",
                "provider.codex",
                "provider-1",
            )
        );
    }

    #[test]
    fn projects_provider_parts_without_flattening_native_tool_payloads() {
        let native_tool = serde_json::json!({
            "type": "tool",
            "callID": "call-1",
            "tool": "mcp__docs__search",
            "state": {
                "status": "completed",
                "input": { "q": "session items" },
                "output": "found"
            }
        });
        let native_tool_json = native_tool.to_string();
        let mut tool_part = AgentPart::tool_call_ref("part-tool", "call-1")
            .with_name("mcp__docs__search")
            .from_provider("opencode")
            .with_metadata("opencode.content_type", "tool")
            .with_metadata("opencode.status", "completed")
            .with_metadata("opencode.has_result", "true");
        tool_part.json = Some(native_tool_json.clone());
        let message = AgentMessage::new(
            "message-1",
            AgentMessageRole::Agent,
            vec![
                AgentPart::text("part-reasoning", "inspect the code")
                    .from_provider("opencode")
                    .with_metadata("opencode.content_type", "reasoning"),
                tool_part,
                AgentPart::text("part-text", "done")
                    .from_provider("opencode")
                    .with_metadata("opencode.content_type", "text"),
                AgentPart::json(
                    "part-step",
                    serde_json::json!({ "type": "step-finish" }).to_string(),
                )
                .from_provider("opencode")
                .with_metadata("opencode.content_type", "step-finish"),
            ],
        );

        let items = provider_session_history_items("opencode", &message);
        assert_eq!(
            items.iter().map(|item| item.kind).collect::<Vec<_>>(),
            vec![
                AgentSessionItemKind::Reasoning,
                AgentSessionItemKind::ToolCall,
                AgentSessionItemKind::ToolResult,
                AgentSessionItemKind::AssistantOutput,
                AgentSessionItemKind::StatusNotice,
            ]
        );
        assert_eq!(items[1].status, AgentSessionItemStatus::Completed);
        assert_eq!(
            items[1].tool_arguments_json.as_deref(),
            Some(native_tool_json.as_str())
        );
        assert_eq!(items[1].tool_result_json, None);
        assert_eq!(items[2].tool_arguments_json, None);
        assert_eq!(
            items[2].tool_result_json.as_deref(),
            Some(native_tool_json.as_str())
        );
        assert_eq!(items[2].tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(items[3].provider_item_key, "message-1");
        assert_eq!(items[4].content_type, "application/json");
    }

    #[test]
    fn projects_provider_neutral_sdk_part_metadata_before_legacy_namespaces() {
        let message = AgentMessage::new(
            "message-neutral",
            AgentMessageRole::Agent,
            vec![AgentPart::text("part-neutral", "inspect the code")
                .with_metadata("sdkwork.provider.content_type", "reasoning")
                .with_metadata("opencode.content_type", "text")],
        );

        let items = provider_session_history_items("opencode", &message);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, AgentSessionItemKind::Reasoning);
    }

    #[test]
    fn hides_codex_raw_provider_parts_when_semantic_content_is_available() {
        let raw_payload = serde_json::json!({
            "type": "agentMessage",
            "id": "message-codex",
            "text": "finished"
        })
        .to_string();
        let message = AgentMessage::new(
            "message-codex",
            AgentMessageRole::Agent,
            vec![
                AgentPart::text("part-text", "finished")
                    .from_provider("codex")
                    .with_metadata("codex.content_type", "agent_message"),
                AgentPart::json("part-raw", raw_payload)
                    .from_provider("codex")
                    .with_metadata("codex.content_type", "raw_provider_item"),
            ],
        );

        let items = provider_session_history_items("codex", &message);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, AgentSessionItemKind::AssistantOutput);
        assert_eq!(items[0].content.as_deref(), Some("finished"));
    }

    #[test]
    fn preserves_codex_raw_provider_payload_inside_tool_items() {
        let raw_payload = serde_json::json!({
            "type": "commandExecution",
            "id": "command-1",
            "command": "cargo test",
            "aggregatedOutput": "passed",
            "status": "completed"
        })
        .to_string();
        let message = AgentMessage::new(
            "command-1",
            AgentMessageRole::Tool,
            vec![
                AgentPart::tool_call_ref("part-tool", "command-1")
                    .with_name("shell_command")
                    .from_provider("codex")
                    .with_metadata("codex.content_type", "command_execution")
                    .with_metadata("codex.status", "completed"),
                AgentPart::text("part-output", "passed")
                    .from_provider("codex")
                    .with_metadata("codex.content_type", "tool_output")
                    .with_metadata("codex.tool_call_id", "command-1"),
                AgentPart::json("part-raw", raw_payload.clone())
                    .from_provider("codex")
                    .with_metadata("codex.content_type", "raw_provider_item"),
            ],
        );

        let items = provider_session_history_items("codex", &message);

        assert_eq!(
            items.iter().map(|item| item.kind).collect::<Vec<_>>(),
            vec![
                AgentSessionItemKind::ToolCall,
                AgentSessionItemKind::ToolResult,
            ]
        );
        assert_eq!(
            items[0].tool_arguments_json.as_deref(),
            Some(raw_payload.as_str())
        );
        assert_eq!(
            items[1].tool_result_json.as_deref(),
            Some(raw_payload.as_str())
        );
        // The provider now carries the originating call id on the output part,
        // so the sync loop can pair this result with its `ToolCall` item and
        // persist the call → result parent chain.
        assert_eq!(items[1].tool_call_id.as_deref(), Some("command-1"));
        // The full raw provider item JSON is preserved on every projected item.
        assert_eq!(
            items[0].provider_payload_json.as_deref(),
            Some(raw_payload.as_str())
        );
        assert_eq!(
            items[1].provider_payload_json.as_deref(),
            Some(raw_payload.as_str())
        );
    }

    #[test]
    fn codex_mcp_tool_call_result_pairs_through_tool_call_id() {
        let raw_payload = serde_json::json!({
            "type": "mcpToolCall",
            "id": "mcp-1",
            "server": "docs",
            "tool": "search",
            "status": "completed",
            "arguments": {"query": "Codex"},
            "result": {"content": [{"type": "text", "text": "Found 3 docs"}]}
        })
        .to_string();
        let message = AgentMessage::new(
            "mcp-1",
            AgentMessageRole::Tool,
            vec![
                AgentPart::tool_call_ref("part-tool", "mcp-1")
                    .with_name("search")
                    .from_provider("codex")
                    .with_metadata("codex.content_type", "mcp_tool_call")
                    .with_metadata("codex.status", "completed"),
                AgentPart::json(
                    "part-result",
                    serde_json::json!({
                        "result": {"content": [{"type": "text", "text": "Found 3 docs"}]}
                    })
                    .to_string(),
                )
                .from_provider("codex")
                .with_metadata("codex.content_type", "mcp_tool_result")
                .with_metadata("codex.status", "completed")
                .with_metadata("codex.tool_call_id", "mcp-1"),
                AgentPart::json("part-raw", raw_payload.clone())
                    .from_provider("codex")
                    .with_metadata("codex.content_type", "raw_provider_item"),
            ],
        );

        let items = provider_session_history_items("codex", &message);

        assert_eq!(
            items.iter().map(|item| item.kind).collect::<Vec<_>>(),
            vec![
                AgentSessionItemKind::ToolCall,
                AgentSessionItemKind::ToolResult,
            ]
        );
        assert_eq!(items[0].status, AgentSessionItemStatus::Completed);
        assert_eq!(items[1].tool_call_id.as_deref(), Some("mcp-1"));
        assert_eq!(
            items[1].tool_result_json.as_deref(),
            Some(
                serde_json::json!({
                    "result": {"content": [{"type": "text", "text": "Found 3 docs"}]}
                })
                .to_string()
                .as_str()
            )
        );
    }

    #[test]
    fn keeps_raw_provider_payload_as_a_visible_fallback_without_semantic_parts() {
        let raw_payload = serde_json::json!({
            "type": "futureProviderItem",
            "id": "future-1"
        })
        .to_string();
        let message = AgentMessage::new(
            "future-1",
            AgentMessageRole::Agent,
            vec![AgentPart::json("part-raw", raw_payload.clone())
                .from_provider("codex")
                .with_metadata("codex.content_type", "raw_provider_item")],
        );

        let items = provider_session_history_items("codex", &message);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, AgentSessionItemKind::ArtifactReference);
        assert_eq!(items[0].content.as_deref(), Some(raw_payload.as_str()));
        assert_eq!(
            items[0].provider_payload_json.as_deref(),
            Some(raw_payload.as_str())
        );
    }

    #[test]
    fn normalizes_postgres_project_timestamp_for_provider_session_fallback() {
        assert_eq!(
            normalize_provider_session_timestamp("2026-07-27 03:11:00+00").as_deref(),
            Some("2026-07-27T03:11:00+00:00")
        );
        assert_eq!(
            normalize_provider_session_timestamp("2026-07-27 03:11:00.123456-07").as_deref(),
            Some("2026-07-27T03:11:00.123456-07:00")
        );
        assert_eq!(
            normalize_provider_session_timestamp("2026-07-27T03:11:00Z").as_deref(),
            Some("2026-07-27T03:11:00Z")
        );
        assert!(normalize_provider_session_timestamp("not-a-timestamp").is_none());
    }

    #[test]
    fn provider_message_requested_at_normalizes_provider_timestamps() {
        let space_separated = AgentMessage::new("message-1", AgentMessageRole::Agent, Vec::new())
            .created_at("2026-07-27 03:11:00.123456+00");
        assert_eq!(
            provider_message_requested_at(&space_separated, "2026-07-26T00:00:00Z"),
            "2026-07-27T03:11:00.123456+00:00"
        );
        let already_rfc3339 = AgentMessage::new("message-2", AgentMessageRole::Agent, Vec::new())
            .created_at("2026-07-27T03:11:00Z");
        assert_eq!(
            provider_message_requested_at(&already_rfc3339, "2026-07-26T00:00:00Z"),
            "2026-07-27T03:11:00Z"
        );
        let no_message_timestamp =
            AgentMessage::new("message-3", AgentMessageRole::Agent, Vec::new());
        assert_eq!(
            provider_message_requested_at(&no_message_timestamp, "2026-07-26 00:00:00+00"),
            "2026-07-26T00:00:00+00:00"
        );
        let unparsable_message_timestamp =
            AgentMessage::new("message-4", AgentMessageRole::Agent, Vec::new())
                .created_at("not-a-timestamp");
        assert_eq!(
            provider_message_requested_at(&unparsable_message_timestamp, "2026-07-26T00:00:00Z"),
            "2026-07-26T00:00:00Z"
        );
        let both_unparsable = AgentMessage::new("message-5", AgentMessageRole::Agent, Vec::new())
            .created_at("not-a-timestamp");
        let now_fallback = provider_message_requested_at(&both_unparsable, "not-a-timestamp");
        assert!(OffsetDateTime::parse(&now_fallback, &Rfc3339).is_ok());
    }

    #[test]
    fn normalizes_provider_session_titles_to_the_service_limit() {
        assert_eq!(
            provider_session_title(Some("  first\n\tsecond  "), "codex"),
            "first second"
        );
        assert_eq!(
            provider_session_title(Some("   "), "codex"),
            "codex session"
        );

        let long_ascii = "a".repeat(PROVIDER_SESSION_TITLE_MAX_BYTES + 100);
        let ascii_title = provider_session_title(Some(&long_ascii), "codex");
        assert_eq!(ascii_title.len(), PROVIDER_SESSION_TITLE_MAX_BYTES);

        let long_unicode = "\u{4f1a}".repeat(200);
        let unicode_title = provider_session_title(Some(&long_unicode), "codex");
        assert!(unicode_title.len() <= PROVIDER_SESSION_TITLE_MAX_BYTES);
        assert!(unicode_title.is_char_boundary(unicode_title.len()));
    }

    fn test_project(state: &AgentHttpState) -> AgentProjectRecord {
        state
            .service
            .create_project(CreateProjectCommand {
                tenant_id: 100_001,
                organization_id: 0,
                project_id: "project.provider-session-inventory".to_string(),
                workspace_id: None,
                owner_user_id: 100,
                name: "provider-session-inventory".to_string(),
                description: None,
                visibility: AgentProjectVisibility::Private,
                drive_access_mode: AgentProjectDriveAccessMode::Disabled,
                default_agent_id: None,
                default_model_id: None,
                requested_by: PolicySubject {
                    subject_id: "100".to_string(),
                    tenant_id: "100001".to_string(),
                    roles: vec![
                        "ai.agents.manage".to_string(),
                        "ai.agents.read".to_string(),
                        "ai.agents.use".to_string(),
                    ],
                },
                requested_at: "2026-07-26T00:00:00Z".to_string(),
            })
            .expect("test project")
    }

    fn read_subject() -> PolicySubject {
        PolicySubject {
            subject_id: "100".to_string(),
            tenant_id: "100001".to_string(),
            roles: vec!["ai.agents.read".to_string()],
        }
    }

    fn inventory_item(
        engine: &AgentEngineCatalogEngine,
        provider_session_id: String,
        ordinal: usize,
    ) -> ProviderSessionInventoryItem {
        let default_model = engine.models.first().expect("engine default model");
        let timestamp = format!("2026-07-26T00:{:02}:00Z", ordinal % 60);
        let mut session = AgentSession::new(provider_session_id)
            .with_title(format!("{} provider session {ordinal}", engine.engine_key))
            .with_model(default_model.model_id.clone())
            .with_cwd(r"E:\sdkwork-space\sdkwork-birdcoder");
        session.created_at = Some(timestamp.clone());
        session.updated_at = Some(timestamp);
        let directory = sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry {
            title: session.title.clone(),
            title_source: Some("provider".to_string()),
            preview: session.preview.clone(),
            created_at: session.created_at.clone(),
            updated_at: session.updated_at.clone(),
            recency_at: session.updated_at.clone(),
            pinned: false,
            archived: false,
            visible: true,
            source: None,
            sort_key: session.session_id.clone(),
        };
        ProviderSessionInventoryItem {
            engine_key: engine.engine_key.clone(),
            agent_id: engine.agent_id.clone(),
            binding_id: engine.binding_id.clone(),
            provider_id: default_model.provider_id.clone(),
            default_model_id: default_model.model_id.clone(),
            directory,
            session,
        }
    }

    fn current_provider_binding(
        state: &AgentHttpState,
        project: &AgentProjectRecord,
        engine_key: &str,
    ) -> crate::domain::AgentSessionRuntimeBindingRecord {
        let agent_id = sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
            .unwrap_or_else(|| panic!("missing canonical agent id for {engine_key}"));
        let session = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id.clone())
                    .for_agent(agent_id)
                    .include_archived(),
                requested_by: read_subject(),
            })
            .expect("provider session list")
            .items
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("missing synchronized {engine_key} session"));
        state
            .service
            .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                query: SessionRuntimeBindingListQuery::for_session(
                    project.tenant_id,
                    project.organization_id,
                    session.session_id,
                )
                .current_only()
                .with_pagination(PaginationParams::default().with_page_size(1)),
                path_agent_id: agent_id.to_string(),
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("current provider binding")
            .items
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("missing current {engine_key} provider binding"))
    }

    #[test]
    fn successful_resolved_empty_snapshot_invalidates_missing_provider_sessions() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let codex = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![inventory_item(codex, "codex-missing".to_string(), 1)],
        )
        .expect("seed provider session");

        let result = synchronize_provider_session_snapshot(
            state.service.clone(),
            &project,
            read_subject(),
            ProviderSessionInventorySnapshot {
                directory_resolved: true,
                items: Vec::new(),
                successful_engine_keys: vec!["codex".to_string()],
                issues: Vec::new(),
                unattributed_provider_sessions: false,
            },
        )
        .expect("reconcile empty successful snapshot");

        assert_eq!(result.synchronized_session_count, 1);
        assert_eq!(result.skipped_session_count, 0);
        assert_eq!(result.failed_session_count, 0);
        let binding = current_provider_binding(&state, &project, "codex");
        assert!(!binding.provider_visible);
        assert!(binding.provider_archived);
        assert!(!binding.provider_pinned);
        assert_eq!(binding.version, 1);
    }

    #[test]
    fn failed_provider_snapshot_does_not_invalidate_previous_directory() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let codex = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![inventory_item(codex, "codex-retained".to_string(), 1)],
        )
        .expect("seed provider session");

        let result = synchronize_provider_session_snapshot(
            state.service.clone(),
            &project,
            read_subject(),
            ProviderSessionInventorySnapshot {
                directory_resolved: true,
                items: Vec::new(),
                successful_engine_keys: Vec::new(),
                issues: vec![
                    sdkwork_agents_runtime_facade::ProviderSessionInventoryIssue {
                        engine_key: "codex".to_string(),
                        reason: "fixture provider failure".to_string(),
                    },
                ],
                unattributed_provider_sessions: false,
            },
        )
        .expect("record failed provider snapshot");

        assert_eq!(result.failed_session_count, 1);
        let binding = current_provider_binding(&state, &project, "codex");
        assert!(binding.provider_visible);
        assert!(!binding.provider_archived);
        assert_eq!(binding.version, 0);
    }

    #[test]
    fn mixed_snapshot_invalidates_only_successful_provider_rows() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = |key: &str| {
            catalog
                .engines
                .iter()
                .find(|engine| engine.engine_key == key)
                .unwrap_or_else(|| panic!("missing {key} engine"))
        };
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![
                inventory_item(engine("codex"), "codex-old".to_string(), 1),
                inventory_item(engine("opencode"), "opencode-old".to_string(), 2),
            ],
        )
        .expect("seed mixed provider sessions");

        synchronize_provider_session_snapshot(
            state.service.clone(),
            &project,
            read_subject(),
            ProviderSessionInventorySnapshot {
                directory_resolved: true,
                items: Vec::new(),
                successful_engine_keys: vec!["codex".to_string()],
                issues: vec![
                    sdkwork_agents_runtime_facade::ProviderSessionInventoryIssue {
                        engine_key: "opencode".to_string(),
                        reason: "fixture provider failure".to_string(),
                    },
                ],
                unattributed_provider_sessions: false,
            },
        )
        .expect("reconcile mixed provider snapshot");

        let codex_binding = current_provider_binding(&state, &project, "codex");
        assert!(!codex_binding.provider_visible);
        assert!(codex_binding.provider_archived);
        let opencode_binding = current_provider_binding(&state, &project, "opencode");
        assert!(opencode_binding.provider_visible);
        assert!(!opencode_binding.provider_archived);
    }

    #[test]
    fn unresolved_directory_does_not_invalidate_successful_provider_rows() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let codex = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![inventory_item(codex, "codex-unresolved".to_string(), 1)],
        )
        .expect("seed provider session");

        synchronize_provider_session_snapshot(
            state.service.clone(),
            &project,
            read_subject(),
            ProviderSessionInventorySnapshot {
                directory_resolved: false,
                items: Vec::new(),
                successful_engine_keys: vec!["codex".to_string()],
                issues: Vec::new(),
                unattributed_provider_sessions: false,
            },
        )
        .expect("ignore unresolved provider directory");

        let binding = current_provider_binding(&state, &project, "codex");
        assert!(binding.provider_visible);
        assert!(!binding.provider_archived);
        assert_eq!(binding.version, 0);
    }

    #[test]
    fn synchronizes_complete_multi_provider_inventory_across_session_pages() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let subject = read_subject();
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = |key: &str| {
            catalog
                .engines
                .iter()
                .find(|engine| engine.engine_key == key)
                .unwrap_or_else(|| panic!("missing {key} engine"))
        };
        let mut inventory = (0..55)
            .map(|index| inventory_item(engine("codex"), format!("codex-{index}"), index))
            .collect::<Vec<_>>();
        inventory.push(inventory_item(
            engine("claude-code"),
            "claude-code-1".to_string(),
            55,
        ));
        inventory.push(inventory_item(
            engine("opencode"),
            "opencode-1".to_string(),
            56,
        ));

        let synchronized = synchronize_provider_session_inventory_with_timeout(
            state.service.clone(),
            &project,
            subject.clone(),
            inventory.clone(),
            Duration::MAX,
        )
        .expect("complete provider inventory sync");
        assert_eq!(synchronized.synchronized_session_count, 57);
        assert_eq!(
            synchronize_provider_session_inventory_with_timeout(
                state.service.clone(),
                &project,
                subject.clone(),
                inventory,
                Duration::MAX,
            )
            .expect("idempotent provider inventory replay")
            .synchronized_session_count,
            57,
        );

        let list_first_page = |cursor| {
            state
                .service
                .list_sessions(ListSessionsCommand {
                    query: SessionListQuery::for_tenant(project.tenant_id)
                        .for_organization(project.organization_id)
                        .for_owner(project.owner_user_id)
                        .for_project(project.project_id.clone())
                        .with_cursor_page(50, cursor),
                    requested_by: subject.clone(),
                })
                .expect("provider session page")
        };
        let first_page = list_first_page(None);
        assert_eq!(first_page.items.len(), 50);
        assert!(first_page.has_more);
        assert_eq!(first_page.total_count, None);
        let second_page = list_first_page(
            first_page
                .next_page_token
                .as_deref()
                .map(decode_session_list_cursor)
                .transpose()
                .expect("decode provider Session cursor"),
        );
        assert_eq!(second_page.items.len(), 7);
        assert!(!second_page.has_more);
        assert_eq!(second_page.total_count, None);

        let activity_query = SessionActivitySummaryListQuery::for_owner(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
        )
        .for_project(project.project_id.clone())
        .with_page_size(50);
        let first_activity_page = state
            .service
            .list_session_activity_summaries(ListSessionActivitySummariesCommand {
                query: activity_query.clone(),
                requested_by: subject.clone(),
            })
            .expect("first synchronized provider Session activity page");
        assert_eq!(first_activity_page.items.len(), 50);
        assert!(first_activity_page.has_more);
        let activity_cursor = crate::session_activity::decode_session_activity_cursor(
            first_activity_page
                .next_page_token
                .as_deref()
                .expect("provider Session activity cursor"),
        )
        .expect("decode provider Session activity cursor");
        let second_activity_page = state
            .service
            .list_session_activity_summaries(ListSessionActivitySummariesCommand {
                query: activity_query.after(activity_cursor),
                requested_by: subject.clone(),
            })
            .expect("second synchronized provider Session activity page");
        assert_eq!(second_activity_page.items.len(), 7);
        assert!(!second_activity_page.has_more);

        let synchronized_session_ids = first_page
            .items
            .iter()
            .chain(second_page.items.iter())
            .map(|session| session.session_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let activity_session_ids = first_activity_page
            .items
            .iter()
            .chain(second_activity_page.items.iter())
            .map(|summary| summary.session.session_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(activity_session_ids, synchronized_session_ids);

        for engine_key in ["codex", "claude-code", "opencode"] {
            let session = first_page
                .items
                .iter()
                .chain(second_page.items.iter())
                .find(|session| session.agent_id == engine(engine_key).agent_id)
                .unwrap_or_else(|| panic!("missing synchronized {engine_key} session"));
            let bindings = state
                .service
                .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                    query: SessionRuntimeBindingListQuery::for_session(
                        project.tenant_id,
                        project.organization_id,
                        session.session_id.clone(),
                    ),
                    path_agent_id: session.agent_id.clone(),
                    owner_scope: Some(project.owner_user_id),
                    requested_by: subject.clone(),
                })
                .expect("provider Session runtime binding");
            let binding = bindings
                .items
                .first()
                .expect("current provider Session binding");
            assert_eq!(binding.provider_binding_id, engine(engine_key).binding_id);
            assert_eq!(
                binding.provider_id,
                engine(engine_key).models[0].provider_id
            );
            assert!(binding.provider_session_id.is_some());
        }
    }

    #[test]
    fn concurrent_provider_session_inventory_refreshes_are_idempotent() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let inventory = ["codex", "claude-code", "opencode"]
            .into_iter()
            .enumerate()
            .map(|(index, key)| {
                let engine = catalog
                    .engines
                    .iter()
                    .find(|engine| engine.engine_key == key)
                    .unwrap_or_else(|| panic!("missing {key} engine"));
                inventory_item(engine, format!("{key}-concurrent"), index)
            })
            .collect::<Vec<_>>();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let workers = (0..2)
            .map(|_| {
                let service = state.service.clone();
                let project = project.clone();
                let inventory = inventory.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    synchronize_provider_session_inventory(
                        service,
                        &project,
                        read_subject(),
                        inventory,
                    )
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            assert_eq!(
                worker
                    .join()
                    .expect("refresh worker")
                    .expect("refresh")
                    .synchronized_session_count,
                3
            );
        }

        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("concurrent inventory result");
        assert_eq!(sessions.items.len(), 3);
    }

    #[test]
    fn provider_inventory_deduplicates_normalized_identity_and_skips_subagents() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let root = inventory_item(engine, " provider-session-1 ".to_string(), 1);
        let mut duplicate = root.clone();
        duplicate.session.session_id = "provider-session-1".to_string();
        duplicate.session.title = Some("Duplicate title must not create a row".to_string());
        let mut subagent = inventory_item(engine, "provider-subagent-1".to_string(), 2);
        subagent.session.kind = SessionKind::Subagent;
        subagent.session.parent_session_id = Some("provider-session-1".to_string());

        let synchronization = synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![root, duplicate, subagent],
        )
        .expect("normalized provider inventory sync");
        assert_eq!(
            synchronization.synchronized_session_count,
            2,
            "issues: {:?}",
            synchronization
                .issues
                .iter()
                .map(|issue| (issue.code, format!("{:?}", issue.disposition)))
                .collect::<Vec<_>>()
        );
        assert_eq!(synchronization.skipped_session_count, 1);
        assert_eq!(synchronization.failed_session_count, 0);

        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id.clone()),
                requested_by: read_subject(),
            })
            .expect("deduplicated provider sessions");
        assert_eq!(sessions.items.len(), 2);
        let root_session = sessions
            .items
            .iter()
            .find(|session| session.parent_session_id.is_none())
            .expect("root provider session");
        let subagent_session = sessions
            .items
            .iter()
            .find(|session| session.parent_session_id.is_some())
            .expect("subagent provider session");
        // The canonical session tree mirrors the provider sub-agent topology:
        // the sub-agent session records the root session as its parent.
        assert_eq!(
            subagent_session.parent_session_id.as_deref(),
            Some(root_session.session_id.as_str())
        );
        let subagent_bindings = state
            .service
            .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                query: SessionRuntimeBindingListQuery::for_session(
                    project.tenant_id,
                    project.organization_id,
                    subagent_session.session_id.clone(),
                ),
                path_agent_id: subagent_session.agent_id.clone(),
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("subagent provider binding");
        let subagent_binding = subagent_bindings.items.first().expect("subagent binding");
        assert_eq!(
            subagent_binding.provider_parent_session_id.as_deref(),
            Some("provider-session-1")
        );
        assert_eq!(
            subagent_binding.provider_session_id.as_deref(),
            Some("provider-subagent-1")
        );
        let session = root_session;
        let bindings = state
            .service
            .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                query: SessionRuntimeBindingListQuery::for_session(
                    project.tenant_id,
                    project.organization_id,
                    session.session_id.clone(),
                ),
                path_agent_id: session.agent_id.clone(),
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("normalized provider binding");
        let binding = bindings.items.first().expect("provider binding");
        assert_eq!(
            binding.provider_session_id.as_deref(),
            Some("provider-session-1")
        );
        assert_eq!(
            binding.provider_session_tree_id.as_deref(),
            Some("provider-session-1")
        );
    }

    #[test]
    fn provider_inventory_reports_invalid_items_without_aborting_valid_reconciliation() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let valid = inventory_item(engine, "provider-session-valid".to_string(), 1);
        let mut invalid = inventory_item(engine, "provider-session-invalid".to_string(), 2);
        invalid.provider_id = " ".to_string();

        let synchronization = synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![invalid, valid],
        )
        .expect("dirty inventory item must not abort valid reconciliation");

        assert_eq!(synchronization.synchronized_session_count, 1);
        assert_eq!(synchronization.skipped_session_count, 0);
        assert_eq!(synchronization.failed_session_count, 1);
        assert_eq!(
            synchronization.issues,
            vec![ProviderSessionSynchronizationIssue {
                code: "invalid_provider_session_identity",
                count: 1,
                disposition: ProviderSessionSynchronizationIssueDisposition::Failed,
            }]
        );
    }

    #[test]
    fn repeated_provider_session_inventory_sync_updates_the_provider_title() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let mut item = inventory_item(engine, "codex-renamed".to_string(), 1);
        item.session.title = Some("Initial provider title".to_string());
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item.clone()],
        )
        .expect("initial provider inventory sync");

        item.session.title = Some("Renamed provider title".to_string());
        item.directory.title = item.session.title.clone();
        item.directory.recency_at = Some("2026-07-26T00:02:00Z".to_string());
        item.directory.updated_at = item.directory.recency_at.clone();
        item.directory.pinned = true;
        item.directory.sort_key = "codex-renamed-refreshed".to_string();
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item],
        )
        .expect("renamed provider inventory sync");

        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("renamed provider session");
        assert_eq!(sessions.items.len(), 1);
        assert_eq!(
            sessions.items[0].title.as_deref(),
            Some("Renamed provider title")
        );
        let binding = state
            .service
            .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                query: SessionRuntimeBindingListQuery::for_session(
                    project.tenant_id,
                    project.organization_id,
                    sessions.items[0].session_id.clone(),
                )
                .current_only()
                .with_pagination(PaginationParams::default().with_page_size(1)),
                path_agent_id: sessions.items[0].agent_id.clone(),
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("current provider binding")
            .items
            .into_iter()
            .next()
            .expect("provider binding exists");
        assert_eq!(
            binding.provider_title.as_deref(),
            Some("Renamed provider title")
        );
        assert_eq!(
            binding.provider_recency_at.as_deref(),
            Some("2026-07-26T00:02:00Z")
        );
        assert!(binding.provider_pinned);
        assert_eq!(
            binding.provider_sort_key.as_deref(),
            Some("codex-renamed-refreshed")
        );
        assert_eq!(binding.version, 1);
    }

    #[test]
    fn provider_inventory_never_overwrites_a_user_renamed_session_title() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let mut item = inventory_item(engine, "codex-user-title".to_string(), 1);
        item.session.title = Some("Provider title".to_string());
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item.clone()],
        )
        .expect("initial provider inventory sync");

        let session = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id.clone()),
                requested_by: read_subject(),
            })
            .expect("provider session")
            .items
            .into_iter()
            .next()
            .expect("one provider session");
        let renamed = state
            .service
            .update_session(UpdateSessionCommand {
                tenant_id: project.tenant_id,
                organization_id: project.organization_id,
                path_agent_id: session.agent_id.clone(),
                session_id: session.session_id.clone(),
                title: Some("User-owned title".to_string()),
                project_id: None,
                expected_version: Some(session.version),
                owner_scope: Some(project.owner_user_id),
                requested_by: PolicySubject {
                    subject_id: "100".to_string(),
                    tenant_id: "100001".to_string(),
                    roles: vec!["ai.agents.use".to_string()],
                },
                requested_at: "2026-07-27T12:00:00Z".to_string(),
            })
            .expect("user rename");
        assert_eq!(
            renamed.title_source,
            crate::domain::AgentSessionTitleSource::User
        );

        item.session.title = Some("Provider title after user rename".to_string());
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item],
        )
        .expect("provider inventory refresh");

        let refreshed = state
            .service
            .get_session(crate::application::GetSessionCommand {
                tenant_id: project.tenant_id,
                organization_id: project.organization_id,
                path_agent_id: session.agent_id,
                session_id: session.session_id,
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("refreshed provider session");
        assert_eq!(refreshed.title.as_deref(), Some("User-owned title"));
        assert_eq!(
            refreshed.title_source,
            crate::domain::AgentSessionTitleSource::User
        );
    }

    #[test]
    fn synchronizes_inventory_without_provider_timestamp_from_postgres_project_time() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let mut project = test_project(&state);
        project.created_at = "2026-07-27 03:10:00+00".to_string();
        project.updated_at = "2026-07-27 03:11:00+00".to_string();
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let mut item = inventory_item(engine, "codex-without-time".to_string(), 0);
        item.session.created_at = None;
        item.session.updated_at = None;

        assert_eq!(
            synchronize_provider_session_inventory(
                state.service.clone(),
                &project,
                read_subject(),
                vec![item],
            )
            .expect("PostgreSQL project time fallback")
            .synchronized_session_count,
            1
        );
        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("synchronized Session list");
        assert_eq!(sessions.items.len(), 1);
        assert_eq!(sessions.items[0].created_at, "2026-07-27T03:11:00+00:00");
    }

    #[test]
    fn provider_session_transcript_items_are_idempotent_and_readable() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![inventory_item(
                engine,
                "provider-session-transcript-1".to_string(),
                1,
            )],
        )
        .expect("provider inventory sync");
        let session = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("provider sessions")
            .items
            .into_iter()
            .next()
            .expect("provider session");
        let item_id =
            stable_provider_session_item_id("codex", "provider-session-transcript-1", "message-1");
        let command = ReconcileProviderSessionHistoryItemCommand {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            session_id: session.session_id.clone(),
            item_id: item_id.clone(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: Some("provider partial response".to_string()),
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            model_id: Some(engine.models[0].model_id.clone()),
            provider_id: Some(engine.models[0].provider_id.clone()),
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: None,
            requested_by: read_subject(),
            requested_at: "2026-07-26T00:01:00Z".to_string(),
        };
        state
            .service
            .reconcile_provider_session_history_session_item(command.clone(), "codex")
            .expect("provider transcript item");
        state
            .service
            .reconcile_provider_session_history_session_item(command.clone(), "codex")
            .expect("idempotent provider transcript replay");
        let corrected = state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    content: Some("provider user message final".to_string()),
                    requested_at: "2026-07-26T00:01:01Z".to_string(),
                    ..command.clone()
                },
                "codex",
            )
            .expect("newer provider narrative snapshot");
        assert_eq!(corrected.version, 1);
        assert_eq!(
            corrected.content.as_deref(),
            Some("provider user message final")
        );
        let stale_correction = state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    content: Some("stale partial".to_string()),
                    requested_at: "2026-07-26T00:01:00Z".to_string(),
                    ..command
                },
                "codex",
            );
        assert!(stale_correction.is_err());
        let tool_item_id = stable_provider_session_item_id(
            "codex",
            "provider-session-transcript-1",
            "tool-message-1\u{0}tool-part-1",
        );
        let pending_tool = serde_json::json!({
            "type": "function_call",
            "id": "provider-tool-item-1",
            "call_id": "provider-tool-call-1",
            "name": "shell_command",
            "arguments": "{\"command\":\"cargo test\"}"
        })
        .to_string();
        let tool_command = ReconcileProviderSessionHistoryItemCommand {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            session_id: session.session_id.clone(),
            item_id: tool_item_id.clone(),
            kind: AgentSessionItemKind::ToolCall,
            content: None,
            content_type: "application/json".to_string(),
            status: AgentSessionItemStatus::Pending,
            model_id: Some(engine.models[0].model_id.clone()),
            provider_id: Some(engine.models[0].provider_id.clone()),
            tool_name: Some("shell_command".to_string()),
            tool_call_id: Some("provider-tool-call-1".to_string()),
            tool_arguments_json: Some(pending_tool),
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: None,
            requested_by: read_subject(),
            requested_at: "2026-07-26T00:02:00Z".to_string(),
        };
        state
            .service
            .reconcile_provider_session_history_session_item(tool_command.clone(), "codex")
            .expect("pending provider tool item");
        let completed_tool_call = serde_json::json!({
            "type": "function_call",
            "id": "provider-tool-item-1",
            "call_id": "provider-tool-call-1",
            "name": "shell_command",
            "arguments": "{\"command\":\"cargo test\"}",
            "status": "completed"
        })
        .to_string();
        let completed = state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    status: AgentSessionItemStatus::Completed,
                    tool_arguments_json: Some(completed_tool_call.clone()),
                    requested_at: "2026-07-26T00:03:00Z".to_string(),
                    ..tool_command
                },
                "codex",
            )
            .expect("completed provider tool item");
        assert_eq!(completed.version, 1);
        assert_eq!(completed.status, AgentSessionItemStatus::Completed);
        assert_eq!(completed.tool_result_json, None);
        let immutable_replay = state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    tenant_id: project.tenant_id,
                    organization_id: project.organization_id,
                    session_id: session.session_id.clone(),
                    item_id: tool_item_id.clone(),
                    kind: AgentSessionItemKind::ToolCall,
                    content: None,
                    content_type: "application/json".to_string(),
                    status: AgentSessionItemStatus::Completed,
                    model_id: Some(engine.models[0].model_id.clone()),
                    provider_id: Some(engine.models[0].provider_id.clone()),
                    tool_name: Some("shell_command".to_string()),
                    tool_call_id: Some("provider-tool-call-1".to_string()),
                    tool_arguments_json: Some(
                        serde_json::json!({
                            "type": "function_call",
                            "id": "provider-tool-item-1",
                            "call_id": "provider-tool-call-1",
                            "name": "shell_command",
                            "arguments": "{\"command\":\"cargo check\"}",
                            "status": "completed"
                        })
                        .to_string(),
                    ),
                    tool_result_json: None,
                    provider_payload_json: None,
                    parent_item_id: None,
                    requested_by: read_subject(),
                    requested_at: "2026-07-26T00:04:00Z".to_string(),
                },
                "codex",
            );
        assert!(immutable_replay.is_err());
        let tool_result_id = format!("{tool_item_id}.result");
        state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    tenant_id: project.tenant_id,
                    organization_id: project.organization_id,
                    session_id: session.session_id.clone(),
                    item_id: tool_result_id,
                    kind: AgentSessionItemKind::ToolResult,
                    content: None,
                    content_type: "application/json".to_string(),
                    status: AgentSessionItemStatus::Completed,
                    model_id: Some(engine.models[0].model_id.clone()),
                    provider_id: Some(engine.models[0].provider_id.clone()),
                    tool_name: Some("shell_command".to_string()),
                    tool_call_id: Some("provider-tool-call-1".to_string()),
                    tool_arguments_json: None,
                    tool_result_json: Some(
                        serde_json::json!({ "output": "ok", "status": "completed" }).to_string(),
                    ),
                    provider_payload_json: None,
                    parent_item_id: Some(tool_item_id.clone()),
                    requested_by: read_subject(),
                    requested_at: "2026-07-26T00:04:00Z".to_string(),
                },
                "codex",
            )
            .expect("provider tool result item");
        let items = state
            .service
            .list_session_items(ListSessionItemsCommand {
                query: SessionItemListQuery::for_session(
                    project.tenant_id,
                    project.organization_id,
                    session.session_id,
                ),
                path_agent_id: session.agent_id,
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("provider transcript items");
        assert_eq!(items.total_count, Some(3));
        let narrative_item = items
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .expect("provider narrative item");
        assert_eq!(
            narrative_item.content.as_deref(),
            Some("provider user message final")
        );
        assert_eq!(narrative_item.version, 1);
        let tool_item = items
            .items
            .iter()
            .find(|item| item.item_id == tool_item_id)
            .expect("provider tool item");
        assert_eq!(tool_item.status, AgentSessionItemStatus::Completed);
        assert_eq!(tool_item.version, 1);
    }

    #[test]
    fn provider_transcript_item_repairs_legacy_non_rfc3339_updated_at() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![inventory_item(
                engine,
                "provider-session-legacy-timestamp".to_string(),
                0,
            )],
        )
        .expect("provider inventory sync");
        let session = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("provider sessions")
            .items
            .into_iter()
            .next()
            .expect("provider session");
        let item_id = stable_provider_session_item_id(
            "codex",
            "provider-session-legacy-timestamp",
            "message-1",
        );
        let command = ReconcileProviderSessionHistoryItemCommand {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            session_id: session.session_id.clone(),
            item_id: item_id.clone(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: Some("provider legacy narrative".to_string()),
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            model_id: Some(engine.models[0].model_id.clone()),
            provider_id: Some(engine.models[0].provider_id.clone()),
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: None,
            requested_by: read_subject(),
            // Legacy transcript synchronizations persisted provider
            // timestamps verbatim without RFC3339 normalization.
            requested_at: "2026-07-26 00:01:00.000+00".to_string(),
        };
        let stored = state
            .service
            .reconcile_provider_session_history_session_item(command.clone(), "codex")
            .expect("legacy provider transcript item");
        assert_eq!(stored.updated_at, "2026-07-26 00:01:00.000+00");
        // A newer terminal narrative snapshot must repair the legacy
        // timestamp instead of failing the whole synchronization.
        let repaired = state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    content: Some("provider narrative final".to_string()),
                    requested_at: "2026-07-26T00:02:00Z".to_string(),
                    ..command.clone()
                },
                "codex",
            )
            .expect("newer provider narrative snapshot repairs legacy timestamp");
        assert_eq!(repaired.version, 1);
        assert_eq!(repaired.updated_at, "2026-07-26T00:02:00Z");
        assert_eq!(
            repaired.content.as_deref(),
            Some("provider narrative final")
        );
        // Terminal history stays immutable once the timestamp is repaired.
        let stale_replay = state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    content: Some("stale replay".to_string()),
                    requested_at: "2026-07-26T00:01:30Z".to_string(),
                    ..command
                },
                "codex",
            );
        assert!(stale_replay.is_err());
    }

    fn manage_subject() -> PolicySubject {
        PolicySubject {
            subject_id: "100".to_string(),
            tenant_id: "100001".to_string(),
            roles: vec![
                "ai.agents.manage".to_string(),
                "ai.agents.read".to_string(),
                "ai.agents.use".to_string(),
            ],
        }
    }

    fn create_test_session(
        state: &AgentHttpState,
        session_id: &str,
        project_id: &str,
        source_context_kind: &str,
    ) -> crate::domain::AgentSessionRecord {
        state
            .service
            .create_session(CreateSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: "agent.codex".to_string(),
                owner_user_id: 100,
                project_id: Some(project_id.to_string()),
                session_id: session_id.to_string(),
                session_kind: crate::domain::AgentSessionKind::Coding,
                entry_surface: crate::domain::AgentSessionEntrySurface::Pc,
                source_module: Some("birdcoder".to_string()),
                source_context_kind: Some(source_context_kind.to_string()),
                source_context_id: Some(project_id.to_string()),
                parent_session_id: None,
                forked_from_turn_id: None,
                title: Some("legacy".to_string()),
                idempotency_key: None,
                payload_hash: None,
                requested_by: manage_subject(),
                requested_at: "2026-07-26T00:00:00Z".to_string(),
            })
            .expect("test session")
    }

    fn create_test_binding(
        state: &AgentHttpState,
        session_id: &str,
        runtime_binding_id: &str,
        transport_kind: &str,
        provider_session_id: Option<&str>,
    ) -> crate::domain::AgentSessionRuntimeBindingRecord {
        state
            .service
            .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.codex".to_string(),
                session_id: session_id.to_string(),
                runtime_binding_id: Some(runtime_binding_id.to_string()),
                runtime_location_id: None,
                host_mode: "server".to_string(),
                transport_kind: transport_kind.to_string(),
                provider_binding_id: "binding.codex".to_string(),
                model_id: "codex-1".to_string(),
                provider_id: "provider.codex".to_string(),
                provider_session_id: provider_session_id.map(str::to_string),
                provider_session_tree_id: provider_session_id.map(str::to_string),
                provider_parent_session_id: None,
                provider_forked_from_session_id: None,
                provider_directory: None,
                owner_scope: Some(100),
                requested_by: manage_subject(),
                requested_at: "2026-07-26T00:00:00Z".to_string(),
            })
            .expect("test binding")
    }

    #[test]
    fn retires_legacy_native_binding_before_provider_import() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let legacy = create_test_session(
            &state,
            "session.native.codex.legacy1",
            &project.project_id,
            "provider_session",
        );
        create_test_binding(
            &state,
            &legacy.session_id,
            "runtime_binding.native.legacy1",
            "native-history",
            Some("provider-thread-legacy-1"),
        );
        let outcome = state
            .service
            .retire_legacy_provider_session_bindings(
                100_001,
                0,
                100,
                "codex",
                "binding.codex",
                "provider-thread-legacy-1",
                "session.codex.target1",
                &project.project_id,
                manage_subject(),
                "2026-07-26T00:00:00Z",
            )
            .expect("legacy retirement");
        assert!(matches!(
            outcome,
            crate::application::ProviderSessionBindingClaim::Retired
        ));
        let retired = state
            .service
            .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.codex".to_string(),
                session_id: legacy.session_id.clone(),
                runtime_binding_id: "runtime_binding.native.legacy1".to_string(),
                owner_scope: Some(100),
                requested_by: manage_subject(),
            })
            .expect("retired binding lookup");
        assert_eq!(retired.provider_session_id, None);
        assert_eq!(
            retired.status,
            crate::domain::AgentSessionRuntimeBindingStatus::Deactivated
        );
        let archived = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.codex".to_string(),
                session_id: legacy.session_id.clone(),
                owner_scope: Some(100),
                requested_by: manage_subject(),
            })
            .expect("archived session lookup");
        assert_eq!(archived.status, crate::domain::AgentSessionStatus::Archived);
    }

    #[test]
    fn retires_stale_provider_import_that_claims_the_same_thread() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        // A Session imported under the legacy `session.provider.*` scheme is
        // retired when the canonical `session.codex.*` import claims the same
        // provider thread identity.
        let stale = create_test_session(
            &state,
            "session.provider.codex.stale-scheme-1",
            &project.project_id,
            "provider_session",
        );
        create_test_binding(
            &state,
            &stale.session_id,
            "runtime_binding.provider.stale-scheme-1",
            "provider-session-history",
            Some("provider-thread-stale-1"),
        );
        let outcome = state
            .service
            .retire_legacy_provider_session_bindings(
                100_001,
                0,
                100,
                "codex",
                "binding.codex",
                "provider-thread-stale-1",
                "session.codex.canonical-1",
                &project.project_id,
                manage_subject(),
                "2026-07-26T00:00:00Z",
            )
            .expect("stale import retirement");
        assert!(matches!(
            outcome,
            crate::application::ProviderSessionBindingClaim::Retired
        ));
    }

    #[test]
    fn never_retires_user_created_session_binding() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let user_session = create_test_session(
            &state,
            "session.340000000000000000",
            &project.project_id,
            "coding-workbench",
        );
        create_test_binding(
            &state,
            &user_session.session_id,
            "runtime_binding.user.1",
            "provider-session-history",
            Some("provider-thread-user-1"),
        );
        let outcome = state
            .service
            .retire_legacy_provider_session_bindings(
                100_001,
                0,
                100,
                "codex",
                "binding.codex",
                "provider-thread-user-1",
                "session.codex.canonical-2",
                &project.project_id,
                manage_subject(),
                "2026-07-26T00:00:00Z",
            )
            .expect("user session claim");
        assert!(matches!(
            outcome,
            crate::application::ProviderSessionBindingClaim::AlreadyBoundByUserSession
        ));
        let still_active = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.codex".to_string(),
                session_id: user_session.session_id.clone(),
                owner_scope: Some(100),
                requested_by: manage_subject(),
            })
            .expect("user session lookup");
        assert_eq!(
            still_active.status,
            crate::domain::AgentSessionStatus::Active
        );
    }

    #[test]
    fn target_binding_is_reported_as_already_claimed() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let canonical = create_test_session(
            &state,
            "session.codex.canonical-3",
            &project.project_id,
            "provider_session",
        );
        create_test_binding(
            &state,
            &canonical.session_id,
            "runtime_binding.codex.canonical-3",
            "provider-session-history",
            Some("provider-thread-target-1"),
        );
        let outcome = state
            .service
            .retire_legacy_provider_session_bindings(
                100_001,
                0,
                100,
                "codex",
                "binding.codex",
                "provider-thread-target-1",
                &canonical.session_id,
                &project.project_id,
                manage_subject(),
                "2026-07-26T00:00:00Z",
            )
            .expect("target claim");
        assert!(matches!(
            outcome,
            crate::application::ProviderSessionBindingClaim::AlreadyTarget
        ));
    }

    #[test]
    fn clamps_oversized_provider_directory_fields() {
        use sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry;
        let long_title = "t".repeat(600);
        let long_preview = "p".repeat(5_000);
        let directory = clamp_provider_session_directory(ProviderSessionDirectoryEntry {
            title: Some(long_title),
            title_source: Some("provider".to_string()),
            preview: Some(long_preview),
            created_at: None,
            updated_at: None,
            recency_at: None,
            pinned: false,
            archived: false,
            visible: true,
            source: Some("codex".to_string()),
            sort_key: "s".repeat(600),
        });
        assert!(directory.title.as_deref().unwrap_or_default().len() <= 512);
        assert!(directory.preview.as_deref().unwrap_or_default().len() <= 4096);
        assert!(directory.sort_key.len() <= 512);
    }

    #[test]
    fn transcript_sync_reports_skipped_outcomes_instead_of_silent_zero() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let subject = manage_subject();
        // A live agent Session is not a provider-history Session.
        let live = create_test_session(
            &state,
            "session.340000000000000000",
            &project.project_id,
            "coding-workbench",
        );
        let outcome = synchronize_provider_session_transcript(
            &state.service,
            100_001,
            0,
            100,
            live.agent_id.clone(),
            live.session_id.clone(),
            subject.clone(),
            None,
        )
        .expect("live Session outcome");
        assert_eq!(
            outcome,
            ProviderSessionTranscriptSyncOutcome::NotProviderSession
        );
        // An agent id that does not match the provider Session pattern is
        // never synchronized.
        let mismatched = create_test_session(
            &state,
            "session.codex.mismatched-1",
            &project.project_id,
            "provider_session",
        );
        let outcome = synchronize_provider_session_transcript(
            &state.service,
            100_001,
            0,
            100,
            "agent.opencode".to_string(),
            mismatched.session_id.clone(),
            subject.clone(),
            None,
        )
        .expect("mismatched agent outcome");
        assert_eq!(
            outcome,
            ProviderSessionTranscriptSyncOutcome::NotProviderSession
        );
        // A provider Session without a runtime binding cannot resolve its
        // provider identity; this is the orphan signature.
        let orphan = create_test_session(
            &state,
            "session.codex.orphan-1",
            &project.project_id,
            "provider_session",
        );
        let outcome = synchronize_provider_session_transcript(
            &state.service,
            100_001,
            0,
            100,
            orphan.agent_id.clone(),
            orphan.session_id.clone(),
            subject.clone(),
            None,
        )
        .expect("orphan outcome");
        assert_eq!(
            outcome,
            ProviderSessionTranscriptSyncOutcome::NoActiveBinding
        );
        // A provider Session whose binding carries no provider Session id is
        // equally unresolvable.
        let unbound = create_test_session(
            &state,
            "session.codex.unbound-1",
            &project.project_id,
            "provider_session",
        );
        create_test_binding(
            &state,
            &unbound.session_id,
            "runtime_binding.codex.unbound-1",
            "provider-session-history",
            None,
        );
        let outcome = synchronize_provider_session_transcript(
            &state.service,
            100_001,
            0,
            100,
            unbound.agent_id.clone(),
            unbound.session_id.clone(),
            subject.clone(),
            None,
        )
        .expect("psid-less outcome");
        assert_eq!(
            outcome,
            ProviderSessionTranscriptSyncOutcome::NoActiveBinding
        );
        // A provider Session bound to another transport is never synchronized.
        let stream_bound = create_test_session(
            &state,
            "session.codex.stream-bound-1",
            &project.project_id,
            "provider_session",
        );
        create_test_binding(
            &state,
            &stream_bound.session_id,
            "runtime_binding.codex.stream-bound-1",
            "sdk-stream",
            Some("provider-thread-stream-1"),
        );
        let outcome = synchronize_provider_session_transcript(
            &state.service,
            100_001,
            0,
            100,
            stream_bound.agent_id.clone(),
            stream_bound.session_id.clone(),
            subject.clone(),
            None,
        )
        .expect("stream-bound outcome");
        assert_eq!(
            outcome,
            ProviderSessionTranscriptSyncOutcome::NoActiveBinding
        );
    }

    #[derive(Debug)]
    struct FailingCwdResolver;

    impl sdkwork_agents_runtime_facade::ProviderSessionProjectCwdResolver for FailingCwdResolver {
        fn resolve_project_cwd(
            &self,
            _selector: &sdkwork_agents_runtime_facade::ProviderSessionProjectCwdSelector,
        ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<Option<String>> {
            Err(
                sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                    "test mount resolution failure".to_string(),
                ),
            )
        }
    }

    #[test]
    fn transcript_sync_degrades_gracefully_when_cwd_resolution_fails() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let bound = create_test_session(
            &state,
            "session.codex.cwd-failure-1",
            &project.project_id,
            "provider_session",
        );
        create_test_binding(
            &state,
            &bound.session_id,
            "runtime_binding.codex.cwd-failure-1",
            "provider-session-history",
            Some("provider-thread-cwd-failure-1"),
        );
        let outcome = synchronize_provider_session_transcript(
            &state.service,
            100_001,
            0,
            100,
            bound.agent_id.clone(),
            bound.session_id.clone(),
            manage_subject(),
            Some(&FailingCwdResolver),
        )
        .expect("a failed working directory resolution must not fail the transcript read");
        assert!(
            matches!(
                outcome,
                ProviderSessionTranscriptSyncOutcome::Imported {
                    imported_item_count: 0
                } | ProviderSessionTranscriptSyncOutcome::EngineUnavailable
            ),
            "cwd resolution failure should degrade to a skipped/empty synchronization, got {outcome:?}"
        );
    }

    #[test]
    fn archives_orphaned_provider_sessions_without_binding_or_items() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let orphan = create_test_session(
            &state,
            "session.codex.orphan-2",
            &project.project_id,
            "provider_session",
        );
        let bound = create_test_session(
            &state,
            "session.codex.bound-2",
            &project.project_id,
            "provider_session",
        );
        create_test_binding(
            &state,
            &bound.session_id,
            "runtime_binding.codex.bound-2",
            "provider-session-history",
            Some("provider-thread-bound-2"),
        );
        let live = create_test_session(
            &state,
            "session.340000000000000001",
            &project.project_id,
            "coding-workbench",
        );

        let mut result = ProviderSessionSynchronizationResult::default();
        reconcile_orphaned_provider_sessions(
            &state.service,
            &project,
            manage_subject(),
            &["codex".to_string()],
            &mut result,
        )
        .expect("orphan reconciliation");

        assert_eq!(result.skipped_session_count, 1);
        assert!(result.issues.iter().any(|issue| {
            issue.code == "orphaned_provider_session_archived" && issue.count == 1
        }));
        let archived = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: orphan.agent_id.clone(),
                session_id: orphan.session_id.clone(),
                owner_scope: Some(100),
                requested_by: manage_subject(),
            })
            .expect("archived orphan lookup");
        assert_eq!(archived.status, crate::domain::AgentSessionStatus::Archived);
        let still_active = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: bound.agent_id.clone(),
                session_id: bound.session_id.clone(),
                owner_scope: Some(100),
                requested_by: manage_subject(),
            })
            .expect("bound session lookup");
        assert_eq!(
            still_active.status,
            crate::domain::AgentSessionStatus::Active
        );
        let live_active = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: live.agent_id.clone(),
                session_id: live.session_id.clone(),
                owner_scope: Some(100),
                requested_by: manage_subject(),
            })
            .expect("live session lookup");
        assert_eq!(
            live_active.status,
            crate::domain::AgentSessionStatus::Active
        );
    }

    #[test]
    fn provider_session_inventory_fingerprint_is_stable_and_order_independent() {
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let codex = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let item_a = inventory_item(codex, "provider-thread-a".to_string(), 1);
        let item_b = inventory_item(codex, "provider-thread-b".to_string(), 2);
        assert_eq!(
            provider_session_inventory_fingerprint(&[item_a.clone(), item_b.clone()]),
            provider_session_inventory_fingerprint(&[item_b.clone(), item_a.clone()]),
            "the fingerprint must be independent of discovery order",
        );
        assert_ne!(
            provider_session_inventory_fingerprint(&[item_a.clone()]),
            provider_session_inventory_fingerprint(&[
                item_a,
                inventory_item(codex, "provider-thread-c".to_string(), 3),
            ]),
            "a changed Session identity set must change the fingerprint",
        );
    }

    #[test]
    fn unchanged_inventory_fingerprint_skips_the_full_inventory_sweeps() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        reset_provider_session_sync_cache_for_testing();
        let catalog = shared_agent_engine_host()
            .expect("agent engine host")
            .catalog();
        let codex = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![inventory_item(codex, "codex-sweep-live".to_string(), 1)],
        )
        .expect("seed provider session");
        // An orphaned provider-history Session without bindings or items that
        // only the full orphan sweep can archive.
        let orphan = create_test_session(
            &state,
            "session.codex.sweep-orphan",
            &project.project_id,
            "provider_session",
        );
        let items = vec![inventory_item(codex, "codex-sweep-live".to_string(), 1)];
        let reconcile = |state: &AgentHttpState,
                         project: &AgentProjectRecord,
                         items: Vec<ProviderSessionInventoryItem>| {
            synchronize_provider_session_snapshot(
                state.service.clone(),
                project,
                manage_subject(),
                ProviderSessionInventorySnapshot {
                    directory_resolved: true,
                    items,
                    successful_engine_keys: vec!["codex".to_string()],
                    issues: Vec::new(),
                    unattributed_provider_sessions: false,
                },
            )
        };
        // The completed entry carries the same fingerprint as the incoming
        // inventory, so both full sweeps must be skipped and the orphan must
        // survive this reconciliation untouched.
        record_completed_provider_session_sync(
            &provider_session_sync_cache_key(&project),
            CompletedProviderSessionSync {
                fingerprint: provider_session_inventory_fingerprint(&items),
                result: ProviderSessionSynchronizationResult::default(),
                completed_at: Instant::now(),
            },
        );
        let result = reconcile(&state, &project, items.clone()).expect("incremental reconcile");
        assert_eq!(result.skipped_session_count, 0);
        assert_eq!(result.failed_session_count, 0);
        let orphan_still_active = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: orphan.agent_id.clone(),
                session_id: orphan.session_id.clone(),
                owner_scope: Some(100),
                requested_by: read_subject(),
            })
            .expect("orphan session lookup");
        assert_eq!(
            orphan_still_active.status,
            crate::domain::AgentSessionStatus::Active
        );

        // Without a matching completed entry the full path runs again and the
        // orphan sweep archives the unbound provider Session (the successful
        // archive itself is accounted as a skipped issue, mirroring the
        // dedicated orphan sweep test).
        reset_provider_session_sync_cache_for_testing();
        let result = reconcile(&state, &project, items).expect("full reconcile");
        assert_eq!(result.skipped_session_count, 1);
        assert_eq!(result.failed_session_count, 0);
        assert!(result.issues.iter().any(|issue| {
            issue.code == "orphaned_provider_session_archived" && issue.count == 1
        }));
        let orphan_archived = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: orphan.agent_id.clone(),
                session_id: orphan.session_id.clone(),
                owner_scope: Some(100),
                requested_by: read_subject(),
            })
            .expect("orphan session lookup");
        assert_eq!(
            orphan_archived.status,
            crate::domain::AgentSessionStatus::Archived
        );
    }

    #[test]
    fn completed_sync_within_refresh_window_skips_provider_discovery() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        reset_provider_session_sync_cache_for_testing();
        // The cached outcome is deliberately non-default so a fresh discovery
        // or reconcile could never produce it: the fast path must be served
        // from the cache without touching the provider inventory at all.
        let mut expected = ProviderSessionSynchronizationResult::default();
        expected.record_issue(
            "provider_inventory_unavailable",
            ProviderSessionSynchronizationIssueDisposition::Failed,
            1,
        );
        expected.record_skipped("subagent_without_parent");
        expected.synchronized_session_count = 7;
        record_completed_provider_session_sync(
            &provider_session_sync_cache_key(&project),
            CompletedProviderSessionSync {
                fingerprint: String::new(),
                result: expected.clone(),
                completed_at: Instant::now(),
            },
        );
        let result = synchronize_project_provider_sessions_with_selector(
            state.service.clone(),
            &project,
            read_subject(),
            None,
            None,
            None,
        )
        .expect("cached synchronization");
        assert_eq!(result, expected);
        reset_provider_session_sync_cache_for_testing();
    }

    #[test]
    fn unattributed_inventory_is_reported_as_skipped_issue() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        reset_provider_session_sync_cache_for_testing();
        let result = synchronize_provider_session_snapshot(
            state.service.clone(),
            &project,
            read_subject(),
            ProviderSessionInventorySnapshot {
                directory_resolved: false,
                items: Vec::new(),
                successful_engine_keys: vec!["codex".to_string()],
                issues: Vec::new(),
                unattributed_provider_sessions: true,
            },
        )
        .expect("reconcile unattributed snapshot");

        assert_eq!(result.synchronized_session_count, 0);
        assert_eq!(result.skipped_session_count, 1);
        assert_eq!(result.failed_session_count, 0);
        let unattributed = result
            .issues
            .iter()
            .find(|issue| issue.code == "provider_inventory_unattributed")
            .expect("unattributed issue");
        assert_eq!(
            unattributed.disposition,
            ProviderSessionSynchronizationIssueDisposition::Skipped
        );
        assert_eq!(unattributed.count, 1);
    }
}
