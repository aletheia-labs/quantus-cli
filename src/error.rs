use thiserror::Error;

/// Main error type for the Quantus CLI
#[derive(Error, Debug)]
pub enum QuantusError {
    /// Wallet-related errors
    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),

    /// Chain/API errors
    #[error("Chain error: {0}")]
    Chain(#[from] ChainError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing errors
    #[error("TOML error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization errors  
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Generic errors
    #[error("Error: {0}")]
    Generic(String),
}

/// Wallet-specific errors
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Wallet not found")]
    NotFound,

    #[error("Wallet already exists")]
    AlreadyExists,

    #[error("Invalid mnemonic phrase")]
    InvalidMnemonic,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Key generation failed")]
    KeyGeneration,

    #[error("Encryption failed")]
    Encryption,

    #[error("Decryption failed")]
    Decryption,

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
}

/// Chain interaction errors
#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Connection failed")]
    ConnectionFailed,

    #[error("API call failed: {0}")]
    ApiCallFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found")]
    NotFound,

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Permission denied")]
    PermissionDenied,
}

/// Type alias for Results using QuantusError
pub type Result<T> = std::result::Result<T, QuantusError>;
