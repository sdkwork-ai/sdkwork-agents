//! Managed agent category catalog.
//!
//! Agents are classified by a stable `categoryId` string carried in their
//! management profile (`AgentManagementProfileDto::category_id`). This module
//! owns the administrative catalog those ids refer to: the ordered, tenant
//! scoped taxonomy rendered by the admin console, plus the upsert semantics
//! that keep the catalog reviewable.
//!
//! The catalog is deliberately narrow — list and upsert only — matching the
//! plugin/MCP category registry contract: categories are taxonomy rows, not
//! independently lifecycle-managed aggregates. Retirement is modelled through
//! the lifecycle status, never through a destructive delete.
//!
//! Design rules enforced here and at the database boundary:
//! - identity is the tenant-scoped `categoryCode`; the code is immutable on
//!   update because the wire update path keys on it;
//! - the taxonomy is at most two levels deep, and a category with children
//!   cannot itself become a child;
//! - a parent must exist in the same tenant and be active;
//! - `expectedVersion` provides optimistic concurrency for edits;
//! - the lifecycle status, sort order and optional icon reference are explicit
//!   admin-managed fields.
//!
//! Persistence follows the module split used by the rest of the Agents managed
//! store: [`InMemoryAgentCategoryStore`] for local development and contract
//! tests, and [`PostgresAgentCategoryStore`] over the canonical
//! `ai_agent_category` table for server-authoritative deployments.

use std::collections::BTreeMap;
use std::sync::Mutex;

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::sha256_hash;
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres-sync")]
use sqlx::Row;

#[cfg(feature = "postgres-sync")]
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
#[cfg(feature = "postgres-sync")]
use crate::postgres_sync_pool::BlockingPostgresPool;

const MAX_CATEGORY_CODE_LENGTH: usize = 128;
const MAX_CATEGORY_NAME_LENGTH: usize = 255;
const MAX_CATEGORY_DESCRIPTION_LENGTH: usize = 2000;
const MAX_CATEGORY_ICON_REF_LENGTH: usize = 512;
const MAX_CATEGORY_SORT_ORDER: i32 = 9999;
const MAX_CATEGORY_DEPTH: usize = 5;
const MAX_CATEGORY_TAGS: usize = 20;
const MAX_CATEGORY_TAG_LENGTH: usize = 64;
const MAX_CATEGORY_ATTRIBUTES: usize = 50;
const MAX_ATTRIBUTE_KEY_LENGTH: usize = 64;
const MAX_ATTRIBUTE_VALUE_LENGTH: usize = 512;

const VERSION_MISMATCH_DETAIL: &str = "sdkwork.version_mismatch";

/// Lifecycle state of a managed agent category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCategoryStatus {
    Draft,
    Active,
    Disabled,
    Archived,
    Deleted,
}

impl AgentCategoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Draft => 0,
            Self::Active => 1,
            Self::Disabled => 2,
            Self::Archived => 3,
            Self::Deleted => 4,
        }
    }

    pub fn from_db_code(code: i16) -> KernelResult<Self> {
        match code {
            0 => Ok(Self::Draft),
            1 => Ok(Self::Active),
            2 => Ok(Self::Disabled),
            3 => Ok(Self::Archived),
            4 => Ok(Self::Deleted),
            other => Err(KernelError::provider_error(
                "agent_category_status",
                format!("unknown agent category status code: {other}"),
            )),
        }
    }

    /// Only active categories may adopt new children.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// The soft-deleted state is server-owned; the admin upsert cannot set it.
    pub fn is_admin_writable(self) -> bool {
        !matches!(self, Self::Deleted)
    }
}

/// Wire projection of one managed agent category.
///
/// `id`, `parentId`, `version`, `tenantId` and `organizationId` are integer
/// identifiers serialized as decimal strings per `API_SPEC.md` §13.6 so the
/// browser never loses precision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCategoryRecord {
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub id: i64,
    pub uuid: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub tenant_id: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub organization_id: i64,
    pub category_code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default, with = "sdkwork_utils_rust::serde_int64::option")]
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub icon_ref: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
    pub lifecycle_status: AgentCategoryStatus,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Admin upsert command for one category code.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAgentCategoryCommand {
    pub category_code: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, with = "sdkwork_utils_rust::serde_int64::option")]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub icon_ref: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
    #[serde(default)]
    pub status: Option<AgentCategoryStatus>,
    #[serde(default, with = "sdkwork_utils_rust::serde_int64::option")]
    pub expected_version: Option<i64>,
}

/// Validated, normalized upsert intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAgentCategoryCommand {
    pub category_code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub icon_ref: Option<String>,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub status: Option<AgentCategoryStatus>,
    pub expected_version: Option<i64>,
}

