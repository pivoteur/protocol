use chrono::NaiveDate;
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Write};
use book::{
        cli_utils::add_banner,
        currency::usd::mk_usd,
        err_utils::ErrStr,
        parse_args_add_banner,
        types::values::Value,
};
use libs::{
        fetchers::calls::{fetch_calls, fetch_call_data},
        processors::virtuals::compute_counter_offer,
        types::calls::Call,
};


//============================================================================
//----- Token Registry -------------------------------------------------------
//============================================================================
/// One entry per token symbol in tokens.toml. `address` is `None` only for
/// the chain's native gas asset (AVAX) — every ERC-20 must have an address.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TokenEntry {
    #[serde(default)]
    pub native: bool,
    #[serde(default)]
    pub address: Option<String>,
    pub decimals: u32,
}

pub type TokenRegistry = HashMap<String, TokenEntry>;

// Embedded at compile time from the file sitting next to this one, so this
// works no matter what directory `cargo run` is invoked from.
const TOKENS_TOML: &str = include_str!("tokens.toml");

pub fn load_token_registry() -> ErrStr<TokenRegistry> {
    toml::from_str(TOKENS_TOML).map_err(|e| format!("Failed to parse tokens.toml: {e}"))
}

/// Looks up a symbol in the registry with a loud error instead of a silent
/// default — an unrecognized token should stop the program, not fall
/// through as "0 balance."
pub fn token_entry<'a>(registry: &'a TokenRegistry, symbol: &str) -> ErrStr<&'a TokenEntry> {
    registry.get(symbol).ok_or_else(|| {
        format!("No tokens.toml entry for '{symbol}' — add one before checking this pool")
    })
}

//============================================================================
//----- Wallet Balance Check -------------------------------------------------
//============================================================================
// Everything in this section is read-only: eth_getBalance / eth_call queries
// against a public RPC. No key, no signature, nothing that can move funds.

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

/// Left-pads an address into the 32-byte word an ABI-encoded call expects.
fn pad_address_for_call(address: &str) -> String {
    let hex = address.trim_start_matches("0x").to_lowercase();
    format!("{hex:0>64}")
}

async fn native_balance(wallet_address: &str) -> ErrStr<u128> {
    let result = rpc_call("eth_getBalance", serde_json::json!([wallet_address, "latest"])).await?;
    hex_to_u128(&result)
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
    let raw = if entry.native {
        native_balance(wallet_address).await?
    } else {
        let addr = entry.address.as_deref().ok_or_else(|| {
            format!("'{symbol}' is not native but has no address in tokens.toml")
        })?;
        erc20_balance(wallet_address, addr).await?
    };
    Ok(raw as f64 / 10f64.powi(entry.decimals as i32))
}

/// Prompts for how much of `token` to commit. Typing "na" is a deliberate
/// skip (returns Ok(None)) — distinct from a parse error, which is still a
/// hard Err. Nothing further is checked or displayed for a skipped row.
fn prompt_for_commit_amount(token: &str, available: f64) -> ErrStr<Option<f32>> {
    println!("    Only {available:.6} {token} available. How much would you like to commit? (or 'na' to skip)");
    print!("    amount: ");
    io::stdout().flush().map_err(|e| format!("stdout flush failed: {e}"))?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("stdin read failed: {e}"))?;
    let trimmed = input.trim();

    if trimmed.eq_ignore_ascii_case("na") {
        return Ok(None);
    }

    let amount: f32 = trimmed
        .parse()
        .map_err(|_| format!("'{trimmed}' is not a valid number (or 'na' to skip)"))?;
    if amount <= 0.0 {
        return Err("Committed amount must be greater than zero".to_string());
    }
    if amount as f64 > available + 1e-6 {
        return Err(format!(
            "Cannot commit {amount:.6} {token} — only {available:.6} available"
        ));
    }
    Ok(Some(amount))
}

