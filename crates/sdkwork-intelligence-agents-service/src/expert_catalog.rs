//! Expert catalog read model over the managed-agent schema.
//!
//! The SDKWork expert catalog stores every expert as an ordinary managed agent
//! (`ai_agent`, `agent.expert.<slug>`) grouped by an `ai_agent_category` row
//! (`expert-<division>`), plus three aggregates defined by migration `0010`:
//! `ai_expert_team`, `ai_expert_team_member` and `ai_expert_persona`. The
//! market's editorial layer from migration `0011` adds `ai_expert_profile`
//! (usage count, badge, catalog order, publish time) and the curated
//! collections `ai_expert_collection` / `ai_expert_collection_item`.
//!
//! The catalog is authored in `sdkwork-specs/specs/expert-catalog` and seeded by
//! `scripts/generate-expert-seeds.mjs`; this module only reads it. It follows the
//! managed category catalog's store pattern: [`InMemoryExpertCatalogStore`] for
//! local development and contract tests, and
//! [`PostgresExpertCatalogStore`] over the canonical tables for
//! server-authoritative deployments.

use std::sync::Mutex;

use sdkwork_agent_kernel::{KernelError, KernelResult};
use serde::Serialize;

#[cfg(feature = "postgres-sync")]
use sqlx::Row;

#[cfg(feature = "postgres-sync")]
use crate::postgres_sync_pool::BlockingPostgresPool;

/// Catalog locale ids carried by the persona rows.
pub const EXPERT_CATALOG_LOCALES: [&str; 2] = ["zh-CN", "en-US"];
/// Locale served when the requested persona is absent.
pub const EXPERT_CATALOG_DEFAULT_LOCALE: &str = "zh-CN";

/// One expert division (the catalog taxonomy level).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertDivisionRecord {
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub expert_count: u64,
}

/// One expert summary: the managed agent's stable identity, display data and
/// the market profile joined from `ai_expert_profile`.
///
/// The profile columns are optional at the database level, so an expert
/// without a profile row serves `usageCount: 0`, `badgeCode: null`,
/// `sortOrder: 0` and `publishedAt: null`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertRecord {
    pub slug: String,
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub emoji: String,
    pub division_code: String,
    pub tags: Vec<String>,
    /// Curated editorial usage figure — never telemetry.
    pub usage_count: i32,
    pub badge_code: Option<String>,
    /// Catalog order of the expert inside its division.
    pub sort_order: i32,
    pub published_at: Option<String>,
}

/// One localized expert persona body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpertPersonaRecord {
    pub expert_agent_id: String,
    pub expert_slug: String,
    pub locale: String,
    pub persona_text: String,
    pub content_sha256: String,
}

/// One ordered expert-team member, both display locales.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpertTeamMemberRow {
    pub expert_agent_id: String,
    pub expert_slug: String,
    pub position: i32,
    pub duty: String,
    pub duty_en: String,
    pub instructions: String,
    pub instructions_en: String,
}

/// One expert team, both display locales and its ordered members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpertTeamRow {
    pub team_id: String,
    pub name: String,
    pub name_en: String,
    pub description: String,
    pub description_en: String,
    pub tags: Vec<String>,
    pub tags_en: Vec<String>,
    pub goal: String,
    pub goal_en: String,
    /// The team's shared constraints. Authored in one language: the catalog
    /// spec carries a single `constraints` string per team.
    pub constraints: String,
    pub delivery_requirements: String,
    pub delivery_requirements_en: String,
    pub examples: Vec<String>,
    pub examples_en: Vec<String>,
    pub coordinator_mode: String,
    pub coordinator_template_id: String,
    /// A custom coordinator's own prompt. Template coordinators leave it empty:
    /// the package resolves their text from `coordinator_template_id`.
    pub coordinator_prompt: String,
    pub sort_order: i32,
    pub members: Vec<ExpertTeamMemberRow>,
}

impl ExpertTeamRow {
    /// The localized (name, description, tags, goal, delivery, examples) slice.
    pub fn localized(&self, locale: &str) -> (&str, &str, &[String], &str, &str, &[String]) {
        if locale == "en-US" {
            (
                &self.name_en,
                &self.description_en,
                &self.tags_en,
                &self.goal_en,
                &self.delivery_requirements_en,
                &self.examples_en,
            )
        } else {
            (
                &self.name,
                &self.description,
                &self.tags,
                &self.goal,
                &self.delivery_requirements,
                &self.examples,
            )
        }
    }
}

impl ExpertTeamMemberRow {
    /// The localized (duty, instructions) pair.
    pub fn localized(&self, locale: &str) -> (&str, &str) {
        if locale == "en-US" {
            (&self.duty_en, &self.instructions_en)
        } else {
            (&self.duty, &self.instructions)
        }
    }
}

