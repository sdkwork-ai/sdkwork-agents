//! Turn-scoped tool calling: descriptors, dispatcher, and executors.
//!
//! The turn executor advertises an effective tool set to the model through
//! OpenAI-compatible function calling and routes every model-selected tool
//! call through this dispatcher. Tool ids follow the kernel MCP naming
//! contract (`mcp__<server>__<tool>`) so built-in and user-registered MCP
//! tools share one collision-free namespace:
//!
//! - `mcp__generations__*` — the built-in generations MCP (image, video,
//!   speech synthesis, music), enabled by default;
//! - `sound-effect.generate` and the rest of the media tool family — the
//!   application-owned media tools (synchronous, cloudrouter-backed);
//! - `mcp__<user-server>__<tool>` — user-registered external MCP servers
//!   invoked over JSON-RPC 2.0 HTTP.
//!
//! The auth token is resolved at the dispatcher boundary from the turn input
//! and never enters model-visible arguments.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::media_tool_registry::MediaToolRegistry;
use sdkwork_agents_tool_contract::MediaToolCall;

/// Namespace prefix of the built-in generations MCP tools.
pub const GENERATIONS_MCP_TOOL_PREFIX: &str = "mcp__generations__";
/// Namespace prefix shared by every external MCP tool.
pub const EXTERNAL_MCP_TOOL_PREFIX: &str = "mcp__";
/// Default per-tool execution budget for synchronous media tools.
pub const DEFAULT_TOOL_TIMEOUT_MS: u64 = 60_000;
/// Execution budget for asynchronous generations tools (vendor tasks may run
/// minutes; the port enforces its own polling/refresh behavior).
pub const GENERATIONS_TOOL_TIMEOUT_MS: u64 = 180_000;
/// Hard cap for tool-result content fed back to the model, preventing
/// unbounded result payloads from exhausting the completion context.
pub const MAX_TOOL_RESULT_CONTENT_CHARS: usize = 8_000;

/// Origin of a tool descriptor: which executor owns the tool id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnToolOrigin {
    /// Built-in sdkwork-generations MCP (`mcp__generations__*`).
    BuiltinGenerations,
    /// Application-owned media tool family.
    BuiltinMedia,
    /// User-registered external MCP server.
    ExternalMcp,
}

/// One tool the model may call during a turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnToolDescriptor {
    /// Stable dispatch id, e.g. `mcp__generations__image.create`.
    pub tool_id: String,
    /// Model-visible function name.
    pub name: String,
    /// Safe summary describing behaviour and side effects to the model.
    pub description: String,
    /// JSON Schema draft 2020-12 input schema (serialized JSON).
    pub input_schema: serde_json::Value,
    /// When true the loop must not execute the call without user approval.
    pub requires_approval: bool,
    /// Primary policy category driving authorization/audit.
    pub policy_category: Option<String>,
    /// Execution budget in milliseconds.
    pub timeout_ms: u64,
    /// Owning executor origin.
    pub origin: TurnToolOrigin,
}

/// One model-selected tool call inside the turn loop.
#[derive(Debug, Clone, PartialEq)]
pub struct TurnToolCall {
    pub tool_call_id: String,
    pub tool_id: String,
    pub arguments: serde_json::Value,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    /// Owning tenant id from the turn session (used by tenant-scoped tools
    /// such as the federated generations API).
    pub tenant_id: Option<u64>,
}

/// Result of dispatching one tool call.
#[derive(Debug, Clone, PartialEq)]
pub enum TurnToolExecution {
    /// The tool produced result content (fed back to the model verbatim).
    Completed { content: String },
    /// The tool is gated on user approval; the loop must suspend instead of
    /// executing it.
    ApprovalRequired { detail: String },
    /// The tool failed; the message is fed back so the model can self-heal.
    Failed { code: String, message: String },
}

/// Whether a tool event records the call or its result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnToolEventKind {
    ToolUse,
    ToolResult,
}

/// Durable record of one tool activity inside the loop (persisted as
/// ToolCall/ToolResult session items and streamed to clients).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnToolEvent {
    pub tool_call_id: String,
    pub tool_id: String,
    pub kind: TurnToolEventKind,
    /// `succeeded` | `failed` | `approval_required`.
    pub status: String,
    /// Serialized tool arguments (length-capped).
    pub arguments_json: Option<String>,
    /// Result or error content (length-capped).
    pub content: Option<String>,
}

/// Per-turn execution context handed to every tool executor.
///
/// Carries the caller's transient credentials (never persisted) and the MCP
/// server connections resolved from the agent's composition-slot policies for
/// this turn.
pub struct TurnToolExecutionContext<'a> {
    pub auth_token: Option<&'a str>,
    pub access_token: Option<&'a str>,
    pub mcp_connections: &'a [McpServerConnection],
}

