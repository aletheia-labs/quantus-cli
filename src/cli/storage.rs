//! `quantus storage` subcommand
use crate::{
    chain::client::ChainClient, error::QuantusError, log_error, log_print, log_success, log_verbose,
};
use clap::Subcommand;
use codec::Encode;
use colored::Colorize;
use sp_core::crypto::{AccountId32, Ss58Codec};
use sp_core::twox_128;
use substrate_api_client::{
    ac_compose_macros::{compose_call, compose_extrinsic},
    SubmitAndWatch,
};

/// Direct interaction with chain storage (Sudo required for set)
#[derive(Subcommand, Debug)]
pub enum StorageCommands {
    /// Get a storage value from a pallet.
    ///
    /// This command constructs a storage key from the pallet and item names,
    /// fetches the raw value from the chain state, and prints it as a hex string.
    Get {
        /// The name of the pallet (e.g., "Scheduler")
        #[arg(long)]
        pallet: String,

        /// The name of the storage item (e.g., "LastProcessedTimestamp")
        #[arg(long)]
        name: String,

        /// Attempt to decode the value as a specific type (e.g., "u64", "AccountId")
        #[arg(long)]
        decode_as: Option<String>,
    },

    /// Set a storage value on the chain.
    ///
    /// This requires sudo privileges. It constructs a `system.set_storage` call
    /// and wraps it in a `sudo.sudo` extrinsic. The provided value should be
    /// a hex-encoded SCALE representation of the value.
    Set {
        /// The name of the pallet (e.g., "Scheduler")
        #[arg(long)]
        pallet: String,

        /// The name of the storage item (e.g., "LastProcessedTimestamp")
        #[arg(long)]
        name: String,

        /// The new value. Can be a plain string if --type is used, otherwise a hex string.
        #[arg(long)]
        value: String,

        /// The type of the value to be encoded (e.g., "u64", "moment", "accountid")
        #[arg(long)]
        r#type: Option<String>,

        /// The name of the wallet to sign the transaction with (must have sudo rights)
        #[arg(long)]
        wallet: String,

        /// The password for the wallet
        #[arg(long)]
        password: Option<String>,
    },
}

/// Handle storage commands
pub async fn handle_storage_command(
    command: StorageCommands,
    node_url: &str,
) -> crate::error::Result<()> {
    match command {
        StorageCommands::Get {
            pallet,
            name,
            decode_as,
        } => {
            log_print!(
                "🔎 Getting storage for {}::{}",
                pallet.bright_green(),
                name.bright_cyan()
            );

            let client = ChainClient::new(node_url).await?;

            // Construct the storage key
            let mut key = twox_128(pallet.as_bytes()).to_vec();
            key.extend(&twox_128(name.as_bytes()));

            let result = client.get_storage_raw(key).await?;

            if let Some(value_bytes) = result {
                log_success!("Raw Value: 0x{}", hex::encode(&value_bytes).bright_yellow());

                if let Some(type_str) = decode_as {
                    use codec::Decode;
                    use sp_core::crypto::{AccountId32, Ss58Codec};

                    log_print!("Attempting to decode as {}...", type_str.bright_cyan());
                    match type_str.to_lowercase().as_str() {
                        "u64" | "moment" => match u64::decode(&mut &value_bytes[..]) {
                            Ok(decoded_value) => {
                                log_success!(
                                    "Decoded Value: {}",
                                    decoded_value.to_string().bright_green()
                                )
                            }
                            Err(e) => log_error!("Failed to decode as u64: {}", e),
                        },
                        "u128" | "balance" => match u128::decode(&mut &value_bytes[..]) {
                            Ok(decoded_value) => {
                                log_success!(
                                    "Decoded Value: {}",
                                    decoded_value.to_string().bright_green()
                                )
                            }
                            Err(e) => log_error!("Failed to decode as u128: {}", e),
                        },
                        "accountid" | "accountid32" => {
                            match AccountId32::decode(&mut &value_bytes[..]) {
                                Ok(account_id) => log_success!(
                                    "Decoded Value: {}",
                                    account_id.to_ss58check().bright_green()
                                ),
                                Err(e) => log_error!("Failed to decode as AccountId32: {}", e),
                            }
                        }
                        _ => {
                            log_error!("Unsupported type for --decode-as: {}", type_str);
                            log_print!("Supported types: u64, moment, u128, balance, accountid");
                        }
                    }
                }
            } else {
                log_print!("{}", "No value found at this storage location.".dimmed());
            }
        }
        StorageCommands::Set {
            pallet,
            name,
            value,
            wallet,
            password,
            r#type,
        } => {
            log_print!(
                "✍️  Setting storage for {}::{}",
                pallet.bright_green(),
                name.bright_cyan()
            );
            log_print!("\n{}", "🛑 This is a SUDO operation!".bright_red().bold());

            // 1. Load wallet and create a signed API instance
            let keypair = crate::wallet::load_keypair_from_wallet(&wallet, password, None)?;
            log_verbose!("🔐 Using wallet: {}", wallet.bright_green());

            let client = ChainClient::new(node_url).await?;
            let api_with_signer = client.create_api_with_signer(&keypair)?;

            // 2. Encode the value based on the --type flag
            let value_bytes = match r#type.as_deref() {
                Some("u64") | Some("moment") => value
                    .parse::<u64>()
                    .map_err(|e| QuantusError::Generic(format!("Invalid u64 value: {}", e)))?
                    .encode(),
                Some("u128") | Some("balance") => value
                    .parse::<u128>()
                    .map_err(|e| QuantusError::Generic(format!("Invalid u128 value: {}", e)))?
                    .encode(),
                Some("accountid") | Some("accountid32") => AccountId32::from_ss58check(&value)
                    .map_err(|e| {
                        QuantusError::Generic(format!("Invalid AccountId value: {:?}", e))
                    })?
                    .encode(),
                None => {
                    // Default to hex decoding if no type is specified
                    let value_hex = value.strip_prefix("0x").unwrap_or(&value);
                    hex::decode(value_hex)
                        .map_err(|e| QuantusError::Generic(format!("Invalid hex value: {}", e)))?
                }
                Some(unsupported) => {
                    return Err(QuantusError::Generic(format!(
                        "Unsupported type for --type: {}",
                        unsupported
                    )))
                }
            };

            log_verbose!(
                "Encoded value bytes: 0x{}",
                hex::encode(&value_bytes).dimmed()
            );

            // 3. Construct the storage key
            let storage_key = {
                let mut key = twox_128(pallet.as_bytes()).to_vec();
                key.extend(&twox_128(name.as_bytes()));
                key
            };

            // 4. Compose the inner `System.set_storage` call
            let set_storage_call = compose_call!(
                api_with_signer.metadata(),
                "System",
                "set_storage",
                vec![(storage_key, value_bytes)]
            )
            .ok_or_else(|| QuantusError::Generic("Failed to compose call".to_string()))?;

            // 5. Compose the outer `Sudo.sudo` extrinsic with the inner call
            let sudo_extrinsic =
                compose_extrinsic!(&api_with_signer, "Sudo", "sudo", set_storage_call).ok_or_else(
                    || QuantusError::Generic("Failed to compose extrinsic".to_string()),
                )?;

            // 6. Submit the final extrinsic
            crate::submit_extrinsic_with_spinner!(&client, &keypair, sudo_extrinsic)?;
        }
    }
    Ok(())
}
