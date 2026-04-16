use std::sync::{Arc, Mutex};
use std::time::Instant;

use windows::core::{implement, Result};
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationFocusChangedEventHandler,
    IUIAutomationFocusChangedEventHandler_Impl, IUIAutomationStructureChangedEventHandler,
    IUIAutomationStructureChangedEventHandler_Impl, StructureChangeType, TreeScope_Subtree,
};
use windows_core::Ref;

use breeze_common::constants::CREDENTIAL_DIALOG_CLASS;

use crate::dialog;

/// Tracks a registered StructureChangedEventHandler for cleanup.
struct RegisteredHandler {
    handler: IUIAutomationStructureChangedEventHandler,
    dialog: IUIAutomationElement,
}

/// Shared state between FocusHandler and StructureHandler.
///
/// # Safety
/// All COM pointers are accessed exclusively from the STA message pump thread.
/// The `Mutex` fields are used for interior mutability, not cross-thread access.
unsafe impl Send for SharedState {}
unsafe impl Sync for SharedState {}

pub struct SharedState {
    pub automation: IUIAutomation,
    pub debounce_ms: u64,
    active_handler: Mutex<Option<RegisteredHandler>>,
    last_click: Mutex<Option<(Vec<i32>, Instant)>>,
}

impl SharedState {
    pub fn new(automation: IUIAutomation, debounce_ms: u64) -> Self {
        Self {
            automation,
            debounce_ms,
            active_handler: Mutex::new(None),
            last_click: Mutex::new(None),
        }
    }

    /// Remove the currently registered StructureChangedEventHandler, if any.
    fn remove_active_handler(&self) {
        let mut guard = self.active_handler.lock().unwrap();
        if let Some(reg) = guard.take() {
            unsafe {
                let _ = self
                    .automation
                    .RemoveStructureChangedEventHandler(&reg.dialog, &reg.handler);
            }
            tracing::debug!("Structure change handler removed");
        }
    }

    /// Register a new StructureChangedEventHandler on the given dialog.
    fn register_structure_handler(&self, dialog: &IUIAutomationElement, shared: Arc<SharedState>) {
        // Remove any previous handler first
        self.remove_active_handler();

        let handler_iface: IUIAutomationStructureChangedEventHandler = StructureHandler {
            dialog: dialog.clone(),
            shared,
        }
        .into();

        unsafe {
            match self.automation.AddStructureChangedEventHandler(
                dialog,
                TreeScope_Subtree,
                None,
                &handler_iface,
            ) {
                Ok(()) => {
                    let mut guard = self.active_handler.lock().unwrap();
                    *guard = Some(RegisteredHandler {
                        handler: handler_iface,
                        dialog: dialog.clone(),
                    });
                    tracing::debug!("Structure change handler registered");
                }
                Err(e) => {
                    tracing::warn!("Failed to register structure handler: {}", e);
                }
            }
        }
    }

    /// Check if this dialog was recently clicked (debounce).
    fn is_debounced(&self, runtime_id: &[i32]) -> bool {
        let guard = self.last_click.lock().unwrap();
        if let Some((ref id, ref instant)) = *guard
            && id == runtime_id
            && instant.elapsed().as_millis() < self.debounce_ms as u128
        {
            return true;
        }
        false
    }

    /// Record that we clicked a dialog.
    fn record_click(&self, runtime_id: Vec<i32>) {
        let mut guard = self.last_click.lock().unwrap();
        *guard = Some((runtime_id, Instant::now()));
    }
}

/// Get the runtime ID of a UIA element (stable identifier for debounce).
fn get_runtime_id(element: &IUIAutomationElement) -> Vec<i32> {
    unsafe {
        element
            .GetRuntimeId()
            .ok()
            .and_then(|sa| {
                let sa_ref = &*sa;
                let len = sa_ref.rgsabound[0].cElements as usize;
                let data = sa_ref.pvData as *const i32;
                if data.is_null() || len == 0 {
                    None
                } else {
                    Some(std::slice::from_raw_parts(data, len).to_vec())
                }
            })
            .unwrap_or_default()
    }
}

