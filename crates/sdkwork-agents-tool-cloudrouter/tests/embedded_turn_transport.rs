//! End-to-end regression for the embedded (standalone) turn transport.
//!
//! The `50301` incident happened because the turn path had exactly one
//! transport arm — HTTP — while a standalone profile deliberately resolves *no*
//! HTTP base URL (`APPLICATION_GATEWAY_SPEC.md` §2.3). Every other test around
//! `turn_transport` asserts the *selection* logic; this file asserts the thing
//! that actually broke: that one turn **completes** over the in-process arm and
//! that the surface sees the request CloudRouter requires.
//!
//! Nothing here is a stub of the transport. The dispatcher is a real
//! `axum::Router` driven through `tower::ServiceExt::oneshot`, exactly as the
//! gateway composition root wires it, and the responses are real SSE streams
//! parsed by the same accumulator the HTTP arm uses.
//!
//! **Why this is one `#[test]` and not nine.** The surface is installed into a
//! process-global `OnceLock` — one process has one CloudRouter surface and one
//! composition root, which is the production contract. Cargo runs tests in a
//! single process with multiple threads, so several `#[test]` functions would
//! race to install and the losers would silently exercise the winner's surface
//! (this actually happened: the first draft ran `1 passed; 4 failed` with the
//! surface installed by whichever test won the race). Beyond the surface slot,
//! the profile and the API key are also process-global environment inputs. So
//! the assertions run as ordered steps of one test under an env lock, which is
//! also what makes them order-independent.

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use axum::body::Body;
use axum::http::{HeaderMap, Response, StatusCode};
use axum::routing::post;
use axum::Router;
use sdkwork_agents_tool_cloudrouter::{
    install_embedded_cloudrouter_surface, CloudRouterTurnRequest, CloudRouterTurnTransport,
    EmbeddedSurfaceDispatcher, WireProtocol,
};
use tower::ServiceExt;

/// SSE body shaped like CloudRouter's OpenAI-compatible stream: two content
/// deltas followed by the `[DONE]` sentinel.
const CHAT_STREAM_BODY: &str = concat!(
    "data: {\"id\":\"chatcmpl-e2e\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o-mini\",",
    "\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"po\"},\"finish_reason\":null}]}\n\n",
    "data: {\"id\":\"chatcmpl-e2e\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o-mini\",",
    "\"choices\":[{\"index\":0,\"delta\":{\"content\":\"ng\"},\"finish_reason\":\"stop\"}]}\n\n",
    "data: [DONE]\n\n",
);

/// The same turn in `openai_responses` framing (no `[DONE]` sentinel; the
/// parser keys off event `type`).
const RESPONSES_STREAM_BODY: &str = concat!(
    "data: {\"type\":\"response.output_text.delta\",\"delta\":\"po\"}\n\n",
    "data: {\"type\":\"response.output_text.delta\",\"delta\":\"ng\"}\n\n",
    "data: {\"type\":\"response.completed\",\"response\":{\"model\":\"gpt-4o-mini\"}}\n\n",
);

/// A Google `streamGenerateContent` chunk carrying a thought part and an
/// answer part, plus a terminal `finishReason`.
const GOOGLE_STREAM_BODY: &str = concat!(
    "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"reason step\",\"thought\":true}]},\"index\":0}],\"modelVersion\":\"gemini-2.5\"}\n\n",
    "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"answer\"}]},\"finishReason\":\"STOP\"}]}\n\n",
);

/// Anthropic Messages framing: `message_start` carries the model,
/// `content_block_delta` carries the text, `message_delta` carries the stop
/// reason.
const ANTHROPIC_STREAM_BODY: &str = concat!(
    "data: {\"type\":\"message_start\",\"message\":{\"model\":\"claude-sonnet-4\",\"role\":\"assistant\"}}\n\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"po\"}}\n\n",
    "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"ng\"}}\n\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n",
);

