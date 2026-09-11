//! PostgreSQL-backed agent configuration profile store.
//!
//! The model configuration runtime persists applied profiles through the
//! kernel [`AgentConfigurationStore`] SPI. The server-authoritative store
//! keeps profiles in the canonical Agents PostgreSQL database
//! (`ai_agent_model_configuration_profile`, applied through the
//! `database/ddl/baseline/postgres/` baseline), so applied model
//! configurations survive process restarts.
//!
//! Every profile row is owner scoped (tenant/organization/owner user). The
//! kernel SPI methods carry no owner scope, so they fail closed; HTTP
//! adapters must use the [`ScopedAgentConfigurationStore`] surface, which
//! always filters on the owner columns, so profiles can never cross tenants.
//! SQLite is client-local only (DATABASE_SPEC: authoritative-server
//! persistence is PostgreSQL only).

use sdkwork_agent_kernel::{
    AgentConfigValue, AgentConfigurationChange, AgentConfigurationProfile, AgentConfigurationStore,
    AgentConfigurationStoreRecord, AgentConfigurationSubscriber, AgentConfigurationUpgradePlan,
    AgentProfileArchiveRequest, AgentSecretBindingKind, ConfigurationSubscription, KernelError,
    KernelResult,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::persistence::AGENTS_DATABASE_SERVICE;
use crate::postgres_sync_pool::BlockingPostgresPool;

/// Owner scope of a model configuration profile.
///
/// Profiles are tenant, organization and owner scoped; every persisted row
/// carries the scope and every scoped read filters on it. The HTTP adapters
/// derive the scope from the trusted `AgentRequestContext` and construct this
/// value through `ProfileScope::try_parse`; it is never taken from client
/// input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileScope {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
}

impl ProfileScope {
    pub fn try_parse(
        tenant_id: &str,
        organization_id: &str,
        owner_user_id: &str,
    ) -> KernelResult<Self> {
        let tenant_id = crate::validation::parse_tenant_id(tenant_id)?;
        let organization_id = crate::validation::parse_organization_id(organization_id)?;
        let owner_user_id = crate::validation::parse_owner_user_id(owner_user_id)?;
        Ok(Self {
            tenant_id,
            organization_id,
            owner_user_id,
        })
    }
}

/// Tenant-scoped configuration store surface.
///
/// The kernel [`AgentConfigurationStore`] SPI is owner-agnostic because agent
/// engines operate on profiles by id; a shared multi-tenant store must not
/// expose those methods to HTTP adapters. This surface is the only way HTTP
/// requests read or mutate profiles, and every method enforces the owner
/// scope in SQL before any row is touched.
pub trait ScopedAgentConfigurationStore: AgentConfigurationStore + Send + Sync {
    fn save_profile_in_scope(
        &mut self,
        profile: AgentConfigurationProfile,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord>;

    fn find_profile_in_scope(
        &self,
        agent_id: &str,
        profile_id: &str,
        scope: &ProfileScope,
    ) -> KernelResult<Option<AgentConfigurationProfile>>;

    fn list_profiles_in_scope(
        &self,
        agent_id: &str,
        scope: &ProfileScope,
    ) -> KernelResult<Vec<AgentConfigurationProfile>>;

    fn migrate_profile_in_scope(
        &mut self,
        plan: &AgentConfigurationUpgradePlan,
        current_profile: AgentConfigurationProfile,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord>;

    fn archive_profile_in_scope(
        &mut self,
        request: &AgentProfileArchiveRequest,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord>;
}

/// Fails closed for owner-agnostic SPI calls: a shared store has no way to
/// derive the owner scope, so any unscoped access is rejected instead of
/// returning cross-tenant rows.
fn scoped_access_required() -> KernelError {
    KernelError::validation(
        "agent configuration store requires an owner scope; use the scoped store surface",
    )
}

/// PostgreSQL-backed [`AgentConfigurationStore`] for the model configuration
/// runtime profiles. SQL executes on the shared process pool through
/// [`BlockingPostgresPool`]; the store stays `Send + Sync` like the kernel
/// SPI requires.
pub struct PostgresAgentConfigurationStore {
    pool: BlockingPostgresPool,
    subscribers: Arc<RwLock<Vec<(String, AgentConfigurationSubscriber)>>>,
    next_subscription_id: AtomicU64,
}

impl std::fmt::Debug for PostgresAgentConfigurationStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresAgentConfigurationStore")
            .field("subscriber_count", &self.subscriber_count())
            .finish()
    }
}

impl PostgresAgentConfigurationStore {
    /// Wraps a shared process PostgreSQL pool (agents service profile).
    pub fn from_pool(pool: BlockingPostgresPool) -> Self {
        Self {
            pool,
            subscribers: Arc::new(RwLock::new(Vec::new())),
            next_subscription_id: AtomicU64::new(0),
        }
    }

