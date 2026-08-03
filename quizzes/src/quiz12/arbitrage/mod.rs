use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{
        transaction::eip2718::TypedTransaction, Address, Bytes, Eip1559TransactionRequest, U256,
    },
};
use book::{
        cli_utils::add_banner,
        err_utils::ErrStr,
        parse_args_add_banner,
};
use libs::{ fetchers::calls::fetch_calls, types::calls::Call };


//============================================================================
//----- Token Registry --------------------------------------------------------
//============================================================================
/// Just BTC and ETH for this program. tokens.toml lives alongside this file.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TokenEntry {
    #[serde(default)]
    pub native: bool,
    #[serde(default)]
    pub address: Option<String>,
    pub decimals: u32,
}

pub type TokenRegistry = HashMap<String, TokenEntry>;

const TOKENS_TOML: &str = include_str!("tokens.toml");

pub fn load_token_registry() -> ErrStr<TokenRegistry> {
    toml::from_str(TOKENS_TOML).map_err(|e| format!("Failed to parse tokens.toml: {e}"))
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
//----- Trade Direction --------------------------------------------------------
//============================================================================
const PRIMARY_SYMBOL: &str = "BTC";
const PIVOT_SYMBOL: &str = "ETH";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Direction {
    Normal,
    Flipped,
}

impl Direction {
    fn symbols(&self) -> (&'static str, &'static str) {
        match self {
            Direction::Normal => (PRIMARY_SYMBOL, PIVOT_SYMBOL),
            Direction::Flipped => (PIVOT_SYMBOL, PRIMARY_SYMBOL),
        }
    }
}

//============================================================================
//----- Shared HTTP Client ----------------------------------------------------
//============================================================================
// Every outbound call (RPC and KyberSwap) goes through this so a hung
// connection fails loudly in seconds instead of hanging the program.

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
// Read-only: eth_getBalance / eth_call queries against a public RPC.
// No key, no signature, nothing that can move funds.

const AVALANCHE_RPC: &str = "https://api.avax.network/ext/bc/C/rpc";

fn wallet_address_from_env() -> ErrStr<String> {
    std::env::var("WALLET_ADDRESS").map_err(|_| {
        "Missing required env var: WALLET_ADDRESS (your public wallet address)".to_string()
    })
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<String>,
    error: Option<serde_json::Value>,
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
    return format!("{hex:0>64}");
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

/// Human-readable balance of `symbol` in `wallet_address`, read-only.
pub async fn wallet_balance(
    wallet_address: &str,
    symbol: &str,
    registry: &TokenRegistry,
) -> ErrStr<f64> {
    let entry = token_entry(registry, symbol)?;
    let addr = entry
        .address
        .as_deref()
        .ok_or_else(|| format!("'{symbol}' has no address in tokens.toml"))?;
    let raw = erc20_balance(wallet_address, addr).await?;
    Ok(raw as f64 / 10f64.powi(entry.decimals as i32))
}

//============================================================================
//----- Live KyberSwap Quote --------------------------------------------------
//============================================================================
// Read-only route lookup — no signing, no submission. Tells you what the
// swap would actually return right now, in either direction.

const KYBERSWAP_CHAIN: &str = "avalanche";
const AVALANCHE_CHAIN_ID: u64 = 43114;

/// A live quote plus everything needed to actually build and sign the swap
/// afterward.
pub struct KyberQuote {
    pub amount_out: f64,
    pub route_summary_raw: serde_json::Value,
    pub router_address: String,
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
        .header("X-Client-Id", "pivoteur-arbitrage")
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
    return format!("{amount:064x}");
}

/// Loads the encrypted keystore and verifies the derived address actually
/// matches WALLET_ADDRESS before handing back a signer — refuses to proceed
/// on a mismatch rather than silently signing with the wrong key.
///
/// Password source: if `KEYSTORE_PASSWORD` is set in the environment, it's
/// used directly (no prompt) — this is what makes unattended CI runs
/// possible. Locally, where that env var typically isn't set, it falls back
/// to an interactive prompt so nothing changes for manual runs. Either way
/// the password itself is never logged or written anywhere.
async fn load_signer(expected_address: &str) -> ErrStr<LocalWallet> {
    let keystore_path = std::env::var("KEYSTORE_PATH").map_err(|_| {
        "Missing required env var: KEYSTORE_PATH (full path to the encrypted keystore file)".to_string()
    })?;
    let password = match std::env::var("KEYSTORE_PASSWORD") {
        Ok(pw) => pw,
        Err(_) => rpassword::prompt_password("Keystore password: ")
            .map_err(|e| format!("Could not read password: {e}"))?,
    };
    let wallet = LocalWallet::decrypt_keystore(&keystore_path, &password)
        .map_err(|e| format!("Could not decrypt keystore: {e}"))?
        .with_chain_id(AVALANCHE_CHAIN_ID);
    let derived = format!("{:?}", wallet.address());
    if !derived.eq_ignore_ascii_case(expected_address) {
        return Err(format!(
            "Keystore address ({derived}) does not match WALLET_ADDRESS ({expected_address}) — refusing to proceed."
        ));
    }
    Ok(wallet)
}

/// Builds an EIP-1559 tx with a buffered max fee (so a base-fee bump between
/// estimation and submission — e.g. during the password prompt — doesn't get
/// the tx rejected pre-mempool) and a buffered gas limit. Fees are
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
/// Works for whichever token is on the "from" side of the current direction.
async fn approve_exact_amount(
    client: &SignerMiddleware<Provider<Http>, LocalWallet>,
    token_contract: &str,
    spender: &str,
    amount_base_units: u128,
) -> ErrStr<()> {
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
            Ok(())
        }
        None => Err("Approve transaction was dropped or replaced".to_string()),
    }
}

