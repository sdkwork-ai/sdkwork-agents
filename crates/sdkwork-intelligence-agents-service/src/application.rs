use std::collections::HashMap;
use std::sync::Arc;

mod commands;
mod turn_input_queue;
pub use commands::*;
pub use turn_input_queue::*;

use crate::agent_turn::{AgentTurnRecord, AgentTurnStatus};
use crate::domain::{
    AgentAuditAction, AgentAuditPayload, AgentBusinessRecord, AgentBusinessStatus,
    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentImplementationKind, AgentImplementationType, AgentInteractionKind, AgentInteractionRecord,
    AgentInteractionStatus, AgentItemDriveRefRecord, AgentItemFeedbackRecord,
    AgentItemResourceRole, AgentProviderBindingRecord, AgentResourceType,
    AgentResourceUserStateRecord, AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord,
    AgentRuntimeExecutionStatus, AgentSessionCheckpointRecord, AgentSessionCheckpointStatus,
    AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionRecord,
    AgentSessionRuntimeBindingRecord, AgentSessionRuntimeBindingStatus, AgentSessionStatus,
    AgentSessionTitleSource, AgentTaskRecord, AgentTaskStatus, AgentVisibility,
    MarketplaceAuditPayload, ProviderBindingAuditPayload, RuntimeExecutionAuditPayload,
    SessionAuditPayload, SessionItemAuditPayload, TaskAuditPayload,
    DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
use crate::agent_turn::AgentTurnMode;
use crate::dto::AgentManagementProfileDto;
use crate::list_cursors::{
    encode_created_at_cursor, encode_session_list_cursor, CreatedAtListCursor, SessionListCursor,
};
use crate::ports::{
    offset_paginated_result, AgentAuditSink, AgentRepository, PaginatedResult, PaginationParams,
    ProviderBindingListQuery, SessionItemListQuery, TurnRequestWriteOutcome, MAX_PAGE_SIZE,
    MAX_TURN_INPUT_CONTENT_BYTES, TURN_CONTEXT_ITEM_LIMIT,
};
use crate::project::{
    project_names_equal, AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode,
    AgentProjectRecord, AgentProjectStatus, AgentProjectVisibility,
};
use crate::provider_stream_items::{
    project_terminal_provider_turn_items, terminal_provider_assistant_item_id,
    MAX_PROVIDER_TURN_FACTS,
};
use crate::runtime_facade_bridge::{
    engine_key_for_binding_id, execute_preview_response, execute_prompt_optimization,
    RUNTIME_MODE_CONTRACT_FALLBACK,
};
use crate::session_activity::SessionActivitySummaryRecord;
use crate::session_id_scheme::{
    is_provider_item_id_for, is_provider_runtime_binding_id_for, is_provider_session_id,
    is_provider_session_id_for,
};
use crate::session_item_cursor::{encode_session_item_cursor, SessionItemCursor};
use crate::task_execution_cursor::{
    encode_task_cursor, encode_task_run_attempt_cursor, encode_task_run_cursor, TaskCursor,
    TaskRunAttemptCursor, TaskRunCursor,
};
use crate::task_scheduler::{
    execute_task_run_claim, ClaimTaskRunsRequest, MaterializeDueTasksRequest,
    ReconcileTaskRunRequest, TaskRunClaim, TaskRunLease, TaskSchedulerRepository,
};
use crate::task_scheduling::{AgentTaskRunRecord, AgentTaskRunStatus};
use crate::turn_runtime::{
    complete_with_timeout, complete_with_timeout_and_sink, is_capacity_error, is_inference_error,
    turn_model_request_id, ContractTurnExecutor, TurnCancellationInput, TurnExecutionInput,
    TurnExecutionStreamSink, TurnExecutor, TURN_EXECUTION_TIMEOUT,
};
use crate::validation::{
    default_json_array_if_blank, default_json_object_if_blank, default_plain_text_if_blank,
    is_trimmed_blank, parse_optional_rfc3339_datetime, parse_rfc3339_datetime, require_non_blank,
    validate_capabilities, validate_requested_at, validate_standard_id, ID_PREFIX_AGENT,
    ID_PREFIX_BINDING, ID_PREFIX_CHECKPOINT, ID_PREFIX_EXECUTION, ID_PREFIX_INTERACTION,
    ID_PREFIX_ITEM, ID_PREFIX_PROFILE, ID_PREFIX_PROJECT, ID_PREFIX_PROVIDER,
    ID_PREFIX_RUNTIME_BINDING, ID_PREFIX_SESSION, ID_PREFIX_SLOT, ID_PREFIX_TASK, ID_PREFIX_TURN,
    ID_PREFIX_WORKSPACE,
};
use crate::workspace::{default_workspace_id, AgentWorkspaceRecord, AgentWorkspaceStatus};
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelErrorKind, KernelEvent, KernelEventRedaction,
    KernelEventSeverity, KernelEventSource, KernelResult, PolicyCategory, PolicyDecisionValue,
    PolicyProvider, PolicyRequest, PolicySubject,
};
use sdkwork_agents_contract::agents_allow_contract_runtime_fallback;
use sdkwork_agents_runtime_facade::AgentEngineInteractionResolution;
use sdkwork_utils_rust::{sha256_hash, trim};
use time::OffsetDateTime;

const MAX_TURN_DRIVE_REFS: usize = 64;
const MAX_INTERACTION_LEASE_SECONDS: u32 = 300;
const MAX_INTERACTION_JSON_BYTES: usize = 64 * 1024;
const MAX_JSON_PAYLOAD_BYTES: usize = 1024 * 1024;
const MAX_METADATA_JSON_BYTES: usize = 64 * 1024;
const MAX_TOOL_ARGUMENTS_JSON_BYTES: usize = 256 * 1024;
const MAX_TOOL_RESULT_JSON_BYTES: usize = 1024 * 1024;
const MAX_PROVIDER_PAYLOAD_JSON_BYTES: usize = 1024 * 1024;
const MAX_TASK_TITLE_BYTES: usize = 512;
const MAX_TASK_CRON_EXPRESSION_BYTES: usize = 256;
const MAX_TASK_TIMEZONE_BYTES: usize = 128;
const MAX_TASK_EXTERNAL_REF_BYTES: usize = 256;

#[derive(Debug, Clone, Copy)]
struct TaskExecutionPolicyInput {
    max_concurrent_runs: u16,
    max_catch_up_runs: u16,
    max_attempts: u16,
    retry_initial_delay_seconds: u32,
    retry_max_delay_seconds: u32,
    timeout_seconds: u32,
    priority: i16,
}

impl From<&CreateTaskCommand> for TaskExecutionPolicyInput {
    fn from(command: &CreateTaskCommand) -> Self {
        Self {
            max_concurrent_runs: command.max_concurrent_runs,
            max_catch_up_runs: command.max_catch_up_runs,
            max_attempts: command.max_attempts,
            retry_initial_delay_seconds: command.retry_initial_delay_seconds,
            retry_max_delay_seconds: command.retry_max_delay_seconds,
            timeout_seconds: command.timeout_seconds,
            priority: command.priority,
        }
    }
}

impl From<&ReplaceTaskCommand> for TaskExecutionPolicyInput {
    fn from(command: &ReplaceTaskCommand) -> Self {
        Self {
            max_concurrent_runs: command.max_concurrent_runs,
            max_catch_up_runs: command.max_catch_up_runs,
            max_attempts: command.max_attempts,
            retry_initial_delay_seconds: command.retry_initial_delay_seconds,
            retry_max_delay_seconds: command.retry_max_delay_seconds,
            timeout_seconds: command.timeout_seconds,
            priority: command.priority,
        }
    }
}

fn validate_task_execution_policy(input: TaskExecutionPolicyInput) -> KernelResult<()> {
    if !(1..=crate::task_scheduling::MAX_TASK_CONCURRENT_RUNS).contains(&input.max_concurrent_runs)
    {
        return Err(KernelError::validation(
            "maxConcurrentRuns must be between 1 and 32",
        ));
    }
    if !(1..=crate::task_scheduling::MAX_TASK_CATCH_UP_RUNS).contains(&input.max_catch_up_runs) {
        return Err(KernelError::validation(
            "maxCatchUpRuns must be between 1 and 100",
        ));
    }
    if !(1..=crate::task_scheduling::MAX_TASK_RUN_ATTEMPTS).contains(&input.max_attempts) {
        return Err(KernelError::validation(
            "maxAttempts must be between 1 and 20",
        ));
    }
    if !(crate::task_scheduling::MIN_TASK_TIMEOUT_SECONDS
        ..=crate::task_scheduling::MAX_TASK_TIMEOUT_SECONDS)
        .contains(&input.timeout_seconds)
    {
        return Err(KernelError::validation(
            "timeoutSeconds must be between 1 and 86400",
        ));
    }
    if input.retry_initial_delay_seconds == 0
        || input.retry_initial_delay_seconds > 86_400
        || input.retry_max_delay_seconds < input.retry_initial_delay_seconds
        || input.retry_max_delay_seconds > 604_800
    {
        return Err(KernelError::validation("invalid retry delay range"));
    }
    if !(-100..=100).contains(&input.priority) {
        return Err(KernelError::validation(
            "priority must be between -100 and 100",
        ));
    }
    Ok(())
}

fn validate_task_definition_fields(
    title: &Option<String>,
    cron_expression: &Option<String>,
    timezone: &str,
    external_ref: &Option<String>,
) -> KernelResult<()> {
    validate_optional_bounded(title, "title", MAX_TASK_TITLE_BYTES)?;
    validate_optional_bounded(
        cron_expression,
        "cronExpression",
        MAX_TASK_CRON_EXPRESSION_BYTES,
    )?;
    require_non_blank(timezone, "timezone")?;
    if timezone.len() > MAX_TASK_TIMEZONE_BYTES {
        return Err(KernelError::validation(format!(
            "timezone exceeds {MAX_TASK_TIMEZONE_BYTES} bytes"
        )));
    }
    validate_optional_bounded(external_ref, "externalRef", MAX_TASK_EXTERNAL_REF_BYTES)
}

fn agent_engine_runtime_manifest(engine_key: &str, agent_id: &str) -> AgentManifest {
    AgentManifest {
        schema_version: "1.0".to_string(),
        manifest_type: "agent".to_string(),
        agent_id: agent_id.to_string(),
        name: engine_key.to_string(),
        display_name: engine_key.to_string(),
        description: "Canonical local agent-engine runtime identity".to_string(),
        version: "1.0.0".to_string(),
        domain: "coding".to_string(),
        required_capabilities: vec!["model.chat".to_string()],
        optional_capabilities: vec!["session.resume".to_string()],
        required_capability_requirements: Vec::new(),
        optional_capability_requirements: Vec::new(),
        event_families: Vec::new(),
        owner_name: "sdkwork-agents".to_string(),
        status: "active".to_string(),
    }
}

fn validate_runtime_token(value: &str, field_name: &str, max_bytes: usize) -> KernelResult<()> {
    require_non_blank(value, field_name)?;
    if value.len() > max_bytes
        || !value.bytes().enumerate().all(|(index, byte)| match byte {
            b'a'..=b'z' => true,
            b'0'..=b'9' | b'_' | b'-' => index > 0,
            _ => false,
        })
    {
        return Err(KernelError::validation(format!("{field_name} is invalid")));
    }
    Ok(())
}

fn validate_optional_bounded(
    value: &Option<String>,
    field_name: &str,
    max_bytes: usize,
) -> KernelResult<()> {
    if let Some(value) = value {
        require_non_blank(value, field_name)?;
        if value.len() > max_bytes {
            return Err(KernelError::validation(format!(
                "{field_name} exceeds {max_bytes} bytes"
            )));
        }
    }
    Ok(())
}

fn normalize_optional_bounded(
    value: Option<String>,
    field_name: &str,
    max_bytes: usize,
) -> KernelResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = trim(&value);
    require_non_blank(&value, field_name)?;
    if value.len() > max_bytes {
        return Err(KernelError::validation(format!(
            "{field_name} exceeds {max_bytes} bytes"
        )));
    }
    Ok(Some(value.to_string()))
}

fn format_utc_seconds(value: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        value.year(),
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute(),
        value.second()
    )
}

fn validate_interaction_options(value: &serde_json::Value) -> KernelResult<()> {
    let options = value
        .as_array()
        .ok_or_else(|| KernelError::validation("options must be an array"))?;
    if options.len() > 128 {
        return Err(KernelError::validation("options exceeds 128 items"));
    }
    let mut values = std::collections::HashSet::with_capacity(options.len());
    for option in options {
        let object = option
            .as_object()
            .ok_or_else(|| KernelError::validation("each option must be an object"))?;
        if object.keys().any(|key| key != "value" && key != "label") {
            return Err(KernelError::validation(
                "options contains an unsupported field",
            ));
        }
        let option_value = object
            .get("value")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| KernelError::validation("options.value is required"))?;
        let label = object
            .get("label")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| KernelError::validation("options.label is required"))?;
        require_non_blank(option_value, "options.value")?;
        require_non_blank(label, "options.label")?;
        if option_value.len() > 256 || label.len() > 512 {
            return Err(KernelError::validation("interaction option is too long"));
        }
        if !values.insert(option_value) {
            return Err(KernelError::conflict(
                "interaction option values must be unique",
            ));
        }
    }
    Ok(())
}

fn validate_typed_interaction_request(
    interaction_kind: AgentInteractionKind,
    request_json: &str,
) -> KernelResult<serde_json::Value> {
    validate_bounded_json_payload(request_json, "request", MAX_INTERACTION_JSON_BYTES)?;
    let request: serde_json::Value = serde_json::from_str(request_json)
        .map_err(|error| KernelError::validation(format!("request must be valid JSON: {error}")))?;
    let object = request
        .as_object()
        .ok_or_else(|| KernelError::validation("request must be an object"))?;
    reject_unknown_json_fields(
        object,
        &[
            "schemaVersion",
            "category",
            "kind",
            "allowedActions",
            "data",
            "correlation",
        ],
        "request",
    )?;
    if object
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Err(KernelError::validation("request.schemaVersion must be 1"));
    }
    let category = required_json_string(object, "category", "request")?;
    let request_kind = required_json_string(object, "kind", "request")?;
    let data = object
        .get("data")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| KernelError::validation("request.data must be an object"))?;
    let (expected_category, expected_actions): (&str, &[&str]) = match request_kind {
        "command_execution" => (
            "approval",
            &[
                "accept",
                "accept_for_session",
                "accept_with_exec_policy_amendment",
                "apply_network_policy_amendment",
                "decline",
                "cancel",
            ],
        ),
        "file_change" => (
            "approval",
            &["accept", "accept_for_session", "decline", "cancel"],
        ),
        "permission_profile" => ("approval", &["grant", "decline", "cancel"]),
        "question_set" | "onboarding_question_set" => {
            validate_interaction_questions(data)?;
            ("user_input", &["submit", "cancel"])
        }
        "option_picker" => {
            require_json_array(data, "options", "request.data")?;
            required_json_bool(data, "allowMultiple", "request.data")?;
            ("user_input", &["submit", "skip", "dismiss"])
        }
        "context_source_picker" => ("user_input", &["continue", "skip", "dismiss"]),
        "setup_step" => {
            let step = required_json_string(data, "step", "request.data")?;
            match step {
                "context" => ("setup", &["continue", "skip", "dismiss"]),
                "role" | "task" => ("setup", &["submit", "skip", "dismiss"]),
                _ => return Err(KernelError::validation("request.data.step is invalid")),
            }
        }
        "mcp_elicitation" => {
            let mode = required_json_string(data, "mode", "request.data")?;
            if !matches!(mode, "form" | "openai/form" | "url") {
                return Err(KernelError::validation("request.data.mode is invalid"));
            }
            required_json_string(data, "serverName", "request.data")?;
            required_json_string(data, "message", "request.data")?;
            if mode == "url" {
                required_json_string(data, "elicitationId", "request.data")?;
                required_json_string(data, "url", "request.data")?;
            } else if !data.contains_key("requestedSchema") {
                return Err(KernelError::validation(
                    "request.data.requestedSchema is required",
                ));
            }
            ("elicitation", &["accept", "decline", "cancel"])
        }
        _ => return Err(KernelError::validation("request.kind is unsupported")),
    };
    if category != expected_category || interaction_kind.as_str() != category_to_kind(category)? {
        return Err(KernelError::validation(
            "request category does not match interaction kind",
        ));
    }
    let actions = require_json_array(object, "allowedActions", "request")?;
    let actual_actions = actions
        .iter()
        .map(|action| {
            action.as_str().ok_or_else(|| {
                KernelError::validation("request.allowedActions must contain strings")
            })
        })
        .collect::<KernelResult<Vec<_>>>()?;
    if actual_actions != expected_actions {
        return Err(KernelError::validation(
            "request.allowedActions does not match request kind",
        ));
    }
    if let Some(correlation) = object.get("correlation") {
        validate_provider_interaction_correlation(correlation)?;
    }
    Ok(request)
}

fn validate_provider_interaction_correlation(correlation: &serde_json::Value) -> KernelResult<()> {
    let correlation = correlation
        .as_object()
        .ok_or_else(|| KernelError::validation("request.correlation must be an object"))?;
    reject_unknown_json_fields(
        correlation,
        &[
            "modelRequestId",
            "providerId",
            "providerInteractionId",
            "providerItemId",
            "providerRequestId",
            "providerRequestIdType",
            "providerSessionId",
            "providerToolCallId",
            "providerToolName",
            "providerToolNamespace",
            "providerTurnId",
            "protocolMethod",
        ],
        "request.correlation",
    )?;
    for field in [
        "modelRequestId",
        "providerId",
        "providerSessionId",
        "providerTurnId",
        "protocolMethod",
    ] {
        require_non_blank(
            required_json_string(correlation, field, "request.correlation")?,
            field,
        )?;
    }
    for field in [
        "providerInteractionId",
        "providerItemId",
        "providerToolCallId",
        "providerToolName",
        "providerToolNamespace",
    ] {
        if let Some(value) = correlation.get(field).filter(|value| !value.is_null()) {
            let value = value.as_str().ok_or_else(|| {
                KernelError::validation(format!(
                    "request.correlation.{field} must be a string or null"
                ))
            })?;
            require_non_blank(value, field)?;
        }
    }
    let request_id = correlation.get("providerRequestId").ok_or_else(|| {
        KernelError::validation("request.correlation.providerRequestId is required")
    })?;
    let request_id_type =
        required_json_string(correlation, "providerRequestIdType", "request.correlation")?;
    match (request_id_type, request_id) {
        ("string", serde_json::Value::String(value)) if !value.trim().is_empty() => {}
        ("number", serde_json::Value::Number(value))
            if value
                .as_i64()
                .map(|value| value.unsigned_abs() <= 9_007_199_254_740_991)
                .or_else(|| value.as_u64().map(|value| value <= 9_007_199_254_740_991))
                == Some(true) => {}
        _ => {
            return Err(KernelError::validation(
                "request.correlation providerRequestId type is invalid",
            ));
        }
    }
    Ok(())
}

fn validate_typed_interaction_resolution(
    request_json: &str,
    resolution_json: &str,
) -> KernelResult<AgentInteractionStatus> {
    validate_bounded_json_payload(resolution_json, "resolution", MAX_INTERACTION_JSON_BYTES)?;
    let request: serde_json::Value = serde_json::from_str(request_json)
        .map_err(|error| KernelError::validation(format!("request must be valid JSON: {error}")))?;
    let request = request
        .as_object()
        .ok_or_else(|| KernelError::validation("request must be an object"))?;
    let request_kind = required_json_string(request, "kind", "request")?;
    let request_data = request
        .get("data")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| KernelError::validation("request.data must be an object"))?;
    let allowed_actions = require_json_array(request, "allowedActions", "request")?;
    let resolution: serde_json::Value = serde_json::from_str(resolution_json).map_err(|error| {
        KernelError::validation(format!("resolution must be valid JSON: {error}"))
    })?;
    let resolution = resolution
        .as_object()
        .ok_or_else(|| KernelError::validation("resolution must be an object"))?;
    let action = required_json_string(resolution, "action", "resolution")?;
    if !allowed_actions
        .iter()
        .any(|allowed| allowed.as_str() == Some(action))
    {
        return Err(KernelError::validation(
            "resolution.action is not allowed for this request",
        ));
    }
    validate_resolution_payload(request_kind, request_data, action, resolution)?;
    Ok(match action {
        "cancel" => AgentInteractionStatus::Cancelled,
        "decline" | "dismiss" => AgentInteractionStatus::Rejected,
        _ => AgentInteractionStatus::Resolved,
    })
}

fn validate_resolution_payload(
    request_kind: &str,
    request_data: &serde_json::Map<String, serde_json::Value>,
    action: &str,
    resolution: &serde_json::Map<String, serde_json::Value>,
) -> KernelResult<()> {
    match (request_kind, action) {
        ("command_execution", "accept_with_exec_policy_amendment") => {
            require_json_object(resolution, "execPolicyAmendment", "resolution")?;
        }
        ("command_execution", "apply_network_policy_amendment") => {
            require_json_object(resolution, "networkPolicyAmendment", "resolution")?;
        }
        ("permission_profile", "grant") => {
            require_json_object(resolution, "permissions", "resolution")?;
            if !matches!(
                required_json_string(resolution, "scope", "resolution")?,
                "turn" | "session"
            ) {
                return Err(KernelError::validation("resolution.scope is invalid"));
            }
            if resolution.contains_key("strictAutoReview") {
                required_json_bool(resolution, "strictAutoReview", "resolution")?;
            }
        }
        ("question_set" | "onboarding_question_set", "submit") => {
            validate_answer_map(request_data, resolution)?;
        }
        ("option_picker", "submit") => {
            validate_string_array(resolution, "selectedOptions", "resolution")?;
            if let Some(value) = resolution.get("freeformAnswer") {
                if !value.is_null() && !value.is_string() {
                    return Err(KernelError::validation(
                        "resolution.freeformAnswer must be a string or null",
                    ));
                }
            }
        }
        ("context_source_picker", "continue") => {
            validate_string_array(resolution, "selectedSources", "resolution")?;
        }
        ("setup_step", "submit" | "continue") => {
            match required_json_string(request_data, "step", "request.data")? {
                "role" => validate_string_array(resolution, "selectedRoles", "resolution")?,
                "task" => validate_answer_map(request_data, resolution)?,
                "context" => validate_string_array(resolution, "selectedSources", "resolution")?,
                _ => return Err(KernelError::validation("request.data.step is invalid")),
            }
        }
        ("mcp_elicitation", "accept") => {
            if !resolution.contains_key("content") {
                return Err(KernelError::validation("resolution.content is required"));
            }
            if resolution.contains_key("metadata") {
                require_json_object(resolution, "metadata", "resolution")?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_interaction_questions(
    data: &serde_json::Map<String, serde_json::Value>,
) -> KernelResult<()> {
    let questions = require_json_array(data, "questions", "request.data")?;
    if questions.is_empty() || questions.len() > 128 {
        return Err(KernelError::validation(
            "request.data.questions must contain between 1 and 128 items",
        ));
    }
    let mut ids = std::collections::HashSet::with_capacity(questions.len());
    for question in questions {
        let question = question
            .as_object()
            .ok_or_else(|| KernelError::validation("each question must be an object"))?;
        let id = required_json_string(question, "id", "request.data.questions[]")?;
        require_non_blank(id, "request.data.questions[].id")?;
        if !ids.insert(id) {
            return Err(KernelError::conflict("question ids must be unique"));
        }
        required_json_string(question, "header", "request.data.questions[]")?;
        required_json_string(question, "prompt", "request.data.questions[]")?;
        required_json_bool(question, "allowOther", "request.data.questions[]")?;
        required_json_bool(question, "secret", "request.data.questions[]")?;
        if let Some(options) = question.get("options") {
            if !options.is_null() {
                require_json_array(question, "options", "request.data.questions[]")?;
            }
        }
    }
    Ok(())
}

fn validate_answer_map(
    request_data: &serde_json::Map<String, serde_json::Value>,
    resolution: &serde_json::Map<String, serde_json::Value>,
) -> KernelResult<()> {
    let answers = require_json_object(resolution, "answers", "resolution")?;
    let question_ids = request_data
        .get("questions")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|question| question.get("id").and_then(serde_json::Value::as_str))
        .collect::<std::collections::HashSet<_>>();
    for (question_id, values) in answers {
        if !question_ids.contains(question_id.as_str()) {
            return Err(KernelError::validation(
                "resolution.answers contains an unknown question id",
            ));
        }
        let values = values
            .as_array()
            .ok_or_else(|| KernelError::validation("each answer must be a string array"))?;
        if values.iter().any(|value| !value.is_string()) {
            return Err(KernelError::validation(
                "each answer must be a string array",
            ));
        }
    }
    Ok(())
}

fn category_to_kind(category: &str) -> KernelResult<&'static str> {
    match category {
        "approval" => Ok("approval"),
        "user_input" => Ok("user_question"),
        "elicitation" => Ok("elicitation"),
        "setup" => Ok("setup"),
        _ => Err(KernelError::validation("request.category is invalid")),
    }
}

fn reject_unknown_json_fields(
    object: &serde_json::Map<String, serde_json::Value>,
    allowed: &[&str],
    field_name: &str,
) -> KernelResult<()> {
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(KernelError::validation(format!(
            "{field_name} contains an unsupported field"
        )));
    }
    Ok(())
}

fn required_json_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
    parent: &str,
) -> KernelResult<&'a str> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| KernelError::validation(format!("{parent}.{field} is required")))
}

fn required_json_bool(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    parent: &str,
) -> KernelResult<bool> {
    object
        .get(field)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| KernelError::validation(format!("{parent}.{field} is required")))
}

fn require_json_array<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
    parent: &str,
) -> KernelResult<&'a Vec<serde_json::Value>> {
    object
        .get(field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| KernelError::validation(format!("{parent}.{field} must be an array")))
}

fn require_json_object<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
    parent: &str,
) -> KernelResult<&'a serde_json::Map<String, serde_json::Value>> {
    object
        .get(field)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| KernelError::validation(format!("{parent}.{field} must be an object")))
}

fn validate_string_array(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    parent: &str,
) -> KernelResult<()> {
    let values = require_json_array(object, field, parent)?;
    if values.iter().any(|value| !value.is_string()) {
        return Err(KernelError::validation(format!(
            "{parent}.{field} must contain strings"
        )));
    }
    Ok(())
}