    /// Connects through the canonical `sdkwork-database-config` Agents
    /// service profile (server-authoritative PostgreSQL).
    pub fn connect_from_agents_database_env() -> KernelResult<Self> {
        Ok(Self::from_pool(
            BlockingPostgresPool::connect_from_sdkwork_env(AGENTS_DATABASE_SERVICE)?,
        ))
    }

    fn subscriber_count(&self) -> usize {
        self.subscribers
            .read()
            .map(|subscribers| subscribers.len())
            .unwrap_or(0)
    }

    /// Atomic upsert: one statement inserts or updates the scoped row, so
    /// concurrent saves of the same profile cannot race through a
    /// check-then-act window.
    fn upsert(&self, profile: AgentConfigurationProfile, scope: &ProfileScope) -> KernelResult<()> {
        let configuration_json = configuration_to_json(&profile).to_string();
        let secret_bindings_json = secret_bindings_to_json(&profile).to_string();
        let now = now_rfc3339();
        let pool = self.pool.clone();
        let profile_id = profile.profile_id.clone();
        let agent_id = profile.agent_id.clone();
        let configuration_version = profile.configuration_version.clone();
        let status = profile.status.as_str().to_owned();
        let tenant_id = scope.tenant_id as i64;
        let organization_id = scope.organization_id as i64;
        let owner_user_id = scope.owner_user_id as i64;
        self.pool.block_on(async move {
            sqlx::query(
                "INSERT INTO ai_agent_model_configuration_profile
                 (profile_id, agent_id, tenant_id, organization_id, owner_user_id,
                  configuration_version, status, configuration_json,
                  secret_bindings_json, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
                 ON CONFLICT (profile_id) DO UPDATE SET
                     agent_id = EXCLUDED.agent_id,
                     configuration_version = EXCLUDED.configuration_version,
                     status = EXCLUDED.status,
                     configuration_json = EXCLUDED.configuration_json,
                     secret_bindings_json = EXCLUDED.secret_bindings_json,
                     updated_at = EXCLUDED.updated_at,
                     version = ai_agent_model_configuration_profile.version + 1
                 WHERE ai_agent_model_configuration_profile.tenant_id = $3
                   AND ai_agent_model_configuration_profile.organization_id = $4
                   AND ai_agent_model_configuration_profile.owner_user_id = $5",
            )
            .bind(&profile_id)
            .bind(&agent_id)
            .bind(tenant_id)
            .bind(organization_id)
            .bind(owner_user_id)
            .bind(&configuration_version)
            .bind(&status)
            .bind(&configuration_json)
            .bind(&secret_bindings_json)
            .bind(&now)
            .execute(pool.pool())
            .await
            .map_err(|error| store_error("upsert", error.to_string()))?;
            Ok(())
        })
    }

    fn notify(&self, record: &AgentConfigurationStoreRecord, change: AgentConfigurationChange) {
        if let Ok(subscribers) = self.subscribers.read() {
            for (_, subscriber) in subscribers.iter() {
                subscriber(record, change);
            }
        }
    }
}

impl AgentConfigurationStore for PostgresAgentConfigurationStore {
    fn save_profile(
        &mut self,
        _profile: AgentConfigurationProfile,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        Err(scoped_access_required())
    }

    fn load_profile(
        &self,
        _agent_id: &str,
        _profile_id: &str,
    ) -> KernelResult<AgentConfigurationProfile> {
        Err(scoped_access_required())
    }

    fn list_profiles(&self, _agent_id: &str) -> KernelResult<Vec<AgentConfigurationProfile>> {
        Err(scoped_access_required())
    }

