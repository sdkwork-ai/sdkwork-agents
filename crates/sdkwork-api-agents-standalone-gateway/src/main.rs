use anyhow::Context;
use sdkwork_api_agents_standalone_gateway::{
    build_router, init_tracing, log_access_urls, run_agents_app_database_migrate_only,
    run_kernel_database_migrate_only, shutdown_signal, signal_agents_background_shutdown,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    match std::env::args().nth(1).as_deref() {
        Some("db-migrate") => {
            run_agents_app_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            run_kernel_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            return Ok(());
        }
        Some("db-migrate:app") => {
            run_agents_app_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            return Ok(());
        }
        Some("db-migrate:kernel") => {
            run_kernel_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            return Ok(());
        }
        _ => {}
    }

    let bind_address = std::env::var("SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND")
        .or_else(|_| std::env::var("SDKWORK_AGENT_SERVER_BIND"))
        .unwrap_or_else(|_| "127.0.0.1:8095".to_owned());

    let app = build_router()
        .await
        .context("sdkwork-api-agents-standalone-gateway bootstrap failed")?;

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("bind sdkwork-api-agents-standalone-gateway on {bind_address}"))?;

    let local_address = listener
        .local_addr()
        .context("resolve sdkwork-api-agents-standalone-gateway listener address")?;
    log_access_urls(local_address);
    // Graceful drain with a hard deadline: in-flight SSE turns can run for
    // up to the turn execution budget, so draining must never wait for the
    // full budget — past the deadline the process exits and the durable turn
    // leases (plus the reconciler on the next replica) recover the rest.
    let drain_timeout = std::time::Duration::from_secs(
        std::env::var("SDKWORK_AGENTS_SHUTDOWN_DRAIN_SECONDS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| (1..=3600).contains(value))
            .unwrap_or(150),
    );
    let serving = async {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_signal().await;
                // Stop the detached reconciliation worker before the drain so
                // it stops competing with in-flight turns for blocking
                // threads.
                signal_agents_background_shutdown();
            })
            .await
    };
    match tokio::time::timeout(drain_timeout, serving).await {
        Ok(result) => result.context("serve sdkwork-api-agents-standalone-gateway")?,
        Err(_) => {
            tracing::warn!(
                drain_timeout_secs = drain_timeout.as_secs(),
                "graceful drain deadline reached; remaining in-flight turns are recovered through their leases"
            );
        }
    }
    Ok(())
}