fn validate_interaction_claim(
    record: &AgentInteractionRecord,
    claim_token: &str,
    fencing_token: u64,
) -> KernelResult<()> {
    if !(32..=256).contains(&claim_token.len()) {
        return Err(KernelError::validation("claimToken is invalid"));
    }
    if record.fencing_token != fencing_token {
        return Err(KernelError::conflict("interaction fencing token mismatch"));
    }
    let expected_hash = record
        .claim_token_hash
        .as_deref()
        .ok_or_else(|| KernelError::conflict("interaction must be claimed before resolution"))?;
    if sha256_hash(claim_token.as_bytes()) != expected_hash {
        return Err(KernelError::conflict("interaction claim token mismatch"));
    }
    let expires_at = record
        .claim_expires_at
        .as_deref()
        .ok_or_else(|| KernelError::conflict("interaction claim has no expiration"))?;
    if parse_rfc3339_datetime(expires_at, "claimExpiresAt")? <= OffsetDateTime::now_utc() {
        return Err(KernelError::conflict("interaction claim has expired"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedItemDriveRef {
    resource_role: AgentItemResourceRole,
    drive_space_id: String,
    drive_node_id: String,
    sort_order: u32,
}

fn normalize_item_drive_resources(
    resources: &[AgentItemDriveRefInput],
) -> KernelResult<Vec<NormalizedItemDriveRef>> {
    if resources.len() > MAX_TURN_DRIVE_REFS {
        return Err(KernelError::validation(format!(
            "driveRefs exceeds maximum item count of {MAX_TURN_DRIVE_REFS}"
        )));
    }

    let mut normalized = Vec::with_capacity(resources.len());
    let mut uniqueness = std::collections::HashSet::with_capacity(resources.len());
    for (sort_order, resource) in resources.iter().enumerate() {
        let drive_space_id = resource.drive_space_id.trim();
        let drive_node_id = resource.drive_node_id.trim();
        if drive_space_id.is_empty() || drive_space_id.len() > 128 {
            return Err(KernelError::validation("driveRefs.driveSpaceId is invalid"));
        }
        if drive_node_id.is_empty() || drive_node_id.len() > 128 {
            return Err(KernelError::validation("driveRefs.driveNodeId is invalid"));
        }
        if !uniqueness.insert((
            drive_space_id.to_string(),
            drive_node_id.to_string(),
            resource.resource_role.as_str(),
        )) {
            return Err(KernelError::conflict(
                "duplicate session-item Drive reference",
            ));
        }
        normalized.push(NormalizedItemDriveRef {
            resource_role: resource.resource_role,
            drive_space_id: drive_space_id.to_string(),
            drive_node_id: drive_node_id.to_string(),
            sort_order: u32::try_from(sort_order)
                .map_err(|_| KernelError::validation("driveRefs sort order overflow"))?,
        });
    }
    Ok(normalized)
}

/// Stateless agent business service.
///
/// All methods take `&self` because `AgentRepository`, `AgentAuditSink`, and
/// `PolicyProvider` are all `Send + Sync`. This eliminates the global Mutex
/// bottleneck and allows true concurrent request processing. Optimistic
/// concurrency (version checks) ensures data integrity without serialization.
pub struct AgentsService<R, A, P>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    repository: R,
    audit_sink: A,
    policy_provider: P,
    turn_executor: Arc<dyn TurnExecutor>,
}

/// Outcome of claiming a provider Session identity for the canonical
/// `session.{engine}.*` Session during inventory synchronization.
pub(crate) enum ProviderSessionBindingClaim {
    /// No binding currently claims the provider Session identity.
    Free,
    /// The canonical target Session already owns the binding.
    AlreadyTarget,
    /// A provider-import Session (canonical `session.{engine}.*`, or a
    /// legacy `session.native.*` / `session.provider.*` Session from an
    /// older scheme or another project) claimed the identity and was
    /// retired (archived Session + released binding).
    Retired,
    /// A user-created Session owns the binding; the provider Session is
    /// already a live Session and must not be imported again.
    AlreadyBoundByUserSession,
}

impl<R, A, P> AgentsService<R, A, P>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    pub fn new(repository: R, audit_sink: A, policy_provider: P) -> Self {
        Self {
            repository,
            audit_sink,
            policy_provider,
            turn_executor: Arc::new(ContractTurnExecutor),
        }
    }

    /// Replace the default contract completer with a kernel-backed implementation
    /// at gateway bootstrap (production) without changing HTTP handlers.
    pub fn with_turn_executor(mut self, turn_executor: Arc<dyn TurnExecutor>) -> Self {
        self.turn_executor = turn_executor;
        self
    }

    /// Verify that the canonical Agents repository can serve requests.
    pub fn check_readiness(&self) -> KernelResult<()> {
        self.repository.check_readiness()
    }

    fn ensure_session_owner_scope(
        session: &AgentSessionRecord,
        owner_scope: Option<u64>,
    ) -> KernelResult<()> {
        if let Some(required_owner) = owner_scope {
            if session.owner_user_id != required_owner {
                return Err(KernelError::not_found("session not found"));
            }
        }
        Ok(())
    }

    fn ensure_project_owner_scope(
        project: &AgentProjectRecord,
        owner_scope: Option<u64>,
    ) -> KernelResult<()> {
        if let Some(required_owner) = owner_scope {
            if project.owner_user_id != required_owner {
                return Err(KernelError::not_found("project not found"));
            }
        }
        Ok(())
    }

    fn resolve_active_project_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        workspace_id: Option<&str>,
        requested_by: PolicySubject,
        requested_at: String,
    ) -> KernelResult<AgentWorkspaceRecord> {
        let workspace = if let Some(workspace_id) = workspace_id {
            validate_standard_id(workspace_id, "workspaceId", Some(ID_PREFIX_WORKSPACE))?;
            self.repository
                .get_workspace(tenant_id, organization_id, workspace_id)?
                .ok_or_else(|| KernelError::not_found("workspace not found"))?
        } else {
            self.ensure_default_workspace(EnsureDefaultWorkspaceCommand {
                tenant_id,
                organization_id,
                owner_user_id,
                default_name: None,
                requested_by,
                requested_at,
            })?
        };
        if workspace.owner_user_id != owner_user_id
            || workspace.status != AgentWorkspaceStatus::Active
        {
            return Err(KernelError::not_found("workspace not found"));
        }
        Ok(workspace)
    }

    fn load_active_project_for_composition(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        owner_scope: Option<u64>,
    ) -> KernelResult<AgentProjectRecord> {
        validate_standard_id(project_id, "projectId", Some(ID_PREFIX_PROJECT))?;
        let project = self
            .repository
            .get_project(tenant_id, organization_id, project_id)?
            .ok_or_else(|| KernelError::not_found("project not found"))?;
        Self::ensure_project_owner_scope(&project, owner_scope)?;
        if project.status != AgentProjectStatus::Active {
            return Err(KernelError::validation("project is not active"));
        }
        Ok(project)
    }

    fn ensure_task_owner_scope(
        task: &AgentTaskRecord,
        owner_scope: Option<u64>,
    ) -> KernelResult<()> {
        if let Some(required_owner) = owner_scope {
            if task.owner_user_id != required_owner {
                return Err(KernelError::not_found("task not found"));
            }
        }
        Ok(())
    }

    fn load_task_for_command_scope(
        &self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: &str,
        task_id: &str,
        owner_scope: Option<u64>,
    ) -> KernelResult<AgentTaskRecord> {
        validate_standard_id(task_id, "taskId", Some(ID_PREFIX_TASK))?;
        validate_agent_id(path_agent_id)?;
        let task = self
            .repository
            .get_task(tenant_id, organization_id, task_id)?
            .ok_or_else(|| KernelError::not_found("task not found"))?;
        Self::ensure_task_owner_scope(&task, owner_scope)?;
        Self::ensure_nested_agent_id(&task.agent_id, path_agent_id, "task")?;
        Ok(task)
    }

    fn ensure_task_run_scope(
        run: &AgentTaskRunRecord,
        task_id: &str,
        owner_scope: Option<u64>,
    ) -> KernelResult<()> {
        if run.task_id != task_id
            || owner_scope.is_some_and(|owner_user_id| run.owner_user_id != owner_user_id)
        {
            return Err(KernelError::not_found("task Run not found"));
        }
        Ok(())
    }

    fn ensure_reconciliation_matches_turn(
        &self,
        run: &AgentTaskRunRecord,
        outcome: TaskRunReconciliationOutcome,
    ) -> KernelResult<()> {
        if run.status != AgentTaskRunStatus::Reconciling {
            return Err(KernelError::validation("task Run is not reconciling"));
        }
        let turn_id = run
            .turn_id
            .as_deref()
            .ok_or_else(|| KernelError::conflict("task Run has no Turn identity"))?;
        let turn = self
            .repository
            .get_turn(run.tenant_id, run.organization_id, turn_id)?
            .ok_or_else(|| KernelError::conflict("task Run Turn not found"))?;
        let matches = matches!(
            (outcome, turn.status),
            (
                TaskRunReconciliationOutcome::Succeeded,
                AgentTurnStatus::Completed
            ) | (
                TaskRunReconciliationOutcome::Failed,
                AgentTurnStatus::Failed
            ) | (
                TaskRunReconciliationOutcome::Cancelled,
                AgentTurnStatus::Cancelled
            )
        );
        if !matches {
            return Err(KernelError::conflict(
                "requested reconciliation outcome does not match the canonical Turn",
            ));
        }
        Ok(())
    }

    fn ensure_nested_agent_id(
        record_agent_id: &str,
        path_agent_id: &str,
        resource_label: &str,
    ) -> KernelResult<()> {
        if record_agent_id != path_agent_id {
            return Err(KernelError::not_found(format!(
                "{resource_label} not found"
            )));
        }
        Ok(())
    }

    fn load_session_for_nested_route(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        path_agent_id: &str,
        owner_scope: Option<u64>,
    ) -> KernelResult<AgentSessionRecord> {
        let session = self
            .repository
            .get_session(tenant_id, organization_id, session_id)?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        Self::ensure_session_owner_scope(&session, owner_scope)?;
        Self::ensure_nested_agent_id(&session.agent_id, path_agent_id, "session")?;
        Ok(session)
    }

    pub fn create_agent(&self, command: CreateAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;

        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.create",
            command.requested_by.clone(),
            policy_resource,
            "create",
        )?;

        if self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .is_some()
        {
            return Err(KernelError::conflict("agent already exists"));
        }

        require_non_blank(command.code.as_str(), "code")?;
        require_non_blank(command.display_name.as_str(), "displayName")?;
        if let Some(provider_id) = command.implementation_provider_id.as_deref() {
            validate_standard_id(
                provider_id,
                "implementationProviderId",
                Some(ID_PREFIX_PROVIDER),
            )?;
        }

        let mut record = AgentBusinessRecord {
            id: self.repository.next_id()?,
            agent_id: command.agent_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: command.owner_user_id,
            code: command.code,
            display_name: command.display_name,
            description: command.description,
            manifest: command.manifest,
            default_code_task_intent: command.default_code_task_intent,
            implementation_provider_id: command.implementation_provider_id,
            implementation_kind: command.implementation_kind,
            implementation_type: command.implementation_type.unwrap_or_default(),
            status: AgentBusinessStatus::Draft,
            visibility: command.visibility,
            tags: command.tags,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            deleted_at: None,
        };
        record.mark_updated(command.requested_at.clone());

        self.repository.insert(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Create,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ensure_agent_engine_runtime_identity(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        engine_key: &str,
        agent_id: &str,
        binding_id: &str,
        provider_id: &str,
        requested_by: PolicySubject,
        requested_at: &str,
    ) -> KernelResult<()> {
        if sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key) != Some(agent_id)
            || sdkwork_agents_runtime_facade::agent_engine_binding_id(engine_key) != Some(binding_id)
        {
            return Err(KernelError::validation(
                "agent-engine runtime identity is not canonical",
            ));
        }
        validate_standard_id(provider_id, "providerId", Some(ID_PREFIX_PROVIDER))?;
        validate_requested_at(requested_at)?;

        let agent = self.repository.get(tenant_id, agent_id)?;
        match agent {
            Some(agent)
                if !agent.is_deleted()
                    && agent.implementation_provider_id.as_deref() == Some(provider_id) => {}
            Some(_) => {
                return Err(KernelError::validation(
                    "canonical agent-engine agent identity conflicts with existing agent",
                ));
            }
            None => {
                let mut record = AgentBusinessRecord {
                    id: self.repository.next_id()?,
                    agent_id: agent_id.to_string(),
                    tenant_id,
                    organization_id,
                    owner_user_id,
                    code: engine_key.to_string(),
                    display_name: engine_key.to_string(),
                    description: Some("Canonical local agent-engine runtime identity".to_string()),
                    manifest: agent_engine_runtime_manifest(engine_key, agent_id),
                    default_code_task_intent: None,
                    implementation_provider_id: Some(provider_id.to_string()),
                    implementation_kind: Some(AgentImplementationKind::TypedLocalProvider),
                    implementation_type: AgentImplementationType::SdkworkNative,
                    status: AgentBusinessStatus::Active,
                    visibility: AgentVisibility::Private,
                    tags: vec!["agent-engine".to_string(), engine_key.to_string()],
                    version: 0,
                    created_at: requested_at.to_string(),
                    updated_at: requested_at.to_string(),
                    deleted_at: None,
                };
                record.mark_updated(requested_at.to_string());
                match self.repository.insert(record.clone()) {
                    Ok(()) => {
                        self.emit_audit_event(
                            AgentAuditAction::Create,
                            &record,
                            None,
                            requested_by.clone(),
                            requested_at.to_string(),
                        )?;
                    }
                    Err(error) if error.kind() == KernelErrorKind::Conflict => {
                        let concurrent = self.repository.get(tenant_id, agent_id)?.ok_or(error)?;
                        if concurrent.is_deleted()
                            || concurrent.implementation_provider_id.as_deref() != Some(provider_id)
                        {
                            return Err(KernelError::validation(
                                "concurrent canonical agent-engine agent identity conflicts",
                            ));
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        match self
            .repository
            .get_provider_binding(tenant_id, agent_id, binding_id)?
        {
            Some(binding) if binding.active && binding.provider_id == provider_id => Ok(()),
            Some(_) => Err(KernelError::validation(
                "canonical agent-engine provider binding conflicts with existing binding",
            )),
            None => {
                let binding = AgentProviderBindingRecord {
                    id: self.repository.next_id()?,
                    tenant_id,
                    agent_id: agent_id.to_string(),
                    binding_id: binding_id.to_string(),
                    provider_id: provider_id.to_string(),
                    implementation_kind: AgentImplementationKind::TypedLocalProvider,
                    configuration_profile_id: format!("{ID_PREFIX_PROFILE}provider.{engine_key}"),
                    capabilities: vec!["model.chat".to_string(), "session.resume".to_string()],
                    active: true,
                    version: 1,
                    created_at: requested_at.to_string(),
                    updated_at: requested_at.to_string(),
                };
                match self.repository.insert_provider_binding(binding.clone()) {
                    Ok(()) => {
                        self.emit_binding_audit_event(
                            AgentAuditAction::ProviderBindingChanged,
                            &binding,
                            requested_by,
                            requested_at.to_string(),
                        )?;
                        Ok(())
                    }
                    Err(error) if error.kind() == KernelErrorKind::Conflict => {
                        let concurrent = self
                            .repository
                            .get_provider_binding(tenant_id, agent_id, binding_id)?
                            .ok_or(error)?;
                        if concurrent.active && concurrent.provider_id == provider_id {
                            Ok(())
                        } else {
                            Err(KernelError::validation(
                                "concurrent canonical agent-engine provider binding conflicts",
                            ))
                        }
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    pub fn add_provider_binding(
        &self,
        command: AgentProviderBindingCommand,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.add",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "provider_binding.add",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;

        validate_standard_id(
            command.binding_id.as_str(),
            "bindingId",
            Some(ID_PREFIX_BINDING),
        )?;
        validate_standard_id(
            command.provider_id.as_str(),
            "providerId",
            Some(ID_PREFIX_PROVIDER),
        )?;
        validate_standard_id(
            command.configuration_profile_id.as_str(),
            "configurationProfileId",
            Some(ID_PREFIX_PROFILE),
        )?;
        validate_capabilities(command.capabilities.as_slice(), "capabilities")?;

        if self
            .repository
            .get_provider_binding(
                command.tenant_id,
                command.agent_id.as_str(),
                command.binding_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "agent provider binding already exists",
            ));
        }

        if command.make_default {
            self.deactivate_provider_bindings(
                command.tenant_id,
                command.agent_id.as_str(),
                command.requested_at.clone(),
            )?;
        }

        let record = AgentProviderBindingRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            agent_id: command.agent_id.clone(),
            binding_id: command.binding_id,
            provider_id: command.provider_id,
            implementation_kind: command.implementation_kind,
            configuration_profile_id: command.configuration_profile_id,
            capabilities: command.capabilities,
            active: command.make_default,
            version: 1,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
        };

        self.repository.insert_provider_binding(record.clone())?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn activate_provider_binding(
        &self,
        command: ActivateAgentProviderBindingCommand,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.activate",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "provider_binding.activate",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        validate_standard_id(
            command.binding_id.as_str(),
            "bindingId",
            Some(ID_PREFIX_BINDING),
        )?;

        let record = self.repository.activate_provider_binding_atomic(
            command.tenant_id,
            command.agent_id.as_str(),
            command.binding_id.as_str(),
            command.requested_at.clone(),
        )?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    fn all_provider_bindings_for_agent(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        let mut all = Vec::new();
        let mut page = 1usize;
        loop {
            let batch = self.repository.list_provider_bindings(
                &ProviderBindingListQuery::for_agent(tenant_id, agent_id).with_pagination(
                    PaginationParams::default()
                        .with_page_size(MAX_PAGE_SIZE)
                        .with_page(page),
                ),
            )?;
            if batch.is_empty() {
                break;
            }
            let batch_len = batch.len();
            all.extend(batch);
            if batch_len < MAX_PAGE_SIZE {
                break;
            }
            page = page.saturating_add(1);
        }
        Ok(all)
    }

    pub fn list_provider_bindings(
        &self,
        command: ProviderBindingListCommand,
    ) -> KernelResult<PaginatedResult<AgentProviderBindingRecord>> {
        validate_agent_id(command.query.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.list",
            command.requested_by,
            format!("agent.business.{}", command.query.agent_id),
            "provider_binding.list",
        )?;
        self.repository
            .get(command.query.tenant_id, command.query.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        let total_count = self.repository.count_provider_bindings(&command.query)?;
        let items = self.repository.list_provider_bindings(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.provider_binding.retrieve",
            requested_by,
            format!("agent.business.{}", agent_id),
            "provider_binding.retrieve",
        )?;
        self.repository
            .get(tenant_id, agent_id)?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        self.repository
            .get_provider_binding(tenant_id, agent_id, binding_id)?
            .ok_or_else(|| KernelError::not_found("provider binding not found"))
    }

    pub fn deactivate_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        requested_at: String,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.provider_binding.deactivate",
            requested_by.clone(),
            format!("agent.business.{}", agent_id),
            "provider_binding.deactivate",
        )?;
        let mut record = self
            .repository
            .get_provider_binding(tenant_id, agent_id, binding_id)?
            .ok_or_else(|| KernelError::not_found("provider binding not found"))?;
        if !record.active {
            return Err(KernelError::validation(
                "provider binding is already inactive",
            ));
        }
        record.active = false;
        record.mark_updated(requested_at.as_str());
        self.repository.update_provider_binding(record.clone())?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            requested_by,
            record.updated_at.clone(),
        )?;
        Ok(record)
    }

    pub fn update_session_metadata(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        title: Option<String>,
        requested_at: String,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentSessionRecord> {
        self.authorize(
            "agent.business.session.update",
            requested_by,
            format!("agent.business.session.{}", session_id),
            "session.update",
        )?;
        let mut record = self
            .repository
            .get_session(tenant_id, organization_id, session_id)?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        if let Some(title) = title {
            record.title = Some(title);
        }
        record.mark_updated(requested_at.as_str());
        self.repository.update_session(record.clone())?;
        Ok(record)
    }

    pub fn list_agent_engine_catalog(
        &self,
        requested_by: PolicySubject,
    ) -> KernelResult<sdkwork_agents_runtime_facade::AgentEngineCatalog> {
        self.authorize(
            "agent.business.agent_engine.list",
            requested_by,
            "agent.business".to_string(),
            "agent_engine.list",
        )?;
        Ok(crate::agent_engine_catalog::list_agent_engine_catalog())
    }

    pub fn list_mcp_marketplace(
        &self,
        command: ListMcpMarketplaceCommand,
    ) -> KernelResult<PaginatedResult<crate::mcp_marketplace::McpServerMarketplaceRecord>> {
        self.authorize(
            "agent.business.mcp_server.list",
            command.requested_by,
            "agent.business".to_string(),
            "mcp_server.list",
        )?;
        let total_count = self
            .repository
            .count_mcp_marketplace_slots(&command.query)?;
        let slots = self.repository.list_mcp_marketplace_slots(&command.query)?;
        let items = slots
            .iter()
            .map(crate::mcp_marketplace::project_mcp_slot)
            .collect();
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn create_preview_response(
        &self,
        command: AgentPreviewResponseCommand,
    ) -> KernelResult<AgentRuntimeExecutionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.runtime.preview_response",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "runtime.preview_response",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        validate_standard_id(
            command.execution_id.as_str(),
            "executionId",
            Some(ID_PREFIX_EXECUTION),
        )?;
        validate_non_empty(command.content.as_str(), "content")?;
        validate_json_payload(command.input_payload_json.as_str(), "inputPayload")?;
        if let Some(model) = command.model.as_deref() {
            validate_optional_plain_ref(Some(model), "model")?;
        }
        if let Some(temperature) = command.temperature {
            if !(0.0..=2.0).contains(&temperature) || !temperature.is_finite() {
                return Err(KernelError::validation(
                    "temperature must be between 0 and 2",
                ));
            }
        }

        let active_binding = self
            .repository
            .get_active_provider_binding(command.tenant_id, command.agent_id.as_str())?;

        let preview = execute_preview_response(
            active_binding.as_ref(),
            command.content.as_str(),
            command.model.as_deref(),
        );
        if preview.runtime_mode == RUNTIME_MODE_CONTRACT_FALLBACK
            && !agents_allow_contract_runtime_fallback()
        {
            return Err(KernelError::validation(
                "preview response requires an active agent-engine provider binding",
            ));
        }

        let output_payload_json = serde_json::json!({
            "content": preview.content,
            "debugMode": command.debug_mode,
            "model": preview.model_id,
            "temperature": command.temperature,
            "runtimeMode": preview.runtime_mode,
        })
        .to_string();

        let record = AgentRuntimeExecutionRecord {
            tenant_id: command.tenant_id,
            agent_id: command.agent_id,
            execution_id: command.execution_id,
            operation: AgentRuntimeExecutionOperation::PreviewResponse,
            status: AgentRuntimeExecutionStatus::Completed,
            input_payload_json: command.input_payload_json,
            output_payload_json,
            requested_at: command.requested_at.clone(),
            completed_at: command.requested_at.clone(),
        };

        self.emit_runtime_execution_audit_event(
            AgentAuditAction::RuntimeExecutionCompleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn create_prompt_optimization(
        &self,
        command: AgentPromptOptimizationCommand,
    ) -> KernelResult<AgentRuntimeExecutionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.runtime.prompt_optimization",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "runtime.prompt_optimization",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        validate_standard_id(
            command.execution_id.as_str(),
            "executionId",
            Some(ID_PREFIX_EXECUTION),
        )?;
        validate_non_empty(command.prompt.as_str(), "prompt")?;
        validate_json_payload(command.input_payload_json.as_str(), "inputPayload")?;

        let active_binding = self
            .repository
            .get_active_provider_binding(command.tenant_id, command.agent_id.as_str())?;

        let optimization =
            execute_prompt_optimization(active_binding.as_ref(), command.prompt.as_str());
        if optimization.runtime_mode == RUNTIME_MODE_CONTRACT_FALLBACK
            && !agents_allow_contract_runtime_fallback()
        {
            return Err(KernelError::validation(
                "prompt optimization requires an active agent-engine provider binding",
            ));
        }

        let output_payload_json = serde_json::json!({
            "optimizedPrompt": optimization.optimized_prompt,
            "runtimeMode": optimization.runtime_mode,
        })
        .to_string();

        let record = AgentRuntimeExecutionRecord {
            tenant_id: command.tenant_id,
            agent_id: command.agent_id,
            execution_id: command.execution_id,
            operation: AgentRuntimeExecutionOperation::PromptOptimization,
            status: AgentRuntimeExecutionStatus::Completed,
            input_payload_json: command.input_payload_json,
            output_payload_json,
            requested_at: command.requested_at.clone(),
            completed_at: command.requested_at.clone(),
        };

        self.emit_runtime_execution_audit_event(
            AgentAuditAction::RuntimeExecutionCompleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn create_composition_slot(
        &self,
        command: AgentCompositionSlotCreateCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.create",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.create",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some(ID_PREFIX_SLOT))?;
        validate_composition_slot_mapping(command.slot_kind, command.target_module)?;
        require_non_blank(command.target_ref.as_str(), "targetRef")?;
        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        if self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("composition slot already exists"));
        }
        let record = AgentCompositionSlotRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id,
            slot_id: command.slot_id,
            slot_kind: command.slot_kind,
            target_module: command.target_module,
            target_ref: command.target_ref,
            target_version_ref: command.target_version_ref,
            priority: command.priority,
            enabled: command.enabled,
            policy_json: command.policy_json,
            status: AgentBusinessStatus::Active,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            deleted_at: None,
        };
        self.repository.insert_composition_slot(record.clone())?;
        self.emit_marketplace_audit_event(AgentBusinessAuditEventInput {
            action: AgentAuditAction::CompositionSlotCreated,
            item_kind: "composition_slot",
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.slot_id.as_str(),
            status: record.status,
            visibility: AgentVisibility::Tenant,
            version: record.version,
            subject: command.requested_by,
            occurred_at: command.requested_at,
        })?;
        Ok(record)
    }

    pub fn list_composition_slots(
        &self,
        command: AgentCompositionSlotListCommand,
    ) -> KernelResult<PaginatedResult<AgentCompositionSlotRecord>> {
        self.authorize(
            "agent.business.composition_slot.list",
            command.requested_by,
            format!("agent.business.{}", command.query.agent_id),
            "composition_slot.list",
        )?;
        validate_agent_id(command.query.agent_id.as_str())?;
        self.repository
            .get(command.query.tenant_id, command.query.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        let total_count = self.repository.count_composition_slots(&command.query)?;
        let items = self.repository.list_composition_slots(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_composition_slot(
        &self,
        command: AgentCompositionSlotGetCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.retrieve",
            command.requested_by,
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.retrieve",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some(ID_PREFIX_SLOT))?;
        self.repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("composition slot not found"))
    }

    pub fn update_composition_slot(
        &self,
        command: AgentCompositionSlotUpdateCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.update",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.update",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some(ID_PREFIX_SLOT))?;
        let mut record = self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("composition slot not found"))?;
        if record.is_deleted() {
            return Err(KernelError::validation("composition slot is deleted"));
        }
        if let Some(expected_version) = command.expected_version {
            if record.version != expected_version {
                return Err(KernelError::conflict(format!(
                    "composition slot version mismatch: expected={expected_version}, actual={}",
                    record.version
                )));
            }
        }
        if let Some(slot_kind) = command.slot_kind {
            record.slot_kind = slot_kind;
        }
        if let Some(target_module) = command.target_module {
            record.target_module = target_module;
        }
        validate_composition_slot_mapping(record.slot_kind, record.target_module)?;
        if let Some(target_ref) = command.target_ref {
            require_non_blank(target_ref.as_str(), "targetRef")?;
            record.target_ref = target_ref;
        }
        if let Some(target_version_ref) = command.target_version_ref {
            record.target_version_ref = target_version_ref;
        }
        if let Some(priority) = command.priority {
            record.priority = priority;
        }
        if let Some(enabled) = command.enabled {
            record.enabled = enabled;
        }
        if let Some(policy_json) = command.policy_json {
            record.policy_json = policy_json;
        }
        record.mark_updated(command.requested_at.clone());
        self.repository.update_composition_slot(record.clone())?;
        self.emit_marketplace_audit_event(AgentBusinessAuditEventInput {
            action: AgentAuditAction::CompositionSlotUpdated,
            item_kind: "composition_slot",
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.slot_id.as_str(),
            status: record.status,
            visibility: AgentVisibility::Tenant,
            version: record.version,
            subject: command.requested_by,
            occurred_at: command.requested_at,
        })?;
        Ok(record)
    }

    pub fn delete_composition_slot(
        &self,
        command: AgentCompositionSlotDeleteCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.delete",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.delete",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some(ID_PREFIX_SLOT))?;
        let mut record = self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("composition slot not found"))?;
        if record.is_deleted() {
            return Err(KernelError::validation("composition slot already deleted"));
        }
        if let Some(expected_version) = command.expected_version {
            if record.version != expected_version {
                return Err(KernelError::conflict(format!(
                    "composition slot version mismatch: expected={expected_version}, actual={}",
                    record.version
                )));
            }
        }
        record.mark_deleted(command.requested_at.clone());
        self.repository.update_composition_slot(record.clone())?;
        self.emit_marketplace_audit_event(AgentBusinessAuditEventInput {
            action: AgentAuditAction::CompositionSlotDeleted,
            item_kind: "composition_slot",
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.slot_id.as_str(),
            status: record.status,
            visibility: AgentVisibility::Tenant,
            version: record.version,
            subject: command.requested_by,
            occurred_at: command.requested_at,
        })?;
        Ok(record)
    }

    pub fn update_agent(&self, command: UpdateAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.update",
            command.requested_by.clone(),
            policy_resource,
            "update",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation("deleted agent cannot be updated"));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

        if let Some(display_name) = command.display_name {
            require_non_blank(display_name.as_str(), "displayName")?;
            record.display_name = display_name;
        }
        if let Some(description) = command.description {
            record.description = Some(description);
        }
        if let Some(manifest) = command.manifest {
            record.manifest = manifest;
        }
        if let Some(visibility) = command.visibility {
            record.visibility = visibility;
        }
        if let Some(tags) = command.tags {
            record.tags = tags;
        }
        if let Some(intent) = command.default_code_task_intent {
            record.default_code_task_intent = Some(intent);
        }
        if let Some(provider_id) = command.implementation_provider_id {
            if let Some(provider_id) = provider_id.as_deref() {
                validate_standard_id(
                    provider_id,
                    "implementationProviderId",
                    Some(ID_PREFIX_PROVIDER),
                )?;
            }
            record.implementation_provider_id = provider_id;
        }
        if let Some(implementation_kind) = command.implementation_kind {
            record.implementation_kind = implementation_kind;
        }
        if let Some(implementation_type) = command.implementation_type {
            record.implementation_type = implementation_type;
        }
        record.mark_updated(command.requested_at.clone());

        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Update,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn change_status(
        &self,
        command: ChangeAgentStatusCommand,
    ) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.status.update",
            command.requested_by.clone(),
            policy_resource,
            "change_status",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation(
                "deleted agent status cannot be changed",
            ));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

        if !is_valid_status_transition(record.status, command.target_status) {
            return Err(KernelError::validation("invalid agent status transition"));
        }

        let previous_status = record.status;
        record.status = command.target_status;
        record.mark_updated(command.requested_at.clone());
        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::ChangeStatus,
            &record,
            Some(previous_status),
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn delete_agent(&self, command: DeleteAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.delete",
            command.requested_by.clone(),
            policy_resource,
            "delete",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation("agent already deleted"));
        }
        if let Some(expected_version) = command.expected_version {
            if record.version != expected_version {
                return Err(KernelError::conflict(format!(
                    "agent version mismatch: expected={expected_version}, actual={}",
                    record.version
                )));
            }
        }

        record.mark_deleted(command.requested_at.clone());
        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Delete,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn restore_agent(&self, command: RestoreAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.restore",
            command.requested_by.clone(),
            policy_resource,
            "restore",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;

        if !record.is_deleted() {
            return Err(KernelError::validation("agent is not deleted"));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

        record.mark_restored(command.requested_at.clone());
        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Restore,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_agent(&self, command: GetAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.retrieve",
            command.requested_by,
            policy_resource,
            "retrieve",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))
            .and_then(|record| {
                if record.is_deleted() {
                    return Err(KernelError::not_found("agent not found"));
                }
                Ok(record)
            })
    }

    pub fn list_agents(
        &self,
        command: ListAgentsCommand,
    ) -> KernelResult<PaginatedResult<AgentBusinessRecord>> {
        self.authorize(
            "agent.business.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "list",
        )?;
        self.repository.list_paginated(&command.query)
    }

    pub fn list_agent_audit_events(
        &self,
        command: ListAgentAuditEventsCommand,
    ) -> KernelResult<PaginatedResult<KernelEvent>> {
        validate_agent_id(command.query.agent_id.as_str())?;
        self.authorize(
            "agent.business.audit.read",
            command.requested_by,
            format!("agent.business.{}", command.query.agent_id),
            "audit.read",
        )?;
        self.audit_sink.list_events(&command.query)
    }

    // -----------------------------------------------------------------------
    // Project management
    // -----------------------------------------------------------------------

    pub fn ensure_default_workspace(
        &self,
        command: EnsureDefaultWorkspaceCommand,
    ) -> KernelResult<AgentWorkspaceRecord> {
        self.authorize(
            "agent.business.workspace.ensure_default",
            command.requested_by.clone(),
            format!("agent.business.workspace.owner.{}", command.owner_user_id),
            "workspace.ensureDefault",
        )?;
        if let Some(existing) = self.repository.get_default_workspace(
            command.tenant_id,
            command.organization_id,
            command.owner_user_id,
        )? {
            return Ok(existing);
        }
        let workspace_id = default_workspace_id(command.owner_user_id);
        if let Some(existing) = self.repository.get_workspace(
            command.tenant_id,
            command.organization_id,
            &workspace_id,
        )? {
            return Ok(existing);
        }
        let name = command
            .default_name
            .as_deref()
            .map(trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "Workspace".to_string());
        let record = AgentWorkspaceRecord {
            id: self.repository.next_id()?,
            workspace_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: command.owner_user_id,
            name,
            description: None,
            is_default: true,
            status: AgentWorkspaceStatus::Active,
            created_by: command.owner_user_id,
            updated_by: command.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        match self.repository.insert_workspace(record.clone()) {
            Ok(()) => {
                self.emit_workspace_audit_event(
                    AgentAuditAction::WorkspaceCreated,
                    &record,
                    command.requested_by,
                    record.created_at.clone(),
                )?;
                Ok(record)
            }
            Err(error) if error.kind() == KernelErrorKind::Conflict => self
                .repository
                .get_default_workspace(
                    command.tenant_id,
                    command.organization_id,
                    command.owner_user_id,
                )?
                .ok_or(error),
            Err(error) => Err(error),
        }
    }

    pub fn create_workspace(
        &self,
        command: CreateWorkspaceCommand,
    ) -> KernelResult<AgentWorkspaceRecord> {
        self.authorize(
            "agent.business.workspace.create",
            command.requested_by.clone(),
            format!("agent.business.workspace.owner.{}", command.owner_user_id),
            "workspace.create",
        )?;
        let name = normalized_workspace_name(&command.name)?;
        let id = self.repository.next_id()?;
        let record = AgentWorkspaceRecord {
            id,
            workspace_id: format!("{ID_PREFIX_WORKSPACE}{id}"),
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: command.owner_user_id,
            name,
            description: command.description,
            is_default: false,
            status: AgentWorkspaceStatus::Active,
            created_by: command.owner_user_id,
            updated_by: command.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        self.repository.insert_workspace(record.clone())?;
        self.emit_workspace_audit_event(
            AgentAuditAction::WorkspaceCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_workspace(
        &self,
        command: GetWorkspaceCommand,
    ) -> KernelResult<AgentWorkspaceRecord> {
        self.authorize(
            "agent.business.workspace.retrieve",
            command.requested_by,
            format!("agent.business.workspace.{}", command.workspace_id),
            "workspace.retrieve",
        )?;
        validate_standard_id(
            &command.workspace_id,
            "workspaceId",
            Some(ID_PREFIX_WORKSPACE),
        )?;
        let record = self
            .repository
            .get_workspace(
                command.tenant_id,
                command.organization_id,
                &command.workspace_id,
            )?
            .ok_or_else(|| KernelError::not_found("workspace not found"))?;
        Self::ensure_workspace_owner_scope(&record, command.owner_user_id)?;
        Ok(record)
    }

    pub fn update_workspace(
        &self,
        command: UpdateWorkspaceCommand,
    ) -> KernelResult<AgentWorkspaceRecord> {
        self.authorize(
            "agent.business.workspace.update",
            command.requested_by.clone(),
            format!("agent.business.workspace.{}", command.workspace_id),
            "workspace.update",
        )?;
        let mut record = self.load_workspace_for_owner(
            command.tenant_id,
            command.organization_id,
            &command.workspace_id,
            command.owner_user_id,
        )?;
        ensure_expected_version(record.version, command.expected_version, "workspace")?;
        if record.status != AgentWorkspaceStatus::Active {
            return Err(KernelError::validation("workspace is not active"));
        }
        if let Some(name) = command.name {
            record.name = normalized_workspace_name(&name)?;
        }
        if let Some(description) = command.description {
            record.description = description;
        }
        record.mark_updated(command.owner_user_id, command.requested_at.clone());
        self.repository.update_workspace(record.clone())?;
        self.emit_workspace_audit_event(
            AgentAuditAction::WorkspaceUpdated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn archive_workspace(
        &self,
        command: WorkspaceMutationCommand,
    ) -> KernelResult<AgentWorkspaceRecord> {
        self.mutate_workspace_status(command, AgentWorkspaceStatus::Archived)
    }

    pub fn delete_workspace(
        &self,
        command: WorkspaceMutationCommand,
    ) -> KernelResult<AgentWorkspaceRecord> {
        self.mutate_workspace_status(command, AgentWorkspaceStatus::Deleted)
    }

    fn mutate_workspace_status(
        &self,
        command: WorkspaceMutationCommand,
        target: AgentWorkspaceStatus,
    ) -> KernelResult<AgentWorkspaceRecord> {
        let (request_id, action, audit_action) = match target {
            AgentWorkspaceStatus::Archived => (
                "agent.business.workspace.archive",
                "workspace.archive",
                AgentAuditAction::WorkspaceArchived,
            ),
            AgentWorkspaceStatus::Deleted => (
                "agent.business.workspace.delete",
                "workspace.delete",
                AgentAuditAction::WorkspaceDeleted,
            ),
            AgentWorkspaceStatus::Active => {
                return Err(KernelError::validation("unsupported workspace transition"));
            }
        };
        self.authorize(
            request_id,
            command.requested_by.clone(),
            format!("agent.business.workspace.{}", command.workspace_id),
            action,
        )?;
        let mut record = self.load_workspace_for_owner(
            command.tenant_id,
            command.organization_id,
            &command.workspace_id,
            command.owner_user_id,
        )?;
        ensure_expected_version(record.version, command.expected_version, "workspace")?;
        if record.is_default {
            return Err(KernelError::validation(
                "default workspace cannot be archived or deleted",
            ));
        }
        if target == AgentWorkspaceStatus::Archived && record.status != AgentWorkspaceStatus::Active
        {
            return Err(KernelError::validation("workspace is not active"));
        }
        if target == AgentWorkspaceStatus::Deleted
            && !matches!(
                record.status,
                AgentWorkspaceStatus::Active | AgentWorkspaceStatus::Archived
            )
        {
            return Err(KernelError::validation("workspace cannot be deleted"));
        }
        let projects = self.repository.count_projects(
            &crate::ports::ProjectListQuery::for_organization(
                command.tenant_id,
                command.organization_id,
            )
            .for_owner(command.owner_user_id)
            .for_workspace(&command.workspace_id),
        )?;
        if projects > 0 {
            return Err(KernelError::conflict(
                "workspace contains projects and cannot be archived or deleted",
            ));
        }
        match target {
            AgentWorkspaceStatus::Archived => {
                record.archive(command.owner_user_id, command.requested_at.clone())
            }
            AgentWorkspaceStatus::Deleted => {
                record.soft_delete(command.owner_user_id, command.requested_at.clone())
            }
            AgentWorkspaceStatus::Active => unreachable!(),
        }
        self.repository.update_workspace(record.clone())?;
        self.emit_workspace_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    fn load_workspace_for_owner(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<AgentWorkspaceRecord> {
        validate_standard_id(workspace_id, "workspaceId", Some(ID_PREFIX_WORKSPACE))?;
        let record = self
            .repository
            .get_workspace(tenant_id, organization_id, workspace_id)?
            .ok_or_else(|| KernelError::not_found("workspace not found"))?;
        Self::ensure_workspace_owner_scope(&record, owner_user_id)?;
        Ok(record)
    }

    fn ensure_workspace_owner_scope(
        record: &AgentWorkspaceRecord,
        owner_user_id: u64,
    ) -> KernelResult<()> {
        if record.owner_user_id != owner_user_id {
            return Err(KernelError::not_found("workspace not found"));
        }
        Ok(())
    }

    pub fn list_workspaces(
        &self,
        command: ListWorkspacesCommand,
    ) -> KernelResult<PaginatedResult<AgentWorkspaceRecord>> {
        self.authorize(
            "agent.business.workspace.list",
            command.requested_by,
            format!(
                "agent.business.workspace.owner.{}",
                command.query.owner_user_id
            ),
            "workspace.list",
        )?;
        let total_count = self.repository.count_workspaces(&command.query)?;
        let items = self.repository.list_workspaces(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn create_project(
        &self,
        command: CreateProjectCommand,
    ) -> KernelResult<AgentProjectRecord> {
        let project_id = if is_trimmed_blank(command.project_id.as_str()) {
            format!("{ID_PREFIX_PROJECT}{}", self.repository.next_id()?)
        } else {
            command.project_id.clone()
        };
        validate_standard_id(project_id.as_str(), "projectId", Some(ID_PREFIX_PROJECT))?;
        require_non_blank(command.name.as_str(), "name")?;
        if command.name.len() > 255 {
            return Err(KernelError::validation("name exceeds 255 bytes"));
        }
        validate_project_drive_access(command.visibility, command.drive_access_mode)?;
        self.authorize(
            "agent.business.project.create",
            command.requested_by.clone(),
            format!("agent.business.project.{project_id}"),
            "project.create",
        )?;
        let workspace = self.resolve_active_project_workspace(
            command.tenant_id,
            command.organization_id,
            command.owner_user_id,
            command.workspace_id.as_deref(),
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;
        if self
            .repository
            .get_project_by_workspace_name(
                command.tenant_id,
                command.organization_id,
                &workspace.workspace_id,
                &command.name,
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "project name already exists in workspace",
            ));
        }
        if let Some(agent_id) = command.default_agent_id.as_deref() {
            validate_agent_id(agent_id)?;
            let agent = self
                .repository
                .get(command.tenant_id, agent_id)?
                .ok_or_else(|| KernelError::not_found("default agent not found"))?;
            if agent.organization_id != command.organization_id {
                return Err(KernelError::not_found("default agent not found"));
            }
        }
        if self
            .repository
            .get_project(command.tenant_id, command.organization_id, &project_id)?
            .is_some()
        {
            return Err(KernelError::conflict("project already exists"));
        }
        let record = AgentProjectRecord {
            id: self.repository.next_id()?,
            project_id,
            workspace_id: workspace.workspace_id.clone(),
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: command.owner_user_id,
            name: trim(command.name.as_str()).to_string(),
            description: command.description,
            visibility: command.visibility,
            status: AgentProjectStatus::Active,
            drive_access_mode: command.drive_access_mode,
            default_agent_id: command.default_agent_id,
            default_model_id: command.default_model_id,
            import_source_kind: None,
            import_source_ref: None,
            drive_space_id: None,
            drive_root_entry_id: None,
            drive_logical_path: None,
            created_by: command.owner_user_id,
            updated_by: command.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        self.repository.insert_project(record.clone())?;
        self.emit_project_audit_event(
            AgentAuditAction::ProjectCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn import_project(
        &self,
        command: ImportProjectCommand,
    ) -> KernelResult<AgentProjectRecord> {
        let source_kind = trim(&command.source_kind).to_string();
        let source_ref = trim(&command.source_ref).to_string();
        let drive_space_id = trim(&command.drive_space_id).to_string();
        let drive_root_entry_id = trim(&command.drive_root_entry_id).to_string();
        let drive_logical_path = trim(&command.drive_logical_path).to_string();
        require_non_blank(&source_kind, "sourceKind")?;
        require_non_blank(&source_ref, "sourceRef")?;
        require_non_blank(&drive_space_id, "driveSpaceId")?;
        require_non_blank(&drive_root_entry_id, "driveRootEntryId")?;
        if source_kind.len() > 64 {
            return Err(KernelError::validation("sourceKind exceeds 64 bytes"));
        }
        if source_ref.len() > 512 {
            return Err(KernelError::validation("sourceRef exceeds 512 bytes"));
        }
        if let Some(existing) = self.repository.get_project_by_import_source(
            command.tenant_id,
            command.organization_id,
            command.owner_user_id,
            &source_kind,
            &source_ref,
        )? {
            if existing.workspace_id != command.workspace_id {
                return Err(KernelError::conflict(
                    "import source is already assigned to another workspace",
                ));
            }
            return Ok(existing);
        }
        let project_id = if is_trimmed_blank(command.project_id.as_str()) {
            format!("{ID_PREFIX_PROJECT}{}", self.repository.next_id()?)
        } else {
            command.project_id.clone()
        };
        validate_standard_id(project_id.as_str(), "projectId", Some(ID_PREFIX_PROJECT))?;
        require_non_blank(command.name.as_str(), "name")?;
        if command.name.len() > 255 {
            return Err(KernelError::validation("name exceeds 255 bytes"));
        }
        self.authorize(
            "agent.business.project.import",
            command.requested_by.clone(),
            format!("agent.business.project.{project_id}"),
            "project.create",
        )?;
        let workspace = self.resolve_active_project_workspace(
            command.tenant_id,
            command.organization_id,
            command.owner_user_id,
            Some(&command.workspace_id),
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;
        if let Some(existing) = self.repository.get_project_by_workspace_name(
            command.tenant_id,
            command.organization_id,
            &workspace.workspace_id,
            &command.name,
        )? {
            return Ok(existing);
        }
        if self
            .repository
            .get_project(command.tenant_id, command.organization_id, &project_id)?
            .is_some()
        {
            return Err(KernelError::conflict("project already exists"));
        }
        let record = AgentProjectRecord {
            id: self.repository.next_id()?,
            project_id,
            workspace_id: workspace.workspace_id.clone(),
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: command.owner_user_id,
            name: trim(command.name.as_str()).to_string(),
            description: command.description,
            visibility: AgentProjectVisibility::Private,
            status: AgentProjectStatus::Active,
            drive_access_mode: AgentProjectDriveAccessMode::ExplicitResources,
            default_agent_id: None,
            default_model_id: None,
            import_source_kind: Some(source_kind.clone()),
            import_source_ref: Some(source_ref.clone()),
            drive_space_id: Some(drive_space_id),
            drive_root_entry_id: Some(drive_root_entry_id),
            drive_logical_path: Some(drive_logical_path),
            created_by: command.owner_user_id,
            updated_by: command.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        match self.repository.insert_project(record.clone()) {
            Ok(()) => {
                self.emit_project_audit_event(
                    AgentAuditAction::ProjectCreated,
                    &record,
                    command.requested_by,
                    command.requested_at,
                )?;
                Ok(record)
            }
            Err(error) if error.kind() == KernelErrorKind::Conflict => {
                let existing_by_source = self.repository.get_project_by_import_source(
                    command.tenant_id,
                    command.organization_id,
                    command.owner_user_id,
                    &source_kind,
                    &source_ref,
                )?;
                match existing_by_source {
                    Some(existing) if existing.workspace_id == command.workspace_id => Ok(existing),
                    Some(_) => Err(KernelError::conflict(
                        "import source is already assigned to another workspace",
                    )),
                    None => self
                        .repository
                        .get_project_by_workspace_name(
                            command.tenant_id,
                            command.organization_id,
                            &workspace.workspace_id,
                            &command.name,
                        )?
                        .ok_or(error),
                }
            }
            Err(error) => Err(error),
        }
    }

    pub fn update_project(
        &self,
        command: UpdateProjectCommand,
    ) -> KernelResult<AgentProjectRecord> {
        validate_standard_id(&command.project_id, "projectId", Some(ID_PREFIX_PROJECT))?;
        self.authorize(
            "agent.business.project.update",
            command.requested_by.clone(),
            format!("agent.business.project.{}", command.project_id),
            "project.update",
        )?;
        let mut record = self
            .repository
            .get_project(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
            )?
            .ok_or_else(|| KernelError::not_found("project not found"))?;
        Self::ensure_project_owner_scope(&record, command.owner_scope)?;
        ensure_expected_version(record.version, command.expected_version, "project")?;
        if record.status != AgentProjectStatus::Active {
            return Err(KernelError::validation("project is not active"));
        }
        if let Some(name) = command.name {
            require_non_blank(&name, "name")?;
            if name.len() > 255 {
                return Err(KernelError::validation("name exceeds 255 bytes"));
            }
            if !project_names_equal(&record.name, &name)
                && self
                    .repository
                    .get_project_by_workspace_name(
                        command.tenant_id,
                        command.organization_id,
                        &record.workspace_id,
                        &name,
                    )?
                    .is_some()
            {
                return Err(KernelError::conflict(
                    "project name already exists in workspace",
                ));
            }
            record.name = trim(&name).to_string();
        }
        if let Some(description) = command.description {
            record.description = description;
        }
        if let Some(visibility) = command.visibility {
            record.visibility = visibility;
        }
        if let Some(drive_access_mode) = command.drive_access_mode {
            record.drive_access_mode = drive_access_mode;
        }
        validate_project_drive_access(record.visibility, record.drive_access_mode)?;
        if let Some(default_agent_id) = command.default_agent_id {
            if let Some(agent_id) = default_agent_id.as_deref() {
                validate_agent_id(agent_id)?;
                let agent = self
                    .repository
                    .get(command.tenant_id, agent_id)?
                    .ok_or_else(|| KernelError::not_found("default agent not found"))?;
                if agent.organization_id != command.organization_id {
                    return Err(KernelError::not_found("default agent not found"));
                }
            }
            record.default_agent_id = default_agent_id;
        }
        if let Some(default_model_id) = command.default_model_id {
            record.default_model_id = default_model_id;
        }
        record.mark_updated(command.requested_user_id, command.requested_at.clone());
        self.repository.update_project(record.clone())?;
        self.emit_project_audit_event(
            AgentAuditAction::ProjectUpdated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn archive_project(
        &self,
        command: ProjectMutationCommand,
    ) -> KernelResult<AgentProjectRecord> {
        self.mutate_project_status(command, AgentProjectStatus::Archived)
    }

    pub fn delete_project(
        &self,
        command: ProjectMutationCommand,
    ) -> KernelResult<AgentProjectRecord> {
        self.mutate_project_status(command, AgentProjectStatus::Deleted)
    }

    fn mutate_project_status(
        &self,
        command: ProjectMutationCommand,
        target: AgentProjectStatus,
    ) -> KernelResult<AgentProjectRecord> {
        validate_standard_id(&command.project_id, "projectId", Some(ID_PREFIX_PROJECT))?;
        let (request_id, action, audit_action) = match target {
            AgentProjectStatus::Archived => (
                "agent.business.project.archive",
                "project.archive",
                AgentAuditAction::ProjectArchived,
            ),
            AgentProjectStatus::Deleted => (
                "agent.business.project.delete",
                "project.delete",
                AgentAuditAction::ProjectDeleted,
            ),
            AgentProjectStatus::Active => {
                return Err(KernelError::validation("unsupported project transition"));
            }
        };
        self.authorize(
            request_id,
            command.requested_by.clone(),
            format!("agent.business.project.{}", command.project_id),
            action,
        )?;
        let mut record = self
            .repository
            .get_project(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
            )?
            .ok_or_else(|| KernelError::not_found("project not found"))?;
        Self::ensure_project_owner_scope(&record, command.owner_scope)?;
        if target != AgentProjectStatus::Deleted || command.expected_version.is_some() {
            ensure_expected_version(record.version, command.expected_version, "project")?;
        }
        match target {
            AgentProjectStatus::Archived => {
                record.archive(command.requested_user_id, command.requested_at.clone())
            }
            AgentProjectStatus::Deleted => {
                record.soft_delete(command.requested_user_id, command.requested_at.clone())
            }
            AgentProjectStatus::Active => unreachable!(),
        }
        self.repository.update_project(record.clone())?;
        self.emit_project_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_project(&self, command: GetProjectCommand) -> KernelResult<AgentProjectRecord> {
        self.authorize(
            "agent.business.project.retrieve",
            command.requested_by,
            format!("agent.business.project.{}", command.project_id),
            "project.retrieve",
        )?;
        validate_standard_id(&command.project_id, "projectId", Some(ID_PREFIX_PROJECT))?;
        self.repository
            .get_project(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
            )?
            .ok_or_else(|| KernelError::not_found("project not found"))
            .and_then(|record| {
                Self::ensure_project_owner_scope(&record, command.owner_scope)?;
                Ok(record)
            })
    }

    pub fn list_projects(
        &self,
        command: ListProjectsCommand,
    ) -> KernelResult<PaginatedResult<AgentProjectRecord>> {
        self.authorize(
            "agent.business.project.list",
            command.requested_by,
            format!(
                "agent.business.project.tenant.{}.organization.{}",
                command.query.tenant_id, command.query.organization_id
            ),
            "project.list",
        )?;
        let total_count = self.repository.count_projects(&command.query)?;
        let items = self.repository.list_projects(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn create_project_composition_slot(
        &self,
        command: CreateProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some(ID_PREFIX_SLOT))?;
        self.authorize(
            "agent.business.project.composition_slot.create",
            command.requested_by.clone(),
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.create",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        validate_project_composition_slot_fields(
            command.slot_kind,
            command.target_module,
            &command.target_ref,
            command.target_version_ref.as_deref(),
            command.priority,
            &command.policy_json,
        )?;
        if self
            .repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "project composition slot already exists",
            ));
        }
        let record = AgentProjectCompositionSlotRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            project_id: command.project_id,
            slot_id: command.slot_id,
            slot_kind: command.slot_kind,
            target_module: command.target_module,
            target_ref: trim(&command.target_ref).to_string(),
            target_version_ref: command.target_version_ref,
            priority: command.priority,
            enabled: command.enabled,
            policy_json: command.policy_json,
            created_by: command.requested_user_id,
            updated_by: command.requested_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        self.repository
            .insert_project_composition_slot(record.clone())?;
        self.emit_project_composition_slot_audit_event(
            AgentAuditAction::ProjectCompositionSlotCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_project_composition_slot(
        &self,
        command: GetProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some(ID_PREFIX_SLOT))?;
        self.authorize(
            "agent.business.project.composition_slot.retrieve",
            command.requested_by,
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.retrieve",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        self.repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .ok_or_else(|| KernelError::not_found("project composition slot not found"))
    }

    pub fn list_project_composition_slots(
        &self,
        command: ListProjectCompositionSlotsCommand,
    ) -> KernelResult<PaginatedResult<AgentProjectCompositionSlotRecord>> {
        self.authorize(
            "agent.business.project.composition_slot.list",
            command.requested_by,
            format!(
                "agent.business.project.{}.composition_slots",
                command.query.project_id
            ),
            "project.composition_slot.list",
        )?;
        self.load_active_project_for_composition(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.project_id,
            command.owner_scope,
        )?;
        let total_count = self
            .repository
            .count_project_composition_slots(&command.query)?;
        let items = self
            .repository
            .list_project_composition_slots(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn update_project_composition_slot(
        &self,
        command: UpdateProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some(ID_PREFIX_SLOT))?;
        self.authorize(
            "agent.business.project.composition_slot.update",
            command.requested_by.clone(),
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.update",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        let mut record = self
            .repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .ok_or_else(|| KernelError::not_found("project composition slot not found"))?;
        ensure_expected_version(
            record.version,
            command.expected_version,
            "project composition slot",
        )?;
        if let Some(slot_kind) = command.slot_kind {
            record.slot_kind = slot_kind;
        }
        if let Some(target_module) = command.target_module {
            record.target_module = target_module;
        }
        if let Some(target_ref) = command.target_ref {
            record.target_ref = trim(&target_ref).to_string();
        }
        if let Some(target_version_ref) = command.target_version_ref {
            record.target_version_ref = target_version_ref;
        }
        if let Some(priority) = command.priority {
            record.priority = priority;
        }
        if let Some(enabled) = command.enabled {
            record.enabled = enabled;
        }
        if let Some(policy_json) = command.policy_json {
            record.policy_json = policy_json;
        }
        validate_project_composition_slot_fields(
            record.slot_kind,
            record.target_module,
            &record.target_ref,
            record.target_version_ref.as_deref(),
            record.priority,
            &record.policy_json,
        )?;
        record.mark_updated(command.requested_user_id, command.requested_at.clone());
        self.repository
            .update_project_composition_slot(record.clone())?;
        self.emit_project_composition_slot_audit_event(
            AgentAuditAction::ProjectCompositionSlotUpdated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn delete_project_composition_slot(
        &self,
        command: DeleteProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some(ID_PREFIX_SLOT))?;
        self.authorize(
            "agent.business.project.composition_slot.delete",
            command.requested_by.clone(),
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.delete",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        let mut record = self
            .repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .ok_or_else(|| KernelError::not_found("project composition slot not found"))?;
        ensure_expected_version(
            record.version,
            command.expected_version,
            "project composition slot",
        )?;
        record.soft_delete(command.requested_user_id, command.requested_at.clone());
        self.repository
            .update_project_composition_slot(record.clone())?;
        self.emit_project_composition_slot_audit_event(
            AgentAuditAction::ProjectCompositionSlotDeleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Session management
    // -----------------------------------------------------------------------

    fn ensure_session_agent_identity(&self, command: &CreateSessionCommand) -> KernelResult<()> {
        let canonical_engine_key = command
            .agent_id
            .strip_prefix("agent.")
            .filter(|engine_key| {
                sdkwork_agents_runtime_facade::is_canonical_agent_engine(engine_key)
            })
            .filter(|engine_key| {
                sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
                    == Some(command.agent_id.as_str())
            });
        if canonical_engine_key.is_some() {
            let identity = sdkwork_agents_runtime_facade::resolve_agent_engine_runtime_identity(
                command.agent_id.as_str(),
            )
            .map_err(|error| {
                KernelError::provider_error("agent_engine_runtime_identity", error.to_string())
            })?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
            return self.ensure_agent_engine_runtime_identity(
                command.tenant_id,
                command.organization_id,
                command.owner_user_id,
                identity.engine_key.as_str(),
                identity.agent_id.as_str(),
                identity.binding_id.as_str(),
                identity.provider_id.as_str(),
                command.requested_by.clone(),
                command.requested_at.as_str(),
            );
        }

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .filter(|agent| !agent.is_deleted())
            .map(|_| ())
            .ok_or_else(|| KernelError::not_found("agent not found"))
    }

    fn find_session_creation_replay(
        &self,
        command: &CreateSessionCommand,
    ) -> KernelResult<Option<AgentSessionRecord>> {
        let Some(idempotency_key) = command.idempotency_key.as_deref() else {
            return Ok(None);
        };
        let Some(existing) = self.repository.get_session_by_creation_idempotency(
            command.tenant_id,
            command.organization_id,
            command.owner_user_id,
            idempotency_key,
        )?
        else {
            return Ok(None);
        };
        if existing.deleted_at.is_some() {
            return Err(KernelError::conflict(
                "session creation idempotency key belongs to a deleted session",
            ));
        }
        if existing.payload_hash.as_deref() != command.payload_hash.as_deref()
            || existing.agent_id != command.agent_id
            || existing.project_id != command.project_id
            || (!is_trimmed_blank(command.session_id.as_str())
                && existing.session_id != command.session_id)
        {
            return Err(KernelError::conflict(
                "session creation idempotency payload conflicts with the existing session",
            ));
        }
        Ok(Some(existing))
    }

    pub fn create_session(
        &self,
        command: CreateSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        self.create_session_with_authorization(command, true)
    }

    pub(crate) fn reconcile_provider_session_history_session(
        &self,
        command: CreateSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        let engine_key = command.agent_id.strip_prefix("agent.").ok_or_else(|| {
            KernelError::validation("provider Session history agent is not canonical")
        })?;
        if sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
            != Some(command.agent_id.as_str())
            || command.project_id.is_none()
            || command.source_module.as_deref() != Some("birdcoder")
            || command.source_context_kind.as_deref() != Some("provider_session")
            || !is_provider_session_id_for(&command.session_id, engine_key)
        {
            return Err(KernelError::validation(
                "provider Session history session reconciliation is not canonical",
            ));
        }
        self.create_session_with_authorization(command, false)
    }

    fn create_session_with_authorization(
        &self,
        command: CreateSessionCommand,
        authorize: bool,
    ) -> KernelResult<AgentSessionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let session_id = if is_trimmed_blank(command.session_id.as_str()) {
            format!("{ID_PREFIX_SESSION}{}", self.repository.next_id()?)
        } else {
            command.session_id.clone()
        };
        validate_standard_id(session_id.as_str(), "sessionId", Some(ID_PREFIX_SESSION))?;
        if authorize {
            self.authorize(
                "agent.business.session.create",
                command.requested_by.clone(),
                format!("agent.business.session.{}", session_id),
                "session.create",
            )?;
        }

        if command.idempotency_key.is_some() != command.payload_hash.is_some() {
            return Err(KernelError::validation(
                "idempotencyKey and payloadHash must be supplied together",
            ));
        }
        if let Some(idempotency_key) = command.idempotency_key.as_deref() {
            require_non_blank(idempotency_key, "idempotencyKey")?;
            require_non_blank(
                command.payload_hash.as_deref().unwrap_or_default(),
                "payloadHash",
            )?;
            if idempotency_key.len() > 256 {
                return Err(KernelError::validation("idempotencyKey exceeds 256 bytes"));
            }
            if command.payload_hash.as_deref().unwrap_or_default().len() > 128 {
                return Err(KernelError::validation("payloadHash exceeds 128 bytes"));
            }
        }
        if let Some(title) = command.title.as_deref() {
            require_non_blank(title, "title")?;
            if title.len() > 512 {
                return Err(KernelError::validation("title exceeds 512 bytes"));
            }
        }
        if let Some(existing) = self.find_session_creation_replay(&command)? {
            return Ok(existing);
        }

        // Ensure session does not already exist
        if self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                session_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("session already exists"));
        }

        if command.source_module.is_some()
            || command.source_context_kind.is_some()
            || command.source_context_id.is_some()
        {
            for (value, field) in [
                (command.source_module.as_deref(), "sourceModule"),
                (command.source_context_kind.as_deref(), "sourceContextKind"),
                (command.source_context_id.as_deref(), "sourceContextId"),
            ] {
                require_non_blank(
                    value.ok_or_else(|| {
                        KernelError::validation(
                            "sourceModule, sourceContextKind and sourceContextId must be supplied together",
                        )
                    })?,
                    field,
                )?;
            }
        }
        if command.forked_from_turn_id.is_some() && command.parent_session_id.is_none() {
            return Err(KernelError::validation(
                "forkedFromTurnId requires parentSessionId",
            ));
        }
        if let Some(parent_session_id) = command.parent_session_id.as_deref() {
            if parent_session_id == session_id {
                return Err(KernelError::validation(
                    "parentSessionId must differ from sessionId",
                ));
            }
            let parent = self
                .repository
                .get_session(
                    command.tenant_id,
                    command.organization_id,
                    parent_session_id,
                )?
                .ok_or_else(|| KernelError::not_found("parent session not found"))?;
            if parent.organization_id != command.organization_id
                || parent.owner_user_id != command.owner_user_id
            {
                return Err(KernelError::validation("parent session scope mismatch"));
            }
            if let Some(forked_from_turn_id) = command.forked_from_turn_id.as_deref() {
                let fork_turn = self
                    .repository
                    .get_turn(
                        command.tenant_id,
                        command.organization_id,
                        forked_from_turn_id,
                    )?
                    .ok_or_else(|| KernelError::not_found("fork turn not found"))?;
                if fork_turn.session_id != parent_session_id {
                    return Err(KernelError::validation(
                        "forkedFromTurnId does not belong to parentSessionId",
                    ));
                }
            }
        }
        if let Some(project_id) = command.project_id.as_deref() {
            validate_standard_id(project_id, "projectId", Some(ID_PREFIX_PROJECT))?;
            let project = self
                .repository
                .get_project(command.tenant_id, command.organization_id, project_id)?
                .ok_or_else(|| KernelError::not_found("project not found"))?;
            Self::ensure_project_owner_scope(&project, Some(command.owner_user_id))?;
            if project.status != AgentProjectStatus::Active {
                return Err(KernelError::validation("project is not active"));
            }
        }

        self.ensure_session_agent_identity(&command)?;
        let replay_command = command.clone();

        let title_source = if command.source_module.as_deref() == Some("birdcoder")
            && command.source_context_kind.as_deref() == Some("provider_session")
        {
            AgentSessionTitleSource::Provider
        } else {
            AgentSessionTitleSource::User
        };
        let record = AgentSessionRecord {
            id: self.repository.next_id()?,
            session_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id,
            owner_user_id: command.owner_user_id,
            project_id: command.project_id,
            session_kind: command.session_kind,
            entry_surface: command.entry_surface,
            source_module: command.source_module,
            source_context_kind: command.source_context_kind,
            source_context_id: command.source_context_id,
            parent_session_id: command.parent_session_id,
            forked_from_turn_id: command.forked_from_turn_id,
            title: command.title.map(|title| trim(&title).to_string()),
            title_source,
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: command.idempotency_key,
            payload_hash: command.payload_hash,
            created_by: command.owner_user_id,
            updated_by: command.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };

        if let Err(error) = self.repository.insert_session(record.clone()) {
            if error.kind() == KernelErrorKind::Conflict && replay_command.idempotency_key.is_some()
            {
                if let Some(existing) = self.find_session_creation_replay(&replay_command)? {
                    return Ok(existing);
                }
            }
            return Err(error);
        }
        self.emit_session_audit_event(
            AgentAuditAction::SessionCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn update_session(
        &self,
        command: UpdateSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        self.update_session_with_authorization(command, true, false)
    }

    pub(crate) fn reconcile_provider_session_history_session_title(
        &self,
        command: UpdateSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        let engine_key = command
            .path_agent_id
            .strip_prefix("agent.")
            .ok_or_else(|| {
                KernelError::validation("provider Session history agent is not canonical")
            })?;
        if sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
            != Some(command.path_agent_id.as_str())
            || !is_provider_session_id_for(&command.session_id, engine_key)
            || command.title.is_none()
            || command.project_id.is_some()
        {
            return Err(KernelError::validation(
                "provider Session history session title reconciliation is not canonical",
            ));
        }
        self.update_session_with_authorization(command, false, true)
    }

    fn update_session_with_authorization(
        &self,
        command: UpdateSessionCommand,
        authorize: bool,
        require_provider_session_history: bool,
    ) -> KernelResult<AgentSessionRecord> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        if authorize {
            self.authorize(
                "agent.business.session.update",
                command.requested_by.clone(),
                format!("agent.business.session.{}", command.session_id),
                "session.update",
            )?;
        }
        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        Self::ensure_session_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "session")?;
        if record.organization_id != command.organization_id {
            return Err(KernelError::validation("session organization mismatch"));
        }
        if command.expected_version.is_some() {
            ensure_expected_version(record.version, command.expected_version, "session")?;
        }
        if record.deleted_at.is_some() {
            return Err(KernelError::not_found("session not found"));
        }
        if require_provider_session_history
            && (record.source_module.as_deref() != Some("birdcoder")
                || record.source_context_kind.as_deref() != Some("provider_session")
                || record.project_id.is_none()
                || record.source_context_id != record.project_id)
        {
            return Err(KernelError::validation(
                "provider Session history session title reconciliation target is not canonical",
            ));
        }

        let mut audit_action = AgentAuditAction::SessionRenamed;
        if let Some(title) = command.title {
            require_non_blank(&title, "title")?;
            if title.len() > 512 {
                return Err(KernelError::validation("title exceeds 512 bytes"));
            }
            let title = trim(&title).to_string();
            if require_provider_session_history
                && (record.title_source != AgentSessionTitleSource::Provider
                    || record.title.as_deref() == Some(title.as_str()))
            {
                return Ok(record);
            }
            record.title = Some(title);
            if !require_provider_session_history {
                record.title_source = AgentSessionTitleSource::User;
            }
        }
        if let Some(project_id) = command.project_id {
            audit_action = AgentAuditAction::SessionMoved;
            if let Some(project_id) = project_id.as_deref() {
                validate_standard_id(project_id, "projectId", Some(ID_PREFIX_PROJECT))?;
                let project = self
                    .repository
                    .get_project(command.tenant_id, command.organization_id, project_id)?
                    .ok_or_else(|| KernelError::not_found("project not found"))?;
                Self::ensure_project_owner_scope(&project, command.owner_scope)?;
                if project.status != AgentProjectStatus::Active {
                    return Err(KernelError::validation("project is not active"));
                }
            }
            record.project_id = project_id;
        }
        record.mark_updated(command.requested_at.clone());
        self.repository.update_session(record.clone())?;
        self.emit_session_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn delete_session(
        &self,
        command: DeleteSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        self.authorize(
            "agent.business.session.delete",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.delete",
        )?;
        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        Self::ensure_session_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "session")?;
        if record.organization_id != command.organization_id {
            return Err(KernelError::validation("session organization mismatch"));
        }
        record.soft_delete(command.requested_at.clone());
        // Soft-delete and queue purge are atomic in durable backends: a
        // partial failure cannot leave a deleted session with queued inputs.
        self.repository.delete_session_and_purge_queue(
            record.clone(),
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            record.owner_user_id,
        )?;
        self.emit_session_audit_event(
            AgentAuditAction::SessionDeleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn close_session(&self, command: CloseSessionCommand) -> KernelResult<AgentSessionRecord> {
        validate_agent_id(command.path_agent_id.as_str())?;
        self.authorize(
            "agent.business.session.close",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.close",
        )?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;

        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;

        Self::ensure_session_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "session")?;

        if !record.status.is_active() {
            return Err(KernelError::validation("session is not active"));
        }

        ensure_expected_version(record.version, command.expected_version, "session")?;

        record.close(command.requested_at.clone());
        self.repository.update_session(record.clone())?;
        self.emit_session_audit_event(
            AgentAuditAction::SessionClosed,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn archive_session(
        &self,
        command: ArchiveSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        validate_agent_id(command.path_agent_id.as_str())?;
        self.authorize(
            "agent.business.session.archive",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.archive",
        )?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;

        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;

        Self::ensure_session_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "session")?;

        if record.status == AgentSessionStatus::Archived {
            return Err(KernelError::validation("session is already archived"));
        }
        if record.status.is_active() {
            return Err(KernelError::validation(
                "session must be closed before archiving",
            ));
        }

        ensure_expected_version(record.version, command.expected_version, "session")?;

        record.archive(command.requested_at.clone());
        self.repository.update_session(record.clone())?;
        self.emit_session_audit_event(
            AgentAuditAction::SessionArchived,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_session(&self, command: GetSessionCommand) -> KernelResult<AgentSessionRecord> {
        validate_agent_id(command.path_agent_id.as_str())?;
        self.authorize(
            "agent.business.session.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "session.retrieve",
        )?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        self.repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))
            .and_then(|record| {
                Self::ensure_session_owner_scope(&record, command.owner_scope)?;
                Self::ensure_nested_agent_id(
                    &record.agent_id,
                    command.path_agent_id.as_str(),
                    "session",
                )?;
                Ok(record)
            })
    }

    pub fn get_project_session(
        &self,
        command: GetProjectSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        validate_standard_id(
            command.project_id.as_str(),
            "projectId",
            Some(ID_PREFIX_PROJECT),
        )?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        self.authorize(
            "agent.business.session.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "session.retrieve",
        )?;
        let record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        Self::ensure_session_owner_scope(&record, command.owner_scope)?;
        if record.project_id.as_deref() != Some(command.project_id.as_str())
            || record.deleted_at.is_some()
        {
            return Err(KernelError::not_found("session not found"));
        }
        Ok(record)
    }

    pub fn list_sessions(
        &self,
        command: ListSessionsCommand,
    ) -> KernelResult<PaginatedResult<AgentSessionRecord>> {
        self.authorize(
            "agent.business.session.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "session.list",
        )?;
        let scope_fingerprint = command.query.scope_fingerprint();
        if command
            .query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
        {
            return Err(KernelError::validation(
                "cursor does not match the requested Session scope",
            ));
        }
        let page_size = command.query.pagination.page_size;
        let mut items = self.repository.list_sessions(&command.query)?;
        let has_more = items.len() > page_size;
        items.truncate(page_size);
        let next_page_token = if has_more {
            items
                .last()
                .map(|session| {
                    encode_session_list_cursor(&SessionListCursor {
                        updated_at: session.updated_at.clone(),
                        session_internal_id: session.id,
                        scope_fingerprint,
                    })
                })
                .transpose()?
        } else {
            None
        };
        Ok(PaginatedResult::new(items, next_page_token, None))
    }

    pub fn list_session_activity_summaries(
        &self,
        command: ListSessionActivitySummariesCommand,
    ) -> KernelResult<PaginatedResult<SessionActivitySummaryRecord>> {
        if command.query.page_size == 0 || command.query.page_size > MAX_PAGE_SIZE {
            return Err(KernelError::validation(
                "page_size must be between 1 and 200",
            ));
        }
        if let Some(agent_id) = command.query.agent_id.as_deref() {
            validate_standard_id(agent_id, "agentId", Some(ID_PREFIX_AGENT))?;
        }
        if let Some(project_id) = command.query.project_id.as_deref() {
            validate_standard_id(project_id, "projectId", Some(ID_PREFIX_PROJECT))?;
        }
        if let Some(workspace_id) = command.query.workspace_id.as_deref() {
            validate_standard_id(workspace_id, "workspaceId", Some(ID_PREFIX_WORKSPACE))?;
        }
        if command
            .query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != command.query.scope_fingerprint())
        {
            return Err(KernelError::validation(
                "cursor does not belong to the requested Session activity scope",
            ));
        }
        self.authorize(
            "agent.business.session.activity.list",
            command.requested_by,
            format!(
                "agent.business.session_activity.owner.{}",
                command.query.owner_user_id
            ),
            "session_activity.list",
        )?;
        self.repository
            .list_session_activity_summaries(&command.query)
    }

    pub fn list_session_user_states(
        &self,
        command: ListSessionUserStatesCommand,
    ) -> KernelResult<PaginatedResult<AgentResourceUserStateRecord>> {
        validate_agent_id(command.path_agent_id.as_str())?;
        if command.query.resource_ids.len() > 100 {
            return Err(KernelError::validation(
                "session user state list accepts at most 100 session ids",
            ));
        }
        for session_id in &command.query.resource_ids {
            validate_standard_id(session_id, "sessionIds", Some(ID_PREFIX_SESSION))?;
        }
        self.authorize(
            "agent.business.session.user_state.list",
            command.requested_by,
            format!(
                "agent.business.agent.{}.session.user_state",
                command.path_agent_id
            ),
            "session.user_state.list",
        )?;
        let mut query = command.query;
        query.resource_type = AgentResourceType::Session;
        query.agent_id = Some(command.path_agent_id);
        let total_count = self.repository.count_resource_user_states(&query)?;
        let items = self.repository.list_resource_user_states(&query)?;
        Ok(offset_paginated_result(
            items,
            &query.pagination,
            total_count,
        ))
    }

    pub fn get_session_user_state(
        &self,
        command: GetSessionUserStateCommand,
    ) -> KernelResult<SessionUserStateResult> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        self.authorize(
            "agent.business.session.user_state.retrieve",
            command.requested_by,
            format!("agent.business.session.{}.user_state", command.session_id),
            "session.user_state.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.user_id),
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        self.repository
            .get_resource_user_state(
                command.tenant_id,
                command.organization_id,
                command.user_id,
                AgentResourceType::Session,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session user state not found"))
    }

    pub fn update_session_user_state(
        &self,
        command: UpdateSessionUserStateCommand,
    ) -> KernelResult<SessionUserStateResult> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        if command.pinned.is_none()
            && command.hidden.is_none()
            && !command.mark_opened
            && command.last_read_item_sequence.is_none()
            && command.custom_title.is_none()
        {
            return Err(KernelError::validation(
                "session user state update requires a changed field",
            ));
        }
        self.authorize(
            "agent.business.session.user_state.update",
            command.requested_by,
            format!("agent.business.session.{}.user_state", command.session_id),
            "session.user_state.update",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.user_id),
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session not found"));
        }

        let existing = self.repository.get_resource_user_state(
            command.tenant_id,
            command.organization_id,
            command.user_id,
            AgentResourceType::Session,
            command.session_id.as_str(),
        )?;
        let mut record = match existing {
            Some(record) => {
                ensure_expected_version(
                    record.version,
                    command.expected_version,
                    "session user state",
                )?;
                record
            }
            None => {
                if command.expected_version.is_some() {
                    return Err(KernelError::conflict("session user state version mismatch"));
                }
                AgentResourceUserStateRecord {
                    id: self.repository.next_id()?,
                    tenant_id: command.tenant_id,
                    organization_id: command.organization_id,
                    user_id: command.user_id,
                    resource_type: AgentResourceType::Session,
                    resource_id: command.session_id.clone(),
                    pinned_at: None,
                    hidden_at: None,
                    last_opened_at: None,
                    last_read_item_sequence: None,
                    custom_title: None,
                    version: 0,
                    created_at: command.requested_at.clone(),
                    updated_at: command.requested_at.clone(),
                }
            }
        };

        if let Some(pinned) = command.pinned {
            record.pinned_at = pinned.then(|| command.requested_at.clone());
        }
        if let Some(hidden) = command.hidden {
            record.hidden_at = hidden.then(|| command.requested_at.clone());
        }
        if command.mark_opened {
            record.last_opened_at = Some(command.requested_at.clone());
        }
        if let Some(sequence) = command.last_read_item_sequence {
            if sequence > session.last_item_sequence {
                return Err(KernelError::validation(
                    "lastReadItemSequence exceeds the session item sequence",
                ));
            }
            if record
                .last_read_item_sequence
                .is_some_and(|current| sequence < current)
            {
                return Err(KernelError::conflict(
                    "lastReadItemSequence cannot move backwards",
                ));
            }
            record.last_read_item_sequence = Some(sequence);
        }
        if let Some(custom_title) = command.custom_title {
            record.custom_title = custom_title
                .map(|title| {
                    require_non_blank(title.as_str(), "customTitle")?;
                    if title.len() > 512 {
                        return Err(KernelError::validation("customTitle exceeds 512 bytes"));
                    }
                    reject_secret_material(title.as_str(), "customTitle")?;
                    Ok(trim(title.as_str()).to_string())
                })
                .transpose()?;
        }
        if command.expected_version.is_some() {
            record.version = record
                .version
                .checked_add(1)
                .ok_or_else(|| KernelError::conflict("session user state version overflow"))?;
        }
        record.updated_at = command.requested_at;
        self.repository
            .upsert_resource_user_state(record, command.expected_version)
    }

    pub fn list_item_feedback(
        &self,
        command: ListItemFeedbackCommand,
    ) -> KernelResult<PaginatedResult<AgentItemFeedbackRecord>> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(
            command.query.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        self.authorize(
            "agent.business.message.feedback.list",
            command.requested_by,
            format!(
                "agent.business.session.{}.message.feedback",
                command.query.session_id
            ),
            "message.feedback.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            command.query.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.query.user_id),
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        let total_count = self.repository.count_item_feedback(&command.query)?;
        let items = self.repository.list_item_feedback(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn update_item_feedback(
        &self,
        command: UpdateItemFeedbackCommand,
    ) -> KernelResult<ItemFeedbackResult> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        validate_standard_id(command.item_id.as_str(), "itemId", Some(ID_PREFIX_ITEM))?;
        self.authorize(
            "agent.business.item_feedback.update",
            command.requested_by.clone(),
            format!("agent.business.session_item.{}.feedback", command.item_id),
            "item_feedback.update",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.user_id),
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session item not found"));
        }
        let item = self
            .repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.item_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session item not found"))?;
        if item.kind != AgentSessionItemKind::AssistantOutput {
            return Err(KernelError::validation(
                "feedback is only supported for assistant items",
            ));
        }
        if let Some(reason_code) = command.reason_code.as_deref() {
            require_non_blank(reason_code, "reasonCode")?;
            if reason_code.len() > 64 {
                return Err(KernelError::validation("reasonCode exceeds 64 bytes"));
            }
            reject_secret_material(reason_code, "reasonCode")?;
        }
        if let Some(comment) = command.comment.as_deref() {
            require_non_blank(comment, "comment")?;
            if comment.len() > 1024 {
                return Err(KernelError::validation("comment exceeds 1024 bytes"));
            }
            reject_secret_material(comment, "comment")?;
        }
        if command.rating.is_none() && (command.reason_code.is_some() || command.comment.is_some())
        {
            return Err(KernelError::validation(
                "clearing feedback cannot include reasonCode or comment",
            ));
        }

        let existing = self.repository.get_item_feedback(
            command.tenant_id,
            command.organization_id,
            command.item_id.as_str(),
            command.user_id,
            true,
        )?;
        let mut record =
            match (existing, command.rating) {
                (Some(mut record), Some(rating)) => {
                    if record.deleted_at.is_none() || command.expected_version.is_some() {
                        ensure_expected_version(
                            record.version,
                            command.expected_version,
                            "message feedback",
                        )?;
                    }
                    record.version = record.version.checked_add(1).ok_or_else(|| {
                        KernelError::conflict("message feedback version overflow")
                    })?;
                    record.rating = rating;
                    record.reason_code = command.reason_code.as_deref().map(trim);
                    record.comment = command.comment.as_deref().map(trim);
                    record.updated_at = command.requested_at.clone();
                    record.deleted_at = None;
                    record
                }
                (None, Some(rating)) => {
                    if command.expected_version.is_some() {
                        return Err(KernelError::conflict("message feedback version mismatch"));
                    }
                    AgentItemFeedbackRecord {
                        id: self.repository.next_id()?,
                        tenant_id: command.tenant_id,
                        organization_id: command.organization_id,
                        item_id: command.item_id.clone(),
                        user_id: command.user_id,
                        rating,
                        reason_code: command.reason_code.as_deref().map(trim),
                        comment: command.comment.as_deref().map(trim),
                        version: 0,
                        created_at: command.requested_at.clone(),
                        updated_at: command.requested_at.clone(),
                        deleted_at: None,
                    }
                }
                (Some(mut record), None) if record.deleted_at.is_none() => {
                    ensure_expected_version(
                        record.version,
                        command.expected_version,
                        "message feedback",
                    )?;
                    record.version = record.version.checked_add(1).ok_or_else(|| {
                        KernelError::conflict("message feedback version overflow")
                    })?;
                    record.updated_at = command.requested_at.clone();
                    record.deleted_at = Some(command.requested_at.clone());
                    record
                }
                (_, None) => {
                    return Err(KernelError::not_found("message feedback not found"));
                }
            };
        record.updated_at = command.requested_at.clone();
        let record = self
            .repository
            .upsert_item_feedback(record, command.expected_version)?;
        self.emit_session_item_audit_event(
            AgentAuditAction::ItemFeedbackChanged,
            &item,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Task management
    // -----------------------------------------------------------------------

    pub fn create_task(&self, command: CreateTaskCommand) -> KernelResult<AgentTaskRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let task_id = if is_trimmed_blank(command.task_id.as_str()) {
            format!("{ID_PREFIX_TASK}{}", self.repository.next_id()?)
        } else {
            command.task_id.clone()
        };
        validate_standard_id(task_id.as_str(), "taskId", Some(ID_PREFIX_TASK))?;
        self.authorize(
            "agent.business.task.create",
            command.requested_by.clone(),
            format!("agent.business.task.{}", task_id),
            "task.create",
        )?;

        require_non_blank(command.prompt.as_str(), "prompt")?;
        reject_secret_material(command.prompt.as_str(), "prompt")?;
        if command.prompt.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "prompt exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        validate_task_definition_fields(
            &command.title,
            &command.cron_expression,
            &command.timezone,
            &command.external_ref,
        )?;

        let agent = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;
        if agent.organization_id != command.organization_id {
            return Err(KernelError::not_found("agent not found"));
        }

        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        if session.agent_id != command.agent_id
            || session.owner_user_id != command.owner_user_id
            || !session.status.is_active()
        {
            return Err(KernelError::not_found("session not found"));
        }

        if self
            .repository
            .get_task(command.tenant_id, command.organization_id, task_id.as_str())?
            .is_some()
        {
            return Err(KernelError::conflict("task already exists"));
        }

        if !is_trimmed_blank(command.metadata_json.as_str()) {
            validate_bounded_json_payload(
                command.metadata_json.as_str(),
                "metadataJson",
                MAX_METADATA_JSON_BYTES,
            )?;
        }

        let metadata_json = default_json_object_if_blank(command.metadata_json.as_str());

        let schedule = crate::task_scheduling::TaskSchedule {
            kind: command.schedule_kind,
            cron_expression: command.cron_expression.clone(),
            timezone: command.timezone.clone(),
            scheduled_at: command.scheduled_at.clone(),
            starts_at: command.starts_at.clone(),
            ends_at: command.ends_at.clone(),
        };
        let next_fire_at = schedule
            .next_after(command.requested_at.as_str())?
            .ok_or_else(|| KernelError::validation("schedule has no future occurrence"))?;
        validate_task_execution_policy(TaskExecutionPolicyInput::from(&command))?;

        let record = AgentTaskRecord {
            id: self.repository.next_id()?,
            task_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id,
            owner_user_id: command.owner_user_id,
            session_id: command.session_id,
            title: command.title,
            prompt: command.prompt,
            schedule_kind: command.schedule_kind,
            cron_expression: command.cron_expression,
            timezone: command.timezone,
            scheduled_at: command.scheduled_at,
            starts_at: command.starts_at,
            ends_at: command.ends_at,
            next_fire_at: Some(next_fire_at),
            misfire_policy: command.misfire_policy,
            overlap_policy: command.overlap_policy,
            max_concurrent_runs: command.max_concurrent_runs,
            max_catch_up_runs: command.max_catch_up_runs,
            max_attempts: command.max_attempts,
            retry_initial_delay_seconds: command.retry_initial_delay_seconds,
            retry_max_delay_seconds: command.retry_max_delay_seconds,
            timeout_seconds: command.timeout_seconds,
            priority: command.priority,
            status: AgentTaskStatus::Active,
            generation: 1,
            external_ref: command.external_ref,
            metadata_json,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at: None,
            paused_at: None,
            cancelled_at: None,
        };

        self.repository.insert_task(record.clone())?;
        self.emit_task_audit_event(
            AgentAuditAction::TaskCreated,
            &record,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;

        Ok(record)
    }

    pub fn cancel_task(&self, command: CancelTaskCommand) -> KernelResult<AgentTaskRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task.cancel",
            command.requested_by.clone(),
            format!("agent.business.task.{}", command.task_id),
            "task.cancel",
        )?;
        validate_standard_id(command.task_id.as_str(), "taskId", Some(ID_PREFIX_TASK))?;
        validate_agent_id(command.path_agent_id.as_str())?;

        let mut record = self
            .repository
            .get_task(
                command.tenant_id,
                command.organization_id,
                command.task_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("task not found"))?;

        Self::ensure_task_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "task")?;

        if !record.status.is_cancellable() {
            return Err(KernelError::validation("task cannot be cancelled"));
        }

        ensure_expected_version(record.version, command.expected_version, "task")?;

        record.cancel(command.requested_at.clone());
        let transition = self.repository.transition_task(record, "task_cancelled")?;
        let record = transition.task;
        self.emit_task_audit_event(
            AgentAuditAction::TaskCancelled,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn replace_task(&self, command: ReplaceTaskCommand) -> KernelResult<AgentTaskRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task.replace",
            command.requested_by.clone(),
            format!("agent.business.task.{}", command.task_id),
            "task.replace",
        )?;
        validate_standard_id(command.task_id.as_str(), "taskId", Some(ID_PREFIX_TASK))?;
        validate_agent_id(command.path_agent_id.as_str())?;
        parse_rfc3339_datetime(&command.requested_at, "requestedAt")?;
        require_non_blank(&command.prompt, "prompt")?;
        reject_secret_material(&command.prompt, "prompt")?;
        if command.prompt.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "prompt exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        validate_task_definition_fields(
            &command.title,
            &command.cron_expression,
            &command.timezone,
            &command.external_ref,
        )?;
        validate_task_execution_policy(TaskExecutionPolicyInput::from(&command))?;
        if !is_trimmed_blank(&command.metadata_json) {
            validate_bounded_json_payload(
                &command.metadata_json,
                "metadataJson",
                MAX_METADATA_JSON_BYTES,
            )?;
        }
        let schedule = crate::task_scheduling::TaskSchedule {
            kind: command.schedule_kind,
            cron_expression: command.cron_expression.clone(),
            timezone: command.timezone.clone(),
            scheduled_at: command.scheduled_at.clone(),
            starts_at: command.starts_at.clone(),
            ends_at: command.ends_at.clone(),
        };
        let next_fire_at = schedule
            .next_after(&command.requested_at)?
            .ok_or_else(|| KernelError::validation("schedule has no future occurrence"))?;

        let mut record = self
            .repository
            .get_task(command.tenant_id, command.organization_id, &command.task_id)?
            .ok_or_else(|| KernelError::not_found("task not found"))?;
        Self::ensure_task_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, &command.path_agent_id, "task")?;
        if !matches!(
            record.status,
            AgentTaskStatus::Active | AgentTaskStatus::Paused
        ) {
            return Err(KernelError::validation("task cannot be replaced"));
        }
        ensure_expected_version(record.version, Some(command.expected_version), "task")?;

        record.title = command.title;
        record.prompt = command.prompt;
        record.schedule_kind = command.schedule_kind;
        record.cron_expression = command.cron_expression;
        record.timezone = command.timezone;
        record.scheduled_at = command.scheduled_at;
        record.starts_at = command.starts_at;
        record.ends_at = command.ends_at;
        record.next_fire_at = Some(next_fire_at);
        record.misfire_policy = command.misfire_policy;
        record.overlap_policy = command.overlap_policy;
        record.max_concurrent_runs = command.max_concurrent_runs;
        record.max_catch_up_runs = command.max_catch_up_runs;
        record.max_attempts = command.max_attempts;
        record.retry_initial_delay_seconds = command.retry_initial_delay_seconds;
        record.retry_max_delay_seconds = command.retry_max_delay_seconds;
        record.timeout_seconds = command.timeout_seconds;
        record.priority = command.priority;
        record.external_ref = command.external_ref;
        record.metadata_json = default_json_object_if_blank(&command.metadata_json);
        record.completed_at = None;
        record.cancelled_at = None;
        record.generation = record
            .generation
            .checked_add(1)
            .ok_or_else(|| KernelError::conflict("task generation overflow"))?;
        record.mark_updated(command.requested_at.clone());

        let transition = self
            .repository
            .transition_task(record, "task_definition_replaced")?;
        self.emit_task_audit_event(
            AgentAuditAction::TaskUpdated,
            &transition.task,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(transition.task)
    }

    pub fn pause_task(&self, command: PauseTaskCommand) -> KernelResult<AgentTaskRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task.pause",
            command.requested_by.clone(),
            format!("agent.business.task.{}", command.task_id),
            "task.pause",
        )?;
        let mut record = self.load_task_for_command_scope(
            command.tenant_id,
            command.organization_id,
            &command.path_agent_id,
            &command.task_id,
            command.owner_scope,
        )?;
        if record.status != AgentTaskStatus::Active {
            return Err(KernelError::validation("task is not active"));
        }
        ensure_expected_version(record.version, Some(command.expected_version), "task")?;
        record.status = AgentTaskStatus::Paused;
        record.paused_at = Some(command.requested_at.clone());
        record.generation = record
            .generation
            .checked_add(1)
            .ok_or_else(|| KernelError::conflict("task generation overflow"))?;
        record.mark_updated(command.requested_at.clone());
        let transition = self.repository.transition_task(record, "task_paused")?;
        self.emit_task_audit_event(
            AgentAuditAction::TaskPaused,
            &transition.task,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(transition.task)
    }

    pub fn resume_task(&self, command: ResumeTaskCommand) -> KernelResult<AgentTaskRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task.resume",
            command.requested_by.clone(),
            format!("agent.business.task.{}", command.task_id),
            "task.resume",
        )?;
        let mut record = self.load_task_for_command_scope(
            command.tenant_id,
            command.organization_id,
            &command.path_agent_id,
            &command.task_id,
            command.owner_scope,
        )?;
        if record.status != AgentTaskStatus::Paused {
            return Err(KernelError::validation("task is not paused"));
        }
        ensure_expected_version(record.version, Some(command.expected_version), "task")?;
        if record.next_fire_at.is_none() {
            record.next_fire_at = record.schedule().next_after(&command.requested_at)?;
        }
        if record.next_fire_at.is_none() {
            return Err(KernelError::validation("schedule has no future occurrence"));
        }
        record.status = AgentTaskStatus::Active;
        record.paused_at = None;
        record.generation = record
            .generation
            .checked_add(1)
            .ok_or_else(|| KernelError::conflict("task generation overflow"))?;
        record.mark_updated(command.requested_at.clone());
        let transition = self.repository.transition_task(record, "task_resumed")?;
        self.emit_task_audit_event(
            AgentAuditAction::TaskResumed,
            &transition.task,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(transition.task)
    }

    /// Create an idempotent manual Run for an active Task.
    pub fn execute_task(&self, command: ExecuteTaskCommand) -> KernelResult<AgentTaskRunRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task.execute",
            command.requested_by.clone(),
            format!("agent.business.task.{}", command.task_id),
            "task.execute",
        )?;
        validate_standard_id(command.task_id.as_str(), "taskId", Some(ID_PREFIX_TASK))?;
        validate_agent_id(command.path_agent_id.as_str())?;

        let record = self
            .repository
            .get_task(
                command.tenant_id,
                command.organization_id,
                command.task_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("task not found"))?;

        Self::ensure_task_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "task")?;

        if record.status != AgentTaskStatus::Active {
            return Err(KernelError::validation(
                "task is not active, cannot execute",
            ));
        }

        ensure_expected_version(record.version, command.expected_version, "task")?;

        self.repository.create_manual_task_run(
            &record,
            command.idempotency_key.trim(),
            &command.requested_at,
        )
    }

    pub fn materialize_scheduled_task_runs(
        &self,
        request: &MaterializeDueTasksRequest,
    ) -> KernelResult<Vec<AgentTaskRunRecord>>
    where
        R: TaskSchedulerRepository,
    {
        self.repository.materialize_due_tasks(request)
    }

    pub fn claim_scheduled_task_runs(
        &self,
        request: &ClaimTaskRunsRequest,
    ) -> KernelResult<Vec<TaskRunClaim>>
    where
        R: TaskSchedulerRepository,
    {
        self.repository.claim_task_runs(request)
    }

    pub fn heartbeat_scheduled_task_run(
        &self,
        lease: &TaskRunLease,
        heartbeat_at: &str,
        lease_seconds: u32,
    ) -> KernelResult<AgentTaskRunRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.repository
            .heartbeat_task_run(lease, heartbeat_at, lease_seconds)
    }

    pub fn recover_expired_scheduled_task_run_leases(
        &self,
        now: &str,
        limit: usize,
    ) -> KernelResult<u64>
    where
        R: TaskSchedulerRepository,
    {
        self.repository.recover_expired_task_run_leases(now, limit)
    }

    /// Recovers Runs whose configured execution budget elapsed while they
    /// were still claimed or running (independent of heartbeat liveness).
    pub fn recover_timed_out_scheduled_task_runs(
        &self,
        now: &str,
        limit: usize,
    ) -> KernelResult<u64>
    where
        R: TaskSchedulerRepository,
    {
        self.repository.recover_timed_out_task_runs(now, limit)
    }

    pub fn scheduled_task_metrics_snapshot(
        &self,
        now: &str,
    ) -> KernelResult<crate::TaskSchedulerMetricsSnapshot>
    where
        R: TaskSchedulerRepository,
    {
        parse_rfc3339_datetime(now, "now")?;
        self.repository.scheduler_metrics_snapshot(now)
    }

    pub fn reconcile_scheduled_task_runs(
        &self,
        updated_before: &str,
        occurred_at: &str,
        limit: usize,
    ) -> KernelResult<TaskRunReconciliationResult>
    where
        R: TaskSchedulerRepository,
    {
        parse_rfc3339_datetime(updated_before, "updatedBefore")?;
        parse_rfc3339_datetime(occurred_at, "occurredAt")?;
        let runs = self
            .repository
            .list_reconciling_task_runs(updated_before, limit.clamp(1, 1_000))?;
        let examined = runs.len();
        let mut reconciled = Vec::new();
        let mut pending = 0usize;
        let mut skipped_conflicts = 0usize;
        for run in runs {
            let Some(turn_id) = run.turn_id.as_deref() else {
                pending = pending.saturating_add(1);
                continue;
            };
            let Some(turn) =
                self.repository
                    .get_turn(run.tenant_id, run.organization_id, turn_id)?
            else {
                pending = pending.saturating_add(1);
                continue;
            };
            let terminal_status = match turn.status {
                AgentTurnStatus::Completed => AgentTaskRunStatus::Succeeded,
                AgentTurnStatus::Failed => AgentTaskRunStatus::Failed,
                AgentTurnStatus::Cancelled => AgentTaskRunStatus::Cancelled,
                AgentTurnStatus::Requested | AgentTurnStatus::Running => {
                    pending = pending.saturating_add(1);
                    continue;
                }
            };
            match self
                .repository
                .reconcile_task_run(&ReconcileTaskRunRequest {
                    tenant_id: run.tenant_id,
                    organization_id: run.organization_id,
                    run_id: run.run_id.clone(),
                    expected_version: run.version,
                    terminal_status,
                    error_code: turn.error_code.clone(),
                    reconciled_at: occurred_at.to_string(),
                }) {
                Ok(record) => {
                    self.emit_task_run_audit_event(
                        AgentAuditAction::TaskRunReconciled,
                        &record,
                        PolicySubject::new(
                            "system.agents.task-run-reconciliation",
                            record.tenant_id.to_string(),
                        ),
                        occurred_at.to_string(),
                    )?;
                    reconciled.push(record);
                }
                Err(error) if error.kind() == KernelErrorKind::Conflict => {
                    skipped_conflicts = skipped_conflicts.saturating_add(1);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(TaskRunReconciliationResult {
            examined,
            reconciled,
            pending,
            skipped_conflicts,
        })
    }

    pub fn execute_scheduled_task_run_claim(
        &self,
        claim: &TaskRunClaim,
        requested_by: PolicySubject,
        requested_at: String,
    ) -> KernelResult<AgentTaskRunRecord>
    where
        R: TaskSchedulerRepository,
        P: Send + Sync,
    {
        execute_task_run_claim(
            &self.repository,
            &self.repository,
            self,
            claim,
            requested_by,
            requested_at,
        )
    }

    pub fn get_task(&self, command: GetTaskCommand) -> KernelResult<AgentTaskRecord> {
        self.authorize(
            "agent.business.task.retrieve",
            command.requested_by,
            format!("agent.business.task.{}", command.task_id),
            "task.retrieve",
        )?;
        validate_standard_id(command.task_id.as_str(), "taskId", Some(ID_PREFIX_TASK))?;
        self.repository
            .get_task(
                command.tenant_id,
                command.organization_id,
                command.task_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("task not found"))
            .and_then(|record| {
                Self::ensure_task_owner_scope(&record, command.owner_scope)?;
                Self::ensure_nested_agent_id(
                    &record.agent_id,
                    command.path_agent_id.as_str(),
                    "task",
                )?;
                Ok(record)
            })
    }

    pub fn list_tasks(
        &self,
        command: ListTasksCommand,
    ) -> KernelResult<PaginatedResult<AgentTaskRecord>> {
        self.authorize(
            "agent.business.task.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "task.list",
        )?;
        let scope_fingerprint = command.query.scope_fingerprint();
        if command
            .query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
        {
            return Err(KernelError::validation(
                "cursor does not match the requested Task scope",
            ));
        }
        let page_size = command.query.page_size;
        let mut items = self.repository.list_tasks(&command.query)?;
        let has_more = items.len() > page_size;
        items.truncate(page_size);
        let next_page_token = if has_more {
            items
                .last()
                .map(|task| {
                    encode_task_cursor(&TaskCursor {
                        updated_at: task.updated_at.clone(),
                        task_internal_id: task.id,
                        scope_fingerprint,
                    })
                })
                .transpose()?
        } else {
            None
        };
        Ok(PaginatedResult::new(items, next_page_token, None))
    }

    pub fn list_task_runs(&self, command: ListTaskRunsCommand) -> KernelResult<TaskRunPage>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task_run.list",
            command.requested_by,
            format!("agent.business.task.{}", command.query.task_id),
            "task_run.list",
        )?;
        self.load_task_for_command_scope(
            command.query.tenant_id,
            command.query.organization_id,
            &command.path_agent_id,
            &command.query.task_id,
            command.query.owner_user_id,
        )?;
        let scope_fingerprint = command.query.scope_fingerprint();
        if command
            .query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
        {
            return Err(KernelError::validation(
                "cursor does not match the requested task Run scope",
            ));
        }
        let page_size = command.query.page_size;
        let mut items = self.repository.list_task_runs(&command.query)?;
        let has_more = items.len() > page_size;
        items.truncate(page_size);
        let next_page_token = if has_more {
            items
                .last()
                .map(|run| {
                    encode_task_run_cursor(&TaskRunCursor {
                        run_internal_id: run.id,
                        scope_fingerprint,
                    })
                })
                .transpose()?
        } else {
            None
        };
        Ok(PaginatedResult::new(items, next_page_token, None))
    }

    pub fn get_task_run(&self, command: GetTaskRunCommand) -> KernelResult<AgentTaskRunRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task_run.retrieve",
            command.requested_by,
            format!("agent.business.task_run.{}", command.run_id),
            "task_run.retrieve",
        )?;
        self.load_task_for_command_scope(
            command.tenant_id,
            command.organization_id,
            &command.path_agent_id,
            &command.task_id,
            command.owner_scope,
        )?;
        let run = self
            .repository
            .get_task_run(command.tenant_id, command.organization_id, &command.run_id)?
            .ok_or_else(|| KernelError::not_found("task Run not found"))?;
        Self::ensure_task_run_scope(&run, &command.task_id, command.owner_scope)?;
        Ok(run)
    }

    pub fn retry_task_run(&self, command: RetryTaskRunCommand) -> KernelResult<AgentTaskRunRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task_run.retry",
            command.requested_by.clone(),
            format!("agent.business.task_run.{}", command.run_id),
            "task_run.retry",
        )?;
        let task = self.load_task_for_command_scope(
            command.tenant_id,
            command.organization_id,
            &command.path_agent_id,
            &command.task_id,
            command.owner_scope,
        )?;
        if task.status == AgentTaskStatus::Cancelled {
            return Err(KernelError::validation("cancelled task cannot be retried"));
        }
        let source = self
            .repository
            .get_task_run(command.tenant_id, command.organization_id, &command.run_id)?
            .ok_or_else(|| KernelError::not_found("task Run not found"))?;
        Self::ensure_task_run_scope(&source, &command.task_id, command.owner_scope)?;
        if !matches!(
            source.status,
            AgentTaskRunStatus::Failed
                | AgentTaskRunStatus::Cancelled
                | AgentTaskRunStatus::DeadLetter
        ) {
            return Err(KernelError::validation(
                "task Run is not in a retryable terminal state",
            ));
        }
        let run = self.repository.create_business_retry_task_run(
            &task,
            &source,
            command.idempotency_key.trim(),
            &command.requested_at,
        )?;
        self.emit_task_run_audit_event(
            AgentAuditAction::TaskRunCreated,
            &run,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(run)
    }

    pub fn cancel_task_run(&self, command: CancelTaskRunCommand) -> KernelResult<AgentTaskRunRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task_run.cancel",
            command.requested_by.clone(),
            format!("agent.business.task_run.{}", command.run_id),
            "task_run.cancel",
        )?;
        self.load_task_for_command_scope(
            command.tenant_id,
            command.organization_id,
            &command.path_agent_id,
            &command.task_id,
            command.owner_scope,
        )?;
        let current = self
            .repository
            .get_task_run(command.tenant_id, command.organization_id, &command.run_id)?
            .ok_or_else(|| KernelError::not_found("task Run not found"))?;
        Self::ensure_task_run_scope(&current, &command.task_id, command.owner_scope)?;
        let run = self.repository.request_task_run_cancellation(
            command.tenant_id,
            command.organization_id,
            &command.run_id,
            command.expected_version,
            &command.requested_at,
        )?;
        self.emit_task_run_audit_event(
            AgentAuditAction::TaskRunCancelRequested,
            &run,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(run)
    }

    pub fn list_task_run_attempts(
        &self,
        command: ListTaskRunAttemptsCommand,
    ) -> KernelResult<TaskRunAttemptPage>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task_run_attempt.list",
            command.requested_by,
            format!("agent.business.task_run.{}", command.query.run_id),
            "task_run_attempt.list",
        )?;
        self.load_task_for_command_scope(
            command.query.tenant_id,
            command.query.organization_id,
            &command.path_agent_id,
            &command.task_id,
            command.owner_scope,
        )?;
        let run = self
            .repository
            .get_task_run(
                command.query.tenant_id,
                command.query.organization_id,
                &command.query.run_id,
            )?
            .ok_or_else(|| KernelError::not_found("task Run not found"))?;
        Self::ensure_task_run_scope(&run, &command.task_id, command.owner_scope)?;
        let scope_fingerprint = command.query.scope_fingerprint();
        if command
            .query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
        {
            return Err(KernelError::validation(
                "cursor does not match the requested task Run Attempt scope",
            ));
        }
        let page_size = command.query.page_size;
        let mut items = self.repository.list_task_run_attempts(&command.query)?;
        let has_more = items.len() > page_size;
        items.truncate(page_size);
        let next_page_token = if has_more {
            items
                .last()
                .map(|attempt| {
                    encode_task_run_attempt_cursor(&TaskRunAttemptCursor {
                        attempt_no: attempt.attempt_no,
                        attempt_internal_id: attempt.id,
                        scope_fingerprint,
                    })
                })
                .transpose()?
        } else {
            None
        };
        Ok(PaginatedResult::new(items, next_page_token, None))
    }

    pub fn reconcile_task_run(
        &self,
        command: ReconcileTaskRunCommand,
    ) -> KernelResult<AgentTaskRunRecord>
    where
        R: TaskSchedulerRepository,
    {
        self.authorize(
            "agent.business.task_run.reconcile",
            command.requested_by.clone(),
            format!("agent.business.task_run.{}", command.run_id),
            "task_run.reconcile",
        )?;
        self.load_task_for_command_scope(
            command.tenant_id,
            command.organization_id,
            &command.path_agent_id,
            &command.task_id,
            command.owner_scope,
        )?;
        let run = self
            .repository
            .get_task_run(command.tenant_id, command.organization_id, &command.run_id)?
            .ok_or_else(|| KernelError::not_found("task Run not found"))?;
        Self::ensure_task_run_scope(&run, &command.task_id, command.owner_scope)?;
        self.ensure_reconciliation_matches_turn(&run, command.outcome)?;
        let reconciled = self
            .repository
            .reconcile_task_run(&ReconcileTaskRunRequest {
                tenant_id: command.tenant_id,
                organization_id: command.organization_id,
                run_id: command.run_id,
                expected_version: command.expected_version,
                terminal_status: command.outcome.terminal_status(),
                error_code: command.error_code,
                reconciled_at: command.requested_at.clone(),
            })?;
        self.emit_task_run_audit_event(
            AgentAuditAction::TaskRunReconciled,
            &reconciled,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(reconciled)
    }

    // -----------------------------------------------------------------------
    // Session runtime bindings
    // -----------------------------------------------------------------------

    pub fn create_session_runtime_binding(
        &self,
        command: CreateSessionRuntimeBindingCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        self.create_session_runtime_binding_with_authorization(command, true)
    }

    pub(crate) fn reconcile_provider_session_history_runtime_binding(
        &self,
        command: CreateSessionRuntimeBindingCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        let engine_key = command
            .path_agent_id
            .strip_prefix("agent.")
            .ok_or_else(|| {
                KernelError::validation("provider Session history agent is not canonical")
            })?;
        if sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
            != Some(command.path_agent_id.as_str())
            || sdkwork_agents_runtime_facade::agent_engine_binding_id(engine_key)
                != Some(command.provider_binding_id.as_str())
            || command.host_mode != "server"
            || command.transport_kind != "provider-session-history"
            || command
                .provider_session_id
                .as_deref()
                .is_none_or(str::is_empty)
            || !is_provider_session_id_for(&command.session_id, engine_key)
        {
            return Err(KernelError::validation(
                "provider Session history runtime binding reconciliation is not canonical",
            ));
        }
        self.create_session_runtime_binding_with_authorization(command, false)
    }

    /// Claims a provider Session identity for the canonical provider-history
    /// Session, retiring stale provider-import bindings that violate the unique
    /// provider Session constraint.
    ///
    /// Provider-import Sessions that predate the canonical scheme
    /// (`session.provider.*` / `session.native.*`, detected by
    /// `is_legacy_provider_session_id`) or that are attributed to another
    /// project are archived and their bindings released. User-created
    /// Sessions that already own the identity are left untouched and
    /// reported so the caller skips the redundant import.
    pub(crate) fn retire_legacy_provider_session_bindings(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        _engine_key: &str,
        provider_binding_id: &str,
        provider_session_id: &str,
        target_session_id: &str,
        _target_project_id: &str,
        requested_by: PolicySubject,
        requested_at: &str,
    ) -> KernelResult<ProviderSessionBindingClaim> {
        if provider_binding_id.trim().is_empty() || provider_session_id.trim().is_empty() {
            return Ok(ProviderSessionBindingClaim::Free);
        }
        let Some(binding) = self
            .repository
            .get_session_runtime_binding_by_provider_session(
                tenant_id,
                organization_id,
                owner_user_id,
                provider_binding_id,
                provider_session_id,
            )?
        else {
            return Ok(ProviderSessionBindingClaim::Free);
        };
        if binding.session_id == target_session_id {
            return Ok(ProviderSessionBindingClaim::AlreadyTarget);
        }
        let Some(session) =
            self.repository
                .get_session(tenant_id, organization_id, &binding.session_id)?
        else {
            return Ok(ProviderSessionBindingClaim::Free);
        };
        // A provider Session may be bound to only one SDKWork Session. Retire
        // only provider-import Sessions (canonical `session.{engine}.*` or
        // legacy-scheme imports, including imports attributed to another
        // project); user-created Sessions that already own the provider
        // identity must never be archived. Legacy-scheme Sessions are the
        // only place the old `session.provider.*` / `session.native.*`
        // prefixes are recognized (via `is_legacy_provider_session_id`).
        let is_provider_import = is_provider_session_id(&binding.session_id);
        if !is_provider_import {
            return Ok(ProviderSessionBindingClaim::AlreadyBoundByUserSession);
        }
        let session_agent_id = session.agent_id.clone();
        if session.deleted_at.is_none() && session.status != AgentSessionStatus::Archived {
            // Legacy imports were never closed; close then archive so the
            // retired Session disappears from the active inventory.
            let closed = if session.status == AgentSessionStatus::Active {
                self.close_session(CloseSessionCommand {
                    tenant_id,
                    organization_id,
                    path_agent_id: session_agent_id.clone(),
                    session_id: session.session_id.clone(),
                    expected_version: Some(session.version),
                    owner_scope: Some(owner_user_id),
                    requested_by: requested_by.clone(),
                    requested_at: requested_at.to_string(),
                })?
            } else {
                session.clone()
            };
            self.archive_session(ArchiveSessionCommand {
                tenant_id,
                organization_id,
                path_agent_id: session_agent_id.clone(),
                session_id: session.session_id.clone(),
                expected_version: Some(closed.version),
                owner_scope: Some(owner_user_id),
                requested_by: requested_by.clone(),
                requested_at: requested_at.to_string(),
            })?;
        }
        // Release the unique provider Session identity slot even when a
        // previous retirement already deactivated the row: NULL the provider
        // identity so the canonical provider-history binding can be created.
        let mut retired = binding;
        retired.provider_session_id = None;
        retired.provider_session_tree_id = None;
        retired.status = AgentSessionRuntimeBindingStatus::Deactivated;
        retired.is_current = false;
        retired.version = retired.version.saturating_add(1);
        retired.updated_at = requested_at.to_string();
        retired.deactivated_at = Some(requested_at.to_string());
        self.repository.update_session_runtime_binding(retired)?;
        Ok(ProviderSessionBindingClaim::Retired)
    }

    fn create_session_runtime_binding_with_authorization(
        &self,
        command: CreateSessionRuntimeBindingCommand,
        authorize: bool,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        let mut command = command;
        command.provider_session_id =
            normalize_optional_bounded(command.provider_session_id, "providerSessionId", 256)?;
        command.provider_session_tree_id = normalize_optional_bounded(
            command.provider_session_tree_id,
            "providerSessionTreeId",
            256,
        )?;
        command.provider_parent_session_id = normalize_optional_bounded(
            command.provider_parent_session_id,
            "providerParentSessionId",
            256,
        )?;
        command.provider_forked_from_session_id = normalize_optional_bounded(
            command.provider_forked_from_session_id,
            "providerForkedFromSessionId",
            256,
        )?;
        command.provider_directory = command
            .provider_directory
            .map(normalize_provider_session_directory)
            .transpose()?;
        validate_requested_at(&command.requested_at)?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot create a runtime binding",
            ));
        }
        if authorize {
            self.authorize(
                "agent.business.session_runtime_binding.create",
                command.requested_by.clone(),
                format!(
                    "agent.business.session.{}.runtime_binding",
                    command.session_id
                ),
                "session_runtime_binding.create",
            )?;
        }
        validate_runtime_token(&command.host_mode, "hostMode", 32)?;
        validate_runtime_token(&command.transport_kind, "transportKind", 64)?;
        require_non_blank(&command.model_id, "modelId")?;
        if command.model_id.len() > 128 {
            return Err(KernelError::validation("modelId exceeds 128 bytes"));
        }
        validate_standard_id(&command.provider_id, "providerId", Some(ID_PREFIX_PROVIDER))?;
        let provider_binding = self
            .repository
            .get_provider_binding(
                command.tenant_id,
                &command.path_agent_id,
                &command.provider_binding_id,
            )?
            .ok_or_else(|| KernelError::not_found("agent provider binding not found"))?;
        if provider_binding.provider_id != command.provider_id || !provider_binding.active {
            return Err(KernelError::validation(
                "agent provider binding is not active for providerId",
            ));
        }
        let runtime_binding_id = match command.runtime_binding_id {
            Some(value) => value,
            None => format!("{ID_PREFIX_RUNTIME_BINDING}{}", self.repository.next_id()?),
        };
        validate_standard_id(
            &runtime_binding_id,
            "runtimeBindingId",
            Some(ID_PREFIX_RUNTIME_BINDING),
        )?;
        if self
            .repository
            .get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &runtime_binding_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "session runtime binding already exists",
            ));
        }
        if self
            .repository
            .get_current_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "session already has a current runtime binding; update or activate an existing binding",
            ));
        }
        validate_optional_bounded(&command.runtime_location_id, "runtimeLocationId", 256)?;
        let provider_directory = command.provider_directory.unwrap_or_default();
        let record = AgentSessionRuntimeBindingRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: session.owner_user_id,
            session_id: command.session_id,
            runtime_binding_id,
            runtime_location_id: command.runtime_location_id,
            host_mode: command.host_mode,
            transport_kind: command.transport_kind,
            provider_binding_id: command.provider_binding_id,
            model_id: command.model_id,
            provider_id: command.provider_id,
            provider_session_id: command.provider_session_id,
            provider_session_tree_id: command.provider_session_tree_id,
            provider_parent_session_id: command.provider_parent_session_id,
            provider_forked_from_session_id: command.provider_forked_from_session_id,
            provider_title: provider_directory.title,
            provider_title_source: provider_directory.title_source,
            provider_preview: provider_directory.preview,
            provider_created_at: provider_directory.created_at,
            provider_updated_at: provider_directory.updated_at,
            provider_recency_at: provider_directory.recency_at,
            provider_pinned: provider_directory.pinned,
            provider_archived: provider_directory.archived,
            provider_visible: provider_directory.visible,
            provider_sort_key: (!provider_directory.sort_key.is_empty())
                .then_some(provider_directory.sort_key),
            provider_source: provider_directory.source,
            status: AgentSessionRuntimeBindingStatus::Active,
            is_current: true,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            activated_at: Some(command.requested_at.clone()),
            deactivated_at: None,
        };
        self.repository
            .insert_session_runtime_binding(record.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionRuntimeBindingCreated,
            "runtime_binding",
            &record.runtime_binding_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub(crate) fn reconcile_provider_session_history_runtime_binding_directory(
        &self,
        command: ReconcileProviderSessionRuntimeBindingDirectoryCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_requested_at(&command.requested_at)?;
        let engine_key = command
            .path_agent_id
            .strip_prefix("agent.")
            .ok_or_else(|| {
                KernelError::validation("provider Session history agent is not canonical")
            })?;
        if sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
            != Some(command.path_agent_id.as_str())
            || !is_provider_session_id_for(&command.session_id, engine_key)
            || !is_provider_runtime_binding_id_for(&command.runtime_binding_id, engine_key)
        {
            return Err(KernelError::validation(
                "provider Session history runtime binding directory reconciliation is not canonical",
            ));
        }
        let directory = normalize_provider_session_directory(command.provider_directory)?;
        let mut record = self.get_session_runtime_binding(GetSessionRuntimeBindingCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id,
            session_id: command.session_id,
            runtime_binding_id: command.runtime_binding_id,
            owner_scope: command.owner_scope,
            requested_by: command.requested_by.clone(),
        })?;
        if record.transport_kind != "provider-session-history"
            || !record.is_current
            || record.status != AgentSessionRuntimeBindingStatus::Active
        {
            return Err(KernelError::validation(
                "provider Session history runtime binding directory target is not active",
            ));
        }
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "session runtime binding",
        )?;
        if runtime_binding_provider_directory_matches(&record, &directory) {
            return Ok(record);
        }
        apply_provider_session_directory(&mut record, directory);
        record.mark_updated(command.requested_at.clone());
        self.repository
            .update_session_runtime_binding(record.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionRuntimeBindingUpdated,
            "runtime_binding",
            &record.runtime_binding_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn list_session_runtime_bindings(
        &self,
        command: ListSessionRuntimeBindingsCommand,
    ) -> KernelResult<PaginatedResult<SessionRuntimeBindingResult>> {
        self.authorize(
            "agent.business.session_runtime_binding.list",
            command.requested_by,
            format!(
                "agent.business.session.{}.runtime_binding",
                command.query.session_id
            ),
            "session_runtime_binding.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        let total_count = self
            .repository
            .count_session_runtime_bindings(&command.query)?;
        let items = self
            .repository
            .list_session_runtime_bindings(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_session_runtime_binding(
        &self,
        command: GetSessionRuntimeBindingCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_standard_id(
            &command.runtime_binding_id,
            "runtimeBindingId",
            Some(ID_PREFIX_RUNTIME_BINDING),
        )?;
        self.authorize(
            "agent.business.session_runtime_binding.retrieve",
            command.requested_by,
            format!(
                "agent.business.session.{}.runtime_binding.{}",
                command.session_id, command.runtime_binding_id
            ),
            "session_runtime_binding.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session runtime binding not found"));
        }
        self.repository
            .get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &command.runtime_binding_id,
            )?
            .ok_or_else(|| KernelError::not_found("session runtime binding not found"))
    }

    pub fn update_session_runtime_binding(
        &self,
        command: UpdateSessionRuntimeBindingCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_requested_at(&command.requested_at)?;
        self.authorize(
            "agent.business.session_runtime_binding.update",
            command.requested_by.clone(),
            format!(
                "agent.business.session.{}.runtime_binding.{}",
                command.session_id, command.runtime_binding_id
            ),
            "session_runtime_binding.update",
        )?;
        let mut record = self.get_session_runtime_binding(GetSessionRuntimeBindingCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id.clone(),
            session_id: command.session_id.clone(),
            runtime_binding_id: command.runtime_binding_id.clone(),
            owner_scope: command.owner_scope,
            requested_by: command.requested_by.clone(),
        })?;
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "session runtime binding",
        )?;
        if let Some(runtime_location_id) = command.runtime_location_id {
            validate_optional_bounded(&runtime_location_id, "runtimeLocationId", 256)?;
            record.runtime_location_id = runtime_location_id;
        }
        if let Some(host_mode) = command.host_mode {
            validate_runtime_token(&host_mode, "hostMode", 32)?;
            record.host_mode = host_mode;
        }
        if let Some(transport_kind) = command.transport_kind {
            validate_runtime_token(&transport_kind, "transportKind", 64)?;
            record.transport_kind = transport_kind;
        }
        let provider_binding_id = command
            .provider_binding_id
            .unwrap_or_else(|| record.provider_binding_id.clone());
        let provider_id = command
            .provider_id
            .unwrap_or_else(|| record.provider_id.clone());
        validate_standard_id(&provider_id, "providerId", Some(ID_PREFIX_PROVIDER))?;
        let provider_binding = self
            .repository
            .get_provider_binding(
                command.tenant_id,
                &command.path_agent_id,
                &provider_binding_id,
            )?
            .ok_or_else(|| KernelError::not_found("agent provider binding not found"))?;
        if provider_binding.provider_id != provider_id || !provider_binding.active {
            return Err(KernelError::validation(
                "agent provider binding is not active for providerId",
            ));
        }
        record.provider_binding_id = provider_binding_id;
        record.provider_id = provider_id;
        if let Some(model_id) = command.model_id {
            require_non_blank(&model_id, "modelId")?;
            if model_id.len() > 128 {
                return Err(KernelError::validation("modelId exceeds 128 bytes"));
            }
            record.model_id = model_id;
        }
        for (target, value, field) in [
            (
                &mut record.provider_session_id,
                command.provider_session_id,
                "providerSessionId",
            ),
            (
                &mut record.provider_session_tree_id,
                command.provider_session_tree_id,
                "providerSessionTreeId",
            ),
            (
                &mut record.provider_parent_session_id,
                command.provider_parent_session_id,
                "providerParentSessionId",
            ),
            (
                &mut record.provider_forked_from_session_id,
                command.provider_forked_from_session_id,
                "providerForkedFromSessionId",
            ),
        ] {
            if let Some(value) = value {
                validate_optional_bounded(&Some(value.clone()), field, 256)?;
                *target = Some(value);
            }
        }
        record.mark_updated(command.requested_at.clone());
        self.repository
            .update_session_runtime_binding(record.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionRuntimeBindingUpdated,
            "runtime_binding",
            &record.runtime_binding_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn activate_session_runtime_binding(
        &self,
        command: ChangeSessionRuntimeBindingStatusCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        self.change_session_runtime_binding_status(command, true)
    }

    pub fn deactivate_session_runtime_binding(
        &self,
        command: ChangeSessionRuntimeBindingStatusCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        self.change_session_runtime_binding_status(command, false)
    }

    fn change_session_runtime_binding_status(
        &self,
        command: ChangeSessionRuntimeBindingStatusCommand,
        activate: bool,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_requested_at(&command.requested_at)?;
        let action = if activate { "activate" } else { "deactivate" };
        self.authorize(
            format!("agent.business.session_runtime_binding.{action}"),
            command.requested_by.clone(),
            format!(
                "agent.business.session.{}.runtime_binding.{}",
                command.session_id, command.runtime_binding_id
            ),
            format!("session_runtime_binding.{action}"),
        )?;
        let mut record = self.get_session_runtime_binding(GetSessionRuntimeBindingCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id,
            session_id: command.session_id,
            runtime_binding_id: command.runtime_binding_id,
            owner_scope: command.owner_scope,
            requested_by: command.requested_by.clone(),
        })?;
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "session runtime binding",
        )?;
        if activate {
            if record.is_current && record.status == AgentSessionRuntimeBindingStatus::Active {
                return Ok(record);
            }
            record = self.repository.activate_session_runtime_binding_atomic(
                record.tenant_id,
                record.organization_id,
                &record.session_id,
                &record.runtime_binding_id,
                command.expected_version,
                command.requested_at.clone(),
            )?;
        } else {
            if !record.is_current || record.status != AgentSessionRuntimeBindingStatus::Active {
                return Err(KernelError::validation(
                    "session runtime binding is not active",
                ));
            }
            record.deactivate(
                AgentSessionRuntimeBindingStatus::Deactivated,
                command.requested_at.clone(),
            );
            self.repository
                .update_session_runtime_binding(record.clone())?;
        }
        self.emit_session_resource_audit_event(
            if activate {
                AgentAuditAction::SessionRuntimeBindingActivated
            } else {
                AgentAuditAction::SessionRuntimeBindingDeactivated
            },
            "runtime_binding",
            &record.runtime_binding_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Session checkpoints
    // -----------------------------------------------------------------------

    pub fn create_session_checkpoint(
        &self,
        command: CreateSessionCheckpointCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        validate_requested_at(&command.requested_at)?;
        parse_optional_rfc3339_datetime(command.retention_until.as_deref(), "retentionUntil")?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot create a checkpoint",
            ));
        }
        self.authorize(
            "agent.business.checkpoint.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}.checkpoint", command.session_id),
            "checkpoint.create",
        )?;
        validate_runtime_token(&command.checkpoint_kind, "checkpointKind", 64)?;
        let has_provider =
            command.runtime_binding_id.is_some() || command.provider_checkpoint_ref.is_some();
        let has_drive = command.drive_space_id.is_some() || command.drive_node_id.is_some();
        if has_provider == has_drive {
            return Err(KernelError::validation(
                "checkpoint must use exactly one provider or Drive reference",
            ));
        }
        if has_provider {
            let runtime_binding_id = command.runtime_binding_id.as_deref().ok_or_else(|| {
                KernelError::validation(
                    "runtimeBindingId and providerCheckpointRef must be supplied together",
                )
            })?;
            require_non_blank(
                command.provider_checkpoint_ref.as_deref().ok_or_else(|| {
                    KernelError::validation(
                        "runtimeBindingId and providerCheckpointRef must be supplied together",
                    )
                })?,
                "providerCheckpointRef",
            )?;
            self.repository
                .get_session_runtime_binding(
                    command.tenant_id,
                    command.organization_id,
                    &command.session_id,
                    runtime_binding_id,
                )?
                .ok_or_else(|| KernelError::not_found("session runtime binding not found"))?;
        } else {
            require_non_blank(
                command.drive_space_id.as_deref().ok_or_else(|| {
                    KernelError::validation(
                        "driveSpaceId and driveNodeId must be supplied together",
                    )
                })?,
                "driveSpaceId",
            )?;
            require_non_blank(
                command.drive_node_id.as_deref().ok_or_else(|| {
                    KernelError::validation(
                        "driveSpaceId and driveNodeId must be supplied together",
                    )
                })?,
                "driveNodeId",
            )?;
        }
        if let Some(turn_id) = command.turn_id.as_deref() {
            validate_standard_id(turn_id, "turnId", Some(ID_PREFIX_TURN))?;
            let turn = self
                .repository
                .get_turn(command.tenant_id, command.organization_id, turn_id)?
                .ok_or_else(|| KernelError::not_found("turn not found"))?;
            if turn.session_id != command.session_id {
                return Err(KernelError::not_found("turn not found"));
            }
        }
        let checkpoint_id = match command.checkpoint_id {
            Some(value) => value,
            None => format!("{ID_PREFIX_CHECKPOINT}{}", self.repository.next_id()?),
        };
        validate_standard_id(&checkpoint_id, "checkpointId", Some(ID_PREFIX_CHECKPOINT))?;
        if self
            .repository
            .get_session_checkpoint(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &checkpoint_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict("session checkpoint already exists"));
        }
        let record = AgentSessionCheckpointRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id,
            checkpoint_id,
            turn_id: command.turn_id,
            runtime_binding_id: command.runtime_binding_id,
            checkpoint_kind: command.checkpoint_kind,
            provider_checkpoint_ref: command.provider_checkpoint_ref,
            drive_space_id: command.drive_space_id,
            drive_node_id: command.drive_node_id,
            resumable: command.resumable,
            status: AgentSessionCheckpointStatus::Active,
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            restored_at: None,
            invalidated_at: None,
            retention_until: command.retention_until,
        };
        self.repository.insert_session_checkpoint(record.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionCheckpointCreated,
            "checkpoint",
            &record.checkpoint_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn list_session_checkpoints(
        &self,
        command: ListSessionCheckpointsCommand,
    ) -> KernelResult<PaginatedResult<SessionCheckpointResult>> {
        self.authorize(
            "agent.business.checkpoint.list",
            command.requested_by,
            format!(
                "agent.business.session.{}.checkpoint",
                command.query.session_id
            ),
            "checkpoint.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        let total_count = self.repository.count_session_checkpoints(&command.query)?;
        let items = self.repository.list_session_checkpoints(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_session_checkpoint(
        &self,
        command: GetSessionCheckpointCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        validate_standard_id(
            &command.checkpoint_id,
            "checkpointId",
            Some(ID_PREFIX_CHECKPOINT),
        )?;
        self.authorize(
            "agent.business.checkpoint.retrieve",
            command.requested_by,
            format!(
                "agent.business.session.{}.checkpoint.{}",
                command.session_id, command.checkpoint_id
            ),
            "checkpoint.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session checkpoint not found"));
        }
        self.repository
            .get_session_checkpoint(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &command.checkpoint_id,
            )?
            .ok_or_else(|| KernelError::not_found("session checkpoint not found"))
    }

    pub fn restore_session_checkpoint(
        &self,
        command: ChangeSessionCheckpointStatusCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        self.change_session_checkpoint_status(command, true)
    }

    pub fn invalidate_session_checkpoint(
        &self,
        command: ChangeSessionCheckpointStatusCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        self.change_session_checkpoint_status(command, false)
    }

    fn change_session_checkpoint_status(
        &self,
        command: ChangeSessionCheckpointStatusCommand,
        restore: bool,
    ) -> KernelResult<SessionCheckpointResult> {
        validate_requested_at(&command.requested_at)?;
        let action = if restore { "restore" } else { "invalidate" };
        self.authorize(
            format!("agent.business.checkpoint.{action}"),
            command.requested_by.clone(),
            format!(
                "agent.business.session.{}.checkpoint.{}",
                command.session_id, command.checkpoint_id
            ),
            format!("{ID_PREFIX_CHECKPOINT}{action}"),
        )?;
        let mut record = self.get_session_checkpoint(GetSessionCheckpointCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id,
            session_id: command.session_id,
            checkpoint_id: command.checkpoint_id,
            owner_scope: command.owner_scope,
            requested_by: command.requested_by.clone(),
        })?;
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "session checkpoint",
        )?;
        if record.status != AgentSessionCheckpointStatus::Active {
            return Err(KernelError::validation("session checkpoint is not active"));
        }
        if restore {
            if !record.resumable {
                return Err(KernelError::validation(
                    "session checkpoint is not resumable",
                ));
            }
            record.mark_restored(command.requested_at.clone());
        } else {
            record.invalidate(command.requested_at.clone());
        }
        self.repository.update_session_checkpoint(record.clone())?;
        self.emit_session_resource_audit_event(
            if restore {
                AgentAuditAction::SessionCheckpointRestored
            } else {
                AgentAuditAction::SessionCheckpointInvalidated
            },
            "checkpoint",
            &record.checkpoint_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Session item and turn management
    // -----------------------------------------------------------------------

    /// Low-level item insert for tests and internal tooling. HTTP surfaces use
    /// [`Self::execute_turn`] which persists input and output items atomically.
    pub fn create_session_item(
        &self,
        command: CreateSessionItemCommand,
    ) -> KernelResult<AgentSessionItemRecord> {
        validate_standard_id(command.item_id.as_str(), "itemId", Some(ID_PREFIX_ITEM))?;
        self.authorize(
            "agent.business.session_item.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session_item.create",
        )?;

        // Ensure session exists and is active
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;

        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot create an item",
            ));
        }

        // Ensure the item does not already exist.
        if self
            .repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.item_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("session item already exists"));
        }

        require_non_blank(command.content.as_str(), "content")?;
        if command.content.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "content exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        reject_secret_material(command.content.as_str(), "content")?;

        let record = AgentSessionItemRecord {
            id: self.repository.next_id()?,
            item_id: command.item_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id.clone(),
            kind: command.kind,
            content: Some(command.content),
            content_type: default_plain_text_if_blank(command.content_type.as_str()),
            status: AgentSessionItemStatus::Completed,
            sequence: 0,
            input_tokens: command.input_tokens,
            output_tokens: command.output_tokens,
            model_id: command.model_id,
            provider_id: command.provider_id,
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: command.provider_payload_json,
            parent_item_id: command.parent_item_id,
            turn_id: None,
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at: Some(command.requested_at.clone()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };

        let (_, record) = self.repository.append_session_item(record)?;

        self.emit_session_item_audit_event(
            AgentAuditAction::SessionItemCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub(crate) fn reconcile_provider_session_history_session_item(
        &self,
        command: ReconcileProviderSessionHistoryItemCommand,
        engine_key: &str,
    ) -> KernelResult<AgentSessionItemRecord> {
        let expected_agent_id = sdkwork_agents_runtime_facade::agent_engine_agent_id(engine_key)
            .ok_or_else(|| {
                KernelError::validation("provider Session history engine is not canonical")
            })?;
        if !is_provider_session_id_for(&command.session_id, engine_key)
            || !is_provider_item_id_for(&command.item_id, engine_key)
        {
            return Err(KernelError::validation(
                "provider Session history session item reconciliation is not canonical",
            ));
        }
        validate_standard_id(command.item_id.as_str(), "itemId", Some(ID_PREFIX_ITEM))?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        if session.agent_id != expected_agent_id
            || session.source_module.as_deref() != Some("birdcoder")
            || session.source_context_kind.as_deref() != Some("provider_session")
            || !session.status.is_active()
        {
            return Err(KernelError::validation(
                "provider Session history session item reconciliation is not canonical",
            ));
        }
        let content = command
            .content
            .as_deref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if content
            .as_ref()
            .is_some_and(|value| value.len() > MAX_TURN_INPUT_CONTENT_BYTES)
        {
            return Err(KernelError::validation(format!(
                "provider Session history content exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        for (field_name, value, max_bytes) in [
            (
                "toolArgumentsJson",
                command.tool_arguments_json.as_deref(),
                MAX_TOOL_ARGUMENTS_JSON_BYTES,
            ),
            (
                "toolResultJson",
                command.tool_result_json.as_deref(),
                MAX_TOOL_RESULT_JSON_BYTES,
            ),
            (
                "providerPayloadJson",
                command.provider_payload_json.as_deref(),
                MAX_PROVIDER_PAYLOAD_JSON_BYTES,
            ),
        ] {
            if let Some(value) = value {
                validate_bounded_json_payload(value, field_name, max_bytes)?;
            }
        }
        validate_provider_session_item_payload(&command, content.as_deref())?;
        if content.is_none()
            && command.tool_arguments_json.is_none()
            && command.tool_result_json.is_none()
        {
            return Err(KernelError::validation(
                "provider Session history item has no content or structured payload",
            ));
        }
        let content_type = default_plain_text_if_blank(command.content_type.as_str());
        let completed_at = match command.status {
            AgentSessionItemStatus::Pending => None,
            AgentSessionItemStatus::Completed
            | AgentSessionItemStatus::Failed
            | AgentSessionItemStatus::Cancelled
            | AgentSessionItemStatus::Redacted => Some(command.requested_at.clone()),
        };
        if let Some(mut existing) = self.repository.get_session_item(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.item_id.as_str(),
        )? {
            let unchanged = existing.kind == command.kind
                && existing.content == content
                && existing.content_type == content_type
                && existing.status == command.status
                && existing.model_id == command.model_id
                && existing.provider_id == command.provider_id
                && existing.tool_name == command.tool_name
                && existing.tool_call_id == command.tool_call_id
                && existing.tool_arguments_json == command.tool_arguments_json
                && existing.tool_result_json == command.tool_result_json
                && existing.provider_payload_json == command.provider_payload_json
                && existing.parent_item_id == command.parent_item_id;
            if unchanged {
                return Ok(existing);
            }
            let is_newer_terminal_narrative_snapshot = matches!(
                existing.kind,
                AgentSessionItemKind::AssistantOutput | AgentSessionItemKind::Reasoning
            ) && existing.status == command.status
                && command.status == AgentSessionItemStatus::Completed
                && is_newer_provider_session_item_snapshot(
                    &existing.updated_at,
                    &command.requested_at,
                );
            if existing.status != AgentSessionItemStatus::Pending
                && !is_newer_terminal_narrative_snapshot
            {
                return Err(KernelError::conflict(
                    "terminal provider Session history item is immutable",
                ));
            }
            if command.status == AgentSessionItemStatus::Redacted {
                return Err(KernelError::validation(
                    "provider synchronization cannot redact a Session history item",
                ));
            }
            if existing.kind != command.kind
                || existing.content_type != content_type
                || existing.model_id != command.model_id
                || existing.provider_id != command.provider_id
                || existing.tool_name != command.tool_name
                || existing.tool_call_id != command.tool_call_id
                || existing.parent_item_id != command.parent_item_id
            {
                return Err(KernelError::conflict(
                    "provider Session history item identity is immutable",
                ));
            }
            existing.kind = command.kind;
            existing.content = content;
            existing.content_type = content_type;
            existing.status = command.status;
            existing.model_id = command.model_id;
            existing.provider_id = command.provider_id;
            existing.tool_name = command.tool_name;
            existing.tool_call_id = command.tool_call_id;
            existing.tool_arguments_json = command.tool_arguments_json;
            existing.tool_result_json = command.tool_result_json;
            existing.provider_payload_json = command.provider_payload_json;
            existing.parent_item_id = command.parent_item_id;
            existing.version = existing.version.checked_add(1).ok_or_else(|| {
                KernelError::conflict("provider Session history item version overflow")
            })?;
            existing.updated_at = command.requested_at;
            existing.completed_at = completed_at;
            self.repository.update_session_item(existing.clone())?;
            return Ok(existing);
        }
        let record = AgentSessionItemRecord {
            id: self.repository.next_id()?,
            item_id: command.item_id.clone(),
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id.clone(),
            kind: command.kind,
            content,
            content_type,
            status: command.status,
            sequence: 0,
            input_tokens: 0,
            output_tokens: 0,
            model_id: command.model_id,
            provider_id: command.provider_id,
            tool_name: command.tool_name,
            tool_call_id: command.tool_call_id,
            tool_arguments_json: command.tool_arguments_json,
            tool_result_json: command.tool_result_json,
            provider_payload_json: command.provider_payload_json,
            parent_item_id: command.parent_item_id,
            turn_id: None,
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at,
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };
        let record = match self.repository.append_session_item(record) {
            Ok((_, record)) => record,
            Err(error) if error.kind() == sdkwork_agent_kernel::KernelErrorKind::Conflict => self
                .repository
                .get_session_item(
                    command.tenant_id,
                    command.organization_id,
                    command.session_id.as_str(),
                    command.item_id.as_str(),
                )?
                .ok_or(error)?,
            Err(error) => return Err(error),
        };
        self.emit_session_item_audit_event(
            AgentAuditAction::SessionItemCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_session_item(
        &self,
        command: GetSessionItemCommand,
    ) -> KernelResult<AgentSessionItemRecord> {
        self.authorize(
            "agent.business.session_item.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "session_item.retrieve",
        )?;
        validate_standard_id(command.item_id.as_str(), "itemId", Some(ID_PREFIX_ITEM))?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        Self::ensure_session_owner_scope(&session, command.owner_scope)?;
        Self::ensure_nested_agent_id(&session.agent_id, command.path_agent_id.as_str(), "session")?;
        self.repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.item_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session item not found"))
    }

    pub fn list_session_items(
        &self,
        command: ListSessionItemsCommand,
    ) -> KernelResult<PaginatedResult<AgentSessionItemRecord>> {
        validate_agent_id(command.path_agent_id.as_str())?;
        self.authorize(
            "agent.business.session_item.list",
            command.requested_by,
            format!("agent.business.session.{}", command.query.session_id),
            "session_item.list",
        )?;
        self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            command.query.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if command.query.cursor_mode {
            let scope_fingerprint = command.query.cursor_scope_fingerprint();
            if command
                .query
                .cursor
                .as_ref()
                .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
            {
                return Err(KernelError::validation(
                    "cursor does not match the requested session item scope",
                ));
            }
            let page_size = command.query.pagination.page_size;
            let mut items = self.repository.list_session_items(&command.query)?;
            let has_more = items.len() > page_size;
            items.truncate(page_size);
            let next_page_token = if has_more {
                items
                    .last()
                    .map(|item| {
                        encode_session_item_cursor(&SessionItemCursor {
                            sequence: item.sequence,
                            item_internal_id: item.id,
                            scope_fingerprint,
                        })
                    })
                    .transpose()?
            } else {
                None
            };
            return Ok(PaginatedResult::new(items, next_page_token, None));
        }

        let total_count = self.repository.count_session_items(&command.query)?;
        let items = self.repository.list_session_items(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn list_session_items_with_drive_refs(
        &self,
        command: ListSessionItemsCommand,
    ) -> KernelResult<PaginatedResult<AgentSessionItemWithDriveRefs>> {
        let tenant_id = command.query.tenant_id;
        let organization_id = command.query.organization_id;
        let page = self.list_session_items(command)?;
        let item_ids = page
            .items
            .iter()
            .map(|item| item.item_id.clone())
            .collect::<Vec<_>>();
        let mut refs_by_item = HashMap::<String, Vec<AgentItemDriveRefRecord>>::new();
        for drive_ref in
            self.repository
                .list_item_drive_refs_batch(tenant_id, organization_id, &item_ids)?
        {
            refs_by_item
                .entry(drive_ref.item_id.clone())
                .or_default()
                .push(drive_ref);
        }
        let items = page
            .items
            .into_iter()
            .map(|item| AgentSessionItemWithDriveRefs {
                drive_refs: refs_by_item.remove(&item.item_id).unwrap_or_default(),
                item,
            })
            .collect();
        Ok(PaginatedResult {
            items,
            has_more: page.has_more,
            next_page_token: page.next_page_token,
            total_count: page.total_count,
        })
    }

    pub fn get_session_item_with_drive_refs(
        &self,
        command: GetSessionItemCommand,
    ) -> KernelResult<AgentSessionItemWithDriveRefs> {
        let tenant_id = command.tenant_id;
        let organization_id = command.organization_id;
        let item = self.get_session_item(command)?;
        let drive_refs =
            self.repository
                .list_item_drive_refs(tenant_id, organization_id, &item.item_id)?;
        Ok(AgentSessionItemWithDriveRefs { item, drive_refs })
    }

    pub fn get_turn(&self, command: GetTurnCommand) -> KernelResult<AgentTurnRecord> {
        validate_agent_id(&command.path_agent_id)?;
        validate_standard_id(&command.session_id, "sessionId", Some(ID_PREFIX_SESSION))?;
        validate_standard_id(&command.turn_id, "turnId", Some(ID_PREFIX_TURN))?;
        self.authorize(
            "agent.business.turn.retrieve",
            command.requested_by,
            format!("agent.business.turn.{}", command.turn_id),
            "turn.retrieve",
        )?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        Self::ensure_session_owner_scope(&session, command.owner_scope)?;
        if session.organization_id != command.organization_id
            || session.agent_id != command.path_agent_id
        {
            return Err(KernelError::not_found("turn not found"));
        }
        let turn = self
            .repository
            .get_turn(command.tenant_id, command.organization_id, &command.turn_id)?
            .ok_or_else(|| KernelError::not_found("turn not found"))?;
        if turn.session_id != command.session_id || turn.agent_id != command.path_agent_id {
            return Err(KernelError::not_found("turn not found"));
        }
        Ok(turn)
    }

    pub fn list_turns(
        &self,
        command: ListTurnsCommand,
    ) -> KernelResult<PaginatedResult<AgentTurnRecord>> {
        self.authorize(
            "agent.business.turn.list",
            command.requested_by,
            format!("agent.business.session.{}.turn", command.query.session_id),
            "turn.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        let scope_fingerprint = command.query.scope_fingerprint();
        if command
            .query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
        {
            return Err(KernelError::validation(
                "cursor does not match the requested Turn scope",
            ));
        }
        let page_size = command.query.pagination.page_size;
        let mut items = self.repository.list_turns(&command.query)?;
        let has_more = items.len() > page_size;
        items.truncate(page_size);
        let next_page_token = if has_more {
            items
                .last()
                .map(|turn| {
                    encode_created_at_cursor(
                        "turn",
                        &CreatedAtListCursor {
                            created_at: turn.created_at.clone(),
                            internal_id: turn.id,
                            scope_fingerprint,
                        },
                    )
                })
                .transpose()?
        } else {
            None
        };
        Ok(PaginatedResult::new(items, next_page_token, None))
    }

    pub fn get_turn_by_idempotency(
        &self,
        command: GetTurnByIdempotencyCommand,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        validate_agent_id(&command.path_agent_id)?;
        validate_standard_id(&command.session_id, "sessionId", Some(ID_PREFIX_SESSION))?;
        require_non_blank(&command.idempotency_key, "idempotencyKey")?;
        if command.idempotency_key.len() > 256 {
            return Err(KernelError::validation("idempotencyKey exceeds 256 bytes"));
        }
        self.authorize(
            "agent.business.turn.retrieve",
            command.requested_by,
            format!(
                "agent.business.turn.idempotency.{}",
                command.idempotency_key
            ),
            "turn.retrieve",
        )?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;
        Self::ensure_session_owner_scope(&session, Some(command.owner_user_id))?;
        if session.organization_id != command.organization_id
            || session.agent_id != command.path_agent_id
        {
            return Err(KernelError::not_found("turn not found"));
        }
        let turn = self.repository.get_turn_by_idempotency(
            command.tenant_id,
            command.organization_id,
            command.owner_user_id,
            &command.idempotency_key,
        )?;
        if let Some(turn) = turn.as_ref() {
            if turn.session_id != command.session_id || turn.agent_id != command.path_agent_id {
                return Err(KernelError::not_found("turn not found"));
            }
        }
        Ok(turn)
    }

    pub fn cancel_turn(&self, command: CancelTurnCommand) -> KernelResult<AgentTurnRecord> {
        let audit_subject = command.requested_by.clone();
        self.authorize(
            "agent.business.turn.cancel",
            command.requested_by.clone(),
            format!("agent.business.turn.{}", command.turn_id),
            "turn.cancel",
        )?;
        let mut turn = self.get_turn(GetTurnCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id,
            session_id: command.session_id,
            turn_id: command.turn_id,
            owner_scope: command.owner_scope,
            requested_by: command.requested_by,
        })?;
        if let Some(expected_version) = command.expected_version {
            if expected_version != turn.version {
                return Err(KernelError::conflict("turn version mismatch"));
            }
        }
        if !matches!(
            turn.status,
            AgentTurnStatus::Requested | AgentTurnStatus::Running
        ) {
            return Err(KernelError::validation("turn cannot be cancelled"));
        }
        self.emit_turn_audit_event(
            AgentAuditAction::TurnCancelRequested,
            &turn,
            audit_subject.clone(),
            command.requested_at.clone(),
        )?;

        let model_request_id = turn_model_request_id(&turn.turn_id);
        let finish_reason = if turn.status == AgentTurnStatus::Running {
            let provider_binding = match turn.provider_binding_id.as_deref() {
                Some(provider_binding_id) => Some(
                    self.repository
                        .get_provider_binding(turn.tenant_id, &turn.agent_id, provider_binding_id)?
                        .ok_or_else(|| {
                            KernelError::not_found("agent provider binding not found")
                        })?,
                ),
                None => None,
            };
            let cancellation = self.turn_executor.cancel(&TurnCancellationInput {
                turn_id: turn.turn_id.clone(),
                model_request_id: model_request_id.clone(),
                session_id: turn.session_id.clone(),
                binding_id: provider_binding
                    .as_ref()
                    .map(|binding| binding.binding_id.clone()),
                provider_has_model_chat: provider_binding.as_ref().is_some_and(|binding| {
                    binding
                        .capabilities
                        .iter()
                        .any(|capability| capability == "model.chat")
                }),
                tenant_id: turn.tenant_id,
                agent_id: turn.agent_id.clone(),
            })?;
            if cancellation.model_request_id != model_request_id
                || cancellation.finish_reason != "cancelled"
            {
                return Err(KernelError::provider_error(
                    "turn_cancellation_unconfirmed",
                    "Turn executor did not return a correlated cancelled acknowledgement",
                ));
            }
            cancellation.finish_reason
        } else {
            "cancelled".to_string()
        };

        turn = self
            .repository
            .get_turn(turn.tenant_id, turn.organization_id, &turn.turn_id)?
            .ok_or_else(|| KernelError::not_found("turn not found"))?;
        if turn.status == AgentTurnStatus::Cancelled {
            return Ok(turn);
        }
        if !matches!(
            turn.status,
            AgentTurnStatus::Requested | AgentTurnStatus::Running
        ) {
            return Err(KernelError::conflict(
                "turn reached a terminal state before cancellation was acknowledged",
            ));
        }
        let expected_version = turn.version;
        turn.finish_reason = Some(finish_reason);
        turn.mark_cancelled(command.requested_at.clone());
        let turn = self.repository.update_turn_state(turn, expected_version)?;
        self.emit_turn_audit_event(
            AgentAuditAction::TurnCancelled,
            &turn,
            audit_subject,
            command.requested_at,
        )?;
        Ok(turn)
    }

    pub fn reconcile_stale_turns(
        &self,
        stale_before: &str,
        occurred_at: &str,
        limit: usize,
    ) -> KernelResult<TurnReconciliationResult> {
        if is_trimmed_blank(stale_before) || is_trimmed_blank(occurred_at) {
            return Err(KernelError::validation(
                "stale_before and occurred_at are required",
            ));
        }
        let turns = self
            .repository
            .list_reconcilable_turns(stale_before, limit.clamp(1, 200))?;
        let examined = turns.len();
        let mut failed = Vec::with_capacity(examined);
        let mut skipped_conflicts = 0usize;
        for mut turn in turns {
            let expected_version = turn.version;
            turn.mark_failed(
                "turn_reconciliation_timeout",
                "turn did not reach a terminal state before the reconciliation deadline",
                occurred_at,
            );
            match self.repository.update_turn_state(turn, expected_version) {
                Ok(record) => {
                    self.emit_turn_audit_event(
                        AgentAuditAction::TurnFailed,
                        &record,
                        PolicySubject::new(
                            "system.agents.reconciliation",
                            record.tenant_id.to_string(),
                        ),
                        occurred_at.to_string(),
                    )?;
                    failed.push(record);
                }
                Err(error) if error.kind() == KernelErrorKind::Conflict => {
                    skipped_conflicts = skipped_conflicts.saturating_add(1);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(TurnReconciliationResult {
            examined,
            failed,
            skipped_conflicts,
        })
    }

    /// Maps a durable failed Turn back to the failure the original attempt
    /// reported. Idempotent replays of a failed Turn must surface the same
    /// outcome (provider/capacity/timeout) instead of a generic 409 conflict;
    /// the SDK retries 5xx responses with the same idempotency key, so a bare
    /// conflict would hide the real failure behind "turn execution already
    /// failed". Replays never re-execute the Turn.
    fn failed_turn_replay_error(turn: &AgentTurnRecord) -> KernelError {
        let error_code = turn.error_code.as_deref().unwrap_or("turn_execution_failed");
        let detail = turn
            .error_detail
            .clone()
            .unwrap_or_else(|| "turn execution already failed".to_string());
        match error_code {
            "turn_provider_capacity_exhausted" => KernelError::resource_exhausted(detail),
            "turn_reconciliation_timeout" => KernelError::timeout(detail),
            "turn_execution_identity_mismatch" => KernelError::Internal { message: detail },
            _ => KernelError::provider_error(error_code, detail),
        }
    }

    fn replay_existing_turn(
        &self,
        command: &CreateTurnCommand,
        session: AgentSessionRecord,
        existing_turn: AgentTurnRecord,
    ) -> KernelResult<TurnExecutionResult> {
        if existing_turn.tenant_id != command.tenant_id
            || existing_turn.organization_id != command.organization_id
            || existing_turn.owner_user_id != session.owner_user_id
            || existing_turn.session_id != command.session_id
            || existing_turn.agent_id != command.agent_id
        {
            return Err(KernelError::conflict(
                "idempotency key was already used for a different turn scope",
            ));
        }
        if existing_turn.payload_hash != command.payload_hash.trim() {
            return Err(KernelError::conflict(
                "idempotency key was already used with a different payload",
            ));
        }
        if existing_turn.status != AgentTurnStatus::Completed {
            return Err(match existing_turn.status {
                AgentTurnStatus::Requested | AgentTurnStatus::Running => {
                    KernelError::conflict("turn execution is already in progress")
                }
                AgentTurnStatus::Failed => Self::failed_turn_replay_error(&existing_turn),
                AgentTurnStatus::Cancelled => {
                    KernelError::conflict("turn execution was already cancelled")
                }
                AgentTurnStatus::Completed => unreachable!(),
            });
        }
        let response_item_id =
            existing_turn
                .response_item_id
                .as_deref()
                .ok_or_else(|| KernelError::Internal {
                    message: "completed turn is missing response_item_id".to_string(),
                })?;
        let user_input_item = self
            .repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &existing_turn.request_item_id,
            )?
            .ok_or_else(|| KernelError::Internal {
                message: "completed turn is missing request item".to_string(),
            })?;
        let assistant_output_item = self
            .repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                response_item_id,
            )?
            .ok_or_else(|| KernelError::Internal {
                message: "completed turn is missing response item".to_string(),
            })?;
        let user_item_drive_refs = self.repository.list_item_drive_refs(
            command.tenant_id,
            session.organization_id,
            &user_input_item.item_id,
        )?;
        let turn_items = self.repository.list_session_items_by_turn(
            command.tenant_id,
            session.organization_id,
            &command.session_id,
            &existing_turn.turn_id,
            MAX_PROVIDER_TURN_FACTS + 3,
        )?;
        if turn_items.len() > MAX_PROVIDER_TURN_FACTS + 2
            || turn_items
                .first()
                .is_none_or(|item| item.item_id != existing_turn.request_item_id)
            || turn_items
                .last()
                .is_none_or(|item| item.item_id != response_item_id)
        {
            return Err(KernelError::Internal {
                message: "completed Turn has an inconsistent authoritative item set".to_string(),
            });
        }
        Ok(TurnExecutionResult {
            session,
            turn: existing_turn,
            user_input_item,
            assistant_output_item,
            turn_items,
            user_item_drive_refs,
            stream_deltas: Vec::new(),
            stream_events: Vec::new(),
        })
    }

    fn persist_turn_provider_session_identity(
        &self,
        runtime_binding: &AgentSessionRuntimeBindingRecord,
        provider_session_id: Option<&str>,
        requested_by: PolicySubject,
        requested_at: &str,
    ) -> KernelResult<AgentSessionRuntimeBindingRecord> {
        let Some(provider_session_id) = normalize_optional_bounded(
            provider_session_id.map(str::to_string),
            "providerSessionId",
            256,
        )?
        else {
            return Ok(runtime_binding.clone());
        };

        if let Some(existing_provider_session_id) = runtime_binding.provider_session_id.as_deref() {
            if existing_provider_session_id == provider_session_id {
                return Ok(runtime_binding.clone());
            }
            return Err(KernelError::conflict(
                "providerSessionId changed for the active Session runtime binding",
            ));
        }

        let mut updated = runtime_binding.clone();
        updated.provider_session_id = Some(provider_session_id);
        updated.mark_updated(requested_at.to_string());
        self.repository
            .update_session_runtime_binding(updated.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionRuntimeBindingUpdated,
            "runtime_binding",
            &updated.runtime_binding_id,
            &updated.session_id,
            updated.tenant_id,
            updated.organization_id,
            updated.version,
            requested_by,
            requested_at.to_string(),
        )?;
        Ok(updated)
    }

    /// Execute one turn and atomically persist its input, output and session counters.
    pub fn execute_turn(&self, command: CreateTurnCommand) -> KernelResult<TurnExecutionResult> {
        self.execute_turn_internal(command, None)
    }

    /// Execute one durable Turn while forwarding provider-neutral live output.
    pub fn execute_turn_with_stream_sink(
        &self,
        command: CreateTurnCommand,
        stream_sink: Arc<dyn TurnExecutionStreamSink>,
    ) -> KernelResult<TurnExecutionResult> {
        self.execute_turn_internal(command, Some(stream_sink))
    }

    /// Resumes a non-terminal Turn on the task-scheduler retry path.
    ///
    /// A retried Run reuses its persisted Turn (idempotency contract: retries
    /// never create a second Turn). When the previous attempt left the Turn
    /// in `Requested`/`Running`/`Failed`/`Cancelled`, this resets it to
    /// `Requested` with a version bump — fencing any stale worker's late
    /// writes — and re-executes the same request through the shared committed
    /// execution path. Completed Turns are replayed instead of resumed.
    fn resume_task_turn(
        &self,
        command: &CreateTurnCommand,
        agent: AgentBusinessRecord,
        session: AgentSessionRecord,
        mut existing_turn: AgentTurnRecord,
        stream_sink: Option<Arc<dyn TurnExecutionStreamSink>>,
    ) -> KernelResult<TurnExecutionResult> {
        let expected_version = existing_turn.version;
        existing_turn.status = AgentTurnStatus::Requested;
        existing_turn.finish_reason = None;
        existing_turn.error_code = None;
        existing_turn.error_detail = None;
        existing_turn.updated_at = command.requested_at.clone();
        existing_turn.version = existing_turn.version.saturating_add(1);
        let turn = self.repository.update_turn_state(existing_turn, expected_version)?;
        self.emit_turn_audit_event(
            AgentAuditAction::TurnRequested,
            &turn,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;

        let user_input_item = self
            .repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &turn.request_item_id,
            )?
            .ok_or_else(|| KernelError::Internal {
                message: "resumed task Turn is missing its request item".to_string(),
            })?;
        let user_item_drive_refs = self.repository.list_item_drive_refs(
            command.tenant_id,
            session.organization_id,
            &user_input_item.item_id,
        )?;

                let session_runtime_binding = match command.runtime_binding_id.as_deref() {
            Some(runtime_binding_id) => self.repository.get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                runtime_binding_id,
            )?,
            None => self.repository.get_current_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?,
        };
        // Cloud-router account-pool routing: turns carrying a user auth token
        // may execute without any local session runtime binding (no API key /
        // provider binding needed); the cloudrouter gateway routes the model
        // request through its account pool. Sessions with a binding keep the
        // local binding chain (backward compatible).
        let cloud_router_routed = command.auth_token.is_some()
            && session_runtime_binding.is_none()
            && command.runtime_binding_id.is_none();
        let session_runtime_binding = match session_runtime_binding {
            Some(binding) => Some(binding),
            None if cloud_router_routed => None,
            None => {
                return Err(KernelError::not_found("active session runtime binding not found"));
            }
        };
        let provider_binding = if let Some(binding) = session_runtime_binding.as_ref() {
            if !binding.is_current || binding.status != AgentSessionRuntimeBindingStatus::Active {
                return Err(KernelError::validation(
                    "session runtime binding is not active",
                ));
            }
            if let Some(requested_model_id) = command.requested_model_id.as_deref() {
                // The RIG simple-agent binding (`binding.rig`) resolves the
                // model from its live configuration (`llm.rig.default_model`);
                // the client-requested model id is advisory there, so the
                // strict equality check only applies to other engines.
                let rig_bound = binding.provider_binding_id == "binding.rig";
                if !rig_bound && requested_model_id != binding.model_id {
                    return Err(KernelError::validation(
                        "requestedModelId does not match the active session runtime binding",
                    ));
                }
            }
            let provider_binding = self
                .repository
                .get_provider_binding(
                    command.tenant_id,
                    command.agent_id.as_str(),
                    &binding.provider_binding_id,
                )?
                .ok_or_else(|| KernelError::not_found("agent provider binding not found"))?;
            if !provider_binding.active || provider_binding.provider_id != binding.provider_id {
                return Err(KernelError::validation(
                    "session runtime binding references an inactive provider binding",
                ));
            }
            Some(provider_binding)
        } else {
            None
        };
        validate_optional_bounded(&command.access_mode_id, "accessModeId", 64)?;
        if let Some(access_mode_id) = command.access_mode_id.as_deref() {
            let engine_key = provider_binding
                .as_ref()
                .and_then(|provider_binding| {
                    engine_key_for_binding_id(&provider_binding.binding_id)
                })
                .ok_or_else(|| {
                    KernelError::validation(
                        "accessModeId is not supported by the active provider binding",
                    )
                })?;
            let host_guard = crate::runtime_facade_bridge::shared_agent_engine_host();
            let engine_slot = host_guard
                .as_deref()
                .and_then(|host| host.slot(engine_key))
                .ok_or_else(|| {
                    KernelError::provider_error(
                        "agent_engine_bootstrap_failed",
                        format!("shared agent engine slot is unavailable for {engine_key}"),
                    )
                })?;
            engine_slot.resolve_execution_settings(access_mode_id)?;
        }
        let history_items =
            self.repository
                .list_session_items(&SessionItemListQuery::for_recent_turn_context(
                    command.tenant_id,
                    command.organization_id,
                    command.session_id.clone(),
                    TURN_CONTEXT_ITEM_LIMIT,
                ))?;
        let history = history_items
            .iter()
            .filter_map(|record| record.content.clone().map(|content| (record.kind, content)))
            .collect::<Vec<_>>();

        self.execute_committed_turn(
            command,
            agent,
            session,
            turn,
            user_input_item,
            user_item_drive_refs,
            history,
            session_runtime_binding,
            provider_binding,
            stream_sink,
        )
    }

    fn execute_turn_internal(
        &self,
        command: CreateTurnCommand,
        stream_sink: Option<Arc<dyn TurnExecutionStreamSink>>,
    ) -> KernelResult<TurnExecutionResult> {
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        self.authorize(
            "agent.business.turn.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "turn.create",
        )?;

        let agent = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::not_found("agent not found"))?;

        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))?;

        Self::ensure_session_owner_scope(&session, command.owner_scope)?;

        if session.agent_id != command.agent_id
            || session.organization_id != command.organization_id
        {
            return Err(KernelError::not_found("session not found"));
        }
        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot execute a turn",
            ));
        }

        require_non_blank(command.content.as_str(), "content")?;
        if command.content.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "content exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        reject_secret_material(command.content.as_str(), "content")?;
        let normalized_drive_refs = normalize_item_drive_resources(&command.drive_refs)?;
        let idempotency_key = command.idempotency_key.trim().to_string();
        require_non_blank(&idempotency_key, "idempotencyKey")?;
        if idempotency_key.len() > 256 {
            return Err(KernelError::validation("idempotencyKey exceeds 256 bytes"));
        }
        let payload_hash = command.payload_hash.trim().to_string();
        require_non_blank(&payload_hash, "payloadHash")?;
        if payload_hash.len() > 128 {
            return Err(KernelError::validation("payloadHash exceeds 128 bytes"));
        }
        if let Some(existing_turn) = self.repository.get_turn_by_idempotency(
            command.tenant_id,
            session.organization_id,
            session.owner_user_id,
            &idempotency_key,
        )? {
            if existing_turn.tenant_id != command.tenant_id
                || existing_turn.organization_id != command.organization_id
                || existing_turn.owner_user_id != session.owner_user_id
                || existing_turn.session_id != command.session_id
                || existing_turn.agent_id != command.agent_id
            {
                return Err(KernelError::conflict(
                    "idempotency key was already used for a different turn scope",
                ));
            }
            if existing_turn.payload_hash != payload_hash {
                return Err(KernelError::conflict(
                    "idempotency key was already used with a different payload",
                ));
            }
            if existing_turn.status == AgentTurnStatus::Completed {
                return self.replay_existing_turn(&command, session, existing_turn);
            }
            if command.turn_mode == AgentTurnMode::Automation {
                // Task-scheduler retry path: an attempt crashed or failed
                // with the Turn left in a non-terminal state. The retry must
                // resume the same Turn (never create a second one); a stale
                // worker's late writes are fenced by the version CAS below.
                return self.resume_task_turn(&command, agent, session, existing_turn, stream_sink);
            }
            return Err(match existing_turn.status {
                AgentTurnStatus::Requested | AgentTurnStatus::Running => {
                    KernelError::conflict("turn execution is already in progress")
                }
                AgentTurnStatus::Failed => Self::failed_turn_replay_error(&existing_turn),
                AgentTurnStatus::Cancelled => {
                    KernelError::conflict("turn execution was already cancelled")
                }
                AgentTurnStatus::Completed => unreachable!(),
            });
        }

        let session_runtime_binding = match command.runtime_binding_id.as_deref() {
            Some(runtime_binding_id) => self.repository.get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                runtime_binding_id,
            )?,
            None => self.repository.get_current_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?,
        };
        // Cloud-router account-pool routing: turns carrying a user auth token
        // may execute without any local session runtime binding (no API key /
        // provider binding needed); the cloudrouter gateway routes the model
        // request through its account pool. Sessions with a binding keep the
        // local binding chain (backward compatible).
        let cloud_router_routed = command.auth_token.is_some()
            && session_runtime_binding.is_none()
            && command.runtime_binding_id.is_none();
        let session_runtime_binding = match session_runtime_binding {
            Some(binding) => Some(binding),
            None if cloud_router_routed => None,
            None => {
                return Err(KernelError::not_found("active session runtime binding not found"));
            }
        };
        let provider_binding = if let Some(binding) = session_runtime_binding.as_ref() {
            if !binding.is_current || binding.status != AgentSessionRuntimeBindingStatus::Active {
                return Err(KernelError::validation(
                    "session runtime binding is not active",
                ));
            }
            if let Some(requested_model_id) = command.requested_model_id.as_deref() {
                // The RIG simple-agent binding (`binding.rig`) resolves the
                // model from its live configuration (`llm.rig.default_model`);
                // the client-requested model id is advisory there, so the
                // strict equality check only applies to other engines.
                let rig_bound = binding.provider_binding_id == "binding.rig";
                if !rig_bound && requested_model_id != binding.model_id {
                    return Err(KernelError::validation(
                        "requestedModelId does not match the active session runtime binding",
                    ));
                }
            }
            let provider_binding = self
                .repository
                .get_provider_binding(
                    command.tenant_id,
                    command.agent_id.as_str(),
                    &binding.provider_binding_id,
                )?
                .ok_or_else(|| KernelError::not_found("agent provider binding not found"))?;
            if !provider_binding.active || provider_binding.provider_id != binding.provider_id {
                return Err(KernelError::validation(
                    "session runtime binding references an inactive provider binding",
                ));
            }
            Some(provider_binding)
        } else {
            None
        };
        validate_optional_bounded(&command.access_mode_id, "accessModeId", 64)?;
        if let Some(access_mode_id) = command.access_mode_id.as_deref() {
            let engine_key = provider_binding
                .as_ref()
                .and_then(|provider_binding| {
                    engine_key_for_binding_id(&provider_binding.binding_id)
                })
                .ok_or_else(|| {
                    KernelError::validation(
                        "accessModeId is not supported by the active provider binding",
                    )
                })?;
            let host_guard = crate::runtime_facade_bridge::shared_agent_engine_host();
            let engine_slot = host_guard
                .as_deref()
                .and_then(|host| host.slot(engine_key))
                .ok_or_else(|| {
                    KernelError::provider_error(
                        "agent_engine_bootstrap_failed",
                        format!("shared agent engine slot is unavailable for {engine_key}"),
                    )
                })?;
            engine_slot.resolve_execution_settings(access_mode_id)?;
        }
        let history_items =
            self.repository
                .list_session_items(&SessionItemListQuery::for_recent_turn_context(
                    command.tenant_id,
                    command.organization_id,
                    command.session_id.clone(),
                    TURN_CONTEXT_ITEM_LIMIT,
                ))?;
        let history = history_items
            .iter()
            .filter_map(|record| record.content.clone().map(|content| (record.kind, content)))
            .collect::<Vec<_>>();

        let turn_id = match command.turn_id.as_deref() {
            Some(turn_id) => {
                validate_standard_id(turn_id, "turnId", Some(ID_PREFIX_TURN))?;
                turn_id.to_string()
            }
            None => format!("{ID_PREFIX_TURN}{}", self.repository.next_id()?),
        };
        let user_input_item_id = format!("{ID_PREFIX_ITEM}{}", self.repository.next_id()?);
        let turn = AgentTurnRecord {
            id: self.repository.next_id()?,
            turn_id: turn_id.clone(),
            tenant_id: command.tenant_id,
            organization_id: session.organization_id,
            session_id: command.session_id.clone(),
            agent_id: command.agent_id.clone(),
            owner_user_id: session.owner_user_id,
            runtime_binding_id: session_runtime_binding
                .as_ref()
                .map(|binding| binding.runtime_binding_id.clone()),
            client_request_id: command.client_request_id.clone(),
            idempotency_key: idempotency_key.clone(),
            payload_hash: payload_hash.clone(),
            request_item_id: user_input_item_id.clone(),
            response_item_id: None,
            turn_mode: command.turn_mode,
            status: AgentTurnStatus::Requested,
            requested_model_id: command.requested_model_id.clone(),
            provider_binding_id: session_runtime_binding
                .as_ref()
                .map(|binding| binding.provider_binding_id.clone()),
            model_id: session_runtime_binding
                .as_ref()
                .map(|binding| binding.model_id.clone()),
            provider_id: session_runtime_binding
                .as_ref()
                .map(|binding| binding.provider_id.clone()),
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            finish_reason: None,
            error_code: None,
            error_detail: None,
            trace_id: command.client_request_id.clone(),
            attempt_count: 0,
            max_attempts: 3,
            next_retry_at: None,
            available_at: command.requested_at.clone(),
            lease_owner: None,
            lease_token: None,
            lease_expires_at: None,
            fencing_token: 0,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            started_at: None,
            completed_at: None,
            cancel_requested_at: None,
            cancelled_at: None,
            retention_until: None,
        };
        let user_content = command.content.clone();
        let user_input_item = AgentSessionItemRecord {
            id: self.repository.next_id()?,
            item_id: user_input_item_id,
            tenant_id: command.tenant_id,
            organization_id: session.organization_id,
            session_id: command.session_id.clone(),
            kind: AgentSessionItemKind::UserInput,
            content: Some(user_content.clone()),
            content_type: default_plain_text_if_blank(command.content_type.as_str()),
            status: AgentSessionItemStatus::Completed,
            sequence: 0,
            input_tokens: 0,
            output_tokens: 0,
            model_id: None,
            provider_id: None,
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: None,
            turn_id: Some(turn_id.clone()),
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at: Some(command.requested_at.clone()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };
        let user_item_drive_refs = normalized_drive_refs
            .into_iter()
            .map(|resource| {
                Ok(AgentItemDriveRefRecord {
                    id: self.repository.next_id()?,
                    tenant_id: command.tenant_id,
                    organization_id: session.organization_id,
                    item_id: user_input_item.item_id.clone(),
                    resource_role: resource.resource_role,
                    drive_space_id: resource.drive_space_id,
                    drive_node_id: resource.drive_node_id,
                    media_resource_id: None,
                    object_blob_id: None,
                    resource_hash: None,
                    alt_text: None,
                    sort_order: resource.sort_order,
                    status: 0,
                    created_by: session.owner_user_id,
                    created_at: command.requested_at.clone(),
                    updated_at: command.requested_at.clone(),
                    deleted_at: None,
                    retention_until: None,
                })
            })
            .collect::<KernelResult<Vec<_>>>()?;
        let (session, user_input_item) = match self.repository.insert_turn_request(
            turn.clone(),
            user_input_item,
            user_item_drive_refs.clone(),
        )? {
            TurnRequestWriteOutcome::Inserted {
                session,
                request_item,
            } => (*session, *request_item),
            TurnRequestWriteOutcome::Existing(existing_turn) => {
                return self.replay_existing_turn(&command, session, *existing_turn);
            }
        };
        self.emit_turn_audit_event(
            AgentAuditAction::TurnRequested,
            &turn,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;
        self.emit_session_item_audit_event(
            AgentAuditAction::SessionItemCreated,
            &user_input_item,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;
        return self.execute_committed_turn(
            &command,
            agent,
            session,
            turn,
            user_input_item,
            user_item_drive_refs,
            history,
            session_runtime_binding,
            provider_binding,
            stream_sink,
        );
    }
    /// Executes a committed Turn request through the provider and persists
    /// the authoritative outcome.
    ///
    /// Shared by fresh Turn creation and by task-scheduler retry recovery:
    /// the request item and Turn already exist, so execution starts from
    /// `mark_running` and the outcome is committed atomically.
    #[allow(clippy::too_many_arguments)]
    fn execute_committed_turn(
        &self,
        command: &CreateTurnCommand,
        agent: AgentBusinessRecord,
        session: AgentSessionRecord,
        turn: AgentTurnRecord,
        user_input_item: AgentSessionItemRecord,
        user_item_drive_refs: Vec<AgentItemDriveRefRecord>,
        history: Vec<(AgentSessionItemKind, String)>,
        session_runtime_binding: Option<AgentSessionRuntimeBindingRecord>,
        provider_binding: Option<AgentProviderBindingRecord>,
        stream_sink: Option<Arc<dyn TurnExecutionStreamSink>>,
    ) -> KernelResult<TurnExecutionResult> {
        let mut turn = turn;
        let turn_id = turn.turn_id.clone();
        let user_content = user_input_item
            .content
            .clone()
            .unwrap_or_default();
        turn.mark_running(command.requested_at.clone());
        // `update_turn_state` expects the pre-incremented version: the caller
        // must bump `turn.version` before the call and pass the stored
        // version. Fresh turns carry version 1 after `insert_turn_request`;
        // resumed turns carry their reset version.
        let stored_version = turn.version.saturating_sub(1);
        turn = self.repository.update_turn_state(turn, stored_version)?;

        let welcome_message = AgentManagementProfileDto::from_default_code_task_intent(
            agent.default_code_task_intent.as_ref(),
        )
        .and_then(|profile| profile.welcome_message);
        let provider_has_model_chat = provider_binding
            .as_ref()
            .is_some_and(|binding| {
                binding
                    .capabilities
                    .iter()
                    .any(|capability| capability == "model.chat")
            });

        let execution_input = TurnExecutionInput {
            turn_id: turn_id.clone(),
            model_request_id: turn_model_request_id(&turn_id),
            agent_display_name: agent.display_name.clone(),
            welcome_message,
            session: session.clone(),
            history,
            user_content: user_content.clone(),
            model_id: session_runtime_binding
                .as_ref()
                .map(|binding| binding.model_id.clone())
                .or_else(|| command.requested_model_id.clone()),
            provider_id: session_runtime_binding
                .as_ref()
                .map(|binding| binding.provider_id.clone()),
            provider_session_id: session_runtime_binding
                .as_ref()
                .and_then(|binding| binding.provider_session_id.clone()),
            binding_id: session_runtime_binding
                .as_ref()
                .map(|binding| binding.provider_binding_id.clone()),
            access_mode_id: command.access_mode_id.clone(),
            provider_has_model_chat,
            system_prompt: command.system_prompt.clone(),
            auth_token: command.auth_token.clone(),
            access_token: command.access_token.clone(),
        };
        let completion = if let Some(stream_sink) = stream_sink.as_ref() {
            stream_sink.begin_turn(&session.session_id, &turn_id);
            complete_with_timeout_and_sink(
                Arc::clone(&self.turn_executor),
                &execution_input,
                Arc::clone(stream_sink),
                TURN_EXECUTION_TIMEOUT,
            )
        } else {
            complete_with_timeout(
                Arc::clone(&self.turn_executor),
                &execution_input,
                command.prefer_stream,
                TURN_EXECUTION_TIMEOUT,
            )
        };
        if is_inference_error(completion.runtime_mode) || is_capacity_error(completion.runtime_mode)
        {
            let capacity_exhausted = is_capacity_error(completion.runtime_mode);
            let (error_code, error_detail) = if capacity_exhausted {
                (
                    "turn_provider_capacity_exhausted",
                    "provider execution capacity is exhausted",
                )
            } else {
                ("turn_inference_failed", "managed turn inference failed")
            };
            turn.mark_failed(error_code, error_detail, command.requested_at.clone());
            let failed_turn = self.repository.update_turn_state(turn, 1)?;
            self.emit_turn_audit_event(
                AgentAuditAction::TurnFailed,
                &failed_turn,
                command.requested_by.clone(),
                command.requested_at.clone(),
            )?;
            return if capacity_exhausted {
                Err(KernelError::resource_exhausted(completion.content))
            } else {
                Err(KernelError::provider_error(error_code, completion.content))
            };
        }

        let expected_model_request_id = turn_model_request_id(&turn_id);
        if completion.model_request_id.as_deref() != Some(expected_model_request_id.as_str()) {
            turn.mark_failed(
                "turn_execution_identity_mismatch",
                "Turn executor returned an uncorrelated model request identity",
                command.requested_at.clone(),
            );
            let failed_turn = self.repository.update_turn_state(turn, 1)?;
            self.emit_turn_audit_event(
                AgentAuditAction::TurnFailed,
                &failed_turn,
                command.requested_by.clone(),
                command.requested_at.clone(),
            )?;
            return Err(KernelError::provider_error(
                "turn_execution_identity_mismatch",
                "Turn executor returned an uncorrelated model request identity",
            ));
        }

        let session_runtime_binding = match session_runtime_binding.as_ref() {
            Some(binding) => Some(self.persist_turn_provider_session_identity(
                binding,
                completion.provider_session_id.as_deref(),
                command.requested_by.clone(),
                &command.requested_at.clone(),
            )?),
            None => None,
        };

        if completion.finish_reason.as_deref() == Some("cancelled") {
            let mut current_turn = self
                .repository
                .get_turn(command.tenant_id, command.organization_id, &turn_id)?
                .ok_or_else(|| KernelError::not_found("turn not found"))?;
            if current_turn.status != AgentTurnStatus::Cancelled {
                if current_turn.status != AgentTurnStatus::Running {
                    return Err(KernelError::conflict(
                        "cancelled provider Turn reached an incompatible terminal state",
                    ));
                }
                let expected_version = current_turn.version;
                current_turn.finish_reason = Some("cancelled".to_string());
                current_turn.mark_cancelled(command.requested_at.clone());
                let cancelled_turn = self
                    .repository
                    .update_turn_state(current_turn, expected_version)?;
                self.emit_turn_audit_event(
                    AgentAuditAction::TurnCancelled,
                    &cancelled_turn,
                    command.requested_by.clone(),
                    command.requested_at.clone(),
                )?;
            }
            return Err(KernelError::cancelled("agent Turn was cancelled"));
        }

        let provider_item_facts = project_terminal_provider_turn_items(
            &completion.stream_events,
            &command.session_id,
            &turn_id,
        )?;
        let completion_model_id = completion
            .model_id
            .clone()
            .or_else(|| {
                session_runtime_binding
                    .as_ref()
                    .map(|binding| binding.model_id.clone())
            })
            .unwrap_or_else(|| command.requested_model_id.clone().unwrap_or_default());
        let mut completed_items = provider_item_facts
            .into_iter()
            .map(|fact| {
                let created_at = fact
                    .created_at
                    .unwrap_or_else(|| command.requested_at.clone());
                parse_rfc3339_datetime(&created_at, "providerItem.createdAt")?;
                let completed_at = fact
                    .completed_at
                    .unwrap_or_else(|| command.requested_at.clone());
                parse_rfc3339_datetime(&completed_at, "providerItem.completedAt")?;
                let redacted = fact.status == AgentSessionItemStatus::Redacted;
                Ok(AgentSessionItemRecord {
                    id: self.repository.next_id()?,
                    item_id: fact.item_id,
                    tenant_id: command.tenant_id,
                    organization_id: session.organization_id,
                    session_id: command.session_id.clone(),
                    kind: fact.kind,
                    content: fact.content,
                    content_type: fact.content_type,
                    status: fact.status,
                    sequence: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    model_id: Some(completion_model_id.clone()),
                    provider_id: Some(fact.provider_id),
                    tool_name: fact.tool_name,
                    tool_call_id: fact.tool_call_id,
                    tool_arguments_json: fact.tool_arguments_json,
                    tool_result_json: fact.tool_result_json,
                    provider_payload_json: fact.provider_payload_json,
                    parent_item_id: fact
                        .parent_item_id
                        .or_else(|| Some(user_input_item.item_id.clone())),
                    turn_id: Some(turn_id.clone()),
                    created_by: session.owner_user_id,
                    version: 0,
                    created_at,
                    updated_at: completed_at.clone(),
                    completed_at: Some(completed_at.clone()),
                    redacted_at: redacted.then_some(completed_at),
                    redacted_by: redacted.then_some(session.owner_user_id),
                    retention_until: None,
                })
            })
            .collect::<KernelResult<Vec<_>>>()?;

        let assistant_output_item_id = match terminal_provider_assistant_item_id(
            &completion.stream_events,
            completion.provider_session_id.as_deref(),
        )? {
            Some(item_id) => item_id,
            None => format!("{ID_PREFIX_ITEM}{}", self.repository.next_id()?),
        };
        let assistant_output_item = AgentSessionItemRecord {
            id: self.repository.next_id()?,
            item_id: assistant_output_item_id,
            tenant_id: command.tenant_id,
            organization_id: session.organization_id,
            session_id: command.session_id.clone(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: Some(completion.content),
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            sequence: 0,
            input_tokens: completion.input_tokens,
            output_tokens: completion.output_tokens,
            model_id: Some(
                completion
                    .model_id
                    .clone()
                    .or_else(|| {
                        session_runtime_binding
                            .as_ref()
                            .map(|binding| binding.model_id.clone())
                    })
                    .unwrap_or_else(|| command.requested_model_id.clone().unwrap_or_default()),
            ),
            provider_id: Some(
                completion
                    .provider_id
                    .clone()
                    .or_else(|| {
                        session_runtime_binding
                            .as_ref()
                            .map(|binding| binding.provider_id.clone())
                    })
                    .unwrap_or_default(),
            ),
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            provider_payload_json: None,
            parent_item_id: Some(user_input_item.item_id.clone()),
            turn_id: Some(turn_id.clone()),
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at: Some(command.requested_at.clone()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };
        completed_items.push(assistant_output_item.clone());

        turn.response_item_id = Some(assistant_output_item.item_id.clone());
        turn.model_id = assistant_output_item.model_id.clone();
        turn.provider_id = assistant_output_item.provider_id.clone();
        turn.input_tokens = completion.input_tokens;
        turn.output_tokens = completion.output_tokens;
        turn.finish_reason = completion.finish_reason.clone();
        let expected_turn_version = turn.version;
        let expected_fencing_token = turn.fencing_token;
        let expected_lease_token = turn.lease_token.clone();
        turn.mark_completed(command.requested_at.clone());
        let completed_turn = turn.clone();

        let completed_turn_id = turn.turn_id.clone();
        let (session, completed_items) = self.repository.complete_turn(
            turn,
            expected_turn_version,
            expected_fencing_token,
            expected_lease_token,
            completed_items,
        )?;
        // The streaming checkpoint is consumed: the durable completion write
        // is authoritative now. A late sink flush after completion is ignored
        // by the adapter (status guard), so clearing here is safe.
        let _ = self.clear_turn_streaming_content(
            session.tenant_id,
            session.organization_id,
            &completed_turn_id,
        );
        let assistant_output_item = completed_items
            .last()
            .filter(|item| item.item_id == assistant_output_item.item_id)
            .cloned()
            .ok_or_else(|| KernelError::Internal {
                message: "completed Turn did not return its final assistant item".to_string(),
            })?;

        self.emit_turn_audit_event(
            AgentAuditAction::TurnCompleted,
            &completed_turn,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;

        for item in &completed_items {
            self.emit_session_item_audit_event(
                AgentAuditAction::SessionItemCreated,
                item,
                command.requested_by.clone(),
                command.requested_at.clone(),
            )?;
        }

        let mut turn_items = Vec::with_capacity(completed_items.len() + 1);
        turn_items.push(user_input_item.clone());
        turn_items.extend(completed_items);

        Ok(TurnExecutionResult {
            session,
            turn: completed_turn,
            user_input_item,
            assistant_output_item,
            turn_items,
            user_item_drive_refs,
            stream_deltas: completion.stream_deltas,
            stream_events: completion.stream_events,
        })
    }


    /// Authorizes a policy resource. Callers pass resources shaped like
    /// `agent.business.{agent_id}`, `agent.business.session.{session_id}`, or
    /// `agent.business.tenant.{tenant_id}` — these are authorization policy
    /// resource names, *not* durable agent ids. The `agent.business.` prefix
    /// intentionally mirrors the `agent.` id namespace for readability, but
    /// resources never pass through `validate_agent_id`.
    fn authorize(
        &self,
        request_id: impl Into<String>,
        subject: PolicySubject,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> KernelResult<()> {
        let policy_request = PolicyRequest::new(
            request_id,
            DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
            resource,
        )
        .with_category(PolicyCategory::ProductSpecific(
            DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY.to_string(),
        ))
        .with_subject(subject)
        .with_action(action)
        .with_redaction(KernelEventRedaction::TenantSensitive);

        let decision = self.policy_provider.evaluate(policy_request)?;
        if decision.decision != PolicyDecisionValue::Allow {
            return Err(KernelError::permission_required(
                decision
                    .safe_reason
                    .unwrap_or_else(|| "agent management denied".to_string()),
            ));
        }
        Ok(())
    }

    fn emit_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentBusinessRecord,
        previous_status: Option<AgentBusinessStatus>,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let mut audit_payload = AgentAuditPayload::new(action, record);
        if let Some(previous_status) = previous_status {
            audit_payload = audit_payload.with_previous_status(previous_status);
        }
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_audit_{}_{}", record.agent_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", AgentAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "agent")
        .with_context("aggregate_id", record.agent_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .with_context("agent_internal_id", record.id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.audit.v1");

        self.record_audit_event(event)
    }

    fn deactivate_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str,
        updated_at: String,
    ) -> KernelResult<()> {
        for mut binding in self.all_provider_bindings_for_agent(tenant_id, agent_id)? {
            if binding.active {
                binding.active = false;
                binding.mark_updated(updated_at.clone());
                self.repository.update_provider_binding(binding)?;
            }
        }
        Ok(())
    }

    fn emit_project_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentProjectRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "projectId": record.project_id,
            "workspaceId": record.workspace_id,
            "tenantId": record.tenant_id.to_string(),
            "organizationId": record.organization_id.to_string(),
            "ownerUserId": record.owner_user_id.to_string(),
            "status": record.status.as_str(),
            "visibility": record.visibility.as_str(),
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!("agent_project_{}_{}", record.project_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "project")
        .with_context("aggregate_id", record.project_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.project.audit.v1");
        self.record_audit_event(event)
    }

    fn emit_workspace_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentWorkspaceRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "workspaceId": record.workspace_id,
            "tenantId": record.tenant_id.to_string(),
            "organizationId": record.organization_id.to_string(),
            "ownerUserId": record.owner_user_id.to_string(),
            "isDefault": record.is_default,
            "status": record.status.as_str(),
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!("agent_workspace_{}_{}", record.workspace_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "workspace")
        .with_context("aggregate_id", record.workspace_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.workspace.audit.v1");
        self.record_audit_event(event)
    }

    fn emit_project_composition_slot_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentProjectCompositionSlotRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "projectId": record.project_id,
            "slotId": record.slot_id,
            "slotKind": record.slot_kind.as_str(),
            "targetModule": record.target_module.as_str(),
            "targetRef": record.target_ref,
            "enabled": record.enabled,
            "priority": record.priority,
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!(
                "agent_project_slot_{}_{}_{}",
                record.project_id, record.slot_id, record.version
            ),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "composition_slot")
        .with_context("aggregate_id", record.slot_id.as_str())
        .with_context("project_id", record.project_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.project-composition-slot.audit.v1");
        self.record_audit_event(event)
    }

    fn emit_turn_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentTurnRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "turnId": record.turn_id,
            "sessionId": record.session_id,
            "agentId": record.agent_id,
            "status": record.status.as_str(),
            "errorCode": record.error_code,
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!(
                "agent_turn_{}_{}_{}",
                record.turn_id,
                action.action_code(),
                record.version
            ),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "turn")
        .with_context("aggregate_id", record.turn_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.chat-turn.audit.v1");
        self.record_audit_event(event)
    }

    fn emit_binding_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentProviderBindingRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        // Use structured JSON payload for provider binding events
        let audit_payload = ProviderBindingAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("binding audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_binding_{}_{}", record.binding_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context(
            "schema_version",
            ProviderBindingAuditPayload::SCHEMA_VERSION,
        )
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "agent")
        .with_context("aggregate_id", record.agent_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context("binding_id", record.binding_id.as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.provider_binding.v1");

        self.record_audit_event(event)
    }

    fn emit_runtime_execution_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentRuntimeExecutionRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        // Use structured JSON payload for runtime execution events
        let audit_payload = RuntimeExecutionAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!(
                "runtime execution audit payload serialization: {error}"
            ))
        })?;

        let event = KernelEvent::new(
            format!("agent_runtime_execution_{}", record.execution_id),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context(
            "schema_version",
            RuntimeExecutionAuditPayload::SCHEMA_VERSION,
        )
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "agent")
        .with_context("aggregate_id", record.agent_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.runtime_execution.v1");

        self.record_audit_event(event)
    }

    fn emit_marketplace_audit_event(
        &self,
        input: AgentBusinessAuditEventInput<'_>,
    ) -> KernelResult<()> {
        // Use structured JSON payload for marketplace events
        let audit_payload =
            MarketplaceAuditPayload::new(crate::domain::MarketplaceAuditPayloadInput {
                action: input.action,
                item_kind: input.item_kind,
                item_id: input.item_id,
                tenant_id: input.tenant_id,
                organization_id: input.organization_id,
                status: input.status,
                visibility: input.visibility,
                version: input.version,
            });
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("marketplace audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!(
                "agent_marketplace_{}_{}_{}",
                input.item_kind, input.item_id, input.version
            ),
            input.action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", MarketplaceAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", input.action.action_code())
        .with_context("aggregate_type", input.item_kind)
        .with_context("aggregate_id", input.item_id)
        .with_context("subject_id", input.subject.subject_id.as_str())
        .with_context("subject_tenant_id", input.subject.tenant_id.as_str())
        .with_context("tenant_id", input.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            input.organization_id.to_string().as_str(),
        )
        .occurred_at(input.occurred_at)
        .with_payload_schema("sdkwork.agent.business.marketplace.v1");

        self.record_audit_event(event)
    }

    fn emit_session_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentSessionRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let audit_payload = SessionAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("session audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_session_{}_{}", record.session_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", SessionAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "session")
        .with_context("aggregate_id", record.session_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.session.v1");

        self.record_audit_event(event)
    }

    fn emit_session_item_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentSessionItemRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let audit_payload = SessionItemAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("session-item audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_session_item_{}_{}", record.item_id, record.sequence),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", SessionItemAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "session_item")
        .with_context("aggregate_id", record.item_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("item_id", record.item_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.session_item.v1");

        self.record_audit_event(event)
    }

    fn emit_task_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentTaskRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let audit_payload = TaskAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("task audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_task_{}_{}", record.task_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", TaskAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "task")
        .with_context("aggregate_id", record.task_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("task_id", record.task_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.task.v1");

        self.record_audit_event(event)
    }

    fn emit_task_run_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentTaskRunRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "runId": record.run_id,
            "taskId": record.task_id,
            "sessionId": record.session_id,
            "agentId": record.agent_id,
            "triggerKind": record.trigger_kind.as_str(),
            "status": record.status.as_str(),
            "attemptCount": record.attempt_count,
            "failureClass": record.failure_class,
            "errorCode": record.error_code,
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!(
                "agent_task_run_{}_{}_{}",
                record.run_id,
                action.action_code(),
                record.version
            ),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "task_run")
        .with_context("aggregate_id", record.run_id.as_str())
        .with_context("task_id", record.task_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.task-run.audit.v1");
        self.record_audit_event(event)
    }

    // -----------------------------------------------------------------------
    // Live interaction operations
    // -----------------------------------------------------------------------

    pub fn create_interaction(
        &self,
        command: CreateInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_requested_at(&command.requested_at)?;
        parse_optional_rfc3339_datetime(command.retention_until.as_deref(), "retentionUntil")?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some(ID_PREFIX_SESSION),
        )?;
        validate_agent_id(command.path_agent_id.as_str())?;
        let interaction_id = if is_trimmed_blank(command.interaction_id.as_str()) {
            format!("{ID_PREFIX_INTERACTION}{}", self.repository.next_id()?)
        } else {
            command.interaction_id.clone()
        };
        validate_standard_id(
            interaction_id.as_str(),
            "interactionId",
            Some(ID_PREFIX_INTERACTION),
        )?;
        self.authorize(
            "agent.business.interaction.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.create",
        )?;

        require_non_blank(command.prompt.as_str(), "prompt")?;
        reject_secret_material(command.prompt.as_str(), "prompt")?;
        if command.prompt.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "prompt exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        let options_json = default_json_array_if_blank(command.options_json.as_str());
        let options_value: serde_json::Value = serde_json::from_str(&options_json)
            .map_err(|error| KernelError::validation(format!("options is invalid: {error}")))?;
        validate_interaction_options(&options_value)?;
        let request_json = command
            .request_json
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        if let Some(request_json) = request_json.as_deref() {
            validate_typed_interaction_request(command.kind, request_json)?;
            if command.provider_interaction_id.is_none() || command.runtime_binding_id.is_none() {
                return Err(KernelError::validation(
                    "typed provider interaction requires runtimeBindingId and providerInteractionId",
                ));
            }
        } else if matches!(
            command.kind,
            AgentInteractionKind::Elicitation | AgentInteractionKind::Setup
        ) {
            return Err(KernelError::validation(
                "elicitation and setup interactions require a typed request",
            ));
        }
        if command.provider_interaction_id.is_some() && command.runtime_binding_id.is_none() {
            return Err(KernelError::validation(
                "runtimeBindingId is required with providerInteractionId",
            ));
        }
        if let Some(runtime_binding_id) = command.runtime_binding_id.as_deref() {
            self.repository
                .get_session_runtime_binding(
                    command.tenant_id,
                    command.organization_id,
                    &command.session_id,
                    runtime_binding_id,
                )?
                .ok_or_else(|| KernelError::not_found("session runtime binding not found"))?;
        }

        self.repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("session not found"))
            .and_then(|session| {
                Self::ensure_session_owner_scope(&session, command.owner_scope)?;
                Self::ensure_nested_agent_id(
                    &session.agent_id,
                    command.path_agent_id.as_str(),
                    "session",
                )?;
                if session.organization_id != command.organization_id {
                    return Err(KernelError::not_found("session not found"));
                }
                if !session.status.is_active() {
                    return Err(KernelError::validation(
                        "session is not active, cannot create interaction",
                    ))?;
                }
                Ok(session)
            })?;

        if let Some(turn_id) = command.turn_id.as_deref() {
            validate_standard_id(turn_id, "turnId", Some(ID_PREFIX_TURN))?;
            let turn = self
                .repository
                .get_turn(command.tenant_id, command.organization_id, turn_id)?
                .ok_or_else(|| KernelError::not_found("turn not found"))?;
            if turn.session_id != command.session_id {
                return Err(KernelError::not_found("turn not found"));
            }
        }

        if self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                interaction_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("interaction already exists"));
        }

        let record = AgentInteractionRecord {
            id: self.repository.next_id()?,
            interaction_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id,
            turn_id: command.turn_id,
            runtime_binding_id: command.runtime_binding_id,
            provider_interaction_id: command.provider_interaction_id,
            kind: command.kind,
            status: AgentInteractionStatus::Pending,
            prompt: command.prompt,
            options_json,
            request_json,
            resolution_json: None,
            claim_owner: None,
            claim_token_hash: None,
            claim_expires_at: None,
            fencing_token: 0,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            resolved_at: None,
            retention_until: command.retention_until,
        };

        self.repository.insert_interaction(record.clone())?;
        self.emit_interaction_audit_event(
            AgentAuditAction::InteractionCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub(crate) fn persist_provider_interaction_event(
        &self,
        command: PersistProviderInteractionEventCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        if !matches!(
            command.event.event_type.as_str(),
            "agent.policy.paused" | "agent.message.paused"
        ) {
            return Err(KernelError::validation(
                "provider interaction event must be a paused event",
            ));
        }
        validate_requested_at(&command.received_at)?;
        validate_standard_id(&command.session_id, "sessionId", Some(ID_PREFIX_SESSION))?;
        validate_standard_id(&command.turn_id, "turnId", Some(ID_PREFIX_TURN))?;
        validate_agent_id(&command.path_agent_id)?;
        if command.event.session_id.as_deref() != Some(command.session_id.as_str()) {
            return Err(KernelError::validation(
                "provider interaction event Session does not match the active Turn",
            ));
        }

        let payload: serde_json::Value =
            serde_json::from_str(&command.event.payload).map_err(|error| {
                KernelError::validation(format!(
                    "provider interaction event payload must be valid JSON: {error}"
                ))
            })?;
        let payload = payload.as_object().ok_or_else(|| {
            KernelError::validation("provider interaction event payload must be an object")
        })?;
        if payload
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64)
            != Some(1)
        {
            return Err(KernelError::validation(
                "provider interaction event payload schemaVersion must be 1",
            ));
        }
        let interaction = payload
            .get("interaction")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                KernelError::validation(
                    "provider paused event must include a normalized interaction",
                )
            })?;
        if interaction
            .get("sessionId")
            .and_then(serde_json::Value::as_str)
            != Some(command.session_id.as_str())
        {
            return Err(KernelError::validation(
                "provider interaction canonical Session does not match the active Turn",
            ));
        }
        let category = required_json_string(interaction, "category", "interaction")?;
        let kind = required_json_string(interaction, "kind", "interaction")?;
        let allowed_actions = interaction
            .get("allowedActions")
            .ok_or_else(|| KernelError::validation("interaction.allowedActions is required"))?;
        let data = interaction
            .get("request")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| KernelError::validation("interaction.request must be an object"))?;
        let correlation = interaction
            .get("correlation")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| KernelError::validation("interaction.correlation must be an object"))?;
        for field in [
            "modelRequestId",
            "providerId",
            "providerInteractionId",
            "providerItemId",
            "providerRequestId",
            "providerRequestIdType",
            "providerSessionId",
            "providerToolCallId",
            "providerToolName",
            "providerToolNamespace",
            "providerTurnId",
            "protocolMethod",
        ] {
            if !correlation.contains_key(field) {
                return Err(KernelError::validation(format!(
                    "provider interaction correlation is missing {field}"
                )));
            }
        }
        let request = serde_json::json!({
            "schemaVersion": 1,
            "category": category,
            "kind": kind,
            "allowedActions": allowed_actions,
            "data": data,
            "correlation": correlation,
        });
        let interaction_kind = match category {
            "approval" => AgentInteractionKind::Approval,
            "user_input" => AgentInteractionKind::UserQuestion,
            "elicitation" => AgentInteractionKind::Elicitation,
            "setup" => AgentInteractionKind::Setup,
            _ => {
                return Err(KernelError::validation(
                    "interaction.category is unsupported",
                ))
            }
        };
        let request_json =
            serde_json::to_string(&request).map_err(|error| KernelError::Internal {
                message: format!("failed to encode provider interaction request: {error}"),
            })?;
        validate_typed_interaction_request(interaction_kind, &request_json)?;

        let model_request_id =
            required_json_string(correlation, "modelRequestId", "interaction.correlation")?;
        if command.event.run_id.as_deref() != Some(model_request_id)
            || command.event.correlation_id.as_deref() != Some(model_request_id)
        {
            return Err(KernelError::validation(
                "provider interaction model request does not match the Kernel event",
            ));
        }
        let provider_id =
            required_json_string(correlation, "providerId", "interaction.correlation")?;
        let turn = self
            .repository
            .get_turn(command.tenant_id, command.organization_id, &command.turn_id)?
            .ok_or_else(|| KernelError::not_found("turn not found"))?;
        if turn.session_id != command.session_id {
            return Err(KernelError::validation(
                "provider interaction does not match the active Turn",
            ));
        }
        let runtime_binding_id = turn.runtime_binding_id.as_deref().ok_or_else(|| {
            KernelError::validation("active Turn is missing its Session runtime binding")
        })?;
        let runtime_binding = self
            .repository
            .get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                runtime_binding_id,
            )?
            .ok_or_else(|| KernelError::not_found("session runtime binding not found"))?;
        if runtime_binding.provider_id != provider_id {
            return Err(KernelError::validation(
                "provider interaction does not match the Session runtime provider",
            ));
        }
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            None,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("session not found"));
        }

        let provider_request_id = correlation.get("providerRequestId").ok_or_else(|| {
            KernelError::validation("request.correlation.providerRequestId is required")
        })?;
        let provider_request_id_type = required_json_string(
            correlation,
            "providerRequestIdType",
            "interaction.correlation",
        )?;
        let protocol_method =
            required_json_string(correlation, "protocolMethod", "interaction.correlation")?;
        let digest_input = format!(
            "{model_request_id}:{provider_request_id_type}:{}:{protocol_method}",
            serde_json::to_string(provider_request_id).map_err(|error| KernelError::Internal {
                message: format!("failed to encode provider request identity: {error}"),
            })?
        );
        let digest = sha256_hash(digest_input.as_bytes());
        let interaction_id = format!("{ID_PREFIX_INTERACTION}{}", &digest[..32]);
        let provider_interaction_id = correlation
            .get("providerInteractionId")
            .and_then(serde_json::Value::as_str)
            .or_else(|| {
                interaction
                    .get("interactionId")
                    .and_then(serde_json::Value::as_str)
            })
            .map(str::to_string)
            .unwrap_or_else(|| provider_request_id.to_string());
        if provider_interaction_id.len() > 256 {
            return Err(KernelError::validation(
                "provider interaction id exceeds 256 bytes",
            ));
        }
        let prompt = required_json_string(interaction, "prompt", "interaction")?.to_string();
        require_non_blank(&prompt, "interaction.prompt")?;
        if prompt.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(
                "provider interaction prompt exceeds the maximum size",
            ));
        }

        if let Some(existing) = self.repository.get_interaction(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &interaction_id,
        )? {
            if existing.turn_id.as_deref() == Some(command.turn_id.as_str())
                && existing.runtime_binding_id.as_deref() == Some(runtime_binding_id)
                && existing.provider_interaction_id.as_deref()
                    == Some(provider_interaction_id.as_str())
                && existing.request_json.as_deref() == Some(request_json.as_str())
            {
                return Ok(existing);
            }
            return Err(KernelError::conflict(
                "provider interaction identity was reused with different content",
            ));
        }

        let record = AgentInteractionRecord {
            id: self.repository.next_id()?,
            interaction_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id,
            turn_id: Some(command.turn_id),
            runtime_binding_id: Some(runtime_binding_id.to_string()),
            provider_interaction_id: Some(provider_interaction_id),
            kind: interaction_kind,
            status: AgentInteractionStatus::Pending,
            prompt,
            options_json: "[]".to_string(),
            request_json: Some(request_json),
            resolution_json: None,
            claim_owner: None,
            claim_token_hash: None,
            claim_expires_at: None,
            fencing_token: 0,
            version: 0,
            created_at: command.received_at.clone(),
            updated_at: command.received_at.clone(),
            resolved_at: None,
            retention_until: None,
        };
        self.repository.insert_interaction(record.clone())?;
        self.emit_interaction_audit_event(
            AgentAuditAction::InteractionCreated,
            &record,
            command.requested_by,
            command.received_at,
        )?;
        Ok(record)
    }

    pub fn list_interactions(
        &self,
        command: ListInteractionsCommand,
    ) -> KernelResult<PaginatedResult<AgentInteractionRecord>> {
        self.authorize(
            "agent.business.interaction.list",
            command.requested_by,
            format!("agent.business.session.{}", command.query.session_id),
            "interaction.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            command.query.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::not_found("session not found"));
        }
        let scope_fingerprint = command.query.scope_fingerprint();
        if command
            .query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.scope_fingerprint != scope_fingerprint)
        {
            return Err(KernelError::validation(
                "cursor does not match the requested Interaction scope",
            ));
        }
        let page_size = command.query.pagination.page_size;
        let mut items = self.repository.list_interactions(&command.query)?;
        let has_more = items.len() > page_size;
        items.truncate(page_size);
        let next_page_token = if has_more {
            items
                .last()
                .map(|interaction| {
                    encode_created_at_cursor(
                        "interaction",
                        &CreatedAtListCursor {
                            created_at: interaction.created_at.clone(),
                            internal_id: interaction.id,
                            scope_fingerprint,
                        },
                    )
                })
                .transpose()?
        } else {
            None
        };
        Ok(PaginatedResult::new(items, next_page_token, None))
    }

    pub fn get_interaction(
        &self,
        command: GetInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        self.authorize(
            "agent.business.interaction.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "interaction.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("interaction not found"));
        }
        self.repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.interaction_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("interaction not found"))
    }

    pub fn claim_interaction(
        &self,
        command: ClaimInteractionCommand,
    ) -> KernelResult<InteractionClaimResult> {
        validate_requested_at(&command.requested_at)?;
        validate_standard_id(
            &command.interaction_id,
            "interactionId",
            Some(ID_PREFIX_INTERACTION),
        )?;
        require_non_blank(&command.claim_owner, "claimOwner")?;
        if command.claim_owner.len() > 128 {
            return Err(KernelError::validation("claimOwner exceeds 128 bytes"));
        }
        if !(1..=MAX_INTERACTION_LEASE_SECONDS).contains(&command.lease_seconds) {
            return Err(KernelError::validation(
                "leaseSeconds must be between 1 and 300",
            ));
        }
        self.authorize(
            "agent.business.interaction.claim",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.claim",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("interaction not found"));
        }
        let mut record = self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &command.interaction_id,
            )?
            .ok_or_else(|| KernelError::not_found("interaction not found"))?;
        if !record.is_pending() {
            return Err(KernelError::validation(
                "interaction is no longer pending and cannot be claimed",
            ));
        }
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "interaction",
        )?;
        let now = OffsetDateTime::now_utc();
        if let (Some(existing_owner), Some(expires_at)) = (
            record.claim_owner.as_deref(),
            record.claim_expires_at.as_deref(),
        ) {
            if parse_rfc3339_datetime(expires_at, "claimExpiresAt")? > now
                && existing_owner != command.claim_owner
            {
                return Err(KernelError::conflict(
                    "interaction is already claimed by another owner",
                ));
            }
        }
        let raw_claim_token = sdkwork_utils_rust::id::random_string(48);
        let claim_token_hash = sha256_hash(raw_claim_token.as_bytes());
        let claim_expires_at =
            format_utc_seconds(now + time::Duration::seconds(i64::from(command.lease_seconds)));
        record.claim(
            command.claim_owner,
            claim_token_hash,
            claim_expires_at.clone(),
            format_utc_seconds(now),
        );
        self.repository.update_interaction(record.clone())?;
        self.emit_interaction_audit_event(
            AgentAuditAction::InteractionClaimed,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(InteractionClaimResult {
            fencing_token: record.fencing_token,
            interaction: record,
            claim_token: raw_claim_token,
            claim_expires_at,
        })
    }

    pub fn approve_interaction(
        &self,
        command: ApproveInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_standard_id(
            command.interaction_id.as_str(),
            "interactionId",
            Some(ID_PREFIX_INTERACTION),
        )?;
        self.authorize(
            "agent.business.interaction.approve",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.approve",
        )?;

        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("interaction not found"));
        }
        let mut record = self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.interaction_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("interaction not found"))?;

        if !record.is_pending() {
            return Err(KernelError::validation(
                "interaction is no longer pending and cannot be approved",
            ));
        }

        if record.kind != AgentInteractionKind::Approval {
            return Err(KernelError::validation(
                "interaction is not an approval type; use answer instead",
            ));
        }
        if record.request_json.is_some() {
            return Err(KernelError::validation(
                "typed interaction must use the resolve operation",
            ));
        }

        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "interaction",
        )?;
        validate_interaction_claim(&record, &command.claim_token, command.fencing_token)?;

        let resolution = serde_json::json!({
            "outcome": if command.approved { "approved" } else { "rejected" },
            "reason": command.reason,
        });

        let new_status = if command.approved {
            AgentInteractionStatus::Resolved
        } else {
            AgentInteractionStatus::Rejected
        };

        record.resolve(
            new_status,
            resolution.to_string(),
            command.requested_at.as_str(),
        );

        self.repository.update_interaction(record.clone())?;

        let audit_action = if command.approved {
            AgentAuditAction::InteractionResolved
        } else {
            AgentAuditAction::InteractionRejected
        };
        self.emit_interaction_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;

        Ok(record)
    }

    pub fn answer_interaction(
        &self,
        command: AnswerInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_standard_id(
            command.interaction_id.as_str(),
            "interactionId",
            Some(ID_PREFIX_INTERACTION),
        )?;
        if !command.rejected {
            require_non_blank(command.answer.as_str(), "answer")?;
        }
        self.authorize(
            "agent.business.interaction.answer",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.answer",
        )?;

        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("interaction not found"));
        }
        let mut record = self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.interaction_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("interaction not found"))?;

        if !record.is_pending() {
            return Err(KernelError::validation(
                "interaction is no longer pending and cannot be answered",
            ));
        }

        if record.kind != AgentInteractionKind::UserQuestion {
            return Err(KernelError::validation(
                "interaction is not a user-question type; use approve instead",
            ));
        }
        if record.request_json.is_some() {
            return Err(KernelError::validation(
                "typed interaction must use the resolve operation",
            ));
        }

        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "interaction",
        )?;
        validate_interaction_claim(&record, &command.claim_token, command.fencing_token)?;

        let resolution = serde_json::json!({
            "outcome": if command.rejected { "rejected" } else { "answered" },
            "answer": command.answer,
            "selectedOptionValue": command.selected_option_value,
        });

        let new_status = if command.rejected {
            AgentInteractionStatus::Rejected
        } else {
            AgentInteractionStatus::Resolved
        };

        record.resolve(
            new_status,
            resolution.to_string(),
            command.requested_at.as_str(),
        );

        self.repository.update_interaction(record.clone())?;

        let audit_action = if command.rejected {
            AgentAuditAction::InteractionRejected
        } else {
            AgentAuditAction::InteractionResolved
        };
        self.emit_interaction_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;

        Ok(record)
    }

    pub fn resolve_interaction(
        &self,
        command: ResolveInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_standard_id(
            command.interaction_id.as_str(),
            "interactionId",
            Some(ID_PREFIX_INTERACTION),
        )?;
        self.authorize(
            "agent.business.interaction.resolve",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.resolve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::not_found("interaction not found"));
        }
        let mut record = self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.interaction_id.as_str(),
            )?
            .ok_or_else(|| KernelError::not_found("interaction not found"))?;
        if !record.is_pending() {
            return Err(KernelError::validation(
                "interaction is no longer pending and cannot be resolved",
            ));
        }
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "interaction",
        )?;
        validate_interaction_claim(&record, &command.claim_token, command.fencing_token)?;
        let request_json = record.request_json.as_deref().ok_or_else(|| {
            KernelError::validation(
                "legacy interaction must use the approve or answer compatibility operation",
            )
        })?;
        let new_status =
            validate_typed_interaction_resolution(request_json, &command.resolution_json)?;
        let request: serde_json::Value = serde_json::from_str(request_json).map_err(|error| {
            KernelError::validation(format!("request must be valid JSON: {error}"))
        })?;
        record.resolve(
            new_status,
            command.resolution_json.clone(),
            command.requested_at.as_str(),
        );
        self.repository.update_interaction(record.clone())?;
        let audit_action = if new_status == AgentInteractionStatus::Rejected {
            AgentAuditAction::InteractionRejected
        } else {
            AgentAuditAction::InteractionResolved
        };
        self.emit_interaction_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        if let Some(correlation) = request.get("correlation") {
            let correlation = correlation
                .as_object()
                .ok_or_else(|| KernelError::validation("request.correlation must be an object"))?;
            let runtime_binding_id = record.runtime_binding_id.as_deref().ok_or_else(|| {
                KernelError::validation(
                    "provider interaction is missing its Session runtime binding",
                )
            })?;
            let runtime_binding = self
                .repository
                .get_session_runtime_binding(
                    command.tenant_id,
                    command.organization_id,
                    &command.session_id,
                    runtime_binding_id,
                )?
                .ok_or_else(|| KernelError::not_found("session runtime binding not found"))?;
            let provider_id =
                required_json_string(correlation, "providerId", "request.correlation")?;
            if runtime_binding.provider_id != provider_id {
                return Err(KernelError::conflict(
                    "provider interaction no longer matches its Session runtime provider",
                ));
            }
            let provider_session_id =
                required_json_string(correlation, "providerSessionId", "request.correlation")?;
            if runtime_binding
                .provider_session_id
                .as_deref()
                .is_some_and(|value| value != provider_session_id)
            {
                return Err(KernelError::conflict(
                    "provider interaction no longer matches its provider Session",
                ));
            }
            let engine_key = engine_key_for_binding_id(&runtime_binding.provider_binding_id)
                .ok_or_else(|| {
                    KernelError::provider_error(
                        "provider_interaction_resolution_unsupported",
                        "provider binding does not expose interaction resolution",
                    )
                })?;
            let host =
                crate::runtime_facade_bridge::shared_agent_engine_host().ok_or_else(|| {
                    KernelError::provider_error(
                        "agent_engine_bootstrap_failed",
                        "shared agent engine host is unavailable",
                    )
                })?;
        // Deliver the resolution to the provider engine AFTER the interaction
        // is durably resolved (CAS): a concurrent resolution loses the CAS and
        // never reaches the engine, so the engine side effect is only ever
        // issued once for the winning write. An engine delivery failure is
        // surfaced to the caller; the durable resolution stands and provider
        // reconciliation converges the engine state.
                    let turn_id = record.turn_id.as_deref().ok_or_else(|| {
                KernelError::validation("provider interaction is missing its canonical Turn")
            })?;
            let resolution: serde_json::Value = serde_json::from_str(&command.resolution_json)
                .map_err(|error| {
                    KernelError::validation(format!("resolution must be valid JSON: {error}"))
                })?;
            host.resolve_interaction(
                engine_key,
                &AgentEngineInteractionResolution {
                    model_request_id: required_json_string(
                        correlation,
                        "modelRequestId",
                        "request.correlation",
                    )?
                    .to_string(),
                    session_id: command.session_id.clone(),
                    turn_id: turn_id.to_string(),
                    provider_session_id: provider_session_id.to_string(),
                    provider_turn_id: required_json_string(
                        correlation,
                        "providerTurnId",
                        "request.correlation",
                    )?
                    .to_string(),
                    provider_request_id: correlation.get("providerRequestId").cloned().ok_or_else(
                        || {
                            KernelError::validation(
                                "request.correlation.providerRequestId is required",
                            )
                        },
                    )?,
                    resolution,
                },
            )
            .map_err(|error| {
                KernelError::provider_error(
                    "provider_interaction_resolution_failed",
                    error.to_string(),
                )
            })?;
        }

        Ok(record)
    }

    fn emit_interaction_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentInteractionRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schema_version": "v1",
            "action": action.action_code(),
            "interaction_id": record.interaction_id,
            "session_id": record.session_id,
            "tenant_id": record.tenant_id,
            "organization_id": record.organization_id,
            "kind": record.kind.as_str(),
            "status": record.status.as_str(),
            "version": record.version,
        })
        .to_string();

        let event = KernelEvent::new(
            format!(
                "agent_interaction_{}_{}",
                record.interaction_id, record.version
            ),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "interaction")
        .with_context("aggregate_id", record.interaction_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("interaction_id", record.interaction_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.interaction.v1");

        self.record_audit_event(event)
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_session_resource_audit_event(
        &self,
        action: AgentAuditAction,
        resource_kind: &str,
        resource_id: &str,
        session_id: &str,
        tenant_id: u64,
        organization_id: u64,
        version: u64,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schema_version": "v1",
            "action": action.action_code(),
            "resource_kind": resource_kind,
            "resource_id": resource_id,
            "session_id": session_id,
            "tenant_id": tenant_id,
            "organization_id": organization_id,
            "version": version,
        })
        .to_string();
        let event = KernelEvent::new(
            format!("agent_{resource_kind}_{resource_id}_{version}"),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", resource_kind)
        .with_context("aggregate_id", resource_id)
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("resource_kind", resource_kind)
        .with_context("resource_id", resource_id)
        .with_context("session_id", session_id)
        .with_context("tenant_id", tenant_id.to_string().as_str())
        .with_context("organization_id", organization_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.session-resource.v1");
        self.record_audit_event(event)
    }

    fn record_audit_event(&self, event: KernelEvent) -> KernelResult<()> {
        validate_audit_aggregate_context(&event)?;
        // Audit persistence is best-effort by design: the business write is
        // the authority and must not be rolled back (or reported as failed)
        // when the audit sink is unavailable. Failures surface through
        // structured logs and sink-side monitoring so compliance gaps stay
        // detectable without corrupting the client-visible outcome (H2).
        if let Err(error) = self.audit_sink.record(event) {
            tracing::error!(
                target: "sdkwork.agents.audit",
                error = %error,
                "failed to persist audit event; business operation already committed"
            );
        }
        Ok(())
    }

    /// Checkpoints accumulated streaming deltas onto the running turn row so
    /// a crash during a long turn retains the partial reply (H4). Called by
    /// the HTTP stream sink on a throttle; the durable adapter only accepts
    /// writes while the turn is pending/running.
    pub(crate) fn checkpoint_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
        content: &str,
    ) -> KernelResult<()> {
        let updated_at = format_utc_seconds(OffsetDateTime::now_utc());
        self.repository.append_turn_streaming_content(
            tenant_id,
            organization_id,
            turn_id,
            content,
            &updated_at,
        )
    }

    /// Clears the streaming checkpoint after the turn completes durably.
    pub(crate) fn clear_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<()> {
        self.repository
            .clear_turn_streaming_content(tenant_id, organization_id, turn_id)
    }
}

