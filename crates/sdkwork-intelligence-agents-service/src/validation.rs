use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{is_blank, trim};
use std::collections::HashSet;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Canonical durable id prefixes enforced by `validate_standard_id`. Keeping
/// them in one place prevents the id namespaces from drifting apart: every
/// id kind is generated and validated against the same literal.
pub(crate) const ID_PREFIX_AGENT: &str = "agent.";
pub(crate) const ID_PREFIX_ATTEMPT: &str = "attempt.";
pub(crate) const ID_PREFIX_BINDING: &str = "binding.";
pub(crate) const ID_PREFIX_CHECKPOINT: &str = "checkpoint.";
pub(crate) const ID_PREFIX_EXECUTION: &str = "execution.";
pub(crate) const ID_PREFIX_INTERACTION: &str = "interaction.";
pub(crate) const ID_PREFIX_ITEM: &str = "item.";
pub(crate) const ID_PREFIX_MODEL: &str = "model.";
pub(crate) const ID_PREFIX_PROFILE: &str = "profile.";
pub(crate) const ID_PREFIX_PROJECT: &str = "project.";
pub(crate) const ID_PREFIX_PROVIDER: &str = "provider.";
pub(crate) const ID_PREFIX_QUEUE_ENTRY: &str = "queue-entry.";
pub(crate) const ID_PREFIX_REQUEST: &str = "request.";
pub(crate) const ID_PREFIX_RUN: &str = "run.";
pub(crate) const ID_PREFIX_RUNTIME_BINDING: &str = "runtime_binding.";
pub(crate) const ID_PREFIX_SESSION: &str = "session.";
pub(crate) const ID_PREFIX_SLOT: &str = "slot.";
pub(crate) const ID_PREFIX_TASK: &str = "task.";
pub(crate) const ID_PREFIX_TURN: &str = "turn.";
pub(crate) const ID_PREFIX_VERSION: &str = "version.";
pub(crate) const ID_PREFIX_WEBHOOK: &str = "webhook.";
pub(crate) const ID_PREFIX_DELIVERY: &str = "delivery.";
pub(crate) const ID_PREFIX_WORKSPACE: &str = "workspace.";

pub(crate) fn require_non_blank(value: &str, field_name: &str) -> KernelResult<()> {
    if is_blank(Some(value)) {
        return Err(KernelError::validation(format!("{field_name} is required")));
    }
    Ok(())
}

pub(crate) fn require_trimmed_non_blank(value: &str, field_name: &str) -> KernelResult<()> {
    if is_blank(Some(value)) {
        return Err(KernelError::validation(format!("{field_name} is required")));
    }
    if trim(value) != value {
        return Err(KernelError::validation(format!(
            "{field_name} must not contain leading or trailing whitespace"
        )));
    }
    Ok(())
}

pub(crate) fn optional_non_blank(value: String) -> Option<String> {
    if is_blank(Some(value.as_str())) {
        None
    } else {
        Some(value)
    }
}

pub(crate) fn is_trimmed_blank(value: &str) -> bool {
    is_blank(Some(value))
}

pub(crate) fn default_json_object_if_blank(value: &str) -> String {
    if is_trimmed_blank(value) {
        "{}".to_string()
    } else {
        value.to_string()
    }
}

pub(crate) fn default_json_array_if_blank(value: &str) -> String {
    if is_trimmed_blank(value) {
        "[]".to_string()
    } else {
        value.to_string()
    }
}

pub(crate) fn default_plain_text_if_blank(value: &str) -> String {
    if is_trimmed_blank(value) {
        "text/plain".to_string()
    } else {
        value.to_string()
    }
}

pub(crate) fn parse_int64_string_field(value: &str, field_name: &str) -> KernelResult<u64> {
    value
        .parse::<u64>()
        .map_err(|_| KernelError::validation(format!("{field_name} must be int64 string")))
}

pub(crate) fn parse_tenant_id(value: &str) -> KernelResult<u64> {
    let parsed = parse_int64_string_field(value, "tenant_id")?;
    // tenant_id is the mandatory multi-tenant isolation key. The value `0` is
    // reserved (used by `unwrap_or(0)` fallbacks in audit/persistence code
    // paths) and must never be accepted as a real tenant identifier, otherwise
    // a request that loses its tenant header could read or write cross-tenant
    // data scoped to the synthetic tenant `0`. Reject it at the parsing boundary.
    if parsed == 0 {
        return Err(KernelError::validation("tenant_id must be greater than 0"));
    }
    Ok(parsed)
}

pub(crate) fn parse_organization_id(value: &str) -> KernelResult<u64> {
    parse_int64_string_field(value, "organization_id")
}

pub(crate) fn parse_owner_user_id(value: &str) -> KernelResult<u64> {
    parse_int64_string_field(value, "owner_user_id")
}

