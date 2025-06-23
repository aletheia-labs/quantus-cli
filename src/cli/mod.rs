use crate::{log_error, log_print, log_success, log_verbose};
use clap::Subcommand;

pub mod send;
pub mod wallet;

/// Main CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Wallet management commands
    #[command(subcommand)]
    Wallet(wallet::WalletCommands),

    /// Send tokens to another account
    Send {
        /// The recipient's account address
        #[arg(short, long)]
        to: String,

        /// Amount to send (in the smallest unit)
        #[arg(short, long)]
        amount: u128,

        /// Wallet name to send from
        #[arg(short, long)]
        from: String,
    },

    /// Query account balance
    Balance {
        /// Account address to query (SS58 format)
        #[arg(short, long)]
        address: String,
    },

    /// Developer utilities and testing tools
    #[command(subcommand)]
    Developer(DeveloperCommands),

    /// Query system information
    System,

    /// Show version information
    Version,
}

/// Developer subcommands
#[derive(Subcommand, Debug)]
pub enum DeveloperCommands {
    /// Create standard test wallets (crystal_alice, crystal_bob, crystal_charlie)
    CreateTestWallets,
}

/// Execute a CLI command
pub async fn execute_command(command: Commands, node_url: &str) -> crate::error::Result<()> {
    match command {
        Commands::Wallet(wallet_cmd) => wallet::handle_wallet_command(wallet_cmd, node_url).await,
        Commands::Send { to, amount, from } => {
            send::handle_send_command(from, to, amount, node_url).await
        }
        Commands::Balance { address } => handle_balance_command(address, node_url).await,
        Commands::Developer(dev_cmd) => handle_developer_command(dev_cmd, node_url).await,
        Commands::System => handle_system_command(node_url).await,
        Commands::Version => {
            log_print!("Quantus CLI v{}", env!("CARGO_PKG_VERSION"));
            log_print!("Build: {}", env!("CARGO_PKG_DESCRIPTION"));
            Ok(())
        }
    }
}

/// Handle the balance query command
async fn handle_balance_command(address: String, node_url: &str) -> crate::error::Result<()> {
    use crate::chain::client::ChainClient;
    use colored::Colorize;

    log_verbose!(
        "💰 {} Querying balance for address",
        "BALANCE".bright_cyan().bold()
    );
    log_verbose!("🔍 Address: {}", address.bright_green());

    // Create chain client
    let client = ChainClient::new(node_url).await?;

    // Query balance
    let balance = client.get_balance(&address).await?;

    log_print!(
        "✅ Balance: {} tokens",
        balance.to_string().bright_yellow().bold()
    );

    Ok(())
}

/// Handle the system info command
async fn handle_system_command(node_url: &str) -> crate::error::Result<()> {
    use crate::chain::client::ChainClient;
    use colored::Colorize;

    log_verbose!(
        "🔍 {} Querying system information",
        "SYSTEM".bright_cyan().bold()
    );

    // Create chain client
    let client = ChainClient::new(node_url).await?;

    // Query system info
    client.get_system_info().await?;

    Ok(())
}

/// Handle developer commands
async fn handle_developer_command(
    command: DeveloperCommands,
    _node_url: &str,
) -> crate::error::Result<()> {
    match command {
        DeveloperCommands::CreateTestWallets => {
            use crate::wallet::WalletManager;
            use colored::Colorize;

            log_print!(
                "🧪 {} Creating standard test wallets...",
                "DEVELOPER".bright_magenta().bold()
            );
            log_print!("");

            let wallet_manager = WalletManager::new()?;

            // Standard test wallets with well-known names
            let test_wallets = vec![
                ("crystal_alice", "Alice's test wallet for development"),
                ("crystal_bob", "Bob's test wallet for development"),
                ("crystal_charlie", "Charlie's test wallet for development"),
            ];

            let mut created_count = 0;
            let mut skipped_count = 0;

            for (name, description) in test_wallets {
                log_verbose!("Creating wallet: {}", name.bright_green());

                // Create wallet with a default password for testing
                match wallet_manager.create_developer_wallet(name).await {
                    Ok(wallet_info) => {
                        log_success!("✅ Created {}", name.bright_green());
                        log_success!("   Address: {}", wallet_info.address.bright_cyan());
                        log_success!("   Description: {}", description.dimmed());
                        created_count += 1;
                    }
                    Err(e) => {
                        log_error!("❌ Failed to create {}: {}", name.bright_red(), e);
                    }
                }
            }

            log_print!("");
            log_success!("🎉 Test wallet creation complete!");
            log_success!(
                "   Created: {} wallets",
                created_count.to_string().bright_green()
            );
            if skipped_count > 0 {
                log_success!(
                    "   Skipped: {} existing wallets",
                    skipped_count.to_string().bright_yellow()
                );
            }

            log_print!("");
            log_print!(
                "💡 {} You can now use these wallets:",
                "TIP".bright_blue().bold()
            );
            log_print!("   quantus send --from crystal_alice --to <address> --amount 1000");
            log_print!("   quantus send --from crystal_bob --to <address> --amount 1000");
            log_print!("   quantus send --from crystal_charlie --to <address> --amount 1000");
            log_print!("");
            log_print!(
                "🔑 {} All test wallets use password: {}",
                "NOTE".bright_blue().bold(),
                "test123".bright_yellow()
            );

            Ok(())
        }
    }
}