/// OpenAI 工具调用流的真实形状：id/name 只到一次，arguments 分片到达。
/// 这一形状正是 HTTP arm 的 `stream_chat_completion_with_tools_blocking`
/// 存在的理由，in-process arm 必须归一出完全一致的结果。
const TOOL_CALL_STREAM_BODY: &str = concat!(
    "data: {\"id\":\"chatcmpl-tool\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o-mini\",",
    "\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"tool_calls\":[",
    "{\"index\":0,\"id\":\"call_abc123\",\"type\":\"function\",\"function\":{\"name\":\"read_file\",\"arguments\":\"\"}}]},\"finish_reason\":null}]}\n\n",
    "data: {\"id\":\"chatcmpl-tool\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o-mini\",",
    "\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[",
    "{\"index\":0,\"function\":{\"arguments\":\"{\\\"path\\\":\"}}]},\"finish_reason\":null}]}\n\n",
    "data: {\"id\":\"chatcmpl-tool\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o-mini\",",
    "\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[",
    "{\"index\":0,\"function\":{\"arguments\":\"\\\"a.txt\\\"}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n\n",
    "data: [DONE]\n\n",
);

/// What the recording surface does with an inbound request. The endpoint
/// selects the protocol-shaped body, so a turn dispatched at the wrong
/// endpoint fails loudly instead of receiving a silently plausible stream.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SurfaceMode {
    /// Answer every request with the endpoint-appropriate body.
    Serve,
    /// Answer every request with `500` and a JSON error body, exercising the
    /// error path of the in-process arm.
    Fail,
    /// Panic while handling the request, simulating a surface that unwinds
    /// (a `unwrap()` on bad upstream state, an out-of-bounds index, an
    /// `expect()` on a poisoned lock). This is the third failure shape a turn
    /// can see and the only one that crosses a thread boundary.
    Panic,
}

/// One dispatch the surface observed.
struct Dispatch {
    path: String,
    content_type: Option<String>,
    headers: HeaderMap,
    body: serde_json::Value,
}

/// Builds a dispatcher backed by a real router that records every inbound
/// request. The catch-all route mirrors the CloudRouter assembly's own
/// `/{*path}` contribution router
/// (`sdkwork-api-cloudrouter-assembly::bootstrap`), so the surface records
/// what the *endpoint* was rather than 404-ing it away — the recorded path is
/// then asserted per step, which is what catches a turn dispatched at the
/// wrong endpoint.
///
/// The URI keeps its query string (`path_and_query`), because Google's
/// streaming endpoint is distinguished by `?alt=sse` and `uri.path()` alone
/// drops it — a trap that makes google turns silently receive chat framing.
fn recording_surface(
    recorded: Arc<Mutex<Vec<Dispatch>>>,
    reachable: Arc<Mutex<Vec<String>>>,
    mode: SurfaceMode,
) -> EmbeddedSurfaceDispatcher {
    let router = Router::new().route(
        "/{*path}",
        post(
            move |uri: axum::http::Uri, headers: HeaderMap, body: Body| {
                let recorded = Arc::clone(&recorded);
                let reachable = Arc::clone(&reachable);
                async move {
                    let path = uri
                        .path_and_query()
                        .map(|pq| pq.as_str().to_string())
                        .unwrap_or_else(|| uri.path().to_string());
                    let raw = axum::body::to_bytes(body, 8 * 1024 * 1024)
                        .await
                        .expect("request body readable");
                    let request_body: serde_json::Value =
                        serde_json::from_slice(&raw).unwrap_or(serde_json::Value::Null);
                    let tool_turn = request_body
                        .get("tools")
                        .and_then(|tools| tools.as_array())
                        .is_some_and(|tools| !tools.is_empty());
                    recorded.lock().expect("recorded lock").push(Dispatch {
                        path: path.clone(),
                        content_type: headers
                            .get("content-type")
                            .and_then(|value| value.to_str().ok())
                            .map(str::to_string),
                        headers,
                        body: request_body,
                    });

                    if mode == SurfaceMode::Fail {
                        return Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("content-type", "application/json")
                            .body(Body::from(
                                r#"{"type":"about:blank","title":"Internal error","status":500}"#,
                            ))
                            .expect("error response builds");
                    }

                    if mode == SurfaceMode::Panic {
                        // The request was already recorded above, so the test
                        // can still prove the turn reached the surface before
                        // the thread died. Unwinding here is exactly what a
                        // real surface does when an invariant breaks.
                        panic!("embedded surface unwound while handling the request");
                    }

                    reachable.lock().expect("reachable lock").push(path.clone());
                    // The routing below mirrors what a real CloudRouter open-api
                    // surface does: the *endpoint* selects the protocol, and every
                    // streaming request arrives with `accept: text/event-stream`
                    // (both arms hardcode that — `chat_stream::open_gateway_response`
                    // passes it for all four protocols). A chat request that carried
                    // `tools` gets a tool-call stream, because that is what an
                    // upstream model does when it is offered functions.
                    let body = if path.ends_with(":streamGenerateContent?alt=sse") {
                        GOOGLE_STREAM_BODY
                    } else if path.starts_with("/v1/responses") {
                        RESPONSES_STREAM_BODY
                    } else if path.starts_with("/anthropic/v1/messages") {
                        ANTHROPIC_STREAM_BODY
                    } else if tool_turn {
                        TOOL_CALL_STREAM_BODY
                    } else {
                        CHAT_STREAM_BODY
                    };
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/event-stream")
                        .body(Body::from(body))
                        .expect("stream response builds")
                }
            },
        ),
    );
    Arc::new(move |request| {
        let router = router.clone();
        Box::pin(async move { ServiceExt::oneshot(router, request).await })
    })
}

