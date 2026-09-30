use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{bail, Context};
use sdkwork_intelligence_agents_service::{
    DEFAULT_CLAIM_BATCH_SIZE, DEFAULT_MATERIALIZE_BATCH_SIZE, DEFAULT_RUN_LEASE_SECONDS,
    DEFAULT_TENANT_CONCURRENT_RUNS, MAX_CLAIM_BATCH_SIZE, MAX_MATERIALIZE_BATCH_SIZE,
    MAX_RUN_LEASE_SECONDS, MAX_TENANT_CONCURRENT_RUNS,
};

pub const ENV_TASK_WORKER_ID: &str = "SDKWORK_AGENTS_TASK_WORKER_ID";
pub const ENV_TASK_WORKER_BIND: &str = "SDKWORK_AGENTS_TASK_WORKER_BIND";
const ENV_MATERIALIZE_INTERVAL_MILLIS: &str = "SDKWORK_AGENTS_TASK_MATERIALIZE_INTERVAL_MILLIS";
const ENV_CLAIM_INTERVAL_MILLIS: &str = "SDKWORK_AGENTS_TASK_CLAIM_INTERVAL_MILLIS";
const ENV_RECOVERY_INTERVAL_SECONDS: &str = "SDKWORK_AGENTS_TASK_RECOVERY_INTERVAL_SECONDS";
const ENV_METRICS_SNAPSHOT_INTERVAL_SECONDS: &str =
    "SDKWORK_AGENTS_TASK_METRICS_SNAPSHOT_INTERVAL_SECONDS";
const ENV_RECONCILIATION_INTERVAL_SECONDS: &str =
    "SDKWORK_AGENTS_TASK_RECONCILIATION_INTERVAL_SECONDS";
const ENV_RECONCILIATION_MIN_AGE_SECONDS: &str =
    "SDKWORK_AGENTS_TASK_RECONCILIATION_MIN_AGE_SECONDS";
pub const ENV_OUTBOX_DISPATCH_INTERVAL_MILLIS: &str =
    "SDKWORK_AGENTS_OUTBOX_DISPATCH_INTERVAL_MILLIS";
const ENV_MATERIALIZE_BATCH_SIZE: &str = "SDKWORK_AGENTS_TASK_MATERIALIZE_BATCH_SIZE";
const ENV_CLAIM_BATCH_SIZE: &str = "SDKWORK_AGENTS_TASK_CLAIM_BATCH_SIZE";
const ENV_LEASE_SECONDS: &str = "SDKWORK_AGENTS_TASK_LEASE_SECONDS";
const ENV_HEARTBEAT_INTERVAL_SECONDS: &str = "SDKWORK_AGENTS_TASK_HEARTBEAT_INTERVAL_SECONDS";
const ENV_MAX_CONCURRENCY: &str = "SDKWORK_AGENTS_TASK_MAX_CONCURRENCY";
const ENV_TENANT_MAX_CONCURRENCY: &str = "SDKWORK_AGENTS_TASK_TENANT_MAX_CONCURRENCY";
const ENV_DRAIN_TIMEOUT_SECONDS: &str = "SDKWORK_AGENTS_TASK_DRAIN_TIMEOUT_SECONDS";
const ENV_RECOVERY_BATCH_SIZE: &str = "SDKWORK_AGENTS_TASK_RECOVERY_BATCH_SIZE";
const ENV_RECONCILIATION_BATCH_SIZE: &str = "SDKWORK_AGENTS_TASK_RECONCILIATION_BATCH_SIZE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerWorkerConfig {
    pub worker_id: String,
    pub bind_address: SocketAddr,
    pub materialize_interval: Duration,
    pub claim_interval: Duration,
    pub recovery_interval: Duration,
    pub metrics_snapshot_interval: Duration,
    pub reconciliation_interval: Duration,
    pub outbox_dispatch_interval: Duration,
    pub reconciliation_min_age: Duration,
    pub heartbeat_interval: Duration,
    pub drain_timeout: Duration,
    pub materialize_batch_size: usize,
    pub claim_batch_size: usize,
    pub recovery_batch_size: usize,
    pub reconciliation_batch_size: usize,
    pub lease_seconds: u32,
    pub max_concurrency: usize,
    pub tenant_max_concurrency: usize,
}