/// Check if an element is a credential dialog by class name.
fn is_credential_dialog(element: &IUIAutomationElement) -> bool {
    unsafe {
        let class = element.CurrentClassName().unwrap_or_default().to_string();
        class == CREDENTIAL_DIALOG_CLASS
    }
}

/// Try to handle a detected credential dialog: scan and click if appropriate.
/// Returns true if we successfully clicked.
fn try_handle_dialog(shared: &Arc<SharedState>, dialog: &IUIAutomationElement) -> Result<bool> {
    let scan = dialog::scan_dialog(&shared.automation, dialog)?;

    if scan.has_password_field {
        tracing::debug!("PIN mode detected, skipping");
        return Ok(false);
    }

    if let Some(ref button) = scan.ok_button {
        let name = unsafe { button.CurrentName().unwrap_or_default() };
        tracing::info!(name = %name, "Face recognition confirmed, clicking OkButton");

        dialog::invoke_button(button)?;

        let rid = get_runtime_id(dialog);
        shared.record_click(rid);
        shared.remove_active_handler();

        tracing::info!("Auto-click successful");
        return Ok(true);
    }

    Ok(false)
}

// ── StructureChangedHandler ──

#[implement(IUIAutomationStructureChangedEventHandler)]
struct StructureHandler {
    dialog: IUIAutomationElement,
    shared: Arc<SharedState>,
}

impl IUIAutomationStructureChangedEventHandler_Impl for StructureHandler_Impl {
    fn HandleStructureChangedEvent(
        &self,
        _sender: Ref<'_, IUIAutomationElement>,
        _change_type: StructureChangeType,
        _runtime_id: *const SAFEARRAY,
    ) -> Result<()> {
        match try_handle_dialog(&self.shared, &self.dialog) {
            Ok(true) => {} // success — handler will be removed by try_handle_dialog
            Ok(false) => {}
            Err(e) => {
                // Dialog may have closed — remove handler
                tracing::debug!("Structure event error (dialog may have closed): {}", e);
                self.shared.remove_active_handler();
            }
        }
        Ok(())
    }
}

// ── FocusChangedHandler ──

#[implement(IUIAutomationFocusChangedEventHandler)]
pub struct FocusHandler {
    pub shared: Arc<SharedState>,
}

impl IUIAutomationFocusChangedEventHandler_Impl for FocusHandler_Impl {
    fn HandleFocusChangedEvent(
        &self,
        sender: Ref<'_, IUIAutomationElement>,
    ) -> Result<()> {
        let Some(element) = sender.as_ref() else {
            return Ok(());
        };

        // Skip events from our own process
        unsafe {
            let pid = element.CurrentProcessId().unwrap_or(0) as u32;
            if pid == GetCurrentProcessId() {
                return Ok(());
            }
        }

        let shared = &self.shared;

        unsafe {
            let walker = shared.automation.ControlViewWalker()?;

            // Walk up ancestors to find a credential dialog
            let mut current = element.clone();
            for _ in 0..10 {
                if is_credential_dialog(&current) {
                    // Debounce: skip if same dialog was recently handled
                    let rid = get_runtime_id(&current);
                    if shared.is_debounced(&rid) {
                        tracing::trace!("Debounced — same dialog recently handled");
                        return Ok(());
                    }

                    tracing::info!("Credential dialog detected");

                    match try_handle_dialog(shared, &current) {
                        Ok(true) => {
                            // Clicked successfully
                        }
                        Ok(false) => {
                            // OkButton not visible yet — register structure watcher
                            tracing::debug!("OkButton not yet visible, watching for changes");
                            shared.register_structure_handler(&current, Arc::clone(shared));
                        }
                        Err(e) => {
                            tracing::warn!("Error handling dialog: {}", e);
                        }
                    }
                    return Ok(());
                }

                match walker.GetParentElement(&current) {
                    Ok(parent) => current = parent,
                    Err(_) => break,
                }
            }
        }

        Ok(())
    }
}
