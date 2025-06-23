use crate::{
    chain::client::ChainClient,
    error::Result,
    log_error, log_print, log_success, log_verbose,
    wallet::{QuantumKeyPair, WalletManager},
};
use colored::Colorize;
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Simple progress spinner for showing waiting status
struct ProgressSpinner {
    chars: Vec<char>,
    current: usize,
    start_time: Instant,
}

impl ProgressSpinner {
    fn new() -> Self {
        Self {
            chars: vec!['|', '/', '-', '\\'],
            current: 0,
            start_time: Instant::now(),
        }
    }

    fn tick(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs();
        print!(
            "\r🔗 Waiting for confirmation... {} ({}s)",
            self.chars[self.current].to_string().bright_blue(),
            elapsed
        );
        io::stdout().flush().unwrap();
        self.current = (self.current + 1) % self.chars.len();
    }

    fn finish(&self, message: &str) {
        print!("\r{}\n", message);
        io::stdout().flush().unwrap();
    }
}

/// Handle the send command
pub async fn handle_send_command(
    from_wallet: String,
    to_address: String,
    amount_str: &str,
    node_url: &str,
) -> Result<()> {
    // Create chain client early to get formatting
    let chain_client = ChainClient::new(node_url).await?;

    // Parse and validate the amount
    let (amount, formatted_amount) = chain_client.validate_and_format_amount(amount_str).await?;
    log_verbose!(
        "🚀 {} Sending {} to {}",
        "SEND".bright_cyan().bold(),
        formatted_amount.bright_yellow().bold(),
        to_address.bright_green()
    );

    // Get password securely for decryption
    let password = get_password_from_user(&format!("Enter password for wallet '{}'", from_wallet))?;

    // Load and decrypt the sender wallet
    let wallet_manager = WalletManager::new()?;
    let wallet_data = wallet_manager.load_wallet(&from_wallet, &password)?;

    log_verbose!("📦 Using wallet: {}", from_wallet.bright_blue().bold());

    // Get the keypair (already decrypted)
    let keypair = &wallet_data.keypair;

    // Chain client already created above

    // Get account information
    let from_account_id = keypair.to_account_id_ss58check();
    let balance = chain_client.get_balance(&from_account_id).await?;

    // Get formatted balance with proper decimals
    let formatted_balance = chain_client.format_balance_with_symbol(balance).await?;
    log_verbose!("💰 Current balance: {}", formatted_balance.bright_yellow());

    if balance < amount {
        return Err(crate::error::QuantusError::InsufficientBalance {
            available: balance,
            required: amount,
        });
    }

    // Create and submit transaction
    log_verbose!(
        "✍️  {} Signing transaction...",
        "SIGN".bright_magenta().bold()
    );

    let tx_hash = chain_client.transfer(&keypair, &to_address, amount).await?;

    log_print!(
        "✅ {} Transaction submitted!",
        "SUCCESS".bright_green().bold()
    );
    log_print!("📍 Transaction hash: {}", tx_hash.bright_blue());
    log_verbose!("🔗 Waiting for confirmation...");

    // Wait for transaction confirmation
    let spinner = ProgressSpinner::new();
    let success = chain_client.wait_for_finalization(&tx_hash).await?;

    if success {
        log_success!(
            "🎉 {} Transaction confirmed!",
            "FINALIZED".bright_green().bold()
        );

        // Show updated balance with proper formatting
        let new_balance = chain_client.get_balance(&from_account_id).await?;
        let formatted_new_balance = chain_client.format_balance_with_symbol(new_balance).await?;

        // Calculate and display transaction fee in verbose mode
        let fee_paid = balance.saturating_sub(new_balance).saturating_sub(amount);
        if fee_paid > 0 {
            let formatted_fee = chain_client.format_balance_with_symbol(fee_paid).await?;
            log_verbose!("💸 Transaction fee: {}", formatted_fee.bright_cyan());
        }

        log_print!("💰 New balance: {}", formatted_new_balance.bright_yellow());
    } else {
        log_error!("Transaction failed!");
    }

    Ok(())
}

/// Get password from user securely
fn get_password_from_user(prompt: &str) -> Result<String> {
    log_print!("{}", prompt.bright_yellow());
    let password = rpassword::read_password().map_err(|e| {
        crate::error::QuantusError::Generic(format!("Failed to read password: {}", e))
    })?;
    Ok(password)
}

// Send functionality implementation complete
