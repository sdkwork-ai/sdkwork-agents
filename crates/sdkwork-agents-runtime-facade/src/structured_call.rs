//! Structured agent calls: prompt or typed parameters in, validated
//! structured output out.
//!
//! The pipeline owns format mechanics only (input validation, prompt
//! composition, parsing, schema validation, one repair retry). Model
//! execution is injected through [`StructuredTurnExecutor`] so engines,
//! bindings, and product policy stay outside this module (open/closed).
//!
//! Contract authority: `specs/agent-structured-call.contract.json`
//! (`kind: sdkwork.agents.structured-call`).

use serde_json::Value;

use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

/// Default wall-clock budget for one structured call (`policy.timeoutMs`).
pub const DEFAULT_STRUCTURED_CALL_TIMEOUT_MS: u64 = 60_000;
/// Hard upper bound for one structured call.
pub const MAX_STRUCTURED_CALL_TIMEOUT_MS: u64 = 300_000;
/// Exactly one repair retry is permitted after a failed validation.
pub const MAX_STRUCTURED_CALL_REPAIR_ATTEMPTS: usize = 1;
/// Composed prompt budget; mirrors the 1 MiB engine turn prompt cap.
pub const MAX_STRUCTURED_CALL_PROMPT_BYTES: usize = 1024 * 1024;

/// Stable tool id for the agent-as-tool projection (contract §4).
pub const AGENT_CALL_TOOL_ID: &str = "agent_call";
/// Hard nesting limit: a call executed inside a turn loop MUST NOT expose
/// `agent_call` to its own tool loop; recursion is rejected fail-closed.
pub const AGENT_CALL_NESTING_DEPTH_MAXIMUM: usize = 1;

/// JSON Schema (draft 2020-12) for the agent-as-tool invocation payload.
///
/// Host products project this descriptor into their kernel tool-provider
/// surface; the payload maps 1:1 onto [`AgentStructuredCallInput`].
pub fn agent_call_tool_input_schema() -> Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["mode"],
        "additionalProperties": false,
        "properties": {
            "mode": { "type": "string", "enum": ["prompt", "params"] },
            "prompt": { "type": "string", "minLength": 1 },
            "params": { "type": "object" },
            "paramSchema": { "type": "object" },
            "output": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "format": { "type": "string", "enum": ["json", "xml", "text"] },
                    "schema": { "type": "object" },
                    "rootElement": { "type": "string", "minLength": 1 },
                    "strict": { "type": "boolean" }
                }
            },
            "policy": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "timeoutMs": { "type": "integer", "minimum": 1, "maximum": 300000 }
                }
            }
        }
    })
}

/// JSON Schema (draft 2020-12) for the agent-as-tool result payload.
pub fn agent_call_tool_output_schema() -> Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["status", "output", "validation", "attempts"],
        "additionalProperties": false,
        "properties": {
            "status": {
                "type": "string",
                "enum": ["succeeded", "validation_failed", "agent_failed", "timeout"]
            },
            "output": {},
            "rawText": { "type": "string" },
            "validation": {
                "type": "object",
                "required": ["valid", "errors"],
                "properties": {
                    "valid": { "type": "boolean" },
                    "errors": { "type": "array", "items": { "type": "string" } }
                }
            },
            "attempts": { "type": "integer", "minimum": 0 }
        }
    })
}

/// Supported structured-call invocation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCallMode {
    /// Free-text prompt input.
    Prompt,
    /// Typed parameters validated against `paramSchema` before execution.
    Params,
}

impl AgentCallMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::Params => "params",
        }
    }

    pub fn parse(value: &str) -> RuntimeFacadeResult<Self> {
        match value {
            "prompt" => Ok(Self::Prompt),
            "params" => Ok(Self::Params),
            other => Err(RuntimeFacadeError::InvalidInput(format!(
                "mode must be \"prompt\" or \"params\", got \"{other}\""
            ))),
        }
    }
}

/// Supported structured output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCallOutputFormat {
    Json,
    Xml,
    Text,
}

impl AgentCallOutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Text => "text",
        }
    }

    pub fn parse(value: &str) -> RuntimeFacadeResult<Self> {
        match value {
            "json" => Ok(Self::Json),
            "xml" => Ok(Self::Xml),
            "text" => Ok(Self::Text),
            other => Err(RuntimeFacadeError::InvalidInput(format!(
                "output format must be \"json\", \"xml\", or \"text\", got \"{other}\""
            ))),
        }
    }
}

