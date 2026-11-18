//! The single credential-selection rule for the CloudRouter open-api hop.
//!
//! CloudRouter's `/v1/*` operations are `api-key-or-dual-token` (`API_SPEC.md`
//! §10): the caller presents **either** `X-API-Key` alone **or** the complete
//! `Authorization` + `Access-Token` pair, and "API key mixed with either token"
//! is an explicitly forbidden combination (CloudRouter answers `400
//! invalid_request`).
//!
//! Both transports of a chat turn must therefore agree on **which** of the two
//! credentials to present, and on *which principal* the turn runs as. This
//! module exists so there is exactly one answer.
//!
//! The rule, applied identically by [`crate::turn_transport`] and
//! [`crate::chat_stream`]:
//!
//! 1. A turn that carries a caller identity (a non-blank `Authorization` bearer
//!    from the verified request) presents that caller's dual-token pair
//!    verbatim — `APP_SDK_INTEGRATION_SPEC.md` §5.2.1: a user-scoped call must
//!    never be quietly re-authenticated as the deployment.
//! 2. Only a **caller-less** turn (background/worker work) may present the
//!    deployment-provisioned `X-API-Key` as its declared service identity.
//! 3. A turn with neither presents no credential at all: the destination
//!    rejects it with its own 401 instead of receiving a synthesized or empty
//!    credential header.
//!
//! This ordering was previously inverted (a configured key beat the caller
//! pair), which made the same streaming user turn authenticate as two different
//! principals depending only on the deployment profile — a billing and
//! tenant-scope divergence.

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
    /// `X-API-Key` alone; neither token header is present. Only for a
    /// caller-less (background/worker) turn.
    ApiKey(String),
    /// The caller's `Authorization` bearer plus optional `Access-Token`.
    CallerPair {
        /// The caller's bearer auth token (account-pool identity).
        auth_token: String,
        /// The caller's access token, when the inbound request carried one.
        access_token: Option<String>,
    },
    /// Neither a caller identity nor a configured key: present no credential.
    Unavailable,
}

impl HopCredentials {
    /// Selects the credential source for one hop.
    ///
    /// The caller's identity wins whenever it is present; `api_key` (the
    /// deployment-provisioned service identity) is used only for a caller-less
    /// turn. Both are normalized (trimmed; a blank value behaves as absent so it
    /// can never become an empty credential header).
    pub fn select(api_key: Option<&str>, auth_token: &str, access_token: Option<&str>) -> Self {
        if let Some(auth_token) = normalize(Some(auth_token)) {
            return Self::CallerPair {
                auth_token: auth_token.to_string(),
                access_token: normalize(access_token).map(str::to_string),
            };
        }
        if let Some(api_key) = normalize(api_key) {
            return Self::ApiKey(api_key.to_string());
        }
        Self::Unavailable
    }

    /// The API key when this hop authenticates with one.
    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::ApiKey(api_key) => Some(api_key),
            Self::CallerPair { .. } | Self::Unavailable => None,
        }
    }

    /// Whether this hop presents the caller's dual-token pair.
    pub fn is_caller_pair(&self) -> bool {
        matches!(self, Self::CallerPair { .. })
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
    fn the_caller_pair_wins_over_a_configured_key() {
        assert_eq!(
            HopCredentials::select(Some("sk-deployment"), "auth", Some("access")),
            HopCredentials::CallerPair {
                auth_token: "auth".to_string(),
                access_token: Some("access".to_string()),
            },
            "a user-scoped turn must never be re-authenticated as the deployment"
        );
    }

    #[test]
    fn a_configured_key_is_used_only_for_a_caller_less_turn() {
        for blank in ["", "   "] {
            assert_eq!(
                HopCredentials::select(Some("sk-deployment"), blank, None),
                HopCredentials::ApiKey("sk-deployment".to_string()),
                "a caller-less turn uses the declared service identity"
            );
        }
    }

    #[test]
    fn a_blank_key_and_blank_caller_select_no_credential() {
        assert_eq!(
            HopCredentials::select(Some("   "), "  ", Some("  ")),
            HopCredentials::Unavailable
        );
        assert_eq!(
            HopCredentials::select(None, "", None),
            HopCredentials::Unavailable
        );
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
