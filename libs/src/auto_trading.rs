use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use book::err_utils::ErrStr;
use ethers::{
        middleware::SignerMiddleware,
        providers::{Http, Middleware, Provider},
        signers::{LocalWallet, Signer},
        types::{
            transaction::eip2718::TypedTransaction, Address, Bytes, Eip1559TransactionRequest, U256,
        },
};
use serde::Deserialize;

//============================================================================
//----- Token Registry --------------------------------------------------------
//============================================================================
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TokenEntry {
    #[serde(default)]
    pub native:   bool,
    #[serde(default)]
    pub address:  Option<String>,
    pub decimals: u32,
}

pub type TokenRegistry = HashMap<String, TokenEntry>;

/// Each binary embeds its own `tokens.toml` via `include_str!` (the token
/// set differs per binary) and passes the raw string here to parse it.
pub fn parse_token_registry(toml_str: &str) -> ErrStr<TokenRegistry> {
    toml::from_str(toml_str).map_err(|e| format!("Failed to parse tokens.toml: {e}"))
}

pub fn token_entry<'a>(registry: &'a TokenRegistry, symbol: &str) -> ErrStr<&'a TokenEntry> {
    match registry.get(symbol) {
        Some(entry) => Ok(entry),
        None => Err(format!(
            "No tokens.toml entry for '{symbol}' — add one before checking this pool"
        )),
    }
}

//============================================================================
//----- Shared HTTP Client ----------------------------------------------------
//============================================================================
const HTTP_TIMEOUT_SECS: u64 = 15;

fn http_client() -> ErrStr<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Could not build HTTP client: {e}"))
}

//============================================================================
//----- Wallet Balance Check --------------------------------------------------
//============================================================================
pub const AVALANCHE_RPC: &str = "https://api.avax.network/ext/bc/C/rpc";
pub const AVALANCHE_CHAIN_ID: u64 = 43114;
const KYBERSWAP_CHAIN: &str = "avalanche";

pub fn wallet_address_from_env(var_name: &str) -> ErrStr<String> {
    std::env::var(var_name).map_err(|_| {
        let ans = format!("Missing required env var: {var_name} (your public wallet address)");
        ans
    })
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<String>,
    error:  Option<serde_json::Value>,
}

async fn rpc_call(method: &str, params: serde_json::Value) -> ErrStr<String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let resp = http_client()?
        .post(AVALANCHE_RPC)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("RPC request ({method}) failed: {e}"))?;
    let parsed: RpcResponse = resp
        .json()
        .await
        .map_err(|e| format!("RPC response for {method} did not parse: {e}"))?;
    if let Some(err) = parsed.error {
        return Err(format!("RPC error for {method}: {err}"));
    }
    parsed
        .result
        .ok_or_else(|| format!("RPC call {method} returned no result"))
}

fn hex_to_u128(hex: &str) -> ErrStr<u128> {
    let trimmed = hex.trim_start_matches("0x");
    let trimmed = if trimmed.is_empty() { "0" } else { trimmed };
    u128::from_str_radix(trimmed, 16)
        .map_err(|e| format!("Could not parse hex balance '{hex}': {e}"))
}

fn pad_address_for_call(address: &str) -> String {
    let hex = address.trim_start_matches("0x").to_lowercase();
    let ans = format!("{hex:0>64}");
    ans
}

async fn erc20_balance(wallet_address: &str, token_contract: &str) -> ErrStr<u128> {
    // balanceOf(address) selector = 0x70a08231
    let data = format!("0x70a08231{}", pad_address_for_call(wallet_address));
    let result = rpc_call(
        "eth_call",
        serde_json::json!([{ "to": token_contract, "data": data }, "latest"]),
    )
    .await?;
    hex_to_u128(&result)
}

pub async fn wallet_balance(
    wallet_address: &str,
    symbol: &str,
    registry: &TokenRegistry,
) -> ErrStr<f64> {
    let entry = token_entry(registry, symbol)?;
    let addr = entry
        .address
        .as_deref()
        .ok_or_else(|| format!("'{symbol}' has no address in tokens.toml (or is native — not supported by this helper)"))?;
    let raw = erc20_balance(wallet_address, addr).await?;
    Ok(raw as f64 / 10f64.powi(entry.decimals as i32))
}

