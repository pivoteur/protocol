use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use clap::Parser;
use book::{
        cli_utils::generate_banner,
        err_utils::ErrStr,
        parse_args_add_banner,
};
use libs::auto_trading::{
            TokenRegistry, 
            parse_token_registry, 
            wallet_address_from_env,
            wallet_balance, 
            live_quote, 
            execute_trade,
};

//============================================================================
//----- Token Registry --------------------------------------------------------
//============================================================================
const TOKENS_TOML: &str = include_str!("tokens.toml");

pub fn load_token_registry() -> ErrStr<TokenRegistry> {
    parse_token_registry(TOKENS_TOML)
}

//============================================================================
//----- Fixed Trade Sizes ------------------------------------------------------
//============================================================================
const UNDEAD_TRADE_AMOUNT: f64 = 500_000.0;
const BTC_TRADE_AMOUNT: f64 = 0.005;
const SLIPPAGE_BPS: u16 = 200;

// execute_trade requires min_floor > 0. Opens have no real floor, so this
// is close enough to zero to never meaningfully constrain anything.
const NO_REAL_FLOOR: f64 = 0.000_000_01;

//============================================================================
//----- Trade Log — format & replay --------------------------------------------
//============================================================================
//   OPEN:  timestamp,OPEN,pivot_id,prim,prim_amount,proper,proper_amount,gas_avax,tx_hash
//   CLOSE: timestamp,CLOSE,pivot_id,close_id,proper_amount,gain,roi,apr,gas_avax,tx_hash
//   CHECK: timestamp,CHECK,pivot_id,not_closed
//
// proper_amount is the ACTUAL amount received (balance delta), not the quote.
const TRADE_LOG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/quiz12/tva/tva-trades.log");

#[derive(Debug, Clone)]
struct OpenPivot {
    pivot_id:      u32,
    opened_at:     u64,
    prim:          String,
    prim_amount:   f64,
    proper:        String,
    proper_amount: f64,
}

fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn append_log(line: &str) {
    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(TRADE_LOG_PATH)
        .and_then(|mut f| writeln!(f, "{line}"));
    if let Err(e) = result {
        eprintln!("Warning: could not write to trade log ({TRADE_LOG_PATH}): {e}");
    }
}

fn log_open(pivot_id: u32, prim: &str, prim_amount: f64, proper: &str, proper_amount: f64, gas_avax: f64, tx_hash: &str) {
    append_log(&format!(
        "{},OPEN,{pivot_id},{prim},{prim_amount:.8},{proper},{proper_amount:.8},{gas_avax:.8},{tx_hash}",
        now_ts()
    ));
}

fn log_close(pivot_id: u32, close_id: u32, proper_amount: f64, gain: f64, roi: f64, apr: f64, gas_avax: f64, tx_hash: &str) {
    append_log(&format!(
        "{},CLOSE,{pivot_id},{close_id},{proper_amount:.8},{gain:.8},{roi:.6},{apr:.6},{gas_avax:.8},{tx_hash}",
        now_ts()
    ));
}

fn log_check_not_closed(pivot_id: u32) {
    append_log(&format!("{},CHECK,{pivot_id},not_closed", now_ts()));
}

