use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Environment switch between human-readable and JSON log lines. Cluster
/// deployments set `SDKWORK_LOG_FORMAT=json` so the collector receives
/// structured records; local development defaults to readable text.
pub const ENV_LOG_FORMAT: &str = "SDKWORK_LOG_FORMAT";

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let json_format = std::env::var(ENV_LOG_FORMAT)
        .map(|value| value.trim().eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    if json_format {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}