    fn migrate_profile(
        &mut self,
        _plan: &AgentConfigurationUpgradePlan,
        _current_profile: AgentConfigurationProfile,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        Err(scoped_access_required())
    }

    fn archive_profile(
        &mut self,
        _request: &AgentProfileArchiveRequest,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        Err(scoped_access_required())
    }

    fn subscribe(&mut self, subscriber: AgentConfigurationSubscriber) -> ConfigurationSubscription {
        let subscription_id = format!(
            "subscription.{}",
            self.next_subscription_id.fetch_add(1, Ordering::Relaxed)
        );
        if let Ok(mut subscribers) = self.subscribers.write() {
            subscribers.push((subscription_id.clone(), subscriber));
        }
        ConfigurationSubscription::registered(subscription_id, self.subscribers.clone())
    }
}

impl ScopedAgentConfigurationStore for PostgresAgentConfigurationStore {
    fn save_profile_in_scope(
        &mut self,
        profile: AgentConfigurationProfile,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        self.upsert(profile.clone(), scope)?;
        let record = AgentConfigurationStoreRecord::created(profile);
        self.notify(&record, AgentConfigurationChange::Saved);
        Ok(record)
    }

    fn find_profile_in_scope(
        &self,
        agent_id: &str,
        profile_id: &str,
        scope: &ProfileScope,
    ) -> KernelResult<Option<AgentConfigurationProfile>> {
        let pool = self.pool.clone();
        let agent_id = agent_id.to_owned();
        let profile_id = profile_id.to_owned();
        let tenant_id = scope.tenant_id as i64;
        let organization_id = scope.organization_id as i64;
        let owner_user_id = scope.owner_user_id as i64;
        let row: Option<(String, String, String, String, String, String)> =
            self.pool.block_on(async move {
                sqlx::query_as::<_, (String, String, String, String, String, String)>(
                    "SELECT profile_id, agent_id, configuration_version, status,
                            configuration_json, secret_bindings_json
                     FROM ai_agent_model_configuration_profile
                     WHERE profile_id = $1 AND agent_id = $2
                       AND tenant_id = $3 AND organization_id = $4
                       AND owner_user_id = $5",
                )
                .bind(&profile_id)
                .bind(&agent_id)
                .bind(tenant_id)
                .bind(organization_id)
                .bind(owner_user_id)
                .fetch_optional(pool.pool())
                .await
                .map_err(|error| store_error("find_query", error.to_string()))
            })?;
        row.map(
            |(
                profile_id,
                agent_id,
                configuration_version,
                status,
                configuration_json,
                secret_bindings_json,
            )| {
                profile_from_columns(
                    profile_id,
                    agent_id,
                    configuration_version,
                    status,
                    &configuration_json,
                    &secret_bindings_json,
                )
            },
        )
        .transpose()
    }

    fn list_profiles_in_scope(
        &self,
        agent_id: &str,
        scope: &ProfileScope,
    ) -> KernelResult<Vec<AgentConfigurationProfile>> {
        let pool = self.pool.clone();
        let agent_id = agent_id.to_owned();
        let tenant_id = scope.tenant_id as i64;
        let organization_id = scope.organization_id as i64;
        let owner_user_id = scope.owner_user_id as i64;
        let rows: Vec<(String, String, String, String, String, String)> =
            self.pool.block_on(async move {
                sqlx::query_as::<_, (String, String, String, String, String, String)>(
                    "SELECT profile_id, agent_id, configuration_version, status,
                            configuration_json, secret_bindings_json
                     FROM ai_agent_model_configuration_profile
                     WHERE agent_id = $1
                       AND tenant_id = $2 AND organization_id = $3
                       AND owner_user_id = $4
                     ORDER BY profile_id",
                )
                .bind(&agent_id)
                .bind(tenant_id)
                .bind(organization_id)
                .bind(owner_user_id)
                .fetch_all(pool.pool())
                .await
                .map_err(|error| store_error("list_query", error.to_string()))
            })?;
        rows.into_iter()
            .map(
                |(
                    profile_id,
                    agent_id,
                    configuration_version,
                    status,
                    configuration_json,
                    secret_bindings_json,
                )| {
                    profile_from_columns(
                        profile_id,
                        agent_id,
                        configuration_version,
                        status,
                        &configuration_json,
                        &secret_bindings_json,
                    )
                },
            )
            .collect()
    }

