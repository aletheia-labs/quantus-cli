use crate::chain::client::ChainClient;
use crate::chain::quantus_runtime_config::QuantusRuntimeConfig;
use crate::error::Result;
use crate::wallet::{password, WalletManager};
use crate::{log_error, log_print, log_success, log_verbose};
use codec::Compact;
use colored::Colorize;
use serde_json::Value;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58Codec;
use sp_runtime::MultiAddress;
use substrate_api_client::ac_compose_macros::compose_extrinsic;
use substrate_api_client::ac_primitives::ExtrinsicSigner;
use substrate_api_client::rpc_api::chain;
use substrate_api_client::{SubmitAndWatch, XtStatus};

/// Execute a generic call to any pallet
pub async fn execute_generic_call(
    chain_client: &ChainClient,
    pallet: &str,
    call: &str,
    args: Vec<Value>,
    from: &str,
    tip: Option<String>,
) -> Result<()> {
    log_print!("🚀 Executing generic call");
    log_print!("Pallet: {}", pallet.bright_green());
    log_print!("Call: {}", call.bright_cyan());
    log_print!("From: {}", from.bright_yellow());
    if let Some(tip) = &tip {
        log_print!("Tip: {}", tip.bright_magenta());
    }

    let wallet_manager = WalletManager::new()?;
    let wallet_password = password::get_wallet_password(from, None, None)?;

    let wallet_data = wallet_manager.load_wallet(from, &wallet_password)?;
    let keypair = &wallet_data.keypair;
    log_verbose!("✅ Loaded keypair for {}", from);

    // Get metadata and validate pallet/call exists
    let api = chain_client.get_api();
    let metadata = api.metadata();

    // Find the pallet in metadata
    let pallet_metadata = metadata.pallet_by_name(pallet).ok_or_else(|| {
        crate::error::QuantusError::Generic(format!("Pallet '{}' not found in metadata", pallet))
    })?;

    log_verbose!(
        "✅ Found pallet '{}' with index {}",
        pallet,
        pallet_metadata.index()
    );

    // Find the call in the pallet
    let call_variant = pallet_metadata.call_variant_by_name(call).ok_or_else(|| {
        crate::error::QuantusError::Generic(format!(
            "Call '{}' not found in pallet '{}'",
            call, pallet
        ))
    })?;

    log_verbose!("✅ Found call '{}' with index {}", call, call_variant.index);

    // Create API with signer
    let api_with_signer = chain_client.create_api_with_signer(keypair)?;

    // Create and submit extrinsic based on pallet and call - all logic in one place
    log_print!("🔧 Creating extrinsic for {}.{}", pallet, call);

    let tx_hash = match (pallet, call) {
        // Balances pallet calls
        ("Balances", "transfer_allow_death") => {
            submit_balance_transfer_extrinsic(chain_client, &api_with_signer, keypair, &args, false)
                .await?
        }
        ("Balances", "transfer_keep_alive") => {
            submit_balance_transfer_extrinsic(chain_client, &api_with_signer, keypair, &args, true)
                .await?
        }

        // System pallet calls
        ("System", "remark") => {
            submit_system_remark_extrinsic(chain_client, &api_with_signer, keypair, &args).await?
        }

        // ReversibleTransfers pallet calls
        ("ReversibleTransfers", "schedule_transfer") => {
            submit_reversible_transfer_extrinsic(chain_client, &api_with_signer, keypair, &args)
                .await?
        }

        // Unsupported combinations
        ("Balances", _) => {
            log_error!("❌ Balances call '{}' is not supported yet", call);
            log_print!("💡 Supported Balances calls: transfer_allow_death, transfer_keep_alive");
            return Err(crate::error::QuantusError::Generic(format!(
                "Unsupported Balances call: {}",
                call
            ))
            .into());
        }
        ("System", _) => {
            log_error!("❌ System call '{}' is not supported yet", call);
            log_print!("💡 Supported System calls: remark");
            return Err(crate::error::QuantusError::Generic(format!(
                "Unsupported System call: {}",
                call
            ))
            .into());
        }
        ("ReversibleTransfers", _) => {
            log_error!(
                "❌ ReversibleTransfers call '{}' is not supported yet",
                call
            );
            log_print!("💡 Supported ReversibleTransfers calls: schedule_transfer");
            return Err(crate::error::QuantusError::Generic(format!(
                "Unsupported ReversibleTransfers call: {}",
                call
            ))
            .into());
        }
        (_, _) => {
            log_error!("❌ Pallet '{}' is not supported yet", pallet);
            log_print!("💡 Supported pallets: Balances, System, ReversibleTransfers");
            return Err(crate::error::QuantusError::Generic(format!(
                "Unsupported pallet: {}. Only Balances, System, and ReversibleTransfers are currently supported.",
                pallet
            )).into());
        }
    };

    log_success!("🎉 Transaction submitted successfully!");
    log_print!("📋 Transaction hash: {}", tx_hash.bright_yellow());

    Ok(())
}

