//! Media tool HTTP handlers: tool directory, invocation (with optional drive
//! persistence), generated-asset listing, and admin configuration.

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use axum::Json;
use sdkwork_agents_tool_contract::{MediaToolDefinition, MediaToolError, ToolAvailability};
use sdkwork_drive_uploader_service::service::UploaderActor;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};

use crate::domain::{AgentToolAssetRecord, AgentToolConfigurationRecord};
use crate::http::AgentHttpState;
use crate::response::{finish_api_json, finish_created_api_json, ApiProblem, ApiResult};
use crate::tool_invocation::{MediaToolInvocationRequest, ToolInvocationOutcome};

use super::context::{AgentRequestContext, RequestScope};

/// Tool directory entry: definition plus the tenant's effective configuration.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolDirectoryEntry {
    pub tool_id: String,
    pub category: String,
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub side_effect_level: String,
    pub policy_categories: Vec<String>,
    pub timeout_ms: u64,
    pub availability: String,
    pub enabled: bool,
    pub save_to_drive_default: bool,
    pub configured: bool,
}

impl MediaToolDirectoryEntry {
    fn from_definition(
        definition: &MediaToolDefinition,
        configuration: &AgentToolConfigurationRecord,
    ) -> Self {
        Self {
            tool_id: definition.tool_id.clone(),
            category: definition.category.to_string(),
            name: definition.name.clone(),
            display_name: definition.display_name.clone(),
            version: definition.version.clone(),
            description: definition.description.clone(),
            input_schema: definition.input_schema.clone(),
            output_schema: definition.output_schema.clone(),
            side_effect_level: definition.side_effect_level.clone(),
            policy_categories: definition.policy_categories.clone(),
            timeout_ms: definition.timeout_ms,
            availability: match definition.availability {
                ToolAvailability::Available => "available".to_string(),
                ToolAvailability::PendingCapability { ref reason } => {
                    format!("pending_capability:{reason}")
                }
            },
            enabled: configuration.enabled,
            save_to_drive_default: configuration.save_to_drive_default,
            configured: configuration.version > 0,
        }
    }
}

/// Tool invocation request body (saveToDrive travels out-of-band).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolInvokeBody {
    pub arguments: serde_json::Value,
    #[serde(default)]
    pub save_to_drive: Option<bool>,
}

/// Tool invocation response: normalized result plus drive asset reference.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolInvokeResponse {
    pub tool_call_id: String,
    pub status: String,
    pub output: serde_json::Value,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drive_asset: Option<DriveAssetView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveAssetView {
    pub space_id: String,
    pub node_id: String,
    pub drive_uri: String,
}

/// Tool configuration update body (backend admin).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolConfigurationBody {
    pub enabled: bool,
    #[serde(default)]
    pub save_to_drive_default: bool,
    #[serde(default = "empty_object")]
    pub default_arguments: serde_json::Value,
    #[serde(default)]
    pub expected_version: Option<u64>,
}

fn empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Generated-asset view for the unified asset entry surface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAssetView {
    pub tool_id: String,
    pub tool_call_id: String,
    pub media_kind: String,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub drive_uri: String,
    pub source_url: Option<String>,
    pub created_at: String,
}

fn tool_asset_view(record: &AgentToolAssetRecord) -> ToolAssetView {
    ToolAssetView {
        tool_id: record.tool_id.clone(),
        tool_call_id: record.tool_call_id.clone(),
        media_kind: record.media_kind.clone(),
        drive_space_id: record.drive_space_id.clone(),
        drive_node_id: record.drive_node_id.clone(),
        drive_uri: record.drive_uri.clone(),
        source_url: record.source_url.clone(),
        created_at: record.created_at.clone(),
    }
}

fn resolve_invocation(
    state: &AgentHttpState,
) -> Result<&crate::tool_invocation::MediaToolInvocationService, ApiProblem> {
    state.media_tool_invocation.as_deref().ok_or_else(|| {
        ApiProblem::dependency_unavailable("media tool invocation is not configured")
    })
}

