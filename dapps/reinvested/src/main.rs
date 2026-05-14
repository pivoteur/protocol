use book::{
    err_utils::ErrStr,
    utils::get_args,
};


fn version()  -> String { "1.00".to_string() }
fn app_name() -> String { "reinvested".to_string() }

fn usage() -> ErrStr<()> {
    eprintln!("Usage: {} <token_a> <token_b> <pivot_count> <amount> <url>", app_name());
    eprintln!("  token_a      : reinvested token, left side of pool  (e.g. AVAX)");
    eprintln!("  token_b      : paired token,    right side of pool  (e.g. BTC)");
    eprintln!("  pivot_count  : number of pivots closed              (e.g. 2)");
    eprintln!("  amount       : amount reinvested                    (e.g. 0.59)");
    eprintln!("  url          : tweet URL");
    Err(format!("Need <token_a> <token_b> <pivot_count> <amount> <url> arguments"))
}

// ===========================================================================
//----- Configuration and Message Building -----------------------------------
// ===========================================================================
pub fn build_message(
    token_a:  &str,
    token_b:  &str,
    pivot_count: &str,
    amount: &str,
    url:      &str,
) -> String {
    format!(
        "I closed {pivot_count} {token_a}-on-{token_b} pivots and reinvested \
         {amount} ${token_a} into the {token_b}+{token_a} pivot pool for you; \
         tweet: {url}"
    )
}

pub fn runoff_with_args() -> ErrStr<()> {
    let args = get_args();
    match args.as_slice() {
        [token_a, token_b, pivot_count, amount, url] => {
            println!("{}", build_message(token_a, token_b, pivot_count, amount, url));
            Ok(())
        }
        _ => usage(),
    }
}

fn main() -> ErrStr<()> {
    eprintln!("{}, version: {}", app_name(), version());
    runoff_with_args()
}

// ===========================================================================
//----- UNIT TESTS -----------------------------------------------------------
// ===========================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;


    #[test]
    fn test_exact_sample_message() {
        let msg = build_message(
            "AVAX", "BTC", "2", "0.59",
            "https://x.com/pivocateur/status/2047688113024086275",
        );
        assert_eq!(
            msg,
            "I closed 2 AVAX-on-BTC pivots and reinvested 0.59 $AVAX \
             into the BTC+AVAX pivot pool for you; \
             tweet: https://x.com/pivocateur/status/2047688113024086275"
        );
    }

    #[test]
    fn test_token_positions() {
        let msg = build_message("ETH", "BTC", "1", "1.5", "https://x.com/test");
        assert!(msg.contains("ETH-on-BTC"), "should show token_a-on-token_b");
        assert!(msg.contains("BTC+ETH"),    "should show token_b+token_a in pool name");
        assert!(msg.contains("$ETH"),       "should show $token_a as the reinvested token");
    }

    #[test]
    fn test_different_token_pair() {
        let msg = build_message("SOL", "AVAX", "3", "12.5", "https://x.com/test");
        assert!(msg.contains("3 SOL-on-AVAX pivots"));
        assert!(msg.contains("AVAX+SOL pivot pool"));
        assert!(msg.contains("12.5 $SOL"));
    }

    #[test]
    fn test_usage_returns_err() {
        assert!(usage().is_err());
    }
}

// ===========================================================================
//----- FUNCTIONAL TESTS -----------------------------------------------------
// ===========================================================================
// #[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    // use paste::paste;
    // ues book::{
    //     create_testing
    // };


    // create_testing!("quiz11::a_reinvest")

    pub fn runoff() -> ErrStr<()> {
        let msg = build_message(
            "AVAX", "BTC", "2", "0.59",
            "https://x.com/pivocateur/status/2047688113024086275",
        );
        println!("{msg}");
        Ok(())
    }
}

