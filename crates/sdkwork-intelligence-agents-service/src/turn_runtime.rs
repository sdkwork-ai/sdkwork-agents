//! Managed-agent inference for durable session turns.
//!
//! Product HTTP APIs call this module to produce assistant replies after a user
//! input item is accepted. Inject a custom [`TurnExecutor`] at service bootstrap
//! for live provider inference; the default [`ContractTurnExecutor`] keeps HTTP
//! contracts stable without a kernel provider registry in-process.

use crate::domain::{AgentSessionItemKind, AgentSessionRecord};
use crate::runtime_facade_bridge::{agent_engine_host_for, engine_key_for_binding_id};
use sdkwork_agent_kernel::{
    KernelError, KernelEvent, KernelResult, ModelProvider, ModelRequest, ModelResponse,
    ModelStatus, ModelStreamChunk, ModelStreamSink,
};
use sdkwork_agents_runtime_facade::{
    agent_engine_model_request_id, execute_agent_engine_turn,
    execute_agent_engine_turn_with_stream, execute_agent_engine_turn_with_stream_sink,
    AgentEngineTurnInput,
};
use sdkwork_utils_rust::string::is_blank;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::Semaphore;

/// Runtime mode when managed turn inference succeeds through the agent-engine facade.
pub const RUNTIME_MODE_FACADE: &str = "agents-runtime-facade";
/// Runtime mode when inference was attempted but failed (no silent contract fallback).
pub const RUNTIME_MODE_INFERENCE_ERROR: &str = "managed-agent-inference-error";
/// Runtime mode when the bounded provider worker capacity is exhausted.
pub const RUNTIME_MODE_CAPACITY_ERROR: &str = "managed-agent-capacity-error";

pub fn is_inference_error(runtime_mode: &str) -> bool {
    runtime_mode == RUNTIME_MODE_INFERENCE_ERROR
}

pub fn is_capacity_error(runtime_mode: &str) -> bool {
    runtime_mode == RUNTIME_MODE_CAPACITY_ERROR
}

const DEFAULT_PROVIDER_WORKER_LIMIT: usize = 32;

static PROVIDER_WORKER_LIMIT: LazyLock<Arc<Semaphore>> = LazyLock::new(|| {
    let configured = std::env::var("SDKWORK_AGENTS_PROVIDER_WORKER_LIMIT")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| (1..=1024).contains(value))
        .unwrap_or(DEFAULT_PROVIDER_WORKER_LIMIT);
    Arc::new(Semaphore::new(configured))
});

/// Input for one durable turn execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnExecutionInput {
    /// Canonical SDKWork Turn identity established before provider execution.
    pub turn_id: String,
    /// Stable provider request identity established from the canonical Turn.
    pub model_request_id: String,
    pub agent_display_name: String,
    pub welcome_message: Option<String>,
    pub session: AgentSessionRecord,
    pub history: Vec<(AgentSessionItemKind, String)>,
    pub user_content: String,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    /// Opaque provider continuation identity. This must never be synthesized
    /// from the canonical SDKWork Session id.
    pub provider_session_id: Option<String>,
    pub access_mode_id: Option<String>,
    /// Active provider binding id (used to resolve canonical agent-engine keys).
    pub binding_id: Option<String>,
    /// When true, an active binding exposes `model.chat` and the gateway may
    /// replace this completer with a kernel-backed implementation.
    pub provider_has_model_chat: bool,
    /// Original user auth token from the authenticated request (transient —
    /// never persisted). When present, the turn may be executed through the
    /// cloudrouter account-pool routing gateway instead of a local engine.
    /// Agent system prompt injected ahead of the turn history.
    pub system_prompt: Option<String>,
    pub auth_token: Option<String>,
    /// Transient user access token for dual-token cloudrouter routing.
    pub access_token: Option<String>,
    /// Optional LLM wire protocol override for the cloudrouter gateway
    /// invocation (`chat_completions` default, `anthropic_messages`,
    /// `google_content`, `openai_responses`). Transient — never persisted.
    pub wire_protocol: Option<String>,
}

/// Output from one durable turn execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnExecutionOutput {
    pub model_request_id: Option<String>,
    pub finish_reason: Option<String>,
    pub content: String,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_session_id: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub runtime_mode: &'static str,
    pub stream_deltas: Vec<String>,
    pub stream_events: Vec<KernelEvent>,
}

/// Provider-neutral input for cancelling one active durable Turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCancellationInput {
    pub turn_id: String,
    pub model_request_id: String,
    pub session_id: String,
    pub binding_id: Option<String>,
    pub provider_has_model_chat: bool,
    /// Scope used to reach the per-agent engine host (never persisted).
    pub tenant_id: u64,
    pub agent_id: String,
}

/// Correlated cancellation acknowledgement returned by the active executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCancellationOutput {
    pub model_request_id: String,
    pub finish_reason: String,
}

pub fn turn_model_request_id(turn_id: &str) -> String {
    agent_engine_model_request_id(turn_id)
}

