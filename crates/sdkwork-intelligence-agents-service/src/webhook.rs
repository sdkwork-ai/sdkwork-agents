//! Webhook subscriptions and signed delivery facts (`agents.webhooks.*`).
//!
//! Commercial integrations (approval flows, result callbacks) subscribe to
//! Agents lifecycle events instead of polling. This module owns the
//! subscription records, the event-type vocabulary, and the Stripe-style
//! HMAC signature scheme used for delivery authentication. Actual HTTP
//! delivery executes at the HTTP edge; the durable delivery record keeps the
//! observable attempt facts (status, response code, error).

use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{hmac_sha256, secure_compare};
use serde::{Deserialize, Serialize};

/// Event types a webhook subscription can listen to. The list is a closed
/// vocabulary; adding a variant requires a contract change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentWebhookEventType {
    AgentCallCompleted,
    AgentCallFailed,
    TaskRunCompleted,
    TaskRunFailed,
    InteractionRequested,
}

impl AgentWebhookEventType {
    pub const ALL: &'static [Self] = &[
        Self::AgentCallCompleted,
        Self::AgentCallFailed,
        Self::TaskRunCompleted,
        Self::TaskRunFailed,
        Self::InteractionRequested,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AgentCallCompleted => "agent_call.completed",
            Self::AgentCallFailed => "agent_call.failed",
            Self::TaskRunCompleted => "task_run.completed",
            Self::TaskRunFailed => "task_run.failed",
            Self::InteractionRequested => "interaction.requested",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|event_type| event_type.as_str() == code)
    }
}

/// Subscription lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentWebhookStatus {
    Active,
    Disabled,
}

impl AgentWebhookStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Active => 0,
            Self::Disabled => 1,
        }
    }

    pub fn from_db_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::Active),
            1 => Some(Self::Disabled),
            _ => None,
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "active" => Some(Self::Active),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

/// Result of `agents.webhooks.test`: the queued delivery record plus the
/// endpoint URL the HTTP transport must POST to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentWebhookTestOutcome {
    pub delivery: AgentWebhookDeliveryRecord,
    pub url: String,
}

/// One webhook subscription. The signing secret is returned once on
/// creation and is never part of read responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentWebhookRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub webhook_id: String,
    pub url: String,
    pub event_types: Vec<AgentWebhookEventType>,
    pub status: AgentWebhookStatus,
    pub secret: String,
    pub description: Option<String>,
    pub created_by: u64,
    pub created_at: String,
    pub updated_at: String,
}

/// Delivery attempt facts for one webhook event push.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentWebhookDeliveryRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub webhook_id: String,
    pub delivery_id: String,
    pub event_type: String,
    pub payload_json: String,
    pub signature: String,
    pub status: String,
    pub response_code: Option<i32>,
    pub error_detail: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Offset-paginated list query. Webhook subscriptions are a low-volume
/// per-tenant configuration set, so offset paging follows the
/// workspace/project list precedent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentWebhookListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub pagination: crate::ports::PaginationParams,
}