/// Asks KyberSwap to encode the actual swap calldata for the route.
/// Prints the raw response every time — verify it before
/// trusting it. `slippage_bps` is basis points
/// (e.g. 50 = 0.50%) and comes from the CLI, not a hardcoded value.
async fn kyberswap_build(
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
        .header("X-Client-Id", "pivoteur-arbitrage")
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

/// Signs and sends the swap transaction. Returns the <tx hash> on success;
/// hard errors on revert, drop, or replacement rather than reporting a
/// false success.
/// To see real url, type: snowtrace.io/tx/<tx_hash> in a browser.
async fn send_swap_tx(
    client: &SignerMiddleware<Provider<Http>, LocalWallet>,
    router: &str,
    calldata_hex: &str,
) -> ErrStr<String> {
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
            Ok(tx_hash)
        }
        Some(_) => Err(format!("Swap transaction REVERTED on-chain. Hash: {tx_hash}")),
        None => Err(format!("Swap transaction was dropped or replaced. Hash: {tx_hash}")),
    }
}

//============================================================================
//----- Trade Log --------------------------------------------------------------
//============================================================================
const TRADE_LOG_PATH: &str = "arbitrage_trades.log";

fn log_trade_outcome(
    from_symbol: &str,
    to_symbol: &str,
    amount: f64,
    min_floor: f64,
    quote_out: f64,
    outcome: &str,
) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!(
        "{ts},{from_symbol}->{to_symbol},{amount:.6},{min_floor:.8},{quote_out:.8},{outcome}\n"
    );
    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(TRADE_LOG_PATH)
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(e) = result {
        eprintln!("Warning: could not write to trade log ({TRADE_LOG_PATH}): {e}");
    }
}

//============================================================================
//----- Trade Flow -------------------------------------------------------------
//============================================================================
/// Everything from keystore unlock through the swap. Split out of
/// `run_trade_for_symbols` so the quote can be refreshed right before
/// committing funds. The password prompt takes real wall-clock time, and
/// Avalanche's base fee (and the KyberSwap route itself) can move during it.
async fn execute_trade(
    wallet_address: &str,
    registry: &TokenRegistry,
    from_symbol: &str,
    to_symbol: &str,
    amount: f64,
    min_floor: f64,
    slippage_bps: u16,
) -> ErrStr<String> {
    let signer = load_signer(wallet_address).await?;
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
    approve_exact_amount(&client, &from_addr, &fresh_quote.router_address, amount_base).await?;

    println!(">>> Requesting swap calldata from KyberSwap...");
    let (router, calldata) =
        kyberswap_build(&fresh_quote.route_summary_raw, wallet_address, slippage_bps).await?;

    println!(">>> Sending swap transaction...");
    send_swap_tx(&client, &router, &calldata).await
}