/// Create and submit balance transfer extrinsic using the central macro
async fn submit_balance_transfer_extrinsic(
    chain_client: &ChainClient,
    api: &substrate_api_client::Api<
        crate::chain::quantus_runtime_config::QuantusRuntimeConfig,
        substrate_api_client::rpc::JsonrpseeClient,
    >,
    keypair: &crate::wallet::QuantumKeyPair,
    args: &[Value],
    keep_alive: bool,
) -> Result<String> {
    if args.len() != 2 {
        return Err(crate::error::QuantusError::Generic(
            "Balance transfer requires 2 arguments: dest, value".to_string(),
        )
        .into());
    }

    // Parse destination address
    let dest_str = args[0].as_str().ok_or_else(|| {
        crate::error::QuantusError::Generic("First argument must be a string address".to_string())
    })?;
    let dest_account = AccountId32::from_ss58check(dest_str).map_err(|e| {
        crate::error::QuantusError::Generic(format!("Invalid destination address: {:?}", e))
    })?;
    let dest: MultiAddress<AccountId32, u32> = MultiAddress::Id(dest_account);

    // Parse amount
    let amount_value = parse_balance_amount(&args[1]).await?;

    let call_name = if keep_alive {
        "transfer_keep_alive"
    } else {
        "transfer_allow_death"
    };

    log_verbose!(
        "✅ Parsed {}: dest={:?}, amount={}",
        call_name,
        dest,
        amount_value
    );

    // Create extrinsic
    let extrinsic = compose_extrinsic!(api, "Balances", call_name, dest, Compact(amount_value))
        .ok_or_else(|| {
            crate::error::QuantusError::Generic(format!("Failed to create {} extrinsic", call_name))
        })?;

    // Use the central submit_extrinsic! macro - ONE place for all submission logic
    log_print!("📡 Submitting extrinsic to chain...");
    crate::submit_extrinsic!(chain_client, keypair, extrinsic)
}

/// Create and submit system remark extrinsic using the central macro
async fn submit_system_remark_extrinsic(
    chain_client: &ChainClient,
    api: &substrate_api_client::Api<
        crate::chain::quantus_runtime_config::QuantusRuntimeConfig,
        substrate_api_client::rpc::JsonrpseeClient,
    >,
    keypair: &crate::wallet::QuantumKeyPair,
    args: &[Value],
) -> Result<String> {
    if args.len() != 1 {
        return Err(crate::error::QuantusError::Generic(
            "remark requires 1 argument: remark (string or hex bytes)".to_string(),
        )
        .into());
    }

    // Parse remark data
    let remark_bytes = parse_bytes_argument(&args[0])?;

    log_verbose!("✅ Parsed remark: {} bytes", remark_bytes.len());

    // Create extrinsic
    let extrinsic = compose_extrinsic!(api, "System", "remark", remark_bytes).ok_or_else(|| {
        crate::error::QuantusError::Generic("Failed to create remark extrinsic".to_string())
    })?;

    // Use the central submit_extrinsic! macro - ONE place for all submission logic
    log_print!("📡 Submitting extrinsic to chain...");
    crate::submit_extrinsic!(chain_client, keypair, extrinsic)
}