/// Persistence contract for the managed category catalog.
pub trait AgentCategoryStore: Send + Sync {
    fn list(&self, tenant_id: u64, organization_id: u64) -> KernelResult<Vec<AgentCategoryRecord>>;

    fn upsert(
        &self,
        tenant_id: u64,
        organization_id: u64,
        operator_id: u64,
        command: &ValidatedAgentCategoryCommand,
    ) -> KernelResult<AgentCategoryRecord>;

    /// Soft-deletes one category and returns the tombstoned row.
    fn delete(
        &self,
        tenant_id: u64,
        organization_id: u64,
        operator_id: u64,
        category_code: &str,
    ) -> KernelResult<AgentCategoryRecord>;

    /// Rewrites one sibling group's sort order from the caller-computed set.
    fn apply_sort_order(
        &self,
        tenant_id: u64,
        organization_id: u64,
        operator_id: u64,
        orders: &[(i64, i32)],
    ) -> KernelResult<()>;

    /// Number of live agents filed under one category code.
    fn count_agents_in_category(
        &self,
        tenant_id: u64,
        organization_id: u64,
        category_code: &str,
    ) -> KernelResult<i64>;
}

/// Trims and validates an upsert command, failing before any row is touched.
pub fn validate_upsert_command(
    command: &UpsertAgentCategoryCommand,
) -> KernelResult<ValidatedAgentCategoryCommand> {
    let category_code = command.category_code.trim();
    validate_category_code(category_code)?;

    let name = command.name.trim();
    if name.is_empty() {
        return Err(KernelError::validation(
            "agent category name must not be empty",
        ));
    }
    if name.chars().count() > MAX_CATEGORY_NAME_LENGTH {
        return Err(KernelError::validation(format!(
            "agent category name must be at most {MAX_CATEGORY_NAME_LENGTH} characters"
        )));
    }

    let description = normalize_optional_text(command.description.as_deref());
    if let Some(description) = description {
        if description.chars().count() > MAX_CATEGORY_DESCRIPTION_LENGTH {
            return Err(KernelError::validation(format!(
                "agent category description must be at most {MAX_CATEGORY_DESCRIPTION_LENGTH} characters"
            )));
        }
    }

    let icon_ref = normalize_optional_text(command.icon_ref.as_deref());
    if let Some(icon_ref) = icon_ref {
        if icon_ref.chars().count() > MAX_CATEGORY_ICON_REF_LENGTH {
            return Err(KernelError::validation(format!(
                "agent category icon reference must be at most {MAX_CATEGORY_ICON_REF_LENGTH} characters"
            )));
        }
    }

    let tags = normalize_category_tags(&command.tags);
    let attributes = normalize_category_attributes(command.attributes.clone());
    validate_category_taxonomy(&tags, &attributes)?;

    if !(0..=MAX_CATEGORY_SORT_ORDER).contains(&command.sort_order) {
        return Err(KernelError::validation(format!(
            "agent category sort order must be between 0 and {MAX_CATEGORY_SORT_ORDER}"
        )));
    }

    if let Some(parent_id) = command.parent_id {
        if parent_id <= 0 {
            return Err(KernelError::validation(
                "agent category parent id must be a positive id",
            ));
        }
    }

    if let Some(status) = command.status {
        if !status.is_admin_writable() {
            return Err(KernelError::validation(
                "agent category status `deleted` is server-owned and cannot be set through the admin upsert",
            ));
        }
    }

    if let Some(expected_version) = command.expected_version {
        if expected_version < 0 {
            return Err(KernelError::validation(
                "agent category expectedVersion must not be negative",
            ));
        }
    }

    Ok(ValidatedAgentCategoryCommand {
        category_code: category_code.to_owned(),
        name: name.to_owned(),
        description: description.map(str::to_owned),
        parent_id: command.parent_id,
        sort_order: command.sort_order,
        icon_ref: icon_ref.map(str::to_owned),
        tags,
        attributes,
        status: command.status,
        expected_version: command.expected_version,
    })
}

