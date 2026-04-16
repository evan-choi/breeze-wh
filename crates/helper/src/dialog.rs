use breeze_common::constants::{OK_BUTTON_AUTOMATION_ID, PASSWORD_FIELD_AUTOMATION_ID};
use windows::core::{Interface, Result};
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationInvokePattern, TreeScope_Descendants,
    UIA_ButtonControlTypeId, UIA_InvokePatternId, UIA_CONTROLTYPE_ID,
};

/// Result of scanning a credential dialog's UI tree in a single pass.
pub struct DialogScanResult {
    pub has_password_field: bool,
    pub ok_button: Option<IUIAutomationElement>,
}

/// Scan all descendants of `dialog` in one FindAll pass.
/// Returns whether a password field exists (PIN mode) and the OkButton element if found.
pub fn scan_dialog(
    automation: &IUIAutomation,
    dialog: &IUIAutomationElement,
) -> Result<DialogScanResult> {
    let mut has_password_field = false;
    let mut ok_button: Option<IUIAutomationElement> = None;

    unsafe {
        let condition = automation.CreateTrueCondition()?;
        let all_elements = dialog.FindAll(TreeScope_Descendants, &condition)?;
        let count = all_elements.Length()?;

        for i in 0..count {
            let el = all_elements.GetElement(i)?;
            let auto_id = el.CurrentAutomationId().unwrap_or_default().to_string();

            if auto_id == PASSWORD_FIELD_AUTOMATION_ID {
                has_password_field = true;
                // Early return: PIN mode, no need to find OkButton
                return Ok(DialogScanResult {
                    has_password_field,
                    ok_button: None,
                });
            }

            if auto_id == OK_BUTTON_AUTOMATION_ID {
                let ctrl_type = el.CurrentControlType().unwrap_or(UIA_CONTROLTYPE_ID(0));
                if ctrl_type == UIA_ButtonControlTypeId {
                    ok_button = Some(el);
                }
            }
        }
    }

    Ok(DialogScanResult {
        has_password_field,
        ok_button,
    })
}

/// Click a button via InvokePattern.
pub fn invoke_button(button: &IUIAutomationElement) -> Result<()> {
    unsafe {
        let pattern = button.GetCurrentPattern(UIA_InvokePatternId)?;
        let invoke: IUIAutomationInvokePattern = pattern.cast()?;
        invoke.Invoke()
    }
}