/// Missing log file means a fresh start (ids begin at 1).
fn replay_log(path: &str) -> ErrStr<(Vec<OpenPivot>, u32, u32)> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Ok((Vec::new(), 1, 1)),
    };
    let reader = BufReader::new(file);

    let mut opens: Vec<OpenPivot> = Vec::new();
    let mut closed_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut max_pivot_id: u32 = 0;
    let mut max_close_id: u32 = 0;

    for (line_no, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Could not read {path} at line {}: {e}", line_no + 1))?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        match fields.get(1) {
            Some(&"OPEN") => {
                if fields.len() < 9 {
                    return Err(format!("malformed OPEN line at {path}:{}: '{line}'", line_no + 1));
                }
                let opened_at: u64 = fields[0].parse()
                    .map_err(|_| format!("bad timestamp at {path}:{}: '{line}'", line_no + 1))?;
                let pivot_id: u32 = fields[2].parse()
                    .map_err(|_| format!("bad pivot_id at {path}:{}: '{line}'", line_no + 1))?;
                let prim_amount: f64 = fields[4].parse()
                    .map_err(|_| format!("bad prim_amount at {path}:{}: '{line}'", line_no + 1))?;
                let proper_amount: f64 = fields[6].parse()
                    .map_err(|_| format!("bad proper_amount at {path}:{}: '{line}'", line_no + 1))?;
                let _gas_avax: f64 = fields[7].parse()
                    .map_err(|_| format!("bad gas_avax at {path}:{}: '{line}'", line_no + 1))?;
                max_pivot_id = max_pivot_id.max(pivot_id);
                opens.push(OpenPivot {
                    pivot_id,
                    opened_at,
                    prim: fields[3].to_string(),
                    prim_amount,
                    proper: fields[5].to_string(),
                    proper_amount,
                });
            }
            Some(&"CLOSE") => {
                if fields.len() < 4 {
                    return Err(format!("malformed CLOSE line at {path}:{}: '{line}'", line_no + 1));
                }
                let pivot_id: u32 = fields[2].parse()
                    .map_err(|_| format!("bad pivot_id at {path}:{}: '{line}'", line_no + 1))?;
                let close_id: u32 = fields[3].parse()
                    .map_err(|_| format!("bad close_id at {path}:{}: '{line}'", line_no + 1))?;
                closed_ids.insert(pivot_id);
                max_close_id = max_close_id.max(close_id);
            }
            Some(&"CHECK") => {}
            _ => return Err(format!("unrecognized log line at {path}:{}: '{line}'", line_no + 1)),
        }
    }

    let still_open: Vec<OpenPivot> = opens.into_iter().filter(|p| !closed_ids.contains(&p.pivot_id)).collect();
    Ok((still_open, max_pivot_id + 1, max_close_id + 1))
}

//============================================================================
//----- One Trading Cycle -------------------------------------------------------
//============================================================================
enum AttemptOutcome {
    NotCleared,
    DryRunWouldClear { quoted_amount_out: f64 },
    Executed { tx_hash: String, actual_received: f64, gas_avax: f64 },
}

async fn attempt_trade_with_actual_amount(
    wallet_address: &str,
    registry: &TokenRegistry,
    from_symbol: &str,
    to_symbol: &str,
    amount: f64,
    min_floor: f64,
    dry_run: bool,
) -> ErrStr<AttemptOutcome> {
    let quote = live_quote(registry, from_symbol, to_symbol, amount).await?;
    if quote.amount_out <= min_floor {
        return Ok(AttemptOutcome::NotCleared);
    }

    if dry_run {
        return Ok(AttemptOutcome::DryRunWouldClear { quoted_amount_out: quote.amount_out });
    }

    let balance_before = wallet_balance(wallet_address, to_symbol, registry).await?;
    let (tx_hash, gas_avax) = execute_trade(wallet_address, registry, from_symbol, to_symbol, amount, min_floor, SLIPPAGE_BPS, "TVA_KEYSTORE_PATH").await?;
    let balance_after = wallet_balance(wallet_address, to_symbol, registry).await?;
    let actual_received = balance_after - balance_before;

    Ok(AttemptOutcome::Executed { tx_hash, actual_received, gas_avax })
}