    fn migrate_profile_in_scope(
        &mut self,
        plan: &AgentConfigurationUpgradePlan,
        current_profile: AgentConfigurationProfile,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        self.upsert(current_profile.clone(), scope)?;
        let record = AgentConfigurationStoreRecord::migrated(current_profile, &plan.plan_id);
        self.notify(&record, AgentConfigurationChange::Migrated);
        Ok(record)
    }

    fn archive_profile_in_scope(
        &mut self,
        request: &AgentProfileArchiveRequest,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        let profile = self
            .find_profile_in_scope(&request.agent_id, &request.profile_id, scope)?
            .ok_or_else(|| {
                KernelError::validation(format!(
                    "agent configuration profile not found: {}/{}",
                    request.agent_id, request.profile_id
                ))
            })?
            .archive();
        self.upsert(profile.clone(), scope)?;
        let record = AgentConfigurationStoreRecord::archived(profile, &request.request_id);
        self.notify(&record, AgentConfigurationChange::Archived);
        Ok(record)
    }
}

fn store_error(context: impl AsRef<str>, message: impl AsRef<str>) -> KernelError {
    KernelError::provider_error(
        format!("agent_configuration_store.{}", context.as_ref()),
        message.as_ref(),
    )
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// Serializes the profile `configuration` entries in the kernel manifest
/// shape (`{"entries": [...]}`) so [`AgentConfigurationProfile::from_json`]
/// can parse it back.
fn configuration_to_json(profile: &AgentConfigurationProfile) -> serde_json::Value {
    let entries = profile
        .configuration
        .entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "key": entry.key,
                "value_kind": value_kind_str(&entry.value),
                "value": value_to_json(&entry.value),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({ "entries": entries })
}

/// Serializes the profile secret bindings in the kernel manifest shape.
fn secret_bindings_to_json(profile: &AgentConfigurationProfile) -> serde_json::Value {
    let bindings = profile
        .secret_bindings
        .iter()
        .map(|binding| {
            let mut value = serde_json::json!({
                "field_key": binding.field_key,
                "kind": binding_kind_str(binding.binding_kind),
                "secret_ref": binding.secret_ref,
            });
            if let Some(provider_hint) = &binding.provider_hint {
                value["provider_hint"] = serde_json::Value::String(provider_hint.clone());
            }
            value
        })
        .collect::<Vec<_>>();
    serde_json::Value::Array(bindings)
}

/// Reconstructs a profile from its column values through the kernel's
/// validated manifest parser.
fn profile_from_columns(
    profile_id: String,
    agent_id: String,
    configuration_version: String,
    status: String,
    configuration_json: &str,
    secret_bindings_json: &str,
) -> KernelResult<AgentConfigurationProfile> {
    let configuration =
        serde_json::from_str::<serde_json::Value>(configuration_json).map_err(|error| {
            store_error(
                "parse_configuration",
                format!("stored configuration is not valid JSON: {error}"),
            )
        })?;
    let secret_bindings =
        serde_json::from_str::<serde_json::Value>(secret_bindings_json).map_err(|error| {
            store_error(
                "parse_bindings",
                format!("stored secret bindings are not valid JSON: {error}"),
            )
        })?;
    let document = serde_json::json!({
        "manifest_type": "agent_configuration_profile",
        "profile_id": profile_id,
        "agent_id": agent_id,
        "configuration_version": configuration_version,
        "status": status,
        "configuration": configuration,
        "secret_bindings": secret_bindings,
    });
    AgentConfigurationProfile::from_json(&document.to_string())
}

fn value_kind_str(value: &AgentConfigValue) -> &'static str {
    match value {
        AgentConfigValue::String(_) => "string",
        AgentConfigValue::Boolean(_) => "boolean",
        AgentConfigValue::Integer(_) => "integer",
        AgentConfigValue::SecretRef(_) => "secret_ref",
        AgentConfigValue::StringList(_) => "string_list",
        AgentConfigValue::Json(_) => "json",
    }
}

fn value_to_json(value: &AgentConfigValue) -> serde_json::Value {
    match value {
        AgentConfigValue::String(value) => serde_json::Value::String(value.clone()),
        AgentConfigValue::Boolean(value) => serde_json::Value::Bool(*value),
        AgentConfigValue::Integer(value) => serde_json::Value::Number((*value).into()),
        AgentConfigValue::SecretRef(value) => serde_json::Value::String(value.clone()),
        AgentConfigValue::StringList(values) => serde_json::Value::Array(
            values
                .iter()
                .cloned()
                .map(serde_json::Value::String)
                .collect(),
        ),
        AgentConfigValue::Json(value) => {
            serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.clone()))
        }
    }
}