fn normalize_optional_text(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Trims tags, drops empty entries and removes case-insensitive duplicates.
pub fn normalize_category_tags(tags: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut normalized = Vec::new();
    for tag in tags {
        let value = tag.trim();
        if value.is_empty() {
            continue;
        }
        if seen.insert(value.to_lowercase()) {
            normalized.push(value.to_owned());
        }
    }
    normalized
}

/// Trims attribute keys and values and drops rows without a key.
pub fn normalize_category_attributes(
    attributes: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    attributes
        .into_iter()
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .filter(|(key, _)| !key.is_empty())
        .collect()
}

fn validate_category_taxonomy(
    tags: &[String],
    attributes: &BTreeMap<String, String>,
) -> KernelResult<()> {
    if tags.len() > MAX_CATEGORY_TAGS {
        return Err(KernelError::validation(format!(
            "agent category tags cannot contain more than {MAX_CATEGORY_TAGS} entries"
        )));
    }
    for tag in tags {
        if tag.chars().count() > MAX_CATEGORY_TAG_LENGTH {
            return Err(KernelError::validation(format!(
                "agent category tag must be at most {MAX_CATEGORY_TAG_LENGTH} characters"
            )));
        }
    }
    if attributes.len() > MAX_CATEGORY_ATTRIBUTES {
        return Err(KernelError::validation(format!(
            "agent category attributes cannot contain more than {MAX_CATEGORY_ATTRIBUTES} entries"
        )));
    }
    for (key, value) in attributes {
        if key.chars().count() > MAX_ATTRIBUTE_KEY_LENGTH {
            return Err(KernelError::validation(format!(
                "agent category attribute key must be at most {MAX_ATTRIBUTE_KEY_LENGTH} characters"
            )));
        }
        if value.chars().count() > MAX_ATTRIBUTE_VALUE_LENGTH {
            return Err(KernelError::validation(format!(
                "agent category attribute {key} must be at most {MAX_ATTRIBUTE_VALUE_LENGTH} characters"
            )));
        }
    }
    Ok(())
}

/// Validates one placement decision against the current catalog.
///
/// The caller passes every non-deleted category in the same tenant scope. The
/// walk rejects self-parenting, cycles (including moving a subtree under its
/// own descendant), unknown parents and any placement deeper than
/// [`MAX_CATEGORY_DEPTH`]; a newly selected parent must still be active.
pub fn validate_category_placement(
    categories: &[AgentCategoryRecord],
    category_code: &str,
    parent_id: Option<i64>,
) -> KernelResult<()> {
    let Some(parent_id) = parent_id else {
        return Ok(());
    };

    let parent = categories
        .iter()
        .find(|record| record.id == parent_id)
        .ok_or_else(|| KernelError::validation("agent category parent does not exist"))?;

    let current = categories
        .iter()
        .find(|record| record.category_code == category_code);

    // Keeping an already-assigned parent is always allowed, even if that
    // parent has since been disabled: an unrelated edit must not force the
    // operator to reactivate the parent. Selecting a newly inactive parent is
    // rejected.
    let retains_current_parent = current.and_then(|record| record.parent_id) == Some(parent_id);
    if !retains_current_parent && !parent.lifecycle_status.is_active() {
        return Err(KernelError::validation(
            "agent category parent must be active",
        ));
    }

    let mut depth = 1usize;
    let mut cursor = Some(parent_id);
    let mut visited = std::collections::HashSet::new();
    while let Some(current_id) = cursor {
        if let Some(current) = current {
            if current.id == current_id {
                return Err(KernelError::validation(
                    "agent category cannot be moved under its own descendant",
                ));
            }
            if current.id == parent_id {
                return Err(KernelError::validation(
                    "agent category cannot be its own parent",
                ));
            }
        }
        if !visited.insert(current_id) {
            return Err(KernelError::validation(
                "agent category hierarchy contains a cycle",
            ));
        }
        let node = categories
            .iter()
            .find(|record| record.id == current_id)
            .ok_or_else(|| KernelError::validation("agent category parent does not exist"))?;
        let Some(next) = node.parent_id else {
            break;
        };
        depth += 1;
        cursor = Some(next);
    }
    if depth + 1 > MAX_CATEGORY_DEPTH {
        return Err(KernelError::validation(format!(
            "agent category hierarchy cannot be deeper than {MAX_CATEGORY_DEPTH} levels"
        )));
    }

    Ok(())
}

fn validate_category_code(value: &str) -> KernelResult<()> {
    let length = value.chars().count();
    if length < 2 || length > MAX_CATEGORY_CODE_LENGTH {
        return Err(KernelError::validation(format!(
            "agent category code must be between 2 and {MAX_CATEGORY_CODE_LENGTH} characters"
        )));
    }

    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return Err(KernelError::validation(
            "agent category code must not be empty",
        ));
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(KernelError::validation(
            "agent category code must start with a lowercase letter or digit",
        ));
    }
    if !characters.all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '_'
            || character == '-'
    }) {
        return Err(KernelError::validation(
            "agent category code may only contain lowercase letters, digits, `_` and `-`",
        ));
    }
    Ok(())
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn build_category_uuid(tenant_id: u64, organization_id: u64, category_code: &str) -> String {
    let material = format!(
        "sdkwork.agents.storage.v1\n{}:agent-category\n{}:{}\n{}:{}\n{}:{}",
        "agent-category".len(),
        tenant_id.to_string().len(),
        tenant_id,
        organization_id.to_string().len(),
        organization_id,
        category_code.len(),
        category_code,
    );
    sha256_hash(material.as_bytes())
}