fn normalize_provider_session_directory(
    mut directory: sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry,
) -> KernelResult<sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry> {
    directory.title =
        normalize_optional_bounded(directory.title.take(), "providerDirectory.title", 512)?;
    directory.title_source = normalize_optional_bounded(
        directory.title_source.take(),
        "providerDirectory.titleSource",
        64,
    )?;
    directory.preview =
        normalize_optional_bounded(directory.preview.take(), "providerDirectory.preview", 4096)?;
    directory.source =
        normalize_optional_bounded(directory.source.take(), "providerDirectory.source", 256)?;
    directory.sort_key = trim(&directory.sort_key).to_string();
    if directory.sort_key.len() > 512 {
        return Err(KernelError::validation(
            "providerDirectory.sortKey exceeds 512 bytes",
        ));
    }
    for (value, field) in [
        (
            directory.created_at.as_deref(),
            "providerDirectory.createdAt",
        ),
        (
            directory.updated_at.as_deref(),
            "providerDirectory.updatedAt",
        ),
        (
            directory.recency_at.as_deref(),
            "providerDirectory.recencyAt",
        ),
    ] {
        parse_optional_rfc3339_datetime(value, field)?;
    }
    Ok(directory)
}

fn runtime_binding_provider_directory_matches(
    record: &AgentSessionRuntimeBindingRecord,
    directory: &sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry,
) -> bool {
    record.provider_title == directory.title
        && record.provider_title_source == directory.title_source
        && record.provider_preview == directory.preview
        && record.provider_created_at == directory.created_at
        && record.provider_updated_at == directory.updated_at
        && record.provider_recency_at == directory.recency_at
        && record.provider_pinned == directory.pinned
        && record.provider_archived == directory.archived
        && record.provider_visible == directory.visible
        && record.provider_sort_key
            == (!directory.sort_key.is_empty()).then(|| directory.sort_key.clone())
        && record.provider_source == directory.source
}

