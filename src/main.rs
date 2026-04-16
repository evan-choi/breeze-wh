mod cli;
mod common;
mod helper;
mod service;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // If elevated process passed --output-file, redirect stdout/stderr to it
    let (output_file, args) = common::elevation::extract_output_file(&args);
    let _output_guard = output_file.map(|path| OutputRedirect::new(&path));

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

        // CLI commands (require elevation)
        Some(cmd @ ("install" | "uninstall" | "start" | "stop")) => {
            if !common::elevation::is_elevated() {
                // Quick status check — give useful error before UAC prompt
                if matches!(cmd, "start" | "stop" | "uninstall") {
                    if let Err(e) = cli::check_service_exists() {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                }

                eprintln!("Requesting administrator privileges for '{cmd}'...");
                let code = common::elevation::elevate_and_wait();
                std::process::exit(if code == std::process::ExitCode::SUCCESS {
                    0
                } else {
                    1
                });
            }
            cli::run(&args)
        }

        // CLI commands (no elevation needed)
        _ => cli::run(&args),
    }
}

/// Redirects stdout and stderr to a file (used by elevated child process).
struct OutputRedirect {
    _file: std::fs::File,
}

impl OutputRedirect {
    fn new(path: &std::path::Path) -> Self {
        use std::os::windows::io::AsRawHandle;

        let file = std::fs::File::create(path).expect("failed to create output redirect file");

        // Duplicate the handle for stderr
        let stdout_file = file.try_clone().expect("failed to clone file handle");
        let stderr_file = file.try_clone().expect("failed to clone file handle");

        // Redirect using raw handles via Windows API
        unsafe {
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::System::Console::{
                STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
            };
            let _ = SetStdHandle(STD_OUTPUT_HANDLE, HANDLE(stdout_file.as_raw_handle() as _));
            let _ = SetStdHandle(STD_ERROR_HANDLE, HANDLE(stderr_file.as_raw_handle() as _));
        }

        // Also update Rust's stdout/stderr (they cache the handles)
        // The actual redirection is via SetStdHandle; println!/eprintln! will pick it up
        // after the next write since they call GetStdHandle internally.

        Self { _file: file }
    }
}
