//! `quantus storage` subcommand
use crate::{chain::client::ChainClient, error::QuantusError, log_print, log_success, log_verbose};
use clap::Subcommand;
use colored::Colorize;
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

        /// The new value, as a hex-encoded string (e.g., "0xdeadbeef")
        #[arg(long)]
        value: String,

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
        StorageCommands::Get { pallet, name } => {
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

            if let Some(value) = result {
                log_success!("Value: 0x{}", hex::encode(value).bright_yellow());
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

            // 2. Decode hex value
            let value_hex = value.strip_prefix("0x").unwrap_or(&value);
            let value_bytes = hex::decode(value_hex)
                .map_err(|e| QuantusError::Generic(format!("Invalid hex value: {}", e)))?;

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
