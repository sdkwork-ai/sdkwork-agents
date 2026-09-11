//! Opaque keyset cursors for the high-volume Agents list feeds.
//!
//! Sessions, Turns, audit events and Interactions are fast-growing, unstable
//! lists; PAGINATION_SPEC §3 requires cursor mode for such feeds instead of
//! offset mode. Every cursor carries the scope fingerprint of the query that
//! produced it so a token can never be replayed against a different filter
//! scope (DATABASE_SPEC §20.5).

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{
    encoding::{base64url_decode, base64url_encode},
    parse_datetime, sha256_hash,
};
use serde::{Deserialize, Serialize};

pub(crate) const LIST_CURSOR_VERSION: u8 = 1;
const MAX_LIST_CURSOR_BYTES: usize = 2048;

/// Keyset position for `ORDER BY updated_at DESC, id DESC` lists (sessions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListCursor {
    pub updated_at: String,
    pub session_internal_id: u64,
    pub scope_fingerprint: String,
}

/// Keyset position for `ORDER BY created_at DESC, id DESC` lists (turns,
/// interactions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedAtListCursor {
    pub created_at: String,
    pub internal_id: u64,
    pub scope_fingerprint: String,
}

/// Keyset position for the Audit event feed
/// (`ORDER BY created_at DESC, event_ref DESC`). The tiebreak is a stable
/// unique string: the storage `uuid` column in PostgreSQL and the `event_id`
/// in the in-memory sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventListCursor {
    pub created_at: String,
    pub event_ref: String,
    pub scope_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListCursorPayload {
    version: u8,
    kind: String,
    position: String,
    internal_id: String,
    scope_fingerprint: String,
}

pub(crate) fn encode_session_list_cursor(cursor: &SessionListCursor) -> KernelResult<String> {
    encode_cursor(ListCursorPayload {
        version: LIST_CURSOR_VERSION,
        kind: "session".to_string(),
        position: cursor.updated_at.clone(),
        internal_id: cursor.session_internal_id.to_string(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    })
}

pub(crate) fn decode_session_list_cursor(value: &str) -> KernelResult<SessionListCursor> {
    let payload = decode_cursor(value, "session")?;
    if parse_datetime(&payload.position, None).is_none() {
        return Err(invalid_cursor());
    }
    Ok(SessionListCursor {
        updated_at: payload.position,
        session_internal_id: parse_positive_u64(&payload.internal_id)?,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn encode_created_at_cursor(
    kind: &str,
    cursor: &CreatedAtListCursor,
) -> KernelResult<String> {
    encode_cursor(ListCursorPayload {
        version: LIST_CURSOR_VERSION,
        kind: kind.to_string(),
        position: cursor.created_at.clone(),
        internal_id: cursor.internal_id.to_string(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    })
}

pub(crate) fn decode_created_at_cursor(
    value: &str,
    kind: &str,
) -> KernelResult<CreatedAtListCursor> {
    let payload = decode_cursor(value, kind)?;
    if parse_datetime(&payload.position, None).is_none() {
        return Err(invalid_cursor());
    }
    Ok(CreatedAtListCursor {
        created_at: payload.position,
        internal_id: parse_positive_u64(&payload.internal_id)?,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn encode_audit_event_list_cursor(
    cursor: &AuditEventListCursor,
) -> KernelResult<String> {
    encode_cursor(ListCursorPayload {
        version: LIST_CURSOR_VERSION,
        kind: "audit".to_string(),
        position: cursor.created_at.clone(),
        internal_id: cursor.event_ref.clone(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    })
}

pub(crate) fn decode_audit_event_list_cursor(value: &str) -> KernelResult<AuditEventListCursor> {
    let payload = decode_cursor(value, "audit")?;
    if parse_datetime(&payload.position, None).is_none()
        || payload.internal_id.is_empty()
        || payload.internal_id.len() > 128
    {
        return Err(invalid_cursor());
    }
    Ok(AuditEventListCursor {
        created_at: payload.position,
        event_ref: payload.internal_id,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

/// Scope fingerprint for the Session list feed
/// (`ORDER BY updated_at DESC, id DESC`).
pub(crate) fn session_list_scope_fingerprint(
    tenant_id: u64,
    organization_id: Option<u64>,
    agent_id: Option<&str>,
    project_id: Option<&str>,
    workspace_id: Option<&str>,
    owner_user_id: Option<u64>,
    status: Option<&str>,
    include_archived: bool,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": LIST_CURSOR_VERSION,
            "kind": "session",
            "tenantId": tenant_id.to_string(),
            "organizationId": organization_id.map(|value| value.to_string()),
            "agentId": agent_id,
            "projectId": project_id,
            "workspaceId": workspace_id,
            "ownerUserId": owner_user_id.map(|value| value.to_string()),
            "status": status,
            "includeArchived": include_archived,
            "sort": ["-updatedAt", "-id"],
        })
        .to_string()
        .as_bytes(),
    )
}

/// Scope fingerprint for the Turn list feed
/// (`ORDER BY created_at DESC, id DESC`).
pub(crate) fn turn_list_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    session_id: &str,
    status: Option<&str>,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": LIST_CURSOR_VERSION,
            "kind": "turn",
            "tenantId": tenant_id.to_string(),
            "organizationId": organization_id.to_string(),
            "sessionId": session_id,
            "status": status,
            "sort": ["-createdAt", "-id"],
        })
        .to_string()
        .as_bytes(),
    )
}

/// Scope fingerprint for the Audit event feed
/// (`ORDER BY created_at DESC, id DESC`).
pub(crate) fn audit_list_scope_fingerprint(
    tenant_id: u64,
    agent_id: &str,
    action: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": LIST_CURSOR_VERSION,
            "kind": "audit",
            "tenantId": tenant_id.to_string(),
            "agentId": agent_id,
            "action": action,
            "from": from,
            "to": to,
            "sort": ["-createdAt", "-id"],
        })
        .to_string()
        .as_bytes(),
    )
}

/// Scope fingerprint for the Interaction list feed
/// (`ORDER BY created_at DESC, id DESC`).
pub(crate) fn interaction_list_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    session_id: &str,
    kind: Option<&str>,
    status: Option<&str>,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": LIST_CURSOR_VERSION,
            "kind": "interaction",
            "tenantId": tenant_id.to_string(),
            "organizationId": organization_id.to_string(),
            "sessionId": session_id,
            "kind": kind,
            "status": status,
            "sort": ["-createdAt", "-id"],
        })
        .to_string()
        .as_bytes(),
    )
}

