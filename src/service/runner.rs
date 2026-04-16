use std::sync::mpsc;
use std::time::Duration;

use anyhow::Context;
use windows::Win32::System::RemoteDesktop::WTSGetActiveConsoleSessionId;
use windows_service::define_windows_service;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
    ServiceType, SessionChangeReason,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

use crate::common::constants::{SERVICE_NAME, SUPERVISOR_POLL_INTERVAL_MS};

use super::supervisor::Supervisor;

const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

/// Events sent from the SCM control handler to the main service loop.
enum ServiceEvent {
    Stop,
    SessionLogon,
    SessionLogoff,
}

define_windows_service!(ffi_service_main, service_main);

/// Entry point called from main(). Registers with the SCM dispatcher.
pub fn run() -> anyhow::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .context("failed to start service dispatcher")?;
    Ok(())
}

/// Service main function invoked by the SCM via the FFI trampoline.
fn service_main(_args: Vec<std::ffi::OsString>) {
    if let Err(e) = run_service() {
        tracing::error!("service failed: {e:#}");
    }
}

fn run_service() -> anyhow::Result<()> {
    let config = crate::common::config::load_config()?;
    let _log_guard = crate::common::logging::init_logging("service", &config);

    tracing::info!("breeze-wh service starting");

    let (event_tx, event_rx) = mpsc::channel::<ServiceEvent>();

    let mut supervisor = Supervisor::new();

    // Register the service control handler
    let status_handle = service_control_handler::register(
        SERVICE_NAME,
        move |control| -> ServiceControlHandlerResult {
            match control {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    tracing::info!("received stop/shutdown control");
                    let _ = event_tx.send(ServiceEvent::Stop);
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::SessionChange(param) => {
                    match param.reason {
                        SessionChangeReason::SessionLogon => {
                            tracing::info!(
                                session_id = param.notification.session_id,
                                "session logon detected"
                            );
                            let _ = event_tx.send(ServiceEvent::SessionLogon);
                        }
                        SessionChangeReason::SessionLogoff => {
                            tracing::info!(
                                session_id = param.notification.session_id,
                                "session logoff detected"
                            );
                            let _ = event_tx.send(ServiceEvent::SessionLogoff);
                        }
                        _ => {}
                    }
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        },
    )
    .context("failed to register service control handler")?;

    // Report Running
    status_handle
        .set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP
                | ServiceControlAccept::SHUTDOWN
                | ServiceControlAccept::SESSION_CHANGE,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .context("failed to report running status")?;

    tracing::info!("service reported running");

    // Check for an existing active console session on startup
    let session_id = unsafe { WTSGetActiveConsoleSessionId() };
    if session_id != 0xFFFFFFFF {
        tracing::info!(
            session_id,
            "active console session found, starting supervisor"
        );
        if let Err(e) = supervisor.start() {
            tracing::error!("failed to start supervisor on startup: {e:#}");
        }
    }

    // Main service loop
    let poll = Duration::from_millis(SUPERVISOR_POLL_INTERVAL_MS);
    loop {
        match event_rx.recv_timeout(poll) {
            Ok(ServiceEvent::Stop) => {
                tracing::info!("stop signal received, shutting down");
                break;
            }
            Ok(ServiceEvent::SessionLogon) => {
                tracing::info!("session logon — starting supervisor");
                if let Err(e) = supervisor.start() {
                    tracing::error!("failed to start supervisor on logon: {e:#}");
                }
            }
            Ok(ServiceEvent::SessionLogoff) => {
                tracing::info!("session logoff — stopping supervisor");
                supervisor.stop();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Normal poll tick
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                tracing::warn!("control channel disconnected, shutting down");
                break;
            }
        }

        if let Err(e) = supervisor.tick() {
            tracing::error!("supervisor tick error: {e:#}");
        }
    }

    // Shutdown: kill helper, report stopped
    supervisor.stop();

    status_handle
        .set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .context("failed to report stopped status")?;

    tracing::info!("service stopped");
    Ok(())
}
