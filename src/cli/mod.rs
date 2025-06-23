use clap::Subcommand;

pub mod wallet;

/// Main CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Wallet management commands
    #[command(subcommand)]
    Wallet(wallet::WalletCommands),

    /// Show version information
    Version,
}

/// Execute a CLI command
pub async fn execute_command(command: Commands, node_url: &str) -> crate::error::Result<()> {
    match command {
        Commands::Wallet(wallet_cmd) => wallet::handle_wallet_command(wallet_cmd, node_url).await,
        Commands::Version => {
            println!("Quantus CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Build: {}", env!("CARGO_PKG_DESCRIPTION"));
            Ok(())
        }
    }
}
