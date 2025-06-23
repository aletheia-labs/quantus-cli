/// Quantum-safe keystore for wallet data
///
/// This module handles:
/// - Quantum-safe encrypting and storing wallet data using Argon2 + AES-256-GCM
/// - Loading and decrypting wallet data with post-quantum cryptography
/// - Managing wallet files on disk with quantum-resistant security
use crate::error::{Result, WalletError};
use poseidon_resonance::PoseidonHasher;
use rusty_crystals_dilithium::ml_dsa_87::{Keypair, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use sp_core::crypto::{AccountId32, Ss58Codec};
use sp_core::Hasher;

// Quantum-safe encryption imports
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

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

/// Quantum-safe encrypted wallet data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedWallet {
    pub name: String,
    pub encrypted_data: Vec<u8>,
    pub kyber_ciphertext: Vec<u8>, // Reserved for future ML-KEM implementation
    pub kyber_public_key: Vec<u8>, // Reserved for future ML-KEM implementation
    pub argon2_salt: Vec<u8>,      // Salt for password-based key derivation
    pub argon2_params: String,     // Argon2 parameters for verification
    pub aes_nonce: Vec<u8>,        // AES-GCM nonce
    pub encryption_version: u32,   // Version for future crypto upgrades
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

    /// Encrypt wallet data using quantum-safe Argon2 + AES-256-GCM
    /// This provides quantum-safe symmetric encryption with strong password derivation
    pub fn encrypt_wallet_data(
        &self,
        data: &WalletData,
        password: &str,
    ) -> Result<EncryptedWallet> {
        // 1. Generate salt for Argon2
        let mut argon2_salt = [0u8; 16];
        OsRng.fill_bytes(&mut argon2_salt);

        // 2. Derive encryption key from password using Argon2 (quantum-safe)
        let argon2 = Argon2::default();
        let salt_string = argon2::password_hash::SaltString::encode_b64(&argon2_salt)
            .map_err(|_| WalletError::Encryption)?;
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|_| WalletError::Encryption)?;

        // 3. Use password hash as AES-256 key (quantum-safe with 256-bit key)
        let hash_bytes = password_hash.hash.as_ref().unwrap().as_bytes();
        let aes_key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
        let cipher = Aes256Gcm::new(aes_key);

        // 4. Generate nonce and encrypt the wallet data
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let serialized_data = serde_json::to_vec(data)?;
        let encrypted_data = cipher
            .encrypt(&nonce, serialized_data.as_ref())
            .map_err(|_| WalletError::Encryption)?;

        Ok(EncryptedWallet {
            name: data.name.clone(),
            encrypted_data,
            kyber_ciphertext: vec![], // Reserved for future ML-KEM implementation
            kyber_public_key: vec![], // Reserved for future ML-KEM implementation
            argon2_salt: argon2_salt.to_vec(),
            argon2_params: password_hash.to_string(),
            aes_nonce: nonce.to_vec(),
            encryption_version: 1, // Version 1: Argon2 + AES-256-GCM (quantum-safe)
            created_at: chrono::Utc::now(),
        })
    }

    /// Decrypt wallet data using quantum-safe decryption
    pub fn decrypt_wallet_data(
        &self,
        encrypted: &EncryptedWallet,
        password: &str,
    ) -> Result<WalletData> {
        // 1. Verify password using stored Argon2 hash
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::new(&encrypted.argon2_params)
            .map_err(|_| WalletError::InvalidPassword)?;

        argon2
            .verify_password(password.as_bytes(), &password_hash)
            .map_err(|_| WalletError::InvalidPassword)?;

        // 2. Derive AES key from verified password hash
        let hash_bytes = password_hash.hash.as_ref().unwrap().as_bytes();
        let aes_key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
        let cipher = Aes256Gcm::new(aes_key);

        // 3. Decrypt the data
        let nonce = Nonce::from_slice(&encrypted.aes_nonce);
        let decrypted_data = cipher
            .decrypt(nonce, encrypted.encrypted_data.as_ref())
            .map_err(|_| WalletError::Decryption)?;

        // 4. Deserialize the wallet data
        let wallet_data: WalletData = serde_json::from_slice(&decrypted_data)?;

        Ok(wallet_data)
    }
}
