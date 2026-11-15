//! Blocking facade over `sdkwork-database-sqlx` PostgreSQL pools for sync repositories.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};
use sqlx::PgPool;
use std::future::Future;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub type PgRow = sqlx::postgres::PgRow;

pub fn map_sqlx_error(error: sqlx::Error) -> sdkwork_agent_kernel::KernelError {
    use sdkwork_agent_kernel::KernelError;

    match error {
        sqlx::Error::Protocol(message) => {
            if let Some(message) = message.strip_prefix("sdkwork-domain-validation:") {
                return KernelError::validation(message);
            }
            if let Some(message) = message.strip_prefix("sdkwork-domain-conflict:") {
                return KernelError::conflict(message);
            }
            if message.starts_with("sdkwork-domain-internal:") {
                return KernelError::Internal {
                    message: "database transaction invariant failed".to_string(),
                };
            }
            tracing::error!(target: "sdkwork.agents.postgres", "postgres protocol failure");
            KernelError::provider_error("postgres_protocol_error", "database operation failed")
        }
        sqlx::Error::Database(database) => {
            let sqlstate = database
                .code()
                .map(|code| code.into_owned())
                .unwrap_or_else(|| "unknown".to_string());
            let constraint = database.constraint().unwrap_or("unknown");
            let message = database.message().to_string();
            tracing::warn!(
                target: "sdkwork.agents.postgres",
                sqlstate,
                constraint,
                "postgres rejected a database operation: {message}"
            );
            match sqlstate.as_str() {
                "23505" => KernelError::conflict("database uniqueness conflict"),
                "23502" | "23503" | "23514" | "22001" => {
                    KernelError::validation("database constraint violation")
                }
                "40001" | "40P01" => KernelError::ProviderUnavailable {
                    provider_id: "postgres_transaction".to_string(),
                },
                "53300" => KernelError::resource_exhausted("database connection limit reached"),
                "57014" => KernelError::timeout("database operation timed out"),
                code if code.starts_with("08") => KernelError::ProviderUnavailable {
                    provider_id: "postgres".to_string(),
                },
                _ => KernelError::provider_error(
                    "postgres_database_error",
                    "database operation failed",
                ),
            }
        }
        sqlx::Error::PoolTimedOut => KernelError::timeout("database pool acquisition timed out"),
        sqlx::Error::PoolClosed | sqlx::Error::WorkerCrashed => KernelError::ProviderUnavailable {
            provider_id: "postgres".to_string(),
        },
        sqlx::Error::RowNotFound => KernelError::not_found("database row not found"),
        error => {
            tracing::error!(
                target: "sdkwork.agents.postgres",
                error_kind = ?error,
                "postgres adapter failure"
            );
            KernelError::provider_error("postgres_error", "database operation failed")
        }
    }
}

pub fn map_pool_error(error: PoolError) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::validation(format!("database pool: {error}"))
}

pub fn map_database_config_error(
    error: sdkwork_database_config::ConfigError,
) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::validation(format!("database config: {error}"))
}

#[derive(Debug, Clone)]
pub struct BlockingPostgresPool {
    pool: PgPool,
    runtime: ManuallyDrop<Arc<Runtime>>,
    database_pool: DatabasePool,
}

impl BlockingPostgresPool {
    pub fn from_database_pool(
        database_pool: DatabasePool,
        runtime: Arc<Runtime>,
    ) -> Result<Self, PoolError> {
        let pool = database_pool.as_postgres().cloned().ok_or_else(|| {
            PoolError::DatabaseConfig("expected postgres database pool".to_owned())
        })?;
        Ok(Self {
            pool,
            runtime: ManuallyDrop::new(runtime),
            database_pool,
        })
    }

    pub fn connect_from_config(
        config: DatabaseConfig,
    ) -> Result<Self, sdkwork_agent_kernel::KernelError> {
        let runtime = Arc::new(Runtime::new().map_err(|error| {
            sdkwork_agent_kernel::KernelError::provider_error(
                "postgres_runtime_error",
                format!("tokio runtime: {error}"),
            )
        })?);
        let database_pool = block_on_runtime(runtime.as_ref(), create_pool_from_config(config))
            .map_err(map_pool_error)?;
        Self::from_database_pool(database_pool, runtime).map_err(map_pool_error)
    }