/// Provider-neutral observer for one Turn's live output.
///
/// Implementations must remain synchronous because provider SDK callbacks are
/// synchronous. A closed consumer must not fail or cancel durable Turn
/// execution; persistence remains authoritative even after an HTTP disconnect.
pub trait TurnExecutionStreamSink: Send + Sync {
    fn begin_turn(&self, _session_id: &str, _turn_id: &str) {}

    fn push_delta(&self, delta: &str);

    fn push_event(&self, event: &KernelEvent) -> KernelResult<()>;

    fn close(&self) {}
}

/// Pluggable turn execution strategy (Open/Closed: swap at service bootstrap).
pub trait TurnExecutor: Send + Sync {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput;

    fn cancel(&self, input: &TurnCancellationInput) -> KernelResult<TurnCancellationOutput> {
        Ok(local_turn_cancellation(input))
    }

    fn complete_with_stream_preference(
        &self,
        input: &TurnExecutionInput,
        prefer_stream: bool,
    ) -> TurnExecutionOutput {
        let _ = prefer_stream;
        self.complete(input)
    }

    fn complete_with_stream_sink(
        &self,
        input: &TurnExecutionInput,
        sink: Arc<dyn TurnExecutionStreamSink>,
    ) -> TurnExecutionOutput {
        let output = self.complete_with_stream_preference(input, true);
        replay_turn_execution_stream(&output, sink.as_ref());
        output
    }
}

/// Default managed-agent contract completer used in tests and local deployments.
/// Maximum wall-clock time for one managed turn execution (30 minutes).
///
/// Coding-style turns legitimately stream for many minutes; the prior
/// 120-second budget truncated them with "turn inference timed out". Stalled
/// providers are still cut quickly by the HTTP client's read timeout and the
/// gateway's per-frame idle timeout, so this ceiling only bounds worker
/// occupancy for otherwise-healthy long turns.
pub const TURN_EXECUTION_TIMEOUT: Duration = Duration::from_secs(1800);

/// Run turn inference with a hard timeout.
///
/// Uses the tokio bounded blocking pool (when a runtime is available) instead
/// of spawning an unbounded OS thread per request. This prevents thread
/// exhaustion and OOM under high concurrency. The completer runs on a separate
/// blocking worker so the timeout can be enforced; on timeout the detached
/// task remains bounded by the pool size and its result is dropped.
///
/// When no tokio runtime is available (pure unit tests without a runtime
/// context), falls back to inline execution without timeout isolation.
pub fn complete_with_timeout(
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    prefer_stream: bool,
    timeout: Duration,
) -> TurnExecutionOutput {
    #[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return run_with_bounded_timeout(&handle, completer, input, prefer_stream, timeout);
        }
        tracing::warn!(
            "no tokio runtime available; running turn inference inline without timeout isolation"
        );
    }
    completer.complete_with_stream_preference(input, prefer_stream)
}

/// Run streaming Turn inference with the same worker bound and timeout as the
/// buffered path while forwarding provider output as it arrives.
pub fn complete_with_timeout_and_sink(
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    sink: Arc<dyn TurnExecutionStreamSink>,
    timeout: Duration,
) -> TurnExecutionOutput {
    #[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return run_with_bounded_timeout_and_sink(&handle, completer, input, sink, timeout);
        }
        tracing::warn!(
            "no tokio runtime available; running streamed turn inference inline without timeout isolation"
        );
    }
    completer.complete_with_stream_sink(input, sink)
}

#[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
fn run_with_bounded_timeout(
    handle: &tokio::runtime::Handle,
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    prefer_stream: bool,
    timeout: Duration,
) -> TurnExecutionOutput {
    let permit = match PROVIDER_WORKER_LIMIT.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            crate::infrastructure::AgentMetricsRegistry::global()
                .record_provider_worker_rejection();
            return capacity_error(input);
        }
    };
    let input_owned = input.clone();
    let worker_completer = Arc::clone(&completer);
    let join_handle = handle.spawn_blocking(move || {
        let _permit = permit;
        worker_completer.complete_with_stream_preference(&input_owned, prefer_stream)
    });
    // Safe to block_on here: complete_with_timeout runs inside a spawn_blocking
    // worker (see http.rs handler dispatch), not on an async executor thread.
    match handle.block_on(tokio::time::timeout(timeout, join_handle)) {
        Ok(Ok(output)) => output,
        Ok(Err(_join_error)) => inference_error("turn inference task failed"),
        Err(_elapsed) => {
            // The detached worker still holds its provider request and its
            // worker permit until the provider returns. Send a correlated
            // cancellation so the provider stops instead of leaking a
            // process; the cancellation itself is bounded so a hung provider
            // cannot block this worker.
            cancel_timed_out_turn(handle, &completer, input);
            inference_error("turn inference timed out")
        }
    }
}

