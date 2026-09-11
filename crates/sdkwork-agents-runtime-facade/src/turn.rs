use std::path::PathBuf;

use sdkwork_agent_kernel::{
    AgentExecutionProviderOptionValue, KernelEvent, KernelResult, ModelRequest, ModelResponse,
    ModelStatus, ModelStreamChunk, ModelStreamSink, ToolCall,
};
use sdkwork_agent_provider_spi::SdkRuntimeStreamCompletion;
use sdkwork_utils_rust::string::is_blank;

use crate::agent_engines::AgentEngineSlot;
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

/// Maximum prompt size accepted by the runtime facade (1 MiB).
pub const MAX_AGENT_ENGINE_PROMPT_BYTES: usize = 1_048_576;
/// Maximum stream chunks collected before failing closed.
pub const MAX_AGENT_ENGINE_STREAM_CHUNKS: usize = 8_192;
/// Maximum aggregated stream output size (4 MiB).
pub const MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES: usize = 4_194_304;
/// Maximum provider request identity size accepted by the runtime facade.
pub const MAX_AGENT_ENGINE_MODEL_REQUEST_ID_BYTES: usize = 512;

const WORKING_DIRECTORY_METADATA_KEY: &str = "sdkwork.agent_engine.working_directory";
const APPROVAL_POLICY_METADATA_KEY: &str = "sdkwork.agent_engine.approval_policy";
const SANDBOX_MODE_METADATA_KEY: &str = "sdkwork.agent_engine.sandbox_mode";
const FULL_AUTO_METADATA_KEY: &str = "sdkwork.agent_engine.full_auto";
const SKIP_GIT_REPO_CHECK_METADATA_KEY: &str = "sdkwork.agent_engine.skip_git_repo_check";
const EPHEMERAL_METADATA_KEY: &str = "sdkwork.agent_engine.ephemeral";
const REQUIRE_LIVE_PROVIDER_METADATA_KEY: &str = "sdkwork.agent_engine.require_live_provider";
const MAX_OUTPUT_BYTES_METADATA_KEY: &str = "sdkwork.agent_engine.max_output_bytes";
const TEMPERATURE_METADATA_KEY: &str = "sdkwork.agent_engine.temperature";
const TOP_P_METADATA_KEY: &str = "sdkwork.agent_engine.top_p";
const MAX_TOKENS_METADATA_KEY: &str = "sdkwork.agent_engine.max_tokens";
const PROVIDER_SESSION_DIAGNOSTIC_KEYS: [&str; 6] = [
    "sdk_runtime_session_id",
    "sdk_runtime_provider_session_id",
    "sdkwork.agent_engine.provider_session_id",
    "sdkwork.provider.session_id",
    "provider_session_id",
    "provider_session_id",
];

/// Product-neutral agent-engine turn input consumed by the agents runtime facade.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AgentEngineTurnInput {
    pub engine_key: String,
    pub model_id: String,
    /// Stable request identity established by the application before execution.
    pub model_request_id: Option<String>,
    /// Canonical SDKWork Session identity used by kernel-owned lifecycle state.
    pub session_id: Option<String>,
    /// Canonical SDKWork Turn identity used for provider event correlation.
    pub turn_id: Option<String>,
    /// Opaque continuation identity established by the selected provider.
    pub provider_session_id: Option<String>,
    pub prompt: String,
    pub working_directory: Option<PathBuf>,
    pub timeout_ms: Option<u64>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
    pub full_auto: bool,
    pub skip_git_repo_check: bool,
    pub ephemeral: bool,
    pub require_live_provider: bool,
    pub max_output_bytes: Option<usize>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub access_mode_id: Option<String>,
    /// Transient caller auth token (`Authorization: Bearer`) forwarded to the
    /// model provider for product-facing routing (cloud-router dual-token).
    /// Never persisted.
    pub auth_token: Option<String>,
    /// Transient caller access token (`Access-Token` header) paired with
    /// `auth_token` for cloud-router dual-token routing. Never persisted.
    pub access_token: Option<String>,
}

/// Provider-neutral terminal metadata for a streamed agent-engine turn.
///
/// The facade constructs this value only after the runtime verifies that the
/// terminal `model_request_id` belongs to the active turn and that the
/// provider supplied a non-empty provider session id. Product callers therefore
/// never inspect provider diagnostics or transport frames to resume a turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentEngineTurnStreamCompletion {
    pub model_request_id: String,
    pub finish_reason: String,
    pub provider_session_id: String,
}

/// Product-neutral agent-engine turn output produced by the agents runtime facade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentEngineTurnOutput {
    pub model_request_id: String,
    pub finish_reason: Option<String>,
    pub assistant_content: String,
    pub provider_session_id: Option<String>,
    /// Provider-neutral tool calls returned by the kernel model provider.
    pub tool_calls: Vec<ToolCall>,
    /// Token/word deltas when streaming is available; empty when invoke-only.
    pub stream_deltas: Vec<String>,
    /// Provider-neutral lifecycle and item events emitted during the turn.
    pub stream_events: Vec<KernelEvent>,
    /// Verified terminal metadata for a streamed turn. `None` means the turn
    /// used invoke-only execution or the provider cannot prove completion.
    pub stream_completion: Option<AgentEngineTurnStreamCompletion>,
}

/// Correlated provider acknowledgement for one cancelled agent-engine Turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentEngineTurnCancellation {
    pub model_request_id: String,
    pub finish_reason: String,
}

