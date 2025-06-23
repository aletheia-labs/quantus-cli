/*!
 * Quantus CLI - Command line interface for the Quantus Network
 *
 * A modern, user-friendly CLI for interacting with the Quantus blockchain,
 * featuring built-in wallet management and simplified chain operations.
 */

use clap::Parser;
use colored::Colorize;

mod chain;
mod cli;
mod config;
mod error;
mod wallet;

use cli::Commands;
use error::QuantusError;

#[derive(Parser)]
#[command(name = "quantus")]
#[command(author = "Quantus Network")]
#[command(version = "0.1.0")]
#[command(about = "Command line interface for the Quantus Network", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Node endpoint URL
    #[arg(long, global = true, default_value = "ws://127.0.0.1:9944")]
    node_url: String,
}

#[tokio::main]
async fn main() -> Result<(), QuantusError> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Print welcome message
    println!("{}", "🔮 Quantus CLI".bright_cyan().bold());
    println!("{}", "Connecting to the quantum future...".dimmed());
    println!();

    // Execute the command
    match cli::execute_command(cli.command, &cli.node_url).await {
        Ok(_) => {
            println!();
            println!("{}", "✅ Command executed successfully!".green());
            Ok(())
        }
        Err(e) => {
            eprintln!();
            eprintln!("{} {}", "❌ Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
