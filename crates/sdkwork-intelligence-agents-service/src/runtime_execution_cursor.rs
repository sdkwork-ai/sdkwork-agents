use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{
    encoding::{base64url_decode, base64url_encode},
    parse_datetime, sha256_hash,
};
use serde::{Deserialize, Serialize};

const RUNTIME_EXECUTION_CURSOR_VERSION: u8 = 1;
const MAX_RUNTIME_EXECUTION_CURSOR_BYTES: usize = 2048;

/// Keyset cursor for `agents.calls.list`.
///
/// The compound position `(requested_at, execution_id)` is unique per
/// `(tenant_id, agent_id)` scope, so no internal id is required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeExecutionCursor {
    pub requested_at: String,
    pub execution_id: String,
    pub scope_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeExecutionCursorPayload {
    version: u8,
    kind: String,
    position: String,
    execution_id: String,
    scope_fingerprint: String,
}

pub(crate) fn encode_runtime_execution_cursor(
    cursor: &RuntimeExecutionCursor,
) -> KernelResult<String> {
    encode_cursor(RuntimeExecutionCursorPayload {
        version: RUNTIME_EXECUTION_CURSOR_VERSION,
        kind: "runtime-execution".to_string(),
        position: cursor.requested_at.clone(),
        execution_id: cursor.execution_id.clone(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    })
}

pub(crate) fn decode_runtime_execution_cursor(value: &str) -> KernelResult<RuntimeExecutionCursor> {
    let payload = decode_cursor(value, "runtime-execution")?;
    if parse_datetime(&payload.position, None).is_none() {
        return Err(invalid_cursor());
    }
    if payload.execution_id.is_empty() {
        return Err(invalid_cursor());
    }
    Ok(RuntimeExecutionCursor {
        requested_at: payload.position,
        execution_id: payload.execution_id,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn runtime_execution_scope_fingerprint(
    tenant_id: u64,
    agent_id: &str,
    status: Option<&str>,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": RUNTIME_EXECUTION_CURSOR_VERSION,
            "kind": "runtime-execution",
            "tenantId": tenant_id.to_string(),
            "agentId": agent_id,
            "status": status,
            "sort": ["-requestedAt", "-executionId"],
        })
        .to_string()
        .as_bytes(),
    )
}

fn encode_cursor(payload: RuntimeExecutionCursorPayload) -> KernelResult<String> {
    let json = serde_json::to_vec(&payload).map_err(|error| KernelError::Internal {
        message: format!("failed to encode runtime execution cursor: {error}"),
    })?;
    Ok(base64url_encode(&json))
}

fn decode_cursor(value: &str, expected_kind: &str) -> KernelResult<RuntimeExecutionCursorPayload> {
    if value.is_empty() || value.len() > MAX_RUNTIME_EXECUTION_CURSOR_BYTES || value.trim() != value
    {
        return Err(invalid_cursor());
    }
    let decoded = base64url_decode(value).ok_or_else(invalid_cursor)?;
    let payload: RuntimeExecutionCursorPayload =
        serde_json::from_slice(&decoded).map_err(|_| invalid_cursor())?;
    if payload.version != RUNTIME_EXECUTION_CURSOR_VERSION
        || payload.kind != expected_kind
        || payload.scope_fingerprint.len() != 64
        || !payload
            .scope_fingerprint
            .bytes()
            .all(|value| value.is_ascii_hexdigit())
    {
        return Err(invalid_cursor());
    }
    Ok(payload)
}

fn invalid_cursor() -> KernelError {
    KernelError::validation("cursor is not a valid opaque token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_execution_cursor_is_opaque_scope_bound_and_kind_safe() {
        let cursor = RuntimeExecutionCursor {
            requested_at: "2026-09-06T12:00:00.000Z".to_string(),
            execution_id: "execution.structuredcall.1".to_string(),
            scope_fingerprint: runtime_execution_scope_fingerprint(
                10,
                "agent.cursor",
                Some("queued"),
            ),
        };
        let encoded = encode_runtime_execution_cursor(&cursor).expect("encode cursor");
        assert!(!encoded.contains("agent.cursor"));
        assert_eq!(decode_runtime_execution_cursor(&encoded).unwrap(), cursor);
        assert!(decode_runtime_execution_cursor("execution.1").is_err());
    }
}
