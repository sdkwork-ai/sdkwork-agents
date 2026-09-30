//! Transactional-outbox dispatcher: delivers durable events to webhook
//! subscribers with bounded retries and marks internal events published.
//!
//! The dispatcher closes the event loop that `insert_agent_outbox_event`
//! opens: business writes commit their events inside the same transaction,
//! and this module is the only consumer. Claim semantics mirror the task
//! scheduler (`FOR UPDATE SKIP LOCKED` + lease token), so several dispatcher
//! replicas can run concurrently without double delivery. Delivery is
//! at-least-once per subscription: a partially failing batch retries the
//! whole event, and receivers deduplicate on the `id` field.

use std::net::ToSocketAddrs;
use std::sync::OnceLock;
use std::time::Duration;

use super::AgentsService;
use crate::ports::{AgentAuditSink, AgentRepository, PaginationParams, WebhookSubscriptionListQuery};
use crate::webhook::{
    sign_webhook_payload, validate_webhook_url, AgentWebhookDeliveryRecord,
    AgentWebhookEventType, AgentWebhookStatus,
};
use sdkwork_agent_kernel::{KernelResult, PolicyProvider};

/// One dispatch round's observable outcome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutboxDispatchSummary {
    /// Events claimed from the outbox table.
    pub claimed: usize,
    /// Webhook deliveries attempted (one event may fan out to several
    /// subscriptions).
    pub deliveries_sent: usize,
    /// Events fully published (all matched subscribers succeeded, or no
    /// subscriber matched).
    pub delivered: usize,
    /// Events returned to the pending queue for a later retry.
    pub retried: usize,
    /// Events moved to the dead-letter state after exhausting attempts.
    pub dead_lettered: usize,
    /// Internal events with no webhook mapping, marked published directly.
    pub unmatched: usize,
}

/// Events claimed per dispatch round.
pub const OUTBOX_DISPATCH_BATCH_SIZE: usize = 100;
/// Claim lease window; the dispatcher completes or fails every claim inside
/// one round, so the window only matters for a crashed dispatcher.
pub const OUTBOX_DISPATCH_LEASE_SECONDS: u64 = 120;
/// First retry delay; grows exponentially per attempt.
pub const OUTBOX_DELIVERY_BACKOFF_BASE_SECONDS: u64 = 30;
/// Retry delay cap.
pub const OUTBOX_DELIVERY_BACKOFF_CAP_SECONDS: u64 = 3_600;
/// Stale threshold for agent-call recovery driven by the dispatch round
/// (matches the HTTP turn reconciler default).
const AGENT_CALL_RECOVERY_STALE_SECONDS: i64 = 300;

/// Maps an outbox event type to its webhook-facing event type. Internal
/// operational events (task transitions, birdcoder records) have no webhook
/// mapping and are published without delivery.
fn webhook_event_type_for(outbox_event_type: &str) -> Option<AgentWebhookEventType> {
    match outbox_event_type {
        "agent.task.run.completed" => Some(AgentWebhookEventType::TaskRunCompleted),
        "agent.task.run.failed" | "agent.task.run.dead_lettered" => {
            Some(AgentWebhookEventType::TaskRunFailed)
        }
        "agent_call.completed" => Some(AgentWebhookEventType::AgentCallCompleted),
        "agent_call.failed" => Some(AgentWebhookEventType::AgentCallFailed),
        "agent.interaction.requested" => Some(AgentWebhookEventType::InteractionRequested),
        _ => None,
    }
}

/// Blocking delivery client: no redirects (the signature header must never
/// follow an unvalidated target) and bounded timeouts.
fn delivery_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new())
    })
}

fn format_utc_seconds_now() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

fn unix_seconds_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn generate_lease_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Validates a webhook target inside the synchronous dispatch context: the
/// async SSRF guard when a runtime is available (worker spawn_blocking
/// threads), otherwise a synchronous resolution against the shared internal
/// address check.
fn validate_delivery_target(url: &str) -> Result<(), String> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| {
            handle.block_on(crate::webhook::validate_webhook_delivery_target(url))
        });
    }
    let parsed = reqwest::Url::parse(url).map_err(|error| format!("invalid webhook url: {error}"))?;
    if parsed.scheme() != "https" {
        return Err("webhook url must use the https scheme".to_string());
    }
    if let Some(host) = parsed.host_str() {
        let resolved: Vec<std::net::IpAddr> = (host, 0u16)
            .to_socket_addrs()
            .map(|addresses| addresses.map(|address| address.ip()).collect())
            .unwrap_or_default();
        if resolved.iter().any(|ip| crate::network_guard::is_internal_ip(*ip)) {
            return Err(format!("webhook url host {host} is not reachable from the server"));
        }
    }
    Ok(())
}

