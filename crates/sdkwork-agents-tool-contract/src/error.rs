//! Typed media tool errors with stable kinds for boundary mapping.

use std::fmt;

/// Failure kinds for media tool invocations.
///
/// The variant names mirror the Agent Kernel error taxonomy
/// (`AGENT_TOOL_PROVIDER_SPI_SPEC.md` section 8) so application boundaries can
/// map them to stable kernel errors or HTTP problem details without reaching
/// into provider internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaToolError {
    /// Input does not match the tool input schema.
    InvalidInput(String),
    /// Tool is not implemented or not available for the current capability
    /// surface (e.g. sound-effect pending upstream capability).
    CapabilityMissing(String),
    /// Invocation requires an auth token that was not provided.
    AuthRequired(String),
    /// The upstream provider or gateway is unavailable.
    ProviderUnavailable(String),
    /// The upstream provider rejected or failed the call.
    ProviderError(String),
    /// The call exceeded its timeout budget.
    Timeout(String),
    /// The upstream gateway applied a quota or rate limit.
    RateLimited(String),
    /// The caller's own wallet cannot fund this call. Distinct from
    /// `ProviderError`: nothing is broken, the user must recharge, so
    /// boundaries must surface a funding affordance instead of a retry hint.
    FundingRequired(String),
}

impl MediaToolError {
    /// Stable machine-readable error code.
    pub fn code(&self) -> &'static str {
        match self {
            MediaToolError::InvalidInput(_) => "invalid_input",
            MediaToolError::CapabilityMissing(_) => "capability_missing",
            MediaToolError::AuthRequired(_) => "auth_required",
            MediaToolError::ProviderUnavailable(_) => "provider_unavailable",
            MediaToolError::ProviderError(_) => "provider_error",
            MediaToolError::Timeout(_) => "timeout",
            MediaToolError::RateLimited(_) => "rate_limited",
            MediaToolError::FundingRequired(_) => "funding_required",
        }
    }

    /// Convenience constructor for invalid tool arguments.
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        MediaToolError::InvalidInput(message.into())
    }

    /// Convenience constructor for pending upstream capabilities.
    pub fn pending_capability(reason: impl Into<String>) -> Self {
        MediaToolError::CapabilityMissing(reason.into())
    }
}

impl fmt::Display for MediaToolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaToolError::InvalidInput(message)
            | MediaToolError::CapabilityMissing(message)
            | MediaToolError::AuthRequired(message)
            | MediaToolError::ProviderUnavailable(message)
            | MediaToolError::ProviderError(message)
            | MediaToolError::Timeout(message)
            | MediaToolError::RateLimited(message)
            | MediaToolError::FundingRequired(message) => {
                write!(formatter, "{}: {message}", self.code())
            }
        }
    }
}

impl std::error::Error for MediaToolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        assert_eq!(
            MediaToolError::InvalidInput("x".into()).code(),
            "invalid_input"
        );
        assert_eq!(
            MediaToolError::CapabilityMissing("x".into()).code(),
            "capability_missing"
        );
        assert_eq!(
            MediaToolError::AuthRequired("x".into()).code(),
            "auth_required"
        );
        assert_eq!(
            MediaToolError::ProviderUnavailable("x".into()).code(),
            "provider_unavailable"
        );
        assert_eq!(
            MediaToolError::ProviderError("x".into()).code(),
            "provider_error"
        );
        assert_eq!(MediaToolError::Timeout("x".into()).code(), "timeout");
        assert_eq!(
            MediaToolError::RateLimited("x".into()).code(),
            "rate_limited"
        );
        assert_eq!(
            MediaToolError::FundingRequired("x".into()).code(),
            "funding_required"
        );
    }

    #[test]
    fn display_embeds_code_and_message() {
        let error = MediaToolError::pending_capability("cloudrouter has no sound-effect endpoint");
        assert_eq!(
            error.to_string(),
            "capability_missing: cloudrouter has no sound-effect endpoint"
        );
    }
}
