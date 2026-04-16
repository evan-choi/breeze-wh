use crate::constants;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BreezeConfig {
    pub enabled: bool,
    pub debounce_ms: u64,
    pub log_level: String,
    pub log_max_files: usize,
}

impl Default for BreezeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: constants::DEBOUNCE_MS,
            log_level: "info".to_string(),
            log_max_files: 7,
        }
    }
}

pub fn load_config() -> anyhow::Result<BreezeConfig> {
    let path = constants::config_path();
    if !path.exists() {
        return Ok(BreezeConfig::default());
    }
    let text = std::fs::read_to_string(&path)?;
    let config: BreezeConfig = toml::from_str(&text)?;
    Ok(config)
}