pub async fn run_cycle(dry_run: bool) -> ErrStr<()> {
    let wallet_address = wallet_address_from_env("TVA_WALLET_ADDRESS")?;

    println!("Using wallet: {wallet_address}");
    if let Ok(expected) = std::env::var("TVA_EXPECTED_WALLET") {
        if !expected.eq_ignore_ascii_case(&wallet_address) {
            return Err(format!(
                "WALLET_ADDRESS ({wallet_address}) does not match TVA_EXPECTED_WALLET ({expected}) — refusing to run."
            ));
        }
    }

    let registry = load_token_registry()?;
    let (open_pivots, mut next_pivot_id, mut next_close_id) = replay_log(TRADE_LOG_PATH)?;

    println!("=== tvá cycle starting: {} open pivot(s) ===", open_pivots.len());

    let mut remaining_open: Vec<OpenPivot> = Vec::new();
    for pivot in open_pivots {
        println!(
            "--- Checking pivot #{}: {:.8} {} -> {:.8} {} (opened {}) ---",
            pivot.pivot_id, pivot.prim_amount, pivot.prim, pivot.proper_amount, pivot.proper, pivot.opened_at
        );

        match attempt_trade_with_actual_amount(
            &wallet_address, &registry, &pivot.proper, &pivot.prim,
            pivot.proper_amount, pivot.prim_amount, dry_run,
        ).await {
            Ok(AttemptOutcome::Executed { tx_hash, actual_received, gas_avax }) => {
                let gain = actual_received - pivot.prim_amount;
                let roi = gain / pivot.prim_amount;
                let days_held = (now_ts().saturating_sub(pivot.opened_at)) as f64 / 86_400.0;
                let apr = if days_held > 0.0 { roi * 365.0 / days_held } else { 0.0 };
                println!(
                    ">>> Closed pivot #{}: received {actual_received:.8} {} (gain {gain:.8}, roi {:.4}%, apr {:.2}%, gas {gas_avax:.8} AVAX)",
                    pivot.pivot_id, pivot.prim, roi * 100.0, apr * 100.0
                );
                log_close(pivot.pivot_id, next_close_id, actual_received, gain, roi, apr, gas_avax, &tx_hash);
                next_close_id += 1;
            }
            Ok(AttemptOutcome::DryRunWouldClear { quoted_amount_out }) => {
                println!(
                    ">>> DRY RUN: would close here — quoted {quoted_amount_out:.8} {} > original {:.8} {}. No funds moved.",
                    pivot.prim, pivot.prim_amount, pivot.prim
                );
                remaining_open.push(pivot);
            }
            Ok(AttemptOutcome::NotCleared) => {
                println!(">>> Not clearing yet — staying open.");
                if !dry_run {
                    log_check_not_closed(pivot.pivot_id);
                }
                remaining_open.push(pivot);
            }
            Err(e) => {
                println!(">>> Close attempt for pivot #{} failed, staying open: {e}", pivot.pivot_id);
                remaining_open.push(pivot);
            }
        }
    }

    let btc_committed: f64 = remaining_open.iter().filter(|p| p.proper == "BTC").map(|p| p.proper_amount).sum();
    let undead_committed: f64 = remaining_open.iter().filter(|p| p.proper == "UNDEAD").map(|p| p.proper_amount).sum();

    let btc_balance = wallet_balance(&wallet_address, "BTC", &registry).await?;
    let undead_balance = wallet_balance(&wallet_address, "UNDEAD", &registry).await?;
    let btc_available = btc_balance - btc_committed;
    let undead_available = undead_balance - undead_committed;

    println!("BTC:    {btc_balance:.8} total, {btc_committed:.8} committed, {btc_available:.8} available");
    println!("UNDEAD: {undead_balance:.2} total, {undead_committed:.2} committed, {undead_available:.2} available");

    if btc_available > BTC_TRADE_AMOUNT {
        println!("--- Opening pivot #{next_pivot_id}: {BTC_TRADE_AMOUNT} BTC -> UNDEAD ---");
        let attempted_id = next_pivot_id;
        match attempt_trade_with_actual_amount(
            &wallet_address, &registry, "BTC", "UNDEAD", BTC_TRADE_AMOUNT, NO_REAL_FLOOR, dry_run,
        ).await {
            Ok(AttemptOutcome::Executed { tx_hash, actual_received, gas_avax }) => {
                println!(">>> Opened: {BTC_TRADE_AMOUNT} BTC -> {actual_received:.2} UNDEAD (gas {gas_avax:.8} AVAX)");
                log_open(attempted_id, "BTC", BTC_TRADE_AMOUNT, "UNDEAD", actual_received, gas_avax, &tx_hash);
            }
            Ok(AttemptOutcome::DryRunWouldClear { quoted_amount_out }) => {
                println!(">>> DRY RUN: would open — {BTC_TRADE_AMOUNT} BTC -> ~{quoted_amount_out:.2} UNDEAD. No funds moved.");
            }
            Ok(AttemptOutcome::NotCleared) => println!(">>> Unexpected: open quote didn't clear the near-zero floor — check the pool."),
            Err(e) => println!(">>> Open (BTC->UNDEAD) failed: {e}"),
        }
        // Advances once an attempt is actually made, regardless of outcome —
        // otherwise a dry-run or failed first attempt leaves the second
        // attempt showing the same id, even though they're two distinct
        // attempts. A skipped id (attempt failed, never logged) is
        // harmless: replay_log only tracks the max logged id, not a count.
        next_pivot_id += 1;
    } else {
        println!("Not enough BTC available to open ({btc_available:.8} <= {BTC_TRADE_AMOUNT}).");
    }

    if undead_available > UNDEAD_TRADE_AMOUNT {
        println!("--- Opening pivot #{next_pivot_id}: {UNDEAD_TRADE_AMOUNT} UNDEAD -> BTC ---");
        match attempt_trade_with_actual_amount(
            &wallet_address, &registry, "UNDEAD", "BTC", UNDEAD_TRADE_AMOUNT, NO_REAL_FLOOR, dry_run,
        ).await {
            Ok(AttemptOutcome::Executed { tx_hash, actual_received, gas_avax }) => {
                println!(">>> Opened: {UNDEAD_TRADE_AMOUNT} UNDEAD -> {actual_received:.8} BTC (gas {gas_avax:.8} AVAX)");
                log_open(next_pivot_id, "UNDEAD", UNDEAD_TRADE_AMOUNT, "BTC", actual_received, gas_avax, &tx_hash);
            }
            Ok(AttemptOutcome::DryRunWouldClear { quoted_amount_out }) => {
                println!(">>> DRY RUN: would open — {UNDEAD_TRADE_AMOUNT} UNDEAD -> ~{quoted_amount_out:.8} BTC. No funds moved.");
            }
            Ok(AttemptOutcome::NotCleared) => println!(">>> Unexpected: open quote didn't clear the near-zero floor — check the pool."),
            Err(e) => println!(">>> Open (UNDEAD->BTC) failed: {e}"),
        }
    } else {
        println!("Not enough UNDEAD available to open ({undead_available:.2} <= {UNDEAD_TRADE_AMOUNT}).");
    }

    println!("=== tvá cycle complete ===");
    Ok(())
}