/// Derives the stable provider request identity for one canonical SDKWork Turn.
///
/// The `agents-turn-` prefix is a model-request protocol field: it is
/// correlated with the model provider (cancellation matching, message
/// association) and is not a durable id, so it intentionally stays outside
/// the dot-separated durable id scheme.
pub fn agent_engine_model_request_id(turn_id: &str) -> String {
    format!("agents-turn-{turn_id}")
}

pub fn execute_agent_engine_turn(
    slot: &AgentEngineSlot,
    input: &AgentEngineTurnInput,
) -> RuntimeFacadeResult<AgentEngineTurnOutput> {
    if slot.engine_key() != input.engine_key {
        return Err(RuntimeFacadeError::EngineMismatch {
            slot_engine: slot.engine_key().to_string(),
            input_engine: input.engine_key.clone(),
        });
    }
    if is_blank(Some(input.prompt.as_str())) {
        return Err(RuntimeFacadeError::BlankPrompt);
    }
    if input.prompt.len() > MAX_AGENT_ENGINE_PROMPT_BYTES {
        return Err(RuntimeFacadeError::Kernel(format!(
            "prompt exceeds maximum size of {MAX_AGENT_ENGINE_PROMPT_BYTES} bytes"
        )));
    }

    let model_request = build_model_request(slot, input)?;

    let response = slot
        .invoke_model(model_request)
        .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))?;

    let assistant_content = response.messages.join("\n");
    validate_output_size(assistant_content.len(), effective_max_output_bytes(input))?;

    Ok(build_turn_output(response, input))
}

fn build_turn_output(response: ModelResponse, input: &AgentEngineTurnInput) -> AgentEngineTurnOutput {
    let assistant_content = response.messages.join("\n");
    AgentEngineTurnOutput {
        model_request_id: response.model_request_id.clone(),
        finish_reason: response.finish_reason.clone(),
        assistant_content,
        provider_session_id: resolve_provider_session_id(&response, input),
        tool_calls: response.tool_calls,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
        stream_completion: None,
    }
}

fn build_model_request(
    slot: &AgentEngineSlot,
    input: &AgentEngineTurnInput,
) -> RuntimeFacadeResult<ModelRequest> {
    let resolved_execution_settings = input
        .access_mode_id
        .as_deref()
        .map(str::trim)
        .filter(|access_mode_id| !access_mode_id.is_empty())
        .map(|access_mode_id| {
            slot.resolve_execution_settings(access_mode_id)
                .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))
        })
        .transpose()?;
    let model_request_id = match input.model_request_id.as_deref() {
        Some(model_request_id) if is_blank(Some(model_request_id)) => {
            return Err(RuntimeFacadeError::InvalidInput(
                "model_request_id must not be blank".to_string(),
            ));
        }
        Some(model_request_id)
            if model_request_id.len() > MAX_AGENT_ENGINE_MODEL_REQUEST_ID_BYTES =>
        {
            return Err(RuntimeFacadeError::InvalidInput(format!(
                "model_request_id exceeds {MAX_AGENT_ENGINE_MODEL_REQUEST_ID_BYTES} bytes"
            )));
        }
        Some(model_request_id) => model_request_id.to_string(),
        None => format!("agents-turn-{}", sdkwork_utils_rust::uuid()),
    };
    let mut model_request = ModelRequest::new(model_request_id, vec![input.prompt.clone()]);
    if !is_blank(Some(input.model_id.as_str())) {
        model_request.model_id = Some(input.model_id.clone());
    }
    if let Some(session_id) = input.session_id.as_ref() {
        model_request.session_id = Some(session_id.clone());
    }
    if let Some(turn_id) = input.turn_id.as_ref() {
        model_request.step_id = Some(turn_id.clone());
    }
    if let Some(provider_session_id) = input.provider_session_id.as_ref() {
        model_request.provider_session_id = Some(provider_session_id.clone());
    }
    if let Some(timeout_ms) = input.timeout_ms {
        model_request.timeout_ms = Some(timeout_ms);
    }
    if input.auth_token.is_some() || input.access_token.is_some() {
        model_request =
            model_request.for_caller(input.auth_token.clone(), input.access_token.clone());
    }
    if let Some(working_directory) = input.working_directory.as_ref() {
        let value = working_directory.to_string_lossy();
        if !is_blank(Some(value.as_ref())) {
            model_request = model_request.with_metadata(WORKING_DIRECTORY_METADATA_KEY, value);
        }
    }
    if resolved_execution_settings.is_none() {
        model_request = with_optional_metadata(
            model_request,
            APPROVAL_POLICY_METADATA_KEY,
            input.approval_policy.as_deref(),
        );
        model_request = with_optional_metadata(
            model_request,
            SANDBOX_MODE_METADATA_KEY,
            input.sandbox_mode.as_deref(),
        );
    }
    model_request = model_request
        .with_metadata(FULL_AUTO_METADATA_KEY, input.full_auto.to_string())
        .with_metadata(
            SKIP_GIT_REPO_CHECK_METADATA_KEY,
            input.skip_git_repo_check.to_string(),
        )
        .with_metadata(EPHEMERAL_METADATA_KEY, input.ephemeral.to_string())
        .with_metadata(
            REQUIRE_LIVE_PROVIDER_METADATA_KEY,
            input.require_live_provider.to_string(),
        )
        .with_metadata(
            MAX_OUTPUT_BYTES_METADATA_KEY,
            effective_max_output_bytes(input).to_string(),
        );
    if let Some(temperature) = input.temperature {
        model_request =
            model_request.with_metadata(TEMPERATURE_METADATA_KEY, temperature.to_string());
    }
    if let Some(top_p) = input.top_p {
        model_request = model_request.with_metadata(TOP_P_METADATA_KEY, top_p.to_string());
    }
    if let Some(max_tokens) = input.max_tokens {
        model_request =
            model_request.with_metadata(MAX_TOKENS_METADATA_KEY, max_tokens.to_string());
    }
    if let Some(resolved) = resolved_execution_settings {
        for option in resolved.provider_options {
            let value = match option.value {
                AgentExecutionProviderOptionValue::String(value) => value,
                AgentExecutionProviderOptionValue::Boolean(value) => value.to_string(),
            };
            model_request = model_request.with_metadata(option.key, value);
        }
    }
    Ok(model_request)
}