fn binding_kind_str(kind: AgentSecretBindingKind) -> &'static str {
    match kind {
        AgentSecretBindingKind::LoginPassword => "login_password",
        AgentSecretBindingKind::LoginToken => "login_token",
        AgentSecretBindingKind::OAuthCredential => "oauth_credential",
        AgentSecretBindingKind::LlmApiKey => "llm_api_key",
        AgentSecretBindingKind::CustomSecret => "custom_secret",
    }
}

/// Owner-scoped in-memory configuration store.
///
/// Mirrors the PostgreSQL store semantics for local development and tests:
/// rows are keyed by owner scope plus agent/profile id, scoped methods filter
/// on the scope, and the owner-agnostic kernel SPI fails closed. Without this
/// wrapper the kernel `InMemoryAgentConfigurationStore` would leak profiles
/// across tenants in shared development processes.
#[derive(Default)]
pub struct ScopedInMemoryAgentConfigurationStore {
    profiles: Mutex<Vec<(ProfileScope, AgentConfigurationProfile)>>,
    subscribers: Arc<RwLock<Vec<(String, AgentConfigurationSubscriber)>>>,
    next_subscription_id: AtomicU64,
}

impl ScopedInMemoryAgentConfigurationStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn notify(&self, record: &AgentConfigurationStoreRecord, change: AgentConfigurationChange) {
        if let Ok(subscribers) = self.subscribers.read() {
            for (_, subscriber) in subscribers.iter() {
                subscriber(record, change);
            }
        }
    }

    fn save_locked(
        profiles: &mut Vec<(ProfileScope, AgentConfigurationProfile)>,
        profile: AgentConfigurationProfile,
        scope: &ProfileScope,
    ) {
        profiles.retain(|(stored_scope, existing)| {
            stored_scope != scope || existing.profile_id != profile.profile_id
        });
        profiles.push((*scope, profile));
    }

    fn find_locked(
        profiles: &[(ProfileScope, AgentConfigurationProfile)],
        agent_id: &str,
        profile_id: &str,
        scope: &ProfileScope,
    ) -> Option<AgentConfigurationProfile> {
        profiles
            .iter()
            .find(|(stored_scope, profile)| {
                stored_scope == scope
                    && profile.agent_id == agent_id
                    && profile.profile_id == profile_id
            })
            .map(|(_, profile)| profile.clone())
    }
}

impl AgentConfigurationStore for ScopedInMemoryAgentConfigurationStore {
    fn save_profile(
        &mut self,
        _profile: AgentConfigurationProfile,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        Err(scoped_access_required())
    }

    fn load_profile(
        &self,
        _agent_id: &str,
        _profile_id: &str,
    ) -> KernelResult<AgentConfigurationProfile> {
        Err(scoped_access_required())
    }

    fn list_profiles(&self, _agent_id: &str) -> KernelResult<Vec<AgentConfigurationProfile>> {
        Err(scoped_access_required())
    }

    fn migrate_profile(
        &mut self,
        _plan: &AgentConfigurationUpgradePlan,
        _current_profile: AgentConfigurationProfile,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        Err(scoped_access_required())
    }

    fn archive_profile(
        &mut self,
        _request: &AgentProfileArchiveRequest,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        Err(scoped_access_required())
    }

    fn subscribe(&mut self, subscriber: AgentConfigurationSubscriber) -> ConfigurationSubscription {
        let subscription_id = format!(
            "subscription.{}",
            self.next_subscription_id.fetch_add(1, Ordering::Relaxed)
        );
        if let Ok(mut subscribers) = self.subscribers.write() {
            subscribers.push((subscription_id.clone(), subscriber));
        }
        ConfigurationSubscription::registered(subscription_id, self.subscribers.clone())
    }
}

