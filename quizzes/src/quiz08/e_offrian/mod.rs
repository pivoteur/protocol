use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   csv_utils::as_csv,
   currency::usd::mk_usd,
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   fetchers::calls::fetch_call_data,
   processors::virtuals::compute_counter_offer,
   types::calls::Call
};

/// Makes a counter-offer to a proposed close pivot
#[derive(Debug, Parser)]
#[command(name = "offrian")]
#[command(version = "1.04")]
struct Args {
   /// protocol to make the counter-offer, e.g.: PIVOT
   protocol: UppercaseString,

   /// call id being countered, e.g. 1
   ix: usize,

   /// target volume (the volume we want the open pivots to have been)
   volume: f32,

   /// show debug information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   runoff_continuation(&args.protocol, args.ix, args.volume, args.debug).await
}

async fn runoff_continuation(auth: &str, ix: usize, vol: f32, debug: bool)
      -> ErrStr<()> {
   let root_url = get_env(&format!("{auth}_URL"))?;
   let volume = mk_usd(vol);
   let call_data = fetch_call_data(&root_url, ix, debug).await?;
   let offrian =
      compute_counter_offer(&call_data, &volume, debug)?;
   report_counter_offer(&offrian, debug)
}

fn report_counter_offer(nc: &Call, debug: bool) -> ErrStr<()> {
   if debug {
      let prop = &nc.proposed_token;
      let a = format!("swap {} {} (quote: {})", nc.pivot_amount,
                      nc.pivot_token, nc.pivot_close_price);
      let b = format!("for at least {} {} (quote: {})", nc.gain_10_percent,
                      prop, nc.proposed_close_price);
      let c  = format!("(expecting swap to {} {})", nc.proposed_amount,
                       prop);
      println!("\nOffrian:\n{a}\n{b}\n{c};\nROI {} / {} APR\n", nc.roi, nc.apr);
   }
   let output = as_csv(&[nc])?;
   println!("{output}");
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;

   use paste::paste;
   use book::{ create_testing, utils::now };

   create_testing!("quizzes::quiz08::e_offrian");

   run!("debug_offrian",
        now(runoff_continuation("PIVOT", 1, 9.0, true)));

   run!("offrian", now(runoff_continuation("PIVOT", 1, 9.0, false)));
}
