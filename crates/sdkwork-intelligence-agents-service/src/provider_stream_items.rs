use std::collections::HashMap;

use sdkwork_agent_kernel::{KernelError, KernelEvent, KernelEventRedaction, KernelResult};
use serde_json::{json, Map, Value};

use crate::domain::{AgentSessionItemKind, AgentSessionItemStatus};
use crate::ports::MAX_TURN_INPUT_CONTENT_BYTES;
use crate::provider_session_sync::stable_provider_session_item_id;

const PROVIDER_STREAM_PAYLOAD_SCHEMA: &str = "sdkwork.agent.provider_stream_event.v1";
const MAX_PROVIDER_ITEM_ID_BYTES: usize = 512;
const MAX_PROVIDER_ITEM_TYPE_BYTES: usize = 128;
const MAX_TOOL_ARGUMENTS_JSON_BYTES: usize = 256 * 1024;
const MAX_TOOL_RESULT_JSON_BYTES: usize = 1024 * 1024;
const MAX_PROVIDER_PAYLOAD_JSON_BYTES: usize = 1024 * 1024;
const REDACTED_TEXT: &str = "[redacted]";

/// Maximum number of durable canonical facts projected from one provider turn.
pub(crate) const MAX_PROVIDER_TURN_FACTS: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderTurnItemFact {
    pub item_id: String,
    pub kind: AgentSessionItemKind,
    pub content: Option<String>,
    pub content_type: String,
    pub status: AgentSessionItemStatus,
    pub provider_id: String,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_arguments_json: Option<String>,
    pub tool_result_json: Option<String>,
    pub provider_payload_json: Option<String>,
    pub parent_item_id: Option<String>,
    pub created_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedProviderItemEvent {
    identity: String,
    provider_id: String,
    provider_session_id: String,
    provider_event_type: String,
    item_id: String,
    item_type: String,
    item: Value,
    occurred_at: Option<String>,
    redacted: bool,
}

#[derive(Debug, Clone)]
struct ProviderItemLifecycle {
    first_occurred_at: Option<String>,
    terminal: Option<ParsedProviderItemEvent>,
}

/// Collapse provider item lifecycle events and project only terminal snapshots.
///
/// Provider-native ids remain correlation fields, while durable Agents item ids
/// are deterministic hashes over the provider Session and item identity.
///
/// Note: event kinds such as `item.started` / `item.completed` are provider
/// lifecycle event types, not durable ids — they share the `item.` stem with
/// durable `item.{engine}.{stable}` ids by coincidence and must not be parsed
/// as ids.
pub(crate) fn project_terminal_provider_turn_items(
    events: &[KernelEvent],
    session_id: &str,
    turn_id: &str,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    let mut lifecycle_by_identity = HashMap::<String, usize>::new();
    let mut lifecycles = Vec::<ProviderItemLifecycle>::new();

    for event in events {
        let Some(parsed) = parse_provider_item_event(event, turn_id)? else {
            continue;
        };
        let lifecycle_index = match lifecycle_by_identity.get(&parsed.identity) {
            Some(index) => *index,
            None => {
                let index = lifecycles.len();
                lifecycle_by_identity.insert(parsed.identity.clone(), index);
                lifecycles.push(ProviderItemLifecycle {
                    first_occurred_at: parsed.occurred_at.clone(),
                    terminal: None,
                });
                index
            }
        };
        let lifecycle = &mut lifecycles[lifecycle_index];
        if lifecycle.first_occurred_at.is_none() {
            lifecycle.first_occurred_at = parsed.occurred_at.clone();
        }
        if parsed.provider_event_type == "item.completed" {
            lifecycle.terminal = Some(parsed);
        }
    }

    let mut facts = Vec::new();
    for lifecycle in lifecycles {
        let Some(terminal) = lifecycle.terminal else {
            continue;
        };
        let remaining = MAX_PROVIDER_TURN_FACTS.saturating_sub(facts.len());
        if remaining == 0 {
            return Err(KernelError::validation(format!(
                "provider turn exceeds maximum canonical item count of {MAX_PROVIDER_TURN_FACTS}"
            )));
        }
        let mut projected =
            project_terminal_item(terminal, session_id, turn_id, lifecycle.first_occurred_at)?;
        if projected.len() > remaining {
            return Err(KernelError::validation(format!(
                "provider turn exceeds maximum canonical item count of {MAX_PROVIDER_TURN_FACTS}"
            )));
        }
        facts.append(&mut projected);
    }
    Ok(facts)
}

pub(crate) fn terminal_provider_assistant_item_id(
    events: &[KernelEvent],
    fallback_provider_session_id: Option<&str>,
) -> KernelResult<Option<String>> {
    let mut terminal = None;
    for event in events {
        let Some(parsed) = parse_provider_item_event(event, "")? else {
            continue;
        };
        if parsed.provider_event_type == "item.completed" && parsed.item_type == "agent_message" {
            terminal = Some(parsed);
        }
    }
    Ok(terminal.and_then(|item| {
        let provider_session_id = if item.provider_session_id.trim().is_empty() {
            fallback_provider_session_id.unwrap_or_default().trim()
        } else {
            item.provider_session_id.trim()
        };
        (!provider_session_id.is_empty()).then(|| {
            stable_provider_session_item_id(&item.provider_id, provider_session_id, &item.item_id)
        })
    }))
}

fn parse_provider_item_event(
    event: &KernelEvent,
    turn_id: &str,
) -> KernelResult<Option<ParsedProviderItemEvent>> {
    if event.payload_schema.as_deref() != Some(PROVIDER_STREAM_PAYLOAD_SCHEMA) {
        return Ok(None);
    }
    let payload: Value = serde_json::from_str(&event.payload).map_err(|error| {
        KernelError::validation(format!(
            "provider stream event payload must be valid JSON: {error}"
        ))
    })?;
    let payload = payload.as_object().ok_or_else(|| {
        KernelError::validation("provider stream event payload must be a JSON object")
    })?;
    if payload.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err(KernelError::validation(
            "provider stream event payload schemaVersion must be 1",
        ));
    }
    let provider_event_type = required_string(payload, "providerEventType", 64)?;
    // The kernel forwards the raw JSON-RPC notification method names (for
    // example "item/completed"); historical consumers used the dotted form.
    // Both spellings are normalized so the live projection is
    // convention-agnostic and never silently skips terminal items.
    let provider_event_type = provider_event_type.replace('/', ".");
    if !matches!(
        provider_event_type.as_str(),
        "item.started" | "item.updated" | "item.completed"
    ) {
        return Ok(None);
    }
    let item = payload
        .get("item")
        .and_then(Value::as_object)
        .ok_or_else(|| KernelError::validation("provider item event payload must contain item"))?;
    let item_id = required_string(item, "id", MAX_PROVIDER_ITEM_ID_BYTES)?;
    let item_type = required_string(item, "type", MAX_PROVIDER_ITEM_TYPE_BYTES)?;
    if !turn_id.is_empty()
        && event
            .step_id
            .as_deref()
            .is_some_and(|step_id| step_id != turn_id)
    {
        return Err(KernelError::validation(
            "provider stream event step identity does not match Turn identity",
        ));
    }
    if optional_string(payload, "providerItemId", MAX_PROVIDER_ITEM_ID_BYTES)?
        .as_deref()
        .is_some_and(|provider_item_id| provider_item_id != item_id)
    {
        return Err(KernelError::validation(
            "provider stream payload item identity does not match item snapshot",
        ));
    }
    let provider_id = required_string(payload, "providerId", 128)?;
    let provider_session_id =
        optional_string(payload, "providerSessionId", MAX_PROVIDER_ITEM_ID_BYTES)?
            // Compatibility for provider events persisted before the normalized
            // payload adopted providerSessionId. KernelEvent.session_id is canonical
            // SDKWork Session identity and must never be used as this fallback.
            .or(optional_string(
                payload,
                "threadId",
                MAX_PROVIDER_ITEM_ID_BYTES,
            )?)
            .unwrap_or_default();
    let run_id = event
        .run_id
        .as_deref()
        .or(event.correlation_id.as_deref())
        .unwrap_or_default()
        .trim()
        .to_string();
    let identity = format!(
        "{provider_id}\u{0}{provider_session_id}\u{0}{run_id}\u{0}{turn_id}\u{0}{item_id}\u{0}{item_type}"
    );

    Ok(Some(ParsedProviderItemEvent {
        identity,
        provider_id,
        provider_session_id,
        provider_event_type,
        item_id,
        item_type,
        item: Value::Object(item.clone()),
        occurred_at: event.occurred_at.clone(),
        redacted: matches!(
            event.redaction_classification,
            KernelEventRedaction::Secret | KernelEventRedaction::Regulated
        ),
    }))
}