#[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
fn run_with_bounded_timeout_and_sink(
    handle: &tokio::runtime::Handle,
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    sink: Arc<dyn TurnExecutionStreamSink>,
    timeout: Duration,
) -> TurnExecutionOutput {
    let permit = match PROVIDER_WORKER_LIMIT.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            crate::infrastructure::AgentMetricsRegistry::global()
                .record_provider_worker_rejection();
            sink.close();
            return capacity_error(input);
        }
    };
    let input_owned = input.clone();
    let worker_sink = Arc::clone(&sink);
    let worker_completer = Arc::clone(&completer);
    let join_handle = handle.spawn_blocking(move || {
        let _permit = permit;
        worker_completer.complete_with_stream_sink(&input_owned, worker_sink)
    });
    match handle.block_on(tokio::time::timeout(timeout, join_handle)) {
        Ok(Ok(output)) => output,
        Ok(Err(_join_error)) => {
            sink.close();
            inference_error("turn inference task failed")
        }
        Err(_elapsed) => {
            sink.close();
            // See run_with_bounded_timeout: cancel the still-running provider
            // request so it cannot leak a process or hold the worker slot.
            cancel_timed_out_turn(handle, &completer, input);
            inference_error("turn inference timed out")
        }
    }
}

/// Bound for the best-effort cancellation issued after a timed-out turn.
const TURN_CANCEL_TIMEOUT: Duration = Duration::from_secs(5);

/// Sends a correlated cancellation for a turn whose wall-clock budget
/// elapsed while the provider was still executing. Runs on a bounded
/// blocking worker so a hung provider cannot block the current worker;
/// the cancellation outcome is logged, never surfaced.
#[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
fn cancel_timed_out_turn(
    handle: &tokio::runtime::Handle,
    completer: &Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
) {
    let cancel_input = TurnCancellationInput {
        turn_id: input.turn_id.clone(),
        model_request_id: input.model_request_id.clone(),
        session_id: input.session.session_id.clone(),
        binding_id: input.binding_id.clone(),
        provider_has_model_chat: input.provider_has_model_chat,
        tenant_id: input.session.tenant_id,
        agent_id: input.session.agent_id.clone(),
    };
    let completer = Arc::clone(completer);
    let join_handle = handle.spawn_blocking(move || completer.cancel(&cancel_input));
    match handle.block_on(tokio::time::timeout(TURN_CANCEL_TIMEOUT, join_handle)) {
        Ok(Ok(Ok(_output))) => {
            tracing::warn!(
                turn_id = %input.turn_id,
                "timed-out turn cancelled at the provider"
            );
        }
        Ok(Ok(Err(error))) => {
            tracing::warn!(
                turn_id = %input.turn_id,
                error = %error,
                "timed-out turn cancellation was rejected by the provider"
            );
        }
        Ok(Err(_join_error)) => {
            tracing::warn!(
                turn_id = %input.turn_id,
                "timed-out turn cancellation worker failed"
            );
        }
        Err(_elapsed) => {
            tracing::warn!(
                turn_id = %input.turn_id,
                "timed-out turn cancellation did not complete within the bound"
            );
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContractTurnExecutor;

impl TurnExecutor for ContractTurnExecutor {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        execute_agent_turn(input)
    }
}

/// Kernel-backed turn executor for production gateway bootstrap.
///
/// Invokes a mounted [`ModelProvider`] when `provider_has_model_chat` is true;
/// otherwise falls back to [`ContractTurnExecutor`] semantics.
pub struct KernelModelTurnExecutor<P> {
    provider: Arc<P>,
    fallback: ContractTurnExecutor,
}

impl<P> KernelModelTurnExecutor<P>
where
    P: ModelProvider + Send + Sync + 'static,
{
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            fallback: ContractTurnExecutor,
        }
    }
}

impl<P> TurnExecutor for KernelModelTurnExecutor<P>
where
    P: ModelProvider + Send + Sync + 'static,
{
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        if !input.provider_has_model_chat {
            return self.fallback.complete(input);
        }
        match invoke_kernel_model(self.provider.as_ref(), input) {
            Ok(output) => output,
            Err(error) => {
                tracing::warn!(
                    session_id = %input.session.session_id,
                    error = %error,
                    "kernel model invoke failed"
                );
                inference_error(format!("model inference failed: {error}"))
            }
        }
    }

    fn cancel(&self, input: &TurnCancellationInput) -> KernelResult<TurnCancellationOutput> {
        if !input.provider_has_model_chat {
            return Ok(local_turn_cancellation(input));
        }
        let response = self.provider.cancel(&input.model_request_id)?;
        cancellation_from_model_response(input, response)
    }
}

/// Production chat completer: routes active provider bindings through the
/// agents runtime facade (canonical agent engines). Never silently echoes user
/// input when `model.chat` is configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeFacadeTurnExecutor;

