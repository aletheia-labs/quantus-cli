use crate::wallet::WalletManager;
use clap::Subcommand;
use colored::Colorize;

/// Wallet subcommands
#[derive(Subcommand, Debug)]
pub enum WalletCommands {
    /// Create a new wallet with quantum-safe keys
    Create {
        /// Wallet name
        #[arg(short, long)]
        name: String,

        /// Password to encrypt the wallet (optional, will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },

    /// View wallet information
    View {
        /// Wallet name to view
        #[arg(short, long)]
        name: Option<String>,

        /// Show all wallets if no name specified
        #[arg(short, long)]
        all: bool,
    },

    /// Export wallet (private key or mnemonic)
    Export {
        /// Wallet name to export
        #[arg(short, long)]
        name: String,

        /// Export format: mnemonic, private-key
        #[arg(short, long, default_value = "mnemonic")]
        format: String,
    },

    /// Import wallet from mnemonic phrase
    Import {
        /// Wallet name
        #[arg(short, long)]
        name: String,

        /// Mnemonic phrase (24 words, will prompt if not provided)
        #[arg(short, long)]
        mnemonic: Option<String>,

        /// Password to encrypt the wallet (optional, will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },

    /// List all wallets
    List,

    /// Delete a wallet
    Delete {
        /// Wallet name to delete
        #[arg(short, long)]
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

/// Handle wallet commands
pub async fn handle_wallet_command(
    command: WalletCommands,
    _node_url: &str,
) -> crate::error::Result<()> {
    match command {
        WalletCommands::Create { name, password } => {
            println!(
                "{}",
                "🔐 Creating new quantum wallet...".bright_blue().bold()
            );

            let wallet_manager = WalletManager::new()?;

            match wallet_manager
                .create_wallet(&name, password.as_deref())
                .await
            {
                Ok(wallet_info) => {
                    println!("Wallet name: {}", name.bright_green());
                    println!("Address: {}", wallet_info.address.bright_cyan());
                    println!("Key type: {}", wallet_info.key_type.bright_yellow());
                    println!(
                        "Created: {}",
                        wallet_info
                            .created_at
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                            .dimmed()
                    );
                    println!("{}", "✅ Wallet created successfully!".green());
                }
                Err(e) => {
                    println!("{}", format!("❌ Failed to create wallet: {}", e).red());
                    return Err(e);
                }
            }

            Ok(())
        }

        WalletCommands::View { name, all } => {
            println!(
                "{}",
                "👁️  Viewing wallet information...".bright_blue().bold()
            );

            let wallet_manager = WalletManager::new()?;

            if all {
                // Show all wallets (same as list command but with different header)
                match wallet_manager.list_wallets() {
                    Ok(wallets) => {
                        if wallets.is_empty() {
                            println!("{}", "No wallets found.".dimmed());
                        } else {
                            println!("All wallets ({}):\n", wallets.len());

                            for (i, wallet) in wallets.iter().enumerate() {
                                println!(
                                    "{}. {}",
                                    (i + 1).to_string().bright_yellow(),
                                    wallet.name.bright_green()
                                );
                                println!("   Address: {}", wallet.address.bright_cyan());
                                println!("   Type: {}", wallet.key_type.bright_yellow());
                                println!(
                                    "   Created: {}",
                                    wallet
                                        .created_at
                                        .format("%Y-%m-%d %H:%M:%S UTC")
                                        .to_string()
                                        .dimmed()
                                );
                                if i < wallets.len() - 1 {
                                    println!();
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}", format!("❌ Failed to view wallets: {}", e).red());
                        return Err(e);
                    }
                }
            } else if let Some(wallet_name) = name {
                // Show specific wallet details
                match wallet_manager.get_wallet(&wallet_name, None) {
                    Ok(Some(wallet_info)) => {
                        println!("Wallet Details:\n");
                        println!("Name: {}", wallet_info.name.bright_green());
                        println!("Address: {}", wallet_info.address.bright_cyan());
                        println!("Key Type: {}", wallet_info.key_type.bright_yellow());
                        println!(
                            "Created: {}",
                            wallet_info
                                .created_at
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string()
                                .dimmed()
                        );

                        if wallet_info.address.contains("[") {
                            println!(
                                "\n{}",
                                "💡 To see the full address, use the export command with password"
                                    .dimmed()
                            );
                        }
                    }
                    Ok(None) => {
                        println!("{}", format!("❌ Wallet '{}' not found", wallet_name).red());
                        println!(
                            "Use {} to see available wallets",
                            "quantus wallet list".bright_green()
                        );
                    }
                    Err(e) => {
                        println!("{}", format!("❌ Failed to view wallet: {}", e).red());
                        return Err(e);
                    }
                }
            } else {
                println!(
                    "{}",
                    "Please specify a wallet name with --name or use --all to show all wallets"
                        .yellow()
                );
                println!("Examples:");
                println!(
                    "  {}",
                    "quantus wallet view --name my-wallet".bright_green()
                );
                println!("  {}", "quantus wallet view --all".bright_green());
            }

            Ok(())
        }

        WalletCommands::Export { name, format } => {
            println!("{}", "📤 Exporting wallet...".bright_blue().bold());
            println!("Wallet: {}", name.bright_green());
            println!("Format: {}", format.bright_yellow());
            println!("{}", "✅ Export completed! (STUB)".green());
            Ok(())
        }

        WalletCommands::Import {
            name,
            mnemonic,
            password,
        } => {
            println!("{}", "📥 Importing wallet...".bright_blue().bold());

            let wallet_manager = WalletManager::new()?;

            // Get mnemonic from user if not provided
            let mnemonic_phrase = if let Some(mnemonic) = mnemonic {
                mnemonic
            } else {
                println!("Please enter your 24-word mnemonic phrase:");
                // For now, we'll require the mnemonic to be provided via command line
                // In a real implementation, you'd want to prompt securely for this
                return Err(crate::error::QuantusError::Generic(
                    "Mnemonic phrase is required. Please provide it with --mnemonic flag."
                        .to_string(),
                )
                .into());
            };

            match wallet_manager
                .import_wallet(&name, &mnemonic_phrase, password.as_deref())
                .await
            {
                Ok(wallet_info) => {
                    println!("Wallet name: {}", name.bright_green());
                    println!("Address: {}", wallet_info.address.bright_cyan());
                    println!("Key type: {}", wallet_info.key_type.bright_yellow());
                    println!(
                        "Imported: {}",
                        wallet_info
                            .created_at
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                            .dimmed()
                    );
                    println!("{}", "✅ Wallet imported successfully!".green());
                }
                Err(e) => {
                    println!("{}", format!("❌ Failed to import wallet: {}", e).red());
                    return Err(e);
                }
            }

            Ok(())
        }

        WalletCommands::List => {
            println!("{}", "📋 Listing all wallets...".bright_blue().bold());

            let wallet_manager = WalletManager::new()?;

            match wallet_manager.list_wallets() {
                Ok(wallets) => {
                    if wallets.is_empty() {
                        println!("{}", "No wallets found.".dimmed());
                        println!(
                            "Create a new wallet with: {}",
                            "quantus wallet create --name <name>".bright_green()
                        );
                    } else {
                        println!("Found {} wallet(s):\n", wallets.len());

                        for (i, wallet) in wallets.iter().enumerate() {
                            println!(
                                "{}. {}",
                                (i + 1).to_string().bright_yellow(),
                                wallet.name.bright_green()
                            );
                            println!("   Address: {}", wallet.address.bright_cyan());
                            println!("   Type: {}", wallet.key_type.bright_yellow());
                            println!(
                                "   Created: {}",
                                wallet
                                    .created_at
                                    .format("%Y-%m-%d %H:%M:%S UTC")
                                    .to_string()
                                    .dimmed()
                            );
                            if i < wallets.len() - 1 {
                                println!();
                            }
                        }

                        println!(
                            "\n{}",
                            "💡 Use 'quantus wallet view --name <wallet>' to see full details"
                                .dimmed()
                        );
                    }
                }
                Err(e) => {
                    println!("{}", format!("❌ Failed to list wallets: {}", e).red());
                    return Err(e);
                }
            }

            Ok(())
        }

        WalletCommands::Delete { name, force } => {
            println!("{}", "🗑️  Deleting wallet...".bright_red().bold());
            println!("Wallet: {}", name.bright_green());
            println!(
                "Force: {}",
                if force { "Yes" } else { "No" }.bright_yellow()
            );
            println!("{}", "✅ Wallet deleted! (STUB)".green());
            Ok(())
        }
    }
}