/// Pluggable executor owning one tool-id namespace.
pub trait TurnToolExecutor: Send + Sync {
    /// True when this executor owns (and can dispatch) the tool id.
    fn owns(&self, tool_id: &str) -> bool;

    /// Model-visible descriptors this executor contributes to the default
    /// toolkit (empty for executors that only serve explicit configuration).
    fn descriptors(&self) -> Vec<TurnToolDescriptor> {
        Vec::new()
    }

    /// Executes one tool call within the given turn context.
    fn execute(&self, call: &TurnToolCall, context: &TurnToolExecutionContext<'_>)
        -> TurnToolExecution;
}

/// Aggregated turn-scoped tool dispatcher: routes model-selected calls by
/// tool-id namespace to the owning executor and fails closed for unknown ids.
#[derive(Default)]
pub struct TurnToolDispatcher {
    executors: Vec<Box<dyn TurnToolExecutor>>,
}

impl TurnToolDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one executor (builder style). Later registrations take
    /// precedence for overlapping namespaces.
    pub fn with_executor(mut self, executor: Box<dyn TurnToolExecutor>) -> Self {
        self.executors.push(executor);
        self
    }

    /// True when some registered executor owns the tool id.
    pub fn executes(&self, tool_id: &str) -> bool {
        self.executors.iter().any(|executor| executor.owns(tool_id))
    }

    /// The default toolkit: descriptors contributed by every registered
    /// executor, deduplicated by tool id (first registration wins).
    pub fn default_descriptors(&self) -> Vec<TurnToolDescriptor> {
        let mut descriptors: Vec<TurnToolDescriptor> = Vec::new();
        for executor in &self.executors {
            for descriptor in executor.descriptors() {
                if !descriptors.iter().any(|known| known.tool_id == descriptor.tool_id) {
                    descriptors.push(descriptor);
                }
            }
        }
        descriptors
    }

    /// Dispatches one call within the turn context; returns an error string
    /// for unknown tool ids (fail closed — never a bare panic).
    pub fn execute(
        &self,
        call: &TurnToolCall,
        context: &TurnToolExecutionContext<'_>,
    ) -> Result<TurnToolExecution, String> {
        for executor in self.executors.iter().rev() {
            if executor.owns(&call.tool_id) {
                return Ok(executor.execute(call, context));
            }
        }
        Err(format!(
            "no tool executor registered for tool id `{}`",
            call.tool_id
        ))
    }
}

/// Execution backend of the built-in generations tools.
pub enum GenerationsToolRuntime {
    /// Embedded generations port (in-process tests/local state).
    Embedded(Arc<dyn sdkwork_generations_mcp_service::GenerationsMcpPort>),
    /// Federated generations app API on the cloudrouter gateway (production).
    Http(Arc<crate::generations_tool_port::HttpGenerationsPort>),
}

/// Executor for the built-in generations MCP (`mcp__generations__*`).
pub struct GenerationsToolExecutor {
    runtime: GenerationsToolRuntime,
    provider: sdkwork_generations_mcp_service::GenerationsMcpProvider,
}

impl GenerationsToolExecutor {
    /// Build the executor over an embedded generations port (tests/local).
    pub fn new(port: Arc<dyn sdkwork_generations_mcp_service::GenerationsMcpPort>) -> Self {
        Self {
            provider: sdkwork_generations_mcp_service::GenerationsMcpProvider::new(
                Arc::clone(&port),
            ),
            runtime: GenerationsToolRuntime::Embedded(port),
        }
    }

    /// Build the executor over the federated generations app API (production).
    pub fn new_http(port: Arc<crate::generations_tool_port::HttpGenerationsPort>) -> Self {
        // Descriptors are static; an in-memory port backs the provider for
        // descriptor listing only — invocation goes through the HTTP runtime.
        let listing_port = Arc::new(sdkwork_generations_mcp_service::InMemoryGenerationsMcpPort::new());
        Self {
            provider: sdkwork_generations_mcp_service::GenerationsMcpProvider::new(listing_port),
            runtime: GenerationsToolRuntime::Http(port),
        }
    }

    /// Tool descriptors exposed to the model, projected onto the turn shape.
    pub fn descriptors(&self) -> Vec<TurnToolDescriptor> {
        self.provider
            .tool_descriptors()
            .into_iter()
            .map(|descriptor| TurnToolDescriptor {
                tool_id: descriptor.tool_id.clone(),
                // The model-visible function name equals the dispatch id so
                // every echoed call round-trips without a lookup table.
                name: descriptor.tool_id.clone(),
                description: descriptor.description.unwrap_or_default(),
                input_schema: descriptor
                    .input_schema
                    .and_then(|schema| schema.document)
                    .and_then(|document| serde_json::from_str(&document).ok())
                    .unwrap_or_else(|| serde_json::json!({"type": "object"})),
                requires_approval: false,
                policy_category: descriptor.policy_categories.first().cloned(),
                timeout_ms: descriptor
                    .timeout_ms
                    .unwrap_or(GENERATIONS_TOOL_TIMEOUT_MS),
                origin: TurnToolOrigin::BuiltinGenerations,
            })
            .collect()
    }