/// The process-global inputs (profile, override, API key) are shared by every
/// step and are mutated by some; this serializes env-mutating assertions.
fn env_guard() -> MutexGuard<'static, ()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Copies the observable fields of one recorded dispatch.
struct Observed {
    path: String,
    content_type: Option<String>,
    body: serde_json::Value,
    credential_sources: usize,
}

fn observed_at(recorded: &Arc<Mutex<Vec<Dispatch>>>, slot: usize) -> Observed {
    let dispatches = recorded.lock().expect("recorded lock");
    let dispatch = dispatches.get(slot).unwrap_or_else(|| {
        panic!(
            "expected a dispatch at slot {slot}; the surface saw {}",
            dispatches.len()
        )
    });
    let credential_sources = ["x-api-key", "authorization"]
        .iter()
        .filter(|name| dispatch.headers.contains_key(**name))
        .count();
    Observed {
        path: dispatch.path.clone(),
        content_type: dispatch.content_type.clone(),
        body: dispatch.body.clone(),
        credential_sources,
    }
}

/// One turn's observable outcome.
struct TurnOutcome {
    deltas: Vec<String>,
    reasoning: Vec<String>,
    model: Option<String>,
    finish_reason: Option<String>,
    tool_fragments: Vec<String>,
    tool_calls: Vec<sdkwork_agents_tool_cloudrouter::StreamedToolCall>,
}

