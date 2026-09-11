//! Application-level media tool invocation pipeline.
//!
//! Wraps the aggregated [`MediaToolRegistry`] with tenant configuration
//! resolution (enabled state, save-to-drive default, default arguments) and
//! optional server-side persistence of generated content into the SDKWork
//! Drive. The auth token and the save-to-drive decision travel out-of-band —
//! they never enter model-visible tool arguments.

use sdkwork_agents_tool_contract::{MediaResource, MediaToolCall, MediaToolError, MediaToolResult};
use sdkwork_drive_uploader_service::service::UploaderActor;

use crate::domain::{AgentToolAssetRecord, AgentToolConfigurationRecord};
use crate::drive_asset_saver::{
    fetch_resource_bytes, DriveAssetRef, DriveAssetSaver, DriveSaveContext,
};
use crate::media_tool_registry::MediaToolRegistry;
use crate::ports::AgentRepository;

/// Result of one tool invocation, optionally extended with the Drive asset
/// reference when `saveToDrive` was requested.
#[derive(Debug, Clone)]
pub struct ToolInvocationOutcome {
    pub result: MediaToolResult,
    /// Drive asset reference when the generated content was persisted.
    pub drive_asset: Option<DriveAssetRef>,
    /// The configuration that governed this invocation (may be defaults).
    pub configuration: AgentToolConfigurationRecord,
}

/// One media tool invocation request from an application surface.
#[derive(Debug, Clone)]
pub struct MediaToolInvocationRequest {
    pub tool_call_id: String,
    pub tool_id: String,
    pub arguments: serde_json::Value,
    pub tenant_id: u64,
    pub organization_id: u64,
    /// Caller auth token for cloudrouter account-pool routing.
    pub auth_token: Option<String>,
    /// Explicit save-to-drive request; falls back to the tenant configuration
    /// default when `None`.
    pub save_to_drive: Option<bool>,
    /// Operator identity recorded on drive uploads (user or system id).
    pub actor: UploaderActor,
    /// App resource id recorded on drive uploads (e.g. generation id).
    pub app_resource_id: String,
}

impl MediaToolInvocationRequest {
    /// Builds the tool call shared with the registry.
    pub fn as_tool_call(&self) -> MediaToolCall {
        MediaToolCall {
            tool_call_id: self.tool_call_id.clone(),
            tool_id: self.tool_id.clone(),
            arguments: self.arguments.clone(),
            session_id: Some(self.app_resource_id.clone()),
        }
    }
}

/// Runs the configured media tool invocation pipeline.
///
/// The pipeline is synchronous from the caller's perspective: the drive
/// fetch/upload path is driven on the service blocking runtime via
/// [`crate::tool_invocation::execute_invocation`] helpers.
pub struct MediaToolInvocationService {
    registry: MediaToolRegistry,
    drive_saver: Option<DriveAssetSaver>,
    repository: Box<dyn AgentRepository>,
}

impl std::fmt::Debug for MediaToolInvocationService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MediaToolInvocationService")
            .finish_non_exhaustive()
    }
}

impl MediaToolInvocationService {
    pub fn new(
        registry: MediaToolRegistry,
        drive_saver: Option<DriveAssetSaver>,
        repository: Box<dyn AgentRepository>,
    ) -> Self {
        Self {
            registry,
            drive_saver,
            repository,
        }
    }

    /// The aggregated tool registry backing this service.
    pub fn registry(&self) -> &MediaToolRegistry {
        &self.registry
    }

    /// Allocates a stable tool call id (repository-backed id generator).
    pub fn next_tool_call_id(&self) -> String {
        let id = self
            .repository
            .next_id()
            .unwrap_or_else(|_| chrono::Utc::now().timestamp_millis() as u64);
        format!("tool-call-{id}")
    }

    /// Lists generated assets for the unified asset entry surface.
    pub fn list_tool_assets(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        limit: u64,
    ) -> sdkwork_agent_kernel::KernelResult<Vec<crate::domain::AgentToolAssetRecord>> {
        self.repository
            .list_tool_assets(tenant_id, organization_id, user_id, limit)
    }

    /// Loads the tenant configuration for one tool, applying registry-level
    /// defaults when the tenant has never configured it.
    pub fn tool_configuration(
        &self,
        tenant_id: u64,
        organization_id: u64,
        tool_id: &str,
    ) -> sdkwork_agent_kernel::KernelResult<AgentToolConfigurationRecord> {
        match self
            .repository
            .get_tool_configuration(tenant_id, organization_id, tool_id)?
        {
            Some(record) => Ok(record),
            None => Ok(default_tool_configuration(
                tenant_id,
                organization_id,
                tool_id,
            )),
        }
    }