impl AgentWebhookListQuery {
    pub fn for_tenant(tenant_id: u64, organization_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            pagination: crate::ports::PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: crate::ports::PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Generates a fresh signing secret (`whsec_<64 hex chars>`) from the OS
/// entropy source. Secrets are returned exactly once at creation time.
pub fn generate_webhook_secret() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    format!("whsec_{}", hex_encode(&bytes))
}

/// Encodes bytes as lowercase hex.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Computes the `Sdkwork-Signature` header value:
/// `t=<unix-seconds>,v1=<hmac-sha256(secret, "<timestamp>.<payload>")>`.
/// The timestamp is part of the signed content so replays across time fail.
pub fn sign_webhook_payload(secret: &str, payload: &str, unix_seconds: u64) -> String {
    let signed_content = format!("{unix_seconds}.{payload}");
    let signature = hmac_sha256(signed_content.as_bytes(), secret.as_bytes());
    format!("t={unix_seconds},v1={signature}")
}

/// Verifies a signature header against a payload within a clock-skew window.
pub fn verify_webhook_signature(
    secret: &str,
    payload: &str,
    unix_seconds: u64,
    header_value: &str,
    max_skew_seconds: u64,
) -> bool {
    let expected = sign_webhook_payload(secret, payload, unix_seconds);
    let header_ts = header_value
        .split(',')
        .find_map(|part| part.trim().strip_prefix("t="))
        .and_then(|value| value.parse::<u64>().ok());
    let Some(header_ts) = header_ts else {
        return false;
    };
    let skew = header_ts.abs_diff(unix_seconds);
    skew <= max_skew_seconds && secure_compare(&expected, header_value.trim())
}

/// Validates the subscription endpoint: HTTPS-only, no whitespace, bounded
/// length. Plaintext HTTP would leak the signing secret header content.
pub fn validate_webhook_url(url: &str) -> KernelResult<()> {
    if url.len() > 2048 {
        return Err(KernelError::validation("webhook url is too long"));
    }
    if url.chars().any(char::is_whitespace) || url.is_empty() {
        return Err(KernelError::validation(
            "webhook url must be a non-empty https url without whitespace",
        ));
    }
    let parsed = reqwest::Url::parse(url)
        .map_err(|_| KernelError::validation("webhook url must be a valid absolute url"))?;
    if parsed.scheme() != "https" {
        return Err(KernelError::validation(
            "webhook url must use the https scheme",
        ));
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WebhookEventTypeListPayload {
    version: u8,
    event_types: Vec<String>,
}

/// Serializes the subscription's event-type list for durable storage.
pub(crate) fn event_types_to_json(event_types: &[AgentWebhookEventType]) -> String {
    let payload = WebhookEventTypeListPayload {
        version: 1,
        event_types: event_types
            .iter()
            .map(|event_type| event_type.as_str().to_string())
            .collect(),
    };
    serde_json::to_string(&payload).unwrap_or_else(|_| {
        WebhookEventTypeListPayload {
            version: 1,
            event_types: Vec::new(),
        }
        .to_json()
    })
}

impl WebhookEventTypeListPayload {
    fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| r#"{"version":1,"eventTypes":[]}"#.into())
    }
}

/// Parses the durable event-type list; unknown codes fail closed because a
/// silently dropped event type would break the subscription contract.
pub(crate) fn event_types_from_json(json: &str) -> KernelResult<Vec<AgentWebhookEventType>> {
    let payload: WebhookEventTypeListPayload = serde_json::from_str(json).map_err(|error| {
        KernelError::validation(format!("webhook event types corrupt: {error}"))
    })?;
    payload
        .event_types
        .iter()
        .map(|code| {
            AgentWebhookEventType::from_code(code).ok_or_else(|| {
                KernelError::validation(format!("unknown webhook event type: {code}"))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_is_deterministic_and_timestamp_bound() {
        let payload = r#"{"eventId":"evt.test.1","eventType":"agent_call.completed"}"#;
        let header = sign_webhook_payload("whsec_abc", payload, 1_700_000_000);
        assert_eq!(
            header,
            "t=1700000000,v1=".to_string()
                + &hmac_sha256(format!("1700000000.{payload}").as_bytes(), b"whsec_abc")
        );
        assert!(verify_webhook_signature(
            "whsec_abc",
            payload,
            1_700_000_000,
            &header,
            300
        ));
        // A mismatched timestamp inside the window must fail.
        assert!(!verify_webhook_signature(
            "whsec_abc",
            payload,
            1_700_000_100,
            &header,
            300
        ));
        // A tampered payload must fail.
        assert!(!verify_webhook_signature(
            "whsec_abc",
            r#"{"eventId":"evt.test.2"}"#,
            1_700_000_000,
            &header,
            300
        ));
    }

    #[test]
    fn webhook_url_requires_https() {
        assert!(validate_webhook_url("https://hooks.example.com/sdkwork").is_ok());
        assert!(validate_webhook_url("http://hooks.example.com/sdkwork").is_err());
        assert!(validate_webhook_url("ftp://hooks.example.com").is_err());
        assert!(validate_webhook_url("").is_err());
        assert!(validate_webhook_url("https://hooks.example.com/with space").is_err());
    }

    #[test]
    fn event_type_vocabulary_round_trips() {
        let all = AgentWebhookEventType::ALL;
        let json = event_types_to_json(all);
        assert_eq!(event_types_from_json(&json).expect("parse"), all.to_vec());
        assert!(event_types_from_json(r#"{"version":1,"eventTypes":["bogus.event"]}"#).is_err());
        assert!(AgentWebhookEventType::from_code("agent_call.completed").is_some());
        assert!(AgentWebhookEventType::from_code("agent_call.bogus").is_none());
    }

    #[test]
    fn secrets_are_random_and_prefixed() {
        let first = generate_webhook_secret();
        let second = generate_webhook_secret();
        assert!(first.starts_with("whsec_"));
        assert_eq!(first.len(), "whsec_".len() + 64);
        assert_ne!(first, second, "secrets must be random");
    }
}