/// Checks that `pivot_token` is held and `pivot_amount` is available for a
/// candidate. If the pool wants more than is on hand, prompts for how much
/// to commit instead. Returns `Ok(None)` if the user typed "na" to skip,
/// or a hard `Err` if nothing is held at all.
pub async fn ensure_committed_amount(
    wallet_address: &str,
    registry: &TokenRegistry,
    candidate: &ArbitrageCandidate,
) -> ErrStr<Option<f32>> {
    let available = wallet_balance(wallet_address, &candidate.pivot_token, registry).await?;
    if available <= 0.0 {
        return Err(format!(
            "No {} found in wallet — cannot commit to this trade",
            candidate.pivot_token
        ));
    }
    if available >= candidate.pivot_amount as f64 {
        Ok(Some(candidate.pivot_amount))
    } else {
        prompt_for_commit_amount(&candidate.pivot_token, available)
    }
}

//============================================================================
//----- Live KyberSwap Quote -------------------------------------------------
//============================================================================
// Read-only route lookup — no signing, no submission. Tells you what a swap
// would actually return right now.

const KYBERSWAP_CHAIN: &str = "avalanche";
const NATIVE_TOKEN_PLACEHOLDER: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

#[derive(Debug, Deserialize)]
struct KyberRouteSummary {
    #[serde(rename = "amountOut")]
    amount_out: String,
}

#[derive(Debug, Deserialize)]
struct KyberRouteData {
    #[serde(rename = "routeSummary")]
    route_summary: KyberRouteSummary,
}

#[derive(Debug, Deserialize)]
struct KyberRouteResponse {
    data: Option<KyberRouteData>,
    message: Option<String>,
}

fn kyber_token_address(entry: &TokenEntry) -> ErrStr<String> {
    if entry.native {
        Ok(NATIVE_TOKEN_PLACEHOLDER.to_string())
    } else {
        entry
            .address
            .clone()
            .ok_or_else(|| "Token entry missing address".to_string())
    }
}

/// What KyberSwap would actually return right now for swapping `amount_in`
/// of `pivot_token` into `proposed_token`. Read-only — no signing.
pub async fn live_quote(
    registry: &TokenRegistry,
    pivot_token: &str,
    amount_in: f32,
    proposed_token: &str,
) -> ErrStr<f64> {
    let from_entry = token_entry(registry, pivot_token)?;
    let to_entry = token_entry(registry, proposed_token)?;
    let token_in = kyber_token_address(from_entry)?;
    let token_out = kyber_token_address(to_entry)?;
    let amount_in_base = (amount_in as f64 * 10f64.powi(from_entry.decimals as i32)) as u128;

    let url = format!(
        "https://aggregator-api.kyberswap.com/{KYBERSWAP_CHAIN}/api/v1/routes?tokenIn={token_in}&tokenOut={token_out}&amountIn={amount_in_base}"
    );

    let resp = reqwest::Client::new()
        .get(&url)
        .header("X-Client-Id", "pivoteur-arbitrage")
        .send()
        .await
        .map_err(|e| format!("KyberSwap route request failed: {e}"))?;

    let body: KyberRouteResponse = resp
        .json()
        .await
        .map_err(|e| format!("KyberSwap response did not parse: {e}"))?;

    match body.data {
        Some(d) => {
            let raw: u128 = d.route_summary.amount_out.parse().map_err(|_| {
                format!("Could not parse KyberSwap amountOut '{}'", d.route_summary.amount_out)
            })?;
            Ok(raw as f64 / 10f64.powi(to_entry.decimals as i32))
        }
        None => Err(format!(
            "KyberSwap returned no route ({pivot_token} -> {proposed_token}): {}",
            body.message.unwrap_or_else(|| "no message".to_string())
        )),
    }
}

//============================================================================
//----- Offrian-Rescaled Target ----------------------------------------------
//============================================================================
/// When the committed amount is smaller than the pool's full pivot_amount,
/// the CSV's proposed_amount was sized for the full trade and isn't a valid
/// comparison target. Rather than scale it down linearly (which ignores
/// price impact), this asks `offrian`'s own logic — fetch_call_data +
/// compute_counter_offer — for what the pool would actually offer at the
/// smaller volume. Read-only: another data fetch, no signing.
pub async fn rescaled_target(
    root_url: &str,
    candidate: &ArbitrageCandidate,
    committed_amount: f32,
) -> ErrStr<f32> {
    if (committed_amount - candidate.pivot_amount).abs() < f32::EPSILON {
        return Ok(candidate.proposed_amount);
    }
    if candidate.pivot_amount <= 0.0 {
        return Err("pivot_amount is zero — cannot derive a price ratio".to_string());
    }
    let price_per_unit = candidate.val1 / candidate.pivot_amount;
    let committed_usd = committed_amount * price_per_unit;

    let call_data = fetch_call_data(root_url, candidate.ix, false).await?;
    let rescaled_call = compute_counter_offer(&call_data, &mk_usd(committed_usd), false)?;
    Ok(rescaled_call.proposed_amount)
}

