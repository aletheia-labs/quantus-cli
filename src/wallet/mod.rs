/// Wallet management module
///
/// This module provides functionality for:
/// - Creating quantum-safe wallets using Dilithium keys
/// - Importing/exporting wallets with mnemonic phrases
/// - Encrypting/decrypting wallet data
/// - Managing multiple wallets
pub mod keystore;

use crate::error::{Result, WalletError};
use keystore::{Keystore, QuantumKeyPair, WalletData};
use rusty_crystals_hdwallet::{generate_mnemonic, HDLattice};
use serde::{Deserialize, Serialize};
/// Wallet information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub name: String,
    pub address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub key_type: String,
}

/// Main wallet manager
pub struct WalletManager {
    wallets_dir: std::path::PathBuf,
}

impl WalletManager {
    /// Create a new wallet manager
    pub fn new() -> Result<Self> {
        let wallets_dir = dirs::home_dir()
            .ok_or(WalletError::KeyGeneration)?
            .join(".quantus")
            .join("wallets");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&wallets_dir)?;

        Ok(Self { wallets_dir })
    }

    /// Create a new wallet
    pub async fn create_wallet(&self, name: &str, password: Option<&str>) -> Result<WalletInfo> {
        // Check if wallet already exists
        let keystore = Keystore::new(&self.wallets_dir);
        if keystore.load_wallet(name)?.is_some() {
            return Err(WalletError::AlreadyExists.into());
        }

        // Generate a new Dilithium keypair'
        let mnemonic = generate_mnemonic(24).map_err(|_| WalletError::KeyGeneration)?;
        let lattice =
            HDLattice::from_mnemonic(&mnemonic, None).expect("Failed to generate lattice");
        let dilithium_keypair = lattice.generate_keys();
        let quantum_keypair = QuantumKeyPair::from_dilithium_keypair(&dilithium_keypair);

        // Create wallet data
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("algorithm".to_string(), "ML-DSA-87".to_string());

        // Generate address from public key (simplified version)
        let address = quantum_keypair.to_account_id_ss58check();

        let wallet_data = WalletData {
            name: name.to_string(),
            keypair: quantum_keypair,
            mnemonic: Some(mnemonic.clone()),
            metadata,
        };

        // Encrypt and save the wallet
        let password = password.unwrap_or(""); // Use empty password if none provided
        let encrypted_wallet = keystore.encrypt_wallet_data(&wallet_data, password)?;
        keystore.save_wallet(&encrypted_wallet)?;

        Ok(WalletInfo {
            name: name.to_string(),
            address,
            created_at: encrypted_wallet.created_at,
            key_type: "Dilithium ML-DSA-87".to_string(),
        })
    }

    /// List all wallets
    pub fn list_wallets(&self) -> Result<Vec<WalletInfo>> {
        // TODO: Implement wallet listing
        Ok(vec![])
    }

    /// Import wallet from mnemonic phrase
    pub async fn import_wallet(
        &self,
        name: &str,
        mnemonic: &str,
        password: Option<&str>,
    ) -> Result<WalletInfo> {
        // Check if wallet already exists
        let keystore = Keystore::new(&self.wallets_dir);
        if keystore.load_wallet(name)?.is_some() {
            return Err(WalletError::AlreadyExists.into());
        }

        // Validate and import from mnemonic
        let lattice =
            HDLattice::from_mnemonic(mnemonic, None).map_err(|_| WalletError::InvalidMnemonic)?;
        let dilithium_keypair = lattice.generate_keys();
        let quantum_keypair = QuantumKeyPair::from_dilithium_keypair(&dilithium_keypair);

        // Create wallet data
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("algorithm".to_string(), "ML-DSA-87".to_string());
        metadata.insert("imported".to_string(), "true".to_string());

        // Generate address from public key
        let address = quantum_keypair.to_account_id_ss58check();

        let wallet_data = WalletData {
            name: name.to_string(),
            keypair: quantum_keypair,
            mnemonic: Some(mnemonic.to_string()),
            metadata,
        };

        // Encrypt and save the wallet
        let password = password.unwrap_or(""); // Use empty password if none provided
        let encrypted_wallet = keystore.encrypt_wallet_data(&wallet_data, password)?;
        keystore.save_wallet(&encrypted_wallet)?;

        Ok(WalletInfo {
            name: name.to_string(),
            address,
            created_at: encrypted_wallet.created_at,
            key_type: "Dilithium ML-DSA-87".to_string(),
        })
    }

    /// Get wallet by name
    pub fn get_wallet(&self, name: &str) -> Result<Option<WalletInfo>> {
        // TODO: Implement wallet retrieval
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_wallet_manager() -> (WalletManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let wallets_dir = temp_dir.path().join("wallets");
        fs::create_dir_all(&wallets_dir).expect("Failed to create wallets directory");

        let wallet_manager = WalletManager { wallets_dir };

        (wallet_manager, temp_dir)
    }

    #[tokio::test]
    async fn test_wallet_creation() {
        let (wallet_manager, _temp_dir) = create_test_wallet_manager().await;

        // Test wallet creation
        let wallet_info = wallet_manager
            .create_wallet("test-wallet", Some("test-password"))
            .await
            .expect("Failed to create wallet");

        // Verify wallet info
        assert_eq!(wallet_info.name, "test-wallet");
        assert!(wallet_info.address.starts_with("5")); // SS58 addresses start with 5
        assert_eq!(wallet_info.key_type, "Dilithium ML-DSA-87");
        assert!(wallet_info.created_at <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_wallet_already_exists() {
        let (wallet_manager, _temp_dir) = create_test_wallet_manager().await;

        // Create first wallet
        wallet_manager
            .create_wallet("duplicate-wallet", None)
            .await
            .expect("Failed to create first wallet");

        // Try to create wallet with same name
        let result = wallet_manager.create_wallet("duplicate-wallet", None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::QuantusError::Wallet(WalletError::AlreadyExists) => {}
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_wallet_file_creation() {
        let (wallet_manager, _temp_dir) = create_test_wallet_manager().await;

        // Create wallet
        let wallet_info = wallet_manager
            .create_wallet("file-test-wallet", Some("password123"))
            .await
            .expect("Failed to create wallet");

        // Check if wallet file exists
        let wallet_file = wallet_manager.wallets_dir.join("file-test-wallet.json");
        assert!(wallet_file.exists(), "Wallet file should exist");

        // Verify file is not empty
        let file_size = fs::metadata(&wallet_file)
            .expect("Failed to get file metadata")
            .len();
        assert!(file_size > 0, "Wallet file should not be empty");
    }

    #[tokio::test]
    async fn test_keystore_encryption_decryption() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let keystore = keystore::Keystore::new(temp_dir.path());

        // Create test wallet data
        let entropy = [1u8; 32]; // Use fixed entropy for deterministic tests
        let dilithium_keypair =
            rusty_crystals_dilithium::ml_dsa_87::Keypair::generate(Some(&entropy));
        let quantum_keypair = keystore::QuantumKeyPair::from_dilithium_keypair(&dilithium_keypair);

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());

        let original_wallet_data = keystore::WalletData {
            name: "test-wallet".to_string(),
            keypair: quantum_keypair,
            mnemonic: Some(
                "test mnemonic phrase with twenty four words here for testing purposes only"
                    .to_string(),
            ),
            metadata,
        };

        // Test encryption
        let encrypted_wallet = keystore
            .encrypt_wallet_data(&original_wallet_data, "test-password")
            .expect("Failed to encrypt wallet data");

        assert_eq!(encrypted_wallet.name, "test-wallet");
        assert!(!encrypted_wallet.encrypted_data.is_empty());
        assert!(!encrypted_wallet.salt.is_empty());
        assert!(!encrypted_wallet.nonce.is_empty());

        // Test decryption
        let decrypted_wallet_data = keystore
            .decrypt_wallet_data(&encrypted_wallet, "test-password")
            .expect("Failed to decrypt wallet data");

        // Verify decrypted data matches original
        assert_eq!(decrypted_wallet_data.name, original_wallet_data.name);
        assert_eq!(
            decrypted_wallet_data.mnemonic,
            original_wallet_data.mnemonic
        );
        assert_eq!(
            decrypted_wallet_data.metadata,
            original_wallet_data.metadata
        );
        assert_eq!(
            decrypted_wallet_data.keypair.public_key,
            original_wallet_data.keypair.public_key
        );
        assert_eq!(
            decrypted_wallet_data.keypair.private_key,
            original_wallet_data.keypair.private_key
        );
    }

    #[tokio::test]
    async fn test_quantum_keypair_address_generation() {
        // Generate keypair
        let entropy = [2u8; 32]; // Use different entropy for variety
        let dilithium_keypair =
            rusty_crystals_dilithium::ml_dsa_87::Keypair::generate(Some(&entropy));
        let quantum_keypair = keystore::QuantumKeyPair::from_dilithium_keypair(&dilithium_keypair);

        // Test address generation
        let account_id = quantum_keypair.to_account_id_32();
        let ss58_address = quantum_keypair.to_account_id_ss58check();

        // Verify SS58 address format
        assert!(
            ss58_address.starts_with("5"),
            "SS58 address should start with 5"
        );
        assert!(
            ss58_address.len() >= 47,
            "SS58 address should be at least 47 characters"
        );

        // Test round-trip conversion
        let converted_account_bytes = keystore::QuantumKeyPair::ss58_to_account_id(&ss58_address);
        let account_bytes: &[u8] = account_id.as_ref();
        assert_eq!(converted_account_bytes, account_bytes);
    }

    #[tokio::test]
    async fn test_keystore_save_and_load() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let keystore = keystore::Keystore::new(temp_dir.path());

        // Create and encrypt wallet data
        let entropy = [3u8; 32]; // Use different entropy for each test
        let dilithium_keypair =
            rusty_crystals_dilithium::ml_dsa_87::Keypair::generate(Some(&entropy));
        let quantum_keypair = keystore::QuantumKeyPair::from_dilithium_keypair(&dilithium_keypair);

        let wallet_data = keystore::WalletData {
            name: "save-load-test".to_string(),
            keypair: quantum_keypair,
            mnemonic: Some("save load test mnemonic phrase".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        let encrypted_wallet = keystore
            .encrypt_wallet_data(&wallet_data, "save-load-password")
            .expect("Failed to encrypt wallet");

        // Save wallet
        keystore
            .save_wallet(&encrypted_wallet)
            .expect("Failed to save wallet");

        // Load wallet
        let loaded_wallet = keystore
            .load_wallet("save-load-test")
            .expect("Failed to load wallet")
            .expect("Wallet should exist");

        // Verify loaded wallet matches saved wallet
        assert_eq!(loaded_wallet.name, encrypted_wallet.name);
        assert_eq!(
            loaded_wallet.encrypted_data,
            encrypted_wallet.encrypted_data
        );
        assert_eq!(loaded_wallet.salt, encrypted_wallet.salt);
        assert_eq!(loaded_wallet.nonce, encrypted_wallet.nonce);

        // Test loading non-existent wallet
        let non_existent = keystore
            .load_wallet("non-existent-wallet")
            .expect("Load should succeed but return None");
        assert!(non_existent.is_none());
    }

    #[tokio::test]
    async fn test_mnemonic_generation_and_key_derivation() {
        let (wallet_manager, _temp_dir) = create_test_wallet_manager().await;

        // Create multiple wallets to test mnemonic uniqueness
        let wallet1 = wallet_manager
            .create_wallet("mnemonic-test-1", None)
            .await
            .expect("Failed to create wallet 1");

        let wallet2 = wallet_manager
            .create_wallet("mnemonic-test-2", None)
            .await
            .expect("Failed to create wallet 2");

        // Addresses should be different (extremely unlikely to be the same)
        assert_ne!(wallet1.address, wallet2.address);

        // Both should be valid SS58 addresses
        assert!(wallet1.address.starts_with("5"));
        assert!(wallet2.address.starts_with("5"));
    }

    #[tokio::test]
    async fn test_wallet_import() {
        let (wallet_manager, _temp_dir) = create_test_wallet_manager().await;

        // Test mnemonic phrase (24 words)
        let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        // Import wallet
        let imported_wallet = wallet_manager
            .import_wallet(
                "imported-test-wallet",
                test_mnemonic,
                Some("import-password"),
            )
            .await
            .expect("Failed to import wallet");

        // Verify wallet info
        assert_eq!(imported_wallet.name, "imported-test-wallet");
        assert!(imported_wallet.address.starts_with("5"));
        assert_eq!(imported_wallet.key_type, "Dilithium ML-DSA-87");

        // Import the same mnemonic again should create the same address
        let imported_wallet2 = wallet_manager
            .import_wallet("imported-test-wallet-2", test_mnemonic, None)
            .await
            .expect("Failed to import wallet again");

        assert_eq!(imported_wallet.address, imported_wallet2.address);
    }

    #[tokio::test]
    async fn test_wallet_import_invalid_mnemonic() {
        let (wallet_manager, _temp_dir) = create_test_wallet_manager().await;

        // Test with invalid mnemonic
        let invalid_mnemonic = "invalid mnemonic phrase that should not work";

        let result = wallet_manager
            .import_wallet("invalid-wallet", invalid_mnemonic, None)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::QuantusError::Wallet(WalletError::InvalidMnemonic) => {}
            _ => panic!("Expected InvalidMnemonic error"),
        }
    }

    #[tokio::test]
    async fn test_wallet_import_already_exists() {
        let (wallet_manager, _temp_dir) = create_test_wallet_manager().await;

        let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        // Import first wallet
        wallet_manager
            .import_wallet("duplicate-import-wallet", test_mnemonic, None)
            .await
            .expect("Failed to import first wallet");

        // Try to import with same name
        let result = wallet_manager
            .import_wallet("duplicate-import-wallet", test_mnemonic, None)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::QuantusError::Wallet(WalletError::AlreadyExists) => {}
            _ => panic!("Expected AlreadyExists error"),
        }
    }
}