//============================================================================
//----- Live KyberSwap Quote --------------------------------------------------
//============================================================================
/// A live quote plus everything needed to actually build and sign the swap
/// afterward.
pub struct KyberQuote {
    pub amount_out:         f64,
    pub route_summary_raw:  serde_json::Value,
    pub router_address:     String,
}

pub async fn live_quote(
    registry: &TokenRegistry,
    from_symbol: &str,
    to_symbol: &str,
    amount: f64,
) -> ErrStr<KyberQuote> {
    let from_entry = token_entry(registry, from_symbol)?;
    let to_entry = token_entry(registry, to_symbol)?;
    let token_in = from_entry.address.as_deref().ok_or_else(|| format!("{from_symbol} missing address"))?;
    let token_out = to_entry.address.as_deref().ok_or_else(|| format!("{to_symbol} missing address"))?;
    let amount_in_base = (amount * 10f64.powi(from_entry.decimals as i32)).round() as u128;

    let url = format!(
        "https://aggregator-api.kyberswap.com/{KYBERSWAP_CHAIN}/api/v1/routes?tokenIn={token_in}&tokenOut={token_out}&amountIn={amount_in_base}"
    );

    let resp = http_client()?
        .get(&url)
        .header("X-Client-Id", "pivoteur-autotrader")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("KyberSwap route request failed: {e}"))?;

    let status = resp.status();
    let raw_body = resp
        .text()
        .await
        .map_err(|e| format!("Could not read KyberSwap response body: {e}"))?;

    let parsed: serde_json::Value = serde_json::from_str(&raw_body).map_err(|e| {
        format!("KyberSwap response did not parse (HTTP {status}): {e}\nRaw body: {raw_body}")
    })?;

    let data = parsed
        .get("data")
        .ok_or_else(|| format!("KyberSwap returned no route ({from_symbol} -> {to_symbol}). Raw: {raw_body}"))?;
    let route_summary_raw = data
        .get("routeSummary")
        .cloned()
        .ok_or_else(|| format!("Response missing routeSummary. Raw: {raw_body}"))?;
    let router_address = data
        .get("routerAddress")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Response missing routerAddress. Raw: {raw_body}"))?
        .to_string();
    let amount_out_str = route_summary_raw
        .get("amountOut")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("routeSummary missing amountOut. Raw: {raw_body}"))?;
    let raw: u128 = amount_out_str
        .parse()
        .map_err(|_| format!("Could not parse amountOut '{amount_out_str}'"))?;
    let amount_out = raw as f64 / 10f64.powi(to_entry.decimals as i32);

    Ok(KyberQuote { amount_out, route_summary_raw, router_address })
}

//============================================================================
//----- Signing & Execution ----------------------------------------------------
//============================================================================
// Everything past this point can move real funds. Every function here is
// deliberately loud on failure.

fn pad_u256_for_call(amount: u128) -> String {
    let ans = format!("{amount:064x}");
    ans
}

/// AVAX cost of a confirmed transaction, computed from its own receipt
/// (gas_used * effective_gas_price), not estimated beforehand. If a
/// receipt is somehow missing pricing info, returns 0.0 rather than
/// failing the whole trade over a cosmetic figure — the trade itself
/// already succeeded by the time this is called.
fn gas_cost_avax(gas_used: Option<U256>, effective_gas_price: Option<U256>) -> f64 {
    match (gas_used, effective_gas_price) {
        (Some(g), Some(p)) => {
            let wei = g.saturating_mul(p);
            wei.as_u128() as f64 / 1e18
        }
        _ => 0.0,
    }
}

pub async fn load_signer(expected_address: &str, keystore_path_var: &str) -> ErrStr<LocalWallet> {
    let keystore_path = std::env::var(keystore_path_var).map_err(|_| {
        format!("Missing required env var: {keystore_path_var} (full path to the encrypted keystore file). No funds moved.")
    })?;
    let password = match std::env::var("KEYSTORE_PASSWORD") {
        Ok(pw) => pw,
        Err(_) => rpassword::prompt_password("Keystore password: ")
            .map_err(|e| format!("Could not read password: {e}. No funds moved."))?,
    };
    let wallet = LocalWallet::decrypt_keystore(&keystore_path, &password)
        .map_err(|e| format!("Could not decrypt keystore: {e}. No funds moved."))?
        .with_chain_id(AVALANCHE_CHAIN_ID);
    let derived = format!("{:?}", wallet.address());
    if !derived.eq_ignore_ascii_case(expected_address) {
        return Err(format!(
            "Keystore address ({derived}) does not match expected address ({expected_address}) — refusing to proceed. No funds moved."
        ));
    }
    Ok(wallet)
}

