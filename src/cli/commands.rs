use crate::common::{
    config::BreezeConfig,
    constants::{SERVICE_DISPLAY_NAME, SERVICE_NAME, config_path, data_dir},
};
use anyhow::Context;
use windows_service::{
    service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState,
        ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};

/// Quick check if the Breeze service is registered (no elevation needed).
pub fn check_service_exists() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("Failed to open service manager")?;

    manager
        .open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        .context("Breeze service is not installed. Run 'breeze install' first.")?;

    Ok(())
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("install") => cmd_install(),
        Some("uninstall") => cmd_uninstall(),
        Some("start") => cmd_start(),
        Some("stop") => cmd_stop(),
        Some("status") => cmd_status(),
        _ => {
            eprintln!("Breeze - Auto Windows Hello");
            eprintln!();
            eprintln!("Usage: breeze <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  install    Install the Breeze service");
            eprintln!("  uninstall  Uninstall the Breeze service");
            eprintln!("  start      Start the Breeze service");
            eprintln!("  stop       Stop the Breeze service");
            eprintln!("  status     Show the Breeze service status");
            eprintln!();
            eprintln!("Internal (used by the service):");
            eprintln!("  service    Run as Windows Service");
            eprintln!("  helper     Run the UI Automation helper");
            std::process::exit(1);
        }
    }
}

fn cmd_install() -> anyhow::Result<()> {
    let service_exe =
        std::env::current_exe().context("Failed to resolve current executable path")?;

    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )
    .context("Failed to open service manager (try running as Administrator)")?;

    let info = ServiceInfo {
        name: SERVICE_NAME.into(),
        display_name: SERVICE_DISPLAY_NAME.into(),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_exe,
        launch_arguments: vec!["service".into()],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = manager
        .create_service(&info, ServiceAccess::CHANGE_CONFIG)
        .context("Failed to create service")?;

    service
        .set_description("Automatically confirms Windows Hello face recognition dialogs")
        .context("Failed to set service description")?;

    let dir = data_dir();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create data directory: {}", dir.display()))?;

    let cfg_path = config_path();
    if !cfg_path.exists() {
        let default_cfg = toml::to_string_pretty(&BreezeConfig::default())
            .context("Failed to serialize default config")?;
        std::fs::write(&cfg_path, default_cfg)
            .with_context(|| format!("Failed to write config: {}", cfg_path.display()))?;
        println!("Written default config to {}", cfg_path.display());
    }

    println!("Service '{}' installed successfully.", SERVICE_NAME);
    println!("Run 'breeze start' to start the service.");
    Ok(())
}

fn cmd_uninstall() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("Failed to open service manager (try running as Administrator)")?;

    let service = manager
        .open_service(
            SERVICE_NAME,
            ServiceAccess::STOP | ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS,
        )
        .context("Failed to open service (is it installed?)")?;

    let status = service
        .query_status()
        .context("Failed to query service status")?;

    if status.current_state != ServiceState::Stopped
        && let Err(e) = service.stop()
    {
        eprintln!("Warning: could not stop service before deletion: {e}");
    }

    service.delete().context("Failed to delete service")?;

    println!("Service '{}' uninstalled successfully.", SERVICE_NAME);
    Ok(())
}

fn cmd_start() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("Failed to open service manager (try running as Administrator)")?;

    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::START)
        .context("Failed to open service (is it installed?)")?;

    service
        .start(&["service"])
        .context("Failed to start service")?;

    println!("Service '{}' started.", SERVICE_NAME);
    Ok(())
}

fn cmd_stop() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("Failed to open service manager (try running as Administrator)")?;

    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::STOP)
        .context("Failed to open service (is it installed?)")?;

    service.stop().context("Failed to stop service")?;

    println!("Service '{}' stopped.", SERVICE_NAME);
    Ok(())
}

fn cmd_status() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("Failed to open service manager")?;

    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        .context("Failed to open service (is it installed?)")?;

    let status = service
        .query_status()
        .context("Failed to query service status")?;

    let state_label = match status.current_state {
        ServiceState::Stopped => "Stopped",
        ServiceState::StartPending => "Start Pending",
        ServiceState::StopPending => "Stop Pending",
        ServiceState::Running => "Running",
        ServiceState::ContinuePending => "Continue Pending",
        ServiceState::PausePending => "Pause Pending",
        ServiceState::Paused => "Paused",
    };

    println!("Service '{}': {}", SERVICE_NAME, state_label);
    Ok(())
}