fn version_mismatch(message: &str) -> KernelError {
    KernelError::conflict(message.to_owned()).with_detail(VERSION_MISMATCH_DETAIL, "true")
}

/// In-memory catalog for development and contract tests.
///
/// Rows are keyed by scope plus category code, and upsert keeps the original
/// id/uuid/createdAt while bumping the version, matching the SQL
/// `ON CONFLICT ... DO UPDATE` semantics. Entity ids come from a process-wide
/// counter: in-memory storage is dev/test-only, and the counter keeps ids
/// unique across every in-memory store in the same process.
pub struct InMemoryAgentCategoryStore {
    records: Mutex<BTreeMap<(u64, u64, String), AgentCategoryRecord>>,
}

static IN_MEMORY_CATEGORY_NEXT_ID: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(1);

impl InMemoryAgentCategoryStore {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(BTreeMap::new()),
        }
    }

    fn locked(
        &self,
    ) -> KernelResult<std::sync::MutexGuard<'_, BTreeMap<(u64, u64, String), AgentCategoryRecord>>>
    {
        self.records.lock().map_err(|_| {
            KernelError::provider_error("agent_category_store", "category store lock poisoned")
        })
    }
}

impl Default for InMemoryAgentCategoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentCategoryStore for InMemoryAgentCategoryStore {
    fn list(&self, tenant_id: u64, organization_id: u64) -> KernelResult<Vec<AgentCategoryRecord>> {
        let records = self.locked()?;
        let mut items = records
            .iter()
            .filter(|((tenant, organization, _), record)| {
                *tenant == tenant_id
                    && *organization == organization_id
                    && record.lifecycle_status != AgentCategoryStatus::Deleted
            })
            .map(|(_, record)| record.clone())
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.sort_order
                .cmp(&right.sort_order)
                .then(left.category_code.cmp(&right.category_code))
        });
        Ok(items)
    }

    fn upsert(
        &self,
        tenant_id: u64,
        organization_id: u64,
        _operator_id: u64,
        command: &ValidatedAgentCategoryCommand,
    ) -> KernelResult<AgentCategoryRecord> {
        let key = (tenant_id, organization_id, command.category_code.clone());
        let now = now_rfc3339();
        let mut records = self.locked()?;
        let existing = records.get(&key).cloned();

        if let Some(expected_version) = command.expected_version {
            let Some(existing) = existing.as_ref() else {
                return Err(version_mismatch(
                    "agent category does not exist at the expected version",
                ));
            };
            if existing.version != expected_version {
                return Err(version_mismatch("agent category version mismatch"));
            }
        }

        let id = existing
            .as_ref()
            .map(|record| record.id)
            .unwrap_or_else(|| {
                IN_MEMORY_CATEGORY_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            });
        let uuid = existing
            .as_ref()
            .map(|record| record.uuid.clone())
            .unwrap_or_else(|| {
                build_category_uuid(tenant_id, organization_id, &command.category_code)
            });
        let created_at = existing
            .as_ref()
            .map(|record| record.created_at.clone())
            .unwrap_or_else(|| now.clone());
        let version = existing
            .as_ref()
            .map(|record| record.version + 1)
            .unwrap_or(0);
        let lifecycle_status = command
            .status
            .or_else(|| existing.as_ref().map(|record| record.lifecycle_status))
            .unwrap_or(AgentCategoryStatus::Active);

        let record = AgentCategoryRecord {
            id,
            uuid,
            tenant_id: tenant_id as i64,
            organization_id: organization_id as i64,
            category_code: command.category_code.clone(),
            name: command.name.clone(),
            description: command.description.clone(),
            parent_id: command.parent_id,
            sort_order: command.sort_order,
            icon_ref: command.icon_ref.clone(),
            tags: command.tags.clone(),
            attributes: command.attributes.clone(),
            lifecycle_status,
            version,
            created_at,
            updated_at: now,
        };
        records.insert(key, record.clone());
        Ok(record)
    }

    fn delete(
        &self,
        tenant_id: u64,
        organization_id: u64,
        _operator_id: u64,
        category_code: &str,
    ) -> KernelResult<AgentCategoryRecord> {
        let key = (tenant_id, organization_id, category_code.to_string());
        let mut records = self.locked()?;
        let record = records
            .get_mut(&key)
            .ok_or_else(|| KernelError::not_found("agent category does not exist"))?;
        record.lifecycle_status = AgentCategoryStatus::Deleted;
        record.version = record.version.saturating_add(1);
        record.updated_at = now_rfc3339();
        Ok(record.clone())
    }

    fn apply_sort_order(
        &self,
        tenant_id: u64,
        organization_id: u64,
        _operator_id: u64,
        orders: &[(i64, i32)],
    ) -> KernelResult<()> {
        let mut records = self.locked()?;
        for (id, sort_order) in orders {
            for ((tenant, organization, _), record) in records.iter_mut() {
                if *tenant == tenant_id && *organization == organization_id && record.id == *id {
                    record.sort_order = *sort_order;
                    record.version = record.version.saturating_add(1);
                    record.updated_at = now_rfc3339();
                }
            }
        }
        Ok(())
    }

    fn count_agents_in_category(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        _category_code: &str,
    ) -> KernelResult<i64> {
        // The in-memory category store is not composed with the agent store;
        // callers that need the reference count use the PostgreSQL store.
        Ok(0)
    }
}

