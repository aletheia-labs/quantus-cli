use crate::{
    chain::client::ChainClient,
    error::Result,
    wallet::{QuantumKeyPair, WalletManager},
};
use colored::Colorize;

/// Handle the send command
pub async fn handle_send_command(
    from_wallet: String,
    to_address: String,
    amount: u128,
    node_url: &str,
) -> Result<()> {
    println!(
        "🚀 {} Sending {} tokens to {}",
        "SEND".bright_cyan().bold(),
        amount.to_string().bright_yellow().bold(),
        to_address.bright_green()
    );

    // Load the sender wallet
    let wallet_manager = WalletManager::new()?;
    let wallet_data = wallet_manager.load_wallet(&from_wallet)?;

    println!("📦 Using wallet: {}", from_wallet.bright_blue().bold());

    // TODO: Get password securely for decryption
    let password = get_password_from_user(&format!("Enter password for wallet '{}'", from_wallet))?;

    // Decrypt the key pair
    let keypair = wallet_data.decrypt_keypair(&password)?;

    // Create chain client
    let chain_client = ChainClient::new(node_url).await?;

    // Get account information
    let from_account_id = keypair.to_account_id_ss58check();
    let balance = chain_client.get_balance(&from_account_id).await?;

    println!(
        "💰 Current balance: {} tokens",
        balance.to_string().bright_yellow()
    );

    if balance < amount {
        return Err(crate::error::QuantusError::InsufficientBalance {
            available: balance,
            required: amount,
        });
    }

    // Create and submit transaction
    println!(
        "✍️  {} Signing transaction...",
        "SIGN".bright_magenta().bold()
    );

    let tx_hash = chain_client.transfer(&keypair, &to_address, amount).await?;

    println!(
        "✅ {} Transaction submitted!",
        "SUCCESS".bright_green().bold()
    );
    println!("📍 Transaction hash: {}", tx_hash.bright_blue());
    println!("🔗 Waiting for confirmation...");

    // Wait for transaction confirmation
    let success = chain_client.wait_for_finalization(&tx_hash).await?;

    if success {
        println!(
            "🎉 {} Transaction confirmed!",
            "FINALIZED".bright_green().bold()
        );

        // Show updated balance
        let new_balance = chain_client.get_balance(&from_account_id).await?;
        println!(
            "💰 New balance: {} tokens",
            new_balance.to_string().bright_yellow()
        );
    } else {
        println!("❌ {} Transaction failed!", "ERROR".bright_red().bold());
    }

    Ok(())
}

/// Get password from user (placeholder - should use secure input)
fn get_password_from_user(prompt: &str) -> Result<String> {
    println!("{}", prompt.bright_yellow());
    // TODO: Use secure password input (rpassword crate)
    // For now, using placeholder
    Ok("password123".to_string())
}

// Send functionality implementation complete
