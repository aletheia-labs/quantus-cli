/// Configuration management module
///
/// This module handles:
/// - Loading and saving CLI configuration
/// - Managing default settings
/// - Environment variable handling
/// - Configuration file validation
use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default node URL
    pub node_url: String,

    /// Default wallet name
    pub default_wallet: Option<String>,

    /// Logging level
    pub log_level: String,

    /// Configuration file version
    pub version: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_url: "ws://127.0.0.1:9944".to_string(),
            default_wallet: None,
            log_level: "info".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or(ConfigError::NotFound)?
            .join("quantus")
            .join("config.toml");

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self { config_path })
    }

    /// Load configuration from file
    pub fn load(&self) -> Result<Config> {
        if self.config_path.exists() {
            let config_str = std::fs::read_to_string(&self.config_path)?;
            let config: Config = toml::from_str(&config_str)?;
            Ok(config)
        } else {
            // Create default config if it doesn't exist
            let config = Config::default();
            self.save(&config)?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self, config: &Config) -> Result<()> {
        let config_str = toml::to_string_pretty(config)?;
        std::fs::write(&self.config_path, config_str)?;
        Ok(())
    }

    /// Get configuration file path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
