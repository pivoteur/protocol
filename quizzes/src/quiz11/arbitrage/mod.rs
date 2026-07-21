use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, TransactionRequest},
};
use book::{
        cli_utils::add_banner,
        err_utils::ErrStr,
        parse_args_add_banner,
};


//============================================================================
//----- Token Registry --------------------------------------------------------
//============================================================================
/// Just ETH and BTC for this program. tokens.toml lives alongside this file.
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
    registry.get(symbol).ok_or_else(|| {
        format!("No tokens.toml entry for '{symbol}' — add one before checking this pool")
    })
}

//============================================================================
//----- Wallet Balance Check --------------------------------------------------
//============================================================================
// Read-only: eth_getBalance / eth_call queries against a public RPC.
// No key, no signature, nothing that can move funds. Kept as-is from the
// earlier version — this is the piece worth keeping.

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
    let resp = reqwest::Client::new()
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
    format!("{hex:0>64}")
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
// ETH -> BTC swap would actually return right now.

const KYBERSWAP_CHAIN: &str = "avalanche";
const AVALANCHE_CHAIN_ID: u64 = 43114;

/// A live quote plus everything needed to actually build and sign the swap
/// afterward. Parsed defensively from raw JSON rather than a rigid struct,
/// since the exact response shape wasn't something I could fully verify
/// from documentation alone.
pub struct KyberQuote {
    pub amount_out: f64,
    pub route_summary_raw: serde_json::Value,
    pub router_address: String,
}