    pub fn connect(connection_uri: &str) -> Result<Self, sdkwork_agent_kernel::KernelError> {
        let engine = DatabaseEngine::from_url(connection_uri).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::validation(format!(
                "unsupported postgres connection url: {connection_uri}"
            ))
        })?;
        if engine != DatabaseEngine::Postgres {
            return Err(sdkwork_agent_kernel::KernelError::validation(format!(
                "expected postgres engine for url: {connection_uri}"
            )));
        }
        Self::connect_from_config(DatabaseConfig {
            engine,
            url: connection_uri.to_owned(),
            ..DatabaseConfig::default()
        })
    }

    pub fn connect_from_sdkwork_env(
        service_name: &str,
    ) -> Result<Self, sdkwork_agent_kernel::KernelError> {
        let config = DatabaseConfig::from_env(service_name).map_err(map_database_config_error)?;
        match config.engine {
            DatabaseEngine::Postgres => Self::connect_from_config(config),
            other => Err(sdkwork_agent_kernel::KernelError::validation(format!(
                "service {service_name} resolved database engine {other:?}, expected Postgres"
            ))),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn database_pool(&self) -> &DatabasePool {
        &self.database_pool
    }

    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        block_on_runtime(self.runtime.as_ref(), future)
    }

    pub fn run<F, T>(&self, future: F) -> Result<T, sqlx::Error>
    where
        F: Future<Output = Result<T, sqlx::Error>>,
    {
        block_on_runtime(self.runtime.as_ref(), future)
    }

    pub fn run_kernel<F, T>(&self, future: F) -> Result<T, sdkwork_agent_kernel::KernelError>
    where
        F: Future<Output = Result<T, sqlx::Error>>,
    {
        self.run(future).map_err(map_sqlx_error)
    }

    /// Returns the number of connections currently in the pool (both idle and active).
    pub fn pool_size(&self) -> u32 {
        self.pool.size()
    }

    /// Returns the number of idle connections in the pool.
    pub fn pool_idle_connections(&self) -> u32 {
        self.pool.num_idle() as u32
    }

    /// Returns pool utilization as a ratio (0.0 to 1.0).
    /// Calculated as (total - idle) / total when total > 0, else 0.0.
    pub fn pool_utilization(&self) -> f64 {
        let total = self.pool.size();
        if total == 0 {
            return 0.0;
        }
        let idle = self.pool.num_idle() as u32;
        let active = total.saturating_sub(idle);
        active as f64 / total as f64
    }

    /// Returns pool health metrics as a tuple: (total_connections, idle_connections, active_connections, utilization_ratio).
    pub fn pool_metrics(&self) -> PoolMetrics {
        let total = self.pool.size();
        let idle = self.pool.num_idle() as u32;
        let active = total.saturating_sub(idle);
        let utilization = if total > 0 {
            active as f64 / total as f64
        } else {
            0.0
        };
        PoolMetrics {
            total_connections: total,
            idle_connections: idle,
            active_connections: active,
            utilization,
        }
    }
}

/// Drives `future` on the pool's private runtime from any calling context.
///
/// The pool's connections are registered with the private runtime's reactor,
/// so the future must be polled on that runtime; polling it on the caller's
/// reactor (for example through `Handle::block_on`) stalls the socket
/// readiness that only the private driver can deliver.
///
/// * Outside an async runtime (startup, worker threads, CLI) the private
///   runtime is entered directly.
/// * Inside an async host the calling thread is already driving another
///   runtime, so entering the private one would panic with
///   `Cannot start a runtime from within a runtime`. `block_in_place` hands the
///   current worker core to a sibling thread and clears the entered-runtime
///   marker for the whole closure, which makes the private runtime enterable
///   here. The embedding runtime must be multi-threaded (`#[tokio::main]`,
///   `Runtime::new()`); blocking a `current_thread` runtime is impossible, and
///   tokio reports that with its own panic.
fn block_on_runtime<F, T>(runtime: &Runtime, future: F) -> T
where
    F: Future<Output = T>,
{
    if tokio::runtime::Handle::try_current().is_err() {
        return runtime.block_on(future);
    }

    tokio::task::block_in_place(|| runtime.block_on(future))
}

impl Drop for BlockingPostgresPool {
    fn drop(&mut self) {
        // `Runtime` must not be dropped from inside another Tokio runtime (for example
        // when standalone gateway tears down embedded agents routes after bind/serve failure).
        let runtime = unsafe { ManuallyDrop::take(&mut self.runtime) };
        if Arc::strong_count(&runtime) == 1 && tokio::runtime::Handle::try_current().is_ok() {
            if std::thread::spawn(move || drop(runtime)).join().is_err() {
                tracing::warn!(
                    target: "sdkwork.agents.postgres_sync_pool",
                    "failed to join runtime shutdown thread"
                );
            }
            return;
        }

        drop(runtime);
    }
}

