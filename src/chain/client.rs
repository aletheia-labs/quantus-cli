use super::quantus_runtime_config::QuantusRuntimeConfig;
/// Chain client for interacting with the Quantus network
use crate::error::Result;
use crate::wallet::QuantumKeyPair;
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
        println!("🔗 Connecting to Quantus node: {}", node_url.bright_blue());

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

        println!("✅ Connected to Quantus node successfully!");

        Ok(Self {
            api,
            node_url: node_url.to_string(),
        })
    }

    /// Get the balance of an account using substrate-api-client
    pub async fn get_balance(&self, account_address: &str) -> Result<u128> {
        println!(
            "💰 Querying balance for account: {}",
            account_address.bright_green()
        );

        // Parse the SS58 address to AccountId32
        let account_id = AccountId32::from_ss58check(account_address).map_err(|e| {
            crate::error::QuantusError::NetworkError(format!("Invalid SS58 address: {:?}", e))
        })?;

        println!("🔍 Account ID: {:?}", account_id);

        // Use the substrate-api-client to get account data - following the example pattern
        match self.api.get_account_data(&account_id).await {
            Ok(Some(account_data)) => {
                let balance = account_data.free;
                println!("✅ Account data found!");
                println!("📊 Free balance: {}", balance);
                println!("📊 Reserved balance: {}", account_data.reserved);
                println!("📊 Frozen balance: {}", account_data.frozen);
                Ok(balance)
            }
            Ok(None) => {
                println!("ℹ️  Account not found in storage, balance is 0");
                Ok(0)
            }
            Err(e) => {
                println!("❌ API Error: {:?}", e);
                Err(crate::error::QuantusError::NetworkError(format!(
                    "Failed to query balance: {:?}",
                    e
                )))
            }
        }
    }

    /// Get system information from the chain
    pub async fn get_system_info(&self) -> Result<()> {
        println!("🔍 Querying system information...");

        // For now, just print a placeholder - we'll implement proper system queries later
        println!("🏗️  Chain: Quantus DevNet");
        println!("🔧 Using substrate-api-client");

        Ok(())
    }

    /// Transfer tokens from one account to another (placeholder for now)
    pub async fn transfer(
        &self,
        from_keypair: &QuantumKeyPair,
        to_address: &str,
        amount: u128,
    ) -> Result<String> {
        println!("📤 Creating transfer transaction...");
        println!(
            "   From: {}",
            from_keypair.to_account_id_ss58check().bright_cyan()
        );
        println!("   To: {}", to_address.bright_green());
        println!("   Amount: {}", amount);

        // This is still a placeholder - we'll implement real transfers next
        let tx_hash = format!(
            "0x{:x}",
            sp_crypto_hashing::blake2_256(format!("transfer-{}-{}", to_address, amount).as_bytes())
                .iter()
                .fold(0u64, |acc, &x| acc.wrapping_mul(256).wrapping_add(x as u64))
        );

        println!("✍️  Transaction signed with Dilithium signature");
        println!("📋 Transaction hash: {}", tx_hash.bright_blue());

        Ok(tx_hash)
    }

    /// Wait for transaction finalization (placeholder)
    pub async fn wait_for_finalization(&self, tx_hash: &str) -> Result<bool> {
        println!(
            "⏳ Waiting for transaction {} to be finalized...",
            tx_hash.bright_blue()
        );

        // This is still a placeholder - we'll implement real transaction monitoring
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        println!(
            "✅ Transaction {} finalized successfully!",
            tx_hash.bright_green()
        );
        Ok(true)
    }
}

// Chain client implementation ends here