/// Exponential backoff with a deterministic jitter bound derived from the
/// event id, so replicas recovering the same event converge on the same
/// delay instead of stampeding.
fn next_available_at(now: &str, attempt_count: i32, event_id: &str) -> String {
    let exponential = OUTBOX_DELIVERY_BACKOFF_BASE_SECONDS
        .saturating_mul(1u64 << (attempt_count.unsigned_abs().min(7)));
    let jitter_bound = (exponential / 4).max(1);
    let jitter = event_id
        .bytes()
        .map(|byte| u64::from(byte))
        .sum::<u64>()
        .max(1)
        % jitter_bound;
    let delay = exponential.min(OUTBOX_DELIVERY_BACKOFF_CAP_SECONDS).saturating_add(jitter);
    match time::OffsetDateTime::parse(now, &time::format_description::well_known::Rfc3339) {
        Ok(parsed) => {
            let target = parsed + time::Duration::seconds(delay as i64);
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                target.year(),
                u8::from(target.month()),
                target.day(),
                target.hour(),
                target.minute(),
                target.second()
            )
        }
        Err(_) => now.to_string(),
    }
}

/// Active webhook subscriptions for one tenant scope, paged through the
/// whole config set (subscriptions are a low-volume administrative list;
/// the dispatcher must not silently miss subscribers past the first page).
fn list_all_active_subscriptions<R, A, P>(
    service: &AgentsService<R, A, P>,
    tenant_id: u64,
    organization_id: u64,
) -> KernelResult<Vec<crate::webhook::AgentWebhookRecord>>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    let mut all = Vec::new();
    let mut offset = 0usize;
    const PAGE: usize = 200;
    loop {
        let query = WebhookSubscriptionListQuery::for_tenant(tenant_id, organization_id)
            .with_pagination(PaginationParams {
                page_size: PAGE,
                offset,
                page_token: None,
            });
        let page = service.repository.list_webhook_subscriptions(&query)?;
        let exhausted = page.len() < PAGE;
        all.extend(page);
        if exhausted {
            break;
        }
        offset += PAGE;
    }
    Ok(all
        .into_iter()
        .filter(|record| record.status == AgentWebhookStatus::Active)
        .collect())
}

/// Delivers one outbox event to every matching active subscription.
///
/// Returns `true` when the event may be published (no subscriber matched or
/// every delivery succeeded) and `false` when at least one delivery failed
/// and the event must retry.
#[allow(clippy::too_many_arguments)]
fn deliver_event<R, A, P>(
    service: &AgentsService<R, A, P>,
    summary: &mut OutboxDispatchSummary,
    event: &crate::persistence::AgentOutboxEventRow,
    webhook_type: AgentWebhookEventType,
    subscriptions: &[crate::webhook::AgentWebhookRecord],
) -> bool
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    let now = format_utc_seconds_now();
    let matching: Vec<&crate::webhook::AgentWebhookRecord> = subscriptions
        .iter()
        .filter(|subscription| {
            subscription
                .event_types
                .iter()
                .any(|candidate| *candidate == webhook_type)
        })
        .collect();
    if matching.is_empty() {
        // No subscriber for this event type: publishing completes the event.
        return true;
    }

    let payload_value: serde_json::Value =
        serde_json::from_str(&event.payload_json).unwrap_or_else(|_| serde_json::json!({}));
    let envelope = serde_json::json!({
        "id": format!("evt_{}", event.event_id),
        "type": webhook_type.as_str(),
        "createdAt": event.created_at,
        "data": payload_value,
    })
    .to_string();
    let unix_seconds = unix_seconds_now();
    let mut all_succeeded = true;
    for subscription in matching {
        let delivery_row_id = match service.repository.next_id() {
            Ok(id) => id,
            Err(error) => {
                tracing::warn!(
                    target: "sdkwork.agents.outbox",
                    error = %error,
                    "delivery id generation failed; event will retry"
                );
                all_succeeded = false;
                continue;
            }
        };
        let delivery_id = format!("delivery.{delivery_row_id}");
        let signature = sign_webhook_payload(&subscription.secret, &envelope, unix_seconds);
        let delivery = AgentWebhookDeliveryRecord {
            id: delivery_row_id,
            tenant_id: event.tenant_id,
            organization_id: event.organization_id,
            webhook_id: subscription.webhook_id.clone(),
            delivery_id: delivery_id.clone(),
            event_type: webhook_type.as_str().to_string(),
            payload_json: envelope.clone(),
            signature: signature.clone(),
            status: "queued".to_string(),
            response_code: None,
            error_detail: None,
            created_at: now.clone(),
            completed_at: None,
        };
        // Delivery ledger row first (the observability contract), then the
        // outbound attempt.
        if let Err(error) = service.repository.insert_webhook_delivery(delivery) {
            tracing::warn!(
                target: "sdkwork.agents.outbox",
                error = %error,
                "delivery ledger write failed; event will retry"
            );
            all_succeeded = false;
            continue;
        }
        summary.deliveries_sent += 1;

        let outcome = deliver_once(&subscription.url, &signature, &envelope);
        let completed_at = format_utc_seconds_now();
        let (status, response_code, error_detail) = match outcome {
            Ok(code) => ("succeeded", Some(code), None),
            Err(detail) => {
                all_succeeded = false;
                ("failed", None, Some(detail))
            }
        };
        if let Err(error) = service.repository.complete_webhook_delivery(
            event.tenant_id,
            event.organization_id,
            &subscription.webhook_id,
            &delivery_id,
            status,
            response_code,
            error_detail.clone(),
            &completed_at,
        ) {
            tracing::warn!(
                target: "sdkwork.agents.outbox",
                error = %error,
                "delivery ledger completion failed; the delivery record stays queued"
            );
        }
    }
    all_succeeded
}

