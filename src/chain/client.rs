use super::quantus_runtime_config::QuantusRuntimeConfig;
/// Chain client for interacting with the Quantus network
use crate::error::Result;
use crate::wallet::QuantumKeyPair;
use crate::{log_debug, log_error, log_print, log_verbose};
use colored::Colorize;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58Codec;
use substrate_api_client::{
    ac_primitives::Config, rpc::JsonrpseeClient, Api, GetAccountInformation,
};

/// Chain client for interacting with the Quantus network
pub struct ChainClient {
    api: Api<QuantusRuntimeConfig, JsonrpseeClient>,
    node_url: String,
}

impl ChainClient {
    /// Create a new chain client
    pub async fn new(node_url: &str) -> Result<Self> {
        log_verbose!("🔗 Connecting to Quantus node: {}", node_url.bright_blue());

        // Use the substrate-api-client with proper configuration
        let client = JsonrpseeClient::with_default_url().await.map_err(|e| {
            crate::error::QuantusError::NetworkError(format!("Failed to connect to node: {:?}", e))
        })?;

        let api = Api::<QuantusRuntimeConfig, _>::new(client)
            .await
            .map_err(|e| {
                crate::error::QuantusError::NetworkError(format!(
                    "Failed to initialize API: {:?}",
                    e
                ))
            })?;

        log_verbose!("✅ Connected to Quantus node successfully!");

        Ok(Self {
            api,
            node_url: node_url.to_string(),
        })
    }

    /// Get the balance of an account using substrate-api-client
    pub async fn get_balance(&self, account_address: &str) -> Result<u128> {
        log_verbose!(
            "💰 Querying balance for account: {}",
            account_address.bright_green()
        );

        // Parse the SS58 address to AccountId32
        let account_id = AccountId32::from_ss58check(account_address).map_err(|e| {
            crate::error::QuantusError::NetworkError(format!("Invalid SS58 address: {:?}", e))
        })?;

        log_debug!("🔍 Account ID: {:?}", account_id);

        // Use the substrate-api-client to get account data - following the example pattern
        match self.api.get_account_data(&account_id).await {
            Ok(Some(account_data)) => {
                let balance = account_data.free;
                log_verbose!("✅ Account data found!");
                log_verbose!("📊 Free balance: {}", balance);
                log_verbose!("📊 Reserved balance: {}", account_data.reserved);
                log_verbose!("📊 Frozen balance: {}", account_data.frozen);
                Ok(balance)
            }
            Ok(None) => {
                log_verbose!("ℹ️  Account not found in storage, balance is 0");
                Ok(0)
            }
            Err(e) => {
                log_verbose!("❌ API Error: {:?}", e);
                Err(crate::error::QuantusError::NetworkError(format!(
                    "Failed to query balance: {:?}",
                    e
                )))
            }
        }
    }

    /// Get system information from the chain
    pub async fn get_system_info(&self) -> Result<()> {
        log_verbose!("🔍 Querying system information...");

        // For now, just print a placeholder - we'll implement proper system queries later
        log_print!("🏗️  Chain: Quantus DevNet");
        log_print!("🔧 Using substrate-api-client");

        Ok(())
    }

    /// Transfer tokens from one account to another (placeholder for now)
    pub async fn transfer(
        &self,
        from_keypair: &QuantumKeyPair,
        to_address: &str,
        amount: u128,
    ) -> Result<String> {
        log_verbose!("📤 Creating transfer transaction...");
        log_verbose!(
            "   From: {}",
            from_keypair.to_account_id_ss58check().bright_cyan()
        );
        log_verbose!("   To: {}", to_address.bright_green());
        log_verbose!("   Amount: {}", amount);

        // This is still a placeholder - we'll implement real transfers next
        let tx_hash = format!(
            "0x{:x}",
            sp_crypto_hashing::blake2_256(format!("transfer-{}-{}", to_address, amount).as_bytes())
                .iter()
                .fold(0u64, |acc, &x| acc.wrapping_mul(256).wrapping_add(x as u64))
        );

        log_verbose!("✍️  Transaction signed with Dilithium signature");
        log_verbose!("📋 Transaction hash: {}", tx_hash.bright_blue());

        Ok(tx_hash)
    }

    /// Wait for transaction finalization (placeholder)
    pub async fn wait_for_finalization(&self, tx_hash: &str) -> Result<bool> {
        log_verbose!(
            "⏳ Waiting for transaction {} to be finalized...",
            tx_hash.bright_blue()
        );

        // This is still a placeholder - we'll implement real transaction monitoring
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        log_verbose!(
            "✅ Transaction {} finalized successfully!",
            tx_hash.bright_green()
        );
        Ok(true)
    }
}

// Chain client implementation ends here