    /// Lists tenant tool configurations for admin surfaces.
    pub fn list_tool_configurations(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> sdkwork_agent_kernel::KernelResult<Vec<AgentToolConfigurationRecord>> {
        self.repository
            .list_tool_configurations(tenant_id, organization_id)
    }

    /// Persists an admin-updated tool configuration.
    pub fn save_tool_configuration(
        &self,
        record: AgentToolConfigurationRecord,
        expected_version: Option<u64>,
    ) -> sdkwork_agent_kernel::KernelResult<AgentToolConfigurationRecord> {
        self.repository
            .upsert_tool_configuration(record, expected_version)
    }

    /// Executes one tool invocation with configuration resolution and
    /// optional drive persistence.
    pub fn invoke(
        &self,
        request: &MediaToolInvocationRequest,
    ) -> Result<ToolInvocationOutcome, MediaToolError> {
        let configuration = self
            .tool_configuration(request.tenant_id, request.organization_id, &request.tool_id)
            .map_err(|error| {
                MediaToolError::ProviderError(format!("tool configuration lookup failed: {error}"))
            })?;

        if !configuration.enabled {
            return Err(MediaToolError::CapabilityMissing(format!(
                "tool `{}` is disabled for this tenant by the administrator",
                request.tool_id
            )));
        }

        let call = request.as_tool_call();
        let result = self.registry.invoke(&call, request.auth_token.as_deref())?;

        let save_to_drive = request
            .save_to_drive
            .unwrap_or(configuration.save_to_drive_default);
        if !save_to_drive || result.status != "succeeded" {
            return Ok(ToolInvocationOutcome {
                result,
                drive_asset: None,
                configuration,
            });
        }

        let drive_asset = self.persist_generated_media(request, &result, &configuration)?;
        Ok(ToolInvocationOutcome {
            result,
            drive_asset,
            configuration,
        })
    }

    /// Persists generated media from a succeeded result into the drive and
    /// registers the asset record for the unified asset entry surface.
    fn persist_generated_media(
        &self,
        request: &MediaToolInvocationRequest,
        result: &MediaToolResult,
        _configuration: &AgentToolConfigurationRecord,
    ) -> Result<Option<DriveAssetRef>, MediaToolError> {
        let saver = self.drive_saver.as_ref().ok_or_else(|| {
            MediaToolError::ProviderError(
                "drive persistence requested but the drive saver is not configured".to_string(),
            )
        })?;

        let mut persisted: Option<DriveAssetRef> = None;

        for resource in extract_resources(result)? {
            let source_url = resource.url.clone();
            let body = blocking_fetch(&source_url)?;
            let context = DriveSaveContext {
                tenant_id: request.tenant_id.to_string(),
                organization_id: Some(request.organization_id.to_string()),
                actor: request.actor.clone(),
                app_resource_id: request.app_resource_id.clone(),
                tool_call_id: request.tool_call_id.clone(),
                tool_id: request.tool_id.clone(),
            };
            let asset_ref = blocking_save(saver, &context, &resource, body)?;
            let user_id = operator_user_id(&request.actor);
            let now = now_rfc3339();
            let record = AgentToolAssetRecord {
                id: 0,
                tenant_id: request.tenant_id,
                organization_id: request.organization_id,
                user_id,
                tool_id: request.tool_id.clone(),
                tool_call_id: request.tool_call_id.clone(),
                media_kind: resource.kind.clone(),
                drive_space_id: asset_ref.space_id.clone(),
                drive_node_id: asset_ref.node_id.clone(),
                drive_uri: asset_ref.drive_uri.clone(),
                source_url: Some(source_url),
                created_by: user_id,
                created_at: now.clone(),
                updated_at: now,
                deleted_at: None,
            };
            // Asset registration is best-effort: the drive node is already
            // the system of record, the agents row powers the unified asset
            // entry surface.
            let _ = self.repository.insert_tool_asset(record);
            persisted = Some(asset_ref);
        }

        Ok(persisted)
    }
}

/// Extracts normalized media resources from a succeeded tool result.
///
/// Supports both single-resource outputs (`{kind,source,url,...}`) and
/// multi-item outputs (`{items: [...]}`, `{images: [...]}`, `{tracks: [...]}`).
fn extract_resources(result: &MediaToolResult) -> Result<Vec<MediaResource>, MediaToolError> {
    let mut resources = Vec::new();
    collect_resource_values(&result.output, &mut resources)?;
    if resources.is_empty() {
        return Err(MediaToolError::ProviderError(format!(
            "tool result carries no generated media URL to persist (status {})",
            result.status
        )));
    }
    Ok(resources)
}

fn collect_resource_values(
    value: &serde_json::Value,
    resources: &mut Vec<MediaResource>,
) -> Result<(), MediaToolError> {
    match value {
        serde_json::Value::Object(map) => {
            let kind = map.get("kind").and_then(serde_json::Value::as_str);
            let url = map
                .get("url")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty());
            if kind.is_some() && url.is_some() {
                resources.push(MediaResource {
                    kind: kind.unwrap_or("other").to_string(),
                    source: map
                        .get("source")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("provider_asset")
                        .to_string(),
                    url: url.unwrap_or_default().to_string(),
                    task_id: map
                        .get("taskId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    mime_type: None,
                    file_name: None,
                });
                return Ok(());
            }
            for (key, child) in map {
                if matches!(key.as_str(), "items" | "images" | "tracks" | "videos") {
                    collect_resource_values(child, resources)?;
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_resource_values(item, resources)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Runs the fetch on the service blocking runtime (the invocation pipeline is
/// synchronous like kernel tool providers).
fn blocking_fetch(url: &str) -> Result<Vec<u8>, MediaToolError> {
    blocking_runtime()
        .block_on(fetch_resource_bytes(url))
        .map_err(|error| {
            MediaToolError::ProviderError(format!("drive fetch failed for {url}: {error}"))
        })
}

/// Runs the drive upload on the service blocking runtime.
fn blocking_save(
    saver: &DriveAssetSaver,
    context: &DriveSaveContext,
    resource: &MediaResource,
    body: Vec<u8>,
) -> Result<DriveAssetRef, MediaToolError> {
    blocking_runtime()
        .block_on(saver.save_generated_media(context, resource, body))
        .map_err(|error| {
            MediaToolError::ProviderError(format!(
                "drive persistence failed for {}: {error}",
                resource.url
            ))
        })
}

/// Default tenant configuration applied when the tenant has never configured
/// a tool: enabled, no drive persistence, empty defaults.
pub fn default_tool_configuration(
    tenant_id: u64,
    organization_id: u64,
    tool_id: &str,
) -> AgentToolConfigurationRecord {
    AgentToolConfigurationRecord {
        id: 0,
        tenant_id,
        organization_id,
        tool_id: tool_id.to_string(),
        enabled: true,
        save_to_drive_default: false,
        default_arguments_json: "{}".to_string(),
        version: 0,
        created_by: 0,
        updated_by: 0,
        created_at: now_rfc3339(),
        updated_at: now_rfc3339(),
        deleted_at: None,
    }
}

/// Resolves the user id from an uploader actor for audit columns.
fn operator_user_id(actor: &UploaderActor) -> u64 {
    match actor {
        UploaderActor::Anonymous { .. } | UploaderActor::System { .. } => 0,
        UploaderActor::User { user_id } => user_id.parse().unwrap_or(0),
    }
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn blocking_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tool invocation tokio runtime")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource_value(url: &str) -> serde_json::Value {
        serde_json::json!({
            "kind": "image",
            "source": "provider_asset",
            "url": url,
        })
    }

    #[test]
    fn extracts_single_and_nested_resources() {
        let single = MediaToolResult::succeeded("call.1", resource_value("https://a.png"));
        let resources = extract_resources(&single).expect("single resource");
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].url, "https://a.png");
        assert_eq!(resources[0].kind, "image");

        let multi = MediaToolResult::succeeded(
            "call.2",
            serde_json::json!({
                "taskId": "task.1",
                "status": "completed",
                "items": [resource_value("https://a.png"), resource_value("https://b.png")]
            }),
        );
        let resources = extract_resources(&multi).expect("nested resources");
        assert_eq!(resources.len(), 2);
    }

    #[test]
    fn extract_rejects_results_without_media() {
        let no_media = MediaToolResult::succeeded("call.3", serde_json::json!({ "text": "hi" }));
        assert!(extract_resources(&no_media).is_err());

        let no_url = MediaToolResult::succeeded(
            "call.4",
            serde_json::json!({ "kind": "image", "source": "provider_asset" }),
        );
        assert!(extract_resources(&no_url).is_err());
    }

    #[test]
    fn default_configuration_is_enabled_without_drive() {
        let configuration = default_tool_configuration(1, 0, "audio.speech.create");
        assert!(configuration.enabled);
        assert!(!configuration.save_to_drive_default);
        assert_eq!(configuration.default_arguments_json, "{}");
        assert_eq!(configuration.tool_id, "audio.speech.create");
    }

    #[test]
    fn extract_skips_task_outputs_without_urls() {
        let task_pending = MediaToolResult::succeeded(
            "call.5",
            serde_json::json!({ "taskId": "task.5", "status": "processing", "items": [] }),
        );
        assert!(extract_resources(&task_pending).is_err());
    }
}
