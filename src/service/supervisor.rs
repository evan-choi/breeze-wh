use std::time::Instant;

use windows::Win32::Foundation::WAIT_TIMEOUT;
use windows::Win32::System::Threading::{TerminateProcess, WaitForSingleObject};

use crate::common::constants::{BACKOFF_INITIAL_MS, BACKOFF_MAX_MS, BACKOFF_RESET_AFTER_MS};

use super::session::{self, OwnedHandle};

pub struct Supervisor {
    helper_handle: Option<OwnedHandle>,
    backoff_ms: u64,
    last_start: Option<Instant>,
    stable_since: Option<Instant>,
}

impl Supervisor {
    pub fn new() -> Self {
        Self {
            helper_handle: None,
            backoff_ms: BACKOFF_INITIAL_MS,
            last_start: None,
            stable_since: None,
        }
    }

    /// Launch the helper process in the active user session.
    pub fn start(&mut self) -> anyhow::Result<()> {
        let cmd = session::get_helper_command_line()?;
        let (pid, handle) = session::launch_in_user_session(&cmd)?;
        tracing::info!(pid, "helper process started");
        self.helper_handle = Some(handle);
        self.last_start = Some(Instant::now());
        self.stable_since = Some(Instant::now());
        Ok(())
    }

    /// Terminate the helper process if it is running.
    pub fn stop(&mut self) {
        if let Some(handle) = self.helper_handle.take() {
            tracing::info!("terminating helper process");
            unsafe {
                let _ = TerminateProcess(handle.raw(), 0);
            }
            // handle is dropped here, closing it via OwnedHandle::drop
        }
        self.stable_since = None;
    }

    /// Called periodically (every ~2 s). Checks helper health and restarts if needed.
    pub fn tick(&mut self) -> anyhow::Result<()> {
        let Some(handle) = self.helper_handle.as_ref() else {
            return Ok(());
        };

        let alive = unsafe { WaitForSingleObject(handle.raw(), 0) } == WAIT_TIMEOUT;

        if alive {
            // Check if the process has been stable long enough to reset backoff
            if let Some(since) = self.stable_since
                && since.elapsed().as_millis() as u64 >= BACKOFF_RESET_AFTER_MS
                && self.backoff_ms != BACKOFF_INITIAL_MS
            {
                tracing::info!("helper stable for 30 s, resetting backoff");
                self.backoff_ms = BACKOFF_INITIAL_MS;
            }
            return Ok(());
        }

        // Helper is dead
        tracing::warn!("helper process exited");
        self.helper_handle = None;
        self.stable_since = None;

        // Check backoff
        if let Some(last) = self.last_start {
            let elapsed = last.elapsed().as_millis() as u64;
            if elapsed < self.backoff_ms {
                // Not yet time to restart
                return Ok(());
            }
        }

        // Restart
        tracing::info!(backoff_ms = self.backoff_ms, "restarting helper");
        match self.start() {
            Ok(()) => {
                // Double the backoff for next crash, capped at max
                self.backoff_ms = (self.backoff_ms * 2).min(BACKOFF_MAX_MS);
            }
            Err(e) => {
                tracing::error!("failed to restart helper: {e:#}");
                self.last_start = Some(Instant::now());
                self.backoff_ms = (self.backoff_ms * 2).min(BACKOFF_MAX_MS);
            }
        }

        Ok(())
    }
}
