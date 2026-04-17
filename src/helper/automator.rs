use std::sync::Arc;

use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationFocusChangedEventHandler,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostQuitMessage,
};

use crate::common::config::BreezeConfig;

use super::handlers::{FocusHandler, SharedState};

pub fn run(config: BreezeConfig) -> anyhow::Result<()> {
    unsafe {
        // STA: UIA event handlers are serialized on the message pump thread
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;

        let automation: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)?;
        tracing::info!("UI Automation initialized");

        let shared = Arc::new(SharedState::new(automation.clone(), config.debounce_ms));

        let handler: IUIAutomationFocusChangedEventHandler = FocusHandler {
            shared: Arc::clone(&shared),
        }
        .into();

        automation.AddFocusChangedEventHandler(None, &handler)?;
        tracing::info!("Focus event handler registered, waiting for events");

        // Startup is done — let the OS reclaim cold pages from COM/UIA init.
        crate::common::mem::trim_working_set();

        // Ctrl+C → break message loop
        ctrlc::set_handler(move || {
            tracing::info!("Shutdown signal received");
            PostQuitMessage(0);
        })
        .expect("Failed to set Ctrl-C handler");

        // STA message loop — drives COM event dispatch
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if ret.0 <= 0 {
                break;
            }
            DispatchMessageW(&msg);
        }

        automation.RemoveFocusChangedEventHandler(&handler)?;
        tracing::info!("Helper shutdown complete");
    }

    Ok(())
}