/// Connection pool health metrics for monitoring and observability.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoolMetrics {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub utilization: f64,
}

#[macro_export]
macro_rules! pg_execute {
    ($pool:expr, $sql:expr $(, $param:expr)* $(,)?) => {{
        let pg = $pool.pool().clone();
        $pool.run_kernel(async {
            sqlx::query::<sqlx::Postgres>($sql) $(.bind(&$param))* .execute(&pg).await.map(|result| result.rows_affected())
        })
    }};
}

#[macro_export]
macro_rules! pg_query {
    ($pool:expr, $sql:expr $(, $param:expr)* $(,)?) => {{
        let pg = $pool.pool().clone();
        $pool.run_kernel(async {
            sqlx::query::<sqlx::Postgres>($sql) $(.bind(&$param))* .fetch_all(&pg).await
        })
    }};
}

#[macro_export]
macro_rules! pg_query_optional {
    ($pool:expr, $sql:expr $(, $param:expr)* $(,)?) => {{
        let pg = $pool.pool().clone();
        $pool.run_kernel(async {
            sqlx::query::<sqlx::Postgres>($sql) $(.bind(&$param))* .fetch_optional(&pg).await
        })
    }};
}

#[cfg(test)]
mod tests {
    use super::{block_on_runtime, map_sqlx_error, BlockingPostgresPool};
    use sdkwork_agent_kernel::KernelErrorKind;
    use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
    use sdkwork_database_sqlx::{DatabasePool, PoolContext};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    /// A pool whose connection is never opened. The assertions below exercise
    /// the runtime bridging only, so no PostgreSQL server is required.
    fn lazy_test_pool() -> BlockingPostgresPool {
        let config = DatabaseConfig {
            engine: DatabaseEngine::Postgres,
            url: "postgres://sdkwork:sdkwork@127.0.0.1:1/sdkwork_agents_test".to_owned(),
            ..DatabaseConfig::default()
        };
        let pg_pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy(&config.url)
            .expect("lazy postgres pool builds");
        BlockingPostgresPool::from_database_pool(
            DatabasePool::Postgres(pg_pool, PoolContext { config }),
            Arc::new(Runtime::new().expect("private runtime builds")),
        )
        .expect("blocking pool wraps a postgres database pool")
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blocking_adapter_can_drive_a_future_inside_an_async_host() {
        let runtime = Runtime::new().expect("private runtime builds");
        assert_eq!(block_on_runtime(&runtime, async { 42 }), 42);
        tokio::task::block_in_place(|| drop(runtime));
    }

    /// Regression: `run`/`run_kernel` (and therefore every `pg_query!`,
    /// `pg_query_optional!` and `pg_execute!` call site) used to enter the
    /// private runtime directly, which panics on the request path with
    /// `Cannot start a runtime from within a runtime` and surfaces as a 503
    /// "embedded dependency returned a server error" envelope.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_drives_the_private_runtime_from_inside_an_async_host() {
        let pool = lazy_test_pool();
        let value = pool.run(async { Ok::<u32, sqlx::Error>(7) });
        assert_eq!(value.expect("blocking run completes"), 7);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn block_on_drives_the_private_runtime_from_inside_an_async_host() {
        let pool = lazy_test_pool();
        assert_eq!(pool.block_on(async { 13 }), 13);
    }

    #[test]
    fn transaction_domain_errors_keep_their_stable_kind() {
        let validation = map_sqlx_error(sqlx::Error::Protocol(
            "sdkwork-domain-validation:invalid session".to_string(),
        ));
        assert_eq!(validation.kind(), KernelErrorKind::ValidationError);
        assert_eq!(validation.safe_message(), "invalid session");

        let conflict = map_sqlx_error(sqlx::Error::Protocol(
            "sdkwork-domain-conflict:turn completion conflict".to_string(),
        ));
        assert_eq!(conflict.kind(), KernelErrorKind::Conflict);
        assert_eq!(conflict.safe_message(), "turn completion conflict");
    }

    #[test]
    fn pool_failures_do_not_expose_sqlx_details() {
        let error = map_sqlx_error(sqlx::Error::PoolTimedOut);
        assert_eq!(error.kind(), KernelErrorKind::Timeout);
        assert_eq!(error.safe_message(), "database pool acquisition timed out");
    }
}