/// Builds an EIP-1559 tx with a buffered max fee (so a base-fee bump between
/// estimation and submission — e.g. during the password prompt — doesn't
/// get the tx rejected pre-mempool) and a buffered gas limit. Fees are
/// re-estimated fresh on every call rather than reused across steps.
async fn build_tx_with_fee_buffer(
    client: &SignerMiddleware<Provider<Http>, LocalWallet>,
    to: Address,
    data: Bytes,
) -> ErrStr<Eip1559TransactionRequest> {
    let (max_fee, max_priority_fee) = client
        .estimate_eip1559_fees(None)
        .await
        .map_err(|e| format!("Could not estimate EIP-1559 fees: {e}"))?;

    // 30% buffer on the max fee absorbs a base-fee bump between estimation
    // and submission without overpaying on the priority fee.
    let buffered_max_fee = max_fee.saturating_mul(U256::from(130)) / U256::from(100);

    let mut tx = Eip1559TransactionRequest::new()
        .to(to)
        .data(data)
        .max_fee_per_gas(buffered_max_fee)
        .max_priority_fee_per_gas(max_priority_fee);

    let typed: TypedTransaction = tx.clone().into();
    let gas_estimate = client
        .estimate_gas(&typed, None)
        .await
        .map_err(|e| format!("Could not estimate gas limit: {e}"))?;
    // 20% buffer on gas so a slightly-off estimate doesn't run out mid-execution.
    let buffered_gas = gas_estimate.saturating_mul(U256::from(120)) / U256::from(100);
    tx = tx.gas(buffered_gas);

    Ok(tx)
}

/// Approves the router for EXACTLY this trade's amount — never a standing
/// allowance. The router can never pull more than what's approved here.
pub async fn approve_exact_amount(
    client: &SignerMiddleware<Provider<Http>, LocalWallet>,
    token_contract: &str,
    spender: &str,
    amount_base_units: u128,
) -> ErrStr<f64> {
    let data_hex = format!(
        "0x095ea7b3{}{}",
        pad_address_for_call(spender),
        pad_u256_for_call(amount_base_units)
    );
    let to = Address::from_str(token_contract).map_err(|e| format!("Bad token address: {e}"))?;
    let data = Bytes::from_str(&data_hex).map_err(|e| format!("Bad approve calldata: {e}"))?;
    let tx = build_tx_with_fee_buffer(client, to, data).await?;

    let pending = client
        .send_transaction(tx, None)
        .await
        .map_err(|e| format!("Approve transaction failed to send: {e}"))?;
    println!("    Approve tx submitted: {:?}", pending.tx_hash());

    let receipt = pending
        .await
        .map_err(|e| format!("Approve transaction failed while confirming: {e}"))?;
    match receipt {
        Some(r) => {
            println!("    Approve confirmed in block {:?}", r.block_number);
            Ok(gas_cost_avax(r.gas_used, r.effective_gas_price))
        }
        None => Err("Approve transaction was dropped or replaced".to_string()),
    }
}