/// One ordered member of a curated expert collection, both display locales.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpertCollectionItemRow {
    pub expert_agent_id: String,
    pub expert_slug: String,
    pub position: i32,
    pub label: String,
    pub label_en: String,
}

/// One curated expert collection (the market's featured shelf), both display
/// locales and its ordered members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpertCollectionRow {
    pub collection_code: String,
    pub name: String,
    pub name_en: String,
    pub description: String,
    pub description_en: String,
    pub cover_key: String,
    pub sort_order: i32,
    pub items: Vec<ExpertCollectionItemRow>,
}

impl ExpertCollectionRow {
    /// The localized (name, description) pair.
    pub fn localized(&self, locale: &str) -> (&str, &str) {
        if locale == "en-US" {
            (&self.name_en, &self.description_en)
        } else {
            (&self.name, &self.description)
        }
    }
}

impl ExpertCollectionItemRow {
    /// The localized member label.
    pub fn localized(&self, locale: &str) -> &str {
        if locale == "en-US" {
            &self.label_en
        } else {
            &self.label
        }
    }
}

/// Persistence contract for the expert catalog read model.
pub trait ExpertCatalogStore: Send + Sync {
    fn divisions(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> KernelResult<Vec<ExpertDivisionRecord>>;

    fn experts(&self, tenant_id: u64, organization_id: u64) -> KernelResult<Vec<ExpertRecord>>;

    fn expert(
        &self,
        tenant_id: u64,
        organization_id: u64,
        slug: &str,
    ) -> KernelResult<Option<ExpertRecord>>;

    fn persona(
        &self,
        tenant_id: u64,
        organization_id: u64,
        expert_agent_id: &str,
        locale: &str,
    ) -> KernelResult<Option<ExpertPersonaRecord>>;

    fn personas(
        &self,
        tenant_id: u64,
        organization_id: u64,
        locale: &str,
    ) -> KernelResult<Vec<ExpertPersonaRecord>>;

    fn teams(&self, tenant_id: u64, organization_id: u64) -> KernelResult<Vec<ExpertTeamRow>>;

    fn team(
        &self,
        tenant_id: u64,
        organization_id: u64,
        team_id: &str,
    ) -> KernelResult<Option<ExpertTeamRow>>;

    fn collections(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> KernelResult<Vec<ExpertCollectionRow>>;

    fn collection(
        &self,
        tenant_id: u64,
        organization_id: u64,
        collection_code: &str,
    ) -> KernelResult<Option<ExpertCollectionRow>>;
}

/// Local-development catalog backing: empty by default, seeded explicitly by
/// tests (`with_*` builders) that need a populated catalog.
#[derive(Default)]
pub struct InMemoryExpertCatalogStore {
    divisions: Mutex<Vec<ExpertDivisionRecord>>,
    experts: Mutex<Vec<ExpertRecord>>,
    personas: Mutex<Vec<ExpertPersonaRecord>>,
    teams: Mutex<Vec<ExpertTeamRow>>,
    collections: Mutex<Vec<ExpertCollectionRow>>,
}

impl InMemoryExpertCatalogStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_divisions(self, divisions: Vec<ExpertDivisionRecord>) -> Self {
        *self.divisions.lock().unwrap() = divisions;
        self
    }

    pub fn with_experts(self, experts: Vec<ExpertRecord>) -> Self {
        *self.experts.lock().unwrap() = experts;
        self
    }

    pub fn with_personas(self, personas: Vec<ExpertPersonaRecord>) -> Self {
        *self.personas.lock().unwrap() = personas;
        self
    }

    pub fn with_teams(self, teams: Vec<ExpertTeamRow>) -> Self {
        *self.teams.lock().unwrap() = teams;
        self
    }

    pub fn with_collections(self, collections: Vec<ExpertCollectionRow>) -> Self {
        *self.collections.lock().unwrap() = collections;
        self
    }
}

impl ExpertCatalogStore for InMemoryExpertCatalogStore {
    fn divisions(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
    ) -> KernelResult<Vec<ExpertDivisionRecord>> {
        Ok(self.divisions.lock().unwrap().clone())
    }

    fn experts(&self, _tenant_id: u64, _organization_id: u64) -> KernelResult<Vec<ExpertRecord>> {
        Ok(self.experts.lock().unwrap().clone())
    }

    fn expert(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        slug: &str,
    ) -> KernelResult<Option<ExpertRecord>> {
        Ok(self
            .experts
            .lock()
            .unwrap()
            .iter()
            .find(|expert| expert.slug == slug)
            .cloned())
    }

    fn persona(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        expert_agent_id: &str,
        locale: &str,
    ) -> KernelResult<Option<ExpertPersonaRecord>> {
        Ok(self
            .personas
            .lock()
            .unwrap()
            .iter()
            .find(|persona| persona.expert_agent_id == expert_agent_id && persona.locale == locale)
            .cloned())
    }

