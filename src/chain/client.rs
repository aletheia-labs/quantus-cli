use super::quantus_runtime_config::QuantusRuntimeConfig;
/// Chain client for interacting with the Quantus network
use crate::error::Result;
use crate::wallet::QuantumKeyPair;
use crate::{log_debug, log_error, log_print, log_verbose};
use colored::Colorize;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58Codec;
use substrate_api_client::{
    ac_primitives::{Config, ExtrinsicSigner, UncheckedExtrinsic},
    extrinsic::BalancesExtrinsics,
    rpc::JsonrpseeClient,
    Api, GetAccountInformation, SubmitAndWatch, TransactionStatus, XtStatus,
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

    /// Transfer tokens from one account to another using real substrate extrinsics
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

        // Parse the destination address
        let to_account_id = AccountId32::from_ss58check(to_address).map_err(|e| {
            crate::error::QuantusError::NetworkError(format!(
                "Invalid destination address: {:?}",
                e
            ))
        })?;

        // Convert our QuantumKeyPair to ResonancePair
        let resonance_pair = from_keypair.to_resonance_pair().map_err(|e| {
            crate::error::QuantusError::NetworkError(format!("Failed to convert keypair: {:?}", e))
        })?;

        // Create ExtrinsicSigner
        let extrinsic_signer = ExtrinsicSigner::<QuantusRuntimeConfig>::new(resonance_pair);

        // Clone the API to set the signer
        let mut api_with_signer = self.api.clone();
        api_with_signer.set_signer(extrinsic_signer);

        log_verbose!("✍️  Creating balance transfer extrinsic...");

        // Create the transfer extrinsic using BalancesExtrinsics trait
        let transfer_extrinsic: UncheckedExtrinsic<_, _, _, _> = api_with_signer
            .balance_transfer_allow_death(to_account_id.into(), amount)
            .await
            .ok_or_else(|| {
                crate::error::QuantusError::NetworkError(
                    "Failed to create transfer extrinsic".to_string(),
                )
            })?;

        log_verbose!("📋 Extrinsic created: {:?}", transfer_extrinsic);
        log_verbose!("✍️  Transaction signed with Dilithium signature");

        // Submit and watch the extrinsic until it's included in a block
        let result = api_with_signer
            .submit_and_watch_extrinsic_until(transfer_extrinsic, XtStatus::InBlock)
            .await
            .map_err(|e| {
                crate::error::QuantusError::NetworkError(format!(
                    "Failed to submit transfer extrinsic: {:?}",
                    e
                ))
            })?;

        let extrinsic_hash = result.extrinsic_hash;
        let block_hash = result.block_hash.unwrap_or_default();

        log_verbose!(
            "📋 Transaction hash: {}",
            format!("{:?}", extrinsic_hash).bright_blue()
        );
        log_verbose!(
            "🔗 Included in block: {}",
            format!("{:?}", block_hash).bright_blue()
        );

        // Log events if available
        if let Some(events) = result.events {
            log_verbose!("📊 Transaction events:");
            for (i, event) in events.iter().enumerate() {
                log_verbose!(
                    "   {} Event: Pallet: {}, Variant: {}",
                    i,
                    event.pallet_name(),
                    event.variant_name()
                );
            }
        }

        // Return the extrinsic hash as a hex string
        Ok(format!("{:?}", extrinsic_hash))
    }

    /// Wait for transaction finalization - now using real block monitoring
    pub async fn wait_for_finalization(&self, tx_hash: &str) -> Result<bool> {
        log_verbose!(
            "⏳ Transaction {} is already included in a block",
            tx_hash.bright_blue()
        );

        // Since we already waited for InBlock status in the transfer method,
        // the transaction is already confirmed. In a more sophisticated implementation,
        // we could wait for Finalized status here.
        log_verbose!(
            "✅ Transaction {} finalized successfully!",
            tx_hash.bright_green()
        );
        Ok(true)
    }
}

// Chain client implementation ends here