fn scope_numbers(
    scope: &RequestScope,
    context: &AgentRequestContext,
) -> Result<(u64, u64, u64), ApiProblem> {
    let tenant_id = scope.tenant_id_u64()?;
    let organization_id = context
        .organization_id
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let user_id = scope.owner_scope()?.unwrap_or(0);
    Ok((tenant_id, organization_id, user_id))
}

/// GET /app/v3/api/ai/tools — tool directory with effective tenant config.
pub async fn app_list_media_tools(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
) -> Response {
    let result: ApiResult<Vec<MediaToolDirectoryEntry>> = async {
        let invocation = resolve_invocation(&state)?;
        let (tenant_id, organization_id, _) =
            scope_numbers(&RequestScope::from_context(context.clone()), &context)?;
        let registry = invocation.registry();

        let mut entries = Vec::new();
        for definition in registry.list_tools() {
            let configuration = invocation
                .tool_configuration(tenant_id, organization_id, &definition.tool_id)
                .map_err(ApiProblem::from_kernel_error)?;
            entries.push(MediaToolDirectoryEntry::from_definition(
                &definition,
                &configuration,
            ));
        }
        Ok(entries)
    }
    .await;
    finish_api_json(&web_ctx, result)
}

/// Wall-clock bound for one media tool invocation (cloudrouter call plus
/// optional drive fetch/upload). The invocation pipeline is synchronous; a
/// hung cloudrouter or storage provider must never pin the async executor
/// or leave the HTTP request suspended.
const MEDIA_TOOL_INVOKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// POST /app/v3/api/ai/tools/{toolId}/invoke — execute one media tool.
pub async fn app_invoke_media_tool(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    path: Result<Path<ToolIdPathParams>, axum::extract::rejection::PathRejection>,
    headers: axum::http::HeaderMap,
    body: Result<Json<MediaToolInvokeBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let result: ApiResult<MediaToolInvokeResponse> = async {
        let invocation = state
            .media_tool_invocation
            .as_ref()
            .cloned()
            .ok_or_else(|| {
                ApiProblem::dependency_unavailable("media tool invocation is not configured")
            })?;
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let body = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context.clone());
        let (tenant_id, organization_id, user_id) = scope_numbers(&scope, &context)?;

        let auth_token = crate::http::extract_bearer_auth_token(&headers);
        let request = MediaToolInvocationRequest {
            tool_call_id: invocation.next_tool_call_id(),
            tool_id: path.tool_id,
            arguments: body.arguments.clone(),
            tenant_id,
            organization_id,
            auth_token,
            save_to_drive: body.save_to_drive,
            actor: UploaderActor::User {
                user_id: user_id.to_string(),
            },
            app_resource_id: format!("tool-call-direct-{user_id}"),
            trace_id: context.trace_id.clone(),
        };

        // Run the synchronous pipeline on the blocking pool with a hard
        // timeout so a hung cloudrouter/storage call cannot block the async
        // executor; the worker keeps running after the bound and its result
        // is dropped.
        let outcome = tokio::time::timeout(
            MEDIA_TOOL_INVOKE_TIMEOUT,
            tokio::task::spawn_blocking(move || invocation.invoke(&request)),
        )
        .await
        .map_err(|_| ApiProblem::gateway_timeout("media tool invocation timed out"))?
        .map_err(|error| ApiProblem::internal(format!("media tool worker failed: {error}")))?
        .map_err(map_tool_error)?;
        Ok(to_invoke_response(outcome))
    }
    .await;
    finish_api_json(&web_ctx, result)
}

fn to_invoke_response(outcome: ToolInvocationOutcome) -> MediaToolInvokeResponse {
    MediaToolInvokeResponse {
        tool_call_id: outcome.result.tool_call_id.clone(),
        status: outcome.result.status.clone(),
        output: outcome.result.output.clone(),
        error: outcome.result.error.clone(),
        drive_asset: outcome.drive_asset.map(|asset| DriveAssetView {
            space_id: asset.space_id,
            node_id: asset.node_id,
            drive_uri: asset.drive_uri,
        }),
    }
}