    fn parse_arguments<T: serde::de::DeserializeOwned>(
        call: &TurnToolCall,
    ) -> Result<T, TurnToolExecution> {
        serde_json::from_value::<T>(call.arguments.clone()).map_err(|error| {
            TurnToolExecution::Failed {
                code: "invalid_tool_arguments".to_string(),
                message: format!("tool {} arguments are invalid: {error}", call.tool_id),
            }
        })
    }

    fn execute_http(
        &self,
        http: &crate::generations_tool_port::HttpGenerationsPort,
        tool_name: &str,
        call: &TurnToolCall,
        auth_token: &str,
        access_token: Option<&str>,
    ) -> TurnToolExecution {
        use crate::generations_tool_port as port;
        use sdkwork_generations_mcp_service::{
            GenerateImageInput, GenerateMusicInput, GenerateVideoInput, GenerationRetrieveInput,
            SynthesizeSpeechInput,
        };
        let tenant_id = call.tenant_id.unwrap_or(0);
        let result = match tool_name {
            "image.create" => {
                let input = match Self::parse_arguments::<GenerateImageInput>(call) {
                    Ok(input) => input,
                    Err(error) => return error,
                };
                let operation = if input.reference_images.is_empty() {
                    "text_to_image"
                } else {
                    "image_edit"
                };
                http.create_generation(
                    "image",
                    operation,
                    tenant_id,
                    &input.prompt,
                    input.model.as_deref(),
                    port::image_parameters(&input),
                    None,
                    auth_token,
                    access_token,
                    &call.tool_call_id,
                )
                .and_then(|item| generation_payload_with_results(http, &item, auth_token, access_token))
            }
            "video.create" => {
                let input = match Self::parse_arguments::<GenerateVideoInput>(call) {
                    Ok(input) => input,
                    Err(error) => return error,
                };
                let operation = if !input.last_frame.is_some() {
                    if input.reference_images.is_empty() {
                        "text_to_video"
                    } else {
                        "image_to_video"
                    }
                } else {
                    "video_extend"
                };
                http.create_generation(
                    "video",
                    operation,
                    tenant_id,
                    &input.prompt,
                    input.model.as_deref(),
                    port::video_parameters(&input),
                    None,
                    auth_token,
                    access_token,
                    &call.tool_call_id,
                )
                .and_then(|item| generation_payload_with_results(http, &item, auth_token, access_token))
            }
            "speech.create" => {
                let input = match Self::parse_arguments::<SynthesizeSpeechInput>(call) {
                    Ok(input) => input,
                    Err(error) => return error,
                };
                http.create_generation(
                    "voice",
                    "speech",
                    tenant_id,
                    &input.text,
                    input.model.as_deref(),
                    port::speech_parameters(&input),
                    None,
                    auth_token,
                    access_token,
                    &call.tool_call_id,
                )
                .and_then(|item| generation_payload_with_results(http, &item, auth_token, access_token))
            }
            "music.create" => {
                let input = match Self::parse_arguments::<GenerateMusicInput>(call) {
                    Ok(input) => input,
                    Err(error) => return error,
                };
                let operation = if input.lyrics.is_some() {
                    "lyrics_to_music"
                } else {
                    "text_to_music"
                };
                http.create_generation(
                    "music",
                    operation,
                    tenant_id,
                    &input.prompt,
                    input.model.as_deref(),
                    port::music_parameters(&input),
                    None,
                    auth_token,
                    access_token,
                    &call.tool_call_id,
                )
                .and_then(|item| generation_payload_with_results(http, &item, auth_token, access_token))
            }
            "image.retrieve" | "video.retrieve" | "music.retrieve" => {
                let input = match Self::parse_arguments::<GenerationRetrieveInput>(call) {
                    Ok(input) => input,
                    Err(error) => return error,
                };
                let generation = match http.get_generation(&input.generation_id, auth_token, access_token) {
                    Ok(generation) => generation,
                    Err(message) => {
                        return TurnToolExecution::Failed {
                            code: "generations_retrieve_failed".to_string(),
                            message,
                        }
                    }
                };
                let results = http
                    .list_results(&input.generation_id, auth_token, access_token)
                    .unwrap_or_else(|_| serde_json::json!({ "items": [] }));
                let media_urls =
                    extract_generation_media_urls(results.as_array().map(Vec::as_slice).unwrap_or(&[]));
                serde_json::to_string(&serde_json::json!({
                    "generation": generation,
                    "results": results,
                    "mediaUrls": media_urls,
                }))
                .map_err(|error| format!("serialize generation payload failed: {error}"))
            }
            other => Err(format!("generations tool {other:?} is not implemented")),
        };
        match result {
            Ok(content) => TurnToolExecution::Completed { content },
            Err(message) => TurnToolExecution::Failed {
                code: "generations_tool_failed".to_string(),
                message,
            },
        }
    }
}