//============================================================================
//----- Trade Check (read-only recommendation) -------------------------------
//============================================================================
/// Combines the wallet check, the offrian-rescaled target, and the live
/// KyberSwap quote into one read-only trade preview per row. Shows exactly
/// what the trade WOULD look like if it happened. Signs and sends nothing.
pub async fn check_trade(
    root_url: &str,
    wallet_address: &str,
    registry: &TokenRegistry,
    candidate: &ArbitrageCandidate,
) -> ErrStr<()> {
    let committed = match ensure_committed_amount(wallet_address, registry, candidate).await? {
        Some(amount) => amount,
        None => {
            println!("ix={}: skipped by user (na)", candidate.ix);
            return Ok(());
        }
    };

    let target = rescaled_target(root_url, candidate, committed).await?;
    let quote = live_quote(registry, &candidate.pivot_token, committed, &candidate.proposed_token).await?;
    let pct_of_target = if target > 0.0 { (quote / target as f64) * 100.0 } else { 0.0 };

    println!("ix={}: TRADE PREVIEW (nothing signed or sent)", candidate.ix);
    println!("    Send:     {committed:.6} {}", candidate.pivot_token);
    println!("    Receive:  {quote:.6} {}  (live KyberSwap quote)", candidate.proposed_token);
    println!("    Target:   {target:.6} {}  (offrian-rescaled for this size)", candidate.proposed_token);

    if quote >= target as f64 {
        println!("    Result:   WOULD CLEAR target ({pct_of_target:.1}% of target)");
    } else {
        println!("    Result:   would NOT clear target ({pct_of_target:.1}% of target)");
    }
    Ok(())
}

//============================================================================
//----- Data Structure -------------------------------------------------------
//============================================================================
/// Exact columns requested and everything else on `Call` is ignored here.
#[derive(Debug, Clone, PartialEq)]
pub struct ArbitrageCandidate {
    pub ix:              usize,
    pub pool:             String,
    pub close_date:       NaiveDate,
    pub pivot_token:      String,
    pub pivot_amount:     f32,
    pub val1:             f32,
    pub proposed_token:   String,
    pub proposed_amount:  f32,
    pub gain_10_percent:  f32,
    pub roi:              f32,
}

const CALLS_ROOT_URL: &str = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io";

/// Accepts either "calls" or a full URL passed through as an argument.
/// So user doesn't have to type the full URL every time, but can if they want to.
fn resolve_root_url(input: &str) -> String {
    if input.eq_ignore_ascii_case("calls") {
        CALLS_ROOT_URL.to_string()
    } else {
        input.to_string()
    }
}

impl From<&Call> for ArbitrageCandidate {
    fn from(call: &Call) -> Self {
        ArbitrageCandidate {
            ix:              call.ix,
            pool:            call.pool.pool_name(),
            close_date:      call.close_date,
            pivot_token:     call.pivot_token.clone(),
            pivot_amount:    call.pivot_amount,
            val1:            call.val1.amount(),
            proposed_token:  call.proposed_token.clone(),
            proposed_amount: call.proposed_amount,
            gain_10_percent: call.gain_10_percent,
            roi:             call.roi.value(),
        }
    }
}

impl ArbitrageCandidate {
    pub fn header() -> String {
        [
            "IX", "POOL", "CLOSE DATE", "PIVOT", "PIVOT AMT", "VAL1",
            "PROPOSED", "PROP AMT", "GAIN 10%", "ROI",
        ].join(", ")
    }
}