fn map_tool_error(error: MediaToolError) -> ApiProblem {
    match error {
        MediaToolError::InvalidInput(message) => ApiProblem::bad_request(message),
        MediaToolError::CapabilityMissing(message) | MediaToolError::AuthRequired(message) => {
            ApiProblem::permission(message)
        }
        MediaToolError::ProviderUnavailable(message) | MediaToolError::ProviderError(message) => {
            ApiProblem::dependency_unavailable(message)
        }
        MediaToolError::Timeout(message) => ApiProblem::gateway_timeout(message),
        MediaToolError::RateLimited(message) => ApiProblem::too_many_requests(message, None),
    }
}

/// GET /app/v3/api/ai/assets — generated assets for the caller.
pub async fn app_list_tool_assets(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
) -> Response {
    let result: ApiResult<Vec<ToolAssetView>> = async {
        let invocation = resolve_invocation(&state)?;
        let (tenant_id, organization_id, user_id) =
            scope_numbers(&RequestScope::from_context(context.clone()), &context)?;
        let records = invocation
            .list_tool_assets(tenant_id, organization_id, user_id, 200)
            .map_err(ApiProblem::from_kernel_error)?;
        Ok(records.iter().map(tool_asset_view).collect())
    }
    .await;
    finish_api_json(&web_ctx, result)
}

/// GET /backend/v3/api/ai/tools — full tool directory with tenant configs.
pub async fn backend_list_media_tools(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
) -> Response {
    let result: ApiResult<Vec<MediaToolDirectoryEntry>> = async {
        let invocation = resolve_invocation(&state)?;
        let (tenant_id, organization_id, _) =
            scope_numbers(&RequestScope::from_context(context.clone()), &context)?;
        let registry = invocation.registry();

        let mut entries = Vec::new();
        for definition in registry.list_tools() {
            let configuration = invocation
                .tool_configuration(tenant_id, organization_id, &definition.tool_id)
                .map_err(ApiProblem::from_kernel_error)?;
            entries.push(MediaToolDirectoryEntry::from_definition(
                &definition,
                &configuration,
            ));
        }
        Ok(entries)
    }
    .await;
    finish_api_json(&web_ctx, result)
}

/// PUT /backend/v3/api/ai/tools/{toolId}/configuration — admin update.
pub async fn backend_update_media_tool_configuration(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<WebRequestContext>,
    path: Result<Path<ToolIdPathParams>, axum::extract::rejection::PathRejection>,
    body: Result<Json<MediaToolConfigurationBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let result: ApiResult<MediaToolDirectoryEntry> = async {
        let invocation = resolve_invocation(&state)?;
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let body = body.map_err(ApiProblem::from_json_rejection)?;
        let (tenant_id, organization_id, _) =
            scope_numbers(&RequestScope::from_context(context.clone()), &context)?;

        // The registry must know the tool; unknown tool ids are rejected.
        let definition = invocation
            .registry()
            .describe_tool(&path.tool_id)
            .ok_or_else(|| {
                ApiProblem::not_found(format!("unknown media tool `{}`", path.tool_id))
            })?;

        let current = invocation
            .tool_configuration(tenant_id, organization_id, &path.tool_id)
            .map_err(ApiProblem::from_kernel_error)?;

        let mut updated = current;
        updated.enabled = body.enabled;
        updated.save_to_drive_default = body.save_to_drive_default;
        updated.default_arguments_json = body.default_arguments.to_string();
        updated.updated_by = scope_user_id(&context);
        updated.updated_at =
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        if updated.version == 0 {
            updated.created_by = updated.updated_by;
            updated.created_at = updated.updated_at.clone();
        }

        let persisted = invocation
            .save_tool_configuration(updated, body.expected_version)
            .map_err(ApiProblem::from_kernel_error)?;
        Ok(MediaToolDirectoryEntry::from_definition(
            &definition,
            &persisted,
        ))
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

fn scope_user_id(context: &AgentRequestContext) -> u64 {
    context.owner_user_id.parse().unwrap_or(0)
}

/// Path params shared by tool routes.
#[derive(Debug, Deserialize)]
pub struct ToolIdPathParams {
    pub tool_id: String,
}
