use book::err_utils::{ErrStr,err_or};

use libs::types::calls::{Call,parse_calls};

pub parse_crypto_csv(csv: &str) -> ErrStr<Vec<Call>> {
   parse_calls(csv)
}

// -----------------------------------------------------------------------------
// 🚀 RUNTIME VERIFICATION
// -----------------------------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;

   use paste::paste;
   use book::{create_testing,err_utils::ErrStr};

   create_testing!("quizzes::quiz08::e_offrian");

   run!("parse_calls", {
      let raw_csv_data = "\
ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,BTC+USDC,10,2026-04-16,2026-04-15,27;29,8,2026-06-10,BTC,Avalanche,0,0.452206,$81812.00,$36995.88,0.4974266,USDC,Avalanche,$1.00,37005.758,BTC,Avalanche,$61419.00,0.6023795,33.21%,216.45%
2,BTC+UNDEAD,20,2026-04-09,2026-02-07,3;5;8;10;28;32;34;36;40,15,2026-06-10,UNDEAD,Avalanche,2189400,540280.56,$0.001782,$4863.69,3002648.5,BTC,Avalanche,$61419.00,0.0646658,UNDEAD,Avalanche,$0.000960,4135559.8,51.50%,152.84%";

      let parsed_records = parse_crypto_csv(raw_csv_data)?;

      for record in parsed_records {
         println!(
            "Pool: {:<11} | ROI: {:>5}% | IDs Vector: {:?}",
            record.pool, record.roi, record.ids
         );
      }
   });
}