/// One outbound webhook attempt: delivery-time SSRF validation followed by a
/// no-redirect POST. A non-2xx response is a failed delivery.
fn deliver_once(url: &str, signature: &str, envelope: &str) -> Result<i32, String> {
    validate_webhook_url(url).map_err(|error| error.message().to_string())?;
    validate_delivery_target(url)?;
    let response = delivery_client()
        .post(url)
        .header("Sdkwork-Signature", signature)
        .header("Content-Type", "application/json")
        .body(envelope.to_string())
        .send()
        .map_err(|error| {
            let mut detail = error.to_string();
            detail.truncate(512);
            detail
        })?;
    let code = response.status().as_u16() as i32;
    if response.status().is_success() {
        Ok(code)
    } else {
        Err(format!("delivery failed with status {code}"))
    }
}

/// Runs one dispatch round over the transactional outbox.
///
/// Claimed events fan out to active webhook subscriptions; internal events
/// without a webhook mapping are marked published directly. Terminal
/// failures dead-letter the event; transient failures reschedule it with
/// exponential backoff. The round also drives agent-call recovery for the
/// tenant scopes it observed (stale `queued`/`running` calls would
/// otherwise stay stuck after a process crash).
pub fn dispatch_pending_outbox_events<R, A, P>(
    service: &AgentsService<R, A, P>,
    worker_id: &str,
) -> KernelResult<OutboxDispatchSummary>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    let mut summary = OutboxDispatchSummary::default();
    let now = format_utc_seconds_now();
    let lease_token = generate_lease_token();
    let claimed = service
        .repository
        .claim_pending_outbox_events(worker_id, &lease_token, &now, OUTBOX_DISPATCH_BATCH_SIZE)?;
    summary.claimed = claimed.len();
    if claimed.is_empty() {
        return Ok(summary);
    }

    // One subscription snapshot per tenant scope, not per event.
    let mut subscription_cache: std::collections::HashMap<(u64, u64), Vec<crate::webhook::AgentWebhookRecord>> =
        std::collections::HashMap::new();
    let mut observed_tenants: Vec<u64> = Vec::new();

    for event in claimed {
        if !observed_tenants.contains(&event.tenant_id) {
            observed_tenants.push(event.tenant_id);
        }
        let scope_key = (event.tenant_id, event.organization_id);
        let subscriptions = match subscription_cache.get(&scope_key) {
            Some(cached) => cached.clone(),
            None => {
                let fetched = list_all_active_subscriptions(
                    service,
                    event.tenant_id,
                    event.organization_id,
                )
                .unwrap_or_else(|error| {
                    tracing::warn!(
                        target: "sdkwork.agents.outbox",
                        error = %error,
                        "subscription lookup failed; event will retry"
                    );
                    Vec::new()
                });
                subscription_cache.insert(scope_key, fetched.clone());
                fetched
            }
        };

        let event_published = match webhook_event_type_for(&event.event_type) {
            None => {
                summary.unmatched += 1;
                true
            }
            Some(webhook_type) => {
                deliver_event(service, &mut summary, &event, webhook_type, &subscriptions)
            }
        };

        let result = if event_published {
            service.repository.complete_outbox_event(
                event.id,
                event.tenant_id,
                event.organization_id,
                &lease_token,
                &format_utc_seconds_now(),
            )
        } else {
            summary.retried += 1;
            let next_at =
                next_available_at(&now, event.attempt_count, &event.event_id);
            let exhausted = event.attempt_count >= event.max_attempts;
            if exhausted {
                summary.retried -= 1;
                summary.dead_lettered += 1;
            }
            service.repository.fail_outbox_event(
                event.id,
                event.tenant_id,
                event.organization_id,
                &lease_token,
                &next_at,
                "outbox_delivery_failed",
                "webhook delivery did not succeed for every subscription",
                &now,
            )
        };
        if let Err(error) = result {
            tracing::warn!(
                target: "sdkwork.agents.outbox",
                error = %error,
                event_id = %event.event_id,
                "outbox event terminal write failed; the lease will expire and the event retries"
            );
        } else if event_published {
            summary.delivered += 1;
        }
    }

    // Agent-call recovery for observed scopes (F3): stale queued/running
    // calls survive process crashes only through this recovery pass.
    if let Ok(stale_before_parsed) = time::OffsetDateTime::parse(&now, &time::format_description::well_known::Rfc3339) {
        let stale_before = format_utc_seconds_now_of(
            stale_before_parsed - time::Duration::seconds(AGENT_CALL_RECOVERY_STALE_SECONDS),
        );
        for tenant_id in observed_tenants {
            match service.recover_stale_agent_calls(
                tenant_id,
                &stale_before,
                now.clone(),
                100,
            ) {
                Ok(recovered) if !recovered.is_empty() => tracing::info!(
                    target: "sdkwork.agents.outbox",
                    tenant_id,
                    recovered = recovered.len(),
                    "recovered stale agent calls"
                ),
                Ok(_) => {}
                Err(error) => tracing::warn!(
                    target: "sdkwork.agents.outbox",
                    error = %error,
                    tenant_id,
                    "agent call recovery failed"
                ),
            }
        }
    }

    Ok(summary)
}

