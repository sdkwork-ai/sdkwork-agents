//! App-facing expert catalog read surface
//! (`/app/v3/api/ai/expert_divisions`, `/experts`, `/expert_teams`).
//!
//! The catalog is read-only: it is authored in `sdkwork-specs` and seeded into
//! the managed-agent tables, so this module never writes. List endpoints follow
//! the standard offset pagination contract; detail endpoints add the localized
//! persona (expert) or the localized team shape.

use axum::extract::rejection::PathRejection;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Extension, Router};
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};

use crate::expert_catalog::{
    normalize_catalog_locale, persona_locale_candidates, ExpertCatalogStore, ExpertDivisionRecord,
    ExpertPersonaRecord, ExpertRecord, ExpertTeamMemberRow, ExpertTeamRow, EXPERT_CATALOG_LOCALES,
};
use crate::http::AgentHttpState;
use crate::response::{
    finish_api_json, ApiProblem, ApiResult, PageData, PageInfo, PageMode, ResourceData,
};

use super::context::{AgentRequestContext, RequestScope};

const DEFAULT_PAGE: i64 = 1;
const DEFAULT_PAGE_SIZE: i64 = 20;
const MAX_PAGE_SIZE: i64 = 200;
const MAX_PAGE: i64 = 10_000;

/// App router fragment owned by the expert catalog read surface.
pub(crate) fn routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/app/v3/api/ai/expert_divisions",
            get(app_list_expert_divisions),
        )
        .route("/app/v3/api/ai/experts", get(app_list_experts))
        .route("/app/v3/api/ai/experts/{slug}", get(app_get_expert))
        .route("/app/v3/api/ai/expert_teams", get(app_list_expert_teams))
        .route("/app/v3/api/ai/expert_personas", get(app_list_expert_personas))
        .route(
            "/app/v3/api/ai/expert_teams/{teamId}",
            get(app_get_expert_team),
        )
}

/// Standard offset list query plus the expert filters.
#[derive(Debug, Default, Deserialize)]
pub struct ExpertListQuery {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub page_size: Option<i64>,
    #[serde(default)]
    pub division: Option<String>,
    #[serde(default)]
    pub q: Option<String>,
}

/// List query for the divider/team endpoints.
#[derive(Debug, Default, Deserialize)]
pub struct ExpertCatalogListQuery {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub page_size: Option<i64>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

/// Detail query: the persona/team locale selector.
#[derive(Debug, Default, Deserialize)]
pub struct ExpertDetailQuery {
    #[serde(default)]
    pub locale: Option<String>,
}

/// One localized team member.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpertTeamMemberResponse {
    expert_slug: String,
    expert_agent_id: String,
    duty: String,
    instructions: String,
}

/// One localized team.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpertTeamResponse {
    team_id: String,
    name: String,
    description: String,
    tags: Vec<String>,
    goal: String,
    delivery_requirements: String,
    examples: Vec<String>,
    coordinator_mode: String,
    coordinator_template_id: String,
    locale: String,
    members: Vec<ExpertTeamMemberResponse>,
}

/// One expert detail with the persona served for the resolved locale.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpertDetailResponse {
    slug: String,
    agent_id: String,
    name: String,
    description: String,
    emoji: String,
    division_code: String,
    tags: Vec<String>,
    persona: String,
    persona_locale: String,
    persona_locales: Vec<&'static str>,
    content_sha256: String,
}

fn localized_team(team: &ExpertTeamRow, locale: &str) -> ExpertTeamResponse {
    let (name, description, tags, goal, delivery, examples) = team.localized(locale);
    ExpertTeamResponse {
        team_id: team.team_id.clone(),
        name: name.to_owned(),
        description: description.to_owned(),
        tags: tags.to_vec(),
        goal: goal.to_owned(),
        delivery_requirements: delivery.to_owned(),
        examples: examples.to_vec(),
        coordinator_mode: team.coordinator_mode.clone(),
        coordinator_template_id: team.coordinator_template_id.clone(),
        locale: locale.to_owned(),
        members: team
            .members
            .iter()
            .map(|member| {
                let (duty, instructions) = member.localized(locale);
                ExpertTeamMemberResponse {
                    expert_slug: member.expert_slug.clone(),
                    expert_agent_id: member.expert_agent_id.clone(),
                    duty: duty.to_owned(),
                    instructions: instructions.to_owned(),
                }
            })
            .collect(),
    }
}

fn normalize_keyword(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase())
}