    fn personas(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        locale: &str,
    ) -> KernelResult<Vec<ExpertPersonaRecord>> {
        Ok(self
            .personas
            .lock()
            .unwrap()
            .iter()
            .filter(|persona| persona.locale == locale)
            .cloned()
            .collect())
    }

    fn teams(&self, _tenant_id: u64, _organization_id: u64) -> KernelResult<Vec<ExpertTeamRow>> {
        Ok(self.teams.lock().unwrap().clone())
    }

    fn team(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        team_id: &str,
    ) -> KernelResult<Option<ExpertTeamRow>> {
        Ok(self
            .teams
            .lock()
            .unwrap()
            .iter()
            .find(|team| team.team_id == team_id)
            .cloned())
    }

    fn collections(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
    ) -> KernelResult<Vec<ExpertCollectionRow>> {
        Ok(self.collections.lock().unwrap().clone())
    }

    fn collection(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        collection_code: &str,
    ) -> KernelResult<Option<ExpertCollectionRow>> {
        Ok(self
            .collections
            .lock()
            .unwrap()
            .iter()
            .find(|collection| collection.collection_code == collection_code)
            .cloned())
    }
}

#[cfg(feature = "postgres-sync")]
const EXPERT_SELECT: &str = "a.agent_id, \
    COALESCE(a.display_name, '') AS display_name, \
    COALESCE(a.description, '') AS description, \
    COALESCE(a.manifest_json #>> '{expert,slug}', '') AS slug, \
    COALESCE(a.manifest_json #>> '{expert,emoji}', '') AS emoji, \
    COALESCE(a.manifest_json #>> '{expert,division_code}', '') AS division_code, \
    a.tags_json::text AS tags_json, \
    COALESCE(p.usage_count, 0) AS usage_count, \
    p.badge_code AS badge_code, \
    COALESCE(p.sort_order, 0) AS sort_order, \
    to_char(p.published_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS published_at";

/// The expert profile is optional: a LEFT JOIN keeps experts without an
/// editorial profile row serving with neutral defaults.
#[cfg(feature = "postgres-sync")]
const EXPERT_PROFILE_JOIN: &str = "LEFT JOIN ai_expert_profile p \
    ON p.tenant_id = a.tenant_id AND p.organization_id = a.organization_id \
   AND p.expert_agent_id = a.agent_id AND p.deleted_at IS NULL";

#[cfg(feature = "postgres-sync")]
const COLLECTION_SELECT: &str = "collection_code, name, name_en, description, description_en, \
    cover_key, sort_order";

#[cfg(feature = "postgres-sync")]
const COLLECTION_ITEM_SELECT: &str = "i.collection_code, i.expert_agent_id, i.position, \
    i.label, i.label_en, \
    COALESCE(a.manifest_json #>> '{expert,slug}', '') AS expert_slug";

#[cfg(feature = "postgres-sync")]
const TEAM_SELECT: &str = "team_id, name, name_en, description, description_en, \
    tags_json::text AS tags_json, tags_en_json::text AS tags_en_json, \
    goal, goal_en, constraints_text, delivery_requirements, delivery_requirements_en, \
    examples_json::text AS examples_json, examples_en_json::text AS examples_en_json, \
    coordinator_mode, coordinator_template_id, coordinator_prompt, sort_order";

#[cfg(feature = "postgres-sync")]
const TEAM_MEMBER_SELECT: &str = "m.team_id, m.expert_agent_id, m.position, \
    m.duty, m.duty_en, m.instructions, m.instructions_en, \
    COALESCE(a.manifest_json #>> '{expert,slug}', '') AS expert_slug";

/// PostgreSQL-backed expert catalog over the canonical managed-agent tables.
#[cfg(feature = "postgres-sync")]
pub struct PostgresExpertCatalogStore {
    pool: BlockingPostgresPool,
}

#[cfg(feature = "postgres-sync")]
impl PostgresExpertCatalogStore {
    pub fn from_pool(pool: BlockingPostgresPool) -> KernelResult<Self> {
        Ok(Self { pool })
    }
}

#[cfg(feature = "postgres-sync")]
fn row_error(error: sqlx::Error) -> KernelError {
    KernelError::provider_error("expert_catalog_row", error.to_string())
}

#[cfg(feature = "postgres-sync")]
fn string_list(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

#[cfg(feature = "postgres-sync")]
fn team_from_rows(team: &sqlx::postgres::PgRow, members: &[sqlx::postgres::PgRow]) -> KernelResult<ExpertTeamRow> {
    let team_id: String = team.try_get("team_id").map_err(row_error)?;
    let mut ordered = Vec::new();
    for member in members {
        let member_team: String = member.try_get("team_id").map_err(row_error)?;
        if member_team != team_id {
            continue;
        }
        ordered.push(ExpertTeamMemberRow {
            expert_agent_id: member.try_get("expert_agent_id").map_err(row_error)?,
            expert_slug: member.try_get("expert_slug").map_err(row_error)?,
            position: member.try_get("position").map_err(row_error)?,
            duty: member.try_get("duty").map_err(row_error)?,
            duty_en: member.try_get("duty_en").map_err(row_error)?,
            instructions: member.try_get("instructions").map_err(row_error)?,
            instructions_en: member.try_get("instructions_en").map_err(row_error)?,
        });
    }
    Ok(ExpertTeamRow {
        team_id,
        name: team.try_get("name").map_err(row_error)?,
        name_en: team.try_get("name_en").map_err(row_error)?,
        description: team.try_get("description").map_err(row_error)?,
        description_en: team.try_get("description_en").map_err(row_error)?,
        tags: string_list(&team.try_get::<String, _>("tags_json").map_err(row_error)?),
        tags_en: string_list(&team.try_get::<String, _>("tags_en_json").map_err(row_error)?),
        goal: team.try_get("goal").map_err(row_error)?,
        goal_en: team.try_get("goal_en").map_err(row_error)?,
        constraints: team.try_get("constraints_text").map_err(row_error)?,
        delivery_requirements: team.try_get("delivery_requirements").map_err(row_error)?,
        delivery_requirements_en: team
            .try_get("delivery_requirements_en")
            .map_err(row_error)?,
        examples: string_list(&team.try_get::<String, _>("examples_json").map_err(row_error)?),
        examples_en: string_list(&team.try_get::<String, _>("examples_en_json").map_err(row_error)?),
        coordinator_mode: team.try_get("coordinator_mode").map_err(row_error)?,
        coordinator_template_id: team.try_get("coordinator_template_id").map_err(row_error)?,
        coordinator_prompt: team.try_get("coordinator_prompt").map_err(row_error)?,
        sort_order: team.try_get("sort_order").map_err(row_error)?,
        members: ordered,
    })
}

#[cfg(feature = "postgres-sync")]
impl ExpertCatalogStore for PostgresExpertCatalogStore {
    fn divisions(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> KernelResult<Vec<ExpertDivisionRecord>> {
        let query = "SELECT c.category_code, c.name, c.sort_order, \
            (SELECT count(*) FROM ai_agent a \
              WHERE a.tenant_id = c.tenant_id AND a.organization_id = c.organization_id \
                AND a.category_code = c.category_code AND a.deleted_at IS NULL AND a.status = 1 \
                AND a.manifest_json->>'domain' = 'experts') AS expert_count \
            FROM ai_agent_category c \
            WHERE c.tenant_id = $1 AND c.organization_id = $2 AND c.deleted_at IS NULL \
              AND c.status = 1 AND c.category_code LIKE 'expert-%' \
            ORDER BY c.sort_order ASC, c.category_code ASC";
        let rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(query),
            tenant_id as i64,
            organization_id as i64
        )?;
        rows.iter()
            .map(|row| {
                Ok(ExpertDivisionRecord {
                    code: row.try_get("category_code").map_err(row_error)?,
                    name: row.try_get("name").map_err(row_error)?,
                    sort_order: row.try_get("sort_order").map_err(row_error)?,
                    expert_count: row
                        .try_get::<i64, _>("expert_count")
                        .map_err(row_error)? as u64,
                })
            })
            .collect()
    }

    fn experts(&self, tenant_id: u64, organization_id: u64) -> KernelResult<Vec<ExpertRecord>> {
        let query = format!(
            "SELECT {EXPERT_SELECT} FROM ai_agent a {EXPERT_PROFILE_JOIN} \
             WHERE a.tenant_id = $1 AND a.organization_id = $2 \
               AND a.deleted_at IS NULL AND a.status = 1 \
               AND a.manifest_json->>'domain' = 'experts' \
               AND a.manifest_json->>'manifest_type' = 'agent' \
             ORDER BY a.display_name ASC, a.agent_id ASC"
        );
        let rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(query.as_str()),
            tenant_id as i64,
            organization_id as i64
        )?;
        rows.iter().map(row_to_expert).collect()
    }

    fn expert(
        &self,
        tenant_id: u64,
        organization_id: u64,
        slug: &str,
    ) -> KernelResult<Option<ExpertRecord>> {
        let query = format!(
            "SELECT {EXPERT_SELECT} FROM ai_agent a {EXPERT_PROFILE_JOIN} \
             WHERE a.tenant_id = $1 AND a.organization_id = $2 \
               AND a.deleted_at IS NULL AND a.status = 1 \
               AND a.manifest_json->>'domain' = 'experts' \
               AND a.manifest_json #>> '{{expert,slug}}' = $3 \
             LIMIT 1"
        );
        let row = crate::pg_query_optional!(
            self.pool,
            sqlx::AssertSqlSafe(query.as_str()),
            tenant_id as i64,
            organization_id as i64,
            slug
        )?;
        row.as_ref().map(row_to_expert).transpose()
    }

    fn persona(
        &self,
        tenant_id: u64,
        organization_id: u64,
        expert_agent_id: &str,
        locale: &str,
    ) -> KernelResult<Option<ExpertPersonaRecord>> {
        let query = "SELECT p.expert_agent_id, \
            COALESCE(a.manifest_json #>> '{expert,slug}', '') AS expert_slug, \
            p.locale, p.persona_text, p.content_sha256 \
            FROM ai_expert_persona p \
            LEFT JOIN ai_agent a \
              ON a.tenant_id = p.tenant_id AND a.organization_id = p.organization_id \
             AND a.agent_id = p.expert_agent_id \
            WHERE p.tenant_id = $1 AND p.organization_id = $2 \
              AND p.expert_agent_id = $3 AND p.locale = $4 \
            LIMIT 1";
        let row = crate::pg_query_optional!(
            self.pool,
            sqlx::AssertSqlSafe(query),
            tenant_id as i64,
            organization_id as i64,
            expert_agent_id,
            locale
        )?;
        row.as_ref()
            .map(|row| {
                Ok(ExpertPersonaRecord {
                    expert_agent_id: row.try_get("expert_agent_id").map_err(row_error)?,
                    expert_slug: row.try_get("expert_slug").map_err(row_error)?,
                    locale: row.try_get("locale").map_err(row_error)?,
                    persona_text: row.try_get("persona_text").map_err(row_error)?,
                    content_sha256: row.try_get("content_sha256").map_err(row_error)?,
                })
            })
            .transpose()
    }

    fn personas(
        &self,
        tenant_id: u64,
        organization_id: u64,
        locale: &str,
    ) -> KernelResult<Vec<ExpertPersonaRecord>> {
        let query = "SELECT p.expert_agent_id, \
            COALESCE(a.manifest_json #>> '{expert,slug}', '') AS expert_slug, \
            p.locale, p.persona_text, p.content_sha256 \
            FROM ai_expert_persona p \
            LEFT JOIN ai_agent a \
              ON a.tenant_id = p.tenant_id AND a.organization_id = p.organization_id \
             AND a.agent_id = p.expert_agent_id \
            WHERE p.tenant_id = $1 AND p.organization_id = $2 AND p.locale = $3 \
            ORDER BY p.expert_agent_id ASC";
        let rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(query),
            tenant_id as i64,
            organization_id as i64,
            locale
        )?;
        rows.iter()
            .map(|row| {
                Ok(ExpertPersonaRecord {
                    expert_agent_id: row.try_get("expert_agent_id").map_err(row_error)?,
                    expert_slug: row.try_get("expert_slug").map_err(row_error)?,
                    locale: row.try_get("locale").map_err(row_error)?,
                    persona_text: row.try_get("persona_text").map_err(row_error)?,
                    content_sha256: row.try_get("content_sha256").map_err(row_error)?,
                })
            })
            .collect()
    }