impl TurnExecutor for RuntimeFacadeTurnExecutor {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        execute_runtime_facade_turn(input, false, None)
    }

    fn complete_with_stream_preference(
        &self,
        input: &TurnExecutionInput,
        prefer_stream: bool,
    ) -> TurnExecutionOutput {
        execute_runtime_facade_turn(input, prefer_stream, None)
    }

    fn complete_with_stream_sink(
        &self,
        input: &TurnExecutionInput,
        sink: Arc<dyn TurnExecutionStreamSink>,
    ) -> TurnExecutionOutput {
        execute_runtime_facade_turn(input, true, Some(sink))
    }

    fn cancel(&self, input: &TurnCancellationInput) -> KernelResult<TurnCancellationOutput> {
        if !input.provider_has_model_chat {
            return Ok(local_turn_cancellation(input));
        }
        let binding_id = input
            .binding_id
            .as_deref()
            .filter(|value| !is_blank(Some(*value)))
            .ok_or_else(|| KernelError::validation("active provider binding id is required"))?;
        let engine_key = engine_key_for_binding_id(binding_id).ok_or_else(|| {
            KernelError::validation(
                "active provider binding is not mapped to a canonical agent engine",
            )
        })?;
        let host = agent_engine_host_for(input.tenant_id, &input.agent_id).ok_or_else(|| {
            KernelError::provider_error(
                "turn_cancellation_unavailable",
                "agent engine host is unavailable",
            )
        })?;
        let cancellation = host
            .cancel_turn(engine_key, &input.model_request_id)
            .map_err(|error| {
                KernelError::provider_error("turn_cancellation_failed", error.to_string())
            })?;
        Ok(TurnCancellationOutput {
            model_request_id: cancellation.model_request_id,
            finish_reason: cancellation.finish_reason,
        })
    }
}

fn local_turn_cancellation(input: &TurnCancellationInput) -> TurnCancellationOutput {
    TurnCancellationOutput {
        model_request_id: input.model_request_id.clone(),
        finish_reason: "cancelled".to_string(),
    }
}

fn cancellation_from_model_response(
    input: &TurnCancellationInput,
    response: ModelResponse,
) -> KernelResult<TurnCancellationOutput> {
    if response.model_request_id != input.model_request_id
        || response.status != ModelStatus::Cancelled
        || response.finish_reason.as_deref() != Some("cancelled")
    {
        return Err(KernelError::provider_error(
            "turn_cancellation_unconfirmed",
            "model provider did not return a correlated cancelled acknowledgement",
        ));
    }
    Ok(TurnCancellationOutput {
        model_request_id: response.model_request_id,
        finish_reason: "cancelled".to_string(),
    })
}

struct RuntimeFacadeModelStreamSink {
    sink: Arc<dyn TurnExecutionStreamSink>,
}

impl ModelStreamSink for RuntimeFacadeModelStreamSink {
    fn push_chunk(&mut self, chunk: ModelStreamChunk) -> KernelResult<()> {
        self.sink.push_delta(&chunk.content);
        Ok(())
    }

    fn push_event(&mut self, event: KernelEvent) -> KernelResult<()> {
        self.sink.push_event(&event)
    }
}

fn execute_runtime_facade_turn(
    input: &TurnExecutionInput,
    prefer_stream: bool,
    sink: Option<Arc<dyn TurnExecutionStreamSink>>,
) -> TurnExecutionOutput {
    if !input.provider_has_model_chat {
        return execute_agent_turn(input);
    }

    let binding_id = input
        .binding_id
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
        .unwrap_or("");
    let Some(engine_key) = engine_key_for_binding_id(binding_id) else {
        return inference_error(
            "active provider binding is not mapped to a canonical agent engine",
        );
    };

    let Some(host) = agent_engine_host_for(input.session.tenant_id, &input.session.agent_id) else {
        return inference_error("agent engine host is unavailable");
    };
    let Some(slot) = host.slot(engine_key) else {
        return inference_error(format!("agent engine bootstrap failed for {engine_key}"));
    };

    let model_id = resolve_turn_model_id(input, slot);
    let prompt = build_managed_chat_prompt(input);
    let turn_input = AgentEngineTurnInput {
        engine_key: engine_key.to_string(),
        model_id: model_id.clone(),
        model_request_id: Some(input.model_request_id.clone()),
        session_id: Some(input.session.session_id.clone()),
        turn_id: Some(input.turn_id.clone()),
        provider_session_id: input.provider_session_id.clone(),
        prompt,
        // Propagate the wall-clock execution budget to the provider so it can
        // stop itself instead of relying solely on the server-side join
        // timeout (which cannot terminate a hung provider process).
        timeout_ms: Some(TURN_EXECUTION_TIMEOUT.as_millis() as u64),
        access_mode_id: input.access_mode_id.clone(),
        require_live_provider: true,
        // Propagate the caller dual tokens so the provider can route the model
        // call through the cloud router account pool (never persisted).
        auth_token: input.auth_token.clone(),
        access_token: input.access_token.clone(),
        ..Default::default()
    };

    let turn_result = if prefer_stream {
        if let Some(sink) = sink {
            let mut facade_sink = RuntimeFacadeModelStreamSink { sink };
            execute_agent_engine_turn_with_stream_sink(slot, &turn_input, &mut facade_sink)
        } else {
            execute_agent_engine_turn_with_stream(slot, &turn_input)
        }
    } else {
        execute_agent_engine_turn(slot, &turn_input)
    };

    match turn_result {
        Ok(output) => {
            let content = output.assistant_content.trim().to_string();
            let cancelled = output.finish_reason.as_deref() == Some("cancelled");
            if content.is_empty() && !cancelled {
                return inference_error("agent engine returned empty assistant content");
            }
            TurnExecutionOutput {
                model_request_id: Some(output.model_request_id),
                finish_reason: output.finish_reason,
                content,
                model_id: Some(model_id),
                provider_id: input.provider_id.clone(),
                provider_session_id: output.provider_session_id,
                input_tokens: estimate_tokens(input.user_content.as_str()),
                output_tokens: estimate_tokens(output.assistant_content.as_str()),
                runtime_mode: RUNTIME_MODE_FACADE,
                stream_deltas: output.stream_deltas,
                stream_events: output.stream_events,
            }
        }
        Err(error) => inference_error(format!("agent engine turn failed: {error}")),
    }
}

