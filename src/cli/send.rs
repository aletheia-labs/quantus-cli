use crate::{
    chain::client::ChainClient, error::Result, log_error, log_print, log_success, log_verbose,
};
use colored::Colorize;

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
    log_verbose!("📦 Using wallet: {}", from_wallet.bright_blue().bold());
    let keypair = crate::wallet::load_keypair_from_wallet(&from_wallet, password, password_file)?;

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

    // Submit transaction (this is where it might hang/take long)
    let tx_hash = chain_client.transfer(&keypair, &to_address, amount).await?;

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
