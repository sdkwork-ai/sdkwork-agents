//! Usage metering read model (`agents.usage.*`).
//!
//! Commercial metering surface over the durable turn facts
//! (`ai_agent_turn.input_tokens / output_tokens / cached_tokens`). Billing,
//! orders and quotas stay owned by the platform gateway; this module only
//! exposes the aggregation facts that answering "how much did tenant X spend
//! on agent Y" requires. Aggregates are computed from the same rows that back
//! `agents.turns.list`; no separate ledger is maintained here.

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{parse_datetime, sha256_hash};
use serde::{Deserialize, Serialize};

use crate::list_cursors::{
    decode_created_at_cursor, encode_created_at_cursor, CreatedAtListCursor, LIST_CURSOR_VERSION,
};

const USAGE_CURSOR_KIND: &str = "usage";

/// Aggregated usage totals for one tenant scope and filter window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUsageSummary {
    pub turn_count: u64,
    pub session_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
}

/// One turn-level usage fact row. `internal_id` is the storage tiebreak for
/// keyset pagination (`ORDER BY created_at DESC, id DESC`) and is never part
/// of the wire DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUsageRecord {
    pub internal_id: u64,
    pub turn_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub status: String,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Filter scope for `agents.usage.summary`. All filters are conjunctive; an
/// absent filter widens the aggregate. The window bounds are inclusive
/// (`from <= created_at < to`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsageSummaryQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub model_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

impl UsageSummaryQuery {
    pub fn for_tenant(tenant_id: u64, organization_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            ..Self::default()
        }
    }

    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_model(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    /// Sets the aggregation window. Both bounds must be RFC 3339 timestamps.
    pub fn with_window(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self.to = Some(to.into());
        self
    }

    /// Validates the optional window bounds; keeps the canonical string form
    /// so the same value can be hashed into the cursor fingerprint.
    pub fn validated(self) -> KernelResult<Self> {
        for bound in [self.from.as_deref(), self.to.as_deref()]
            .into_iter()
            .flatten()
        {
            if parse_datetime(bound, None).is_none() {
                return Err(KernelError::validation(
                    "from and to must be RFC 3339 timestamps",
                ));
            }
        }
        Ok(self)
    }
}

/// Filter scope for `agents.usage.records` — same filters as the summary plus
/// cursor-window pagination over `ORDER BY created_at DESC, id DESC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecordListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub model_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub pagination: crate::ports::PaginationParams,
    pub cursor: Option<CreatedAtListCursor>,
}

impl UsageRecordListQuery {
    pub fn for_tenant(tenant_id: u64, organization_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            agent_id: None,
            session_id: None,
            model_id: None,
            from: None,
            to: None,
            pagination: crate::ports::PaginationParams::default(),
            cursor: None,
        }
    }

    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_model(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn with_window(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self.to = Some(to.into());
        self
    }

    pub fn with_pagination(mut self, pagination: crate::ports::PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    pub fn with_cursor(mut self, cursor: CreatedAtListCursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn validated(self) -> KernelResult<Self> {
        for bound in [self.from.as_deref(), self.to.as_deref()]
            .into_iter()
            .flatten()
        {
            if parse_datetime(bound, None).is_none() {
                return Err(KernelError::validation(
                    "from and to must be RFC 3339 timestamps",
                ));
            }
        }
        Ok(self)
    }
}

/// Scope fingerprint shared by the summary and records feeds so a records
/// cursor can never be replayed against a different filter window.
pub(crate) fn usage_list_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    agent_id: Option<&str>,
    session_id: Option<&str>,
    model_id: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": LIST_CURSOR_VERSION,
            "kind": USAGE_CURSOR_KIND,
            "tenantId": tenant_id.to_string(),
            "organizationId": organization_id.to_string(),
            "agentId": agent_id,
            "sessionId": session_id,
            "modelId": model_id,
            "from": from,
            "to": to,
            "sort": ["-createdAt", "-id"],
        })
        .to_string()
        .as_bytes(),
    )
}

pub(crate) fn encode_usage_record_cursor(cursor: &CreatedAtListCursor) -> KernelResult<String> {
    encode_created_at_cursor(USAGE_CURSOR_KIND, cursor)
}

pub(crate) fn decode_usage_record_cursor(value: &str) -> KernelResult<CreatedAtListCursor> {
    decode_created_at_cursor(value, USAGE_CURSOR_KIND)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_cursor_is_kind_isolated() {
        let fingerprint = usage_list_scope_fingerprint(
            10,
            20,
            Some("agent.usage"),
            None,
            Some("gpt-5.4"),
            None,
            None,
        );
        let cursor = CreatedAtListCursor {
            created_at: "2026-09-01T00:00:00.000Z".to_string(),
            internal_id: 9,
            scope_fingerprint: fingerprint.clone(),
        };
        let encoded = encode_usage_record_cursor(&cursor).expect("encode usage cursor");
        assert_eq!(decode_usage_record_cursor(&encoded).unwrap(), cursor);
        // A turn-list cursor must not be replayable as a usage cursor.
        assert!(crate::list_cursors::decode_created_at_cursor(&encoded, "turn").is_err());
    }

    #[test]
    fn window_bounds_must_be_rfc3339() {
        let ok = UsageSummaryQuery::for_tenant(1, 2)
            .with_window("2026-09-01T00:00:00Z", "2026-09-02T00:00:00Z");
        assert!(ok.validated().is_ok());
        let bad =
            UsageSummaryQuery::for_tenant(1, 2).with_window("not-a-date", "2026-09-02T00:00:00Z");
        assert!(bad.validated().is_err());
    }

    #[test]
    fn fingerprint_depends_on_every_filter() {
        let base = usage_list_scope_fingerprint(10, 20, None, None, None, None, None);
        assert_ne!(
            base,
            usage_list_scope_fingerprint(10, 20, Some("agent.a"), None, None, None, None)
        );
        assert_ne!(
            base,
            usage_list_scope_fingerprint(
                10,
                20,
                None,
                None,
                None,
                Some("2026-09-01T00:00:00Z"),
                None
            )
        );
    }
}