fn resolve_turn_model_id(
    input: &TurnExecutionInput,
    slot: &sdkwork_agents_runtime_facade::AgentEngineSlot,
) -> String {
    if let Some(model_id) = input
        .model_id
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
    {
        return model_id.to_string();
    }
    slot.list_model_ids()
        .into_iter()
        .next()
        .unwrap_or_else(|| "default".to_string())
}

/// Builds the managed-chat prompt for agent-engine turns.
///
/// The full transcript is encoded with the same role-prefix convention as
/// [`build_model_items`] (system prompt, welcome message, history, then the
/// current user content) so simple-agent providers receive the complete
/// context instead of only the latest user message.
fn build_managed_chat_prompt(input: &TurnExecutionInput) -> String {
    build_model_items(input).join("\n")
}

fn replay_turn_execution_stream(output: &TurnExecutionOutput, sink: &dyn TurnExecutionStreamSink) {
    let mut delta_index = 0usize;
    let mut agent_message_text_by_item = HashMap::new();
    for event in &output.stream_events {
        let _ = sink.push_event(event);
        if let Some(expected_delta) =
            buffered_agent_message_delta(event, &mut agent_message_text_by_item)
        {
            if let Some(delta) = output
                .stream_deltas
                .get(delta_index)
                .filter(|delta| delta.as_str() == expected_delta)
            {
                sink.push_delta(delta);
                delta_index += 1;
            }
        }
    }
    for delta in output.stream_deltas.iter().skip(delta_index) {
        sink.push_delta(delta);
    }
}

fn buffered_agent_message_delta(
    event: &KernelEvent,
    text_by_item: &mut HashMap<String, String>,
) -> Option<String> {
    if !matches!(
        event.event_type.as_str(),
        "agent.message.started" | "agent.message.updated" | "agent.message.completed"
    ) {
        return None;
    }
    let payload = serde_json::from_str::<serde_json::Value>(&event.payload).ok()?;
    let item = payload.get("item")?;
    if item.get("type")?.as_str()? != "agent_message" {
        return None;
    }
    let item_id = item.get("id")?.as_str()?.to_string();
    let current = item.get("text")?.as_str()?.to_string();
    let previous = text_by_item
        .insert(item_id, current.clone())
        .unwrap_or_default();
    if event.event_type == "agent.message.started"
        || !current.starts_with(&previous)
        || current.len() == previous.len()
    {
        return None;
    }
    Some(current[previous.len()..].to_string())
}

pub(crate) fn inference_error(message: impl Into<String>) -> TurnExecutionOutput {
    TurnExecutionOutput {
        model_request_id: None,
        finish_reason: None,
        content: message.into(),
        model_id: None,
        provider_id: None,
        provider_session_id: None,
        input_tokens: 0,
        output_tokens: 0,
        runtime_mode: RUNTIME_MODE_INFERENCE_ERROR,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    }
}

fn capacity_error(input: &TurnExecutionInput) -> TurnExecutionOutput {
    TurnExecutionOutput {
        model_request_id: Some(input.model_request_id.clone()),
        finish_reason: None,
        content: "provider concurrency limit reached".to_string(),
        model_id: input.model_id.clone(),
        provider_id: input.provider_id.clone(),
        provider_session_id: input.provider_session_id.clone(),
        input_tokens: 0,
        output_tokens: 0,
        runtime_mode: RUNTIME_MODE_CAPACITY_ERROR,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    }
}

fn build_model_items(input: &TurnExecutionInput) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(system_prompt) = input
        .system_prompt
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
    {
        items.push(format!("system: {system_prompt}"));
    }
    if let Some(welcome) = input
        .welcome_message
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
    {
        items.push(format!("system: {welcome}"));
    }
    for (role, content) in &input.history {
        items.push(format!("{}: {content}", role.as_str()));
    }
    items.push(format!("user: {}", input.user_content));
    items
}

fn invoke_kernel_model(
    provider: &dyn ModelProvider,
    input: &TurnExecutionInput,
) -> KernelResult<TurnExecutionOutput> {
    let model_request_id = input.model_request_id.clone();
    let model_id = input.model_id.clone();
    let mut request = ModelRequest::new(model_request_id.clone(), build_model_items(input))
        .for_session(input.session.session_id.clone())
        .for_step(input.turn_id.clone());
    if let Some(provider_session_id) = input.provider_session_id.as_ref() {
        request = request.for_provider_session(provider_session_id.clone());
    }
    if let Some(model_id) = model_id.clone() {
        request = request.with_model_id(model_id);
    }
    if input.auth_token.is_some() || input.access_token.is_some() {
        request = request.for_caller(input.auth_token.clone(), input.access_token.clone());
    }

    let response = provider.invoke(request)?;
    map_model_response(input, model_id, response)
}