/// Terminal status of a structured call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCallStatus {
    Succeeded,
    ValidationFailed,
    AgentFailed,
    Timeout,
}

impl AgentCallStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::ValidationFailed => "validation_failed",
            Self::AgentFailed => "agent_failed",
            Self::Timeout => "timeout",
        }
    }
}

/// Input contract for one structured call.
///
/// Wire serialization is owned by the service layer; this facade type stays
/// plain so the pipeline is reusable by any consumer (HTTP, agent-as-tool,
/// tests).
#[derive(Debug, Clone, PartialEq)]
pub struct AgentStructuredCallInput {
    pub mode: AgentCallMode,
    /// Free-text prompt; required and non-blank when `mode` is [`AgentCallMode::Prompt`].
    pub prompt: String,
    /// Typed parameters; required JSON object when `mode` is [`AgentCallMode::Params`].
    pub params: Value,
    /// JSON Schema (draft 2020-12) for `params`; required in params mode.
    pub param_schema: Option<Value>,
    pub output_format: AgentCallOutputFormat,
    /// JSON Schema (draft 2020-12) the JSON output must satisfy; JSON only.
    pub output_schema: Option<Value>,
    /// Required XML root element name; XML only.
    pub output_root_element: Option<String>,
    /// Strict mode fails the call when validation cannot be satisfied.
    pub strict: bool,
    pub timeout_ms: u64,
}

impl Default for AgentStructuredCallInput {
    fn default() -> Self {
        Self {
            mode: AgentCallMode::Prompt,
            prompt: String::new(),
            params: Value::Null,
            param_schema: None,
            output_format: AgentCallOutputFormat::Json,
            output_schema: None,
            output_root_element: None,
            strict: true,
            timeout_ms: DEFAULT_STRUCTURED_CALL_TIMEOUT_MS,
        }
    }
}

/// Output contract for one structured call.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentStructuredCallOutput {
    pub status: AgentCallStatus,
    /// Parsed structured value for JSON; string value for XML/text; `Null`
    /// on strict validation failure.
    pub output: Value,
    pub raw_text: Option<String>,
    pub validation_errors: Vec<String>,
    pub agent_error: Option<String>,
    pub attempts: usize,
}

/// Model execution seam for the structured-call pipeline.
///
/// Implementations translate one composed prompt into one raw model answer.
/// Keeping this a trait (instead of calling the engine directly) is what lets
/// the pipeline stay engine-agnostic and unit-testable.
pub trait StructuredTurnExecutor {
    fn execute_turn(&self, prompt: &str) -> RuntimeFacadeResult<String>;
}

impl<F> StructuredTurnExecutor for F
where
    F: Fn(&str) -> RuntimeFacadeResult<String>,
{
    fn execute_turn(&self, prompt: &str) -> RuntimeFacadeResult<String> {
        self(prompt)
    }
}

/// Validates the call input before any model invocation.
///
/// Schema payloads are compiled eagerly so malformed schemas fail the call
/// without consuming model quota.
pub fn validate_structured_call_input(input: &AgentStructuredCallInput) -> RuntimeFacadeResult<()> {
    match input.mode {
        AgentCallMode::Prompt => {
            if input.prompt.trim().is_empty() {
                return Err(RuntimeFacadeError::BlankPrompt);
            }
        }
        AgentCallMode::Params => {
            if !input.params.is_object() {
                return Err(RuntimeFacadeError::InvalidInput(
                    "params must be a JSON object in params mode".to_string(),
                ));
            }
            let schema = input.param_schema.as_ref().ok_or_else(|| {
                RuntimeFacadeError::InvalidInput(
                    "paramSchema is required in params mode".to_string(),
                )
            })?;
            compile_validator(schema)
                .map_err(|error| invalid_schema("paramSchema", error))?;
            validate_against_schema(schema, &input.params)
                .map_err(|error| RuntimeFacadeError::InvalidInput(error))?;
        }
    }

    if input.output_format == AgentCallOutputFormat::Json && input.output_schema.is_some() {
        compile_validator(input.output_schema.as_ref().expect("checked above"))
            .map_err(|error| invalid_schema("output.schema", error))?;
    }
    if input.output_format != AgentCallOutputFormat::Json && input.output_schema.is_some() {
        return Err(RuntimeFacadeError::InvalidInput(
            "output.schema is only valid with the json output format".to_string(),
        ));
    }
    if input.output_format != AgentCallOutputFormat::Xml && input.output_root_element.is_some() {
        return Err(RuntimeFacadeError::InvalidInput(
            "output.rootElement is only valid with the xml output format".to_string(),
        ));
    }
    if let Some(root) = input.output_root_element.as_deref() {
        if root.trim().is_empty() || !root.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            return Err(RuntimeFacadeError::InvalidInput(
                "output.rootElement must be a non-empty XML element name".to_string(),
            ));
        }
    }

    if input.timeout_ms == 0 || input.timeout_ms > MAX_STRUCTURED_CALL_TIMEOUT_MS {
        return Err(RuntimeFacadeError::InvalidInput(format!(
            "policy.timeoutMs must be between 1 and {MAX_STRUCTURED_CALL_TIMEOUT_MS}"
        )));
    }
    Ok(())
}

