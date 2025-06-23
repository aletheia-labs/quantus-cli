/// Configuration management module
///
/// This module handles:
/// - Loading and saving CLI configuration
/// - Managing default settings
/// - Environment variable handling
/// - Configuration file validation
use serde::{Deserialize, Serialize};

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