//============================================================================
//----- CLI --------------------------------------------------------------------
//============================================================================
#[derive(Debug, Parser)]
#[command(name = "tva")]
#[command(version = "0.4.0")]
struct Args {
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

pub async fn runoff_with_args() -> ErrStr<()> {
    let args = parse_args_add_banner!(Args);
    run_cycle(args.dry_run).await
}

//============================================================================
//----- UNIT TESTS -------------------------------------------------------------
//============================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;


    #[test]
    fn test_load_token_registry_has_btc_undead_avax() -> ErrStr<()> {
        let registry = load_token_registry()?;
        for symbol in ["BTC", "UNDEAD", "AVAX"] {
            assert!(registry.contains_key(symbol), "missing '{symbol}' in tokens.toml");
        }
        Ok(())
    }

    #[test]
    fn test_replay_log_missing_file_is_fresh_start() -> ErrStr<()> {
        let (opens, next_pivot, next_close) = replay_log("/tmp/definitely_does_not_exist.log")?;
        assert!(opens.is_empty());
        assert_eq!(next_pivot, 1);
        assert_eq!(next_close, 1);
        Ok(())
    }

    #[test]
    fn test_replay_log_open_then_close_leaves_nothing_open() -> ErrStr<()> {
        let path = std::env::temp_dir().join("tva_test_open_close.log");
        let path_str = path.to_str().unwrap();
        std::fs::write(
            &path,
            "1000,OPEN,1,UNDEAD,500000.00000000,BTC,0.00502601,0.00500000,0xabc\n\
             2000,CLOSE,1,1,511112.13000000,11112.13000000,0.022224,167.780000,0.00300000,0xdef\n",
        ).map_err(|e| format!("could not write test fixture: {e}"))?;

        let (opens, next_pivot, next_close) = replay_log(path_str)?;
        assert!(opens.is_empty(), "pivot 1 was closed, should not appear as open");
        assert_eq!(next_pivot, 2);
        assert_eq!(next_close, 2);

        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn test_replay_log_open_without_close_stays_open() -> ErrStr<()> {
        let path = std::env::temp_dir().join("tva_test_open_only.log");
        let path_str = path.to_str().unwrap();
        std::fs::write(
            &path,
            "1000,OPEN,1,UNDEAD,500000.00000000,BTC,0.00502601,0.00500000,0xabc\n\
             1000,OPEN,2,BTC,0.00500000,UNDEAD,487122.54000000,0.00300000,0xdef\n\
             2000,CHECK,1,not_closed\n",
        ).map_err(|e| format!("could not write test fixture: {e}"))?;

        let (opens, next_pivot, next_close) = replay_log(path_str)?;
        assert_eq!(opens.len(), 2, "neither pivot was closed, both should still be open");
        assert_eq!(next_pivot, 3);
        assert_eq!(next_close, 1, "no CLOSE lines yet, so next_close_id stays at 1");

        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn test_replay_log_rejects_malformed_line() {
        let path = std::env::temp_dir().join("tva_test_malformed.log");
        std::fs::write(&path, "not,even,close,to,valid\n").unwrap();
        let result = replay_log(path.to_str().unwrap());
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
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


    create_testing!("quiz12::tva");

    run!("wallet_balance_btc", " (real BTC.b read against dedicated test wallet, read-only)", {
        let registry = load_token_registry()?;
        let balance = now(wallet_balance(
            "0x6700bD7EAE41434f566e48738813fC585B95669a",
            "BTC",
            &registry,
        ))?;
        println!("\ttest wallet BTC balance: {balance:.8}");
    });

    run!("wallet_balance_undead", " (real UNDEAD read against dedicated test wallet, read-only)", {
        let registry = load_token_registry()?;
        let balance = now(wallet_balance(
            "0x6700bD7EAE41434f566e48738813fC585B95669a",
            "UNDEAD",
            &registry,
        ))?;
        println!("\ttest wallet UNDEAD balance: {balance:.2}");
    });

    run!("live_quote_undead_to_btc", " (real KyberSwap route, read-only, 500000 UNDEAD -> BTC)", {
        let registry = load_token_registry()?;
        let quote = now(live_quote(&registry, "UNDEAD", "BTC", 500_000.0))?;
        println!("\t500000 UNDEAD -> {:.8} BTC right now (router: {})", quote.amount_out, quote.router_address);
    });

    run!("live_quote_btc_to_undead", " (real KyberSwap route, read-only, 0.005 BTC -> UNDEAD)", {
        let registry = load_token_registry()?;
        let quote = now(live_quote(&registry, "BTC", "UNDEAD", 0.005))?;
        println!("\t0.005 BTC -> {:.2} UNDEAD right now (router: {})", quote.amount_out, quote.router_address);
    });

    run!("cycle_dry_run", " (real balances + quotes, never touches keystore)", {
        now(run_cycle(true))?;
        println!("\tdry-run cycle completed without touching the keystore");
    });
}
