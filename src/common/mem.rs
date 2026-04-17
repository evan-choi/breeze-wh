use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Threading::GetCurrentProcess;

/// Best-effort: flush the current process's working set to standby so the OS
/// can reclaim idle pages after startup. Pages page back in on first access.
pub fn trim_working_set() {
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}