/// Serializes a generation record into the model-facing tool payload.
///
/// Synchronous vendors (e.g. OpenAI image) finish inside the create call, so
/// the results are fetched immediately and flattened (mirroring the
/// embedded-port `GenerationsToolOutput` shape: `{generation, results,
/// mediaUrls}`). Async vendors return a running record with empty results —
/// the model follows up with the `retrieve` tool, which lists results.
fn generation_payload_with_results(
    http: &crate::generations_tool_port::HttpGenerationsPort,
    generation: &serde_json::Value,
    auth_token: &str,
    access_token: Option<&str>,
) -> Result<String, String> {
    let generation_id = generation
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let results = if generation_id.is_empty() {
        Vec::new()
    } else {
        http.list_results(&generation_id, auth_token, access_token)
            .and_then(|payload| {
                serde_json::from_value::<Vec<serde_json::Value>>(
                    payload.get("items").cloned().unwrap_or_else(|| serde_json::json!([])),
                )
                .map_err(|error| format!("generations results are invalid: {error}"))
            })
            .unwrap_or_default()
    };
    let media_urls = extract_generation_media_urls(&results);
    serde_json::to_string(&serde_json::json!({
        "generation": generation,
        "results": results,
        "mediaUrls": media_urls,
    }))
    .map_err(|error| format!("serialize generation payload failed: {error}"))
}

/// Flattens result `resourceSnapshot.url` values (MEDIA_RESOURCE_SPEC shape).
fn extract_generation_media_urls(results: &[serde_json::Value]) -> Vec<String> {
    results
        .iter()
        .filter_map(|result| {
            result
                .get("resourceSnapshot")
                .and_then(|snapshot| snapshot.get("url"))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|url| !url.is_empty())
                .map(str::to_string)
        })
        .collect()
}

impl TurnToolExecutor for GenerationsToolExecutor {
    fn owns(&self, tool_id: &str) -> bool {
        tool_id.starts_with(GENERATIONS_MCP_TOOL_PREFIX)
    }

    fn descriptors(&self) -> Vec<TurnToolDescriptor> {
        GenerationsToolExecutor::descriptors(self)
    }

    fn execute(
        &self,
        call: &TurnToolCall,
        context: &TurnToolExecutionContext<'_>,
    ) -> TurnToolExecution {
        let tool_name = &call.tool_id[GENERATIONS_MCP_TOOL_PREFIX.len()..];
        match &self.runtime {
            GenerationsToolRuntime::Embedded(port) => {
                let arguments_json =
                    serde_json::to_string(&call.arguments).unwrap_or_else(|_| "{}".to_owned());
                match sdkwork_generations_mcp_service::kernel_invoke::invoke(
                    port,
                    tool_name,
                    &arguments_json,
                ) {
                    Ok(payload) => TurnToolExecution::Completed { content: payload },
                    Err(message) => TurnToolExecution::Failed {
                        code: "generations_tool_failed".to_string(),
                        message,
                    },
                }
            }
            GenerationsToolRuntime::Http(http) => {
                let Some(auth_token) = context
                    .auth_token
                    .map(str::trim)
                    .filter(|token| !token.is_empty())
                else {
                    return TurnToolExecution::Failed {
                        code: "auth_required".to_string(),
                        message: "generations tools require the caller auth token".to_string(),
                    };
                };
                self.execute_http(http, tool_name, call, auth_token, context.access_token)
            }
        }
    }
}

/// Executor for the application-owned media tool family (`sound-effect.*`,
/// `image.*`, `video.*`, `music.*`, `audio.*`, ...).
pub struct MediaToolExecutor {
    registry: Arc<MediaToolRegistry>,
}

impl MediaToolExecutor {
    pub fn new(registry: Arc<MediaToolRegistry>) -> Self {
        Self { registry }
    }

    /// Descriptors for every registered media tool, projected onto the turn
    /// shape. Availability-gated: reserved tools (e.g. sound effects before
    /// the upstream surface opens) are excluded from the model-visible set.
    pub fn descriptors(&self) -> Vec<TurnToolDescriptor> {
        self.registry
            .list_tools()
            .into_iter()
            .filter(|definition| {
                matches!(
                    definition.availability,
                    sdkwork_agents_tool_contract::ToolAvailability::Available
                )
            })
            .map(|definition| TurnToolDescriptor {
                tool_id: definition.tool_id.clone(),
                name: definition.tool_id.clone(),
                description: definition.description.clone(),
                input_schema: definition.input_schema.clone(),
                requires_approval: false,
                policy_category: definition.policy_categories.first().cloned(),
                timeout_ms: definition.timeout_ms.max(1),
                origin: TurnToolOrigin::BuiltinMedia,
            })
            .collect()
    }
}