#[cfg(feature = "postgres-sync")]
const CATEGORY_SELECT: &str = "id, uuid, tenant_id, organization_id, category_code, name, description, parent_id, sort_order, icon_ref, tags_json, attributes_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at";

/// PostgreSQL-backed catalog over the canonical `ai_agent_category` table.
#[cfg(feature = "postgres-sync")]
pub struct PostgresAgentCategoryStore {
    pool: BlockingPostgresPool,
    id_generator: AgentBusinessIdGenerator,
}

#[cfg(feature = "postgres-sync")]
impl PostgresAgentCategoryStore {
    pub fn from_pool(pool: BlockingPostgresPool) -> KernelResult<Self> {
        Ok(Self {
            pool,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }
}

#[cfg(feature = "postgres-sync")]
impl AgentCategoryStore for PostgresAgentCategoryStore {
    fn list(&self, tenant_id: u64, organization_id: u64) -> KernelResult<Vec<AgentCategoryRecord>> {
        let query = format!(
            "SELECT {CATEGORY_SELECT} FROM ai_agent_category \
             WHERE tenant_id = $1 AND organization_id = $2 AND deleted_at IS NULL \
             ORDER BY sort_order ASC, category_code ASC"
        );
        let rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(query.as_str()),
            tenant_id as i64,
            organization_id as i64
        )?;
        rows.iter().map(row_to_category).collect()
    }

    fn upsert(
        &self,
        tenant_id: u64,
        organization_id: u64,
        operator_id: u64,
        command: &ValidatedAgentCategoryCommand,
    ) -> KernelResult<AgentCategoryRecord> {
        let id = self.id_generator.next_id()? as i64;
        let uuid = build_category_uuid(tenant_id, organization_id, &command.category_code);
        let now = now_rfc3339();
        let status = command.status.map(AgentCategoryStatus::as_db_code);
        let tags_json = serde_json::to_value(&command.tags).map_err(|error| {
            KernelError::provider_error("agent_category_store", format!("encode tags: {error}"))
        })?;
        let attributes_json = serde_json::to_value(&command.attributes).map_err(|error| {
            KernelError::provider_error(
                "agent_category_store",
                format!("encode attributes: {error}"),
            )
        })?;
        let query = format!(
            "INSERT INTO ai_agent_category \
             (id, uuid, tenant_id, organization_id, category_code, name, description, parent_id, sort_order, icon_ref, tags_json, attributes_json, status, created_by, updated_by, version, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, COALESCE($13::smallint, 1::smallint), $14, $15, 0, $16, $17) \
             ON CONFLICT (tenant_id, organization_id, category_code) DO UPDATE SET \
                 name = EXCLUDED.name, \
                 description = EXCLUDED.description, \
                 parent_id = EXCLUDED.parent_id, \
                 sort_order = EXCLUDED.sort_order, \
                 icon_ref = EXCLUDED.icon_ref, \
                 tags_json = EXCLUDED.tags_json, \
                 attributes_json = EXCLUDED.attributes_json, \
                 status = COALESCE($13::smallint, ai_agent_category.status), \
                 updated_by = EXCLUDED.updated_by, \
                 updated_at = EXCLUDED.updated_at, \
                 deleted_at = NULL, \
                 deleted_by = NULL, \
                 version = ai_agent_category.version + 1 \
             WHERE ($18::bigint IS NULL OR ai_agent_category.version = $18::bigint) \
             RETURNING {CATEGORY_SELECT}"
        );
        let row = crate::pg_query_optional!(
            self.pool,
            sqlx::AssertSqlSafe(query.as_str()),
            id,
            uuid,
            tenant_id as i64,
            organization_id as i64,
            command.category_code.clone(),
            command.name.clone(),
            command.description.clone(),
            command.parent_id,
            command.sort_order,
            command.icon_ref.clone(),
            tags_json,
            attributes_json,
            status,
            operator_id as i64,
            operator_id as i64,
            now.clone(),
            now,
            command.expected_version
        )?;
        match row {
            Some(row) => row_to_category(&row),
            None if command.expected_version.is_some() => {
                Err(version_mismatch("agent category version mismatch"))
            }
            None => Err(KernelError::provider_error(
                "agent_category_store",
                "agent category upsert returned no row",
            )),
        }
    }

    fn delete(
        &self,
        tenant_id: u64,
        organization_id: u64,
        operator_id: u64,
        category_code: &str,
    ) -> KernelResult<AgentCategoryRecord> {
        let query = format!(
            "UPDATE ai_agent_category SET \
                 status = 4, deleted_at = CURRENT_TIMESTAMP, deleted_by = $3, \
                 updated_by = $3, version = version + 1, updated_at = CURRENT_TIMESTAMP \
             WHERE tenant_id = $1 AND organization_id = $2 AND category_code = $4 \
               AND deleted_at IS NULL \
             RETURNING {CATEGORY_SELECT}"
        );
        let row = crate::pg_query_optional!(
            self.pool,
            sqlx::AssertSqlSafe(query.as_str()),
            tenant_id as i64,
            organization_id as i64,
            operator_id as i64,
            category_code
        )?;
        match row {
            Some(row) => row_to_category(&row),
            None => Err(KernelError::not_found("agent category does not exist")),
        }
    }

    fn apply_sort_order(
        &self,
        tenant_id: u64,
        organization_id: u64,
        operator_id: u64,
        orders: &[(i64, i32)],
    ) -> KernelResult<()> {
        if orders.is_empty() {
            return Ok(());
        }
        let ids = orders.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let sort_orders = orders.iter().map(|(_, order)| *order).collect::<Vec<_>>();
        let affected = crate::pg_execute!(
            self.pool,
            sqlx::AssertSqlSafe(
                "WITH ordered AS ( \
                     SELECT * FROM unnest($4::bigint[], $5::integer[]) AS entries(category_id, sort_order) \
                 ) \
                 UPDATE ai_agent_category AS category \
                 SET sort_order = ordered.sort_order, \
                     version = category.version + 1, \
                     updated_by = $3, \
                     updated_at = CURRENT_TIMESTAMP \
                 FROM ordered \
                 WHERE category.tenant_id = $1 AND category.organization_id = $2 \
                   AND category.id = ordered.category_id AND category.deleted_at IS NULL"
            ),
            tenant_id as i64,
            organization_id as i64,
            operator_id as i64,
            ids,
            sort_orders
        )?;
        if affected != orders.len() as u64 {
            return Err(KernelError::not_found(
                "one or more agent categories no longer exist",
            ));
        }
        Ok(())
    }

    fn count_agents_in_category(
        &self,
        tenant_id: u64,
        organization_id: u64,
        category_code: &str,
    ) -> KernelResult<i64> {
        let rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(
                "SELECT COUNT(*) AS agent_count FROM ai_agent \
                 WHERE tenant_id = $1 AND organization_id = $2 AND category_code = $3 \
                   AND deleted_at IS NULL AND status = 1"
            ),
            tenant_id as i64,
            organization_id as i64,
            category_code
        )?;
        rows.first()
            .map(|row| {
                use sqlx::Row as _;
                row.try_get("agent_count")
                    .map_err(|error| KernelError::provider_error("agent_category_row", error.to_string()))
            })
            .unwrap_or(Ok(0))
    }
}