fn with_optional_metadata(request: ModelRequest, key: &str, value: Option<&str>) -> ModelRequest {
    match value.filter(|candidate| !is_blank(Some(*candidate))) {
        Some(candidate) => request.with_metadata(key, candidate.trim()),
        None => request,
    }
}

fn effective_max_output_bytes(input: &AgentEngineTurnInput) -> usize {
    input
        .max_output_bytes
        .unwrap_or(MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES)
        .min(MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES)
}

fn validate_output_size(output_bytes: usize, max_output_bytes: usize) -> RuntimeFacadeResult<()> {
    if output_bytes > max_output_bytes {
        return Err(RuntimeFacadeError::Kernel(format!(
            "agent-engine output exceeds maximum size of {max_output_bytes} bytes"
        )));
    }
    Ok(())
}

fn resolve_provider_session_id(
    response: &ModelResponse,
    input: &AgentEngineTurnInput,
) -> Option<String> {
    response
        .diagnostics
        .iter()
        .find_map(|diagnostic| {
            let (key, value) = diagnostic.split_once('=')?;
            PROVIDER_SESSION_DIAGNOSTIC_KEYS
                .contains(&key.trim())
                .then(|| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| input.provider_session_id.clone())
}

/// Execute a turn preferring provider stream chunks when supported.
pub fn execute_agent_engine_turn_with_stream(
    slot: &AgentEngineSlot,
    input: &AgentEngineTurnInput,
) -> RuntimeFacadeResult<AgentEngineTurnOutput> {
    let mut sink = DiscardingModelStreamSink;
    execute_agent_engine_turn_with_stream_sink(slot, input, &mut sink)
}

/// Execute a turn and forward each provider-neutral model chunk as it arrives.
/// The caller owns its product representation; this facade owns the kernel SPI
/// boundary, ordering, output budget enforcement, and provider-session proof.
///
/// Codex initial turns can stream only because its runtime terminal frame
/// carries a correlated provider session id. Other engines remain invoke-only
/// until they offer the same proof. Once a chunk is delivered, this function
/// never invokes the provider again as a fallback.
pub fn execute_agent_engine_turn_with_stream_sink(
    slot: &AgentEngineSlot,
    input: &AgentEngineTurnInput,
    sink: &mut dyn ModelStreamSink,
) -> RuntimeFacadeResult<AgentEngineTurnOutput> {
    if slot.engine_key() != input.engine_key {
        return Err(RuntimeFacadeError::EngineMismatch {
            slot_engine: slot.engine_key().to_string(),
            input_engine: input.engine_key.clone(),
        });
    }
    if is_blank(Some(input.prompt.as_str())) {
        return Err(RuntimeFacadeError::BlankPrompt);
    }
    if input.prompt.len() > MAX_AGENT_ENGINE_PROMPT_BYTES {
        return Err(RuntimeFacadeError::Kernel(format!(
            "prompt exceeds maximum size of {MAX_AGENT_ENGINE_PROMPT_BYTES} bytes"
        )));
    }
    if slot.supports_streaming_completion() {
        return execute_with_stream_completion(slot, input, sink);
    }
    if input.provider_session_id.is_none() {
        return execute_agent_engine_turn(slot, input);
    }

    let model_request = build_model_request(slot, input)?;
    let model_request_id = model_request.model_request_id.clone();
    let mut collector = ForwardingStreamCollector::new(sink);
    match slot.stream_model_into(model_request, &mut collector) {
        Ok(()) if !collector.is_empty() => {
            let (stream_deltas, stream_events) = collector.into_parts();
            return build_streamed_turn_output(
                input,
                model_request_id,
                stream_deltas,
                stream_events,
                None,
            );
        }
        Ok(()) => {
            return Err(RuntimeFacadeError::Kernel(
                "provider stream completed without output; invoke fallback is unsafe after execution"
                    .to_string(),
            ));
        }
        Err(_error) if collector.is_empty() => {}
        Err(error) => {
            return Err(RuntimeFacadeError::Kernel(error.to_string()));
        }
    }

    execute_agent_engine_turn(slot, input)
}

fn execute_with_stream_completion(
    slot: &AgentEngineSlot,
    input: &AgentEngineTurnInput,
    sink: &mut dyn ModelStreamSink,
) -> RuntimeFacadeResult<AgentEngineTurnOutput> {
    let model_request = build_model_request(slot, input)?;
    let mut collector = ForwardingStreamCollector::new(sink);
    let runtime_completion = slot
        .stream_model_into_with_completion(model_request, &mut collector)
        .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))?;
    let completion = agent_engine_stream_completion(runtime_completion)?;

    let (stream_deltas, stream_events) = collector.into_parts();
    build_streamed_turn_output(
        input,
        completion.model_request_id.clone(),
        stream_deltas,
        stream_events,
        Some(completion),
    )
}