/// Drives one turn through the embedded arm.
fn run_embedded_turn(
    recorded: &Arc<Mutex<Vec<Dispatch>>>,
    api_key: Option<&str>,
    protocol: WireProtocol,
    tools: Vec<serde_json::Value>,
) -> TurnOutcome {
    match api_key {
        Some(key) => std::env::set_var("SDKWORK_CLOUDROUTER_OPEN_API_KEY", key),
        None => std::env::remove_var("SDKWORK_CLOUDROUTER_OPEN_API_KEY"),
    }

    let transport = CloudRouterTurnTransport::resolve().expect("embedded transport resolves");
    assert!(
        matches!(transport, CloudRouterTurnTransport::InProcess { .. }),
        "a standalone profile with an installed surface must select the in-process arm"
    );

    let mut request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
    request.model = "gpt-4o-mini".to_string();
    // A real conversation, not an empty vec: most body-shaping logic derives
    // from the messages, so an empty history hides body differences (that blind
    // spot is what let the google `stream` field drift go unnoticed).
    request.messages = vec![
        cloudrouter_open_sdk::models::OpenAiChatMessage {
            content: Some("You are a helpful agent.".to_string()),
            role: "system".to_string(),
            ..Default::default()
        },
        cloudrouter_open_sdk::models::OpenAiChatMessage {
            content: Some("read a.txt".to_string()),
            role: "user".to_string(),
            ..Default::default()
        },
    ];
    let tool_count = tools.len();

    let before = recorded.lock().expect("recorded lock").len();
    let mut deltas = Vec::new();
    let mut reasoning = Vec::new();
    let result = transport
        .stream_chat_completion(
            CloudRouterTurnRequest {
                protocol,
                auth_token: "login-auth-token",
                access_token: Some("login-access-token"),
                request,
                tools,
            },
            &mut |delta| {
                if !delta.content.is_empty() {
                    deltas.push(delta.content);
                }
                if !delta.reasoning_content.is_empty() {
                    reasoning.push(delta.reasoning_content);
                }
            },
        )
        .unwrap_or_else(|error| {
            let last = observed_at(recorded, before);
            panic!(
                "embedded turn completes: protocol={:?} tools={tool_count} endpoint={} \
                 dispatched_body={} error={error:?}",
                protocol, last.path, last.body
            )
        });

    assert_eq!(
        before + 1,
        recorded.lock().expect("recorded lock").len(),
        "each turn must produce exactly one dispatch"
    );
    TurnOutcome {
        deltas,
        reasoning,
        model: result.model,
        finish_reason: result.finish_reason,
        tool_fragments: result.tool_call_fragments,
        tool_calls: result.tool_calls,
    }
}

/// Asserts the invariant that keeps the in-process hop accepted by
/// CloudRouter, over *every* dispatch recorded so far: `api-key-or-dual-token`
/// allows exactly one credential source, and "API key mixed with either token"
/// is an explicit 400 (`API_SPEC.md` §10).
///
/// Asserting it globally rather than per-step means a future step that forgets
/// the mutual exclusion fails here even if its own step assertions pass.
fn assert_one_credential_source_per_dispatch(recorded: &Arc<Mutex<Vec<Dispatch>>>) {
    let dispatches = recorded.lock().expect("recorded lock");
    for (slot, dispatch) in dispatches.iter().enumerate() {
        let sources = ["x-api-key", "authorization"]
            .iter()
            .filter(|name| dispatch.headers.contains_key(**name))
            .count();
        assert_eq!(
            1, sources,
            "dispatch {slot} ({}) carried {sources} credential sources; \
             CloudRouter accepts exactly one",
            dispatch.path
        );
    }
}

