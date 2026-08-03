use clap::{Parser, Subcommand, ValueEnum};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use book::{
        cli_utils::generate_banner,
        err_utils::ErrStr,
        parse_args_add_banner,
};
use libs::{
    fetchers::calls::fetch_calls,
    types::calls::Call,
    auto_trading::{
        TokenRegistry, parse_token_registry, token_entry,
        wallet_address_from_env, wallet_balance, live_quote, execute_trade,
    },
};


//============================================================================
//----- Token Registry --------------------------------------------------------
//============================================================================
/// Just BTC and ETH for this program. tokens.toml lives alongside this file.
/// TokenRegistry/token_entry themselves now live in libs::auto_trading —
/// only the embed + parse of *this binary's own* tokens.toml stays local.
const TOKENS_TOML: &str = include_str!("tokens.toml");

pub fn load_token_registry() -> ErrStr<TokenRegistry> {
    parse_token_registry(TOKENS_TOML)
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
//----- Trade Log --------------------------------------------------------------
//============================================================================
const TRADE_LOG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/quiz12/arbitrage/arbitrage_trades.log");

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
// execute_trade itself now lives in libs::auto_trading (imported above) —
// keystore unlock through swap submission is identical for every
// auto-trading binary, so it's not duplicated here.

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
        Ok((tx_hash, _gas_avax)) => {
            // gas tracking isn't part of arbitrage's log format (yet) —
            // available here if that changes later.
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
#[command(version = "0.10.0")]
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

    // test_hex_to_u128_* and test_pad_address_for_call_* moved to
    // libs::auto_trading's own test module — they test private helpers
    // that live there now, not here.

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

    create_testing!("quiz12::arbitrage");

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