/// Executes one structured call through the injected executor.
///
/// Pipeline: input validation → prompt composition → turn execution →
/// parse + validate → at most one repair retry.
pub fn execute_agent_structured_call(
    executor: &dyn StructuredTurnExecutor,
    input: &AgentStructuredCallInput,
) -> RuntimeFacadeResult<AgentStructuredCallOutput> {
    validate_structured_call_input(input)?;

    let mut prompt = compose_structured_call_prompt(input);
    if prompt.len() > MAX_STRUCTURED_CALL_PROMPT_BYTES {
        return Err(RuntimeFacadeError::InvalidInput(format!(
            "composed structured-call prompt exceeds {} bytes",
            MAX_STRUCTURED_CALL_PROMPT_BYTES
        )));
    }

    let mut attempts = 0usize;
    let mut last_raw: Option<String> = None;
    let mut validation_errors: Vec<String> = Vec::new();
    let mut agent_error: Option<String> = None;

    loop {
        attempts += 1;
        let raw = match executor.execute_turn(prompt.as_str()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(AgentStructuredCallOutput {
                    status: AgentCallStatus::AgentFailed,
                    output: Value::Null,
                    raw_text: last_raw,
                    validation_errors,
                    agent_error: Some(error.to_string()),
                    attempts,
                });
            }
        };
        last_raw = Some(raw.clone());
        let (output, errors) = parse_structured_output(raw.as_str(), input);
        if errors.is_empty() {
            return Ok(AgentStructuredCallOutput {
                status: AgentCallStatus::Succeeded,
                output,
                raw_text: None,
                validation_errors,
                agent_error,
                attempts,
            });
        }
        validation_errors = errors;
        if attempts > MAX_STRUCTURED_CALL_REPAIR_ATTEMPTS {
            break;
        }
        prompt = compose_repair_prompt(input, raw.as_str(), &validation_errors);
    }

    let raw_text = last_raw.clone();
    let output = if input.strict {
        Value::Null
    } else {
        last_raw.map(Value::String).unwrap_or(Value::Null)
    };
    Ok(AgentStructuredCallOutput {
        status: AgentCallStatus::ValidationFailed,
        output,
        raw_text,
        validation_errors,
        agent_error,
        attempts,
    })
}

/// Composes the execution prompt: input payload plus output directives.
pub fn compose_structured_call_prompt(input: &AgentStructuredCallInput) -> String {
    let mut sections: Vec<String> = Vec::new();
    match input.mode {
        AgentCallMode::Prompt => sections.push(input.prompt.clone()),
        AgentCallMode::Params => {
            let params_json = serde_json::to_string_pretty(&input.params)
                .unwrap_or_else(|_| input.params.to_string());
            sections.push(format!(
                "Process the following input parameters:\n\n{params_json}"
            ));
        }
    }

    match input.output_format {
        AgentCallOutputFormat::Json => {
            let mut directive = "Respond with valid JSON only. No markdown fences, \
                no commentary, no text before or after the JSON value."
                .to_string();
            if let Some(schema) = input.output_schema.as_ref() {
                let schema_json = serde_json::to_string(schema)
                    .unwrap_or_else(|_| schema.to_string());
                directive.push_str(&format!(
                    "\nThe JSON value MUST conform to this JSON Schema (draft 2020-12):\n{schema_json}"
                ));
            }
            sections.push(directive);
        }
        AgentCallOutputFormat::Xml => {
            let mut directive = "Respond with well-formed XML markup only. \
                No markdown fences, no commentary."
                .to_string();
            if let Some(root) = input.output_root_element.as_deref() {
                directive.push_str(&format!(
                    "\nThe outermost XML element MUST be named <{root}>."
                ));
            }
            sections.push(directive);
        }
        AgentCallOutputFormat::Text => {
            sections.push(
                "Respond with plain text only. No markdown fences, no commentary."
                    .to_string(),
            );
        }
    }
    sections.join("\n\n")
}