#[cfg(feature = "postgres-sync")]
fn row_to_category(row: &crate::postgres_sync_pool::PgRow) -> KernelResult<AgentCategoryRecord> {
    let status_code: i16 = row
        .try_get("status")
        .map_err(|error| KernelError::provider_error("agent_category_row", error.to_string()))?;
    Ok(AgentCategoryRecord {
        id: row.try_get("id").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        uuid: row.try_get("uuid").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        tenant_id: row.try_get("tenant_id").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        organization_id: row.try_get("organization_id").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        category_code: row.try_get("category_code").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        name: row.try_get("name").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        description: row.try_get("description").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        parent_id: row.try_get("parent_id").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        sort_order: row.try_get("sort_order").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        icon_ref: row.try_get("icon_ref").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        tags: json_string_list(row, "tags_json")?,
        attributes: json_string_map(row, "attributes_json")?,
        lifecycle_status: AgentCategoryStatus::from_db_code(status_code)?,
        version: row.try_get("version").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        created_at: row.try_get("created_at").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
        updated_at: row.try_get("updated_at").map_err(|error| {
            KernelError::provider_error("agent_category_row", error.to_string())
        })?,
    })
}

#[cfg(feature = "postgres-sync")]
fn json_string_list(
    row: &crate::postgres_sync_pool::PgRow,
    field: &str,
) -> KernelResult<Vec<String>> {
    use sqlx::Row as _;
    let value: serde_json::Value = row
        .try_get(field)
        .map_err(|error| KernelError::provider_error("agent_category_row", error.to_string()))?;
    let Some(items) = value.as_array() else {
        return Err(KernelError::provider_error(
            "agent_category_row",
            format!("{field} must be a json array"),
        ));
    };
    items
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                KernelError::provider_error(
                    "agent_category_row",
                    format!("{field} items must be strings"),
                )
            })
        })
        .collect()
}

