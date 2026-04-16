#[derive(thiserror::Error, Debug)]
pub enum BreezeError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("Windows API error: {0}")]
    Windows(#[from] windows_core::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("service error: {0}")]
    Service(String),
}
