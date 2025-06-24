use crate::chain::client::ChainClient;
use crate::chain::quantus_runtime_config::QuantusRuntimeConfig;
use crate::error::Result;
use crate::{log_error, log_print, log_success, log_verbose};
use clap::Subcommand;
use colored::Colorize;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58Codec;
use sp_runtime::MultiAddress;
use substrate_api_client::ac_compose_macros::compose_extrinsic;
use substrate_api_client::ac_primitives::ExtrinsicSigner;
use substrate_api_client::{SubmitAndWatch, XtStatus};

/// Reversible transfer commands
#[derive(Subcommand, Debug)]
pub enum ReversibleCommands {
    /// Schedule a transfer with default delay
    ScheduleTransfer {
        /// The recipient's account address
        #[arg(short, long)]
        to: String,

        /// Amount to transfer (e.g., "10", "10.5", "0.0001")
        #[arg(short, long)]
        amount: String,

        /// Wallet name to send from
        #[arg(short, long)]
        from: String,

        /// Password for the wallet
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Schedule a transfer with custom delay
    ScheduleTransferWithDelay {
        /// The recipient's account address
        #[arg(short, long)]
        to: String,

        /// Amount to transfer (e.g., "10", "10.5", "0.0001")
        #[arg(short, long)]
        amount: String,

        /// Delay in blocks or milliseconds
        #[arg(short, long)]
        delay: u64,

        /// Wallet name to send from
        #[arg(short, long)]
        from: String,

        /// Password for the wallet
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Cancel a pending reversible transaction
    Cancel {
        /// Transaction ID to cancel (hex hash)
        #[arg(long)]
        tx_id: String,

        /// Wallet name to sign with
        #[arg(short, long)]
        from: String,

        /// Password for the wallet
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Set reversibility for your account
    SetReversibility {
        /// Delay in blocks or milliseconds (None to disable)
        #[arg(short, long)]
        delay: Option<u64>,

        /// Policy: "BlockDelay" or "TimeDelay"
        #[arg(short, long, default_value = "TimeDelay")]
        policy: String,

        /// Optional reverser account (defaults to self)
        #[arg(long)]
        reverser: Option<String>,

        /// Wallet name to sign with
        #[arg(short, long)]
        from: String,

        /// Password for the wallet
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Execute a pending transfer (called by scheduler)
    ExecuteTransfer {
        /// Transaction ID to execute (hex hash)
        #[arg(long)]
        tx_id: String,

        /// Wallet name to sign with
        #[arg(short, long)]
        from: String,

        /// Password for the wallet
        #[arg(short, long)]
        password: Option<String>,
    },
}

/// Handle reversible transfer commands
pub async fn handle_reversible_command(command: ReversibleCommands, node_url: &str) -> Result<()> {
    log_print!("🔄 Reversible Transfers");

    let chain_client = ChainClient::new(node_url).await?;

    match command {
        ReversibleCommands::ScheduleTransfer {
            to,
            amount,
            from,
            password,
        } => schedule_transfer(&chain_client, &to, &amount, &from, password).await,
        ReversibleCommands::ScheduleTransferWithDelay {
            to,
            amount,
            delay,
            from,
            password,
        } => {
            schedule_transfer_with_delay(&chain_client, &to, &amount, delay, &from, password).await
        }
        ReversibleCommands::Cancel {
            tx_id,
            from,
            password,
        } => cancel_transaction(&chain_client, &tx_id, &from, password).await,
        ReversibleCommands::SetReversibility {
            delay,
            policy,
            reverser,
            from,
            password,
        } => set_reversibility(&chain_client, delay, &policy, reverser, &from, password).await,
        ReversibleCommands::ExecuteTransfer {
            tx_id,
            from,
            password,
        } => execute_transfer(&chain_client, &tx_id, &from, password).await,
    }
}

/// Schedule a transfer with default delay
async fn schedule_transfer(
    chain_client: &ChainClient,
    to: &str,
    amount: &str,
    from: &str,
    password: Option<String>,
) -> Result<()> {
    log_print!("📅 Scheduling transfer");
    log_print!("To: {}", to.bright_green());
    log_print!("Amount: {}", amount.bright_cyan());
    log_print!("From: {}", from.bright_yellow());

    let keypair = crate::wallet::load_keypair_from_wallet(&from, password, None)?;

    // Parse destination address
    let dest_account = AccountId32::from_ss58check(to).map_err(|e| {
        crate::error::QuantusError::Generic(format!("Invalid destination address: {:?}", e))
    })?;
    let dest: MultiAddress<AccountId32, u32> = MultiAddress::Id(dest_account);

    // Parse amount
    let amount_value = chain_client.parse_amount(amount).await?;
    log_verbose!("✅ Parsed amount: {} raw units", amount_value);

    // Create API with signer
    let api_with_signer = chain_client.create_api_with_signer(&keypair)?;

    // Create extrinsic
    let extrinsic = compose_extrinsic!(
        &api_with_signer,
        "ReversibleTransfers",
        "schedule_transfer",
        dest,
        amount_value
    )
    .ok_or_else(|| {
        crate::error::QuantusError::Generic(
            "Failed to create schedule_transfer extrinsic".to_string(),
        )
    })?;

    // Submit extrinsic with spinner
    let tx_hash = crate::submit_extrinsic_with_spinner!(chain_client, keypair, extrinsic)?;

    log_success!("🎉 Transfer scheduled successfully!");
    log_print!("📋 Transaction hash: {}", tx_hash.bright_yellow());
    log_print!("💡 Transfer will execute after the configured delay period");

    Ok(())
}

/// Schedule a transfer with custom delay
async fn schedule_transfer_with_delay(
    chain_client: &ChainClient,
    to: &str,
    amount: &str,
    delay: u64,
    from: &str,
    password: Option<String>,
) -> Result<()> {
    log_print!("📅 Scheduling transfer with custom delay");
    log_print!("To: {}", to.bright_green());
    log_print!("Amount: {}", amount.bright_cyan());
    log_print!("Delay: {} blocks/ms", delay.to_string().bright_magenta());
    log_print!("From: {}", from.bright_yellow());

    let keypair = crate::wallet::load_keypair_from_wallet(&from, password, None)?;

    // Parse destination address
    let dest_account = AccountId32::from_ss58check(to).map_err(|e| {
        crate::error::QuantusError::Generic(format!("Invalid destination address: {:?}", e))
    })?;
    let dest: MultiAddress<AccountId32, u32> = MultiAddress::Id(dest_account);

    // Parse amount
    let amount_value = chain_client.parse_amount(amount).await?;
    log_verbose!("✅ Parsed amount: {} raw units", amount_value);

    // Create API with signer
    let api_with_signer = chain_client.create_api_with_signer(&keypair)?;

    // Create extrinsic
    let extrinsic = compose_extrinsic!(
        &api_with_signer,
        "ReversibleTransfers",
        "schedule_transfer_with_delay",
        dest,
        amount_value,
        delay
    )
    .ok_or_else(|| {
        crate::error::QuantusError::Generic(
            "Failed to create schedule_transfer_with_delay extrinsic".to_string(),
        )
    })?;

    // Submit extrinsic with spinner
    let tx_hash = crate::submit_extrinsic_with_spinner!(chain_client, keypair, extrinsic)?;

    log_success!("🎉 Transfer with custom delay scheduled successfully!");
    log_print!("📋 Transaction hash: {}", tx_hash.bright_yellow());
    log_print!("⏰ Transfer will execute after {} blocks/ms", delay);

    Ok(())
}

/// Cancel a pending reversible transaction
async fn cancel_transaction(
    chain_client: &ChainClient,
    tx_id: &str,
    from: &str,
    password: Option<String>,
) -> Result<()> {
    log_print!("❌ Canceling transaction");
    log_print!("Transaction ID: {}", tx_id.bright_red());
    log_print!("From: {}", from.bright_yellow());

    let keypair = crate::wallet::load_keypair_from_wallet(&from, password, None)?;

    // Parse transaction ID (hex hash)
    let tx_hash = if tx_id.starts_with("0x") {
        hex::decode(&tx_id[2..]).map_err(|e| {
            crate::error::QuantusError::Generic(format!("Invalid hex transaction ID: {:?}", e))
        })?
    } else {
        hex::decode(tx_id).map_err(|e| {
            crate::error::QuantusError::Generic(format!("Invalid hex transaction ID: {:?}", e))
        })?
    };

    if tx_hash.len() != 32 {
        return Err(crate::error::QuantusError::Generic(
            "Transaction ID must be 32 bytes (64 hex characters)".to_string(),
        )
        .into());
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&tx_hash);

    // Create API with signer
    let api_with_signer = chain_client.create_api_with_signer(&keypair)?;

    // Create extrinsic
    let extrinsic = compose_extrinsic!(
        &api_with_signer,
        "ReversibleTransfers",
        "cancel",
        hash_array
    )
    .ok_or_else(|| {
        crate::error::QuantusError::Generic("Failed to create cancel extrinsic".to_string())
    })?;

    // Submit extrinsic with spinner
    let result_hash = crate::submit_extrinsic_with_spinner!(chain_client, keypair, extrinsic)?;

    log_success!("🎉 Transaction canceled successfully!");
    log_print!("📋 Cancellation hash: {}", result_hash.bright_yellow());

    Ok(())
}

/// Set reversibility for an account
async fn set_reversibility(
    chain_client: &ChainClient,
    delay: Option<u64>,
    policy: &str,
    reverser: Option<String>,
    from: &str,
    password: Option<String>,
) -> Result<()> {
    log_print!("⚙️  Setting reversibility");
    log_print!("Delay: {:?}", delay);
    log_print!("Policy: {}", policy.bright_cyan());
    log_print!("From: {}", from.bright_yellow());

    let keypair = crate::wallet::load_keypair_from_wallet(&from, password, None)?;

    // Parse reverser account if provided
    let reverser_account = if let Some(reverser_addr) = reverser {
        Some(AccountId32::from_ss58check(&reverser_addr).map_err(|e| {
            crate::error::QuantusError::Generic(format!("Invalid reverser address: {:?}", e))
        })?)
    } else {
        None
    };

    // Create API with signer
    let api_with_signer = chain_client.create_api_with_signer(&keypair)?;

    // For now, we'll use a simplified approach since the exact DelayPolicy enum structure isn't clear
    // This will need to be adjusted based on the actual runtime types
    let extrinsic = compose_extrinsic!(
        &api_with_signer,
        "ReversibleTransfers",
        "set_reversibility",
        delay,
        policy,
        reverser_account
    )
    .ok_or_else(|| {
        crate::error::QuantusError::Generic(
            "Failed to create set_reversibility extrinsic".to_string(),
        )
    })?;

    // Submit extrinsic with spinner
    let tx_hash = crate::submit_extrinsic_with_spinner!(chain_client, keypair, extrinsic)?;

    log_success!("🎉 Reversibility settings updated successfully!");
    log_print!("📋 Transaction hash: {}", tx_hash.bright_yellow());

    Ok(())
}

/// Execute a pending transfer (usually called by scheduler)
async fn execute_transfer(
    chain_client: &ChainClient,
    tx_id: &str,
    from: &str,
    password: Option<String>,
) -> Result<()> {
    log_print!("▶️  Executing transfer");
    log_print!("Transaction ID: {}", tx_id.bright_green());
    log_print!("From: {}", from.bright_yellow());

    let keypair = crate::wallet::load_keypair_from_wallet(&from, password, None)?;

    // Parse transaction ID (hex hash)
    let tx_hash = if tx_id.starts_with("0x") {
        hex::decode(&tx_id[2..]).map_err(|e| {
            crate::error::QuantusError::Generic(format!("Invalid hex transaction ID: {:?}", e))
        })?
    } else {
        hex::decode(tx_id).map_err(|e| {
            crate::error::QuantusError::Generic(format!("Invalid hex transaction ID: {:?}", e))
        })?
    };

    if tx_hash.len() != 32 {
        return Err(crate::error::QuantusError::Generic(
            "Transaction ID must be 32 bytes (64 hex characters)".to_string(),
        )
        .into());
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&tx_hash);

    // Create API with signer
    let api_with_signer = chain_client.create_api_with_signer(&keypair)?;

    // Create extrinsic
    let extrinsic = compose_extrinsic!(
        &api_with_signer,
        "ReversibleTransfers",
        "execute_transfer",
        hash_array
    )
    .ok_or_else(|| {
        crate::error::QuantusError::Generic(
            "Failed to create execute_transfer extrinsic".to_string(),
        )
    })?;

    // Submit extrinsic with spinner
    let result_hash = crate::submit_extrinsic_with_spinner!(chain_client, keypair, extrinsic)?;

    log_success!("🎉 Transfer executed successfully!");
    log_print!("📋 Execution hash: {}", result_hash.bright_yellow());

    Ok(())
}