fn compose_repair_prompt(
    input: &AgentStructuredCallInput,
    raw: &str,
    errors: &[String],
) -> String {
    let errors_list = errors
        .iter()
        .map(|error| format!("- {error}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{}\n\nYour previous response violated the output contract.\nPrevious response:\n{raw}\n\nViolations:\n{errors_list}\n\nRespond again with a corrected answer that fully satisfies the output contract.",
        compose_structured_call_prompt(input)
    )
}

/// Parses raw model text into the requested output format.
///
/// Returns the structured value (or best-effort fallback) plus the list of
/// validation violations; an empty list means the output is accepted.
pub fn parse_structured_output(
    raw: &str,
    input: &AgentStructuredCallInput,
) -> (Value, Vec<String>) {
    match input.output_format {
        AgentCallOutputFormat::Json => parse_json_output(raw, input),
        AgentCallOutputFormat::Xml => parse_xml_output(raw, input),
        AgentCallOutputFormat::Text => (Value::String(raw.to_string()), Vec::new()),
    }
}

fn parse_json_output(raw: &str, input: &AgentStructuredCallInput) -> (Value, Vec<String>) {
    let Some(value) = extract_json_value(raw) else {
        return (Value::Null, vec!["response does not contain a JSON value".to_string()]);
    };
    let Some(schema) = input.output_schema.as_ref() else {
        return (value, Vec::new());
    };
    let errors = schema_errors(schema, &value);
    if errors.is_empty() {
        (value, Vec::new())
    } else {
        (value, errors)
    }
}

fn parse_xml_output(raw: &str, input: &AgentStructuredCallInput) -> (Value, Vec<String>) {
    let mut errors: Vec<String> = Vec::new();
    match roxmltree::Document::parse(raw) {
        Ok(document) => {
            if let Some(expected) = input.output_root_element.as_deref() {
                let actual = document.root_element().tag_name().name();
                if actual != expected {
                    errors.push(format!(
                        "xml root element must be <{expected}>, got <{actual}>"
                    ));
                }
            }
        }
        Err(error) => {
            errors.push(format!("xml is not well-formed: {error}"));
        }
    }
    (Value::String(raw.trim().to_string()), errors)
}

/// Extracts the first JSON value from raw model text.
///
/// Handles fenced code blocks and prose-wrapped payloads by scanning for the
/// outermost braces/brackets before falling back to a bare scalar parse.
fn extract_json_value(raw: &str) -> Option<Value> {
    let trimmed = strip_code_fence(raw).trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Some(value);
    }
    let opener = trimmed.find(['{', '['])?;
    let close_char = match trimmed.as_bytes()[opener] {
        b'{' => '}',
        _ => ']',
    };
    let Some(closer) = trimmed.rfind(close_char) else {
        return None;
    };
    if closer < opener {
        return None;
    }
    let candidate = &trimmed[opener..=closer];
    serde_json::from_str::<Value>(candidate).ok()
}

fn strip_code_fence(raw: &str) -> &str {
    let trimmed = raw.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return trimmed;
    };
    let rest = rest.trim_start_matches(|c: char| c.is_ascii_alphanumeric() || c == '-');
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    match rest.strip_suffix("```") {
        Some(inner) => inner.trim(),
        None => rest.trim(),
    }
}

fn compile_validator(schema: &Value) -> Result<jsonschema::Validator, String> {
    jsonschema::validator_for(schema).map_err(|error| error.to_string())
}

