//! The single credential-selection rule for the CloudRouter open-api hop.
//!
//! CloudRouter's `/v1/*` operations are `api-key-or-dual-token` (`API_SPEC.md`
//! §10): the caller presents **either** `X-API-Key` alone **or** the complete
//! `Authorization` + `Access-Token` pair, and "API key mixed with either token"
//! is an explicitly forbidden combination (CloudRouter answers `400
//! invalid_request`).
//!
//! Both transports of a chat turn must therefore agree on **which** of the two
//! credentials to present. This module exists so there is exactly one answer:
//! before it, the in-process arm read `SDKWORK_CLOUDROUTER_OPEN_API_KEY` while
//! the HTTP arm ignored the variable entirely and always sent the caller's
//! token pair. The same turn consequently authenticated as a **different
//! principal** depending only on the deployment profile — a billing and
//! tenant-scope divergence that no single-arm test could observe.
//!
//! The rule, applied identically by [`crate::turn_transport`] and
//! [`crate::chat_stream`]:
//!
//! 1. A configured, non-blank API key selects the API-key-only branch. That
//!    identity is what an embedded deployment provisions for in-process
//!    CloudRouter traffic.
//! 2. Otherwise the caller's dual-token pair is presented verbatim, so the
//!    surface projects the same principal and access context as an external
//!    caller.

/// Environment variable holding the deployment-provisioned CloudRouter
/// open-api key.
///
/// This is the same variable the embedded image/voice generation hosts read,
/// so a deployment that already provisions in-process CloudRouter traffic keeps
/// working without a per-consumer setting.
pub const ENV_CLOUDROUTER_OPEN_API_KEY: &str = "SDKWORK_CLOUDROUTER_OPEN_API_KEY";

/// Header carrying the billing API key (`API_SPEC.md` §10).
///
/// The exact spelling matters: CloudRouter matches these names literally
/// (`sdkwork_cloudrouter_http::auth` constants `x-api-key` / `Access-Token`),
/// and a mismatch silently drops the credential instead of failing.
pub const API_KEY_HEADER: &str = "x-api-key";

/// Header carrying the caller's session access context (`API_SPEC.md` §10).
pub const ACCESS_TOKEN_HEADER: &str = "Access-Token";

/// The credential source one open-api hop presents.
///
/// Modelling the branch as a value (rather than as two hand-written `if`
/// blocks) is what makes "both transports agree" assertable: a test can build
/// the selection once and compare what each arm sends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HopCredentials {
    /// `X-API-Key` alone; neither token header is present.
    ApiKey(String),
    /// The caller's `Authorization` bearer plus optional `Access-Token`.
    CallerPair {
        /// The caller's bearer auth token (account-pool identity).
        auth_token: String,
        /// The caller's access token, when the inbound request carried one.
        access_token: Option<String>,
    },
}

impl HopCredentials {
    /// Selects the credential source for one hop.
    ///
    /// `api_key` is the deployment-provisioned key, normalized (trimmed; a
    /// blank value behaves as absent so it can never become an empty
    /// credential header).
    pub fn select(api_key: Option<&str>, auth_token: &str, access_token: Option<&str>) -> Self {
        if let Some(api_key) = normalize(api_key) {
            return Self::ApiKey(api_key.to_string());
        }
        Self::CallerPair {
            auth_token: auth_token.to_string(),
            access_token: normalize(access_token).map(str::to_string),
        }
    }

    /// The API key when this hop authenticates with one.
    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::ApiKey(api_key) => Some(api_key),
            Self::CallerPair { .. } => None,
        }
    }
}

/// Trims a credential and treats a blank value as absent, so an empty string
/// can never be presented as a credential header.
fn normalize(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Reads the deployment-provisioned open-api key from the environment.
///
/// A blank value is treated as absent rather than as an empty credential.
pub fn embedded_open_api_key() -> Option<String> {
    std::env::var(ENV_CLOUDROUTER_OPEN_API_KEY)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_configured_key_wins_over_the_caller_pair() {
        assert_eq!(
            HopCredentials::select(Some("sk-deployment"), "auth", Some("access")),
            HopCredentials::ApiKey("sk-deployment".to_string())
        );
    }

    #[test]
    fn a_blank_key_behaves_as_absent() {
        // A blank value must not shadow the caller's identity with an empty
        // credential header.
        for blank in ["", "   "] {
            assert_eq!(
                HopCredentials::select(Some(blank), "auth", Some("access")),
                HopCredentials::CallerPair {
                    auth_token: "auth".to_string(),
                    access_token: Some("access".to_string()),
                },
                "blank key {blank:?} must fall back to the caller pair"
            );
        }
    }

    #[test]
    fn an_absent_key_uses_the_caller_pair_and_normalizes_the_access_token() {
        assert_eq!(
            HopCredentials::select(None, "auth", None),
            HopCredentials::CallerPair {
                auth_token: "auth".to_string(),
                access_token: None,
            }
        );
        // A blank access token is absent, not an empty header value.
        assert_eq!(
            HopCredentials::select(None, "auth", Some("  ")),
            HopCredentials::CallerPair {
                auth_token: "auth".to_string(),
                access_token: None,
            }
        );
    }
}