/// What KyberSwap would actually return right now for swapping `eth_amount`
/// ETH into BTC. Read-only — no signing.
pub async fn live_quote(registry: &TokenRegistry, eth_amount: f32) -> ErrStr<KyberQuote> {
    let from_entry = token_entry(registry, "ETH")?;
    let to_entry = token_entry(registry, "BTC")?;
    let token_in = from_entry.address.as_deref().ok_or("ETH missing address")?;
    let token_out = to_entry.address.as_deref().ok_or("BTC missing address")?;
    let amount_in_base = (eth_amount as f64 * 10f64.powi(from_entry.decimals as i32)).round() as u128;

    let url = format!(
        "https://aggregator-api.kyberswap.com/{KYBERSWAP_CHAIN}/api/v1/routes?tokenIn={token_in}&tokenOut={token_out}&amountIn={amount_in_base}"
    );

    let resp = reqwest::Client::new()
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
        .ok_or_else(|| format!("KyberSwap returned no route (ETH -> BTC). Raw: {raw_body}"))?;
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
// deliberately loud on failure rather than guessing its way through.

fn pad_u256_for_call(amount: u128) -> String {
    format!("{amount:064x}")
}

/// Loads the encrypted keystore, prompts for its password at runtime (never
/// stored, never logged), and verifies the derived address actually matches
/// WALLET_ADDRESS before handing back a signer — refuses to proceed on a
/// mismatch rather than silently signing with the wrong key.
async fn load_signer(expected_address: &str) -> ErrStr<LocalWallet> {
    let keystore_path = std::env::var("KEYSTORE_PATH").map_err(|_| {
        "Missing required env var: KEYSTORE_PATH (full path to the encrypted keystore file)".to_string()
    })?;
    let password = rpassword::prompt_password("Keystore password: ")
        .map_err(|e| format!("Could not read password: {e}"))?;
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

/// Approves the router for EXACTLY this trade's amount — never a standing
/// allowance. The router can never pull more than what's approved here.
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
    let tx = TransactionRequest::new().to(to).data(data);

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

/// Asks KyberSwap to encode the actual swap calldata for the route we
/// already quoted. Prints the raw response every time — verify it before
/// trusting it, since the exact schema wasn't something I could fully
/// confirm from documentation alone.
async fn kyberswap_build(route_summary_raw: &serde_json::Value, sender: &str) -> ErrStr<(String, String)> {
    let body = serde_json::json!({
        "routeSummary": route_summary_raw,
        "sender": sender,
        "recipient": sender,
        "slippageTolerance": 50
    });

    let resp = reqwest::Client::new()
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

/// Signs and sends the swap transaction. Returns the tx hash on success;
/// hard errors on revert, drop, or replacement rather than reporting a
/// false success.
async fn send_swap_tx(
    client: &SignerMiddleware<Provider<Http>, LocalWallet>,
    router: &str,
    calldata_hex: &str,
) -> ErrStr<String> {
    let to = Address::from_str(router).map_err(|e| format!("Bad router address: {e}"))?;
    let data = Bytes::from_str(calldata_hex).map_err(|e| format!("Bad calldata from KyberSwap: {e}"))?;
    let tx = TransactionRequest::new().to(to).data(data);

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
//----- Trade Flow -------------------------------------------------------------
//============================================================================
/// The whole flow: check wallet, get a live quote, check it against your
/// floor. Running this program IS the approval — there's no interactive
/// prompt. A missed floor is a hard error, not a question: no funds used.
pub async fn run_trade(eth_amount: f32, min_btc_floor: f32) -> ErrStr<()> {
    if eth_amount <= 0.0 {
        return Err("eth_amount must be greater than zero".to_string());
    }
    if min_btc_floor <= 0.0 {
        return Err("min_btc_floor must be greater than zero".to_string());
    }

    let wallet_address = wallet_address_from_env()?;
    let registry = load_token_registry()?;

    let available = wallet_balance(&wallet_address, "ETH", &registry).await?;
    println!("Wallet ({wallet_address}): {available:.6} ETH available");
    if available + 1e-6 < eth_amount as f64 {
        return Err(format!(
            "Insufficient ETH — need {eth_amount:.6}, only {available:.6} available. \
             That's not happening. No funds used."
        ));
    }

    let quote = live_quote(&registry, eth_amount).await?;
    println!("Live quote: {eth_amount:.6} ETH -> {:.8} BTC right now", quote.amount_out);
    println!("Your floor: {min_btc_floor:.8} BTC");

    if quote.amount_out < min_btc_floor as f64 {
        return Err(format!(
            "Quote ({:.8} BTC) is below your floor ({min_btc_floor:.8} BTC). \
             That's not happening. No funds used.",
            quote.amount_out
        ));
    }

    println!(">>> Quote clears your floor. Proceeding to execute.");

    let signer = load_signer(&wallet_address).await?;
    let provider = Provider::<Http>::try_from(AVALANCHE_RPC)
        .map_err(|e| format!("Could not create RPC provider: {e}"))?;
    let client = SignerMiddleware::new(provider, signer);

    let eth_entry = token_entry(&registry, "ETH")?;
    let eth_addr = eth_entry.address.as_deref().ok_or("ETH missing address")?.to_string();
    let amount_base = (eth_amount as f64 * 10f64.powi(eth_entry.decimals as i32)).round() as u128;

    println!(">>> Approving exact amount ({eth_amount:.6} ETH) for the router...");
    approve_exact_amount(&client, &eth_addr, &quote.router_address, amount_base).await?;

    println!(">>> Requesting swap calldata from KyberSwap...");
    let (router, calldata) = kyberswap_build(&quote.route_summary_raw, &wallet_address).await?;

    println!(">>> Sending swap transaction...");
    let tx_hash = send_swap_tx(&client, &router, &calldata).await?;

    println!(">>> Trade complete. Tx hash: {tx_hash}");
    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "arbitrage")]
#[command(version = "0.3.0")]
struct Args {
    /// Amount of ETH to trade from your wallet
    eth_amount: f32,
    /// Minimum acceptable BTC back — trade won't proceed below this
    min_btc_floor: f32,
}

pub async fn runoff_with_args() -> ErrStr<()> {
    let args = parse_args_add_banner!(Args);
    run_trade(args.eth_amount, args.min_btc_floor).await
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

    #[tokio::test]
    async fn test_run_trade_rejects_zero_or_negative_amounts() {
        assert!(run_trade(0.0, 1.0).await.is_err());
        assert!(run_trade(1.0, 0.0).await.is_err());
        assert!(run_trade(-1.0, 1.0).await.is_err());
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

    run!("live_quote", " (real KyberSwap route, read-only, small ETH->BTC)", {
        let registry = load_token_registry()?;
        let quote = now(live_quote(&registry, 0.01))?;
        println!("\t0.01 ETH -> {:.8} BTC right now (router: {})", quote.amount_out, quote.router_address);
    });
}