fn encode_cursor(payload: ListCursorPayload) -> KernelResult<String> {
    let json = serde_json::to_vec(&payload).map_err(|error| KernelError::Internal {
        message: format!("failed to encode list cursor: {error}"),
    })?;
    Ok(base64url_encode(&json))
}

fn decode_cursor(value: &str, expected_kind: &str) -> KernelResult<ListCursorPayload> {
    if value.is_empty() || value.len() > MAX_LIST_CURSOR_BYTES || value.trim() != value {
        return Err(invalid_cursor());
    }
    let decoded = base64url_decode(value).ok_or_else(invalid_cursor)?;
    let payload: ListCursorPayload =
        serde_json::from_slice(&decoded).map_err(|_| invalid_cursor())?;
    if payload.version != LIST_CURSOR_VERSION
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

fn parse_positive_u64(value: &str) -> KernelResult<u64> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(invalid_cursor)
}

fn invalid_cursor() -> KernelError {
    KernelError::validation("cursor is not a valid opaque token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_cursor_is_opaque_and_scope_bound() {
        let cursor = SessionListCursor {
            updated_at: "2026-08-01T10:00:00.000Z".to_string(),
            session_internal_id: 77,
            scope_fingerprint: session_list_scope_fingerprint(
                10,
                Some(20),
                Some("agent.list-cursor"),
                None,
                None,
                Some(30),
                Some("active"),
                false,
            ),
        };
        let encoded = encode_session_list_cursor(&cursor).expect("encode cursor");
        assert!(!encoded.contains("agent.list-cursor"));
        assert_eq!(decode_session_list_cursor(&encoded).unwrap(), cursor);
        assert!(decode_session_list_cursor("77").is_err());
        assert!(decode_created_at_cursor(&encoded, "turn").is_err());
    }

    #[test]
    fn created_at_cursors_reject_other_kinds() {
        let fingerprint = turn_list_scope_fingerprint(10, 20, "session.cursor", None);
        let turn = CreatedAtListCursor {
            created_at: "2026-08-01T11:00:00.000Z".to_string(),
            internal_id: 5,
            scope_fingerprint: fingerprint.clone(),
        };
        let encoded = encode_created_at_cursor("turn", &turn).expect("encode turn cursor");
        assert_eq!(decode_created_at_cursor(&encoded, "turn").unwrap(), turn);
        assert!(decode_created_at_cursor(&encoded, "audit").is_err());

        let audit = AuditEventListCursor {
            created_at: "2026-08-01T12:00:00.000Z".to_string(),
            event_ref: "uuid.audit-event.0001".to_string(),
            scope_fingerprint: audit_list_scope_fingerprint(10, "agent.cursor", None, None, None),
        };
        let encoded = encode_audit_event_list_cursor(&audit).expect("encode audit cursor");
        assert_eq!(decode_audit_event_list_cursor(&encoded).unwrap(), audit);
        assert!(decode_created_at_cursor(&encoded, "interaction").is_err());

        let interaction = CreatedAtListCursor {
            created_at: "2026-08-01T13:00:00.000Z".to_string(),
            internal_id: 11,
            scope_fingerprint: interaction_list_scope_fingerprint(
                10,
                20,
                "session.cursor",
                None,
                None,
            ),
        };
        let encoded = encode_created_at_cursor("interaction", &interaction)
            .expect("encode interaction cursor");
        assert_eq!(
            decode_created_at_cursor(&encoded, "interaction").unwrap(),
            interaction
        );
    }
}
