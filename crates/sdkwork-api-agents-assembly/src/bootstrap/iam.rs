use std::path::PathBuf;

use axum::Router;
use sdkwork_iam_embedded_application_bootstrap::{
    ensure_tenant_application_from_app_root, resolve_bootstrap_environment,
    EmbeddedApplicationBootstrapOptions,
};

#[derive(Clone, Debug)]
struct AgentsIamBootstrapConfig {
    app_root: PathBuf,
    environment: String,
}

fn resolve_agents_iam_bootstrap_config() -> AgentsIamBootstrapConfig {
    AgentsIamBootstrapConfig {
        app_root: resolve_agents_deployment_app_root(),
        environment: resolve_bootstrap_environment(),
    }
}

fn resolve_agents_deployment_app_root() -> PathBuf {
    resolve_agents_deployment_app_root_with(|key| std::env::var(key).ok())
}

fn resolve_agents_deployment_app_root_with(
    mut read_environment: impl FnMut(&str) -> Option<String>,
) -> PathBuf {
    for key in ["SDKWORK_APP_ROOT", "SDKWORK_AGENTS_APP_ROOT"] {
        if let Some(value) = read_environment(key) {
            let path = value.trim();
            if !path.is_empty() {
                return PathBuf::from(path);
            }
        }
    }
    resolve_agents_app_root()
}

async fn ensure_agents_tenant_application_bootstrap(
    bootstrap: &AgentsIamBootstrapConfig,
) -> Result<(), String> {
    ensure_tenant_application_from_app_root(
        bootstrap.app_root.as_path(),
        &EmbeddedApplicationBootstrapOptions {
            environment: bootstrap.environment.clone(),
            ..EmbeddedApplicationBootstrapOptions::default()
        },
        None,
        &[],
    )
    .await
}

pub(super) async fn wire_iam_app_router() -> Result<Router, String> {
    let bootstrap = resolve_agents_iam_bootstrap_config();
    sdkwork_iam_database_host::bootstrap_iam_database_from_env()
        .await
        .map_err(|error| format!("failed to bootstrap IAM database lifecycle: {error}"))?;
    ensure_agents_tenant_application_bootstrap(&bootstrap)
        .await
        .map_err(|error| format!("failed to ensure Agents IAM tenant application: {error}"))?;
    sdkwork_routes_iam_app_api::build_sdkwork_iam_app_api_router()
        .await
        .map_err(|error| format!("failed to build embedded IAM app router: {error}"))
}

fn resolve_agents_app_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

#[cfg(test)] // WORKSPACE-PATH:allow-fixture-block: this module is the file's #[cfg(test)] unit-test fixture data
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn agents_app_root_resolves_to_repository_root() {
        let root = resolve_agents_app_root();
        assert!(root.join("sdkwork.app.config.json").is_file());
    }

    #[test]
    fn agents_deployment_root_prefers_generic_then_product_specific_root() {
        let environment = BTreeMap::from([
            ("SDKWORK_APP_ROOT", "D:/deployment-root"),
            ("SDKWORK_AGENTS_APP_ROOT", "D:/agents-root"),
        ]);
        let selected = resolve_agents_deployment_app_root_with(|key| {
            environment.get(key).map(|value| (*value).to_owned())
        });
        assert_eq!(selected, PathBuf::from("D:/deployment-root"));

        let product_only = BTreeMap::from([("SDKWORK_AGENTS_APP_ROOT", "D:/agents-root")]);
        let selected = resolve_agents_deployment_app_root_with(|key| {
            product_only.get(key).map(|value| (*value).to_owned())
        });
        assert_eq!(selected, PathBuf::from("D:/agents-root"));
    }

    #[test]
    fn agents_deployment_root_falls_back_to_repository_root() {
        assert_eq!(
            resolve_agents_deployment_app_root_with(|_| None),
            resolve_agents_app_root()
        );
    }
}