/// Asks KyberSwap to encode the actual swap calldata for the route.
/// Prints the raw response every time — verify it before trusting it.
/// `slippage_bps` is basis points (e.g. 50 = 0.50%).
pub async fn kyberswap_build(
    route_summary_raw: &serde_json::Value,
    sender: &str,
    slippage_bps: u16,
) -> ErrStr<(String, String)> {
    let body = serde_json::json!({
        "routeSummary": route_summary_raw,
        "sender": sender,
        "recipient": sender,
        "slippageTolerance": slippage_bps
    });

    let resp = http_client()?
        .post(format!("https://aggregator-api.kyberswap.com/{KYBERSWAP_CHAIN}/api/v1/route/build"))
        .header("X-Client-Id", "pivoteur-autotrader")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("KyberSwap build request failed: {e}"))?;

    let status = resp.status();
    let raw_body = resp.text().await.map_err(|e| format!("Could not read build response: {e}"))?;
    println!("    KyberSwap build response (verify this looks right):\n    {raw_body}");

    let parsed: serde_json::Value = serde_json::from_str(&raw_body).map_err(|e| {
        format!("KyberSwap build response did not parse (HTTP {status}): {e}\nRaw body: {raw_body}")
    })?;
    let data = parsed
        .get("data")
        .ok_or_else(|| format!("Build response has no data. Raw: {raw_body}"))?;
    let router = data
        .get("routerAddress")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Build response missing routerAddress. Raw: {raw_body}"))?
        .to_string();
    let calldata = data
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Build response missing calldata. Raw: {raw_body}"))?
        .to_string();

    Ok((router, calldata))
}

/// Signs and sends the swap transaction. Returns the tx hash on success;
/// hard errors on revert, drop, or replacement rather than reporting a
/// false success. To see it: snowtrace.io/tx/<tx_hash>
pub async fn send_swap_tx(
    client: &SignerMiddleware<Provider<Http>, LocalWallet>,
    router: &str,
    calldata_hex: &str,
) -> ErrStr<(String, f64)> {
    let to = Address::from_str(router).map_err(|e| format!("Bad router address: {e}"))?;
    let data = Bytes::from_str(calldata_hex).map_err(|e| format!("Bad calldata from KyberSwap: {e}"))?;
    let tx = build_tx_with_fee_buffer(client, to, data).await?;

    let pending = client
        .send_transaction(tx, None)
        .await
        .map_err(|e| format!("Swap transaction failed to send: {e}"))?;
    let tx_hash = format!("{:?}", pending.tx_hash());
    println!("    Swap tx submitted: {tx_hash}");

    let receipt = pending
        .await
        .map_err(|e| format!("Swap transaction failed while confirming: {e}"))?;
    match receipt {
        Some(r) if r.status == Some(1.into()) => {
            println!("    Swap confirmed in block {:?}", r.block_number);
            Ok((tx_hash, gas_cost_avax(r.gas_used, r.effective_gas_price)))
        }
        Some(_) => Err(format!("Swap transaction REVERTED on-chain. Hash: {tx_hash}")),
        None => Err(format!("Swap transaction was dropped or replaced. Hash: {tx_hash}")),
    }
}

/// Sends `amount` of `symbol` from the wallet straight to `to_address` —
/// a plain ERC-20 transfer(), not a swap. Same safety baseline as
/// execute_trade: verified signer, EIP-1559 fee buffering, hard error on
/// revert/drop/failure rather than a false success. Does not support
/// native AVAX (no tokens.toml address to encode against) — only ERC-20s
/// with a real contract address.
pub async fn send_tokens(
    wallet_address: &str,
    registry: &TokenRegistry,
    symbol: &str,
    to_address: &str,
    amount: f64,
    keystore_path_var: &str,
) -> ErrStr<(String, f64)> {
    if amount <= 0.0 {
        return Err(format!("send_tokens: amount must be positive, got {amount}"));
    }

    let signer = load_signer(wallet_address, keystore_path_var).await?;
    let provider = Provider::<Http>::try_from(AVALANCHE_RPC)
        .map_err(|e| format!("Could not create RPC provider: {e}"))?;
    let client = SignerMiddleware::new(provider, signer);

    let entry = token_entry(registry, symbol)?;
    let token_addr = entry.address.as_deref().ok_or_else(|| {
        format!("'{symbol}' has no address in tokens.toml (or is native — send_tokens only supports ERC-20 transfers)")
    })?;
    let amount_base = (amount * 10f64.powi(entry.decimals as i32)).round() as u128;

    // transfer(address,uint256) selector = 0xa9059cbb
    let data_hex = format!(
        "0xa9059cbb{}{}",
        pad_address_for_call(to_address),
        pad_u256_for_call(amount_base)
    );
    let to = Address::from_str(token_addr).map_err(|e| format!("Bad token address: {e}"))?;
    let data = Bytes::from_str(&data_hex).map_err(|e| format!("Bad transfer calldata: {e}"))?;
    let tx = build_tx_with_fee_buffer(&client, to, data).await?;

    let pending = client
        .send_transaction(tx, None)
        .await
        .map_err(|e| format!("Transfer transaction failed to send: {e}"))?;
    let tx_hash = format!("{:?}", pending.tx_hash());
    println!("    Transfer tx submitted: {tx_hash}");

    let receipt = pending
        .await
        .map_err(|e| format!("Transfer transaction failed while confirming: {e}"))?;
    match receipt {
        Some(r) if r.status == Some(1.into()) => {
            println!("    Transfer confirmed in block {:?}", r.block_number);
            Ok((tx_hash, gas_cost_avax(r.gas_used, r.effective_gas_price)))
        }
        Some(_) => Err(format!("Transfer transaction REVERTED on-chain. Hash: {tx_hash}")),
        None => Err(format!("Transfer transaction was dropped or replaced. Hash: {tx_hash}")),
    }
}