    fn teams(&self, tenant_id: u64, organization_id: u64) -> KernelResult<Vec<ExpertTeamRow>> {
        let teams_query = format!(
            "SELECT {TEAM_SELECT} FROM ai_expert_team \
             WHERE tenant_id = $1 AND organization_id = $2 AND deleted_at IS NULL AND status = 1 \
             ORDER BY sort_order ASC, team_id ASC"
        );
        let members_query = format!(
            "SELECT {TEAM_MEMBER_SELECT} \
             FROM ai_expert_team_member m \
             LEFT JOIN ai_agent a \
               ON a.tenant_id = m.tenant_id AND a.organization_id = m.organization_id \
              AND a.agent_id = m.expert_agent_id \
             WHERE m.tenant_id = $1 AND m.organization_id = $2 AND m.deleted_at IS NULL \
             ORDER BY m.team_id ASC, m.position ASC"
        );
        let team_rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(teams_query.as_str()),
            tenant_id as i64,
            organization_id as i64
        )?;
        let member_rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(members_query.as_str()),
            tenant_id as i64,
            organization_id as i64
        )?;
        team_rows
            .iter()
            .map(|team| team_from_rows(team, &member_rows))
            .collect()
    }

    fn team(
        &self,
        tenant_id: u64,
        organization_id: u64,
        team_id: &str,
    ) -> KernelResult<Option<ExpertTeamRow>> {
        let teams_query = format!(
            "SELECT {TEAM_SELECT} FROM ai_expert_team \
             WHERE tenant_id = $1 AND organization_id = $2 AND team_id = $3 \
               AND deleted_at IS NULL AND status = 1 \
             LIMIT 1"
        );
        let members_query = format!(
            "SELECT {TEAM_MEMBER_SELECT} \
             FROM ai_expert_team_member m \
             LEFT JOIN ai_agent a \
               ON a.tenant_id = m.tenant_id AND a.organization_id = m.organization_id \
              AND a.agent_id = m.expert_agent_id \
             WHERE m.tenant_id = $1 AND m.organization_id = $2 AND m.team_id = $3 \
               AND m.deleted_at IS NULL \
             ORDER BY m.position ASC"
        );
        let team_row = crate::pg_query_optional!(
            self.pool,
            sqlx::AssertSqlSafe(teams_query.as_str()),
            tenant_id as i64,
            organization_id as i64,
            team_id
        )?;
        let Some(team) = team_row else {
            return Ok(None);
        };
        let member_rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(members_query.as_str()),
            tenant_id as i64,
            organization_id as i64,
            team_id
        )?;
        Ok(Some(team_from_rows(&team, &member_rows)?))
    }

    fn collections(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> KernelResult<Vec<ExpertCollectionRow>> {
        let collections_query = format!(
            "SELECT {COLLECTION_SELECT} FROM ai_expert_collection \
             WHERE tenant_id = $1 AND organization_id = $2 AND deleted_at IS NULL AND status = 1 \
             ORDER BY sort_order ASC, collection_code ASC"
        );
        let items_query = format!(
            "SELECT {COLLECTION_ITEM_SELECT} \
             FROM ai_expert_collection_item i \
             LEFT JOIN ai_agent a \
               ON a.tenant_id = i.tenant_id AND a.organization_id = i.organization_id \
              AND a.agent_id = i.expert_agent_id \
             WHERE i.tenant_id = $1 AND i.organization_id = $2 AND i.deleted_at IS NULL \
             ORDER BY i.collection_code ASC, i.position ASC"
        );
        let collection_rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(collections_query.as_str()),
            tenant_id as i64,
            organization_id as i64
        )?;
        let item_rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(items_query.as_str()),
            tenant_id as i64,
            organization_id as i64
        )?;
        collection_rows
            .iter()
            .map(|collection| collection_from_rows(collection, &item_rows))
            .collect()
    }

    fn collection(
        &self,
        tenant_id: u64,
        organization_id: u64,
        collection_code: &str,
    ) -> KernelResult<Option<ExpertCollectionRow>> {
        let collections_query = format!(
            "SELECT {COLLECTION_SELECT} FROM ai_expert_collection \
             WHERE tenant_id = $1 AND organization_id = $2 AND collection_code = $3 \
               AND deleted_at IS NULL AND status = 1 \
             LIMIT 1"
        );
        let items_query = format!(
            "SELECT {COLLECTION_ITEM_SELECT} \
             FROM ai_expert_collection_item i \
             LEFT JOIN ai_agent a \
               ON a.tenant_id = i.tenant_id AND a.organization_id = i.organization_id \
              AND a.agent_id = i.expert_agent_id \
             WHERE i.tenant_id = $1 AND i.organization_id = $2 AND i.collection_code = $3 \
               AND i.deleted_at IS NULL \
             ORDER BY i.position ASC"
        );
        let collection_row = crate::pg_query_optional!(
            self.pool,
            sqlx::AssertSqlSafe(collections_query.as_str()),
            tenant_id as i64,
            organization_id as i64,
            collection_code
        )?;
        let Some(collection) = collection_row else {
            return Ok(None);
        };
        let item_rows = crate::pg_query!(
            self.pool,
            sqlx::AssertSqlSafe(items_query.as_str()),
            tenant_id as i64,
            organization_id as i64,
            collection_code
        )?;
        Ok(Some(collection_from_rows(&collection, &item_rows)?))
    }
}

