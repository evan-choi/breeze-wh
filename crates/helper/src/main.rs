mod automator;
mod dialog;
mod handlers;

fn main() -> anyhow::Result<()> {
    let config = breeze_common::config::load_config()?;
    let _guard = breeze_common::logging::init_logging("helper", &config);

    tracing::info!("Breeze Helper starting");
    automator::run(config)
}
