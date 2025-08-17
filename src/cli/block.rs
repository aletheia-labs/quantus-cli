//! `quantus block` subcommand - detailed block analysis
use crate::{
	chain::client::QuantusClient, cli::storage, error::QuantusError, log_error, log_print,
	log_success,
};
use colored::Colorize;

/// Handle detailed block analysis
pub async fn handle_block_command(
	number: Option<u32>,
	hash: Option<String>,
	latest: bool,
	storage: bool,
	extrinsics: bool,
	events: bool,
	all: bool,
	node_url: &str,
) -> crate::error::Result<()> {
	log_print!("🔍 Block Analysis");

	let quantus_client = QuantusClient::new(node_url).await?;

	// Determine which block to analyze
	let (block_number, block_hash) = if let Some(num) = number {
		// Convert number to hash using our storage function
		let hash = storage::resolve_block_hash(&quantus_client, &num.to_string()).await?;
		(num, hash)
	} else if let Some(h) = hash {
		// Parse hash and get block number from storage
		let parsed_hash = storage::resolve_block_hash(&quantus_client, &h).await?;
		// Get block number by querying System::Number at that block
		let storage_at = quantus_client.client().storage().at(parsed_hash);
		let number_addr = crate::chain::quantus_subxt::api::storage().system().number();
		let block_num = storage_at.fetch_or_default(&number_addr).await.map_err(|e| {
			QuantusError::NetworkError(format!("Failed to get block number: {:?}", e))
		})?;
		(block_num, parsed_hash)
	} else if latest {
		// Use latest block
		let hash = quantus_client.get_latest_block().await?;
		let storage_at = quantus_client.client().storage().at(hash);
		let number_addr = crate::chain::quantus_subxt::api::storage().system().number();
		let block_num = storage_at.fetch_or_default(&number_addr).await.map_err(|e| {
			QuantusError::NetworkError(format!("Failed to get latest block number: {:?}", e))
		})?;
		(block_num, hash)
	} else {
		return Err(QuantusError::Generic("Must specify --number, --hash, or --latest".to_string()));
	};

	log_print!("📦 Block #{} - {:#x}", block_number, block_hash);
	log_print!("");

	// Get basic block information using RPC
	use jsonrpsee::core::client::ClientT;
	let block_data: serde_json::Value = quantus_client
		.rpc_client()
		.request("chain_getBlock", [format!("{:#x}", block_hash)])
		.await
		.map_err(|e| QuantusError::NetworkError(format!("Failed to get block data: {:?}", e)))?;

	// Show basic block header information
	show_block_header(&block_data)?;

	// Show storage statistics if requested or --all
	if storage || all {
		show_storage_statistics(&quantus_client, block_hash).await?;
	}

	// Show events if requested or --all
	if events || all {
		show_block_events(block_number, node_url).await?;
	}

	// Show extrinsic details if requested or --all
	if extrinsics || all {
		show_extrinsic_details(&block_data)?;
	}

	log_success!("✅ Block analysis complete!");
	log_print!("💡 Use --all to see all information, or combine --storage --events --extrinsics");

	Ok(())
}

/// Show block header information
fn show_block_header(block_data: &serde_json::Value) -> crate::error::Result<()> {
	if let Some(block) = block_data.get("block") {
		if let Some(header) = block.get("header") {
			log_print!("📋 Block Header:");
			if let Some(parent_hash) = header.get("parentHash") {
				log_print!(
					"   • Parent Hash: {}",
					parent_hash.as_str().unwrap_or("unknown").bright_blue()
				);
			}
			if let Some(state_root) = header.get("stateRoot") {
				log_print!(
					"   • State Root: {}",
					state_root.as_str().unwrap_or("unknown").bright_green()
				);
			}
			if let Some(extrinsics_root) = header.get("extrinsicsRoot") {
				log_print!(
					"   • Extrinsics Root: {}",
					extrinsics_root.as_str().unwrap_or("unknown").bright_yellow()
				);
			}
		}

		if let Some(extrinsics) = block.get("extrinsics") {
			if let Some(extrinsics_array) = extrinsics.as_array() {
				log_print!(
					"   • Extrinsics Count: {}",
					extrinsics_array.len().to_string().bright_magenta()
				);

				// Calculate approximate block size
				let block_size = serde_json::to_string(block_data).unwrap_or_default().len();
				log_print!("   • Approximate Size: {} bytes", block_size.to_string().bright_cyan());
			}
		}
	}

	log_print!("");
	Ok(())
}

