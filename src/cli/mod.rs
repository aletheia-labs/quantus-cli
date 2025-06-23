use crate::{log_print, log_success, log_verbose};
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

    /// Query system information
    System,

    /// Show version information
    Version,
}

/// Execute a CLI command
pub async fn execute_command(command: Commands, node_url: &str) -> crate::error::Result<()> {
    match command {
        Commands::Wallet(wallet_cmd) => wallet::handle_wallet_command(wallet_cmd, node_url).await,
        Commands::Send { to, amount, from } => {
            send::handle_send_command(from, to, amount, node_url).await
        }
        Commands::Balance { address } => handle_balance_command(address, node_url).await,
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