async fn run_trade_for_symbols(
    wallet_address: &str,
    registry: &TokenRegistry,
    from_symbol: &str,
    to_symbol: &str,
    amount: f64,
    min_floor: f64,
    slippage_bps: u16,
    dry_run: bool,
) -> ErrStr<()> {
    if amount <= 0.0 {
        return Err("amount must be greater than zero".to_string());
    }
    if min_floor <= 0.0 {
        return Err("min_floor must be greater than zero".to_string());
    }

    let available = wallet_balance(wallet_address, from_symbol, registry).await?;
    println!("Wallet ({wallet_address}): {available:.6} {from_symbol} available");
    if available + 1e-6 < amount {
        log_trade_outcome(from_symbol, to_symbol, amount, min_floor, 0.0, "REJECTED_INSUFFICIENT_FUNDS");
        return Err(format!(
            "Insufficient {from_symbol} — need {amount:.6}, only {available:.6} available. \
             That's not happening. No funds used."
        ));
    }

    let quote = live_quote(registry, from_symbol, to_symbol, amount).await?;
    println!("Live quote: {amount:.6} {from_symbol} -> {:.8} {to_symbol} right now", quote.amount_out);
    println!("Your floor: {min_floor:.8} {to_symbol}");

    if quote.amount_out < min_floor {
        log_trade_outcome(from_symbol, to_symbol, amount, min_floor, quote.amount_out, "REJECTED_FLOOR");
        return Err(format!(
            "Quote ({:.8} {to_symbol}) is below your floor ({min_floor:.8} {to_symbol}). \
             That's not happening. No funds used.",
            quote.amount_out
        ));
    }

    if dry_run {
        println!(">>> DRY RUN: quote clears your floor. No keystore touched, nothing sent, no funds moved.");
        log_trade_outcome(from_symbol, to_symbol, amount, min_floor, quote.amount_out, "DRY_RUN_OK");
        return Ok(());
    }

    println!(">>> Quote clears your floor. Proceeding to execute.");

    match execute_trade(wallet_address, registry, from_symbol, to_symbol, amount, min_floor, slippage_bps).await {
        Ok(tx_hash) => {
            println!(">>> Trade complete. Tx hash: {tx_hash}");
            log_trade_outcome(from_symbol, to_symbol, amount, min_floor, quote.amount_out, &format!("SUCCESS tx={tx_hash}"));
            Ok(())
        }
        Err(e) => {
            log_trade_outcome(from_symbol, to_symbol, amount, min_floor, quote.amount_out, &format!("ERROR: {e}"));
            Err(e)
        }
    }
}

pub async fn run_trade(
    amount: f64,
    min_floor: f64,
    direction: Direction,
    slippage_bps: u16,
    dry_run: bool,
) -> ErrStr<()> {
    let (from_symbol, to_symbol) = direction.symbols();
    let wallet_address = wallet_address_from_env()?;
    let registry = load_token_registry()?;
    run_trade_for_symbols(&wallet_address, &registry, from_symbol, to_symbol, amount, min_floor, slippage_bps, dry_run).await
}

