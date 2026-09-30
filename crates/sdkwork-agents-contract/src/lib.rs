//! Runtime environment helpers for SDKWork Agents.
//!
//! This crate centralises process-environment detection used by route and
//! integration-test crates: environment-name resolution, production-profile
//! gating, and the dev inline-auth bypass switch. It intentionally owns no
//! DTO, enum, path, or error contract — the service crate owns the live
//! wire-level types under `sdkwork-intelligence-agents-service`.

mod runtime_env;

pub use runtime_env::env_test_lock;

// ── Runtime environment helpers ─────────────────────────────────────────────

const PRODUCTION_LIKE_ENV_IDENTIFIERS: &[&str] =
    &["production", "prod", "staging", "stage", "live", "test"];

/// Returns the canonical deployment environment name used for security gating.
///
/// Resolution order matches platform deployment conventions so a single source
/// governs dev bypass, policy providers, and bootstrap wiring.
pub fn agents_deployment_environment_name() -> String {
    std::env::var("SDKWORK_DEPLOYMENT_ENV")
        .or_else(|_| std::env::var("ENVIRONMENT"))
        .or_else(|_| std::env::var("SDKWORK_AGENTS_ENVIRONMENT"))
        .or_else(|_| std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE"))
        .unwrap_or_else(|_| "development".to_string())
        .to_ascii_lowercase()
}

/// Returns the agents application profile name (development, production, etc.).
pub fn agents_environment_name() -> String {
    std::env::var("SDKWORK_AGENTS_ENVIRONMENT")
        .or_else(|_| std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE"))
        .unwrap_or_else(|_| agents_deployment_environment_name())
        .to_ascii_lowercase()
}

/// Returns true for production-like profiles that must never use dev inline auth.
pub fn agents_is_production_like_environment() -> bool {
    PRODUCTION_LIKE_ENV_IDENTIFIERS
        .iter()
        .any(|id| agents_deployment_environment_name() == *id)
}

/// True when at least one variable naming the deployment environment is
/// explicitly configured (non-empty).
pub fn agents_environment_is_explicitly_configured() -> bool {
    [
        "SDKWORK_DEPLOYMENT_ENV",
        "ENVIRONMENT",
        "SDKWORK_AGENTS_ENVIRONMENT",
        "SDKWORK_AGENTS_CONFIG_PROFILE",
    ]
    .iter()
    .any(|key| {
        std::env::var(key)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
    })
}

/// When false, preview/prompt-optimization must not silently echo input without an agent engine.
///
/// Fail-closed: an unset environment is treated as production-like, matching
/// the web layer's posture for unknown environments. A deployment that forgot
/// to configure its environment must not silently enable the deterministic
/// fallback that echoes user input as model output.
pub fn agents_allow_contract_runtime_fallback() -> bool {
    agents_environment_is_explicitly_configured() && !agents_is_production_like_environment()
}

/// `SDKWORK_AGENTS_DEV_AUTH_BYPASS` enables inline dev credentials only outside production profiles.
pub fn agents_dev_auth_bypass_enabled() -> bool {
    std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS")
        .ok()
        .and_then(|value| sdkwork_utils_rust::parse_bool(value.trim()))
        .unwrap_or(false)
}

/// Whether HTTP surfaces may use inline dev auth resolver.
pub fn agents_use_dev_inline_auth_resolver() -> bool {
    !agents_is_production_like_environment() && agents_dev_auth_bypass_enabled()
}

/// Fail closed when dev auth bypass is enabled in a production-like deployment.
pub fn ensure_dev_auth_bypass_allowed() -> Result<(), String> {
    if agents_dev_auth_bypass_enabled() && agents_is_production_like_environment() {
        return Err(format!(
            "SDKWORK_AGENTS_DEV_AUTH_BYPASS is enabled while deployment environment '{}' \
             is production-like. Remove the bypass or set SDKWORK_DEPLOYMENT_ENV to development.",
            agents_deployment_environment_name()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bool_from_utils_for_dev_bypass() {
        let _guard = env_test_lock();
        let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();
        std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");
        assert!(agents_dev_auth_bypass_enabled());
        restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
    }

    #[test]
    fn dev_bypass_rejected_in_production_like_env() {
        let _guard = env_test_lock();
        let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();
        let previous_deploy = std::env::var("SDKWORK_DEPLOYMENT_ENV").ok();
        let previous_environment = std::env::var("ENVIRONMENT").ok();
        let previous_agents_env = std::env::var("SDKWORK_AGENTS_ENVIRONMENT").ok();
        let previous_profile = std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE").ok();
        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_CONFIG_PROFILE");
        std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");
        std::env::set_var("SDKWORK_DEPLOYMENT_ENV", "production");
        assert!(ensure_dev_auth_bypass_allowed().is_err());
        restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
        restore_optional_env("SDKWORK_DEPLOYMENT_ENV", previous_deploy);
        restore_optional_env("ENVIRONMENT", previous_environment);
        restore_optional_env("SDKWORK_AGENTS_ENVIRONMENT", previous_agents_env);
        restore_optional_env("SDKWORK_AGENTS_CONFIG_PROFILE", previous_profile);
    }

    #[test]
    fn production_like_profiles_disable_contract_fallback_and_dev_inline_auth() {
        let _guard = env_test_lock();
        let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();
        let previous_deploy = std::env::var("SDKWORK_DEPLOYMENT_ENV").ok();
        let previous_environment = std::env::var("ENVIRONMENT").ok();
        let previous_agents_env = std::env::var("SDKWORK_AGENTS_ENVIRONMENT").ok();
        let previous_profile = std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE").ok();

        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_CONFIG_PROFILE");
        std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");

        for profile in ["production", "prod", "staging", "stage", "live", "test"] {
            std::env::set_var("SDKWORK_DEPLOYMENT_ENV", profile);

            assert!(agents_is_production_like_environment());
            assert!(!agents_allow_contract_runtime_fallback());
            assert!(!agents_use_dev_inline_auth_resolver());
            assert!(ensure_dev_auth_bypass_allowed().is_err());
        }

        restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
        restore_optional_env("SDKWORK_DEPLOYMENT_ENV", previous_deploy);
        restore_optional_env("ENVIRONMENT", previous_environment);
        restore_optional_env("SDKWORK_AGENTS_ENVIRONMENT", previous_agents_env);
        restore_optional_env("SDKWORK_AGENTS_CONFIG_PROFILE", previous_profile);
    }

    #[test]
    fn development_profiles_allow_contract_fallback_and_explicit_dev_inline_auth() {
        let _guard = env_test_lock();
        let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();
        let previous_deploy = std::env::var("SDKWORK_DEPLOYMENT_ENV").ok();
        let previous_environment = std::env::var("ENVIRONMENT").ok();
        let previous_agents_env = std::env::var("SDKWORK_AGENTS_ENVIRONMENT").ok();
        let previous_profile = std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE").ok();

        std::env::remove_var("SDKWORK_DEPLOYMENT_ENV");
        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_CONFIG_PROFILE");
        std::env::remove_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS");

        assert_eq!(agents_deployment_environment_name(), "development");
        // Fail-closed: an unset environment is treated as production-like for
        // the security gates even though the canonical name stays development.
        assert!(!agents_allow_contract_runtime_fallback());
        assert!(!agents_use_dev_inline_auth_resolver());

        std::env::set_var("SDKWORK_DEPLOYMENT_ENV", "development");
        assert!(agents_allow_contract_runtime_fallback());
        assert!(!agents_use_dev_inline_auth_resolver());

        std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");

        assert!(!agents_is_production_like_environment());
        assert!(agents_allow_contract_runtime_fallback());
        assert!(agents_use_dev_inline_auth_resolver());
        assert!(ensure_dev_auth_bypass_allowed().is_ok());

        restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
        restore_optional_env("SDKWORK_DEPLOYMENT_ENV", previous_deploy);
        restore_optional_env("ENVIRONMENT", previous_environment);
        restore_optional_env("SDKWORK_AGENTS_ENVIRONMENT", previous_agents_env);
        restore_optional_env("SDKWORK_AGENTS_CONFIG_PROFILE", previous_profile);
    }

    fn restore_optional_env(key: &str, value: Option<String>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