impl SchedulerWorkerConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let environment = std::env::var("SDKWORK_AGENTS_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());
        let worker_id = match std::env::var(ENV_TASK_WORKER_ID) {
            Ok(value) => value,
            Err(_) if matches!(environment.as_str(), "staging" | "production") => {
                bail!("{ENV_TASK_WORKER_ID} is required in {environment}")
            }
            Err(_) => default_worker_id(),
        };
        validate_worker_id(&worker_id)?;

        let bind_address = std::env::var(ENV_TASK_WORKER_BIND)
            .unwrap_or_else(|_| "127.0.0.1:8096".to_string())
            .parse()
            .with_context(|| format!("{ENV_TASK_WORKER_BIND} must be a socket address"))?;
        let lease_seconds = env_number(
            ENV_LEASE_SECONDS,
            DEFAULT_RUN_LEASE_SECONDS,
            5,
            MAX_RUN_LEASE_SECONDS,
        )?;
        let heartbeat_seconds = env_number(
            ENV_HEARTBEAT_INTERVAL_SECONDS,
            (lease_seconds / 3).max(1),
            1,
            MAX_RUN_LEASE_SECONDS,
        )?;
        if heartbeat_seconds >= lease_seconds {
            bail!("{ENV_HEARTBEAT_INTERVAL_SECONDS} must be less than {ENV_LEASE_SECONDS}");
        }

        Ok(Self {
            worker_id,
            bind_address,
            materialize_interval: Duration::from_millis(env_number(
                ENV_MATERIALIZE_INTERVAL_MILLIS,
                1_000_u64,
                100,
                60_000,
            )?),
            claim_interval: Duration::from_millis(env_number(
                ENV_CLAIM_INTERVAL_MILLIS,
                250_u64,
                25,
                60_000,
            )?),
            recovery_interval: Duration::from_secs(env_number(
                ENV_RECOVERY_INTERVAL_SECONDS,
                15_u64,
                1,
                3_600,
            )?),
            metrics_snapshot_interval: Duration::from_secs(env_number(
                ENV_METRICS_SNAPSHOT_INTERVAL_SECONDS,
                60_u64,
                10,
                3_600,
            )?),
            reconciliation_interval: Duration::from_secs(env_number(
                ENV_RECONCILIATION_INTERVAL_SECONDS,
                30_u64,
                1,
                3_600,
            )?),
            outbox_dispatch_interval: Duration::from_millis(env_number(
                ENV_OUTBOX_DISPATCH_INTERVAL_MILLIS,
                1_000_u64,
                100,
                60_000,
            )?),
            reconciliation_min_age: Duration::from_secs(env_number(
                ENV_RECONCILIATION_MIN_AGE_SECONDS,
                300_u64,
                30,
                86_400,
            )?),
            heartbeat_interval: Duration::from_secs(u64::from(heartbeat_seconds)),
            drain_timeout: Duration::from_secs(env_number(
                ENV_DRAIN_TIMEOUT_SECONDS,
                150_u64,
                1,
                3_600,
            )?),
            materialize_batch_size: env_number(
                ENV_MATERIALIZE_BATCH_SIZE,
                DEFAULT_MATERIALIZE_BATCH_SIZE,
                1,
                MAX_MATERIALIZE_BATCH_SIZE,
            )?,
            claim_batch_size: env_number(
                ENV_CLAIM_BATCH_SIZE,
                DEFAULT_CLAIM_BATCH_SIZE,
                1,
                MAX_CLAIM_BATCH_SIZE,
            )?,
            recovery_batch_size: env_number(
                ENV_RECOVERY_BATCH_SIZE,
                DEFAULT_MATERIALIZE_BATCH_SIZE,
                1,
                MAX_MATERIALIZE_BATCH_SIZE,
            )?,
            reconciliation_batch_size: env_number(
                ENV_RECONCILIATION_BATCH_SIZE,
                DEFAULT_MATERIALIZE_BATCH_SIZE,
                1,
                MAX_MATERIALIZE_BATCH_SIZE,
            )?,
            lease_seconds,
            max_concurrency: env_number(
                ENV_MAX_CONCURRENCY,
                DEFAULT_CLAIM_BATCH_SIZE,
                1,
                MAX_CLAIM_BATCH_SIZE,
            )?,
            tenant_max_concurrency: env_number(
                ENV_TENANT_MAX_CONCURRENCY,
                DEFAULT_TENANT_CONCURRENT_RUNS,
                1,
                MAX_TENANT_CONCURRENT_RUNS,
            )?,
        })
    }
}

fn env_number<T>(key: &str, default: T, minimum: T, maximum: T) -> anyhow::Result<T>
where
    T: Copy + Ord + std::str::FromStr + std::fmt::Display,
    T::Err: std::fmt::Display,
{
    let value = match std::env::var(key) {
        Ok(raw) => raw
            .parse::<T>()
            .map_err(|error| anyhow::anyhow!("{key} must be an integer: {error}"))?,
        Err(_) => default,
    };
    if value < minimum || value > maximum {
        bail!("{key} must be between {minimum} and {maximum}");
    }
    Ok(value)
}

fn validate_worker_id(value: &str) -> anyhow::Result<()> {
    if value.trim() != value || value.is_empty() || value.len() > 128 {
        bail!("{ENV_TASK_WORKER_ID} must contain 1 to 128 trimmed bytes");
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
    {
        bail!("{ENV_TASK_WORKER_ID} contains unsupported characters");
    }
    Ok(())
}

fn default_worker_id() -> String {
    let host = std::env::var("POD_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "localhost".to_string());
    let host = host
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .take(96)
        .collect::<String>();
    format!("agents-task-worker:{host}:{}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_id_rejects_whitespace_and_unbounded_values() {
        assert!(validate_worker_id("worker:one").is_ok());
        assert!(validate_worker_id(" worker").is_err());
        assert!(validate_worker_id(&"x".repeat(129)).is_err());
        assert!(validate_worker_id("worker/one").is_err());
    }
}
