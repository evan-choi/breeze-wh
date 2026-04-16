use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken, WaitForSingleObject,
};
use windows::Win32::UI::Shell::{
    SEE_MASK_NO_CONSOLE, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::core::PCWSTR;

/// Check if the current process is running elevated (admin).
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );

        let _ = CloseHandle(token);
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Re-launch the current process elevated via UAC, capture output, and print it.
/// Returns the exit code of the elevated process.
pub fn elevate_and_wait() -> std::process::ExitCode {
    let exe = std::env::current_exe().expect("failed to get current exe path");

    // Create a temp file for the elevated process to write its output to
    let output_file =
        std::env::temp_dir().join(format!("breeze-wh-elevated-{}.log", std::process::id()));

    // Build args: original args + --output-file <path>
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    args.push("--output-file".to_string());
    args.push(output_file.to_string_lossy().into_owned());
    let args_str = args.join(" ");

    let exe_wide: Vec<u16> = OsStr::new(&exe).encode_wide().chain(Some(0)).collect();
    let args_wide: Vec<u16> = OsStr::new(&args_str).encode_wide().chain(Some(0)).collect();
    let runas: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();

    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NO_CONSOLE,
        lpVerb: PCWSTR(runas.as_ptr()),
        lpFile: PCWSTR(exe_wide.as_ptr()),
        lpParameters: PCWSTR(args_wide.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };

    let code = unsafe {
        if ShellExecuteExW(&mut sei).is_err() {
            eprintln!("Failed to request administrator privileges.");
            return std::process::ExitCode::FAILURE;
        }

        let process = sei.hProcess;
        if process.is_invalid() {
            eprintln!("UAC prompt was cancelled.");
            return std::process::ExitCode::FAILURE;
        }

        let wait = WaitForSingleObject(process, INFINITE);
        if wait != WAIT_OBJECT_0 {
            let _ = CloseHandle(process);
            eprintln!("Failed to wait for elevated process.");
            return std::process::ExitCode::FAILURE;
        }

        let mut exit_code = 0u32;
        let _ = GetExitCodeProcess(process, &mut exit_code);
        let _ = CloseHandle(process);
        exit_code
    };

    // Read and print captured output
    if let Ok(output) = std::fs::read_to_string(&output_file) {
        if !output.is_empty() {
            eprint!("{output}");
        }
    }
    let _ = std::fs::remove_file(&output_file);

    if code == 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// Extract `--output-file <path>` from args if present.
/// Returns the file path and the remaining args.
pub fn extract_output_file(args: &[String]) -> (Option<PathBuf>, Vec<String>) {
    let mut output_file = None;
    let mut filtered = Vec::new();
    let mut skip_next = false;

    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--output-file" {
            if let Some(path) = args.get(i + 1) {
                output_file = Some(PathBuf::from(path));
                skip_next = true;
            }
        } else {
            filtered.push(arg.clone());
        }
    }

    (output_file, filtered)
}