fn apply_provider_session_directory(
    record: &mut AgentSessionRuntimeBindingRecord,
    directory: sdkwork_agents_runtime_facade::ProviderSessionDirectoryEntry,
) {
    record.provider_title = directory.title;
    record.provider_title_source = directory.title_source;
    record.provider_preview = directory.preview;
    record.provider_created_at = directory.created_at;
    record.provider_updated_at = directory.updated_at;
    record.provider_recency_at = directory.recency_at;
    record.provider_pinned = directory.pinned;
    record.provider_archived = directory.archived;
    record.provider_visible = directory.visible;
    record.provider_sort_key = (!directory.sort_key.is_empty()).then_some(directory.sort_key);
    record.provider_source = directory.source;
}

/// JSON field name under which structured context metadata is embedded
/// within the `KernelEvent` payload by [`KernelEventExt::with_context`].
///
/// The value is a flat JSON object of `key → string` pairs that
/// supplements the audit payload with routing metadata (tenant_id,
/// agent_id, subject_id, etc.).  Keeping the context in a dedicated
/// sub-object preserves the integrity of the outer JSON payload.
const AUDIT_CONTEXT_FIELD: &str = "_context";

fn validate_audit_aggregate_context(event: &KernelEvent) -> KernelResult<()> {
    let payload = serde_json::from_str::<serde_json::Value>(event.payload.as_str())
        .map_err(|_| KernelError::validation("audit event payload must be valid JSON"))?;
    let context = payload
        .get(AUDIT_CONTEXT_FIELD)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| KernelError::validation("audit event context is required"))?;
    let aggregate_type = context
        .get("aggregate_type")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| KernelError::validation("audit aggregate_type context is required"))?;
    let aggregate_id = context
        .get("aggregate_id")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| KernelError::validation("audit aggregate_id context is required"))?;
    if !matches!(
        aggregate_type,
        "agent"
            | "runtime_binding"
            | "composition_slot"
            | "workspace"
            | "project"
            | "project_member"
            | "session"
            | "turn"
            | "session_item"
            | "item_feedback"
            | "interaction"
            | "checkpoint"
            | "task"
            | "share_link"
    ) {
        return Err(KernelError::validation(format!(
            "unsupported audit aggregate_type context: {aggregate_type}"
        )));
    }
    if aggregate_id.len() > 128 {
        return Err(KernelError::validation(
            "audit aggregate_id context exceeds 128 bytes",
        ));
    }
    Ok(())
}