fn map_model_response(
    input: &TurnExecutionInput,
    model_id: Option<String>,
    response: ModelResponse,
) -> KernelResult<TurnExecutionOutput> {
    if response.status == ModelStatus::Cancelled {
        return Ok(TurnExecutionOutput {
            model_request_id: Some(response.model_request_id),
            finish_reason: response.finish_reason,
            content: String::new(),
            model_id,
            provider_id: Some(response.provider_id),
            provider_session_id: input.provider_session_id.clone(),
            input_tokens: 0,
            output_tokens: 0,
            runtime_mode: "managed-agent-kernel-model-v1",
            stream_deltas: Vec::new(),
            stream_events: Vec::new(),
        });
    }
    if response.status != ModelStatus::Succeeded {
        return Err(sdkwork_agent_kernel::KernelError::validation(format!(
            "model invoke returned status {:?}",
            response.status
        )));
    }
    let content = response
        .messages
        .into_iter()
        .next()
        .filter(|message| !is_blank(Some(message.as_str())))
        .unwrap_or_else(|| self_fallback_content(input));
    let (input_tokens, output_tokens) = response
        .usage
        .map(|usage| (usage.input_tokens as u64, usage.output_tokens as u64))
        .unwrap_or_else(|| {
            (
                estimate_tokens(input.user_content.as_str()),
                estimate_tokens(content.as_str()),
            )
        });
    Ok(TurnExecutionOutput {
        model_request_id: Some(response.model_request_id),
        finish_reason: response.finish_reason,
        content,
        model_id,
        provider_id: Some(response.provider_id),
        provider_session_id: input.provider_session_id.clone(),
        input_tokens,
        output_tokens,
        runtime_mode: "managed-agent-kernel-model-v1",
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    })
}

fn self_fallback_content(input: &TurnExecutionInput) -> String {
    format!(
        "Hello! I'm {}. I received your message:\n\n> {}",
        input.agent_display_name, input.user_content
    )
}

fn estimate_tokens(text: &str) -> u64 {
    (text.chars().count() as u64).div_ceil(4)
}

fn build_transcript(history: &[(AgentSessionItemKind, String)]) -> String {
    let mut transcript = String::new();
    for (role, message) in history {
        transcript.push_str(role.as_str());
        transcript.push_str(": ");
        transcript.push_str(message);
        transcript.push('\n');
    }
    transcript
}

fn runtime_mode_for(input: &TurnExecutionInput) -> &'static str {
    if input.provider_has_model_chat {
        "managed-agent-provider-bound-v1"
    } else {
        "managed-agent-contract-v1"
    }
}