impl ScopedAgentConfigurationStore for ScopedInMemoryAgentConfigurationStore {
    fn save_profile_in_scope(
        &mut self,
        profile: AgentConfigurationProfile,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        let mut profiles = self.profiles.lock().map_err(|_| scoped_access_required())?;
        Self::save_locked(&mut profiles, profile.clone(), scope);
        drop(profiles);
        let record = AgentConfigurationStoreRecord::created(profile);
        self.notify(&record, AgentConfigurationChange::Saved);
        Ok(record)
    }

    fn find_profile_in_scope(
        &self,
        agent_id: &str,
        profile_id: &str,
        scope: &ProfileScope,
    ) -> KernelResult<Option<AgentConfigurationProfile>> {
        let profiles = self.profiles.lock().map_err(|_| scoped_access_required())?;
        Ok(Self::find_locked(&profiles, agent_id, profile_id, scope))
    }

    fn list_profiles_in_scope(
        &self,
        agent_id: &str,
        scope: &ProfileScope,
    ) -> KernelResult<Vec<AgentConfigurationProfile>> {
        let profiles = self.profiles.lock().map_err(|_| scoped_access_required())?;
        let mut found = profiles
            .iter()
            .filter(|(stored_scope, profile)| stored_scope == scope && profile.agent_id == agent_id)
            .map(|(_, profile)| profile.clone())
            .collect::<Vec<_>>();
        found.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        Ok(found)
    }

    fn migrate_profile_in_scope(
        &mut self,
        plan: &AgentConfigurationUpgradePlan,
        current_profile: AgentConfigurationProfile,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        let mut profiles = self.profiles.lock().map_err(|_| scoped_access_required())?;
        Self::save_locked(&mut profiles, current_profile.clone(), scope);
        drop(profiles);
        let record = AgentConfigurationStoreRecord::migrated(current_profile, &plan.plan_id);
        self.notify(&record, AgentConfigurationChange::Migrated);
        Ok(record)
    }

