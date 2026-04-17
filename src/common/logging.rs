use super::config::BreezeConfig;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;

/// Initialize tracing with rolling file output.
/// Returns a guard that must be kept alive for the logger lifetime.
pub fn init_logging(log_prefix: &str, config: &BreezeConfig) -> WorkerGuard {
    let log_dir = super::constants::log_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, log_prefix);
    // Default buffer is 128k lines (~4 MB allocation). Breeze logs a few lines
    // per minute, so a tiny buffer is plenty and saves ~4 MB of resident memory.
    let (non_blocking, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .buffered_lines_limit(512)
        .finish(file_appender);

    let level = config.log_level.parse::<Level>().unwrap_or(Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::from_level(level))
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    guard
}