fn agent_engine_stream_completion(
    runtime_completion: SdkRuntimeStreamCompletion,
) -> RuntimeFacadeResult<AgentEngineTurnStreamCompletion> {
    let provider_session_id = runtime_completion
        .provider_session_id
        .filter(|value| !is_blank(Some(value.as_str())))
        .ok_or_else(|| {
            RuntimeFacadeError::Kernel(
                "provider stream completed without a verified provider session id".to_string(),
            )
        })?;

    Ok(AgentEngineTurnStreamCompletion {
        model_request_id: runtime_completion.model_request_id,
        finish_reason: runtime_completion.finish_reason,
        provider_session_id,
    })
}

fn build_streamed_turn_output(
    input: &AgentEngineTurnInput,
    model_request_id: String,
    stream_deltas: Vec<String>,
    stream_events: Vec<KernelEvent>,
    stream_completion: Option<AgentEngineTurnStreamCompletion>,
) -> RuntimeFacadeResult<AgentEngineTurnOutput> {
    let finish_reason = stream_completion
        .as_ref()
        .map(|completion| completion.finish_reason.clone());
    let cancelled = finish_reason.as_deref() == Some("cancelled");
    if stream_deltas.is_empty() && !cancelled {
        return Err(RuntimeFacadeError::Kernel(
            "provider stream completed without output".to_string(),
        ));
    }

    let assistant_content = stream_deltas.join("");
    validate_output_size(assistant_content.len(), effective_max_output_bytes(input))?;
    if assistant_content.trim().is_empty() && !cancelled {
        return Err(RuntimeFacadeError::Kernel(
            "provider stream completed with blank output".to_string(),
        ));
    }

    let provider_session_id = stream_completion
        .as_ref()
        .map(|completion| completion.provider_session_id.clone())
        .or_else(|| input.provider_session_id.clone())
        .ok_or_else(|| {
            RuntimeFacadeError::Kernel(
                "provider stream completed without a provider session id".to_string(),
            )
        })?;

    Ok(AgentEngineTurnOutput {
        model_request_id,
        finish_reason,
        assistant_content,
        provider_session_id: Some(provider_session_id),
        tool_calls: Vec::new(),
        stream_deltas,
        stream_events,
        stream_completion,
    })
}

pub fn cancel_agent_engine_turn(
    slot: &AgentEngineSlot,
    model_request_id: &str,
) -> RuntimeFacadeResult<AgentEngineTurnCancellation> {
    if is_blank(Some(model_request_id)) {
        return Err(RuntimeFacadeError::InvalidInput(
            "model_request_id must not be blank".to_string(),
        ));
    }
    let response = slot
        .cancel_model(model_request_id)
        .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))?;
    if response.model_request_id != model_request_id
        || response.status != ModelStatus::Cancelled
        || response.finish_reason.as_deref() != Some("cancelled")
    {
        return Err(RuntimeFacadeError::Kernel(
            "provider cancellation did not return a correlated cancelled acknowledgement"
                .to_string(),
        ));
    }
    Ok(AgentEngineTurnCancellation {
        model_request_id: response.model_request_id,
        finish_reason: "cancelled".to_string(),
    })
}

struct ForwardingStreamCollector<'a> {
    inner: &'a mut dyn ModelStreamSink,
    deltas: Vec<String>,
    events: Vec<KernelEvent>,
    bytes: usize,
}

impl<'a> ForwardingStreamCollector<'a> {
    fn new(inner: &'a mut dyn ModelStreamSink) -> Self {
        Self {
            inner,
            deltas: Vec::new(),
            events: Vec::new(),
            bytes: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.deltas.is_empty() && self.events.is_empty()
    }

    fn into_parts(self) -> (Vec<String>, Vec<KernelEvent>) {
        (self.deltas, self.events)
    }
}

impl ModelStreamSink for ForwardingStreamCollector<'_> {
    fn push_chunk(&mut self, chunk: ModelStreamChunk) -> KernelResult<()> {
        if chunk.content.is_empty() {
            return Ok(());
        }
        self.bytes = self.bytes.checked_add(chunk.content.len()).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::resource_exhausted(
                "agent-engine stream output byte count overflow",
            )
        })?;
        if self.bytes > MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES {
            return Err(sdkwork_agent_kernel::KernelError::resource_exhausted(
                "agent-engine stream output exceeds maximum size",
            ));
        }

        // Reasoning and tool-call argument deltas carry into the rich
        // `agent.stream.*` event channel so the visible assistant text
        // (`stream_deltas`) never mixes in hidden reasoning or serialized
        // tool-call JSON. Only visible text chunks feed the product text
        // assembly path.
        match chunk.chunk_kind {
            sdkwork_agent_kernel::ModelChunkKind::Reasoning
            | sdkwork_agent_kernel::ModelChunkKind::ToolCallArguments
            | sdkwork_agent_kernel::ModelChunkKind::Usage
            | sdkwork_agent_kernel::ModelChunkKind::Status => {
                let event =
                    sdkwork_agent_kernel::AgentStreamEvent::from(&chunk).to_kernel_event();
                self.inner.push_event(event.clone())?;
                self.events.push(event);
                Ok(())
            }
            sdkwork_agent_kernel::ModelChunkKind::Text => {
                if self.deltas.len() >= MAX_AGENT_ENGINE_STREAM_CHUNKS {
                    return Err(sdkwork_agent_kernel::KernelError::resource_exhausted(
                        "agent-engine stream chunk limit exceeded",
                    ));
                }
                self.inner.push_chunk(chunk.clone())?;
                self.deltas.push(chunk.content);
                Ok(())
            }
        }
    }

    fn push_event(&mut self, event: KernelEvent) -> KernelResult<()> {
        self.inner.push_event(event.clone())?;
        self.events.push(event);
        Ok(())
    }
}

struct DiscardingModelStreamSink;