    fn archive_profile_in_scope(
        &mut self,
        request: &AgentProfileArchiveRequest,
        scope: &ProfileScope,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        let mut profiles = self.profiles.lock().map_err(|_| scoped_access_required())?;
        let profile = Self::find_locked(&profiles, &request.agent_id, &request.profile_id, scope)
            .ok_or_else(|| {
                KernelError::validation(format!(
                    "agent configuration profile not found: {}/{}",
                    request.agent_id, request.profile_id
                ))
            })?
            .archive();
        Self::save_locked(&mut profiles, profile.clone(), scope);
        drop(profiles);
        let record = AgentConfigurationStoreRecord::archived(profile, &request.request_id);
        self.notify(&record, AgentConfigurationChange::Archived);
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::{
        AgentConfiguration, AgentConfigurationProfileStatus, AgentSecretBinding,
    };

    fn sample_profile(agent_id: &str, profile_id: &str) -> AgentConfigurationProfile {
        AgentConfigurationProfile::new(
            profile_id,
            agent_id,
            "0.2.0",
            AgentConfiguration::new(agent_id, profile_id)
                .set(
                    "codex.model.base_url",
                    AgentConfigValue::string("https://api.birdcoder.com/v1"),
                )
                .set("codex.model.default", AgentConfigValue::string("gpt-5.4"))
                .set(
                    "codex.model.api_key",
                    AgentConfigValue::secret_ref("secret.model.key-1"),
                )
                .set(
                    "codex.model.supported",
                    AgentConfigValue::string_list(vec!["gpt-5.4".to_string()]),
                )
                .set(
                    "codex.model.supports_multimodal",
                    AgentConfigValue::boolean(false),
                ),
        )
        .add_secret_binding(AgentSecretBinding::llm_api_key(
            "codex.model.api_key",
            "openai",
            "secret.model.key-1",
        ))
        .activate()
    }

    #[test]
    fn profile_serialization_round_trips_kernel_manifest_shape() {
        let profile = sample_profile("agent-1", "profile-1");
        let document = serde_json::json!({
            "manifest_type": "agent_configuration_profile",
            "profile_id": profile.profile_id,
            "agent_id": profile.agent_id,
            "configuration_version": profile.configuration_version,
            "status": profile.status.as_str(),
            "configuration": configuration_to_json(&profile),
            "secret_bindings": secret_bindings_to_json(&profile),
        });
        let parsed = AgentConfigurationProfile::from_json(&document.to_string())
            .expect("kernel manifest parser");
        assert_eq!(parsed.profile_id, profile.profile_id);
        assert_eq!(parsed.agent_id, profile.agent_id);
        assert_eq!(parsed.status, AgentConfigurationProfileStatus::Active);
        assert_eq!(parsed.secret_bindings.len(), 1);
    }

    fn test_scope(tenant_id: u64, owner_user_id: u64) -> ProfileScope {
        ProfileScope {
            tenant_id,
            organization_id: 1,
            owner_user_id,
        }
    }

    #[test]
    fn scoped_in_memory_store_isolates_profiles_across_tenants() {
        let mut store = ScopedInMemoryAgentConfigurationStore::new();
        let tenant_a = test_scope(1001, 2001);
        let tenant_b = test_scope(1002, 2002);
        let agent_id = "agent.codex";

        store
            .save_profile_in_scope(sample_profile(agent_id, "profile.shared-name"), &tenant_a)
            .expect("tenant A save");

        // Tenant B lists nothing even though the agent id matches.
        let tenant_b_profiles = store
            .list_profiles_in_scope(agent_id, &tenant_b)
            .expect("tenant B list");
        assert!(
            tenant_b_profiles.is_empty(),
            "tenant B must not observe tenant A profiles"
        );

        // Tenant B cannot read or archive tenant A profiles by id.
        assert!(
            store
                .find_profile_in_scope(agent_id, "profile.shared-name", &tenant_b)
                .expect("tenant B find")
                .is_none(),
            "tenant B must not read tenant A profiles"
        );
        assert!(
            store
                .archive_profile_in_scope(
                    &AgentProfileArchiveRequest::new("request.1", agent_id, "profile.shared-name",),
                    &tenant_b,
                )
                .is_err(),
            "tenant B must not archive tenant A profiles"
        );

        // Tenant A still sees and can mutate its own profile.
        let tenant_a_profiles = store
            .list_profiles_in_scope(agent_id, &tenant_a)
            .expect("tenant A list");
        assert_eq!(tenant_a_profiles.len(), 1);
        assert_eq!(tenant_a_profiles[0].profile_id, "profile.shared-name");

        // The owner-agnostic kernel SPI fails closed on shared stores.
        assert!(
            store.list_profiles(agent_id).is_err(),
            "unscoped list must fail closed"
        );
        assert!(
            store.load_profile(agent_id, "profile.shared-name").is_err(),
            "unscoped load must fail closed"
        );
        assert!(
            store
                .save_profile(sample_profile(agent_id, "profile.shared-name"))
                .is_err(),
            "unscoped save must fail closed"
        );
    }

    #[test]
    fn scoped_in_memory_store_supports_same_profile_id_per_owner() {
        let mut store = ScopedInMemoryAgentConfigurationStore::new();
        let tenant_a = test_scope(1001, 2001);
        let tenant_b = test_scope(1002, 2002);
        let agent_id = "agent.codex";

        store
            .save_profile_in_scope(sample_profile(agent_id, "profile.1"), &tenant_a)
            .expect("tenant A save");
        store
            .save_profile_in_scope(sample_profile(agent_id, "profile.1"), &tenant_b)
            .expect("tenant B save");

        assert_eq!(
            store
                .list_profiles_in_scope(agent_id, &tenant_a)
                .expect("tenant A list")
                .len(),
            1
        );
        assert_eq!(
            store
                .list_profiles_in_scope(agent_id, &tenant_b)
                .expect("tenant B list")
                .len(),
            1
        );
        assert!(
            store
                .find_profile_in_scope(agent_id, "profile.1", &tenant_a)
                .expect("tenant A find")
                .is_some(),
            "tenant A keeps its own profile with the same id as tenant B"
        );
    }
}
