use anyhow::{Context, bail};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    DuplicateTokenEx, GetTokenInformation, SecurityImpersonation, TOKEN_ALL_ACCESS,
    TOKEN_ELEVATION, TOKEN_LINKED_TOKEN, TokenElevation, TokenLinkedToken, TokenPrimary,
};
use windows::Win32::System::RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken};
use windows::Win32::System::Threading::{
    CREATE_NO_WINDOW, CreateProcessAsUserW, PROCESS_INFORMATION, STARTUPINFOW,
};
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

/// Check if a token is elevated.
fn is_token_elevated(token: HANDLE) -> bool {
    unsafe {
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Get the linked (elevated) token from a filtered user token.
/// On UAC-enabled systems, admin users have a filtered token and a linked elevated token.
fn get_linked_token(token: HANDLE) -> anyhow::Result<OwnedHandle> {
    unsafe {
        let mut linked = TOKEN_LINKED_TOKEN::default();
        let mut size = 0u32;
        GetTokenInformation(
            token,
            TokenLinkedToken,
            Some(&mut linked as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_LINKED_TOKEN>() as u32,
            &mut size,
        )
        .context("GetTokenInformation(TokenLinkedToken) failed")?;

        Ok(OwnedHandle(linked.LinkedToken))
    }
}

/// Build the command line to launch the helper mode: `"<path-to-breeze.exe>" helper`
pub fn get_helper_command_line() -> anyhow::Result<String> {
    let exe = std::env::current_exe().context("failed to get current exe path")?;
    Ok(format!("\"{}\" helper", exe.to_string_lossy()))
}

/// Launch a process in the active console user session with elevated privileges.
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
    let user_token = OwnedHandle(token);

    // Get an elevated token: if the user token is not elevated, get the linked token
    let elevated_token = if is_token_elevated(user_token.raw()) {
        user_token
    } else {
        let linked = get_linked_token(user_token.raw())?;
        drop(user_token); // close the original
        linked
    };

    // Duplicate as a primary token suitable for CreateProcessAsUser
    let mut dup_token = HANDLE::default();
    unsafe {
        DuplicateTokenEx(
            elevated_token.raw(),
            TOKEN_ALL_ACCESS,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &mut dup_token,
        )
        .context("DuplicateTokenEx failed")?;
    }
    let primary_token = OwnedHandle(dup_token);

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
            Some(primary_token.raw()),
            None,
            Some(PWSTR(cmd_line.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_NO_WINDOW,
            None,
            None,
            &si,
            &mut pi,
        )
        .context("CreateProcessAsUserW failed")?;
    }

    let pid = pi.dwProcessId;
    let _thread = OwnedHandle(pi.hThread);
    let process_handle = OwnedHandle(pi.hProcess);

    Ok((pid, process_handle))
}