/// Extension trait that attaches structured context metadata to a
/// [`KernelEvent`] without corrupting its JSON payload.
///
/// Each call to `with_context` parses the event payload as a JSON
/// object, obtains (or creates) the [`AUDIT_CONTEXT_FIELD`] sub-object,
/// inserts the `key → value` pair, and re-serialises.  The payload
/// therefore remains valid JSON at all times, in contrast to the
/// previous string-concatenation approach that produced malformed
/// `{"json":...};key=value` strings.
trait KernelEventExt {
    fn with_context(self, key: impl Into<String>, value: impl Into<String>) -> Self;
}

impl KernelEventExt for KernelEvent {
    fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let value = value.into();

        // All audit event payloads produced by `emit_*_audit_event`
        // are serialised JSON objects, so parsing should succeed.  If
        // the payload is unexpectedly not a JSON object, we silently
        // skip the context addition rather than corrupting it.
        if let Ok(mut payload) = serde_json::from_str::<serde_json::Value>(self.payload.as_str()) {
            if let Some(obj) = payload.as_object_mut() {
                let context = obj
                    .entry(AUDIT_CONTEXT_FIELD)
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(ctx) = context.as_object_mut() {
                    ctx.insert(key, serde_json::Value::String(value));
                    if let Ok(serialised) = serde_json::to_string(&payload) {
                        self.payload = serialised;
                    }
                }
            }
        }
        self
    }
}

