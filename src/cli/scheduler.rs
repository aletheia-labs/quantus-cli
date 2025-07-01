use crate::chain::client::ChainClient;
use crate::error::Result;
use crate::{log_error, log_print, log_success};
use clap::Subcommand;
use substrate_api_client::GetStorage;

/// Scheduler-related commands
#[derive(Subcommand, Debug)]
pub enum SchedulerCommands {
    /// Get the last processed timestamp from the scheduler
    GetLastProcessedTimestamp,
}

/// Handle scheduler commands
pub async fn handle_scheduler_command(command: SchedulerCommands, node_url: &str) -> Result<()> {
    log_print!("🗓️  Scheduler");

    let chain_client = ChainClient::new(node_url).await?;

    match command {
        SchedulerCommands::GetLastProcessedTimestamp => {
            get_last_processed_timestamp(&chain_client).await
        }
    }
}

/// Get the last processed timestamp from the scheduler
async fn get_last_processed_timestamp(chain_client: &ChainClient) -> Result<()> {
    log_print!("🕒 Getting last processed timestamp from the scheduler");

    match chain_client
        .get_api()
        .get_storage::<u64>("Scheduler", "LastProcessedTimestamp", None)
        .await
    {
        Ok(Some(timestamp)) => {
            log_success!("🎉 Last processed timestamp: {}", timestamp);
        }
        Ok(None) => {
            log_print!("🤷 No last processed timestamp found. The scheduler may not have run yet.");
        }
        Err(e) => {
            log_error!("❌ Error getting last processed timestamp: {:?}", e);
        }
    }

    Ok(())
}
