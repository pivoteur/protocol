use chrono::NaiveDate;
use clap::Parser;
use book::{
        cli_utils::add_banner,
        err_utils::ErrStr,
        parse_args_add_banner,
        types::values::Value,
};
use libs::{
        fetchers::calls::fetch_calls,
        types::calls::Call,
};


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
    pub roi:              f32,
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
            roi:             call.roi.value(),
        }
    }
}

impl ArbitrageCandidate {
    pub fn header() -> String {
        [
            "IX", "POOL", "CLOSE DATE", "PIVOT", "PIVOT AMT", "VAL1",
            "PROPOSED", "PROP AMT", "ROI",
        ].join(", ")
    }
}

impl std::fmt::Display for ArbitrageCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {:.6}, ${:.2}, {}, {:.6}, {:.2}%",
            self.ix,
            self.pool,
            self.close_date,
            self.pivot_token,
            self.pivot_amount,
            self.val1,
            self.proposed_token,
            self.proposed_amount,
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
    let candidates = fetch_candidates(root_url).await?;
    println!("{}", ArbitrageCandidate::header());
    for c in &candidates {
        println!("{c}");
    }
    println!("\n{} candidate(s) parsed from calls.csv", candidates.len());
    Ok(())
}

/// Parses calls.csv into arbitrage candidates for the requested columns only.
#[derive(Debug, Parser)]
#[command(name = "arbitrage")]
#[command(version = "0.1.0")]
struct Args {
    /// Root URL of the pivoteur.github.io data (same root used by other dapps)
    root_url: String,
}

pub async fn runoff_with_args() -> ErrStr<()> {
    let args = parse_args_add_banner!(Args);
    process(&args.root_url).await
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
        for label in ["IX", "POOL", "CLOSE DATE", "PIVOT", "VAL1", "PROPOSED", "ROI"] {
            assert!(header.contains(label), "missing '{label}' in header: {header}");
        }
    }

    #[test]
    fn test_ignores_columns_not_requested() -> ErrStr<()> {
        // gain_10_percent, apr, quote1, amount1, virtual, open_pivots, etc.
        // are present in the source CSV but must not appear anywhere on the
        // candidate. This is the "ignore what wasn't listed" contract.
        let shown = format!("{}", first_candidate()?);
        for excluded in ["gain_10_percent", "apr=", "quote1", "virtual"] {
            assert!(!shown.contains(excluded), "unexpected '{excluded}' leaked into: {shown}");
        }
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
}