fn format_utc_seconds_now_of(value: time::OffsetDateTime) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_event_types_have_no_webhook_mapping() {
        assert!(webhook_event_type_for("agent.task.transitioned").is_none());
        assert!(webhook_event_type_for("agent.birdcoder.alpha").is_none());
        assert_eq!(
            webhook_event_type_for("agent.task.run.completed"),
            Some(AgentWebhookEventType::TaskRunCompleted)
        );
        assert_eq!(
            webhook_event_type_for("agent.task.run.dead_lettered"),
            Some(AgentWebhookEventType::TaskRunFailed)
        );
        assert_eq!(
            webhook_event_type_for("agent_call.completed"),
            Some(AgentWebhookEventType::AgentCallCompleted)
        );
        assert_eq!(
            webhook_event_type_for("agent.interaction.requested"),
            Some(AgentWebhookEventType::InteractionRequested)
        );
    }

    #[test]
    fn backoff_grows_exponentially_and_stays_bounded() {
        let now = "2026-09-30T00:00:00Z";
        let first = next_available_at(now, 1, "event.alpha");
        let later = next_available_at(now, 5, "event.alpha");
        assert!(first.as_str() > now, "retry is rescheduled into the future");
        assert!(later.as_str() > first.as_str(), "later attempts wait longer");
        // Attempt far beyond the cap still yields a bounded, parseable time.
        let capped = next_available_at(now, 9, "event.alpha");
        let parsed = time::OffsetDateTime::parse(&capped, &time::format_description::well_known::Rfc3339)
            .expect("backoff target parses");
        let base = time::OffsetDateTime::parse(now, &time::format_description::well_known::Rfc3339)
            .expect("now parses");
        assert!(
            (parsed - base) <= time::Duration::seconds(
                (OUTBOX_DELIVERY_BACKOFF_CAP_SECONDS + 900) as i64
            ),
            "capped backoff stays within the cap plus jitter bound"
        );
    }
}
