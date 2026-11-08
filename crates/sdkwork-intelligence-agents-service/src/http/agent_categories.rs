//! Backend admin HTTP surface for the managed agent category catalog
//! (`/backend/v3/api/ai/agent_categories`).
//!
//! The catalog is the administrative source of truth for the `categoryId`
//! values carried by managed agent management profiles. It follows the
//! standard list/upsert pattern used by the sibling capability modules:
//! `GET` returns `data.items` + `data.pageInfo`, `POST` upserts by
//! `categoryCode` and returns `data.item`.

use axum::extract::rejection::PathRejection;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Extension, Json, Router};
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;

use crate::agent_category::{
    validate_category_placement, validate_upsert_command, AgentCategoryRecord, AgentCategoryStatus,
    UpsertAgentCategoryCommand,
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

/// Query parameters of the category list endpoint.
///
/// Field names stay snake_case because the wire contract is the standard
/// `page` / `page_size` / `q` list query (`PAGINATION_SPEC.md`), not the
/// camelCase JSON body convention.
#[derive(Debug, Default, Deserialize)]
pub struct AgentCategoryListQuery {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub page_size: Option<i64>,
    #[serde(default)]
    pub q: Option<String>,
}

/// Upsert body for the category identified by the path code.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentCategoryUpsertRequest {
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
    pub status: Option<AgentCategoryStatus>,
    #[serde(default, with = "sdkwork_utils_rust::serde_int64::option")]
    pub expected_version: Option<i64>,
}

/// Backend router fragment owned by the agent category catalog.
pub(crate) fn routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/backend/v3/api/ai/agent_categories",
            get(backend_list_agent_categories),
        )
        .route(
            "/backend/v3/api/ai/agent_categories/{categoryCode}",
            put(backend_upsert_agent_category),
        )
}

async fn backend_list_agent_categories(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    query: Result<Query<AgentCategoryListQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentCategoryRecord>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id = scope.organization_id_u64()?;
        let mut items = state
            .agent_category_store
            .list(tenant_id, organization_id)
            .map_err(ApiProblem::from_kernel_error)?;
        if let Some(keyword) = normalize_keyword(query.q.as_deref()) {
            items.retain(|record| category_matches(record, &keyword));
        }
        Ok(paginate(items, &query))
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_upsert_agent_category(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    path: Result<Path<AgentCategoryPathParams>, PathRejection>,
    body: Result<Json<AgentCategoryUpsertRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCategoryRecord>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let command = validate_upsert_command(&UpsertAgentCategoryCommand {
            category_code: path.category_code,
            name: body.name,
            description: body.description,
            parent_id: body.parent_id,
            sort_order: body.sort_order,
            icon_ref: body.icon_ref,
            status: body.status,
            expected_version: body.expected_version,
        })
        .map_err(ApiProblem::from_kernel_error)?;
        let scope = RequestScope::from_context(context.clone());
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id = scope.organization_id_u64()?;
        let operator_id = context.owner_user_id.parse::<u64>().unwrap_or(0);

        // The taxonomy is at most two levels deep. Validate the placement
        // against the current catalog before the write; the database composite
        // foreign key remains the final tenant-isolation guard.
        let categories = state
            .agent_category_store
            .list(tenant_id, organization_id)
            .map_err(ApiProblem::from_kernel_error)?;
        validate_category_placement(&categories, &command.category_code, command.parent_id)
            .map_err(ApiProblem::from_kernel_error)?;

        let saved = state
            .agent_category_store
            .upsert(tenant_id, organization_id, operator_id, &command)
            .map_err(ApiProblem::from_kernel_error)?;
        Ok(ResourceData { item: saved })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

/// Path parameters of the category upsert endpoint.
#[derive(Debug, Deserialize)]
pub struct AgentCategoryPathParams {
    #[serde(rename = "categoryCode")]
    pub category_code: String,
}

fn normalize_keyword(keyword: Option<&str>) -> Option<String> {
    keyword
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

fn category_matches(record: &AgentCategoryRecord, keyword: &str) -> bool {
    record.category_code.to_ascii_lowercase().contains(keyword)
        || record.name.to_ascii_lowercase().contains(keyword)
        || record
            .description
            .as_deref()
            .is_some_and(|description| description.to_ascii_lowercase().contains(keyword))
}

fn paginate(
    items: Vec<AgentCategoryRecord>,
    query: &AgentCategoryListQuery,
) -> PageData<AgentCategoryRecord> {
    let page = query.page.unwrap_or(DEFAULT_PAGE).clamp(1, MAX_PAGE);
    let page_size = query
        .page_size
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_category::AgentCategoryStatus;

    fn record(code: &str, name: &str, sort_order: i32) -> AgentCategoryRecord {
        AgentCategoryRecord {
            id: 1,
            uuid: format!("uuid-{code}"),
            tenant_id: 1,
            organization_id: 0,
            category_code: code.to_owned(),
            name: name.to_owned(),
            description: None,
            parent_id: None,
            sort_order,
            icon_ref: None,
            lifecycle_status: AgentCategoryStatus::Active,
            version: 0,
            created_at: "2026-01-01T00:00:00.000Z".to_owned(),
            updated_at: "2026-01-01T00:00:00.000Z".to_owned(),
        }
    }

    #[test]
    fn paginates_and_reports_total_items() {
        let items = vec![
            record("a1", "A", 0),
            record("b1", "B", 1),
            record("c1", "C", 2),
        ];
        let data = paginate(
            items,
            &AgentCategoryListQuery {
                page: Some(2),
                page_size: Some(2),
                q: None,
            },
        );
        assert_eq!(data.items.len(), 1);
        assert_eq!(data.page_info.total_items.as_deref(), Some("3"));
        assert_eq!(data.page_info.total_pages, Some(2));
        assert_eq!(data.page_info.has_more, Some(false));
    }

    #[test]
    fn keyword_matches_code_name_and_description() {
        let mut item = record("education", "Education", 0);
        assert!(category_matches(&item, "educa"));
        assert!(category_matches(&item, "education"));
        assert!(!category_matches(&item, "finance"));
        item.description = Some("K-12 teachers".to_owned());
        assert!(category_matches(&item, "teacher"));
    }
}
