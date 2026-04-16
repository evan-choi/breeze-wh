use super::config::BreezeConfig;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

/// Initialize tracing with rolling file output.
/// Returns a guard that must be kept alive for the logger lifetime.
pub fn init_logging(log_prefix: &str, config: &BreezeConfig) -> WorkerGuard {
    let log_dir = super::constants::log_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, log_prefix);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    guard
}