fn is_valid_status_transition(from: AgentBusinessStatus, to: AgentBusinessStatus) -> bool {
    matches!(
        (from, to),
        (AgentBusinessStatus::Draft, AgentBusinessStatus::Active)
            | (AgentBusinessStatus::Draft, AgentBusinessStatus::Archived)
            | (AgentBusinessStatus::Active, AgentBusinessStatus::Disabled)
            | (AgentBusinessStatus::Active, AgentBusinessStatus::Archived)
            | (AgentBusinessStatus::Disabled, AgentBusinessStatus::Active)
            | (AgentBusinessStatus::Disabled, AgentBusinessStatus::Archived)
            | (AgentBusinessStatus::Archived, AgentBusinessStatus::Active)
            | (AgentBusinessStatus::Archived, AgentBusinessStatus::Disabled)
            | (AgentBusinessStatus::Deleted, AgentBusinessStatus::Active)
            | (_, AgentBusinessStatus::Deleted)
    ) || from == to
}

fn validate_agent_id(value: &str) -> KernelResult<()> {
    validate_standard_id(value, "agentId", Some(ID_PREFIX_AGENT))
}

fn validate_project_drive_access(
    visibility: AgentProjectVisibility,
    drive_access_mode: AgentProjectDriveAccessMode,
) -> KernelResult<()> {
    if visibility == AgentProjectVisibility::Shared
        && drive_access_mode == AgentProjectDriveAccessMode::OwnerLibrary
    {
        return Err(KernelError::validation(
            "shared projects cannot access the owner's private Drive library",
        ));
    }
    Ok(())
}