pub(crate) fn parse_expected_version(value: &str) -> KernelResult<u64> {
    parse_int64_string_field(value, "expectedVersion")
}

pub(crate) fn validate_requested_at(value: &str) -> KernelResult<()> {
    validate_rfc3339_datetime(value, "requestedAt")
}

pub(crate) fn validate_standard_id(
    value: &str,
    field_name: &str,
    required_prefix: Option<&str>,
) -> KernelResult<()> {
    require_trimmed_non_blank(value, field_name)?;
    if value.chars().count() > 128 {
        return Err(KernelError::validation(format!(
            "{field_name} must be at most 128 characters"
        )));
    }
    if !value.chars().all(is_standard_id_character) {
        return Err(KernelError::validation(format!(
            "{field_name} must use lowercase standard id characters"
        )));
    }
    if !has_non_empty_dot_segments(value) {
        return Err(KernelError::validation(format!(
            "{field_name} must use non-empty dot-delimited segments"
        )));
    }
    if let Some(prefix) = required_prefix {
        if !value.starts_with(prefix) {
            return Err(KernelError::validation(format!(
                "{field_name} must start with {prefix}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_capabilities(capabilities: &[String], field_name: &str) -> KernelResult<()> {
    let mut seen = HashSet::new();
    for capability in capabilities {
        if is_blank(Some(capability.as_str())) {
            return Err(KernelError::validation(format!(
                "{field_name} must not contain empty capability ids"
            )));
        }
        if capability.chars().count() > 128 {
            return Err(KernelError::validation(format!(
                "{field_name} capability ids must be at most 128 characters"
            )));
        }
        if trim(capability) != *capability || !is_valid_capability_id(capability.as_str()) {
            return Err(KernelError::validation(format!(
                "{field_name} must use lowercase namespaced capability ids"
            )));
        }
        if !seen.insert(capability.as_str()) {
            return Err(KernelError::validation(format!(
                "{field_name} must not contain duplicate capability id: {capability}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_rfc3339_datetime(value: &str, field_name: &str) -> KernelResult<()> {
    let _ = parse_rfc3339_datetime(value, field_name)?;
    Ok(())
}

pub(crate) fn parse_rfc3339_datetime(
    value: &str,
    field_name: &str,
) -> KernelResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|error| {
        KernelError::validation(format!("{field_name} must be RFC3339 date-time: {error}"))
    })
}

#[cfg(feature = "http-axum")]
pub(crate) fn parse_optional_rfc3339_datetime(
    value: Option<&str>,
    field_name: &str,
) -> KernelResult<Option<OffsetDateTime>> {
    let Some(value) = value else {
        return Ok(None);
    };
    parse_rfc3339_datetime(value, field_name).map(Some)
}

// ---------------------------------------------------------------------------
// Agent list scope filters
// ---------------------------------------------------------------------------

/// Selector value that keeps every agent category in a listing.
pub(crate) const AGENT_LIST_ALL_CATEGORIES: &str = "all";

/// Localized equivalent of [`AGENT_LIST_ALL_CATEGORIES`] the clients ship.
const AGENT_LIST_ALL_CATEGORIES_ZH: &str = "全部";

/// Normalizes a `category_code` selector to `None` (every category) or one code.
///
/// The category registry codes are lower-kebab (`ai_agent.category_code` is
/// constrained to `^[a-z0-9][a-z0-9_-]{1,127}$`), so the sentinel comparison is
/// case-insensitive and the returned code is lowercased. The SQL push-down then
/// compares the canonical column directly and the in-memory adapter matches the
/// same code set.
pub(crate) fn normalize_category_filter(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case(AGENT_LIST_ALL_CATEGORIES)
        || trimmed == AGENT_LIST_ALL_CATEGORIES_ZH
    {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

/// Parses a comma-separated `role_codes` value into a sorted, unique scope.
///
/// Codes are IAM role codes (`NAMING_SPEC.md` role-code pattern); blank entries
/// are dropped rather than rejected so a trailing comma stays harmless. The
/// stored codes are canonical lowercase, so the filter lowercases each code and
/// normalizes the order; the JSONB `?|` push-down then matches the stored array
/// exactly.
pub(crate) fn normalize_role_code_filter(value: Option<&str>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    let mut codes = value
        .split(',')
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    codes.sort();
    codes.dedup();
    codes
}

/// True when an agent restricted to `allowed` is part of the caller's scope.
///
/// An empty `role_codes` is the "all data" view and matches everything. An
/// empty `allowed` list means the agent is open to every role (the documented
/// default of `ai_agent.allowed_role_codes`), so it matches every scope even
/// when the caller narrows the listing by role.
pub(crate) fn role_scope_allows(allowed: &[String], role_codes: &[String]) -> bool {
    role_codes.is_empty()
        || allowed.is_empty()
        || allowed.iter().any(|code| {
            role_codes
                .iter()
                .any(|scope| scope.eq_ignore_ascii_case(code))
        })
}

/// True when an agent's category matches the normalized category filter.
///
/// An agent without a category never matches a specific category filter; it
/// still appears in the all-category view because that view passes `None`.
pub(crate) fn category_matches(record_category: Option<&str>, category: &str) -> bool {
    record_category.is_some_and(|value| value.eq_ignore_ascii_case(category))
}

/// True when an agent satisfies the optional category and role list filters.
///
/// Both filters only remove rows from the already tenant/org/visibility/owner
/// scoped listing, so this predicate can never widen visibility.
pub(crate) fn agent_list_scope_filter_matches(
    record_category: Option<&str>,
    record_allowed_role_codes: &[String],
    category_code: Option<&str>,
    role_codes: &[String],
) -> bool {
    if let Some(category) = category_code {
        if !category_matches(record_category, category) {
            return false;
        }
    }
    role_scope_allows(record_allowed_role_codes, role_codes)
}

fn is_valid_capability_id(capability_id: &str) -> bool {
    capability_id.chars().all(is_standard_id_character) && has_non_empty_dot_segments(capability_id)
}

fn is_standard_id_character(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-')
}

fn has_non_empty_dot_segments(value: &str) -> bool {
    let mut segments = value.split('.');
    let mut segment_count = 0;
    for segment in &mut segments {
        segment_count += 1;
        if segment.is_empty() {
            return false;
        }
    }
    segment_count >= 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::KernelError;

    #[test]
    fn validate_rfc3339_datetime_accepts_valid_timestamp() {
        let result = validate_rfc3339_datetime("2026-06-01T00:00:00Z", "requestedAt");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_rfc3339_datetime_rejects_invalid_timestamp() {
        let error = validate_rfc3339_datetime("2026-06-01", "requestedAt")
            .expect_err("invalid timestamp should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("requestedAt"));
                assert!(message.contains("RFC3339"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn parse_int64_string_field_accepts_unsigned_integer_text() {
        let value = parse_int64_string_field("12345", "tenant_id")
            .expect("valid int64 string should parse");
        assert_eq!(value, 12345);
    }

    #[test]
    fn parse_int64_string_field_rejects_non_numeric_text() {
        let error = parse_int64_string_field("12x45", "tenant_id")
            .expect_err("invalid int64 string should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("tenant_id"));
                assert!(message.contains("int64 string"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn parse_tenant_id_uses_tenant_specific_error() {
        let error = parse_tenant_id("bad").expect_err("invalid tenant id should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("tenant_id"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn parse_tenant_id_rejects_zero() {
        // tenant_id=0 is reserved and must never be accepted as a real
        // tenant identifier, otherwise cross-tenant access becomes possible.
        let error = parse_tenant_id("0").expect_err("tenant_id 0 should be rejected");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("tenant_id"));
                assert!(message.contains("greater than 0"));
            }
            _ => panic!("expected validation error for tenant_id 0"),
        }
    }

    #[test]
    fn parse_tenant_id_accepts_positive_value() {
        let value = parse_tenant_id("100001").expect("positive tenant_id should parse");
        assert_eq!(value, 100001);
    }

    #[test]
    fn validate_requested_at_uses_api_field_name() {
        let error =
            validate_requested_at("2026-06-01").expect_err("invalid requestedAt should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("requestedAt"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn validate_standard_id_rejects_non_standard_values() {
        let error = validate_standard_id("Provider.Model", "providerId", Some(ID_PREFIX_PROVIDER))
            .expect_err("uppercase standard id should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("providerId"));
                assert!(message.contains("lowercase standard id characters"));
            }
            _ => panic!("expected validation error"),
        }

        let error = validate_standard_id("provider.", "providerId", Some(ID_PREFIX_PROVIDER))
            .expect_err("prefix-only standard id should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("providerId"));
                assert!(message.contains("dot-delimited"));
            }
            _ => panic!("expected validation error"),
        }

        let error = validate_standard_id("provider..rig", "providerId", Some(ID_PREFIX_PROVIDER))
            .expect_err("empty standard id segment should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("providerId"));
                assert!(message.contains("dot-delimited"));
            }
            _ => panic!("expected validation error"),
        }

        let error = validate_standard_id("model.rig", "providerId", Some(ID_PREFIX_PROVIDER))
            .expect_err("missing standard prefix should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("providerId"));
                assert!(message.contains("provider."));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn validate_capabilities_rejects_non_standard_values() {
        let capabilities = vec!["model.chat".to_string(), "model.chat".to_string()];
        let error = validate_capabilities(&capabilities, "capabilities")
            .expect_err("duplicate capability should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("capabilities"));
                assert!(message.contains("duplicate"));
            }
            _ => panic!("expected validation error"),
        }

        let capabilities = vec!["chat".to_string()];
        let error = validate_capabilities(&capabilities, "capabilities")
            .expect_err("unnamespaced capability should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("capabilities"));
                assert!(message.contains("namespaced"));
            }
            _ => panic!("expected validation error"),
        }

        let capabilities = vec!["model.".to_string()];
        let error = validate_capabilities(&capabilities, "capabilities")
            .expect_err("capability with empty segment should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("capabilities"));
                assert!(message.contains("namespaced"));
            }
            _ => panic!("expected validation error"),
        }

        let capabilities = vec![format!("model.{}", "a".repeat(123))];
        let error = validate_capabilities(&capabilities, "capabilities")
            .expect_err("capability over 128 characters should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("capabilities"));
                assert!(message.contains("at most 128"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn parse_expected_version_uses_api_field_name() {
        let error = parse_expected_version("1x").expect_err("invalid expectedVersion should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("expectedVersion"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn category_filter_drops_the_all_selectors() {
        assert_eq!(normalize_category_filter(None), None);
        assert_eq!(normalize_category_filter(Some("  ")), None);
        assert_eq!(normalize_category_filter(Some("ALL")), None);
        assert_eq!(
            normalize_category_filter(Some(AGENT_LIST_ALL_CATEGORIES_ZH)),
            None
        );
        assert_eq!(
            normalize_category_filter(Some(" Engineering ")),
            Some("engineering".to_string())
        );
    }

    #[test]
    fn role_code_filter_trims_sorts_dedupes_and_lowercases() {
        assert_eq!(normalize_role_code_filter(None), Vec::<String>::new());
        assert_eq!(
            normalize_role_code_filter(Some("org_admin, ORG_OPERATIONS,,org_admin,")),
            vec!["org_admin".to_string(), "org_operations".to_string()]
        );
        assert_eq!(
            normalize_role_code_filter(Some(" , ")),
            Vec::<String>::new()
        );
    }

    #[test]
    fn role_scope_matches_open_agents_and_named_roles() {
        let all_data: Vec<String> = Vec::new();
        let operator = vec!["org_admin".to_string()];
        let open: Vec<String> = Vec::new();
        let restricted = vec!["org_finance".to_string()];

        // The all-data view keeps every agent, including restricted ones.
        assert!(role_scope_allows(&restricted, &all_data));
        // An open agent is always visible, even to a role-narrowed caller.
        assert!(role_scope_allows(&open, &operator));
        // The match is case-insensitive across the stored and requested codes.
        assert!(role_scope_allows(&["ORG_ADMIN".to_string()], &operator));
        // A different role scope does not leak the restricted agent.
        assert!(!role_scope_allows(&restricted, &operator));
    }

    #[test]
    fn category_match_is_case_insensitive_and_never_matches_absent_codes() {
        assert!(category_matches(Some("Engineering"), "engineering"));
        assert!(!category_matches(Some("education"), "engineering"));
        assert!(!category_matches(None, "engineering"));
    }

    #[test]
    fn scope_filters_compose_without_widening_the_listing() {
        let open: Vec<String> = Vec::new();
        let finance = vec!["org_finance".to_string()];
        let operator = vec!["org_admin".to_string()];

        // All-data view: no category restriction, no role restriction.
        assert!(agent_list_scope_filter_matches(
            Some("engineering"),
            &finance,
            None,
            &[]
        ));
        // Category alone narrows to that category.
        assert!(agent_list_scope_filter_matches(
            Some("engineering"),
            &finance,
            Some("engineering"),
            &[]
        ));
        assert!(!agent_list_scope_filter_matches(
            Some("education"),
            &finance,
            Some("engineering"),
            &[]
        ));
        // Role alone keeps open agents plus the named role.
        assert!(agent_list_scope_filter_matches(
            Some("engineering"),
            &open,
            None,
            &operator
        ));
        assert!(agent_list_scope_filter_matches(
            Some("engineering"),
            &["org_admin".to_string()],
            None,
            &operator
        ));
        assert!(!agent_list_scope_filter_matches(
            Some("engineering"),
            &finance,
            None,
            &operator
        ));
        // Both filters compose: the row must satisfy category AND role scope.
        assert!(agent_list_scope_filter_matches(
            Some("engineering"),
            &open,
            Some("engineering"),
            &operator
        ));
        assert!(!agent_list_scope_filter_matches(
            Some("education"),
            &open,
            Some("engineering"),
            &operator
        ));
        assert!(!agent_list_scope_filter_matches(
            Some("engineering"),
            &finance,
            Some("engineering"),
            &operator
        ));
    }
}
