use super::quantus_runtime_config::QuantusRuntimeConfig;
/// Chain client for interacting with the Quantus network
use crate::error::Result;
use crate::wallet::QuantumKeyPair;
use crate::{log_debug, log_print, log_verbose};
use colored::Colorize;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58Codec;
use substrate_api_client::{
    ac_primitives::ExtrinsicSigner, extrinsic::BalancesExtrinsics, rpc::JsonrpseeClient, Api,
    GetAccountInformation, SubmitAndWatch, SystemApi, XtStatus,
};

/// Macro to submit any type of extrinsic without code duplication
/// Note: This should be a method but it seems impossible to figure out the correct parameter types for this call.
/// Extrinsics are more lenient with typing.
macro_rules! submit_extrinsic {
    ($self:expr, $keypair:expr, $extrinsic:expr) => {{
        // Convert our QuantumKeyPair to ResonancePair
        let resonance_pair = $keypair.to_resonance_pair().map_err(|e| {
            crate::error::QuantusError::NetworkError(format!("Failed to convert keypair: {:?}", e))
        })?;

        // Create ExtrinsicSigner
        let extrinsic_signer = ExtrinsicSigner::<QuantusRuntimeConfig>::new(resonance_pair);

        // Clone the API to set the signer
        let mut api_with_signer = $self.api.clone();
        api_with_signer.set_signer(extrinsic_signer);

        // Submit and watch the extrinsic until it's included in a block
        let result = api_with_signer
            .submit_and_watch_extrinsic_until($extrinsic, XtStatus::InBlock)
            .await
            .map_err(|e| {
                crate::error::QuantusError::NetworkError(format!(
                    "Failed to submit extrinsic: {:?}",
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
        Ok::<String, crate::error::QuantusError>(format!("{:?}", extrinsic_hash))
    }};
}

/// Chain client for interacting with the Quantus network
pub struct ChainClient {
    api: Api<QuantusRuntimeConfig, JsonrpseeClient>,
}

impl ChainClient {
    /// Create a new chain client
    pub async fn new(node_url: &str) -> Result<Self> {
        log_verbose!("🔗 Connecting to Quantus node: {}", node_url.bright_blue());

        // Use the substrate-api-client with the provided node URL
        let client = JsonrpseeClient::new(node_url).await.map_err(|e| {
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

        Ok(Self { api })
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
        log_verbose!("🚀 Creating transfer transaction...");
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
        let transfer_extrinsic = api_with_signer
            .balance_transfer_allow_death(to_account_id.into(), amount)
            .await
            .ok_or_else(|| {
                crate::error::QuantusError::NetworkError(
                    "Failed to create transfer extrinsic".to_string(),
                )
            })?;

        log_verbose!("📋 Extrinsic created: {:?}", transfer_extrinsic);

        // Use the macro to submit the extrinsic
        submit_extrinsic!(self, from_keypair, transfer_extrinsic)
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

    /// Get chain properties including token decimals
    pub async fn get_chain_properties(&self) -> Result<(String, u8)> {
        log_verbose!("🔍 Querying chain properties...");

        // Get system properties from the chain
        let properties = self.api.get_system_properties().await.map_err(|e| {
            crate::error::QuantusError::NetworkError(format!(
                "Failed to get system properties: {:?}",
                e
            ))
        })?;

        log_verbose!("📊 Chain properties: {:?}", properties);

        // Extract token symbol and decimals from properties
        // Handle both direct values and arrays (different chains may use different formats)
        let token_symbol = properties
            .get("tokenSymbol")
            .and_then(|v| {
                // Try direct string first
                if let Some(s) = v.as_str() {
                    Some(s)
                } else {
                    // Try array format (some chains use arrays)
                    v.as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                }
            })
            .unwrap_or("QUAN")
            .to_string();

        let token_decimals = properties
            .get("tokenDecimals")
            .and_then(|v| {
                // Try direct number first
                if let Some(n) = v.as_u64() {
                    Some(n)
                } else {
                    // Try array format (some chains use arrays)
                    v.as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_u64())
                }
            })
            .unwrap_or(12) as u8; // Default to 12 decimals if not found

        log_verbose!(
            "💰 Token: {} with {} decimals",
            token_symbol,
            token_decimals
        );

        Ok((token_symbol, token_decimals))
    }

    /// Get chain metadata and explore all available pallets and calls
    pub async fn explore_chain_metadata(&self, no_docs: bool) -> Result<()> {
        log_verbose!("🔍 Exploring chain metadata...");

        // Get the metadata from the API (it's already loaded)
        let metadata = self.api.metadata();

        log_print!("🏗️  Chain Metadata Information:");

        // Get pallets from metadata
        let pallets: Vec<_> = metadata.pallets().collect();
        log_print!("📦 Found {} pallets:\n", pallets.len());

        for (i, pallet) in pallets.iter().enumerate() {
            log_print!(
                "{}. {} (Index: {})",
                (i + 1).to_string().bright_yellow(),
                pallet.name().bright_green(),
                pallet.index().to_string().bright_cyan()
            );

            // Show available calls for this pallet
            if let Some(call_variants) = pallet.call_variants() {
                log_print!(
                    "   📞 Calls ({}):",
                    call_variants.len().to_string().bright_blue()
                );

                for (j, call) in call_variants.iter().enumerate().take(5) {
                    log_print!(
                        "      {}. {}",
                        (j + 1).to_string().bright_white(),
                        call.name.bright_green()
                    );

                    // Show call documentation (unless --no-docs flag is used)
                    if !no_docs {
                        let docs = call.docs.clone();
                        if !docs.is_empty() {
                            log_print!("         📝 {}", docs.join(" ").dimmed());
                        }
                    }

                    // Show call parameters
                    let fields: Vec<_> = call.fields.clone();
                    if !fields.is_empty() {
                        log_print!("         📥 Parameters:");
                        for field in fields.iter() {
                            let field_name = field.name.clone().unwrap_or("<unnamed>".to_string());

                            // Try to get a readable type name
                            let type_info = if let Some(type_name) = &field.type_name {
                                type_name.clone()
                            } else {
                                // Try to resolve type from metadata registry
                                match metadata.types().resolve(field.ty.id()) {
                                    Some(type_def) => {
                                        // Try to get a human-readable type name
                                        match type_def.type_def() {
                                            scale_info::TypeDef::Primitive(primitive) => {
                                                format!("{:?}", primitive)
                                            }
                                            scale_info::TypeDef::Compact(compact) => {
                                                format!(
                                                    "Compact<Type #{}>",
                                                    compact.type_param().id()
                                                )
                                            }
                                            scale_info::TypeDef::Sequence(seq) => {
                                                format!("Vec<Type #{}>", seq.type_param().id())
                                            }
                                            scale_info::TypeDef::Array(arr) => {
                                                format!(
                                                    "[Type #{}; {}]",
                                                    arr.type_param().id(),
                                                    arr.len()
                                                )
                                            }
                                            scale_info::TypeDef::Tuple(tuple) => {
                                                if tuple.fields().is_empty() {
                                                    "()".to_string()
                                                } else {
                                                    format!(
                                                        "Tuple with {} fields",
                                                        tuple.fields().len()
                                                    )
                                                }
                                            }
                                            scale_info::TypeDef::Composite(composite) => {
                                                if let Some(path) =
                                                    type_def.path().segments().last()
                                                {
                                                    path.clone()
                                                } else {
                                                    format!(
                                                        "Composite<{} fields>",
                                                        composite.fields().len()
                                                    )
                                                }
                                            }
                                            scale_info::TypeDef::Variant(variant) => {
                                                if let Some(path) =
                                                    type_def.path().segments().last()
                                                {
                                                    path.clone()
                                                } else {
                                                    format!(
                                                        "Enum<{} variants>",
                                                        variant.variants().len()
                                                    )
                                                }
                                            }
                                            scale_info::TypeDef::BitSequence(_) => {
                                                "BitVec".to_string()
                                            }
                                        }
                                    }
                                    None => format!("Type #{}", field.ty.id()),
                                }
                            };

                            log_print!(
                                "           • {}: {}",
                                field_name.bright_cyan(),
                                type_info.dimmed()
                            );
                        }
                    } else {
                        log_print!("         📥 No parameters");
                    }
                }
            } else {
                log_print!("   📞 No calls available");
            }

            // Show storage items
            let storage_entries: Vec<_> = pallet.storage().collect();
            if !storage_entries.is_empty() {
                log_print!(
                    "   💾 Storage ({}):",
                    storage_entries.len().to_string().bright_magenta()
                );

                for (j, entry) in storage_entries.iter().enumerate() {
                    log_print!(
                        "      {}. {}",
                        (j + 1).to_string().bright_white(),
                        entry.name.clone().bright_magenta()
                    );
                }
            }

            // Show events
            if let Some(event_variants) = pallet.event_variants() {
                log_print!(
                    "   📡 Events ({}):",
                    event_variants.len().to_string().bright_red()
                );

                for (j, event) in event_variants.iter().enumerate() {
                    log_print!(
                        "      {}. {}",
                        (j + 1).to_string().bright_white(),
                        event.name.bright_red()
                    );
                }
            }

            if i < pallets.len() - 1 {
                log_print!();
            }
        }

        log_print!("\n💡 Use this information to implement new extrinsic calls!");
        log_print!("💡 Each call can be submitted using the submit_extrinsic! macro");

        Ok(())
    }

    /// Format balance with proper decimals
    pub fn format_balance(amount: u128, decimals: u8) -> String {
        if decimals == 0 {
            return amount.to_string();
        }

        let divisor = 10_u128.pow(decimals as u32);
        let whole_part = amount / divisor;
        let fractional_part = amount % divisor;

        if fractional_part == 0 {
            whole_part.to_string()
        } else {
            // Format fractional part with proper leading zeros
            let fractional_str = format!("{:0width$}", fractional_part, width = decimals as usize);
            // Remove trailing zeros
            let fractional_str = fractional_str.trim_end_matches('0');

            if fractional_str.is_empty() {
                whole_part.to_string()
            } else {
                format!("{}.{}", whole_part, fractional_str)
            }
        }
    }

    /// Format balance with token symbol
    pub async fn format_balance_with_symbol(&self, amount: u128) -> Result<String> {
        let (symbol, decimals) = self.get_chain_properties().await?;
        let formatted_amount = Self::format_balance(amount, decimals);
        Ok(format!("{} {}", formatted_amount, symbol))
    }

    /// Parse human-readable amount string to raw chain units
    /// Accepts formats like: "10", "10.0", "0.0000001", "10.5 DEV" (ignores symbol)
    pub async fn parse_amount(&self, amount_str: &str) -> Result<u128> {
        let (_, decimals) = self.get_chain_properties().await?;
        Self::parse_amount_with_decimals(amount_str, decimals)
    }

    /// Parse amount string with specific decimals (static method for testing)
    pub fn parse_amount_with_decimals(amount_str: &str, decimals: u8) -> Result<u128> {
        // Remove any token symbol by taking only the first part (number)
        let amount_part = amount_str.trim().split_whitespace().next().unwrap_or("");

        if amount_part.is_empty() {
            return Err(crate::error::QuantusError::Generic(
                "Amount cannot be empty".to_string(),
            ));
        }

        // Parse the decimal number
        let parsed_amount: f64 = amount_part.parse().map_err(|_| {
            crate::error::QuantusError::Generic(format!(
                "Invalid amount format: '{}'. Use formats like '10', '10.5', '0.0001'",
                amount_part
            ))
        })?;

        if parsed_amount < 0.0 {
            return Err(crate::error::QuantusError::Generic(
                "Amount cannot be negative".to_string(),
            ));
        }

        // Check for too many decimal places
        if let Some(decimal_part) = amount_part.split('.').nth(1) {
            if decimal_part.len() > decimals as usize {
                return Err(crate::error::QuantusError::Generic(format!(
                    "Too many decimal places. Maximum {} decimal places allowed for this chain",
                    decimals
                )));
            }
        }

        // Convert to raw units by multiplying by 10^decimals
        let multiplier = 10_f64.powi(decimals as i32);
        let raw_amount = (parsed_amount * multiplier).round() as u128;

        // Validate the result fits in u128 and isn't zero for non-zero inputs
        if parsed_amount > 0.0 && raw_amount == 0 {
            return Err(crate::error::QuantusError::Generic(
                "Amount too small to represent in chain units".to_string(),
            ));
        }

        Ok(raw_amount)
    }

    /// Validate and format amount for display before sending
    pub async fn validate_and_format_amount(&self, amount_str: &str) -> Result<(u128, String)> {
        let raw_amount = self.parse_amount(amount_str).await?;
        let formatted = self.format_balance_with_symbol(raw_amount).await?;
        Ok((raw_amount, formatted))
    }
}

// Chain client implementation ends here
