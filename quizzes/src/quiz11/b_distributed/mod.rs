use book::{
    err_utils::ErrStr,
    utils::get_args
};


const DEFAULT_TWEET_URL: &str = "x.com/pivocateur";
const DEFAULT_TX_URL:    &str = "asdf";

fn version()  -> &'static str { "1.00" }
fn app_name() -> &'static str { "distributed" }

fn usage() -> ErrStr<()> {
    eprintln!(
        "Usage: {} <token_a> <token_b> <amount> [tweet_url] [tx_url]",
        app_name()
    );
    eprintln!("  token_a   : distributed token, left side of pool  (e.g. USDC)");
    eprintln!("  token_b   : paired token,      right side of pool (e.g. UNDEAD)");
    eprintln!("  amount    : amount distributed to investor         (e.g. 0.4349)");
    eprintln!("  tweet_url : tweet URL          (default: {DEFAULT_TWEET_URL})");
    eprintln!("  tx_url    : snowtrace tx URL   (default: {DEFAULT_TX_URL})");
    Err("Missing five arguments".to_string())
}

pub fn build_message(
    token_a:   &str,
    token_b:   &str,
    tweet_url: &str,
    amount:    &str,
    tx_url:    &str,
) -> String {
    format!(
        "I close an {token_a}-on-{token_b} pivot (please see the twitter post: {tweet_url}). \
         I sent {amount} {token_a} to you; tx_id: {tx_url}"
    )
}

pub async fn runoff_with_args() -> ErrStr<()> {
    eprintln!("{}, version: {}\n", app_name(), version());
    let args = get_args();
    let (token_a, token_b, tweet_url, amount, tx_url) = match args.as_slice() {
        [token_a, token_b, amount] =>
            (token_a.as_str(), token_b.as_str(), DEFAULT_TWEET_URL, amount.as_str(), DEFAULT_TX_URL),
        [token_a, token_b, amount, tweet_url] =>
            (token_a.as_str(), token_b.as_str(), tweet_url.as_str(), amount.as_str(), DEFAULT_TX_URL),
        [token_a, token_b, amount, tweet_url, tx_url] =>
            (token_a.as_str(), token_b.as_str(), tweet_url.as_str(), amount.as_str(), tx_url.as_str()),
        _ => return usage(),
    };
    let msg       = build_message(token_a, token_b, tweet_url, amount, tx_url);
    println!("{msg}");
    Ok(())
}

//----- UNIT TESTS ------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {
    use super::*;


    #[test]
    fn test_exact_sample_message() {
        let msg = build_message(
            "USDC",
            "UNDEAD",
            "x.com/pivocateur/status/2054570565474635869",
            "0.4349",
            "snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765",
        );
        assert_eq!(
            msg,
            "I close an USDC-on-UNDEAD pivot \
             (please see the twitter post: x.com/pivocateur/status/2054570565474635869). \
             I sent 0.4349 USDC to you; \
             tx_id: snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765"
        );
    }

    #[test]
    fn test_token_positions() {
        let msg = build_message("USDC", "UNDEAD", "tweet", "1.00", "tx");
        assert!(msg.contains("USDC-on-UNDEAD"), "token_a-on-token_b must appear");
        assert!(msg.contains("1.00 USDC"),      "amount token_a must appear");
        assert!(msg.contains("tx_id: tx"),       "tx_url must appear after tx_id:");
    }
 
    #[test]
    fn test_different_token_pair() {
        let msg = build_message("AVAX", "BTC", "tweet", "3.14", "tx");
        assert!(msg.contains("AVAX-on-BTC"));
        assert!(msg.contains("3.14 AVAX"));
    }
 
    #[test]
    fn test_default_urls_appear_correctly() {
        let msg = build_message("USDC", "UNDEAD", DEFAULT_TWEET_URL, "0.5", DEFAULT_TX_URL);
        assert!(msg.contains(DEFAULT_TWEET_URL));
        assert!(msg.contains(DEFAULT_TX_URL));
    }
 
    #[test]
    fn test_usage_returns_err() {
        assert!(usage().is_err());
    }
} 

//----- FUNCTIONAL TESTS -------------------------------------------------------------------
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{
        //utils::now,
        create_testing
    };
 
    create_testing!("quiz11::b_distributed");
 
    run!("build_and_send_message", {
        let msg = build_message(
            "USDC",
            "UNDEAD",
            "x.com/pivocateur/status/2054570565474635869",
            "0.4349",
            "snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765",
        );
        println!("{msg}");
    });
}