/// Reads calls.csv and, for each row, either executes the FULL proposed
/// trade or does nothing at all for that row. 
pub async fn run_calls_batch(root_url: &str, slippage_bps: u16, dry_run: bool) -> ErrStr<()> {
    let wallet_address = wallet_address_from_env()?;
    let registry = load_token_registry()?;

    let calls: Vec<Call> = fetch_calls(root_url)
        .await
        .map_err(|e| format!("Could not fetch calls.csv from {root_url}: {e}"))?;
    println!("Fetched {} call(s) from {root_url}", calls.len());

    for call in &calls {
        let from_symbol = call.pivot_token.as_str();
        let to_symbol = call.proposed_token.as_str();
        let amount = call.pivot_amount as f64;
        let min_floor = call.proposed_amount as f64;

        println!("--- Call #{} ({from_symbol} -> {to_symbol}) ---", call.ix);

        if token_entry(&registry, from_symbol).is_err() || token_entry(&registry, to_symbol).is_err() {
            println!("SKIPPED: '{from_symbol}' or '{to_symbol}' not in tokens.toml");
            log_trade_outcome(from_symbol, to_symbol, amount, min_floor, 0.0, "SKIPPED_UNKNOWN_TOKEN");
            continue;
        }

        if let Err(e) = run_trade_for_symbols(
            &wallet_address, &registry, from_symbol, to_symbol, amount, min_floor, slippage_bps, dry_run,
        ).await {
            println!("Call #{} did not execute: {e}", call.ix);
        }
    }

    Ok(())
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Trade a specific amount directly — manual / one-off mode
    Trade {
        amount: f64,
        min_floor: f64,
        /// Reverse the trade direction (default: BTC -> ETH; --flip: ETH -> BTC)
        #[arg(long, default_value_t = false)]
        flip: bool,
        #[arg(long, default_value_t = 50)]
        slippage_bps: u16,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Read calls.csv and execute any row the wallet can fully cover — 100% or nothing
    Calls {
        /// Root URL calls.csv is fetched relative to. Falls back to the
        /// PIVOT_URL env var (same convention every other binary uses) if
        /// not passed explicitly.
        #[arg(long, env = "PIVOT_URL")]
        root_url: String,
        #[arg(long, default_value_t = 50)]
        slippage_bps: u16,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Debug, Parser)]
#[command(name = "arbitrage")]
#[command(version = "0.9.0")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

pub async fn runoff_with_args() -> ErrStr<()> {
    let args = parse_args_add_banner!(Args);
    match args.command {
        Command::Trade { amount, min_floor, flip, slippage_bps, dry_run } => {
            let direction = if flip { Direction::Flipped } else { Direction::Normal };
            run_trade(amount, min_floor, direction, slippage_bps, dry_run).await
        }
        Command::Calls { root_url, slippage_bps, dry_run } => {
            run_calls_batch(&root_url, slippage_bps, dry_run).await
        }
    }
}

//============================================================================
//----- UNIT TESTS -------------------------------------------------------------
//============================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_load_token_registry_has_eth_and_btc() -> ErrStr<()> {
        let registry = load_token_registry()?;
        for symbol in ["ETH", "BTC"] {
            assert!(registry.contains_key(symbol), "missing '{symbol}' in tokens.toml");
        }
        Ok(())
    }

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

    #[test]
    fn test_direction_symbols_match_pool_convention() {
        assert_eq!(Direction::Normal.symbols(), ("BTC", "ETH"));
        assert_eq!(Direction::Flipped.symbols(), ("ETH", "BTC"));
    }

    #[tokio::test]
    async fn test_run_trade_rejects_zero_or_negative_amounts() {
        assert!(run_trade(0.0, 1.0, Direction::Normal, 50, true).await.is_err());
        assert!(run_trade(1.0, 0.0, Direction::Normal, 50, true).await.is_err());
        assert!(run_trade(-1.0, 1.0, Direction::Normal, 50, true).await.is_err());
    }
}
//============================================================================
//----- FUNCTIONAL TESTS -------------------------------------------------------
//============================================================================
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{ create_testing, utils::now };

    const PIVOT_ROOT_URL: &str = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io";

    create_testing!("quiz11::arbitrage");

    run!("wallet_balance", " (real ETH read against dedicated test wallet, read-only)", {
        let registry = load_token_registry()?;
        let balance = now(wallet_balance(
            "0xd16E431b1363Ed90C4fD4906Cf7Fc33E51115429",
            "ETH",
            &registry,
        ))?;
        println!("\ttest wallet ETH balance: {balance:.6}");
    });

    run!("live_quote_eth_to_btc", " (real KyberSwap route, read-only, small ETH->BTC)", {
        let registry = load_token_registry()?;
        let quote = now(live_quote(&registry, "ETH", "BTC", 0.01))?;
        println!("\t0.01 ETH -> {:.8} BTC right now (router: {})", quote.amount_out, quote.router_address);
    });

    run!("live_quote_btc_to_eth", " (real KyberSwap route, read-only, small BTC->ETH)", {
        let registry = load_token_registry()?;
        let quote = now(live_quote(&registry, "BTC", "ETH", 0.0001))?;
        println!("\t0.0001 BTC -> {:.8} ETH right now (router: {})", quote.amount_out, quote.router_address);
    });

    run!("dry_run", " (real balance + quote, never touches keystore)", {
        let registry = load_token_registry()?;
        let available = now(wallet_balance(
            "0xd16E431b1363Ed90C4fD4906Cf7Fc33E51115429",
            "BTC",
            &registry,
        ))?;
        if available <= 0.0 {
            println!("\tskipping: test wallet currently has 0 BTC (last trade may have converted it to ETH)");
        } else {
            let amount = available * 0.1;
            now(run_trade(amount, 0.00000001, Direction::Normal, 50, true))?;
            println!("\tdry run completed without touching the keystore ({amount:.8} BTC checked)");
        }
    });

    run!("calls_batch_dry_run", " (real calls.csv fetch + read-only per-row checks)", {
        now(run_calls_batch(PIVOT_ROOT_URL, 50, true))?;
        println!("\tcalls batch dry run completed without touching the keystore");
    });
}