impl TurnToolExecutor for MediaToolExecutor {
    fn owns(&self, tool_id: &str) -> bool {
        self.registry.describe_tool(tool_id).is_some()
    }

    fn descriptors(&self) -> Vec<TurnToolDescriptor> {
        MediaToolExecutor::descriptors(self)
    }

    fn execute(
        &self,
        call: &TurnToolCall,
        context: &TurnToolExecutionContext<'_>,
    ) -> TurnToolExecution {
        let media_call = MediaToolCall {
            tool_call_id: call.tool_call_id.clone(),
            tool_id: call.tool_id.clone(),
            arguments: call.arguments.clone(),
            session_id: call.session_id.clone(),
            trace_id: call.trace_id.clone(),
        };
        match self.registry.invoke(&media_call, context.auth_token) {
            Ok(result) if result.status == "succeeded" => TurnToolExecution::Completed {
                content: serde_json::to_string(&result.output).unwrap_or_else(|_| "{}".to_owned()),
            },
            Ok(result) => TurnToolExecution::Failed {
                code: result.status.clone(),
                message: result
                    .error
                    .unwrap_or_else(|| format!("media tool {} returned status {}", call.tool_id, result.status)),
            },
            Err(error) => TurnToolExecution::Failed {
                code: error.code().to_string(),
                message: error.to_string(),
            },
        }
    }
}

/// Connection configuration for one user-registered MCP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerConnection {
    /// Stable server key (the `mcp__<server>__<tool>` namespace segment).
    pub server_key: String,
    /// JSON-RPC 2.0 HTTP endpoint of the published revision.
    pub endpoint_url: String,
    /// `none` | `bearer` | `api_key` — how the secret is presented.
    pub auth_type: String,
    /// Opaque secret reference resolved through [`McpSecretResolver`].
    pub secret_ref: Option<String>,
    /// Connection budget in milliseconds.
    pub timeout_ms: u64,
}

/// Resolves opaque secret references to credential material at call time.
pub trait McpSecretResolver: Send + Sync {
    fn resolve(&self, secret_ref: &str) -> Option<String>;
}

/// Empty secret resolver (no external secrets provisioned; fail closed).
#[derive(Default)]
pub struct EmptyMcpSecretResolver;

impl McpSecretResolver for EmptyMcpSecretResolver {
    fn resolve(&self, _secret_ref: &str) -> Option<String> {
        None
    }
}

/// Environment-backed secret resolver.
///
/// Resolves `secret_ref` from `SDKWORK_AGENTS_MCP_SECRET__<REF>` where `<REF>`
/// is the reference with every non-alphanumeric run collapsed to `_` (e.g.
/// `vault://mcp/browser` becomes `SDKWORK_AGENTS_MCP_SECRET__VAULT_MCP_BROWSER`).
/// The deployment supplies the secret material; unresolved references fail
/// the tool call closed.
#[derive(Debug, Default, Clone)]
pub struct EnvMcpSecretResolver;

impl EnvMcpSecretResolver {
    fn env_key(secret_ref: &str) -> String {
        let mut key = String::from("SDKWORK_AGENTS_MCP_SECRET__");
        let mut previous_was_separator = true;
        for character in secret_ref.chars() {
            if character.is_ascii_alphanumeric() {
                key.extend(character.to_uppercase());
                previous_was_separator = false;
            } else if !previous_was_separator {
                key.push('_');
                previous_was_separator = true;
            }
        }
        while key.ends_with('_') {
            key.pop();
        }
        key
    }
}

impl McpSecretResolver for EnvMcpSecretResolver {
    fn resolve(&self, secret_ref: &str) -> Option<String> {
        let key = Self::env_key(secret_ref);
        std::env::var(&key)
            .ok()
            .filter(|value| !value.trim().is_empty())
    }
}

/// Executor for user-registered external MCP servers over JSON-RPC 2.0 HTTP.
///
/// Connections are resolved per turn from the agent's composition-slot
/// policies (carried on [`TurnToolExecutionContext`]); `tools/call` responses
/// are flattened to the concatenated `text` content blocks; `isError: true`
/// results surface as tool failures so the model can self-heal.
pub struct ExternalMcpToolExecutor {
    secrets: Arc<dyn McpSecretResolver>,
}

impl ExternalMcpToolExecutor {
    pub fn new(secrets: Arc<dyn McpSecretResolver>) -> Self {
        Self { secrets }
    }