impl std::fmt::Display for ArbitrageCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {:.7}, ${:.2}, {}, {:.8}, {:.6}, {:.2}%",
            self.ix,
            self.pool,
            self.close_date,
            self.pivot_token,
            self.pivot_amount,
            self.val1,
            self.proposed_token,
            self.proposed_amount,
            self.gain_10_percent,
            self.roi * 100.0,
        )
    }
}

//============================================================================
//----- Core: fetch calls.csv and map to candidates --------------------------
//============================================================================
pub async fn fetch_candidates(root_url: &str) -> ErrStr<Vec<ArbitrageCandidate>> {
    let calls = fetch_calls(root_url).await?;
    Ok(calls.iter().map(ArbitrageCandidate::from).collect())
}

pub async fn process(root_url: &str) -> ErrStr<()> {
    let wallet_address = wallet_address_from_env()?;
    let registry = load_token_registry()?;
    let candidates = fetch_candidates(root_url).await?;

    println!("{}", ArbitrageCandidate::header());
    for c in &candidates {
        println!("{c}");
    }
    println!("\n{} candidate(s) parsed from calls.csv\n", candidates.len());

    println!("=== Trade check ({wallet_address}) — read-only, nothing signed or sent ===");
    for c in &candidates {
        if let Err(e) = check_trade(root_url, &wallet_address, &registry, c).await {
            println!("ix={}: SKIP — {e}", c.ix);
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "arbitrage")]
#[command(version = "0.1.4")]
struct Args {
    #[arg(value_name = "dusk's output file name")]
    root_url: String,
}

pub async fn runoff_with_args() -> ErrStr<()> {
    let args = parse_args_add_banner!(Args);
    let root_url = resolve_root_url(&args.root_url);
    process(&root_url).await
}

//============================================================================
//----- UNIT TESTS -----------------------------------------------------------
//============================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;
    use libs::types::calls::parse_calls;


    // One header row + one data row, verbatim shape from a real calls.csv.
    fn sample_csv() -> String {
        "ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,\
         from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,\
         pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,\
         proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,\
         roi,apr\n\
         1,BTC+USDC,10,2026-04-16,2026-04-15,27;29,8,2026-07-09,BTC,Avalanche,\
         0,0.452206,$81812.00,$36995.88,0.4974266,USDC,Avalanche,$1.00,37005.758,\
         BTC,Avalanche,$62714.00,0.5899885,30.47%,130.84%\n".to_string()
    }

    fn first_candidate() -> ErrStr<ArbitrageCandidate> {
        let calls = parse_calls(&sample_csv())?;
        let call = calls.first().ok_or("no rows parsed from sample csv")?;
        Ok(ArbitrageCandidate::from(call))
    }

    #[test]
    fn test_from_call_maps_requested_fields_only() -> ErrStr<()> {
        let candidate = first_candidate()?;
        assert_eq!(candidate.ix, 1);
        assert_eq!(candidate.pool, "BTC+USDC");
        assert_eq!(candidate.close_date, NaiveDate::from_ymd_opt(2026, 7, 9).unwrap());
        assert_eq!(candidate.pivot_token, "USDC");
        assert!((candidate.pivot_amount - 37005.758).abs() < 0.01, "{}", candidate.pivot_amount);
        assert!((candidate.val1 - 36995.88).abs() < 0.01, "{}", candidate.val1);
        assert_eq!(candidate.proposed_token, "BTC");
        assert!((candidate.proposed_amount - 0.5899885).abs() < 0.0001, "{}", candidate.proposed_amount);
        assert!((candidate.gain_10_percent - 0.4974266).abs() < 0.0001, "{}", candidate.gain_10_percent);
        assert!((candidate.roi - 0.3047).abs() < 0.001, "{}", candidate.roi);
        Ok(())
    }

    #[test]
    fn test_display_includes_all_requested_columns() -> ErrStr<()> {
        let candidate = first_candidate()?;
        let shown = format!("{candidate}");
        for expected in [
            "BTC+USDC", "2026-07-09", "USDC", "BTC", "30.47%",
        ] {
            assert!(shown.contains(expected), "missing '{expected}' in: {shown}");
        }
        Ok(())
    }

    #[test]
    fn test_header_labels_every_column() {
        let header = ArbitrageCandidate::header();
        for label in ["IX", "POOL", "CLOSE DATE", "PIVOT", "VAL1", "PROPOSED", "GAIN 10%", "ROI"] {
            assert!(header.contains(label), "missing '{label}' in header: {header}");
        }
    }

    #[test]
    fn test_ignores_columns_not_requested() -> ErrStr<()> {
        // apr, quote1, amount1, virtual, open_pivots, etc. are present in the
        // source CSV but must not appear anywhere on the candidate.
        let shown = format!("{}", first_candidate()?);
        for excluded in ["apr=", "quote1", "virtual", "open_pivots"] {
            assert!(!shown.contains(excluded), "unexpected '{excluded}' leaked into: {shown}");
        }
        Ok(())
    }

    #[test]
    fn test_resolve_root_url_shorthand_and_passthrough() {
        assert_eq!(resolve_root_url("calls"), CALLS_ROOT_URL);
        assert_eq!(resolve_root_url("CALLS"), CALLS_ROOT_URL);
        assert_eq!(
            resolve_root_url("https://example.com/other-fork"),
            "https://example.com/other-fork"
        );
    }

    #[test]
    fn test_load_token_registry_parses_the_real_tokens_toml() -> ErrStr<()> {
        let registry = load_token_registry()?;
        for symbol in ["AVAX", "BTC", "ETH", "USDC", "UNDEAD"] {
            assert!(registry.contains_key(symbol), "missing '{symbol}' in tokens.toml");
        }
        Ok(())
    }

    #[test]
    fn test_avax_is_native_with_no_address_required() -> ErrStr<()> {
        let registry = load_token_registry()?;
        let avax = token_entry(&registry, "AVAX")?;
        assert!(avax.native);
        assert_eq!(avax.decimals, 18);
        Ok(())
    }

    #[test]
    fn test_erc20_entries_have_addresses() -> ErrStr<()> {
        let registry = load_token_registry()?;
        for symbol in ["BTC", "ETH"] {
            let entry = token_entry(&registry, symbol)?;
            assert!(!entry.native);
            let addr = entry.address.as_deref().unwrap_or("");
            assert!(addr.starts_with("0x") && addr.len() == 42,
                "'{symbol}' address looks malformed: '{addr}'");
        }
        Ok(())
    }

    #[test]
    fn test_unknown_token_is_a_loud_error_not_a_silent_default() -> ErrStr<()> {
        let registry = load_token_registry()?;
        let result = token_entry(&registry, "NOT_A_REAL_TOKEN");
        assert!(result.is_err(), "expected an error for an unregistered symbol");
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
    async fn test_rescaled_target_full_commit_skips_network() -> ErrStr<()> {
        // When the committed amount equals the pool's own pivot_amount, no
        // offrian call is needed — the CSV's proposed_amount already applies.
        // Passing a garbage root_url proves no network call happens here.
        let candidate = first_candidate()?;
        let target = rescaled_target("not-a-real-url", &candidate, candidate.pivot_amount).await?;
        assert_eq!(target, candidate.proposed_amount);
        Ok(())
    }
}

//============================================================================
//----- FUNCTIONAL TESTS -----------------------------------------------------
//============================================================================
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{ create_testing, utils::now };
    use libs::fetchers::test_helpers::test_functions::marshall;


    create_testing!("quiz11::arbitrage");

    run!("fetch_candidates", " (against real calls.csv)", {
        let (root_url, _aliases) = marshall()?;
        let candidates = now(fetch_candidates(&root_url))?;
        println!("\tfetched {} candidate(s)", candidates.len());
        for c in &candidates {
            println!("\t{c}");
        }
    });

    run!("wallet_balance", " (real AVAX read against dedicated test wallet, read-only)", {
        let registry = load_token_registry()?;
        let balance = now(wallet_balance(
            "0xd16E431b1363Ed90C4fD4906Cf7Fc33E51115429",
            "AVAX",
            &registry,
        ))?;
        println!("\ttest wallet AVAX balance: {balance:.6}");
    });

    run!("live_quote", " (real KyberSwap route, read-only, small AVAX->USDC)", {
        let registry = load_token_registry()?;
        let quote = now(live_quote(&registry, "AVAX", 1.0, "USDC"))?;
        println!("\t1 AVAX -> {quote:.6} USDC right now");
    });
}