fn project_terminal_item(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    match terminal.item_type.as_str() {
        // The existing assistant-output record remains the Turn response item.
        "agent_message" => Ok(Vec::new()),
        "reasoning" => project_reasoning(terminal, session_id, turn_id, first_occurred_at),
        "command_execution"
        | "file_change"
        | "mcp_tool_call"
        | "web_search"
        | "dynamic_tool_call"
        | "collab_agent_tool_call"
        | "sleep"
        | "image_generation" => project_tool(terminal, session_id, turn_id, first_occurred_at),
        "image_view" => project_image(terminal, session_id, turn_id, first_occurred_at),
        "plan" => project_plan(terminal, session_id, turn_id, first_occurred_at),
        "hook_prompt" => json_fact(
            terminal,
            session_id,
            turn_id,
            first_occurred_at,
            AgentSessionItemKind::SystemInstruction,
        ),
        "user_message" => json_fact(
            terminal,
            session_id,
            turn_id,
            first_occurred_at,
            AgentSessionItemKind::UserInput,
        ),
        "todo_list" => project_status_notice(terminal, session_id, turn_id, first_occurred_at),
        "error" => project_error_notice(terminal, session_id, turn_id, first_occurred_at),
        // sub_agent_activity, context_compaction, entered/exited review mode and
        // any future provider item type remain visible status notices carrying
        // the full typed item JSON.
        _ => project_status_notice(terminal, session_id, turn_id, first_occurred_at),
    }
}