    /// Splits `mcp__<server>__<tool>` into server key and tool name.
    fn split_tool_id(tool_id: &str) -> Option<(String, String)> {
        let rest = tool_id.strip_prefix(EXTERNAL_MCP_TOOL_PREFIX)?;
        let (server, tool) = rest.split_once("__")?;
        let server = server.trim();
        let tool = tool.trim();
        (!server.is_empty() && !tool.is_empty()).then(|| (server.to_string(), tool.to_string()))
    }
}

impl TurnToolExecutor for ExternalMcpToolExecutor {
    fn owns(&self, tool_id: &str) -> bool {
        tool_id.starts_with(EXTERNAL_MCP_TOOL_PREFIX)
            && !tool_id.starts_with(GENERATIONS_MCP_TOOL_PREFIX)
    }

    fn execute(
        &self,
        call: &TurnToolCall,
        context: &TurnToolExecutionContext<'_>,
    ) -> TurnToolExecution {
        let Some((server_key, tool_name)) = Self::split_tool_id(&call.tool_id) else {
            return TurnToolExecution::Failed {
                code: "invalid_mcp_tool_id".to_string(),
                message: format!("external MCP tool id `{}` is malformed", call.tool_id),
            };
        };
        let Some(connection) = context
            .mcp_connections
            .iter()
            .find(|connection| connection.server_key == server_key)
        else {
            return TurnToolExecution::Failed {
                code: "mcp_server_unavailable".to_string(),
                message: format!("MCP server `{server_key}` is not bound for this agent"),
            };
        };
        let connection = connection.clone();
        let mut headers = Vec::new();
        match connection.auth_type.as_str() {
            "none" => {}
            "bearer" => match connection
                .secret_ref
                .as_deref()
                .and_then(|secret_ref| self.secrets.resolve(secret_ref))
            {
                Some(secret) => headers.push(("Authorization".to_string(), format!("Bearer {secret}"))),
                None => {
                    return TurnToolExecution::Failed {
                        code: "mcp_credential_unresolved".to_string(),
                        message: format!("MCP server `{server_key}` credential is unavailable"),
                    }
                }
            },
            "api_key" => match connection
                .secret_ref
                .as_deref()
                .and_then(|secret_ref| self.secrets.resolve(secret_ref))
            {
                Some(secret) => headers.push(("x-api-key".to_string(), secret)),
                None => {
                    return TurnToolExecution::Failed {
                        code: "mcp_credential_unresolved".to_string(),
                        message: format!("MCP server `{server_key}` credential is unavailable"),
                    }
                }
            },
            other => {
                return TurnToolExecution::Failed {
                    code: "unsupported_mcp_auth".to_string(),
                    message: format!("MCP server `{server_key}` auth type `{other}` is unsupported"),
                }
            }
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": call.tool_call_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": call.arguments,
            }
        });
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(connection.timeout_ms.max(1_000)))
            .build();
        let client = match client {
            Ok(client) => client,
            Err(error) => {
                return TurnToolExecution::Failed {
                    code: "mcp_client_build_failed".to_string(),
                    message: error.to_string(),
                }
            }
        };
        let mut request = client.post(&connection.endpoint_url).json(&body);
        for (name, value) in &headers {
            request = request.header(name, value);
        }
        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                return TurnToolExecution::Failed {
                    code: "mcp_http_error".to_string(),
                    message: format!("MCP server `{server_key}` call failed: {error}"),
                }
            }
        };
        let payload = match response.json::<serde_json::Value>() {
            Ok(payload) => payload,
            Err(error) => {
                return TurnToolExecution::Failed {
                    code: "mcp_invalid_response".to_string(),
                    message: format!("MCP server `{server_key}` returned an invalid response: {error}"),
                }
            }
        };
        match payload.get("error") {
            Some(error) => TurnToolExecution::Failed {
                code: "mcp_tool_error".to_string(),
                message: serde_json::to_string(error).unwrap_or_else(|_| "mcp tool error".to_string()),
            },
            None => {
                let content = extract_mcp_text_content(&payload);
                let is_error = payload
                    .get("result")
                    .and_then(|result| result.get("isError"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                if is_error {
                    TurnToolExecution::Failed {
                        code: "mcp_tool_failed".to_string(),
                        message: content,
                    }
                } else {
                    TurnToolExecution::Completed { content }
                }
            }
        }
    }
}

/// Flattens a JSON-RPC `tools/call` result to its concatenated text blocks.
fn extract_mcp_text_content(payload: &serde_json::Value) -> String {
    payload
        .get("result")
        .and_then(|result| result.get("content"))
        .and_then(serde_json::Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|block| block.get("text").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|| serde_json::to_string(payload).unwrap_or_default())
}