impl ModelStreamSink for DiscardingModelStreamSink {
    fn push_chunk(&mut self, _chunk: ModelStreamChunk) -> KernelResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_engines::{bootstrap_agent_engine, canonical_agent_engine_keys};
    use sdkwork_agent_kernel::{
        KernelEventSeverity, ModelDescriptor, ModelProvider, ProviderHealth, ProviderManifest,
    };
    #[cfg(feature = "codex-provider")]
    use sdkwork_agent_provider_codex::CodexSdkIntegration;
    use sdkwork_agent_provider_spi::{
        NegotiatedCapability, SdkBackendKind, SdkBackendRuntime, SdkCapabilityNegotiation,
        SdkDriverHealth, SdkRuntimeBackedModelProvider, SdkRuntimeError, SdkRuntimeOperationKind,
        SdkRuntimeRequest, SdkRuntimeResponse, SdkRuntimeRouter, SDK_CAPABILITY_MODEL_CHAT,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[derive(Default)]
    struct RecordingStreamSink {
        contents: Vec<String>,
    }

    impl ModelStreamSink for RecordingStreamSink {
        fn push_chunk(&mut self, chunk: ModelStreamChunk) -> KernelResult<()> {
            self.contents.push(chunk.content);
            Ok(())
        }
    }

    struct NeverInvokeFallback;

    impl ModelProvider for NeverInvokeFallback {
        fn provider_manifest(&self) -> ProviderManifest {
            ProviderManifest::new(
                "provider.codex.test-fallback",
                "model",
                "Codex test fallback",
                "0.1.0",
                Vec::new(),
            )
        }

        fn health(&self) -> ProviderHealth {
            ProviderHealth::available()
        }

        fn list_models(&self) -> Vec<ModelDescriptor> {
            vec![ModelDescriptor::new(
                "gpt-test",
                "provider.codex.test-fallback",
                "Codex test model",
                "codex",
            )]
        }

        fn invoke(&self, _request: ModelRequest) -> KernelResult<ModelResponse> {
            panic!("the controlled Codex stream test must not invoke a fallback provider")
        }
    }

    struct ControlledCodexStreamingRuntime {
        invoke_count: Arc<AtomicUsize>,
        stream_count: Arc<AtomicUsize>,
    }

    impl SdkBackendRuntime for ControlledCodexStreamingRuntime {
        fn backend_kind(&self) -> SdkBackendKind {
            SdkBackendKind::RustNative
        }

        fn health(&self) -> SdkDriverHealth {
            SdkDriverHealth::healthy()
        }

        fn invoke(
            &self,
            request: &SdkRuntimeRequest,
        ) -> Result<SdkRuntimeResponse, SdkRuntimeError> {
            self.invoke_count.fetch_add(1, Ordering::SeqCst);
            let model_request_id = request
                .operation
                .request_id()
                .expect("model invoke operation has a request id");
            Ok(SdkRuntimeResponse::success(
                SdkBackendKind::RustNative,
                &request.capability_id,
                serde_json::json!({
                    "ok": true,
                    "mode": "sdk_live",
                    "model_request_id": model_request_id,
                    "messages": ["controlled response"],
                    "finish_reason": "stop",
                    "provider_session_id": "thread-controlled",
                }),
            ))
        }

        fn invoke_streaming(
            &self,
            request: &SdkRuntimeRequest,
            sink: &mut dyn FnMut(serde_json::Value) -> Result<bool, SdkRuntimeError>,
        ) -> Result<(), SdkRuntimeError> {
            self.stream_count.fetch_add(1, Ordering::SeqCst);
            let model_request_id = request
                .operation
                .request_id()
                .expect("model streaming operation has a request id");
            if !sink(serde_json::json!({
                "event": "stream.chunk",
                "model_request_id": model_request_id,
                "sequence": 0,
                "content": "streamed response",
            }))? {
                return Ok(());
            }
            sink(serde_json::json!({
                "event": "stream.done",
                "model_request_id": model_request_id,
                "finish_reason": "stop",
                "provider_session_id": "thread-controlled",
            }))?;
            Ok(())
        }
    }

    #[cfg(feature = "codex-provider")]
    fn controlled_codex_slot(
        invoke_count: Arc<AtomicUsize>,
        stream_count: Arc<AtomicUsize>,
    ) -> AgentEngineSlot {
        let mut integration = CodexSdkIntegration::bootstrap().expect("codex bootstrap");
        let negotiation = SdkCapabilityNegotiation {
            agent_id: "agent.facade-test".to_string(),
            binding_id: "binding.facade-test".to_string(),
            binding_version: "0.1.0".to_string(),
            selected: vec![NegotiatedCapability {
                capability_id: SDK_CAPABILITY_MODEL_CHAT.to_string(),
                backend_kind: SdkBackendKind::RustNative,
                driver_id: "driver.facade-test".to_string(),
                runtime_operations: vec![
                    SdkRuntimeOperationKind::ModelChat,
                    SdkRuntimeOperationKind::ModelChatStream,
                ],
            }],
            missing_required: Vec::new(),
            degraded_optional: Vec::new(),
        };
        let runtime = Arc::new(
            SdkRuntimeRouter::new(negotiation).with_rust_runtime(Arc::new(
                ControlledCodexStreamingRuntime {
                    invoke_count,
                    stream_count,
                },
            )),
        );
        integration.model = SdkRuntimeBackedModelProvider::new(
            runtime,
            Arc::new(NeverInvokeFallback),
            SDK_CAPABILITY_MODEL_CHAT,
            "provider.codex",
        );
        AgentEngineSlot::Codex(integration)
    }

    #[test]
    fn forwarding_stream_collector_preserves_chunk_order_for_product_output() {
        let mut sink = RecordingStreamSink::default();
        let mut collector = ForwardingStreamCollector::new(&mut sink);

        collector
            .push_chunk(ModelStreamChunk::output("request-1", 0, "Hello"))
            .expect("first chunk");
        collector
            .push_chunk(ModelStreamChunk::output("request-1", 1, " world"))
            .expect("second chunk");

        let (deltas, events) = collector.into_parts();
        assert_eq!(sink.contents, ["Hello", " world"]);
        assert_eq!(deltas, ["Hello", " world"]);
        assert!(events.is_empty());
    }

    #[test]
    fn forwarding_stream_collector_routes_reasoning_and_tool_chunks_to_events() {
        let mut sink = RecordingStreamSink::default();
        let mut collector = ForwardingStreamCollector::new(&mut sink);

        collector
            .push_chunk(ModelStreamChunk::reasoning("request-1", 0, "think"))
            .expect("reasoning chunk");
        collector
            .push_chunk(ModelStreamChunk::output("request-1", 1, "answer"))
            .expect("answer chunk");
        collector
            .push_chunk(ModelStreamChunk::tool_arguments(
                "request-1", 2, "call.1", "{\"q\":1}",
            ))
            .expect("tool chunk");

        let (deltas, events) = collector.into_parts();
        assert_eq!(sink.contents, ["answer"]);
        assert_eq!(deltas, ["answer"]);
        assert!(
            events.len() >= 2,
            "reasoning and tool deltas must surface as rich events, got {}",
            events.len()
        );
        assert!(
            events.iter().any(|e| e.event_type == "agent.stream.message.delta"),
            "expected a reasoning message delta event"
        );
        assert!(
            events.iter().any(|e| e.event_type == "agent.stream.tool.call.delta"),
            "expected a tool call delta event"
        );
    }

    #[test]
    fn forwarding_stream_collector_discards_empty_transport_chunks() {
        let mut sink = RecordingStreamSink::default();
        let mut collector = ForwardingStreamCollector::new(&mut sink);

        collector
            .push_chunk(ModelStreamChunk::output("request-1", 0, ""))
            .expect("empty chunk is ignored");
        collector
            .push_chunk(ModelStreamChunk::output("request-1", 1, "content"))
            .expect("content chunk");

        let (deltas, events) = collector.into_parts();
        assert_eq!(sink.contents, ["content"]);
        assert_eq!(deltas, ["content"]);
        assert!(events.is_empty());
    }

    #[test]
    fn forwarding_stream_collector_treats_provider_events_as_started_execution() {
        let mut sink = RecordingStreamSink::default();
        let mut collector = ForwardingStreamCollector::new(&mut sink);

        collector
            .push_event(KernelEvent::new(
                "event-1",
                "tool.call.started",
                KernelEventSeverity::Info,
                "{}",
            ))
            .expect("provider event");

        assert!(!collector.is_empty());
        let (deltas, events) = collector.into_parts();
        assert!(deltas.is_empty());
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn verified_runtime_completion_becomes_provider_neutral_turn_completion() {
        let completion = agent_engine_stream_completion(SdkRuntimeStreamCompletion {
            model_request_id: "request-1".to_string(),
            finish_reason: "stop".to_string(),
            provider_session_id: Some("thread-1".to_string()),
        })
        .expect("provider session completion");

        assert_eq!(completion.model_request_id, "request-1");
        assert_eq!(completion.finish_reason, "stop");
        assert_eq!(completion.provider_session_id, "thread-1");
    }

    #[test]
    fn incomplete_runtime_stream_cannot_create_a_first_turn_session() {
        let result = agent_engine_stream_completion(SdkRuntimeStreamCompletion {
            model_request_id: "request-1".to_string(),
            finish_reason: "stop".to_string(),
            provider_session_id: None,
        });

        assert!(matches!(result, Err(RuntimeFacadeError::Kernel(_))));
    }

    #[test]
    fn streamed_turn_output_uses_verified_completion_and_ordered_deltas() {
        let input = AgentEngineTurnInput {
            engine_key: "codex".to_string(),
            prompt: "implement this".to_string(),
            ..Default::default()
        };
        let completion = AgentEngineTurnStreamCompletion {
            model_request_id: "request-1".to_string(),
            finish_reason: "stop".to_string(),
            provider_session_id: "thread-1".to_string(),
        };

        let output = build_streamed_turn_output(
            &input,
            completion.model_request_id.clone(),
            vec!["first ".to_string(), "second".to_string()],
            Vec::new(),
            Some(completion.clone()),
        )
        .expect("streamed output");

        assert_eq!(output.assistant_content, "first second");
        assert_eq!(output.provider_session_id.as_deref(), Some("thread-1"));
        assert_eq!(output.stream_deltas, ["first ", "second"]);
        assert!(output.stream_events.is_empty());
        assert_eq!(output.stream_completion, Some(completion));
    }

    #[test]
    #[cfg(feature = "codex-provider")]
    fn codex_first_turn_completion_binds_provider_session_and_resume_does_not_invoke() {
        let invoke_count = Arc::new(AtomicUsize::new(0));
        let stream_count = Arc::new(AtomicUsize::new(0));
        let slot = controlled_codex_slot(invoke_count.clone(), stream_count.clone());

        let mut first_sink = RecordingStreamSink::default();
        let first = execute_agent_engine_turn_with_stream_sink(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "gpt-test".to_string(),
                prompt: "first turn".to_string(),
                require_live_provider: true,
                ..Default::default()
            },
            &mut first_sink,
        )
        .expect("first streamed turn");
        assert_eq!(first_sink.contents, ["streamed response"]);
        assert_eq!(first.assistant_content, "streamed response");
        assert_eq!(
            first.provider_session_id.as_deref(),
            Some("thread-controlled")
        );
        assert_eq!(
            first
                .stream_completion
                .as_ref()
                .map(|completion| completion.provider_session_id.as_str()),
            Some("thread-controlled")
        );

        let mut resumed_sink = RecordingStreamSink::default();
        let resumed = execute_agent_engine_turn_with_stream_sink(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "gpt-test".to_string(),
                provider_session_id: first.provider_session_id.clone(),
                prompt: "resumed turn".to_string(),
                require_live_provider: true,
                ..Default::default()
            },
            &mut resumed_sink,
        )
        .expect("resumed streamed turn");
        assert_eq!(resumed_sink.contents, ["streamed response"]);
        assert_eq!(
            resumed.provider_session_id.as_deref(),
            Some("thread-controlled")
        );
        assert_eq!(
            resumed
                .stream_completion
                .as_ref()
                .map(|completion| completion.provider_session_id.as_str()),
            Some("thread-controlled")
        );
        assert_eq!(invoke_count.load(Ordering::SeqCst), 0);
        assert_eq!(stream_count.load(Ordering::SeqCst), 2);
    }