fn project_plan(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    let content = if terminal.redacted {
        REDACTED_TEXT.to_string()
    } else {
        let text = terminal
            .item
            .get("text")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| KernelError::validation("terminal plan item must contain text"))?;
        bounded_text(text, "provider plan", MAX_TURN_INPUT_CONTENT_BYTES)?
    };
    Ok(vec![text_fact(
        &terminal,
        session_id,
        turn_id,
        first_occurred_at,
        AgentSessionItemKind::AssistantOutput,
        content,
        if terminal.redacted {
            AgentSessionItemStatus::Redacted
        } else {
            AgentSessionItemStatus::Completed
        },
    )?])
}

fn project_image(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    let content = if terminal.redacted {
        REDACTED_TEXT.to_string()
    } else {
        let path = terminal
            .item
            .get("path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .ok_or_else(|| KernelError::validation("terminal image view item must contain path"))?;
        bounded_text(path, "provider image path", MAX_TURN_INPUT_CONTENT_BYTES)?
    };
    Ok(vec![text_fact(
        &terminal,
        session_id,
        turn_id,
        first_occurred_at,
        AgentSessionItemKind::ArtifactReference,
        content,
        if terminal.redacted {
            AgentSessionItemStatus::Redacted
        } else {
            AgentSessionItemStatus::Completed
        },
    )?])
}

fn json_fact(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
    kind: AgentSessionItemKind,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    // Textual provider item types keep their readable text as item content so
    // the canonical projection matches the provider-history reconciliation
    // path; the full typed item JSON is preserved in `provider_payload_json`.
    let content = if terminal.redacted {
        REDACTED_TEXT.to_string()
    } else {
        match terminal.item_type.as_str() {
            "hook_prompt" => {
                let text = terminal
                    .item
                    .get("fragments")
                    .and_then(Value::as_array)
                    .map(|fragments| {
                        fragments
                            .iter()
                            .filter_map(|fragment| fragment.get("text").and_then(Value::as_str))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default();
                bounded_text(&text, "provider hook prompt", MAX_TURN_INPUT_CONTENT_BYTES)?
            }
            "user_message" => {
                let text = terminal
                    .item
                    .get("content")
                    .and_then(Value::as_array)
                    .map(|content| {
                        content
                            .iter()
                            .filter_map(|input| input.get("text").and_then(Value::as_str))
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .unwrap_or_default();
                bounded_text(&text, "provider user message", MAX_TURN_INPUT_CONTENT_BYTES)?
            }
            _ => bounded_json(
                &terminal.item,
                "provider item",
                MAX_TURN_INPUT_CONTENT_BYTES,
            )?,
        }
    };
    Ok(vec![text_fact(
        &terminal,
        session_id,
        turn_id,
        first_occurred_at,
        kind,
        content,
        if terminal.redacted {
            AgentSessionItemStatus::Redacted
        } else {
            AgentSessionItemStatus::Completed
        },
    )?])
}

fn project_reasoning(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    let content = if terminal.redacted {
        REDACTED_TEXT.to_string()
    } else {
        let text = terminal
            .item
            .get("text")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| KernelError::validation("terminal reasoning item must contain text"))?;
        bounded_text(text, "provider reasoning", MAX_TURN_INPUT_CONTENT_BYTES)?
    };
    Ok(vec![text_fact(
        &terminal,
        session_id,
        turn_id,
        first_occurred_at,
        AgentSessionItemKind::Reasoning,
        content,
        if terminal.redacted {
            AgentSessionItemStatus::Redacted
        } else {
            AgentSessionItemStatus::Completed
        },
    )?])
}

fn project_status_notice(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    let content = if terminal.redacted {
        REDACTED_TEXT.to_string()
    } else {
        bounded_json(
            &terminal.item,
            "provider status item",
            MAX_TURN_INPUT_CONTENT_BYTES,
        )?
    };
    let status = if terminal.redacted {
        AgentSessionItemStatus::Redacted
    } else {
        item_status(&terminal.item, AgentSessionItemStatus::Completed)?
    };
    Ok(vec![text_fact(
        &terminal,
        session_id,
        turn_id,
        first_occurred_at,
        AgentSessionItemKind::StatusNotice,
        content,
        status,
    )?])
}

fn project_error_notice(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    let content = if terminal.redacted {
        REDACTED_TEXT.to_string()
    } else {
        let message = terminal
            .item
            .get("message")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|message| !message.is_empty())
            .ok_or_else(|| KernelError::validation("terminal error item must contain message"))?;
        bounded_text(message, "provider error", MAX_TURN_INPUT_CONTENT_BYTES)?
    };
    Ok(vec![text_fact(
        &terminal,
        session_id,
        turn_id,
        first_occurred_at,
        AgentSessionItemKind::ErrorNotice,
        content,
        if terminal.redacted {
            AgentSessionItemStatus::Redacted
        } else {
            AgentSessionItemStatus::Failed
        },
    )?])
}

fn project_tool(
    terminal: ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    first_occurred_at: Option<String>,
) -> KernelResult<Vec<ProviderTurnItemFact>> {
    let tool_name = tool_name(&terminal)?;
    let status = if terminal.redacted {
        AgentSessionItemStatus::Redacted
    } else {
        item_status(&terminal.item, AgentSessionItemStatus::Completed)?
    };
    let call_item_id = durable_item_id(session_id, turn_id, &terminal, "tool_call");
    let result_item_id = durable_item_id(session_id, turn_id, &terminal, "tool_result");
    let tool_call_id = if terminal.redacted {
        call_item_id.clone()
    } else {
        terminal.item_id.clone()
    };
    let redacted_payload = json!({
        "redacted": true,
        "type": terminal.item_type,
    });
    let arguments = if terminal.redacted {
        bounded_json(
            &redacted_payload,
            "redacted provider tool arguments",
            MAX_TOOL_ARGUMENTS_JSON_BYTES,
        )?
    } else {
        bounded_json(
            &tool_arguments(&terminal)?,
            "provider tool arguments",
            MAX_TOOL_ARGUMENTS_JSON_BYTES,
        )?
    };
    let result = if terminal.redacted {
        bounded_json(
            &redacted_payload,
            "redacted provider tool result",
            MAX_TOOL_RESULT_JSON_BYTES,
        )?
    } else {
        bounded_json(
            &terminal.item,
            "provider tool result",
            MAX_TOOL_RESULT_JSON_BYTES,
        )?
    };
    let completed_at = terminal.occurred_at.clone();
    let provider_payload_json = if terminal.redacted {
        None
    } else {
        Some(bounded_json(
            &terminal.item,
            "provider payload",
            MAX_PROVIDER_PAYLOAD_JSON_BYTES,
        )?)
    };
    Ok(vec![
        ProviderTurnItemFact {
            item_id: call_item_id.clone(),
            kind: AgentSessionItemKind::ToolCall,
            content: None,
            content_type: "application/json".to_string(),
            status,
            provider_id: terminal.provider_id.clone(),
            tool_name: Some(tool_name.clone()),
            tool_call_id: Some(tool_call_id.clone()),
            tool_arguments_json: Some(arguments),
            tool_result_json: None,
            provider_payload_json: provider_payload_json.clone(),
            parent_item_id: None,
            created_at: first_occurred_at.clone(),
            completed_at: completed_at.clone(),
        },
        ProviderTurnItemFact {
            item_id: result_item_id,
            kind: AgentSessionItemKind::ToolResult,
            content: None,
            content_type: "application/json".to_string(),
            status,
            provider_id: terminal.provider_id,
            tool_name: Some(tool_name),
            tool_call_id: Some(tool_call_id),
            tool_arguments_json: None,
            tool_result_json: Some(result),
            provider_payload_json,
            parent_item_id: Some(call_item_id),
            created_at: first_occurred_at,
            completed_at,
        },
    ])
}

fn text_fact(
    terminal: &ParsedProviderItemEvent,
    session_id: &str,
    turn_id: &str,
    created_at: Option<String>,
    kind: AgentSessionItemKind,
    content: String,
    status: AgentSessionItemStatus,
) -> KernelResult<ProviderTurnItemFact> {
    let provider_payload_json = if terminal.redacted {
        None
    } else {
        Some(bounded_json(
            &terminal.item,
            "provider payload",
            MAX_PROVIDER_PAYLOAD_JSON_BYTES,
        )?)
    };
    Ok(ProviderTurnItemFact {
        item_id: durable_item_id(session_id, turn_id, terminal, kind.as_str()),
        kind,
        content: Some(content),
        content_type: if terminal.item_type == "todo_list"
            || !matches!(terminal.item_type.as_str(), "reasoning" | "error")
        {
            "application/json".to_string()
        } else {
            "text/plain".to_string()
        },
        status,
        provider_id: terminal.provider_id.clone(),
        tool_name: None,
        tool_call_id: None,
        tool_arguments_json: None,
        tool_result_json: None,
        provider_payload_json,
        parent_item_id: None,
        created_at,
        completed_at: terminal.occurred_at.clone(),
    })
}

fn tool_name(terminal: &ParsedProviderItemEvent) -> KernelResult<String> {
    match terminal.item_type.as_str() {
        "command_execution" => Ok("shell_command".to_string()),
        "file_change" => Ok("apply_patch".to_string()),
        "web_search" => Ok("web_search".to_string()),
        "sleep" => Ok("sleep".to_string()),
        "image_generation" => Ok("image_generation".to_string()),
        "mcp_tool_call" | "dynamic_tool_call" | "collab_agent_tool_call" => terminal
            .item
            .get("tool")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|tool| !tool.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                KernelError::validation(format!(
                    "terminal {} tool item must contain tool",
                    terminal.item_type
                ))
            }),
        _ => Ok(terminal.item_type.clone()),
    }
}

fn tool_arguments(terminal: &ParsedProviderItemEvent) -> KernelResult<Value> {
    // Keep the full terminal provider item as the tool payload so every
    // protocol field (command details, exit code, duration, app context,
    // plugin identity, results, etc.) survives the canonical projection,
    // mirroring the provider-history reconciliation payload.
    Ok(terminal.item.clone())
}

fn item_status(
    item: &Value,
    default: AgentSessionItemStatus,
) -> KernelResult<AgentSessionItemStatus> {
    let Some(status) = item.get("status").and_then(Value::as_str) else {
        return Ok(default);
    };
    match status.trim().to_ascii_lowercase().as_str() {
        "completed" | "complete" | "success" | "succeeded" => Ok(AgentSessionItemStatus::Completed),
        "failed" | "error" | "declined" => Ok(AgentSessionItemStatus::Failed),
        "cancelled" | "canceled" | "aborted" => Ok(AgentSessionItemStatus::Cancelled),
        "pending" | "queued" | "running" | "in_progress" => Err(KernelError::validation(
            "terminal provider item cannot retain a non-terminal status",
        )),
        _ => Err(KernelError::validation(
            "terminal provider item contains an unsupported status",
        )),
    }
}

fn durable_item_id(
    _session_id: &str,
    _turn_id: &str,
    terminal: &ParsedProviderItemEvent,
    fact_kind: &str,
) -> String {
    let provider_item_key = if fact_kind == "tool_result" {
        format!("{}\u{0}result", terminal.item_id)
    } else {
        terminal.item_id.clone()
    };
    stable_provider_session_item_id(
        &terminal.provider_id,
        &terminal.provider_session_id,
        &provider_item_key,
    )
}

fn required_string(
    object: &Map<String, Value>,
    field_name: &str,
    max_bytes: usize,
) -> KernelResult<String> {
    optional_string(object, field_name, max_bytes)?.ok_or_else(|| {
        KernelError::validation(format!(
            "provider stream event {field_name} must be a non-empty string"
        ))
    })
}

fn optional_string(
    object: &Map<String, Value>,
    field_name: &str,
    max_bytes: usize,
) -> KernelResult<Option<String>> {
    let Some(value) = object.get(field_name) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        KernelError::validation(format!(
            "provider stream event {field_name} must be a string"
        ))
    })?;
    let normalized = value.trim();
    if normalized.is_empty() {
        return Ok(None);
    }
    if normalized.len() > max_bytes {
        return Err(KernelError::validation(format!(
            "provider stream event {field_name} exceeds {max_bytes} bytes"
        )));
    }
    Ok(Some(normalized.to_string()))
}