/// Complete one user turn using managed-agent contract semantics.
pub fn execute_agent_turn(input: &TurnExecutionInput) -> TurnExecutionOutput {
    let input_tokens = estimate_tokens(input.user_content.as_str());
    let model_id = input.model_id.clone();
    let provider_id = input.provider_id.clone();
    let runtime_mode = runtime_mode_for(input);

    let content = if input.history.is_empty() {
        if let Some(welcome) = input
            .welcome_message
            .as_deref()
            .filter(|value| !is_blank(Some(*value)))
        {
            format!(
                "{welcome}\n\nHow can I help you today?\n\n> {}",
                input.user_content
            )
        } else if input.provider_has_model_chat {
            format!(
                "Hello! I'm {}. I received your message:\n\n> {}\n\nLive model inference requires a canonical agent-engine provider binding.",
                input.agent_display_name, input.user_content
            )
        } else {
            format!(
                "Hello! I'm {}. I received your message:\n\n> {}\n\nActivate a provider binding with model.chat for live model inference.",
                input.agent_display_name, input.user_content
            )
        }
    } else {
        let transcript = build_transcript(&input.history);
        format!(
            "{transcript}user: {}\n\nassistant: Thanks for the context. Based on our conversation, here is my response to your latest message:\n\n> {}",
            input.user_content, input.user_content
        )
    };

    let output_tokens = estimate_tokens(content.as_str());
    TurnExecutionOutput {
        model_request_id: Some(input.model_request_id.clone()),
        finish_reason: Some("stop".to_string()),
        content,
        model_id,
        provider_id,
        provider_session_id: input.provider_session_id.clone(),
        input_tokens,
        output_tokens,
        runtime_mode,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AgentSessionEntrySurface, AgentSessionKind, AgentSessionRecord, AgentSessionStatus,
        AgentSessionTitleSource,
    };

    fn sample_session() -> AgentSessionRecord {
        AgentSessionRecord {
            id: 1,
            session_id: "session.test".to_string(),
            tenant_id: 100001,
            organization_id: 0,
            agent_id: "agent.test".to_string(),
            owner_user_id: 42,
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Test".to_string()),
            title_source: AgentSessionTitleSource::System,
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: None,
            payload_hash: None,
            created_by: 42,
            updated_by: 42,
            version: 0,
            created_at: "2026-06-28T00:00:00Z".to_string(),
            updated_at: "2026-06-28T00:00:00Z".to_string(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        }
    }

    #[test]
    fn execute_agent_turn_returns_assistant_content() {
        let output = execute_agent_turn(&TurnExecutionInput {
            turn_id: "turn.test".to_string(),
            model_request_id: turn_model_request_id("turn.test"),
            agent_display_name: "Demo Agent".to_string(),
            welcome_message: Some("Welcome".to_string()),
            session: sample_session(),
            history: Vec::new(),
            user_content: "Hello".to_string(),
            model_id: None,
            provider_id: None,
            provider_session_id: None,
            binding_id: None,
            access_mode_id: None,
            provider_has_model_chat: false,
            system_prompt: None,
            auth_token: None,
            access_token: None,
            wire_protocol: None,
        });
        assert!(output.content.contains("Hello"));
        assert!(output.content.contains("Welcome"));
        assert_eq!(output.runtime_mode, "managed-agent-contract-v1");
        assert!(output.output_tokens > 0);
    }

    #[test]
    fn execute_agent_turn_marks_provider_bound_runtime_mode() {
        let output = execute_agent_turn(&TurnExecutionInput {
            turn_id: "turn.test".to_string(),
            model_request_id: turn_model_request_id("turn.test"),
            agent_display_name: "Demo Agent".to_string(),
            welcome_message: None,
            session: sample_session(),
            history: Vec::new(),
            user_content: "Hello".to_string(),
            model_id: None,
            provider_id: Some("provider.rig".to_string()),
            provider_session_id: None,
            binding_id: None,
            access_mode_id: None,
            provider_has_model_chat: true,
            system_prompt: None,
            auth_token: None,
            access_token: None,
            wire_protocol: None,
        });
        assert_eq!(output.runtime_mode, "managed-agent-provider-bound-v1");
        assert!(output.content.contains("canonical agent-engine"));
    }

    #[derive(Default)]
    struct FakeKernelModelProvider {
        cancellation_model_request_id: Option<String>,
    }

    impl ModelProvider for FakeKernelModelProvider {
        fn provider_manifest(&self) -> sdkwork_agent_kernel::ProviderManifest {
            sdkwork_agent_kernel::ProviderManifest::new(
                "provider.fake",
                "model",
                "sdkwork-fake-model",
                "0.1.0",
                vec!["model.chat".to_string()],
            )
        }

        fn health(&self) -> sdkwork_agent_kernel::ProviderHealth {
            sdkwork_agent_kernel::ProviderHealth::available()
        }

        fn invoke(&self, request: ModelRequest) -> KernelResult<ModelResponse> {
            assert_eq!(request.session_id.as_deref(), Some("session.test"));
            assert_eq!(request.step_id.as_deref(), Some("turn.test"));
            assert_eq!(
                request.provider_session_id.as_deref(),
                Some("provider-session.fake")
            );
            Ok(
                ModelResponse::text(request.model_request_id, "provider.fake", "kernel reply")
                    .with_usage(sdkwork_agent_kernel::ModelUsage::new(3, 5)),
            )
        }

        fn cancel(&self, model_request_id: &str) -> KernelResult<ModelResponse> {
            Ok(ModelResponse::cancelled(
                self.cancellation_model_request_id
                    .as_deref()
                    .unwrap_or(model_request_id),
                "provider.fake",
            ))
        }
    }

    #[test]
    fn kernel_model_turn_executor_preserves_session_identities() {
        let completer = KernelModelTurnExecutor::new(Arc::new(FakeKernelModelProvider::default()));
        let output = completer.complete(&TurnExecutionInput {
            turn_id: "turn.test".to_string(),
            model_request_id: turn_model_request_id("turn.test"),
            agent_display_name: "Demo Agent".to_string(),
            welcome_message: None,
            session: sample_session(),
            history: Vec::new(),
            user_content: "Hello".to_string(),
            model_id: Some("model.fake".to_string()),
            provider_id: Some("provider.fake".to_string()),
            provider_session_id: Some("provider-session.fake".to_string()),
            binding_id: None,
            access_mode_id: None,
            provider_has_model_chat: true,
            system_prompt: None,
            auth_token: None,
            access_token: None,
            wire_protocol: None,
        });
        assert_eq!(output.content, "kernel reply");
        assert_eq!(output.runtime_mode, "managed-agent-kernel-model-v1");
        assert_eq!(output.provider_id.as_deref(), Some("provider.fake"));
        assert_eq!(
            output.provider_session_id.as_deref(),
            Some("provider-session.fake")
        );
    }

    #[test]
    fn kernel_model_turn_executor_requires_correlated_cancellation_acknowledgement() {
        let model_request_id = turn_model_request_id("turn.test");
        let input = TurnCancellationInput {
            turn_id: "turn.test".to_string(),
            model_request_id: model_request_id.clone(),
            session_id: "session.test".to_string(),
            binding_id: None,
            provider_has_model_chat: true,
            tenant_id: 1,
            agent_id: "agent.test".to_string(),
        };
        let correlated = KernelModelTurnExecutor::new(Arc::new(FakeKernelModelProvider::default()))
            .cancel(&input)
            .expect("correlated cancellation acknowledgement");
        assert_eq!(correlated.model_request_id, model_request_id);
        assert_eq!(correlated.finish_reason, "cancelled");

        let uncorrelated = KernelModelTurnExecutor::new(Arc::new(FakeKernelModelProvider {
            cancellation_model_request_id: Some("agents-turn-wrong-turn".to_string()),
        }))
        .cancel(&input)
        .expect_err("uncorrelated cancellation acknowledgement must fail closed");
        assert_eq!(
            uncorrelated.kind(),
            sdkwork_agent_kernel::KernelErrorKind::ProviderError
        );
    }

    /// Executor whose `complete` blocks like a hung provider and whose
    /// `cancel` records the correlated call.
    struct RecordingCancelExecutor {
        cancels: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        cancel_block_ms: u64,
    }

    impl TurnExecutor for RecordingCancelExecutor {
        fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
            // Simulate a provider that never returns on its own; the
            // wall-clock timeout must terminate the client wait.
            std::thread::sleep(std::time::Duration::from_secs(5));
            inference_error("hung provider must not complete")
        }

        fn complete_with_stream_preference(
            &self,
            input: &TurnExecutionInput,
            _prefer_stream: bool,
        ) -> TurnExecutionOutput {
            self.complete(input)
        }

        fn complete_with_stream_sink(
            &self,
            input: &TurnExecutionInput,
            _sink: Arc<dyn TurnExecutionStreamSink>,
        ) -> TurnExecutionOutput {
            self.complete(input)
        }

        fn cancel(&self, input: &TurnCancellationInput) -> KernelResult<TurnCancellationOutput> {
            if self.cancel_block_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(self.cancel_block_ms));
            }
            self.cancels
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(TurnCancellationOutput {
                model_request_id: input.model_request_id.clone(),
                finish_reason: "cancelled".to_string(),
            })
        }
    }

    fn sample_turn_execution_input() -> TurnExecutionInput {
        TurnExecutionInput {
            turn_id: "turn.timeout-test".to_string(),
            model_request_id: turn_model_request_id("turn.timeout-test"),
            agent_display_name: "Timeout Test".to_string(),
            welcome_message: None,
            session: sample_session(),
            history: Vec::new(),
            user_content: "run".to_string(),
            model_id: None,
            provider_id: None,
            provider_session_id: None,
            binding_id: Some("binding.codex".to_string()),
            access_mode_id: None,
            provider_has_model_chat: true,
            system_prompt: None,
            auth_token: None,
            access_token: None,
            wire_protocol: None,
        }
    }

    #[test]
    fn timed_out_turn_sends_correlated_cancellation_to_the_provider() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("test runtime");
        let cancels = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let completer = Arc::new(RecordingCancelExecutor {
            cancels: Arc::clone(&cancels),
            cancel_block_ms: 0,
        });
        let input = sample_turn_execution_input();

        let output = runtime.block_on(async {
            let handle = tokio::runtime::Handle::current();
            handle
                .spawn_blocking(move || {
                    complete_with_timeout(
                        completer,
                        &input,
                        false,
                        std::time::Duration::from_millis(150),
                    )
                })
                .await
                .expect("timeout worker")
        });

        // The client wait terminated with a timeout error, not the provider's
        // (never-returning) completion.
        assert_eq!(output.runtime_mode, RUNTIME_MODE_INFERENCE_ERROR);
        assert!(
            output.content.contains("timed out"),
            "timed-out turn must report the timeout, got: {:?}",
            output.content
        );
        assert!(
            output
                .finish_reason
                .as_deref()
                .is_none_or(|reason| reason != "cancelled"),
            "timed-out turn is not a cancellation acknowledgement"
        );

        // The correlated cancellation was issued to the provider so it stops
        // instead of leaking a process. Give the bounded cancel worker time to
        // land before asserting.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while cancels.load(std::sync::atomic::Ordering::SeqCst) == 0
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(
            cancels.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "timed-out turn must be cancelled at the provider"
        );
    }

    #[test]
    fn timed_out_turn_cancellation_is_bounded_when_provider_cancel_hangs() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("test runtime");
        let cancels = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        // The provider cancel itself blocks longer than the cancel bound; the
        // timed-out turn path must still return without waiting for it.
        let completer = Arc::new(RecordingCancelExecutor {
            cancels: Arc::clone(&cancels),
            cancel_block_ms: 20_000,
        });
        let input = sample_turn_execution_input();

        let started = std::time::Instant::now();
        let output = runtime.block_on(async {
            let handle = tokio::runtime::Handle::current();
            handle
                .spawn_blocking(move || {
                    complete_with_timeout(
                        completer,
                        &input,
                        false,
                        std::time::Duration::from_millis(150),
                    )
                })
                .await
                .expect("timeout worker")
        });
        let elapsed = started.elapsed();

        assert!(
            elapsed < std::time::Duration::from_secs(12),
            "timed-out turn path must not wait for a hung provider cancel (took {elapsed:?})"
        );
        assert_eq!(output.runtime_mode, RUNTIME_MODE_INFERENCE_ERROR);
        assert!(output.content.contains("timed out"));
    }
}
