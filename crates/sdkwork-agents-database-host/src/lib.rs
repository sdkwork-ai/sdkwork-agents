mod drift_gate;

use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};

use crate::drift_gate::ensure_agents_schema_current;

pub struct AgentsDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl AgentsDatabaseHost {
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }
}

pub async fn bootstrap_agents_database(pool: DatabasePool) -> Result<AgentsDatabaseHost, String> {
    let app_root = resolve_app_root();
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(&app_root)
            .map_err(|error| format!("load agents database module failed: {error}"))?,
    );
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read agents database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("AGENTS", &manifest);
    let orchestrator =
        LifecycleOrchestrator::new(pool.clone(), module.clone()).with_applied_by("sdkwork-agents");

    orchestrator
        .init()
        .await
        .map_err(|error| format!("agents database init failed: {error}"))?;

    if options.auto_migrate {
        orchestrator
            .migrate()
            .await
            .map_err(|error| format!("agents database migrate failed: {error}"))?;
    }

    ensure_agents_schema_current(&pool, module.clone()).await?;

    Ok(AgentsDatabaseHost { pool, module })
}

/// Interval for the periodic schema-drift re-check, from
/// `SDKWORK_AGENTS_SCHEMA_DRIFT_CHECK_SECONDS` (manifest declares 60s; the
/// env override exists for slow-migrating environments). Zero disables the
/// monitor.
pub const ENV_SCHEMA_DRIFT_CHECK_SECONDS: &str = "SDKWORK_AGENTS_SCHEMA_DRIFT_CHECK_SECONDS";
pub const DEFAULT_SCHEMA_DRIFT_CHECK_SECONDS: u64 = 60;

/// Spawns the periodic schema-drift re-check (manifest
/// `driftCheckIntervalSec`). The startup drift gate only catches drift at
/// boot; a later out-of-band DDL would otherwise serve traffic silently.
/// The returned handle's task ends when the process stops: the monitor owns
/// no connection of its own (each check borrows the pool) and stopping
/// without an explicit signal is safe because the next deployment's startup
/// gate re-verifies the schema regardless.
/// Spawns the periodic drift re-check against a dedicated small pool
/// (`SDKWORK_AGENTS_SCHEMA_DRIFT_CHECK_SECONDS`, 0 disables).
///
/// HTTP entrypoints own no `AgentsDatabaseHost` handle (the kernel bridge
/// consumes it during state bootstrap), so the standalone monitor provisions
/// its own two-connection pool. That is one idle connection of overhead in
/// exchange for catching out-of-band DDL without threading lifecycle types
/// through the state constructors.
pub fn spawn_schema_drift_monitor_standalone() -> Option<tokio::task::JoinHandle<()>> {
    let interval_seconds = std::env::var(ENV_SCHEMA_DRIFT_CHECK_SECONDS)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_SCHEMA_DRIFT_CHECK_SECONDS);
    if interval_seconds == 0 {
        return None;
    }
    Some(tokio::spawn(async move {
        let bootstrap = async {
            let mut config = DatabaseConfig::from_env("AGENTS")
                .map_err(|error| format!("read agents database config failed: {error}"))?;
            // The monitor only reads information_schema: two connections bound
            // its footprint regardless of the shared pool sizing.
            config.max_connections = config.max_connections.min(2);
            let pool = create_pool_from_config(config)
                .await
                .map_err(|error| format!("create agents drift monitor pool failed: {error}"))?;
            let app_root = resolve_app_root();
            let module = Arc::new(
                DefaultDatabaseModule::from_app_root(&app_root)
                    .map_err(|error| format!("load agents database module failed: {error}"))?,
            );
            Ok::<_, String>((pool, module))
        };
        let (pool, module) = match bootstrap.await {
            Ok(pair) => pair,
            Err(error) => {
                tracing::error!(
                    target: "sdkwork.agents.database.drift",
                    error = %error,
                    "schema drift monitor could not start; only the startup gate remains active"
                );
                return;
            }
        };
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(interval_seconds));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        ticker.tick().await; // first tick fires immediately; the startup gate already ran
        loop {
            ticker.tick().await;
            if let Err(error) =
                crate::drift_gate::ensure_agents_schema_current(&pool, module.clone()).await
            {
                // Fail loud, not fatal: a drifted schema is an operations
                // incident, but killing every replica mid-incident amplifies
                // it. The next startup gate remains the hard gate.
                tracing::error!(
                    target: "sdkwork.agents.database.drift",
                    error = %error,
                    "periodic schema drift check failed"
                );
            }
        }
    }))
}

pub fn spawn_schema_drift_monitor(
    host: &AgentsDatabaseHost,
) -> Option<tokio::task::JoinHandle<()>> {
    let interval_seconds = std::env::var(ENV_SCHEMA_DRIFT_CHECK_SECONDS)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_SCHEMA_DRIFT_CHECK_SECONDS);
    if interval_seconds == 0 {
        return None;
    }
    let pool = host.pool.clone();
    let module = host.module.clone();
    Some(tokio::spawn(async move {
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(interval_seconds));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        ticker.tick().await; // first tick fires immediately; the startup gate already ran
        loop {
            ticker.tick().await;
            if let Err(error) = crate::drift_gate::ensure_agents_schema_current(&pool, module.clone()).await {
                // Fail loud, not fatal: a drifted schema is an operations
                // incident, but killing every replica mid-incident amplifies
                // it. The next startup gate remains the hard gate.
                tracing::error!(
                    target: "sdkwork.agents.database.drift",
                    error = %error,
                    "periodic schema drift check failed"
                );
            }
        }
    }))
}

pub async fn bootstrap_agents_database_from_env() -> Result<AgentsDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("AGENTS")
        .map_err(|error| format!("read agents database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create agents database pool failed: {error}"))?;
    bootstrap_agents_database(pool).await
}

fn resolve_app_root() -> PathBuf {
    std::env::var("SDKWORK_AGENTS_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        })
}