/// Show storage statistics for the block
async fn show_storage_statistics(
	quantus_client: &QuantusClient,
	block_hash: subxt::utils::H256,
) -> crate::error::Result<()> {
	log_print!("💾 Storage Statistics:");

	// Account count
	let account_count =
		storage::count_storage_entries(quantus_client, "System", "Account", block_hash).await?;
	log_print!("   • Accounts: {}", account_count.to_string().bright_green());

	// BlockHash count
	let blockhash_count =
		storage::count_storage_entries(quantus_client, "System", "BlockHash", block_hash).await?;
	log_print!("   • Block Hashes: {}", blockhash_count.to_string().bright_blue());

	// Event count
	let storage_at = quantus_client.client().storage().at(block_hash);
	let event_count_addr = crate::chain::quantus_subxt::api::storage().system().event_count();
	let event_count = storage_at
		.fetch_or_default(&event_count_addr)
		.await
		.map_err(|e| {
			log_error!("Failed to get event count: {:?}", e);
			e
		})
		.unwrap_or_default();
	log_print!("   • Events: {}", event_count.to_string().bright_yellow());

	log_print!("");
	Ok(())
}

/// Show events from the block
async fn show_block_events(block_number: u32, node_url: &str) -> crate::error::Result<()> {
	log_print!("📋 Events:");

	// Use the existing events command internally
	match crate::cli::events::handle_events_command(
		Some(block_number),
		None,
		false,
		None,
		false,
		true,
		node_url,
	)
	.await
	{
		Ok(_) => {
			// Events were already printed by the events command
		},
		Err(e) => log_error!("Failed to get events: {}", e),
	}

	log_print!("");
	Ok(())
}

/// Show extrinsic details
fn show_extrinsic_details(block_data: &serde_json::Value) -> crate::error::Result<()> {
	if let Some(block) = block_data.get("block") {
		if let Some(extrinsics) = block.get("extrinsics") {
			if let Some(extrinsics_array) = extrinsics.as_array() {
				log_print!("🔧 Extrinsics Details:");
				log_print!(
					"   • Total Count: {}",
					extrinsics_array.len().to_string().bright_green()
				);

				// Calculate total size of all extrinsics
				let mut total_size = 0;
				for extrinsic in extrinsics_array.iter() {
					if let Some(ext_str) = extrinsic.as_str() {
						total_size += ext_str.len();
					}
				}
				log_print!("   • Total Size: {} bytes", total_size.to_string().bright_magenta());

				// Show first 3 extrinsics with details
				for (index, extrinsic) in extrinsics_array.iter().take(3).enumerate() {
					let ext_str = extrinsic.as_str().unwrap_or("unknown");
					let ext_size = ext_str.len();
					log_print!(
						"   {}. Size: {} bytes, Data: {}...",
						(index + 1).to_string().bright_yellow(),
						ext_size.to_string().bright_blue(),
						if ext_str.len() > 20 { &ext_str[..20] } else { ext_str }.bright_cyan()
					);
				}

				if extrinsics_array.len() > 3 {
					log_print!(
						"   ... and {} more extrinsics",
						(extrinsics_array.len() - 3).to_string().bright_blue()
					);
				}
			}
		}
	}
	log_print!("");
	Ok(())
}