#[cfg(feature = "postgres-sync")]
fn collection_from_rows(
    collection: &sqlx::postgres::PgRow,
    items: &[sqlx::postgres::PgRow],
) -> KernelResult<ExpertCollectionRow> {
    let collection_code: String = collection.try_get("collection_code").map_err(row_error)?;
    let mut ordered = Vec::new();
    for item in items {
        let item_collection: String = item.try_get("collection_code").map_err(row_error)?;
        if item_collection != collection_code {
            continue;
        }
        ordered.push(ExpertCollectionItemRow {
            expert_agent_id: item.try_get("expert_agent_id").map_err(row_error)?,
            expert_slug: item.try_get("expert_slug").map_err(row_error)?,
            position: item.try_get("position").map_err(row_error)?,
            label: item.try_get("label").map_err(row_error)?,
            label_en: item.try_get("label_en").map_err(row_error)?,
        });
    }
    Ok(ExpertCollectionRow {
        collection_code,
        name: collection.try_get("name").map_err(row_error)?,
        name_en: collection.try_get("name_en").map_err(row_error)?,
        description: collection.try_get("description").map_err(row_error)?,
        description_en: collection.try_get("description_en").map_err(row_error)?,
        cover_key: collection.try_get("cover_key").map_err(row_error)?,
        sort_order: collection.try_get("sort_order").map_err(row_error)?,
        items: ordered,
    })
}