/// The full embedded turn path, asserted end to end.
///
/// Kept as a single test so the process-global surface installation happens
/// once and the steps below stay order-independent (see the module docs).
#[test]
fn embedded_turn_transport_end_to_end() {
    let _guard = env_guard();

    // The profile and the installed surface together select the arm; both are
    // process-global, and this test owns the process' transport configuration.
    std::env::set_var("SDKWORK_DEPLOYMENT_PROFILE", "standalone");
    std::env::remove_var("SDKWORK_AGENTS_CLOUDROUTER_BASE_URL");

    let recorded = Arc::new(Mutex::new(Vec::new()));
    let reachable = Arc::new(Mutex::new(Vec::new()));
    assert!(
        install_embedded_cloudrouter_surface(recording_surface(
            Arc::clone(&recorded),
            Arc::clone(&reachable),
            SurfaceMode::Serve,
        )),
        "the first installation must win the process-global surface slot"
    );
    // A second installation must report that it was kept out instead of
    // silently appearing to succeed — the observability gap that made the
    // original `OnceLock` design invisible when five tests raced for the slot.
    assert!(
        !install_embedded_cloudrouter_surface(recording_surface(
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(Vec::new())),
            SurfaceMode::Serve,
        )),
        "a second installation must report `false`, not silently win"
    );

    // --- Step 1: the turn that used to fail with 50301 now completes. ---
    let outcome = run_embedded_turn(&recorded, None, WireProtocol::ChatCompletions, Vec::new());
    assert_eq!(
        vec!["po".to_string(), "ng".to_string()],
        outcome.deltas,
        "the SSE body must be decoded into ordered visible deltas"
    );
    assert_eq!(
        Some("gpt-4o-mini".to_string()),
        outcome.model,
        "the accumulator must recover the model from the stream"
    );
    assert_eq!(
        Some("stop".to_string()),
        outcome.finish_reason,
        "the stop reason must be normalized to the OpenAI-style vocabulary"
    );

    // --- Step 2: it reached the open-api chat endpoint. ---
    let first = observed_at(&recorded, 0);
    assert_eq!("/v1/chat/completions", first.path);
    assert_eq!(
        1, first.credential_sources,
        "the turn must present exactly one credential source"
    );
    // The conversation must ride the request: an empty `messages` array would
    // make every downstream body assertion vacuous.
    {
        let messages = first
            .body
            .get("messages")
            .and_then(|value| value.as_array())
            .expect("the chat body must carry a messages array");
        assert_eq!(2, messages.len(), "both conversation turns must be sent");
        assert_eq!(Some("system"), messages[0]["role"].as_str());
        assert_eq!(
            Some("read a.txt"),
            messages[1]["content"].as_str(),
            "message content must survive into the dispatched body"
        );
        assert_eq!(Some("gpt-4o-mini"), first.body["model"].as_str());
    }

    // --- Step 3: the dual-token pair is present and no API key rides along. ---
    {
        let dispatches = recorded.lock().expect("recorded lock");
        let headers = &dispatches[0].headers;
        assert_eq!(
            Some("Bearer login-auth-token"),
            headers.get("authorization").map(|v| v.to_str().unwrap())
        );
        assert_eq!(
            Some("login-access-token"),
            headers.get("Access-Token").map(|v| v.to_str().unwrap())
        );
        assert!(
            !headers.contains_key("x-api-key"),
            "the dual-token branch must not also carry an API key"
        );
    }

    // --- Step 4: the open-api framing headers are set. ---
    assert_eq!(Some("application/json"), first.content_type.as_deref());
    {
        let dispatches = recorded.lock().expect("recorded lock");
        assert_eq!(
            Some("text/event-stream"),
            dispatches[0]
                .headers
                .get("accept")
                .map(|v| v.to_str().unwrap()),
            "the streaming hop must ask for an SSE response"
        );
    }

    // --- Step 5: configuring an API key switches to the API-key-only branch.
    // CloudRouter answers `400 invalid_request` for "API key mixed with either
    // token" (API_SPEC §10), so this branch must drop both token headers. ---
    let keyed = run_embedded_turn(
        &recorded,
        Some("sk-embedded-key"),
        WireProtocol::ChatCompletions,
        Vec::new(),
    );
    assert_eq!(vec!["po".to_string(), "ng".to_string()], keyed.deltas);
    {
        let dispatches = recorded.lock().expect("recorded lock");
        let headers = &dispatches[1].headers;
        assert_eq!(
            Some("sk-embedded-key"),
            headers.get("x-api-key").map(|v| v.to_str().unwrap())
        );
        assert!(
            !headers.contains_key("authorization"),
            "an API-key turn must not also carry Authorization"
        );
        assert!(
            !headers.contains_key("Access-Token"),
            "an API-key turn must not also carry Access-Token"
        );
    }

    // --- Step 6: a blank key is absent, so the turn falls back to the pair. ---
    run_embedded_turn(
        &recorded,
        Some("   "),
        WireProtocol::ChatCompletions,
        Vec::new(),
    );
    {
        let dispatches = recorded.lock().expect("recorded lock");
        let headers = &dispatches[2].headers;
        assert!(
            !headers.contains_key("x-api-key"),
            "a blank key must not become an empty credential header"
        );
        assert_eq!(
            Some("Bearer login-auth-token"),
            headers.get("authorization").map(|v| v.to_str().unwrap())
        );
    }

    // --- Step 7: a non-chat protocol reaches its own endpoint over the
    // in-process hop, and its native framing is decoded. The HTTP arm already
    // covered this; the in-process arm builds the endpoint independently, so
    // this is the regression guard for the second implementation. ---
    let responses = run_embedded_turn(&recorded, None, WireProtocol::OpenAiResponses, Vec::new());
    assert_eq!(vec!["po".to_string(), "ng".to_string()], responses.deltas);
    let responses_dispatch = observed_at(&recorded, 3);
    assert_eq!(
        "/v1/responses", responses_dispatch.path,
        "openai_responses must not be dispatched at the chat completions endpoint"
    );
    // Each protocol has its own request shape; asserting them catches a body
    // that silently reverts to the OpenAI chat form.
    assert!(
        responses_dispatch.body.get("input").is_some(),
        "the responses body must use `input`, not `messages`"
    );
    assert_eq!(
        Some(true),
        responses_dispatch.body["stream"].as_bool(),
        "the responses body carries a `stream` boolean"
    );

    let google = run_embedded_turn(&recorded, None, WireProtocol::GoogleContent, Vec::new());
    assert_eq!(
        vec!["answer".to_string()],
        google.deltas,
        "a google answer part must arrive as visible content"
    );
    assert_eq!(
        vec!["reason step".to_string()],
        google.reasoning,
        "a google thought part must arrive as reasoning, not content"
    );
    let google_dispatch = observed_at(&recorded, 4);
    assert_eq!(
        "/google/v1beta/models/gpt-4o-mini:streamGenerateContent?alt=sse",
        google_dispatch.path,
    );
    // Google signals streaming through the URL and has no `stream` body field.
    // An earlier build forced `stream: true` on every body, which made the
    // in-process body differ from the HTTP body for this protocol only — a
    // transport-dependent difference for no benefit.
    assert_eq!(
        None,
        google_dispatch.body["stream"].as_bool(),
        "the google body must not carry a `stream` field"
    );
    assert!(
        google_dispatch.body.get("contents").is_some(),
        "the google body must use `contents`, not `messages`"
    );
    assert!(
        google_dispatch.body.get("messages").is_none(),
        "the google body must not carry the OpenAI `messages` shape"
    );

    let anthropic = run_embedded_turn(&recorded, None, WireProtocol::AnthropicMessages, Vec::new());
    let anthropic_dispatch = observed_at(&recorded, 5);
    assert_eq!(
        "/anthropic/v1/messages", anthropic_dispatch.path,
        "anthropic_messages must not be dispatched at the chat completions endpoint"
    );
    // Anthropic mandates an explicit output budget and separates the system
    // prompt; a regression to the chat shape would drop both.
    assert!(
        anthropic_dispatch.body.get("max_tokens").is_some(),
        "the anthropic body must carry the mandatory max_tokens"
    );
    assert_eq!(
        Some("You are a helpful agent."),
        anthropic_dispatch.body["system"].as_str(),
        "the system prompt must be hoisted out of the message list"
    );
    assert!(
        anthropic_dispatch
            .body
            .get("messages")
            .and_then(|value| value.as_array())
            .is_some_and(|messages| messages
                .iter()
                .all(|message| message["role"].as_str() != Some("system"))),
        "the anthropic message list must not contain system-role entries"
    );
    assert_eq!(
        vec!["po".to_string(), "ng".to_string()],
        anthropic.deltas,
        "the anthropic framing must be decoded by the anthropic parser"
    );
    assert_eq!(
        Some("claude-sonnet-4".to_string()),
        anthropic.model,
        "the anthropic parser must recover the model from message_start"
    );
    assert_eq!(
        Some("stop".to_string()),
        anthropic.finish_reason,
        "anthropic's end_turn must normalize to the shared lowercase vocabulary"
    );

    // --- Step 8: the tool-call path. When `tools` is non-empty the arm pins
    // the OpenAI chat shape (`build_in_process_request`), builds the body by
    // hand instead of via `build_protocol_request_body`, and must still land on
    // `/v1/chat/completions` with the tools carried at message fidelity. ---
    let tool_outcome = run_embedded_turn(
        &recorded,
        None,
        WireProtocol::GoogleContent, // deliberately mismatched: tools must pin chat
        vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "read a file",
                "parameters": {"type": "object", "properties": {"path": {"type": "string"}}}
            }
        })],
    );
    let tool_dispatch = observed_at(&recorded, 6);
    assert_eq!(
        "/v1/chat/completions", tool_dispatch.path,
        "a tool-bearing request must pin the chat completions endpoint even when the \
         session asked for another protocol"
    );
    assert_eq!(
        Some("auto"),
        tool_dispatch.body["tool_choice"].as_str(),
        "tool turns must ask the model to choose"
    );
    assert_eq!(
        Some(false),
        tool_dispatch.body["parallel_tool_calls"].as_bool(),
        "tool turns must disable parallel calls so the caller's loop runs one at a time"
    );
    assert_eq!(
        Some(true),
        tool_dispatch.body["stream"].as_bool(),
        "the in-process hop is always streaming"
    );
    let tools = tool_dispatch
        .body
        .get("tools")
        .and_then(|value| value.as_array())
        .expect("the tools array must ride the request");
    assert_eq!(1, tools.len());
    assert_eq!(Some("read_file"), tools[0]["function"]["name"].as_str());
    assert_eq!(
        Some("read a file"),
        tools[0]["function"]["description"].as_str(),
        "the function description must survive the hand-built body"
    );
    assert_eq!(
        Some("string"),
        tools[0]["function"]["parameters"]["properties"]["path"]["type"].as_str(),
        "the JSON Schema parameters must survive the hand-built body"
    );

    // --- Step 9: the streamed tool call is reconstructed, not just forwarded.
    // Upstream sends the id/name once and the JSON arguments as fragments; the
    // caller must receive complete, ready-to-execute calls (the whole reason
    // the HTTP arm has a dedicated tools entry point). ---
    assert_eq!(
        3,
        tool_outcome.tool_fragments.len(),
        "every tool_calls delta must be preserved as a raw fragment"
    );
    assert!(
        !tool_outcome.deltas.contains(&"po".to_string()),
        "a tool-only stream must not invent visible content"
    );
    assert_eq!(
        1,
        tool_outcome.tool_calls.len(),
        "the fragments of one indexed call must merge into exactly one call"
    );
    let call = &tool_outcome.tool_calls[0];
    assert_eq!("call_abc123", call.id, "the call id must survive the hop");
    assert_eq!(
        "read_file", call.name,
        "the function name must survive the hop"
    );
    assert_eq!(
        r#"{"path":"a.txt"}"#, call.arguments,
        "the argument fragments must be concatenated into executable JSON"
    );
    assert_eq!(
        Some("tool_calls".to_string()),
        tool_outcome.finish_reason,
        "the stop reason must report that the model asked for tools"
    );

    // --- Final: the credential invariant, over every dispatch this test
    // produced. Each step asserted its own branch; this catches the case where
    // a new step adds headers without noticing the mutual exclusion. ---
    assert_one_credential_source_per_dispatch(&recorded);
    assert_eq!(
        7,
        recorded.lock().expect("recorded lock").len(),
        "every step in this test must have produced exactly one dispatch"
    );
}