fn validate_project_composition_slot_fields(
    slot_kind: AgentCompositionSlotKind,
    target_module: AgentCompositionTargetModule,
    target_ref: &str,
    target_version_ref: Option<&str>,
    priority: i32,
    policy_json: &str,
) -> KernelResult<()> {
    validate_composition_slot_mapping(slot_kind, target_module)?;
    require_non_blank(target_ref, "targetRef")?;
    if target_ref != trim(target_ref) {
        return Err(KernelError::validation(
            "targetRef must not contain leading or trailing whitespace",
        ));
    }
    if target_ref.chars().count() > 256 {
        return Err(KernelError::validation(
            "targetRef must be at most 256 characters",
        ));
    }
    reject_secret_material(target_ref, "targetRef")?;
    validate_optional_plain_ref(target_version_ref, "targetVersionRef")?;
    if !(-10_000..=10_000).contains(&priority) {
        return Err(KernelError::validation(
            "priority must be between -10000 and 10000",
        ));
    }
    if policy_json.len() > 16 * 1024 {
        return Err(KernelError::validation("policyJson exceeds 16384 bytes"));
    }
    let policy: serde_json::Value = serde_json::from_str(policy_json).map_err(|error| {
        KernelError::validation(format!("policyJson must be valid JSON: {error}"))
    })?;
    if !policy.is_object() {
        return Err(KernelError::validation("policyJson must be a JSON object"));
    }
    Ok(())
}

fn validate_composition_slot_mapping(
    slot_kind: AgentCompositionSlotKind,
    target_module: AgentCompositionTargetModule,
) -> KernelResult<()> {
    if !slot_kind.matches_target_module(target_module) {
        return Err(KernelError::validation(
            "slotKind does not match targetModule",
        ));
    }
    Ok(())
}

fn validate_optional_plain_ref(value: Option<&str>, field_name: &str) -> KernelResult<()> {
    if let Some(value) = value {
        validate_non_empty(value, field_name)?;
        reject_secret_material(value, field_name)?;
        if value.trim() != value {
            return Err(KernelError::validation(format!(
                "{field_name} must not contain leading or trailing whitespace"
            )));
        }
        if value.chars().count() > 128 {
            return Err(KernelError::validation(format!(
                "{field_name} must be at most 128 characters"
            )));
        }
    }
    Ok(())
}

fn ensure_expected_version(
    actual_version: u64,
    expected_version: Option<u64>,
    entity_name: &str,
) -> KernelResult<()> {
    let expected_version = expected_version.ok_or_else(|| {
        KernelError::validation(format!("{entity_name} mutation requires expectedVersion"))
    })?;
    if actual_version != expected_version {
        return Err(KernelError::conflict(format!(
            "{entity_name} version mismatch: expected={expected_version}, actual={actual_version}"
        )));
    }
    Ok(())
}

fn normalized_workspace_name(name: &str) -> KernelResult<String> {
    require_non_blank(name, "name")?;
    let normalized = trim(name).to_string();
    if normalized.len() > 255 {
        return Err(KernelError::validation("name exceeds 255 bytes"));
    }
    Ok(normalized)
}

fn validate_non_empty(value: &str, field_name: &str) -> KernelResult<()> {
    require_non_blank(value, field_name)
}