fn expert_matches(expert: &ExpertRecord, division: Option<&str>, keyword: Option<&str>) -> bool {
    if let Some(division) = division {
        if expert.division_code != division {
            return false;
        }
    }
    let Some(keyword) = keyword else {
        return true;
    };
    expert.slug.to_lowercase().contains(keyword)
        || expert.name.to_lowercase().contains(keyword)
        || expert.description.to_lowercase().contains(keyword)
        || expert
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(keyword))
}

fn paginate<T>(items: Vec<T>, page: Option<i64>, page_size: Option<i64>) -> PageData<T> {
    let page = page.unwrap_or(DEFAULT_PAGE).clamp(1, MAX_PAGE);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let total_items = items.len();
    let offset = ((page - 1) * page_size) as usize;
    let page_items = items
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect::<Vec<_>>();
    let total_pages = if total_items == 0 {
        0
    } else {
        ((total_items as i64 + page_size - 1) / page_size) as i32
    };
    PageData {
        items: page_items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page as i32),
            page_size: Some(page_size as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(total_pages),
            next_cursor: None,
            has_more: Some(page as i64 * page_size < total_items as i64),
        },
    }
}

fn scope_ids(
    context: AgentRequestContext,
) -> ApiResult<(u64, u64)> {
    let scope = RequestScope::from_context(context);
    Ok((scope.tenant_id_u64()?, scope.organization_id_u64()?))
}

/// Resolve the requested persona locale with the catalog fallback order.
fn resolve_persona(
    store: &dyn ExpertCatalogStore,
    tenant_id: u64,
    organization_id: u64,
    expert_agent_id: &str,
    locale: &str,
) -> ApiResult<Option<ExpertPersonaRecord>> {
    for candidate in persona_locale_candidates(locale) {
        if let Some(persona) = store
            .persona(tenant_id, organization_id, expert_agent_id, candidate)
            .map_err(ApiProblem::from_kernel_error)?
        {
            return Ok(Some(persona));
        }
    }
    Ok(None)
}