fn validate_against_schema(schema: &Value, instance: &Value) -> Result<(), String> {
    let validator = compile_validator(schema)?;
    let errors: Vec<String> = validator.iter_errors(instance).map(|e| e.to_string()).collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn schema_errors(schema: &Value, instance: &Value) -> Vec<String> {
    match compile_validator(schema) {
        Ok(validator) => validator
            .iter_errors(instance)
            .map(|error| error.to_string())
            .collect(),
        Err(error) => vec![format!("output schema is invalid: {error}")],
    }
}

fn invalid_schema(field: &str, error: String) -> RuntimeFacadeError {
    RuntimeFacadeError::InvalidInput(format!("{field} is not a valid JSON Schema: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn json_call(schema: Option<Value>, strict: bool) -> AgentStructuredCallInput {
        AgentStructuredCallInput {
            prompt: "return test data".to_string(),
            output_format: AgentCallOutputFormat::Json,
            output_schema: schema,
            strict,
            ..Default::default()
        }
    }

    #[test]
    fn json_output_without_schema_succeeds() {
        let executor = |_prompt: &str| Ok("{\"result\": 1}".to_string());
        let output = execute_agent_structured_call(&executor, &json_call(None, true))
            .expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::Succeeded);
        assert_eq!(output.output, json!({"result": 1}));
        assert_eq!(output.attempts, 1);
    }

    #[test]
    fn fenced_and_wrapped_json_is_extracted() {
        for raw in [
            "```json\n{\"ok\": true}\n```",
            "Here is the answer:\n{\"ok\": true}\nDone.",
        ] {
            let executor = move |_prompt: &str| Ok(raw.to_string());
            let output = execute_agent_structured_call(&executor, &json_call(None, true))
                .expect("call must execute");
            assert_eq!(output.status, AgentCallStatus::Succeeded);
            assert_eq!(output.output, json!({"ok": true}));
        }
    }

    #[test]
    fn schema_conforming_output_passes() {
        let schema = json!({
            "type": "object",
            "required": ["city"],
            "properties": {"city": {"type": "string"}}
        });
        let executor = |_prompt: &str| Ok("{\"city\": \"Shenzhen\"}".to_string());
        let output = execute_agent_structured_call(&executor, &json_call(Some(schema), true))
            .expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::Succeeded);
        assert_eq!(output.output, json!({"city": "Shenzhen"}));
    }

    #[test]
    fn repair_retry_recovers_one_validation_failure() {
        let schema = json!({
            "type": "object",
            "required": ["city"],
            "properties": {"city": {"type": "string"}}
        });
        let answers = std::cell::RefCell::new(std::collections::VecDeque::from(vec![
            "{\"wrong\": 1}".to_string(),
            "{\"city\": \"Xi'an\"}".to_string(),
        ]));
        let executor = move |_prompt: &str| {
            Ok(answers
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| "{\"wrong\": 1}".to_string()))
        };
        let output = execute_agent_structured_call(&executor, &json_call(Some(schema), true))
            .expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::Succeeded);
        assert_eq!(output.attempts, 2);
        assert_eq!(output.output, json!({"city": "Xi'an"}));
    }

    #[test]
    fn strict_mode_fails_after_exhausted_repair() {
        let schema = json!({"type": "object", "required": ["city"]});
        let executor = |_prompt: &str| Ok("{\"wrong\": 1}".to_string());
        let output = execute_agent_structured_call(&executor, &json_call(Some(schema), true))
            .expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::ValidationFailed);
        assert_eq!(output.output, Value::Null);
        assert_eq!(output.attempts, MAX_STRUCTURED_CALL_REPAIR_ATTEMPTS + 1);
        assert!(output.raw_text.is_some());
        assert!(!output.validation_errors.is_empty());
    }

    #[test]
    fn non_strict_mode_returns_raw_text_with_violations() {
        let schema = json!({"type": "object", "required": ["city"]});
        let executor = |_prompt: &str| Ok("{\"wrong\": 1}".to_string());
        let output = execute_agent_structured_call(&executor, &json_call(Some(schema), false))
            .expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::ValidationFailed);
        assert_eq!(output.output, Value::String("{\"wrong\": 1}".to_string()));
        assert!(!output.validation_errors.is_empty());
    }

    #[test]
    fn params_are_validated_before_any_model_invocation() {
        let calls = std::cell::Cell::new(0usize);
        let executor = |_prompt: &str| {
            calls.set(calls.get() + 1);
            Ok("{}".to_string())
        };
        let input = AgentStructuredCallInput {
            mode: AgentCallMode::Params,
            params: json!({"city": 42}),
            param_schema: Some(json!({
                "type": "object",
                "required": ["city"],
                "properties": {"city": {"type": "string"}}
            })),
            ..Default::default()
        };
        let error = execute_agent_structured_call(&executor, &input)
            .expect_err("invalid params must fail before execution");
        assert!(matches!(error, RuntimeFacadeError::InvalidInput(_)));
        assert_eq!(calls.get(), 0, "model must not be invoked for invalid params");
    }

    #[test]
    fn executor_failure_maps_to_agent_failed() {
        let executor = |_prompt: &str| Err(RuntimeFacadeError::Kernel("boom".to_string()));
        let output = execute_agent_structured_call(&executor, &json_call(None, true))
            .expect("agent failure is a typed status, not a pipeline error");
        assert_eq!(output.status, AgentCallStatus::AgentFailed);
        assert_eq!(output.agent_error.as_deref(), Some("kernel error: boom"));
    }

    #[test]
    fn xml_output_checks_wellformedness_and_root_element() {
        let input = AgentStructuredCallInput {
            prompt: "return test data".to_string(),
            output_format: AgentCallOutputFormat::Xml,
            output_root_element: Some("report".to_string()),
            ..Default::default()
        };
        let executor = |_prompt: &str| Ok("<report><ok/></report>".to_string());
        let output = execute_agent_structured_call(&executor, &input)
            .expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::Succeeded);
        assert_eq!(output.output, Value::String("<report><ok/></report>".to_string()));

        let bad_root_executor = |_prompt: &str| Ok("<other/>".to_string());
        let output =
            execute_agent_structured_call(&bad_root_executor, &input).expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::ValidationFailed);
        assert!(output.validation_errors[0].contains("root element"));

        let malformed_executor = |_prompt: &str| Ok("<broken>".to_string());
        let output = execute_agent_structured_call(&malformed_executor, &input)
            .expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::ValidationFailed);
        assert!(output.validation_errors[0].contains("not well-formed"));
    }

    #[test]
    fn input_validation_rejects_malformed_calls() {
        assert!(validate_structured_call_input(&AgentStructuredCallInput::default()).is_err());

        let blank_prompt = AgentStructuredCallInput::default();
        assert!(matches!(
            validate_structured_call_input(&blank_prompt),
            Err(RuntimeFacadeError::BlankPrompt)
        ));

        let bad_timeout = AgentStructuredCallInput {
            prompt: "hi".to_string(),
            timeout_ms: MAX_STRUCTURED_CALL_TIMEOUT_MS + 1,
            ..Default::default()
        };
        assert!(validate_structured_call_input(&bad_timeout).is_err());

        let schema_on_text = AgentStructuredCallInput {
            prompt: "hi".to_string(),
            output_format: AgentCallOutputFormat::Text,
            output_schema: Some(json!({"type": "object"})),
            ..Default::default()
        };
        assert!(validate_structured_call_input(&schema_on_text).is_err());

        let malformed_schema = AgentStructuredCallInput {
            prompt: "hi".to_string(),
            output_schema: Some(json!({"type": "nonexistent-type"})),
            ..Default::default()
        };
        assert!(validate_structured_call_input(&malformed_schema).is_err());
    }

    #[test]
    fn text_output_is_verbatim() {
        let input = AgentStructuredCallInput {
            output_format: AgentCallOutputFormat::Text,
            prompt: "say hi".to_string(),
            ..Default::default()
        };
        let executor = |_prompt: &str| Ok("hi there".to_string());
        let output = execute_agent_structured_call(&executor, &input).expect("call must execute");
        assert_eq!(output.status, AgentCallStatus::Succeeded);
        assert_eq!(output.output, Value::String("hi there".to_string()));
    }

    #[test]
    fn mode_and_format_parsers_fail_closed() {
        assert!(AgentCallMode::parse("streaming").is_err());
        assert!(AgentCallOutputFormat::parse("yaml").is_err());
        assert_eq!(AgentCallMode::parse("params").expect("valid"), AgentCallMode::Params);
        assert_eq!(
            AgentCallOutputFormat::parse("xml").expect("valid"),
            AgentCallOutputFormat::Xml
        );
    }
}