/// Length-caps tool activity before it is persisted or fed back to the model.
pub fn cap_tool_content(value: &str) -> String {
    let mut capped: String = value.chars().take(MAX_TOOL_RESULT_CONTENT_CHARS).collect();
    if value.chars().count() > MAX_TOOL_RESULT_CONTENT_CHARS {
        capped.push_str("…[truncated]");
    }
    capped
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_generations_mcp_service::InMemoryGenerationsMcpPort;

    #[test]
    fn dispatcher_fails_closed_for_unknown_tools() {
        let dispatcher = TurnToolDispatcher::new()
            .with_executor(Box::new(MediaToolExecutor::new(Arc::new(
                MediaToolRegistry::new(),
            ))));
        let call = TurnToolCall {
            tool_call_id: "call.1".to_string(),
            tool_id: "unknown.tool".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
            tenant_id: None,
        };
        let context = TurnToolExecutionContext {
            auth_token: None,
            access_token: None,
            mcp_connections: &[],
        };
        let error = dispatcher.execute(&call, &context).expect_err("unknown tool");
        assert!(error.contains("no tool executor"));
    }

    #[test]
    fn generations_executor_owns_only_its_namespace() {
        let executor = GenerationsToolExecutor::new(Arc::new(
            InMemoryGenerationsMcpPort::new(),
        ));
        assert!(executor.owns("mcp__generations__image.create"));
        assert!(!executor.owns("mcp__other__tool"));
        assert!(!executor.owns("image.generate"));
    }

    #[test]
    fn generations_descriptors_cover_the_default_media_set() {
        let executor = GenerationsToolExecutor::new(Arc::new(
            InMemoryGenerationsMcpPort::new(),
        ));
        let descriptors = executor.descriptors();
        let ids: Vec<&str> = descriptors.iter().map(|d| d.tool_id.as_str()).collect();
        for expected in [
            "mcp__generations__image.create",
            "mcp__generations__image.retrieve",
            "mcp__generations__video.create",
            "mcp__generations__video.retrieve",
            "mcp__generations__speech.create",
            "mcp__generations__music.create",
            "mcp__generations__music.retrieve",
        ] {
            assert!(ids.contains(&expected), "missing default tool {expected}");
        }
        assert!(descriptors
            .iter()
            .all(|descriptor| !descriptor.input_schema.is_null()));
    }

    #[test]
    fn media_executor_excludes_pending_capability_tools() {
        let executor = MediaToolExecutor::new(Arc::new(MediaToolRegistry::new()));
        let descriptors = executor.descriptors();
        // Sound effects are reserved until the upstream surface opens; they
        // must not be advertised to the model while invocation would fail.
        assert!(!descriptors
            .iter()
            .any(|descriptor| descriptor.tool_id == "sound-effect.generate"));
        // The synchronous media family stays available.
        assert!(descriptors
            .iter()
            .any(|descriptor| descriptor.tool_id == "audio.speech.create"));
    }

    #[test]
    fn external_executor_splits_namespaced_tool_ids() {
        assert_eq!(
            ExternalMcpToolExecutor::split_tool_id("mcp__browser__navigate"),
            Some(("browser".to_string(), "navigate".to_string()))
        );
        assert_eq!(
            ExternalMcpToolExecutor::split_tool_id("mcp__generations__image.create"),
            Some(("generations".to_string(), "image.create".to_string()))
        );
        assert_eq!(ExternalMcpToolExecutor::split_tool_id("mcp__onlyone"), None);
        assert_eq!(ExternalMcpToolExecutor::split_tool_id("image.create"), None);
    }

    #[test]
    fn external_executor_rejects_unbound_servers() {
        let executor = ExternalMcpToolExecutor::new(Arc::new(EmptyMcpSecretResolver));
        let call = TurnToolCall {
            tool_call_id: "call.2".to_string(),
            tool_id: "mcp__browser__navigate".to_string(),
            arguments: serde_json::json!({ "url": "https://example.com" }),
            session_id: None,
            trace_id: None,
            tenant_id: None,
        };
        let context = TurnToolExecutionContext {
            auth_token: None,
            access_token: None,
            mcp_connections: &[],
        };
        match executor.execute(&call, &context) {
            TurnToolExecution::Failed { code, .. } => assert_eq!(code, "mcp_server_unavailable"),
            other => panic!("expected failure, got {other:?}"),
        }
    }

    #[test]
    fn external_executor_fails_closed_when_bearer_credential_unresolved() {
        let executor = ExternalMcpToolExecutor::new(Arc::new(EmptyMcpSecretResolver));
        let call = TurnToolCall {
            tool_call_id: "call.3".to_string(),
            tool_id: "mcp__browser__navigate".to_string(),
            arguments: serde_json::json!({ "url": "https://example.com" }),
            session_id: None,
            trace_id: None,
            tenant_id: None,
        };
        let connections = vec![McpServerConnection {
            server_key: "browser".to_string(),
            endpoint_url: "https://mcp.example.test/rpc".to_string(),
            auth_type: "bearer".to_string(),
            secret_ref: Some("vault://mcp/browser".to_string()),
            timeout_ms: 5_000,
        }];
        let context = TurnToolExecutionContext {
            auth_token: None,
            access_token: None,
            mcp_connections: &connections,
        };
        match executor.execute(&call, &context) {
            TurnToolExecution::Failed { code, message } => {
                assert_eq!(code, "mcp_credential_unresolved");
                assert!(message.contains("browser"));
            }
            other => panic!("expected failure, got {other:?}"),
        }
    }

    #[test]
    fn external_executor_rejects_unsupported_auth_types() {
        let executor = ExternalMcpToolExecutor::new(Arc::new(EmptyMcpSecretResolver));
        let call = TurnToolCall {
            tool_call_id: "call.4".to_string(),
            tool_id: "mcp__legacy__call".to_string(),
            arguments: serde_json::json!({}),
            session_id: None,
            trace_id: None,
            tenant_id: None,
        };
        let connections = vec![McpServerConnection {
            server_key: "legacy".to_string(),
            endpoint_url: "https://legacy.example.test/rpc".to_string(),
            auth_type: "oauth2_signing".to_string(),
            secret_ref: None,
            timeout_ms: 5_000,
        }];
        let context = TurnToolExecutionContext {
            auth_token: None,
            access_token: None,
            mcp_connections: &connections,
        };
        match executor.execute(&call, &context) {
            TurnToolExecution::Failed { code, .. } => assert_eq!(code, "unsupported_mcp_auth"),
            other => panic!("expected failure, got {other:?}"),
        }
    }

    #[test]
    fn external_executor_invokes_bound_server_over_jsonrpc() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock mcp server");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = vec![0u8; 64 * 1024];
            let mut received = Vec::new();
            loop {
                let read = stream.read(&mut buffer).expect("read request");
                if read == 0 {
                    break;
                }
                received.extend_from_slice(&buffer[..read]);
                if received.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": "call.5",
                "result": {
                    "content": [{"type": "text", "text": "navigated to example.com"}],
                    "isError": false
                }
            });
            let payload = serde_json::to_string(&body).expect("serialize");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                payload.len(),
                payload
            );
            stream.write_all(response.as_bytes()).expect("write");
        });

        let executor = ExternalMcpToolExecutor::new(Arc::new(EmptyMcpSecretResolver));
        let call = TurnToolCall {
            tool_call_id: "call.5".to_string(),
            tool_id: "mcp__browser__navigate".to_string(),
            arguments: serde_json::json!({ "url": "https://example.com" }),
            session_id: None,
            trace_id: None,
            tenant_id: None,
        };
        let connections = vec![McpServerConnection {
            server_key: "browser".to_string(),
            endpoint_url: format!("{base_url}/rpc"),
            auth_type: "none".to_string(),
            secret_ref: None,
            timeout_ms: 5_000,
        }];
        let context = TurnToolExecutionContext {
            auth_token: None,
            access_token: None,
            mcp_connections: &connections,
        };
        match executor.execute(&call, &context) {
            TurnToolExecution::Completed { content } => {
                assert!(content.contains("navigated to example.com"));
            }
            other => panic!("expected completion, got {other:?}"),
        }
        server.join().expect("mock server thread");
    }

    #[test]
    fn env_secret_resolver_normalizes_references() {
        assert_eq!(
            EnvMcpSecretResolver::env_key("vault://mcp/browser"),
            "SDKWORK_AGENTS_MCP_SECRET__VAULT_MCP_BROWSER"
        );
        assert_eq!(
            EnvMcpSecretResolver::env_key("plain-key"),
            "SDKWORK_AGENTS_MCP_SECRET__PLAIN_KEY"
        );
    }

    #[test]
    fn env_secret_resolver_resolves_provisioned_material() {
        let reference = "vault://mcp/unit-test";
        let key = EnvMcpSecretResolver::env_key(reference);
        // SAFETY: single-threaded test process; the variable name is unique
        // to this test.
        std::env::set_var(&key, "unit-test-secret");
        let resolved = EnvMcpSecretResolver.resolve(reference);
        // SAFETY: cleanup of the variable this test exclusively owns.
        std::env::remove_var(&key);
        assert_eq!(resolved.as_deref(), Some("unit-test-secret"));
    }

    #[test]
    fn tool_content_capping_bounds_results() {
        let long = "x".repeat(MAX_TOOL_RESULT_CONTENT_CHARS + 100);
        let capped = cap_tool_content(&long);
        assert!(capped.ends_with("…[truncated]"));
        assert_eq!(capped.chars().count(), MAX_TOOL_RESULT_CONTENT_CHARS + "…[truncated]".chars().count());
        assert_eq!(cap_tool_content("short"), "short");
    }
}
