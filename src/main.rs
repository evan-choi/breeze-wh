mod cli;
mod common;
mod helper;
mod service;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(|s| s.as_str()) {
        // Internal: called by SCM to run as Windows Service
        Some("service") => service::runner::run(),

        // Internal: called by service to run UI Automation helper in user session
        Some("helper") => {
            let config = common::config::load_config()?;
            let _guard = common::logging::init_logging("helper", &config);
            tracing::info!("Breeze helper starting");
            helper::automator::run(config)
        }

        // CLI commands
        _ => cli::run(&args),
    }
}