async fn app_list_expert_divisions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    query: Result<Query<ExpertCatalogListQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<ExpertDivisionRecord>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let (tenant_id, organization_id) = scope_ids(context)?;
        let mut items = state
            .expert_catalog_store
            .divisions(tenant_id, organization_id)
            .map_err(ApiProblem::from_kernel_error)?;
        if let Some(keyword) = normalize_keyword(query.q.as_deref()) {
            items.retain(|division| {
                division.code.to_lowercase().contains(&keyword)
                    || division.name.to_lowercase().contains(&keyword)
            });
        }
        Ok(paginate(items, query.page, query.page_size))
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_experts(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    query: Result<Query<ExpertListQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<ExpertRecord>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let (tenant_id, organization_id) = scope_ids(context)?;
        let division = query
            .division
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let keyword = normalize_keyword(query.q.as_deref());
        let items = state
            .expert_catalog_store
            .experts(tenant_id, organization_id)
            .map_err(ApiProblem::from_kernel_error)?
            .into_iter()
            .filter(|expert| expert_matches(expert, division, keyword.as_deref()))
            .collect::<Vec<_>>();
        Ok(paginate(items, query.page, query.page_size))
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_expert(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    slug: Result<Path<String>, PathRejection>,
    query: Result<Query<ExpertDetailQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let result: ApiResult<ResourceData<ExpertDetailResponse>> = async {
        let Path(slug) = slug.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let (tenant_id, organization_id) = scope_ids(context)?;
        let expert = state
            .expert_catalog_store
            .expert(tenant_id, organization_id, slug.trim())
            .map_err(ApiProblem::from_kernel_error)?
            .ok_or_else(|| ApiProblem::not_found("expert not found"))?;
        let requested = normalize_catalog_locale(query.locale.as_deref());
        let persona = resolve_persona(
            state.expert_catalog_store.as_ref(),
            tenant_id,
            organization_id,
            &expert.agent_id,
            requested,
        )?
        .ok_or_else(|| ApiProblem::not_found("expert persona not found"))?;
        Ok(ResourceData {
            item: ExpertDetailResponse {
                slug: expert.slug,
                agent_id: expert.agent_id,
                name: expert.name,
                description: expert.description,
                emoji: expert.emoji,
                division_code: expert.division_code,
                tags: expert.tags,
                persona: persona.persona_text,
                persona_locale: persona.locale,
                persona_locales: EXPERT_CATALOG_LOCALES.to_vec(),
                content_sha256: persona.content_sha256,
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_expert_teams(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    query: Result<Query<ExpertCatalogListQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<ExpertTeamResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let (tenant_id, organization_id) = scope_ids(context)?;
        let locale = normalize_catalog_locale(query.locale.as_deref());
        let keyword = normalize_keyword(query.q.as_deref());
        let items = state
            .expert_catalog_store
            .teams(tenant_id, organization_id)
            .map_err(ApiProblem::from_kernel_error)?
            .iter()
            .map(|team| localized_team(team, locale))
            .filter(|team| {
                keyword.as_deref().is_none_or(|keyword| {
                    team.team_id.to_lowercase().contains(keyword)
                        || team.name.to_lowercase().contains(keyword)
                        || team.description.to_lowercase().contains(keyword)
                })
            })
            .collect::<Vec<_>>();
        Ok(paginate(items, query.page, query.page_size))
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_expert_team(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    team_id: Result<Path<String>, PathRejection>,
    query: Result<Query<ExpertDetailQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let result: ApiResult<ResourceData<ExpertTeamResponse>> = async {
        let Path(team_id) = team_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let (tenant_id, organization_id) = scope_ids(context)?;
        let locale = normalize_catalog_locale(query.locale.as_deref());
        let team = state
            .expert_catalog_store
            .team(tenant_id, organization_id, team_id.trim())
            .map_err(ApiProblem::from_kernel_error)?
            .ok_or_else(|| ApiProblem::not_found("expert team not found"))?;
        Ok(ResourceData {
            item: localized_team(&team, locale),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(_locale: &str) -> ExpertTeamRow {
        ExpertTeamRow {
            team_id: "team-product".to_owned(),
            name: "产品评审团".to_owned(),
            name_en: "Product Review Team".to_owned(),
            description: "中文".to_owned(),
            description_en: "English".to_owned(),
            tags: vec!["需求".to_owned()],
            tags_en: vec!["requirements".to_owned()],
            goal: "目标".to_owned(),
            goal_en: "goal".to_owned(),
            delivery_requirements: "交付".to_owned(),
            delivery_requirements_en: "delivery".to_owned(),
            examples: vec!["示例".to_owned()],
            examples_en: vec!["example".to_owned()],
            coordinator_mode: "template".to_owned(),
            coordinator_template_id: "product".to_owned(),
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
    fn paginates_with_offset_metadata() {
        let data = paginate(vec![1, 2, 3], Some(2), Some(2));
        assert_eq!(data.items, vec![3]);
        assert_eq!(data.page_info.total_items.as_deref(), Some("3"));
        assert_eq!(data.page_info.total_pages, Some(2));
        assert_eq!(data.page_info.has_more, Some(false));
    }

    #[test]
    fn localizes_teams_and_members() {
        let zh = localized_team(&team("zh-CN"), "zh-CN");
        assert_eq!(zh.name, "产品评审团");
        assert_eq!(zh.members[0].duty, "需求");
        assert_eq!(zh.locale, "zh-CN");
        let en = localized_team(&team("en-US"), "en-US");
        assert_eq!(en.name, "Product Review Team");
        assert_eq!(en.members[0].duty, "requirements");
        assert_eq!(en.locale, "en-US");
    }

    #[test]
    fn matches_experts_by_division_and_keyword() {
        let expert = ExpertRecord {
            slug: "product-manager".to_owned(),
            agent_id: "agent.expert.product-manager".to_owned(),
            name: "产品经理".to_owned(),
            description: "负责需求与路线图".to_owned(),
            emoji: "🧩".to_owned(),
            division_code: "product".to_owned(),
            tags: vec!["roadmap".to_owned()],
        };
        assert!(expert_matches(&expert, Some("product"), Some("经理")));
        assert!(expert_matches(&expert, None, Some("roadmap")));
        assert!(!expert_matches(&expert, Some("engineering"), None));
        assert!(!expert_matches(&expert, None, Some("missing")));
    }
}

/// One expert persona row for the locale bulk-sync endpoint.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpertPersonaResponse {
    expert_slug: String,
    agent_id: String,
    locale: String,
    persona: String,
    content_sha256: String,
}

async fn app_list_expert_personas(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    query: Result<Query<ExpertCatalogListQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<ExpertPersonaResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let (tenant_id, organization_id) = scope_ids(context)?;
        let locale = normalize_catalog_locale(query.locale.as_deref());
        let items = state
            .expert_catalog_store
            .personas(tenant_id, organization_id, locale)
            .map_err(ApiProblem::from_kernel_error)?
            .into_iter()
            .map(|persona: crate::expert_catalog::ExpertPersonaRecord| ExpertPersonaResponse {
                expert_slug: persona.expert_slug,
                agent_id: persona.expert_agent_id,
                locale: persona.locale,
                persona: persona.persona_text,
                content_sha256: persona.content_sha256,
            })
            .collect::<Vec<_>>();
        Ok(paginate(items, query.page, query.page_size))
    }
    .await;
    finish_api_json(&web_ctx, result)
}