fn validate_json_payload(value: &str, field_name: &str) -> KernelResult<()> {
    validate_bounded_json_payload(value, field_name, MAX_JSON_PAYLOAD_BYTES)
}

fn validate_bounded_json_payload(
    value: &str,
    field_name: &str,
    max_bytes: usize,
) -> KernelResult<()> {
    if value.len() > max_bytes {
        return Err(KernelError::validation(format!(
            "{field_name} exceeds {max_bytes} bytes"
        )));
    }
    let _: serde_json::Value = serde_json::from_str(value).map_err(|error| {
        KernelError::validation(format!("{field_name} must be valid JSON: {error}"))
    })?;
    Ok(())
}

fn validate_provider_session_item_payload(
    command: &ReconcileProviderSessionHistoryItemCommand,
    content: Option<&str>,
) -> KernelResult<()> {
    match command.kind {
        AgentSessionItemKind::ToolCall => {
            if command.tool_name.as_deref().is_none_or(str::is_empty)
                || command.tool_call_id.as_deref().is_none_or(str::is_empty)
                || command.tool_arguments_json.is_none()
                || command.tool_result_json.is_some()
                || content.is_some()
            {
                return Err(KernelError::validation(
                    "provider tool call payload is invalid",
                ));
            }
        }
        AgentSessionItemKind::ToolResult => {
            if command.tool_call_id.as_deref().is_none_or(str::is_empty)
                || command.tool_arguments_json.is_some()
                || command.tool_result_json.is_none()
                || content.is_some()
            {
                return Err(KernelError::validation(
                    "provider tool result payload is invalid",
                ));
            }
        }
        AgentSessionItemKind::ArtifactReference => {
            if command.tool_name.is_some()
                || command.tool_call_id.is_some()
                || command.tool_arguments_json.is_some()
                || command.tool_result_json.is_some()
            {
                return Err(KernelError::validation(
                    "provider artifact payload must not contain tool fields",
                ));
            }
        }
        _ => {
            if content.is_none()
                || command.tool_name.is_some()
                || command.tool_call_id.is_some()
                || command.tool_arguments_json.is_some()
                || command.tool_result_json.is_some()
            {
                return Err(KernelError::validation("provider text payload is invalid"));
            }
        }
    }
    Ok(())
}

/// Returns true when a terminal narrative snapshot must replace the stored
/// provider Session history item. Legacy rows can carry a non-RFC3339
/// updated_at because older transcript synchronizations persisted provider
/// timestamps verbatim; such rows are replaceable so a newer snapshot repairs
/// the stored timestamp instead of failing the whole synchronization. A
/// requested_at that is not RFC3339 is never newer, so unparsable command
/// timestamps cannot overwrite terminal history.
fn is_newer_provider_session_item_snapshot(existing_updated_at: &str, requested_at: &str) -> bool {
    let Ok(existing_updated_at) = parse_rfc3339_datetime(existing_updated_at, "existing.updatedAt")
    else {
        return true;
    };
    parse_rfc3339_datetime(requested_at, "requestedAt")
        .is_ok_and(|requested_at| requested_at > existing_updated_at)
}

fn reject_secret_material(value: &str, field_name: &str) -> KernelResult<()> {
    let normalized = value.to_lowercase();
    for marker in [
        "api_key=",
        "apikey=",
        "access_token=",
        "refresh_token=",
        "secret=",
        "password=",
        "bearer ",
        "sk-",
    ] {
        if normalized.contains(marker) {
            return Err(KernelError::validation(format!(
                "{field_name} must not contain plaintext secret material"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod task_tests {
    use super::*;
    use crate::agent_turn::AgentTurnMode;
    use crate::turn_runtime::{execute_agent_turn, inference_error, TurnExecutionInput};
    use crate::application::{
        CancelTaskCommand, CreateAgentCommand, CreateTaskCommand, ExecuteTaskCommand,
        GetTaskCommand, ListTasksCommand,
    };
    use crate::domain::{AgentBusinessStatus, AgentSessionEntrySurface, AgentSessionKind};
    use crate::infrastructure::{
        IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use crate::ports::TaskListQuery;
    use crate::turn_runtime::{TurnCancellationOutput, TurnExecutionOutput};
    use sdkwork_agent_kernel::{AgentManifest, PolicySubject};
    use std::sync::{Condvar, Mutex};
    use std::time::Duration;

    fn sample_subject() -> PolicySubject {
        PolicySubject {
            subject_id: "user.100".to_string(),
            tenant_id: "100001".to_string(),
            roles: vec!["ai.agents.manage".to_string()],
        }
    }

    fn test_policy_provider() -> IamGatedPolicyProvider {
        IamGatedPolicyProvider::new("policy.agents.test.iam-gated")
    }

    fn sample_manifest(agent_id: &str) -> AgentManifest {
        AgentManifest {
            schema_version: "1.0.0".to_string(),
            manifest_type: "agent".to_string(),
            agent_id: agent_id.to_string(),
            name: "tasks-demo".to_string(),
            display_name: "Tasks Demo".to_string(),
            description: "sample".to_string(),
            version: "0.1.0".to_string(),
            domain: "intelligence".to_string(),
            required_capabilities: vec!["model.chat".to_string()],
            optional_capabilities: vec![],
            required_capability_requirements: vec![],
            optional_capability_requirements: vec![],
            event_families: vec!["agent.lifecycle".to_string()],
            owner_name: "sdkwork".to_string(),
            status: "active".to_string(),
        }
    }

    fn create_agent_cmd(
        agent_id: &str,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        code: &str,
        display_name: &str,
        requested_at: &str,
    ) -> CreateAgentCommand {
        CreateAgentCommand {
            agent_id: agent_id.to_string(),
            tenant_id,
            organization_id,
            owner_user_id,
            code: code.to_string(),
            display_name: display_name.to_string(),
            description: None,
            manifest: sample_manifest(agent_id),
            visibility: AgentVisibility::Private,
            tags: Vec::new(),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: sample_subject(),
            requested_at: requested_at.to_string(),
        }
    }

    type TaskTestService =
        AgentsService<InMemoryAgentRepository, InMemoryAgentAuditSink, IamGatedPolicyProvider>;

    #[derive(Debug, Default)]
    struct BlockingCancellationState {
        started: bool,
        released: bool,
        cancelled_model_request_id: Option<String>,
    }

    #[derive(Debug, Default)]
    struct BlockingCancellationExecutor {
        state: Mutex<BlockingCancellationState>,
        changed: Condvar,
    }

    impl BlockingCancellationExecutor {
        fn wait_until_started(&self) {
            let state = self
                .state
                .lock()
                .expect("blocking cancellation executor state should lock");
            let (state, timeout) = self
                .changed
                .wait_timeout_while(state, Duration::from_secs(10), |state| !state.started)
                .expect("blocking cancellation executor start wait should not be poisoned");
            assert!(state.started, "turn provider did not start before timeout");
            assert!(!timeout.timed_out(), "turn provider start wait timed out");
        }

        fn release(&self) {
            let mut state = self
                .state
                .lock()
                .expect("blocking cancellation executor state should lock");
            state.released = true;
            self.changed.notify_all();
        }

        fn cancelled_model_request_id(&self) -> Option<String> {
            self.state
                .lock()
                .expect("blocking cancellation executor state should lock")
                .cancelled_model_request_id
                .clone()
        }
    }

    impl TurnExecutor for BlockingCancellationExecutor {
        fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
            let mut state = self
                .state
                .lock()
                .expect("blocking cancellation executor state should lock");
            state.started = true;
            self.changed.notify_all();
            let (state, timeout) = self
                .changed
                .wait_timeout_while(state, Duration::from_secs(10), |state| !state.released)
                .expect("blocking cancellation executor release wait should not be poisoned");
            assert!(
                state.released,
                "turn provider was not released before timeout"
            );
            assert!(!timeout.timed_out(), "turn provider release wait timed out");
            drop(state);

            TurnExecutionOutput {
                model_request_id: Some(input.model_request_id.clone()),
                finish_reason: Some("stop".to_string()),
                content: "late provider completion".to_string(),
                model_id: input.model_id.clone(),
                provider_id: input.provider_id.clone(),
                provider_session_id: input.provider_session_id.clone(),
                input_tokens: 3,
                output_tokens: 5,
                runtime_mode: "in-memory-blocking-provider",
                stream_deltas: Vec::new(),
                stream_events: Vec::new(),
            }
        }

        fn cancel(&self, input: &TurnCancellationInput) -> KernelResult<TurnCancellationOutput> {
            self.state
                .lock()
                .expect("blocking cancellation executor state should lock")
                .cancelled_model_request_id = Some(input.model_request_id.clone());
            Ok(TurnCancellationOutput {
                model_request_id: input.model_request_id.clone(),
                finish_reason: "cancelled".to_string(),
            })
        }
    }

    fn create_task_cmd(
        service: &TaskTestService,
        tenant_id: u64,
        organization_id: u64,
        agent_id: &str,
        owner_user_id: u64,
        prompt: &str,
        requested_at: &str,
    ) -> CreateTaskCommand {
        let digest = sha256_hash(
            format!("{tenant_id}:{organization_id}:{agent_id}:{owner_user_id}:{requested_at}")
                .as_bytes(),
        );
        let session_id = format!("{ID_PREFIX_SESSION}test.{}", &digest[..20]);
        service
            .create_session(CreateSessionCommand {
                tenant_id,
                organization_id,
                agent_id: agent_id.to_string(),
                owner_user_id,
                project_id: None,
                session_id: session_id.clone(),
                session_kind: AgentSessionKind::Automation,
                entry_surface: AgentSessionEntrySurface::Automation,
                source_module: Some("sdkwork-agents".to_string()),
                source_context_kind: Some("agent_task".to_string()),
                source_context_id: Some(format!("task-test.{}", &digest[..20])),
                parent_session_id: None,
                forked_from_turn_id: None,
                title: None,
                idempotency_key: None,
                payload_hash: None,
                requested_by: sample_subject(),
                requested_at: requested_at.to_string(),
            })
            .expect("create task Session");
        CreateTaskCommand {
            tenant_id,
            organization_id,
            agent_id: agent_id.to_string(),
            owner_user_id,
            session_id,
            task_id: String::new(),
            title: None,
            prompt: prompt.to_string(),
            schedule_kind: crate::AgentTaskScheduleKind::OneTime,
            cron_expression: None,
            timezone: "UTC".to_string(),
            scheduled_at: Some("2027-06-01T00:00:00Z".to_string()),
            starts_at: None,
            ends_at: None,
            misfire_policy: crate::AgentTaskMisfirePolicy::FireOnce,
            overlap_policy: crate::AgentTaskOverlapPolicy::Skip,
            max_concurrent_runs: 1,
            max_catch_up_runs: 1,
            max_attempts: 3,
            retry_initial_delay_seconds: 5,
            retry_max_delay_seconds: 300,
            timeout_seconds: 900,
            priority: 0,
            external_ref: None,
            metadata_json: "{}".to_string(),
            requested_by: sample_subject(),
            requested_at: requested_at.to_string(),
        }
    }

    #[test]
    fn canonical_agent_engine_identity_bootstrap_does_not_require_manage_permission() {
        let service = AgentsService::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let read_subject = PolicySubject {
            subject_id: "user.100".to_string(),
            tenant_id: "100001".to_string(),
            roles: vec!["ai.agents.read".to_string()],
        };

        service
            .ensure_agent_engine_runtime_identity(
                100_001,
                0,
                100,
                "codex",
                "agent.codex",
                "binding.codex",
                "provider.codex",
                read_subject.clone(),
                "2026-07-26T15:00:00Z",
            )
            .expect("canonical identity bootstrap");

        let agent = service
            .get_agent(GetAgentCommand {
                tenant_id: 100_001,
                agent_id: "agent.codex".to_string(),
                requested_by: read_subject.clone(),
            })
            .expect("canonical agent");
        assert_eq!(agent.status, AgentBusinessStatus::Active);
        let binding = service
            .get_provider_binding(100_001, "agent.codex", "binding.codex", read_subject)
            .expect("canonical provider binding");
        assert!(binding.active);
        assert_eq!(binding.provider_id, "provider.codex");
    }

    #[test]
    fn create_and_list_tasks_roundtrip() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.demo",
                100_001,
                0,
                100,
                "tasks-demo",
                "Tasks Demo",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        service
            .change_status(ChangeAgentStatusCommand {
                tenant_id: 100_001,
                agent_id: created.agent_id.clone(),
                expected_version: Some(created.version),
                target_status: AgentBusinessStatus::Active,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:00:30Z".to_string(),
            })
            .expect("activate agent");

        let mut command = create_task_cmd(
            &service,
            100_001,
            0,
            &created.agent_id,
            100,
            "Run nightly data sync",
            "2026-06-01T05:01:00Z",
        );
        command.title = Some("Nightly sync".to_string());
        command.metadata_json = r#"{"autoExecute":true}"#.to_string();
        let task = service.create_task(command).expect("create task");

        assert!(task.task_id.starts_with(ID_PREFIX_TASK));
        assert_eq!(task.status, AgentTaskStatus::Active);
        assert!(task.completed_at.is_none());

        let listed = service
            .list_tasks(ListTasksCommand {
                query: TaskListQuery::for_organization(100_001, 0).for_agent(created.agent_id),
                requested_by: sample_subject(),
            })
            .expect("list tasks");
        assert_eq!(listed.items.len(), 1);
        assert_eq!(listed.items[0].task_id, task.task_id);
    }

    #[test]
    fn task_list_cursor_is_continuous_and_scope_bound() {
        let service = AgentsService::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let agent = service
            .create_agent(create_agent_cmd(
                "agent.tasks.cursor",
                100_001,
                0,
                100,
                "tasks-cursor",
                "Tasks Cursor",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        for index in 1..=3 {
            let requested_at = format!("2026-06-01T05:0{index}:00Z");
            let mut command = create_task_cmd(
                &service,
                100_001,
                0,
                &agent.agent_id,
                100,
                format!("Cursor task {index}").as_str(),
                &requested_at,
            );
            command.task_id = format!("{ID_PREFIX_TASK}cursor-{index}");
            service.create_task(command).expect("create cursor task");
        }

        let first_page = service
            .list_tasks(ListTasksCommand {
                query: TaskListQuery::for_organization(100_001, 0).with_cursor_page(2, None),
                requested_by: sample_subject(),
            })
            .expect("list first cursor page");
        assert_eq!(
            first_page
                .items
                .iter()
                .map(|task| task.task_id.as_str())
                .collect::<Vec<_>>(),
            vec!["task.cursor-3", "task.cursor-2"]
        );
        assert!(first_page.has_more);
        assert_eq!(first_page.total_count, None);
        let cursor = crate::task_execution_cursor::decode_task_cursor(
            first_page
                .next_page_token
                .as_deref()
                .expect("first page cursor"),
        )
        .expect("decode task cursor");

        let second_page = service
            .list_tasks(ListTasksCommand {
                query: TaskListQuery::for_organization(100_001, 0)
                    .with_cursor_page(2, Some(cursor.clone())),
                requested_by: sample_subject(),
            })
            .expect("list second cursor page");
        assert_eq!(second_page.items.len(), 1);
        assert_eq!(second_page.items[0].task_id, "task.cursor-1");
        assert!(!second_page.has_more);
        assert!(second_page.next_page_token.is_none());
        assert_eq!(second_page.total_count, None);

        let scope_error = service
            .list_tasks(ListTasksCommand {
                query: TaskListQuery::for_organization(100_001, 0)
                    .for_owner(100)
                    .with_cursor_page(2, Some(cursor)),
                requested_by: sample_subject(),
            })
            .expect_err("cursor must not cross Task query scopes");
        assert!(scope_error
            .to_string()
            .contains("cursor does not match the requested Task scope"));
    }

    #[test]
    fn task_definition_field_limits_are_enforced() {
        assert!(validate_task_definition_fields(
            &Some("t".repeat(MAX_TASK_TITLE_BYTES)),
            &Some("0 0 9 * * *".to_string()),
            "UTC",
            &Some("e".repeat(MAX_TASK_EXTERNAL_REF_BYTES)),
        )
        .is_ok());

        for (title, cron_expression, timezone, external_ref, expected_message) in [
            (
                Some("t".repeat(MAX_TASK_TITLE_BYTES + 1)),
                None,
                "UTC".to_string(),
                None,
                "title exceeds 512 bytes",
            ),
            (
                None,
                Some("c".repeat(MAX_TASK_CRON_EXPRESSION_BYTES + 1)),
                "UTC".to_string(),
                None,
                "cronExpression exceeds 256 bytes",
            ),
            (
                None,
                None,
                "z".repeat(MAX_TASK_TIMEZONE_BYTES + 1),
                None,
                "timezone exceeds 128 bytes",
            ),
            (
                None,
                None,
                "UTC".to_string(),
                Some("e".repeat(MAX_TASK_EXTERNAL_REF_BYTES + 1)),
                "externalRef exceeds 256 bytes",
            ),
        ] {
            let error =
                validate_task_definition_fields(&title, &cron_expression, &timezone, &external_ref)
                    .expect_err("oversized Task field must be rejected");
            assert!(error.to_string().contains(expected_message));
        }
    }

    #[test]
    fn cancel_task_rejects_terminal_status() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.cancel",
                100_001,
                0,
                100,
                "tasks-cancel",
                "Tasks Cancel",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        let task = service
            .create_task(create_task_cmd(
                &service,
                100_001,
                0,
                &created.agent_id,
                100,
                "Do work",
                "2026-06-01T05:01:00Z",
            ))
            .expect("create task");

        let cancelled = service
            .cancel_task(CancelTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: task.agent_id.clone(),
                task_id: task.task_id.clone(),
                expected_version: Some(task.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:02:00Z".to_string(),
            })
            .expect("cancel task");
        assert_eq!(cancelled.status, AgentTaskStatus::Cancelled);

        let error = service
            .cancel_task(CancelTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: cancelled.agent_id.clone(),
                task_id: task.task_id,
                expected_version: Some(cancelled.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:03:00Z".to_string(),
            })
            .expect_err("second cancel must fail");
        assert!(error.to_string().contains("cannot be cancelled"));
    }

    #[test]
    fn execute_task_creates_idempotent_pending_run() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.execute",
                100_001,
                0,
                100,
                "tasks-execute",
                "Tasks Execute",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        let task = service
            .create_task(create_task_cmd(
                &service,
                100_001,
                0,
                &created.agent_id,
                100,
                "Run scheduled work",
                "2026-06-01T05:01:00Z",
            ))
            .expect("create task");
        assert_eq!(task.status, AgentTaskStatus::Active);

        let executed = service
            .execute_task(ExecuteTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: created.agent_id.clone(),
                task_id: task.task_id.clone(),
                idempotency_key: "task-run.execute-test".to_string(),
                expected_version: Some(task.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:02:00Z".to_string(),
            })
            .expect("execute task");
        assert_eq!(executed.status, crate::AgentTaskRunStatus::Pending);
        assert_eq!(executed.task_id, task.task_id);
        assert_eq!(executed.session_id, task.session_id);

        let duplicate = service
            .execute_task(ExecuteTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: created.agent_id,
                task_id: task.task_id,
                idempotency_key: "task-run.execute-test".to_string(),
                expected_version: Some(task.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:02:01Z".to_string(),
            })
            .expect("repeat idempotent execute task");
        assert_eq!(duplicate.run_id, executed.run_id);
    }

    #[test]
    fn get_task_rejects_foreign_owner_scope() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.owner",
                100_001,
                0,
                100,
                "tasks-owner",
                "Tasks Owner",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        let path_agent_id = created.agent_id.clone();
        let task = service
            .create_task(create_task_cmd(
                &service,
                100_001,
                0,
                &path_agent_id,
                100,
                "Private task",
                "2026-06-01T05:01:00Z",
            ))
            .expect("create task");

        let error = service
            .get_task(GetTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id,
                task_id: task.task_id,
                owner_scope: Some(999),
                requested_by: sample_subject(),
            })
            .expect_err("foreign owner must not read task");
        assert!(error.to_string().contains("task not found"));
    }

    #[test]
    fn task_operations_are_isolated_by_organization_within_a_tenant() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let organization_one_agent = service
            .create_agent(create_agent_cmd(
                "agent.tasks.org-one",
                100_001,
                10,
                100,
                "tasks-org-one",
                "Tasks Org One",
                "2026-06-01T06:00:00Z",
            ))
            .expect("create organization one agent");
        let organization_two_agent = service
            .create_agent(create_agent_cmd(
                "agent.tasks.org-two",
                100_001,
                20,
                200,
                "tasks-org-two",
                "Tasks Org Two",
                "2026-06-01T06:00:01Z",
            ))
            .expect("create organization two agent");

        let mut cross_organization_command = create_task_cmd(
            &service,
            100_001,
            20,
            &organization_two_agent.agent_id,
            200,
            "Must not be created",
            "2026-06-01T06:01:00Z",
        );
        cross_organization_command.agent_id = organization_one_agent.agent_id.clone();
        cross_organization_command.task_id = "task.cross-organization".to_string();
        let cross_organization_create = service
            .create_task(cross_organization_command)
            .expect_err("an agent from another organization must not be accepted");
        assert!(cross_organization_create
            .to_string()
            .contains("agent not found"));

        let mut organization_one_command = create_task_cmd(
            &service,
            100_001,
            10,
            &organization_one_agent.agent_id,
            100,
            "Organization one work",
            "2026-06-01T06:02:00Z",
        );
        organization_one_command.task_id = "task.shared-id".to_string();
        let organization_one_task = service
            .create_task(organization_one_command)
            .expect("create organization one task");
        let mut organization_two_command = create_task_cmd(
            &service,
            100_001,
            20,
            &organization_two_agent.agent_id,
            200,
            "Organization two work",
            "2026-06-01T06:02:01Z",
        );
        organization_two_command.task_id = "task.shared-id".to_string();
        let organization_two_task = service
            .create_task(organization_two_command)
            .expect("the same task id is valid in another organization");
        assert_ne!(
            organization_one_task.session_id,
            organization_two_task.session_id
        );

        let organization_one_tasks = service
            .list_tasks(ListTasksCommand {
                query: TaskListQuery::for_organization(100_001, 10),
                requested_by: sample_subject(),
            })
            .expect("list organization one tasks");
        assert_eq!(organization_one_tasks.total_count, None);
        assert!(!organization_one_tasks.has_more);
        assert!(organization_one_tasks.next_page_token.is_none());
        assert_eq!(organization_one_tasks.items[0].organization_id, 10);
        let organization_two_tasks = service
            .list_tasks(ListTasksCommand {
                query: TaskListQuery::for_organization(100_001, 20),
                requested_by: sample_subject(),
            })
            .expect("list organization two tasks");
        assert_eq!(organization_two_tasks.total_count, None);
        assert!(!organization_two_tasks.has_more);
        assert!(organization_two_tasks.next_page_token.is_none());
        assert_eq!(organization_two_tasks.items[0].organization_id, 20);

        let organization_one_read = service
            .get_task(GetTaskCommand {
                tenant_id: 100_001,
                organization_id: 10,
                path_agent_id: organization_one_agent.agent_id.clone(),
                task_id: organization_one_task.task_id.clone(),
                owner_scope: None,
                requested_by: sample_subject(),
            })
            .expect("read organization one task");
        assert_eq!(organization_one_read.prompt, "Organization one work");
        let organization_two_read = service
            .get_task(GetTaskCommand {
                tenant_id: 100_001,
                organization_id: 20,
                path_agent_id: organization_two_agent.agent_id.clone(),
                task_id: organization_two_task.task_id.clone(),
                owner_scope: None,
                requested_by: sample_subject(),
            })
            .expect("read organization two task");
        assert_eq!(organization_two_read.prompt, "Organization two work");

        let mut organization_one_only_command = create_task_cmd(
            &service,
            100_001,
            10,
            &organization_one_agent.agent_id,
            100,
            "Organization one private work",
            "2026-06-01T06:03:00Z",
        );
        organization_one_only_command.task_id = "task.organization-one-only".to_string();
        let organization_one_only_task = service
            .create_task(organization_one_only_command)
            .expect("create organization one private task");

        let foreign_read = service
            .get_task(GetTaskCommand {
                tenant_id: 100_001,
                organization_id: 20,
                path_agent_id: organization_one_agent.agent_id.clone(),
                task_id: organization_one_only_task.task_id.clone(),
                owner_scope: None,
                requested_by: sample_subject(),
            })
            .expect_err("another organization must not read the task");
        assert!(foreign_read.to_string().contains("task not found"));

        let foreign_cancel = service
            .cancel_task(CancelTaskCommand {
                tenant_id: 100_001,
                organization_id: 20,
                path_agent_id: organization_one_agent.agent_id.clone(),
                task_id: organization_one_only_task.task_id.clone(),
                expected_version: Some(organization_one_only_task.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T06:04:00Z".to_string(),
            })
            .expect_err("another organization must not cancel the task");
        assert!(foreign_cancel.to_string().contains("task not found"));

        let foreign_execute = service
            .execute_task(ExecuteTaskCommand {
                tenant_id: 100_001,
                organization_id: 20,
                path_agent_id: organization_one_agent.agent_id.clone(),
                task_id: organization_one_only_task.task_id.clone(),
                idempotency_key: "task-run.foreign-org".to_string(),
                expected_version: Some(organization_one_only_task.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T06:05:00Z".to_string(),
            })
            .expect_err("another organization must not execute the task");
        assert!(foreign_execute.to_string().contains("task not found"));

        let unchanged = service
            .get_task(GetTaskCommand {
                tenant_id: 100_001,
                organization_id: 10,
                path_agent_id: organization_one_agent.agent_id,
                task_id: organization_one_only_task.task_id,
                owner_scope: None,
                requested_by: sample_subject(),
            })
            .expect("the owning organization still sees its task");
        assert_eq!(unchanged.status, AgentTaskStatus::Active);
        assert_eq!(unchanged.version, organization_one_only_task.version);
    }

    #[test]
    fn cancel_turn_confirms_model_request_id_and_blocks_late_completion_in_memory() {
        let executor = Arc::new(BlockingCancellationExecutor::default());
        let service = Arc::new(
            AgentsService::new(
                InMemoryAgentRepository::new(),
                InMemoryAgentAuditSink::default(),
                test_policy_provider(),
            )
            .with_turn_executor(executor.clone()),
        );
        let agent = service
            .create_agent(create_agent_cmd(
                "agent.turn.cancel-race",
                100_001,
                0,
                100,
                "turn-cancel-race",
                "Turn Cancel Race",
                "2026-08-02T00:00:00Z",
            ))
            .expect("create cancellation test agent");
        service
            .change_status(ChangeAgentStatusCommand {
                tenant_id: 100_001,
                agent_id: agent.agent_id.clone(),
                expected_version: Some(agent.version),
                target_status: AgentBusinessStatus::Active,
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:01Z".to_string(),
            })
            .expect("activate cancellation test agent");
        let provider_binding = service
            .add_provider_binding(AgentProviderBindingCommand {
                tenant_id: 100_001,
                agent_id: agent.agent_id.clone(),
                binding_id: "binding.turn.cancel-race".to_string(),
                provider_id: "provider.cancel-race".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.turn.cancel-race".to_string(),
                capabilities: vec!["model.chat".to_string()],
                make_default: true,
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:02Z".to_string(),
            })
            .expect("create cancellation test provider binding");
        let session = service
            .create_session(CreateSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: agent.agent_id.clone(),
                owner_user_id: 100,
                project_id: None,
                session_id: "session.test.cancel-race".to_string(),
                session_kind: AgentSessionKind::Coding,
                entry_surface: AgentSessionEntrySurface::Pc,
                source_module: None,
                source_context_kind: None,
                source_context_id: None,
                parent_session_id: None,
                forked_from_turn_id: None,
                title: Some("Cancellation race".to_string()),
                idempotency_key: None,
                payload_hash: None,
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:03Z".to_string(),
            })
            .expect("create cancellation test session");
        let runtime_binding = service
            .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id.clone(),
                session_id: session.session_id.clone(),
                runtime_binding_id: Some("runtime_binding.turn.cancel-race".to_string()),
                runtime_location_id: None,
                host_mode: "managed".to_string(),
                transport_kind: "in_process".to_string(),
                provider_binding_id: provider_binding.binding_id,
                model_id: "model.cancel-race".to_string(),
                provider_id: provider_binding.provider_id,
                provider_session_id: Some("provider-session.cancel-race".to_string()),
                provider_session_tree_id: None,
                provider_parent_session_id: None,
                provider_forked_from_session_id: None,
                provider_directory: None,
                owner_scope: Some(100),
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:04Z".to_string(),
            })
            .expect("create cancellation test runtime binding");
        let turn_id = "turn.cancel-race".to_string();
        let execution_service = Arc::clone(&service);
        let execution_agent_id = agent.agent_id.clone();
        let execution_session_id = session.session_id.clone();
        let execution_turn_id = turn_id.clone();
        let execution = std::thread::spawn(move || {
            execution_service.execute_turn(CreateTurnCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: execution_agent_id,
                session_id: execution_session_id,
                turn_id: Some(execution_turn_id),
                content: "complete after cancellation".to_string(),
                content_type: "text/plain".to_string(),
                turn_mode: AgentTurnMode::Interactive,
                runtime_binding_id: Some(runtime_binding.runtime_binding_id),
                requested_model_id: Some("model.cancel-race".to_string()),
                access_mode_id: None,
                idempotency_key: "idempotency.turn.cancel-race".to_string(),
                payload_hash: "sha256:turn-cancel-race".to_string(),
                client_request_id: Some("request.turn.cancel-race".to_string()),
                drive_refs: Vec::new(),
                owner_scope: Some(100),
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:05Z".to_string(),
                prefer_stream: false,
            system_prompt: None,
            auth_token: None,
            access_token: None,
            })
        });
        executor.wait_until_started();

        let expected_model_request_id = turn_model_request_id(&turn_id);
        let cancelled = service
            .cancel_turn(CancelTurnCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id.clone(),
                session_id: session.session_id.clone(),
                turn_id: turn_id.clone(),
                expected_version: Some(1),
                owner_scope: Some(100),
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:06Z".to_string(),
            })
            .expect("running turn cancellation must be acknowledged");
        assert_eq!(cancelled.status, AgentTurnStatus::Cancelled);
        assert_eq!(cancelled.finish_reason.as_deref(), Some("cancelled"));
        assert_eq!(
            executor.cancelled_model_request_id().as_deref(),
            Some(expected_model_request_id.as_str())
        );

        executor.release();
        let late_completion = execution
            .join()
            .expect("late completion worker should not panic")
            .expect_err("late completion must not overwrite a cancelled turn");
        assert_eq!(late_completion.kind(), KernelErrorKind::Conflict);

        let persisted = service
            .get_turn(GetTurnCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id,
                session_id: session.session_id,
                turn_id,
                owner_scope: Some(100),
                requested_by: sample_subject(),
            })
            .expect("read cancelled turn");
        assert_eq!(persisted.status, AgentTurnStatus::Cancelled);
        assert_eq!(persisted.finish_reason.as_deref(), Some("cancelled"));
        assert!(persisted.response_item_id.is_none());
        assert_eq!((persisted.input_tokens, persisted.output_tokens), (0, 0));
    }


    /// Executor whose failure behavior can be switched mid-test so the same
    /// Turn can first fail and then succeed on a retried attempt.
    #[derive(Default)]
    struct SwitchableFailureExecutor {
        fail: std::sync::atomic::AtomicBool,
    }

    impl TurnExecutor for SwitchableFailureExecutor {
        fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
            if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
                inference_error("provider unavailable on the first attempt")
            } else {
                execute_agent_turn(input)
            }
        }
    }

    #[test]
    fn task_retry_resumes_failed_turn_instead_of_terminating_the_run() {
        let executor = Arc::new(SwitchableFailureExecutor::default());
        executor
            .fail
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let service = Arc::new(
            AgentsService::new(
                InMemoryAgentRepository::new(),
                InMemoryAgentAuditSink::default(),
                test_policy_provider(),
            )
            .with_turn_executor(executor.clone()),
        );
        let agent = service
            .create_agent(create_agent_cmd(
                "agent.turn.retry-resume",
                100_001,
                0,
                100,
                "turn-retry-resume",
                "Turn Retry Resume",
                "2026-08-02T00:00:00Z",
            ))
            .expect("create retry agent");
        service
            .change_status(ChangeAgentStatusCommand {
                tenant_id: 100_001,
                agent_id: agent.agent_id.clone(),
                expected_version: Some(agent.version),
                target_status: AgentBusinessStatus::Active,
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:01Z".to_string(),
            })
            .expect("activate retry agent");
        let provider_binding = service
            .add_provider_binding(AgentProviderBindingCommand {
                tenant_id: 100_001,
                agent_id: agent.agent_id.clone(),
                binding_id: "binding.turn.retry-resume".to_string(),
                provider_id: "provider.retry-resume".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.turn.retry-resume".to_string(),
                capabilities: vec!["model.chat".to_string()],
                make_default: true,
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:02Z".to_string(),
            })
            .expect("create retry provider binding");
        let session = service
            .create_session(CreateSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: agent.agent_id.clone(),
                owner_user_id: 100,
                project_id: None,
                session_id: "session.test.retry-resume".to_string(),
                session_kind: AgentSessionKind::Coding,
                entry_surface: AgentSessionEntrySurface::Pc,
                source_module: None,
                source_context_kind: None,
                source_context_id: None,
                parent_session_id: None,
                forked_from_turn_id: None,
                title: Some("Retry resume".to_string()),
                idempotency_key: None,
                payload_hash: None,
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:03Z".to_string(),
            })
            .expect("create retry session");
        let runtime_binding = service
            .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id.clone(),
                session_id: session.session_id.clone(),
                runtime_binding_id: Some("runtime_binding.turn.retry-resume".to_string()),
                runtime_location_id: None,
                host_mode: "managed".to_string(),
                transport_kind: "in_process".to_string(),
                provider_binding_id: provider_binding.binding_id,
                model_id: "model.retry-resume".to_string(),
                provider_id: provider_binding.provider_id,
                provider_session_id: Some("provider-session.retry-resume".to_string()),
                provider_session_tree_id: None,
                provider_parent_session_id: None,
                provider_forked_from_session_id: None,
                provider_directory: None,
                owner_scope: Some(100),
                requested_by: sample_subject(),
                requested_at: "2026-08-02T00:00:04Z".to_string(),
            })
            .expect("create retry runtime binding");

        let turn_id = "turn.retry-resume".to_string();
        let command = |requested_at: &str| CreateTurnCommand {
            tenant_id: 100_001,
            organization_id: 0,
            agent_id: agent.agent_id.clone(),
            session_id: session.session_id.clone(),
            turn_id: Some(turn_id.clone()),
            content: "retry me".to_string(),
            content_type: "text/plain".to_string(),
            turn_mode: AgentTurnMode::Automation,
            runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
            requested_model_id: Some("model.retry-resume".to_string()),
            access_mode_id: None,
            idempotency_key: "idempotency.turn.retry-resume".to_string(),
            payload_hash: "sha256:turn-retry-resume".to_string(),
            client_request_id: Some("request.turn.retry-resume".to_string()),
            drive_refs: Vec::new(),
            owner_scope: Some(100),
            requested_by: sample_subject(),
            requested_at: requested_at.to_string(),
            prefer_stream: false,
            system_prompt: None,
            auth_token: None,
            access_token: None,
        };

        // First attempt: provider failure marks the Turn Failed.
        let first = service
            .execute_turn(command("2026-08-02T00:00:05Z"))
            .expect_err("first attempt must fail at the provider");
        assert_eq!(
            first.code().to_string(),
            "turn_inference_failed",
            "first attempt fails with the provider error"
        );
        let stored = service
            .get_turn(GetTurnCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: agent.agent_id.clone(),
                session_id: session.session_id.clone(),
                turn_id: turn_id.clone(),
                owner_scope: Some(100),
                requested_by: sample_subject(),
            })
            .expect("get Turn after failure");
        assert_eq!(stored.status, AgentTurnStatus::Failed);
        assert_eq!(stored.turn_id, turn_id);

        // Second attempt (retried Run, same idempotency key): the provider is
        // healthy again; the retry must resume the SAME Turn instead of
        // terminating the Run with a conflict.
        executor
            .fail
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let resumed = service
            .execute_turn(command("2026-08-02T00:00:06Z"))
            .expect("retried attempt must resume the failed Turn");
        assert_eq!(resumed.turn.turn_id, turn_id, "retry reuses the same Turn");
        assert_eq!(resumed.turn.status, AgentTurnStatus::Completed);
        assert!(resumed.turn.response_item_id.is_some());
        assert_eq!(resumed.turn.error_code.as_deref(), None);
    }
}