#[cfg(feature = "postgres-sync")]
fn row_to_expert(row: &sqlx::postgres::PgRow) -> KernelResult<ExpertRecord> {
    Ok(ExpertRecord {
        slug: row.try_get("slug").map_err(row_error)?,
        agent_id: row.try_get("agent_id").map_err(row_error)?,
        name: row.try_get("display_name").map_err(row_error)?,
        description: row.try_get("description").map_err(row_error)?,
        emoji: row.try_get("emoji").map_err(row_error)?,
        division_code: {
            let category: String = row.try_get("division_code").map_err(row_error)?;
            category
                .strip_prefix("expert-")
                .map(str::to_owned)
                .unwrap_or(category)
        },
        tags: display_tags(
            &string_list(&row.try_get::<String, _>("tags_json").map_err(row_error)?),
            &row.try_get::<String, _>("division_code").map_err(row_error)?,
        ),
        usage_count: row.try_get("usage_count").map_err(row_error)?,
        badge_code: row.try_get("badge_code").map_err(row_error)?,
        sort_order: row.try_get("sort_order").map_err(row_error)?,
        published_at: row.try_get("published_at").map_err(row_error)?,
    })
}

/// The managed-agent tag vocabulary that describes the record rather than the
/// expert. A card renders `tags` as its subject chips, so the catalog serves
/// the descriptive remainder and drops these plus the division the card already
/// shows through its category facet.
#[cfg(feature = "postgres-sync")]
const SYSTEM_EXPERT_TAGS: [&str; 4] = ["expert", "conversation", "official", "commercial"];