/// The other half of "the call succeeded": a failure must be loud.
///
/// A surface that answers `500` must surface as a transport error carrying the
/// status **and** the surface's own body — never as an empty successful turn,
/// and never as a generic "stream returned no content" that hides the cause.
///
/// This constructs the `InProcess` arm directly instead of installing a
/// failing surface into the process-global slot (which the end-to-end test
/// owns), so the two tests cannot race for it. The arm's fields are public
/// precisely so a caller/test can pin the transport without the resolver.
#[test]
fn a_failing_surface_surfaces_its_status_and_body() {
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let reachable = Arc::new(Mutex::new(Vec::new()));
    let transport = CloudRouterTurnTransport::InProcess {
        dispatcher: recording_surface(
            Arc::clone(&recorded),
            Arc::clone(&reachable),
            SurfaceMode::Fail,
        ),
        api_key: Some("sk-test-key".to_string()),
    };

    let mut request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
    request.model = "gpt-4o-mini".to_string();
    let mut deltas: Vec<String> = Vec::new();
    let error = transport
        .stream_chat_completion(
            CloudRouterTurnRequest {
                protocol: WireProtocol::ChatCompletions,
                auth_token: "login-auth-token",
                access_token: Some("login-access-token"),
                request,
                tools: Vec::new(),
            },
            &mut |delta| deltas.push(delta.content),
        )
        .expect_err("a 500 from the surface must not become a successful turn");

    assert_eq!(
        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        match &error {
            cloudrouter_open_sdk::SdkworkError::HttpStatus { status, .. } => *status,
            other => panic!("expected the surface status to be preserved, got {other:?}"),
        },
        "the surface's status must reach the caller"
    );
    let message = error.to_string();
    assert!(
        message.contains("500"),
        "the error must name the status: {message}"
    );
    assert!(
        message.contains("Internal error"),
        "the surface's own body must be preserved so an operator can see the cause: {message}"
    );
    assert!(
        deltas.is_empty(),
        "a failed turn must not emit deltas as if it had streamed content"
    );
    assert_eq!(
        1,
        recorded.lock().expect("recorded lock").len(),
        "the surface must still have received the request"
    );
}