#[cfg(feature = "postgres-sync")]
fn json_string_map(
    row: &crate::postgres_sync_pool::PgRow,
    field: &str,
) -> KernelResult<BTreeMap<String, String>> {
    use sqlx::Row as _;
    let value: serde_json::Value = row
        .try_get(field)
        .map_err(|error| KernelError::provider_error("agent_category_row", error.to_string()))?;
    let Some(entries) = value.as_object() else {
        return Err(KernelError::provider_error(
            "agent_category_row",
            format!("{field} must be a json object"),
        ));
    };
    entries
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|text| (key.clone(), text.to_owned()))
                .ok_or_else(|| {
                    KernelError::provider_error(
                        "agent_category_row",
                        format!("{field} values must be strings"),
                    )
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(category_code: &str, name: &str) -> UpsertAgentCategoryCommand {
        UpsertAgentCategoryCommand {
            category_code: category_code.to_owned(),
            name: name.to_owned(),
            description: None,
            parent_id: None,
            sort_order: 0,
            icon_ref: None,
            tags: Vec::new(),
            attributes: BTreeMap::new(),
            status: None,
            expected_version: None,
        }
    }

    fn record(
        id: i64,
        code: &str,
        parent_id: Option<i64>,
        status: AgentCategoryStatus,
    ) -> AgentCategoryRecord {
        AgentCategoryRecord {
            id,
            uuid: format!("uuid-{code}"),
            tenant_id: 1,
            organization_id: 0,
            category_code: code.to_owned(),
            name: code.to_owned(),
            description: None,
            parent_id,
            sort_order: 0,
            icon_ref: None,
            tags: Vec::new(),
            attributes: BTreeMap::new(),
            lifecycle_status: status,
            version: 0,
            created_at: "2026-01-01T00:00:00.000Z".to_owned(),
            updated_at: "2026-01-01T00:00:00.000Z".to_owned(),
        }
    }

    #[test]
    fn validates_category_code_shape() {
        assert!(validate_upsert_command(&command("education", "Education")).is_ok());
        assert!(validate_upsert_command(&command("x", "Too short")).is_err());
        assert!(validate_upsert_command(&command("Upper", "Upper")).is_err());
        assert!(validate_upsert_command(&command("bad space", "Bad")).is_err());
        assert!(validate_upsert_command(&command("-leading", "Bad")).is_err());
    }

    #[test]
    fn trims_and_normalizes_upsert_values() {
        let validated = validate_upsert_command(&UpsertAgentCategoryCommand {
            category_code: "  education  ".to_owned(),
            name: "  Education  ".to_owned(),
            description: Some("  K-12 teachers  ".to_owned()),
            parent_id: None,
            sort_order: 3,
            icon_ref: Some("  drive://icon/education  ".to_owned()),
            tags: vec![" K-12 ".to_owned(), "k-12".to_owned(), "Teachers".to_owned()],
            attributes: BTreeMap::new(),
            status: Some(AgentCategoryStatus::Active),
            expected_version: None,
        })
        .expect("valid command");
        assert_eq!(validated.category_code, "education");
        assert_eq!(validated.name, "Education");
        assert_eq!(validated.description.as_deref(), Some("K-12 teachers"));
        assert_eq!(
            validated.icon_ref.as_deref(),
            Some("drive://icon/education")
        );
        assert_eq!(validated.sort_order, 3);
        assert_eq!(validated.status, Some(AgentCategoryStatus::Active));
    }

    #[test]
    fn rejects_out_of_range_sort_order_and_deleted_status() {
        let mut too_large = command("education", "Education");
        too_large.sort_order = 10_000;
        assert!(validate_upsert_command(&too_large).is_err());

        let mut deleted = command("education", "Education");
        deleted.status = Some(AgentCategoryStatus::Deleted);
        assert!(validate_upsert_command(&deleted).is_err());
    }

    #[test]
    fn enforces_parent_active_cycle_and_depth_rules() {
        let categories = vec![
            record(1, "education", None, AgentCategoryStatus::Active),
            record(2, "education_k12", Some(1), AgentCategoryStatus::Active),
            record(3, "finance", None, AgentCategoryStatus::Disabled),
            record(4, "finance_ops", Some(3), AgentCategoryStatus::Active),
        ];

        // A third level is allowed now; a disabled parent is still rejected.
        assert!(validate_category_placement(&categories, "tech", Some(1)).is_ok());
        assert!(validate_category_placement(&categories, "tech", Some(2)).is_ok());
        assert!(validate_category_placement(&categories, "tech", Some(3)).is_err());
        assert!(
            validate_category_placement(&categories, "finance_ops", Some(3)).is_ok(),
            "an existing placement under a disabled parent must not block unrelated edits"
        );
        // Self-parenting and moving a category under its own descendant are cycles.
        assert!(validate_category_placement(&categories, "education", Some(1)).is_err());
        assert!(validate_category_placement(&categories, "education", Some(2)).is_err());
        assert!(validate_category_placement(&categories, "tech", Some(99)).is_err());
        assert!(validate_category_placement(&categories, "tech", None).is_ok());

        // A sixth level is refused.
        let mut deep = vec![record(10, "l1", None, AgentCategoryStatus::Active)];
        for id in 11..=15 {
            deep.push(record(
                id,
                &format!("l{}", id - 9),
                Some(id - 1),
                AgentCategoryStatus::Active,
            ));
        }
        assert!(validate_category_placement(&deep, "new", Some(13)).is_ok());
        assert!(validate_category_placement(&deep, "new", Some(14)).is_err());
    }

    #[test]
    fn in_memory_upsert_bumps_version_and_keeps_identity() {
        let store = InMemoryAgentCategoryStore::new();
        let first = store
            .upsert(
                7,
                9,
                42,
                &validate_upsert_command(&command("education", "Education")).expect("command"),
            )
            .expect("first upsert");
        let second = store
            .upsert(
                7,
                9,
                42,
                &validate_upsert_command(&command("education", "Education & Training"))
                    .expect("command"),
            )
            .expect("second upsert");

        assert_eq!(first.id, second.id);
        assert_eq!(first.uuid, second.uuid);
        assert_eq!(second.version, first.version + 1);
        assert_eq!(second.name, "Education & Training");
        assert_eq!(store.list(7, 9).expect("list").len(), 1);
        assert!(store.list(7, 10).expect("other scope").is_empty());
    }

    #[test]
    fn in_memory_upsert_enforces_expected_version() {
        let store = InMemoryAgentCategoryStore::new();
        store
            .upsert(
                7,
                9,
                42,
                &validate_upsert_command(&command("education", "Education")).expect("command"),
            )
            .expect("create");

        let mut update = command("education", "Education v2");
        update.expected_version = Some(0);
        store
            .upsert(
                7,
                9,
                42,
                &validate_upsert_command(&update).expect("command"),
            )
            .expect("matching version");

        update.expected_version = Some(0);
        let error = store
            .upsert(
                7,
                9,
                42,
                &validate_upsert_command(&update).expect("command"),
            )
            .expect_err("stale version must conflict");
        assert_eq!(
            error.detail_value(VERSION_MISMATCH_DETAIL),
            Some("true"),
            "stale updates must carry the version-mismatch marker"
        );
    }

    #[test]
    fn in_memory_upsert_persists_placement_and_status() {
        let store = InMemoryAgentCategoryStore::new();
        let mut parent = command("education", "Education");
        parent.icon_ref = Some("drive://icons/education".to_owned());
        let parent = store
            .upsert(
                7,
                9,
                42,
                &validate_upsert_command(&parent).expect("command"),
            )
            .expect("parent");

        let mut child = command("education_k12", "K-12");
        child.parent_id = Some(parent.id);
        child.sort_order = 5;
        child.status = Some(AgentCategoryStatus::Disabled);
        let child = store
            .upsert(7, 9, 42, &validate_upsert_command(&child).expect("command"))
            .expect("child");

        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.sort_order, 5);
        assert_eq!(child.lifecycle_status, AgentCategoryStatus::Disabled);
        assert_eq!(store.list(7, 9).expect("list").len(), 2);
    }

    #[test]
    fn category_uuid_is_stable_and_scoped() {
        assert_eq!(
            build_category_uuid(1, 2, "education"),
            build_category_uuid(1, 2, "education")
        );
        assert_ne!(
            build_category_uuid(1, 2, "education"),
            build_category_uuid(1, 3, "education")
        );
    }
}
