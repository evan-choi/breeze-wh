use anyhow::{Context, bail};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    DuplicateTokenEx, SecurityImpersonation, TOKEN_ALL_ACCESS, TokenPrimary,
};
use windows::Win32::System::RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken};
use windows::Win32::System::Threading::{CreateProcessAsUserW, PROCESS_INFORMATION, STARTUPINFOW};
use windows::core::PWSTR;

/// RAII wrapper around a Win32 HANDLE that calls CloseHandle on drop.
pub struct OwnedHandle(pub HANDLE);

impl OwnedHandle {
    pub fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Build the command line to launch the helper mode: `"<path-to-breeze.exe>" helper`
pub fn get_helper_command_line() -> anyhow::Result<String> {
    let exe = std::env::current_exe().context("failed to get current exe path")?;
    Ok(format!("\"{}\" helper", exe.to_string_lossy()))
}

/// Launch a process in the active console user session.
/// Returns (process_id, process_handle) on success.
pub fn launch_in_user_session(exe_path: &str) -> anyhow::Result<(u32, OwnedHandle)> {
    let session_id = unsafe { WTSGetActiveConsoleSessionId() };
    if session_id == 0xFFFFFFFF {
        bail!("no active console session");
    }

    // Get the user token for the active console session
    let mut token = HANDLE::default();
    unsafe {
        WTSQueryUserToken(session_id, &mut token)
            .context("WTSQueryUserToken failed — service must run as LocalSystem")?;
    }
    let _token_guard = OwnedHandle(token);

    // Duplicate as a primary token suitable for CreateProcessAsUser
    let mut dup_token = HANDLE::default();
    unsafe {
        DuplicateTokenEx(
            token,
            TOKEN_ALL_ACCESS,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &mut dup_token,
        )
        .context("DuplicateTokenEx failed")?;
    }
    let _dup_guard = OwnedHandle(dup_token);

    // Build a mutable wide-char command line (CreateProcessAsUserW may modify it)
    let mut cmd_line: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessAsUserW(
            Some(dup_token),
            None,
            Some(PWSTR(cmd_line.as_mut_ptr())),
            None,
            None,
            false,
            Default::default(),
            None,
            None,
            &si,
            &mut pi,
        )
        .context("CreateProcessAsUserW failed")?;
    }

    let pid = pi.dwProcessId;
    // Close the thread handle immediately — we only need the process handle
    let _thread = OwnedHandle(pi.hThread);
    let process_handle = OwnedHandle(pi.hProcess);

    Ok((pid, process_handle))
}