/// The third failure shape: the dispatch thread **panics**.
///
/// The in-process arm runs the dispatcher future on a dedicated thread, so a
/// panicking surface unwinds *that* thread and `handle.join()` hands back an
/// `Err`. Without the `join` mapping, the panic would either propagate into the
/// synchronous turn worker (taking the agent worker down) or — worse — be
/// swallowed as an empty successful turn. This pins that it becomes an ordinary
/// transport failure carrying `EMBEDDED_LOCAL_FAILURE_STATUS` (`0`, "this never
/// left the process") and a message that names the panic, so an operator can
/// tell it apart from a real upstream error.
///
/// The panicking thread prints a backtrace to stderr; that is expected output,
/// not a test failure — cargo only fails a test when the *test* thread panics.
#[test]
fn a_panicking_surface_becomes_a_transport_failure_not_an_unwind() {
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let reachable = Arc::new(Mutex::new(Vec::new()));
    let transport = CloudRouterTurnTransport::InProcess {
        dispatcher: recording_surface(
            Arc::clone(&recorded),
            Arc::clone(&reachable),
            SurfaceMode::Panic,
        ),
        api_key: Some("sk-test-key".to_string()),
    };

    let mut request = cloudrouter_open_sdk::models::OpenAiChatCompletionRequest::default();
    request.model = "gpt-4o-mini".to_string();
    let mut deltas: Vec<String> = Vec::new();
    let error = transport
        .stream_chat_completion(
            CloudRouterTurnRequest {
                protocol: WireProtocol::ChatCompletions,
                auth_token: "login-auth-token",
                access_token: Some("login-access-token"),
                request,
                tools: Vec::new(),
            },
            &mut |delta| deltas.push(delta.content),
        )
        .expect_err("a panicking surface must not produce a successful turn");

    assert_eq!(
        0,
        match &error {
            cloudrouter_open_sdk::SdkworkError::HttpStatus { status, .. } => *status,
            other => panic!("expected a transport failure, got {other:?}"),
        },
        "an in-process dispatch failure must use the reserved local status (0), \
         not a status that looks like a real upstream response"
    );
    let message = error.to_string();
    assert!(
        message.contains("panicked"),
        "the error must name the panic so it is not mistaken for an upstream error: {message}"
    );
    assert!(
        deltas.is_empty(),
        "a panicked turn must not emit deltas as if it had streamed content"
    );
    assert_eq!(
        1,
        recorded.lock().expect("recorded lock").len(),
        "the turn must have reached the surface before the thread died"
    );
    assert!(
        reachable.lock().expect("reachable lock").is_empty(),
        "a panicking surface never completes a response, so nothing is reachable"
    );
}