/// Create and submit reversible transfer extrinsic using the central macro
async fn submit_reversible_transfer_extrinsic(
    chain_client: &ChainClient,
    api: &substrate_api_client::Api<
        crate::chain::quantus_runtime_config::QuantusRuntimeConfig,
        substrate_api_client::rpc::JsonrpseeClient,
    >,
    keypair: &crate::wallet::QuantumKeyPair,
    args: &[Value],
) -> Result<String> {
    if args.len() != 2 {
        return Err(crate::error::QuantusError::Generic(
            "schedule_transfer requires 2 arguments: dest, amount".to_string(),
        )
        .into());
    }

    // Parse destination address
    let dest_str = args[0].as_str().ok_or_else(|| {
        crate::error::QuantusError::Generic("First argument must be a string address".to_string())
    })?;
    let dest_account = AccountId32::from_ss58check(dest_str).map_err(|e| {
        crate::error::QuantusError::Generic(format!("Invalid destination address: {:?}", e))
    })?;
    let dest: MultiAddress<AccountId32, u32> = MultiAddress::Id(dest_account);

    // Parse amount
    let amount_value = parse_balance_amount(&args[1]).await?;

    log_verbose!(
        "✅ Parsed schedule_transfer: dest={:?}, amount={}",
        dest,
        amount_value
    );

    // Create extrinsic
    let extrinsic = compose_extrinsic!(
        api,
        "ReversibleTransfers",
        "schedule_transfer",
        dest,
        amount_value
    )
    .ok_or_else(|| {
        crate::error::QuantusError::Generic(
            "Failed to create schedule_transfer extrinsic".to_string(),
        )
    })?;

    // Use the central submit_extrinsic! macro - ONE place for all submission logic
    log_print!("📡 Submitting extrinsic to chain...");
    crate::submit_extrinsic!(chain_client, keypair, extrinsic)
}

/// Parse a balance amount from JSON value
async fn parse_balance_amount(value: &Value) -> Result<u128> {
    match value {
        Value::String(s) => {
            // Try to parse as a decimal number (like "10.5")
            if let Ok(parsed) = s.parse::<f64>() {
                // Convert to raw units (assuming 9 decimals for DEV token)
                let raw_amount = (parsed * 1_000_000_000.0) as u128;
                Ok(raw_amount)
            } else {
                Err(
                    crate::error::QuantusError::Generic(format!("Invalid balance amount: {}", s))
                        .into(),
                )
            }
        }
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Ok(u as u128)
            } else if let Some(f) = n.as_f64() {
                // Convert to raw units (assuming 9 decimals for DEV token)
                let raw_amount = (f * 1_000_000_000.0) as u128;
                Ok(raw_amount)
            } else {
                Err(
                    crate::error::QuantusError::Generic(format!("Invalid balance amount: {:?}", n))
                        .into(),
                )
            }
        }
        _ => Err(crate::error::QuantusError::Generic(
            "Balance amount must be a string or number".to_string(),
        )
        .into()),
    }
}

/// Parse bytes from JSON value (string or hex)
fn parse_bytes_argument(value: &Value) -> Result<Vec<u8>> {
    match value {
        Value::String(s) => {
            if s.starts_with("0x") {
                // Hex string
                hex::decode(&s[2..]).map_err(|e| {
                    crate::error::QuantusError::Generic(format!("Invalid hex string: {:?}", e))
                        .into()
                })
            } else {
                // Regular string - convert to bytes
                Ok(s.as_bytes().to_vec())
            }
        }
        _ => Err(crate::error::QuantusError::Generic(
            "Bytes argument must be a string or hex string".to_string(),
        )
        .into()),
    }
}