fn bounded_text(value: &str, field_name: &str, max_bytes: usize) -> KernelResult<String> {
    if value.len() > max_bytes {
        return Err(KernelError::validation(format!(
            "{field_name} exceeds {max_bytes} bytes"
        )));
    }
    Ok(value.to_string())
}

fn bounded_json(value: &Value, field_name: &str, max_bytes: usize) -> KernelResult<String> {
    let serialized = serde_json::to_string(value).map_err(|error| {
        KernelError::validation(format!("{field_name} must be valid JSON: {error}"))
    })?;
    if serialized.len() > max_bytes {
        return Err(KernelError::validation(format!(
            "{field_name} exceeds {max_bytes} bytes"
        )));
    }
    Ok(serialized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::{KernelEventSeverity, KernelEventSource};

    fn event(
        sequence: u64,
        provider_event_type: &str,
        item: Value,
        redaction: KernelEventRedaction,
    ) -> KernelEvent {
        let item_id = item["id"].as_str().expect("item id").to_string();
        KernelEvent::new(
            format!("event.test.{sequence}"),
            if item["type"] == "error" {
                "agent.step.failed"
            } else {
                "agent.step.completed"
            },
            KernelEventSeverity::Info,
            json!({
                "schemaVersion": 1,
                "providerId": "codex",
                "providerEventType": provider_event_type,
                "sequence": sequence,
                "providerSessionId": "provider-session.test",
                "providerItemId": item_id,
                "item": item,
                "usage": null,
                "error": null,
            })
            .to_string(),
        )
        .occurred_at(format!("2026-07-30T00:00:{sequence:02}Z"))
        .from_source(KernelEventSource::Provider)
        .for_session("session.canonical.test")
        .for_run("provider-run.test")
        .with_redaction(redaction)
        .with_payload_schema(PROVIDER_STREAM_PAYLOAD_SCHEMA)
    }

    #[test]
    fn terminal_snapshots_project_to_canonical_facts_in_first_seen_order() {
        let events = vec![
            event(
                0,
                "item.started",
                json!({"id":"reasoning-1","type":"reasoning","text":""}),
                KernelEventRedaction::TenantSensitive,
            ),
            event(
                1,
                "item.completed",
                json!({"id":"command-1","type":"command_execution","command":"cargo test","aggregated_output":"passed","exit_code":0,"status":"completed"}),
                KernelEventRedaction::TenantSensitive,
            ),
            event(
                2,
                "item.completed",
                json!({"id":"reasoning-1","type":"reasoning","text":"Inspect the repository"}),
                KernelEventRedaction::TenantSensitive,
            ),
            event(
                3,
                "item.completed",
                json!({"id":"todo-1","type":"todo_list","items":[{"text":"Run tests","completed":true}]}),
                KernelEventRedaction::TenantSensitive,
            ),
            event(
                4,
                "item.completed",
                json!({"id":"error-1","type":"error","message":"non-fatal provider warning"}),
                KernelEventRedaction::TenantSensitive,
            ),
            event(
                5,
                "item.completed",
                json!({"id":"message-1","type":"agent_message","text":"Done"}),
                KernelEventRedaction::TenantSensitive,
            ),
        ];

        let facts = project_terminal_provider_turn_items(&events, "session.test", "turn.test")
            .expect("provider facts");
        assert_eq!(
            facts.iter().map(|fact| fact.kind).collect::<Vec<_>>(),
            vec![
                AgentSessionItemKind::Reasoning,
                AgentSessionItemKind::ToolCall,
                AgentSessionItemKind::ToolResult,
                AgentSessionItemKind::StatusNotice,
                AgentSessionItemKind::ErrorNotice,
            ]
        );
        assert_eq!(facts[0].created_at.as_deref(), Some("2026-07-30T00:00:00Z"));
        assert_eq!(
            facts[0].completed_at.as_deref(),
            Some("2026-07-30T00:00:02Z")
        );
        assert_eq!(facts[1].tool_name.as_deref(), Some("shell_command"));
        assert_eq!(facts[1].tool_call_id.as_deref(), Some("command-1"));
        assert_eq!(facts[2].parent_item_id, Some(facts[1].item_id.clone()));
        assert_eq!(facts[4].status, AgentSessionItemStatus::Failed);
        assert!(facts.iter().all(|fact| fact.provider_id == "codex"));
        // The full raw provider item JSON is preserved on every projected fact
        // so provider protocol data survives without loss.
        assert!(facts
            .iter()
            .all(|fact| fact.provider_payload_json.is_some()));
        assert_eq!(
            facts[1].provider_payload_json.as_deref(),
            Some(
                serde_json::json!({
                    "id": "command-1",
                    "type": "command_execution",
                    "command": "cargo test",
                    "aggregated_output": "passed",
                    "exit_code": 0,
                    "status": "completed"
                })
                .to_string()
                .as_str()
            )
        );
    }

    #[test]
    fn slash_spelled_provider_event_types_still_project_terminal_facts() {
        // The kernel forwards raw JSON-RPC notification method names
        // ("item/started", "item/completed"); the projection must not depend
        // on the dotted spelling used by older consumers.
        let events = vec![
            event(
                0,
                "item/started",
                json!({"id":"reasoning-1","type":"reasoning","text":""}),
                KernelEventRedaction::TenantSensitive,
            ),
            event(
                1,
                "item/completed",
                json!({"id":"reasoning-1","type":"reasoning","text":"Inspect the repository"}),
                KernelEventRedaction::TenantSensitive,
            ),
            event(
                2,
                "item/completed",
                json!({"id":"message-1","type":"agent_message","text":"Done"}),
                KernelEventRedaction::TenantSensitive,
            ),
        ];
        let facts = project_terminal_provider_turn_items(&events, "session.test", "turn.test")
            .expect("slash-spelled provider facts");
        // assistant output is intentionally carried by the Turn response item,
        // not as a terminal fact; the reasoning item must still project.
        assert_eq!(
            facts.iter().map(|fact| fact.kind).collect::<Vec<_>>(),
            vec![AgentSessionItemKind::Reasoning]
        );
        assert_eq!(facts[0].provider_payload_json.is_some(), true);
        let terminal =
            terminal_provider_assistant_item_id(&events, None).expect("terminal assistant id");
        assert!(terminal.is_some());
    }

    #[test]
    fn redacted_terminal_facts_do_not_preserve_raw_provider_payload() {
        let source = event(
            0,
            "item.completed",
            json!({"id":"secret-1","type":"command_execution","command":"export TOKEN=abc","aggregated_output":"token leaked","status":"completed"}),
            KernelEventRedaction::Secret,
        );
        let facts = project_terminal_provider_turn_items(&[source], "session.test", "turn.test")
            .expect("redacted provider facts");
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].status, AgentSessionItemStatus::Redacted);
        assert_eq!(facts[0].provider_payload_json, None);
        assert_eq!(facts[1].provider_payload_json, None);
        assert_eq!(
            facts[0].tool_arguments_json.as_deref(),
            Some("{\"redacted\":true,\"type\":\"command_execution\"}")
        );
    }

    #[test]
    fn command_execution_terminal_keeps_the_full_provider_payload() {
        let terminal = event(
            0,
            "item.completed",
            json!({
                "id": "command-1",
                "type": "command_execution",
                "command": "cargo test",
                "cwd": "E:/workspace",
                "processId": "p-1",
                "status": "completed",
                "commandActions": [{"kind": "read", "name": "a.rs", "path": "E:/workspace/a.rs"}],
                "aggregatedOutput": "passed",
                "exitCode": 0,
                "durationMs": 42,
                "pluginId": null,
                "scriptPath": null,
                "source": "exec"
            }),
            KernelEventRedaction::TenantSensitive,
        );
        let facts = project_terminal_provider_turn_items(&[terminal], "session.test", "turn.test")
            .expect("provider facts");
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].kind, AgentSessionItemKind::ToolCall);
        let arguments: Value = serde_json::from_str(
            facts[0]
                .tool_arguments_json
                .as_deref()
                .expect("tool arguments"),
        )
        .expect("tool arguments JSON");
        // Every protocol field survives the live projection.
        assert_eq!(arguments["command"], json!("cargo test"));
        assert_eq!(arguments["cwd"], json!("E:/workspace"));
        assert_eq!(arguments["processId"], json!("p-1"));
        assert_eq!(arguments["exitCode"], json!(0));
        assert_eq!(arguments["durationMs"], json!(42));
        assert_eq!(arguments["source"], json!("exec"));
        assert_eq!(arguments["commandActions"][0]["kind"], json!("read"));
        assert_eq!(facts[1].kind, AgentSessionItemKind::ToolResult);
    }

    #[test]
    fn mcp_tool_call_terminal_keeps_app_context_and_result() {
        let terminal = event(
            0,
            "item.completed",
            json!({
                "id": "mcp-1",
                "type": "mcp_tool_call",
                "server": "docs",
                "tool": "search",
                "status": "completed",
                "arguments": {"query": "codex"},
                "appContext": {"connectorId": "c-1", "linkId": "l-1", "resourceUri": "res://1", "appName": "docs", "actionName": "search"},
                "mcpAppResourceUri": "res://1",
                "pluginId": "plugin-1",
                "readOnlyHint": true,
                "result": {"content": [{"type": "text", "text": "found"}]},
                "error": null,
                "durationMs": 7
            }),
            KernelEventRedaction::TenantSensitive,
        );
        let facts = project_terminal_provider_turn_items(&[terminal], "session.test", "turn.test")
            .expect("provider facts");
        assert_eq!(facts[0].kind, AgentSessionItemKind::ToolCall);
        let arguments: Value = serde_json::from_str(
            facts[0]
                .tool_arguments_json
                .as_deref()
                .expect("tool arguments"),
        )
        .expect("tool arguments JSON");
        assert_eq!(arguments["server"], json!("docs"));
        assert_eq!(arguments["appContext"]["connectorId"], json!("c-1"));
        assert_eq!(arguments["mcpAppResourceUri"], json!("res://1"));
        assert_eq!(arguments["pluginId"], json!("plugin-1"));
        assert_eq!(arguments["readOnlyHint"], json!(true));
        assert_eq!(arguments["result"]["content"][0]["text"], json!("found"));
        assert_eq!(facts[1].kind, AgentSessionItemKind::ToolResult);
    }

    #[test]
    fn live_projection_covers_every_tool_and_activity_item_type() {
        let cases: Vec<(Value, AgentSessionItemKind)> = vec![
            (
                json!({"id":"dyn-1","type":"dynamic_tool_call","tool":"lookup","status":"completed","arguments":{},"contentItems":[{"type":"inputText","text":"hit"}],"success":true}),
                AgentSessionItemKind::ToolCall,
            ),
            (
                json!({"id":"collab-1","type":"collab_agent_tool_call","tool":"spawnAgent","status":"completed","senderThreadId":"s-1","receiverThreadIds":["c-1"],"prompt":"go","model":"gpt-5","reasoningEffort":null,"agentsStates":{}}),
                AgentSessionItemKind::ToolCall,
            ),
            (
                json!({"id":"sleep-1","type":"sleep","durationMs":100}),
                AgentSessionItemKind::ToolCall,
            ),
            (
                json!({"id":"img-1","type":"image_generation","status":"completed","revisedPrompt":null,"result":"https://example.invalid/i.png","savedPath":null}),
                AgentSessionItemKind::ToolCall,
            ),
            (
                json!({"id":"view-1","type":"image_view","path":"E:/workspace/a.png"}),
                AgentSessionItemKind::ArtifactReference,
            ),
            (
                json!({"id":"plan-1","type":"plan","text":"step one"}),
                AgentSessionItemKind::AssistantOutput,
            ),
            (
                json!({"id":"hook-1","type":"hook_prompt","fragments":[{"text":"retry","hookRunId":"r-1"}]}),
                AgentSessionItemKind::SystemInstruction,
            ),
            (
                json!({"id":"user-1","type":"user_message","clientId":null,"content":[{"type":"inputText","text":"hello"}]}),
                AgentSessionItemKind::UserInput,
            ),
            (
                json!({"id":"sub-1","type":"sub_agent_activity","kind":"started","agentThreadId":"c-1","agentPath":"0.0"}),
                AgentSessionItemKind::StatusNotice,
            ),
            (
                json!({"id":"compaction-1","type":"context_compaction"}),
                AgentSessionItemKind::StatusNotice,
            ),
        ];
        for (item, expected_kind) in cases {
            let facts = project_terminal_provider_turn_items(
                &[event(
                    0,
                    "item.completed",
                    item,
                    KernelEventRedaction::TenantSensitive,
                )],
                "session.test",
                "turn.test",
            )
            .unwrap_or_else(|error| {
                panic!("item {} failed projection: {error}", expected_kind.as_str())
            });
            assert!(
                facts.iter().any(|fact| fact.kind == expected_kind),
                "item type {} did not project a {} fact: {:?}",
                facts.first().map(|fact| fact.kind.as_str()).unwrap_or("?"),
                expected_kind.as_str(),
                facts
                    .iter()
                    .map(|fact| fact.kind.as_str())
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn provider_item_identity_uses_payload_provider_session_not_canonical_event_session() {
        let source = event(
            0,
            "item.completed",
            json!({"id":"command-1","type":"command_execution","command":"cargo test","aggregated_output":"passed","exit_code":0,"status":"completed"}),
            KernelEventRedaction::TenantSensitive,
        );
        let mut different_canonical_session = source.clone();
        different_canonical_session.session_id = Some("session.canonical.other".to_string());
        let mut different_provider_session = source.clone();
        let mut payload: Value =
            serde_json::from_str(&different_provider_session.payload).expect("provider payload");
        payload["providerSessionId"] = json!("provider-session.other");
        different_provider_session.payload = payload.to_string();

        let source_facts = project_terminal_provider_turn_items(
            &[source],
            "session.canonical.test",
            "turn.canonical.test",
        )
        .expect("source provider facts");
        let canonical_alias_facts = project_terminal_provider_turn_items(
            &[different_canonical_session],
            "session.canonical.test",
            "turn.canonical.test",
        )
        .expect("canonical alias provider facts");
        let provider_alias_facts = project_terminal_provider_turn_items(
            &[different_provider_session],
            "session.canonical.test",
            "turn.canonical.test",
        )
        .expect("provider alias facts");

        assert_eq!(source_facts[0].item_id, canonical_alias_facts[0].item_id);
        assert_ne!(source_facts[0].item_id, provider_alias_facts[0].item_id);
    }

    #[test]
    fn terminal_assistant_identity_matches_provider_history_identity() {
        let assistant = event(
            0,
            "item.completed",
            json!({"id":"message-1","type":"agent_message","text":"Done"}),
            KernelEventRedaction::TenantSensitive,
        );

        let item_id =
            terminal_provider_assistant_item_id(&[assistant], Some("provider-session.fallback"))
                .expect("assistant identity")
                .expect("terminal assistant");

        assert_eq!(
            item_id,
            stable_provider_session_item_id("codex", "provider-session.test", "message-1")
        );
    }

    #[test]
    fn secret_tool_snapshot_is_persistable_without_raw_payload() {
        let facts = project_terminal_provider_turn_items(
            &[event(
                0,
                "item.completed",
                json!({"id":"command-secret","type":"command_execution","command":"token=private","aggregated_output":"private","status":"completed"}),
                KernelEventRedaction::Secret,
            )],
            "session.test",
            "turn.test",
        )
        .expect("redacted facts");
        assert_eq!(facts.len(), 2);
        assert!(facts
            .iter()
            .all(|fact| fact.status == AgentSessionItemStatus::Redacted));
        assert!(!facts[0]
            .tool_arguments_json
            .as_deref()
            .unwrap_or_default()
            .contains("private"));
        assert!(!facts[1]
            .tool_result_json
            .as_deref()
            .unwrap_or_default()
            .contains("private"));
    }
}