//============================================================================
//----- UNIT TESTS -------------------------------------------------------------
//============================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;


    #[test]
    fn test_hex_to_u128_parses_rpc_style_hex() -> ErrStr<()> {
        assert_eq!(hex_to_u128("0x0")?, 0);
        assert_eq!(hex_to_u128("0x")?, 0);
        assert_eq!(hex_to_u128("0xff")?, 255);
        assert_eq!(hex_to_u128("0xde0b6b3a7640000")?, 1_000_000_000_000_000_000);
        Ok(())
    }

    #[test]
    fn test_hex_to_u128_rejects_garbage() {
        assert!(hex_to_u128("0xnotarealnumber").is_err());
    }

    #[test]
    fn test_pad_address_for_call_produces_32_byte_word() {
        let padded = pad_address_for_call("0x69b21DC480CA62E478D997d7313061F765a5B122");
        assert_eq!(padded.len(), 64);
        assert!(padded.ends_with("69b21dc480ca62e478d997d7313061f765a5b122"));
        assert!(padded.starts_with("00000000000000000000"));
    }
}

pub async fn execute_trade(
    wallet_address: &str,
    registry: &TokenRegistry,
    from_symbol: &str,
    to_symbol: &str,
    amount: f64,
    min_floor: f64,
    slippage_bps: u16,
    keystore_path_var: &str,
) -> ErrStr<(String, f64)> {
    let signer = load_signer(wallet_address, keystore_path_var).await?;
    let provider = Provider::<Http>::try_from(AVALANCHE_RPC)
        .map_err(|e| format!("Could not create RPC provider: {e}"))?;
    let client = SignerMiddleware::new(provider, signer);

    println!(">>> Re-checking the quote after keystore unlock (it may have moved)...");
    let fresh_quote = live_quote(registry, from_symbol, to_symbol, amount).await?;
    println!("Fresh quote: {amount:.6} {from_symbol} -> {:.8} {to_symbol} now", fresh_quote.amount_out);
    if fresh_quote.amount_out < min_floor {
        return Err(format!(
            "Quote moved below your floor while unlocking the keystore ({:.8} {to_symbol} < {min_floor:.8} {to_symbol}). \
             That's not happening. No funds used.",
            fresh_quote.amount_out
        ));
    }

    let from_entry = token_entry(registry, from_symbol)?;
    let from_addr = from_entry.address.as_deref().ok_or_else(|| format!("{from_symbol} missing address"))?.to_string();
    let amount_base = (amount * 10f64.powi(from_entry.decimals as i32)).round() as u128;

    println!(">>> Approving exact amount ({amount:.6} {from_symbol}) for the router...");
    let approve_gas = approve_exact_amount(&client, &from_addr, &fresh_quote.router_address, amount_base).await?;

    println!(">>> Requesting swap calldata from KyberSwap...");
    let (router, calldata) =
        kyberswap_build(&fresh_quote.route_summary_raw, wallet_address, slippage_bps).await?;

    println!(">>> Sending swap transaction...");
    let (tx_hash, swap_gas) = send_swap_tx(&client, &router, &calldata).await?;

    Ok((tx_hash, approve_gas + swap_gas))
}
