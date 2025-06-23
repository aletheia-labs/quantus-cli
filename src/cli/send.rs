use crate::{
    chain::client::ChainClient,
    error::Result,
    log_error, log_print, log_success, log_verbose,
    wallet::{QuantumKeyPair, WalletManager},
};
use colored::Colorize;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use tokio::time;

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
    password: Option<String>,
    password_file: Option<String>,
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

    // Get password securely for decryption (with convenience options)
    let wallet_password = get_wallet_password(&from_wallet, password, password_file)?;

    // Load and decrypt the sender wallet
    let wallet_manager = WalletManager::new()?;
    let wallet_data = wallet_manager.load_wallet(&from_wallet, &wallet_password)?;

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

    // Show spinner during the potentially slow network submission
    let mut spinner = ProgressSpinner::new();

    // Create a task that shows the spinner while waiting for network
    let spinner_handle = tokio::spawn(async move {
        let wait_duration = Duration::from_millis(200);
        loop {
            spinner.tick();
            time::sleep(wait_duration).await;
        }
    });

    // Submit transaction (this is where it might hang/take long)
    let tx_hash = chain_client.transfer(&keypair, &to_address, amount).await?;

    // Stop the spinner and clear the line
    spinner_handle.abort();
    print!("\r                                                    \r"); // Clear spinner line
    io::stdout().flush().unwrap();

    log_print!(
        "✅ {} Transaction submitted!",
        "SUCCESS".bright_green().bold()
    );
    log_print!("📍 Transaction hash: {}", tx_hash.bright_blue());

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

/// Get wallet password with convenience options
fn get_wallet_password(
    wallet_name: &str,
    password: Option<String>,
    password_file: Option<String>,
) -> Result<String> {
    // Option 1: Use CLI password flag if provided
    if let Some(pwd) = password {
        log_verbose!("🔑 Using password from --password flag");
        return Ok(pwd);
    }

    // Option 2: Read password from file if provided
    if let Some(file_path) = password_file {
        log_verbose!("🔑 Reading password from file: {}", file_path);
        let pwd = std::fs::read_to_string(&file_path)
            .map_err(|e| {
                crate::error::QuantusError::Generic(format!(
                    "Failed to read password file '{}': {}",
                    file_path, e
                ))
            })?
            .trim()
            .to_string();
        return Ok(pwd);
    }

    // Option 3: Check environment variable
    if let Ok(env_password) = std::env::var("QUANTUS_WALLET_PASSWORD") {
        log_verbose!("🔑 Using password from QUANTUS_WALLET_PASSWORD environment variable");
        return Ok(env_password);
    }

    // Option 4: Check for wallet-specific environment variable
    let wallet_env_var = format!("QUANTUS_WALLET_PASSWORD_{}", wallet_name.to_uppercase());
    if let Ok(env_password) = std::env::var(&wallet_env_var) {
        log_verbose!(
            "🔑 Using password from {} environment variable",
            wallet_env_var
        );
        return Ok(env_password);
    }

    // Option 5: Try empty password first (for development wallets)
    log_verbose!("🔑 Trying empty password first...");
    let wallet_manager = WalletManager::new()?;
    if let Ok(_) = wallet_manager.load_wallet(wallet_name, "") {
        log_verbose!("✅ Empty password works for wallet '{}'", wallet_name);
        return Ok("".to_string());
    }

    // Option 6: Prompt user for password
    get_password_from_user(&format!("Enter password for wallet '{}'", wallet_name))
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