/// Keep only the tags that describe the expert's subject.
#[cfg(feature = "postgres-sync")]
fn display_tags(tags: &[String], category_code: &str) -> Vec<String> {
    let division = category_code
        .strip_prefix("expert-")
        .unwrap_or(category_code)
        .to_lowercase();
    tags.iter()
        .filter(|tag| {
            let lower = tag.to_lowercase();
            !SYSTEM_EXPERT_TAGS.contains(&lower.as_str()) && lower != division
        })
        .cloned()
        .collect()
}

/// Normalize a requested locale to one of the catalog locales.
pub fn normalize_catalog_locale(value: Option<&str>) -> &'static str {
    match value.map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case("en-us") || value.eq_ignore_ascii_case("en") => "en-US",
        _ => EXPERT_CATALOG_DEFAULT_LOCALE,
    }
}

/// Locale fallback order for a persona lookup.
pub fn persona_locale_candidates(locale: &str) -> Vec<&'static str> {
    if locale == "en-US" {
        vec!["en-US", "zh-CN"]
    } else {
        vec!["zh-CN", "en-US"]
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn expert(slug: &str, division: &str) -> ExpertRecord {
        ExpertRecord {
            slug: slug.to_owned(),
            agent_id: format!("agent.expert.{slug}"),
            name: format!("expert {slug}"),
            description: "desc".to_owned(),
            emoji: "🧩".to_owned(),
            division_code: division.to_owned(),
            tags: vec!["expert".to_owned(), division.to_owned()],
            usage_count: 12_800,
            badge_code: Some("leader".to_owned()),
            sort_order: 0,
            published_at: Some("2026-11-20T00:00:00Z".to_owned()),
        }
    }

    fn collection(collection_code: &str) -> ExpertCollectionRow {
        ExpertCollectionRow {
            collection_code: collection_code.to_owned(),
            name: "电商运营".to_owned(),
            name_en: "E-commerce Operations".to_owned(),
            description: "精选电商运营专家。".to_owned(),
            description_en: "Curated e-commerce operators.".to_owned(),
            cover_key: collection_code.to_owned(),
            sort_order: 10,
            items: vec![ExpertCollectionItemRow {
                expert_agent_id: "agent.expert.marketing-ecommerce-operator".to_owned(),
                expert_slug: "marketing-ecommerce-operator".to_owned(),
                position: 0,
                label: "电商运营专家".to_owned(),
                label_en: "E-commerce Operations Specialist".to_owned(),
            }],
        }
    }

    fn team(team_id: &str) -> ExpertTeamRow {
        ExpertTeamRow {
            team_id: team_id.to_owned(),
            name: "团队".to_owned(),
            name_en: "Team".to_owned(),
            description: "描述".to_owned(),
            description_en: "description".to_owned(),
            tags: vec!["评审".to_owned()],
            tags_en: vec!["review".to_owned()],
            goal: "目标".to_owned(),
            goal_en: "goal".to_owned(),
        constraints: "only review".to_owned(),
            delivery_requirements: "交付".to_owned(),
            delivery_requirements_en: "delivery".to_owned(),
            examples: vec!["示例".to_owned()],
            examples_en: vec!["example".to_owned()],
            coordinator_mode: "template".to_owned(),
            coordinator_template_id: "product".to_owned(),
        coordinator_prompt: String::new(),
            sort_order: 10,
            members: vec![ExpertTeamMemberRow {
                expert_agent_id: "agent.expert.product-manager".to_owned(),
                expert_slug: "product-manager".to_owned(),
                position: 0,
                duty: "需求".to_owned(),
                duty_en: "requirements".to_owned(),
                instructions: "检查".to_owned(),
                instructions_en: "check".to_owned(),
            }],
        }
    }

    #[test]
    fn in_memory_store_serves_the_catalog() {
        let store = InMemoryExpertCatalogStore::new()
            .with_experts(vec![expert("product-manager", "product")])
            .with_personas(vec![ExpertPersonaRecord {
                expert_agent_id: "agent.expert.product-manager".to_owned(),
                expert_slug: "product-manager".to_owned(),
                locale: "zh-CN".to_owned(),
                persona_text: "persona".to_owned(),
                content_sha256: "a".repeat(64),
            }])
            .with_teams(vec![team("team-product")])
            .with_collections(vec![collection("collection-ecommerce")]);
        assert_eq!(store.experts(1, 0).unwrap().len(), 1);
        assert_eq!(
            store.expert(1, 0, "product-manager").unwrap().unwrap().division_code,
            "product"
        );
        assert!(store.expert(1, 0, "missing").unwrap().is_none());
        assert!(store
            .persona(1, 0, "agent.expert.product-manager", "zh-CN")
            .unwrap()
            .is_some());
        assert!(store
            .persona(1, 0, "agent.expert.product-manager", "en-US")
            .unwrap()
            .is_none());
        assert_eq!(store.teams(1, 0).unwrap().len(), 1);
        assert_eq!(
            store.team(1, 0, "team-product").unwrap().unwrap().members.len(),
            1
        );
        assert_eq!(store.experts(1, 0).unwrap()[0].usage_count, 12_800);
        assert_eq!(
            store.experts(1, 0).unwrap()[0].badge_code.as_deref(),
            Some("leader")
        );
        assert_eq!(store.collections(1, 0).unwrap().len(), 1);
        assert_eq!(
            store
                .collection(1, 0, "collection-ecommerce")
                .unwrap()
                .unwrap()
                .items
                .len(),
            1
        );
        assert!(store.collection(1, 0, "missing").unwrap().is_none());
    }

    #[test]
    fn localizes_collections_and_member_labels() {
        let collection = collection("collection-ecommerce");
        assert_eq!(
            collection.localized("zh-CN"),
            ("电商运营", "精选电商运营专家。")
        );
        assert_eq!(
            collection.localized("en-US"),
            ("E-commerce Operations", "Curated e-commerce operators.")
        );
        assert_eq!(collection.items[0].localized("zh-CN"), "电商运营专家");
        assert_eq!(
            collection.items[0].localized("en-US"),
            "E-commerce Operations Specialist"
        );
    }

    #[test]
    fn expert_without_profile_serializes_with_neutral_defaults() {
        // The profile is a LEFT JOIN: an expert with no `ai_expert_profile` row
        // must still serialize with zero usage, a null badge and no publish time.
        let value = serde_json::to_value(ExpertRecord {
            usage_count: 0,
            badge_code: None,
            sort_order: 0,
            published_at: None,
            ..expert("product-manager", "product")
        })
        .unwrap();
        assert_eq!(value["usageCount"], 0);
        assert!(value["badgeCode"].is_null());
        assert_eq!(value["sortOrder"], 0);
        assert!(value["publishedAt"].is_null());
    }

    #[test]
    fn locale_normalization_and_fallback() {
        assert_eq!(normalize_catalog_locale(Some("en")), "en-US");
        assert_eq!(normalize_catalog_locale(Some("EN-US")), "en-US");
        assert_eq!(normalize_catalog_locale(Some("fr-FR")), "zh-CN");
        assert_eq!(normalize_catalog_locale(None), "zh-CN");
        assert_eq!(persona_locale_candidates("en-US"), vec!["en-US", "zh-CN"]);
        assert_eq!(persona_locale_candidates("zh-CN"), vec!["zh-CN", "en-US"]);
    }

    #[cfg(feature = "postgres-sync")]
    #[test]
    fn serves_only_the_descriptive_tags() {
        // A managed expert's tags mix record vocabulary (`expert`,
        // `conversation`, `official`, `commercial`) and its division with any
        // descriptive tags. A card renders these as its subject chips, so the
        // catalog serves only what describes the expert.
        let tags = vec![
            "expert".to_owned(),
            "conversation".to_owned(),
            "marketing".to_owned(),
            "official".to_owned(),
            "commercial".to_owned(),
            "电商增长".to_owned(),
        ];
        assert_eq!(
            display_tags(&tags, "expert-marketing"),
            vec!["电商增长".to_owned()]
        );
        // A catalog with no descriptive tags serves an empty list rather than
        // the record vocabulary.
        assert!(display_tags(&tags[..5], "expert-marketing").is_empty());
    }
}