    #[cfg(feature = "codex-provider")]
    #[test]
    fn executes_turn_for_canonical_codex_engine() {
        let invoke_count = Arc::new(AtomicUsize::new(0));
        let slot = controlled_codex_slot(invoke_count.clone(), Arc::new(AtomicUsize::new(0)));
        let model_id = slot.list_model_ids().into_iter().next().expect("model id");
        let output = execute_agent_engine_turn(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id,
                prompt: "hello agents facade".to_string(),
                ..Default::default()
            },
        )
        .expect("turn execution");
        assert_eq!(output.assistant_content, "controlled response");
        assert_eq!(output.finish_reason.as_deref(), Some("stop"));
        assert_eq!(
            output.provider_session_id.as_deref(),
            Some("thread-controlled")
        );
        assert_eq!(invoke_count.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "codex-provider")]
    #[test]
    fn all_canonical_engines_execute_turn() {
        // The TypeScript/Python runtimes fall back to the local sdk_probe mock
        // backend when the official SDK package is not installed; that
        // fallback is fail-closed unless the explicit mock override is
        // enabled.
        std::env::set_var("SDKWORK_KERNEL_ENVIRONMENT", "development");
        std::env::set_var("SDKWORK_KERNEL_ALLOW_MOCK_PROVIDERS", "1");

        for engine in canonical_agent_engine_keys() {
            let slot = if *engine == "codex" {
                controlled_codex_slot(Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)))
            } else {
                bootstrap_agent_engine(engine).expect("bootstrap")
            };
            let model_id = slot.list_model_ids().into_iter().next().expect("model id");
            let output = execute_agent_engine_turn(
                &slot,
                &AgentEngineTurnInput {
                    engine_key: (*engine).to_string(),
                    model_id,
                    prompt: format!("ping {engine}"),
                    ..Default::default()
                },
            )
            .unwrap_or_else(|error| panic!("turn failed for {engine}: {error}"));
            assert!(!output.assistant_content.trim().is_empty());
        }
    }

    #[test]
    fn blank_prompt_returns_typed_error() {
        let slot = bootstrap_agent_engine("codex").expect("codex bootstrap");
        let result = execute_agent_engine_turn(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "model".to_string(),
                prompt: "   ".to_string(),
                ..Default::default()
            },
        );
        assert!(matches!(result, Err(RuntimeFacadeError::BlankPrompt)));
    }

    #[test]
    fn engine_mismatch_returns_typed_error() {
        let slot = bootstrap_agent_engine("codex").expect("codex bootstrap");
        let result = execute_agent_engine_turn(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "gemini".to_string(),
                model_id: "model".to_string(),
                prompt: "hello".to_string(),
                ..Default::default()
            },
        );
        assert!(matches!(
            result,
            Err(RuntimeFacadeError::EngineMismatch { .. })
        ));
    }

    #[test]
    fn build_model_request_preserves_session_identities_execution_context_and_budget() {
        let slot = bootstrap_agent_engine("codex").expect("codex bootstrap");
        let request = build_model_request(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "gpt-5-codex".to_string(),
                model_request_id: Some("agents-turn-turn-canonical".to_string()),
                session_id: Some("session-canonical".to_string()),
                turn_id: Some("turn-canonical".to_string()),
                provider_session_id: Some("provider-session-existing".to_string()),
                prompt: "implement the change".to_string(),
                working_directory: Some(PathBuf::from("C:/workspace/project")),
                timeout_ms: Some(90_000),
                approval_policy: Some("on-request".to_string()),
                sandbox_mode: Some("workspace-write".to_string()),
                full_auto: true,
                skip_git_repo_check: true,
                ephemeral: true,
                require_live_provider: true,
                max_output_bytes: Some(65_536),
                temperature: Some(0.2),
                top_p: Some(0.9),
                max_tokens: Some(4_096),
                access_mode_id: None,
                auth_token: Some("caller-auth-token".to_string()),
                access_token: Some("caller-access-token".to_string()),
            },
        )
        .expect("model request");

        assert_eq!(request.model_id.as_deref(), Some("gpt-5-codex"));
        assert_eq!(request.model_request_id, "agents-turn-turn-canonical");
        assert_eq!(request.session_id.as_deref(), Some("session-canonical"));
        assert_eq!(request.step_id.as_deref(), Some("turn-canonical"));
        assert_eq!(
            request.provider_session_id.as_deref(),
            Some("provider-session-existing")
        );
        assert_eq!(request.timeout_ms, Some(90_000));
        assert_eq!(request.auth_token.as_deref(), Some("caller-auth-token"));
        assert_eq!(
            request.access_token.as_deref(),
            Some("caller-access-token")
        );
        assert_eq!(
            request.metadata_value(WORKING_DIRECTORY_METADATA_KEY),
            Some("C:/workspace/project")
        );
        assert_eq!(
            request.metadata_value(APPROVAL_POLICY_METADATA_KEY),
            Some("on-request")
        );
        assert_eq!(
            request.metadata_value(SANDBOX_MODE_METADATA_KEY),
            Some("workspace-write")
        );
        assert_eq!(request.metadata_value(FULL_AUTO_METADATA_KEY), Some("true"));
        assert_eq!(
            request.metadata_value(SKIP_GIT_REPO_CHECK_METADATA_KEY),
            Some("true")
        );
        assert_eq!(request.metadata_value(EPHEMERAL_METADATA_KEY), Some("true"));
        assert_eq!(
            request.metadata_value(REQUIRE_LIVE_PROVIDER_METADATA_KEY),
            Some("true")
        );
        assert_eq!(
            request.metadata_value(MAX_OUTPUT_BYTES_METADATA_KEY),
            Some("65536")
        );
        assert_eq!(
            request.metadata_value(TEMPERATURE_METADATA_KEY),
            Some("0.2")
        );
        assert_eq!(request.metadata_value(TOP_P_METADATA_KEY), Some("0.9"));
        assert_eq!(
            request.metadata_value(MAX_TOKENS_METADATA_KEY),
            Some("4096")
        );
    }

    #[test]
    fn build_model_request_never_substitutes_session_identities() {
        let slot = bootstrap_agent_engine("codex").expect("codex bootstrap");
        let canonical_only = build_model_request(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                session_id: Some("session-canonical".to_string()),
                prompt: "canonical only".to_string(),
                ..Default::default()
            },
        )
        .expect("canonical-only model request");
        assert_eq!(
            canonical_only.session_id.as_deref(),
            Some("session-canonical")
        );
        assert_eq!(canonical_only.provider_session_id, None);

        let provider_only = build_model_request(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                provider_session_id: Some("provider-session".to_string()),
                prompt: "provider only".to_string(),
                ..Default::default()
            },
        )
        .expect("provider-only model request");
        assert_eq!(provider_only.session_id, None);
        assert_eq!(
            provider_only.provider_session_id.as_deref(),
            Some("provider-session")
        );
    }

    #[test]
    fn access_mode_resolution_overrides_legacy_execution_fields() {
        let slot = bootstrap_agent_engine("codex").expect("codex bootstrap");
        let request = build_model_request(
            &slot,
            &AgentEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "gpt-5-codex".to_string(),
                prompt: "implement the change".to_string(),
                approval_policy: Some("never".to_string()),
                sandbox_mode: Some("danger-full-access".to_string()),
                access_mode_id: Some("approve_for_me".to_string()),
                ..Default::default()
            },
        )
        .expect("model request");

        assert_eq!(
            request.metadata_value(APPROVAL_POLICY_METADATA_KEY),
            Some("on-request")
        );
        assert_eq!(
            request.metadata_value(SANDBOX_MODE_METADATA_KEY),
            Some("workspace-write")
        );
        assert_eq!(
            request.metadata_value("sdkwork.agent_engine.approvals_reviewer"),
            Some("auto_review")
        );
    }

    #[test]
    fn provider_session_diagnostic_overrides_input_session() {
        let response = ModelResponse::text("request-1", "provider.codex", "done")
            .with_diagnostic("sdk_runtime_session_id=session-provider");
        let input = AgentEngineTurnInput {
            provider_session_id: Some("session-input".to_string()),
            ..Default::default()
        };

        assert_eq!(
            resolve_provider_session_id(&response, &input).as_deref(),
            Some("session-provider")
        );
    }

    #[test]
    fn provider_session_resolution_falls_back_to_input_session() {
        let response = ModelResponse::text("request-1", "provider.codex", "done")
            .with_diagnostic("sdk_runtime_mode=sdk_live");
        let input = AgentEngineTurnInput {
            provider_session_id: Some("session-input".to_string()),
            ..Default::default()
        };

        assert_eq!(
            resolve_provider_session_id(&response, &input).as_deref(),
            Some("session-input")
        );
    }

    #[test]
    fn turn_output_preserves_kernel_tool_calls() {
        let response = ModelResponse::text("request-1", "provider.codex", "done").with_tool_call(
            ToolCall::new("call-1", "codex.shell", r#"{"command":"cargo test"}"#),
        );
        let output = build_turn_output(response, &AgentEngineTurnInput::default());

        assert_eq!(output.tool_calls.len(), 1);
        assert_eq!(output.tool_calls[0].tool_call_id, "call-1");
        assert_eq!(output.tool_calls[0].tool_id, "codex.shell");
        assert_eq!(output.stream_completion, None);
    }

    #[test]
    fn output_budget_is_bounded_by_the_facade_limit() {
        let input = AgentEngineTurnInput {
            max_output_bytes: Some(MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES + 1),
            ..Default::default()
        };

        assert_eq!(
            effective_max_output_bytes(&input),
            MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES
        );
        assert!(validate_output_size(
            MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES,
            effective_max_output_bytes(&input)
        )
        .is_ok());
        assert!(validate_output_size(
            MAX_AGENT_ENGINE_STREAM_OUTPUT_BYTES + 1,
            effective_max_output_bytes(&input)
        )
        .is_err());
    }
}
