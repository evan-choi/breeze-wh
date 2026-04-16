pub const CREDENTIAL_DIALOG_CLASS: &str = "Credential Dialog Xaml Host";
pub const OK_BUTTON_AUTOMATION_ID: &str = "OkButton";
pub const PASSWORD_FIELD_AUTOMATION_ID: &str = "PasswordField_4";

pub const SERVICE_NAME: &str = "Breeze";
pub const SERVICE_DISPLAY_NAME: &str = "Breeze - Auto Windows Hello";

pub const DEBOUNCE_MS: u64 = 2000;
pub const SUPERVISOR_POLL_INTERVAL_MS: u64 = 2000;
pub const BACKOFF_INITIAL_MS: u64 = 2000;
pub const BACKOFF_MAX_MS: u64 = 60_000;
pub const BACKOFF_RESET_AFTER_MS: u64 = 30_000;

pub fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(r"C:\ProgramData\Breeze")
}

pub fn log_dir() -> std::path::PathBuf {
    data_dir().join("logs")
}

pub fn config_path() -> std::path::PathBuf {
    data_dir().join("config.toml")
}
