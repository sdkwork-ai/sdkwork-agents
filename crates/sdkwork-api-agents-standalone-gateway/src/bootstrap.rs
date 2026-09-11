use anyhow::Context;
use sdkwork_web_bootstrap::{ApiModuleRegistry, infra_public_path_prefixes};
use axum::Router;
use sdkwork_iam_web_adapter::{
    build_web_framework_builder, iam_web_request_context_resolver_from_env,
};

pub async fn build_router() -> anyhow::Result<Router> {
    let assembly = sdkwork_api_agents_assembly::assemble_api_router()
        .await
        .context("compose agents gateway assembly router")?;
    let framework = build_web_framework_builder(
        iam_web_request_context_resolver_from_env().await,
        assembly.route_manifest.clone(),
        infra_public_path_prefixes(),
    );
    let mut module_registry = ApiModuleRegistry::new();
    module_registry.add_modules(vec![assembly]);
    Ok(
        module_registry
    .try_compose("SDKWork Agents API")
            .map_err(anyhow::Error::msg)?
            .into_hosted(framework)
            .router,
    )
}

pub async fn run_agents_app_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_api_agents_assembly::bootstrap_application_database_from_env()
        .await
        .map_err(|error| format!("{error:#}"))?;
    tracing::info!("sdkwork-agents application database migration completed");
    Ok(())
}

pub async fn run_kernel_database_migrate_only() -> Result<(), String> {
    sdkwork_api_agents_assembly::bootstrap_kernel_database_from_env()
        .await
        .map_err(|error| format!("{error:#}"))?;
    tracing::info!("sdkwork-agents kernel runtime persistence opened for migration");
    Ok(())
}
