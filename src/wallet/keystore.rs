/// Secure keystore for wallet data
///
/// This module handles:
/// - Encrypting and storing wallet data
/// - Loading and decrypting wallet data
/// - Managing wallet files on disk
use crate::error::Result;
use poseidon_resonance::PoseidonHasher;
use rusty_crystals_dilithium::ml_dsa_87::{Keypair, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use sp_core::crypto::{AccountId32, Ss58Codec};
use sp_core::Hasher;

use std::path::Path;

/// Local quantum-safe key pair for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

impl QuantumKeyPair {
    /// Create from rusty-crystals Keypair
    pub fn from_dilithium_keypair(keypair: &Keypair) -> Self {
        Self {
            public_key: keypair.public.to_bytes().to_vec(),
            private_key: keypair.secret.to_bytes().to_vec(),
        }
    }

    /// Convert to rusty-crystals Keypair
    pub fn to_dilithium_keypair(&self) -> Result<Keypair> {
        // TODO: Implement conversion from bytes back to Keypair
        // For now, generate a new one as placeholder
        Ok(Keypair {
            public: PublicKey::from_bytes(&self.public_key).expect("Failed to parse public key"),
            secret: SecretKey::from_bytes(&self.private_key).expect("Failed to parse private key"),
        })
    }

    pub fn to_account_id_32(&self) -> AccountId32 {
        let hashed = <PoseidonHasher as Hasher>::hash(self.public_key.as_slice());
        let account = AccountId32::from(hashed.0);
        account
    }

    pub fn to_account_id_ss58check(&self) -> String {
        let account = self.to_account_id_32();
        let result = account.to_ss58check();
        result
    }

    pub fn ss58_to_account_id(s: &str) -> Vec<u8> {
        // from_ss58check returns a Result, we unwrap it to panic on invalid input.
        // We then convert the AccountId32 struct to a Vec<u8> to be compatible with Polkadart's typedef.
        AsRef::<[u8]>::as_ref(&AccountId32::from_ss58check(s).unwrap()).to_vec()
    }
}

/// Encrypted wallet data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedWallet {
    pub name: String,
    pub encrypted_data: Vec<u8>,
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Wallet data structure (before encryption)
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletData {
    pub name: String,
    pub keypair: QuantumKeyPair,
    pub mnemonic: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Keystore manager for handling encrypted wallet storage
pub struct Keystore {
    storage_path: std::path::PathBuf,
}

impl Keystore {
    /// Create a new keystore instance
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Self {
        Self {
            storage_path: storage_path.as_ref().to_path_buf(),
        }
    }

    /// Save an encrypted wallet to disk
    pub fn save_wallet(&self, wallet: &EncryptedWallet) -> Result<()> {
        let wallet_file = self.storage_path.join(format!("{}.json", wallet.name));
        let wallet_json = serde_json::to_string_pretty(wallet)?;
        std::fs::write(wallet_file, wallet_json)?;
        Ok(())
    }

    /// Load an encrypted wallet from disk
    pub fn load_wallet(&self, name: &str) -> Result<Option<EncryptedWallet>> {
        let wallet_file = self.storage_path.join(format!("{}.json", name));

        if !wallet_file.exists() {
            return Ok(None);
        }

        let wallet_json = std::fs::read_to_string(wallet_file)?;
        let wallet: EncryptedWallet = serde_json::from_str(&wallet_json)?;
        Ok(Some(wallet))
    }

    /// List all wallet files
    pub fn list_wallets(&self) -> Result<Vec<String>> {
        let mut wallets = Vec::new();

        if !self.storage_path.exists() {
            return Ok(wallets);
        }

        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    wallets.push(name.to_string());
                }
            }
        }

        Ok(wallets)
    }

    /// Delete a wallet file
    pub fn delete_wallet(&self, name: &str) -> Result<bool> {
        let wallet_file = self.storage_path.join(format!("{}.json", name));

        if wallet_file.exists() {
            std::fs::remove_file(wallet_file)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Encrypt wallet data (placeholder implementation)
    pub fn encrypt_wallet_data(
        &self,
        data: &WalletData,
        password: &str,
    ) -> Result<EncryptedWallet> {
        // TODO: Implement proper encryption
        // For now, just serialize the data as a placeholder
        let serialized = serde_json::to_vec(data)?;

        Ok(EncryptedWallet {
            name: data.name.clone(),
            encrypted_data: serialized, // This should be encrypted!
            salt: vec![0u8; 32],        // Placeholder salt
            nonce: vec![0u8; 12],       // Placeholder nonce
            created_at: chrono::Utc::now(),
        })
    }

    /// Decrypt wallet data (placeholder implementation)
    pub fn decrypt_wallet_data(
        &self,
        encrypted: &EncryptedWallet,
        password: &str,
    ) -> Result<WalletData> {
        // TODO: Implement proper decryption
        // For now, just deserialize the data as a placeholder
        let data: WalletData = serde_json::from_slice(&encrypted.encrypted_data)?;
        Ok(data)
    }
}
