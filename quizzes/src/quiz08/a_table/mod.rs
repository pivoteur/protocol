use book::{
   csv_utils::CsvWriter,
   err_utils::ErrStr,
   parse_utils::{parse_id,parse_str},
   string_utils::s,
   table_utils::{ingest,val},
};

use libs::{ tables::IxTable, types::util::Id };

fn parse_and_print_call_datum(lines: Vec<String>, row: Id, col: &str)
      -> ErrStr<()> {
   let (table, datum) = parse_datum(lines, row, col)?;
   report(table, row, col, datum);
   Ok(())
}

fn parse_datum(lines: Vec<String>, row: Id, col: &str)
      -> ErrStr<(IxTable, String)> {
   let table = ingest(parse_id, parse_str, parse_str, &lines, ",")?;
   let datum = val(&table, &row, &s(col))
                  .ok_or(format!("Can't get datum at {row}x{col}"))?;
   Ok((table, datum))
}

fn report(table: IxTable, r: Id, col: &str, datum: String) {
   println!("Table is:\n\n{}", table.as_csv());
   println!("\nThe value at row {r} / col {col} is {datum}");
}

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;

   use book::{
      stream_utils::lines_from_stdin,
      test_utils::preamble,
      utils::get_args
   };

   pub fn runoff_with_args() -> ErrStr<()> {
      if let [row, col] = get_args().as_slice() {
         let lines = lines_from_stdin()?;
         let r = parse_id(&row)?;
         parse_and_print_call_datum(lines, r, col)
      } else {
         Err("Select a <row> and <col> view; table from stream".to_string())
      }
   }

   fn calls() -> Vec<String> {
"ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,ETH+UNDEAD,17,2026-04-01,2026-03-28,32;34;38;40,23,2026-04-27,ETH,Avalanche,0,11.823222,$2257.41,$26689.86,13.005545,UNDEAD,Avalanche,$0.001461,21140818,ETH,Avalanche,$2319.89,13.310052,12.58%,153.00%
2,AVAX+UNDEAD,19,2026-04-13,2026-02-20,32;34;42,26,2026-04-27,AVAX,Avalanche,0,186.43323,$9.40,$1752.47,205.07655,UNDEAD,Avalanche,$0.001461,1388041.9,AVAX,Avalanche,$9.26,218.93588,17.43%,96.41%
3,UNDEAD+USDC,11,2026-04-01,2026-03-21,29;31;33;35;37;39;40,17,2026-04-27,USDC,Avalanche,0,28688.19,$1.00,$28688.19,31557.01,UNDEAD,Avalanche,$0.001461,23061802,USDC,Avalanche,$1.00,33688.055,17.43%,171.93%
4,AVAX+UNDEAD,19,2026-04-13,2026-04-02,41,27,2026-04-27,UNDEAD,Avalanche,0,1012051.4,$0.001781,$1802.46,1113256.5,AVAX,Avalanche,$9.26,194.60458,UNDEAD,Avalanche,$0.001461,1233782.8,21.91%,319.87%
5,ETH+UNDEAD,17,2026-04-01,2026-04-01,37;39,24,2026-04-27,UNDEAD,Avalanche,492826,2015274.3,$0.001744,$4373.29,2758910.3,ETH,Avalanche,$2319.89,2.033876,UNDEAD,Avalanche,$0.001461,3230475.5,28.80%,404.33%
6,BTC+UNDEAD,20,2026-04-09,2026-03-28,32;34;40,15,2026-04-27,UNDEAD,Avalanche,833400,540280.56,$0.000843,$1158.61,1511048.6,BTC,Avalanche,$77821.00,0.03893714,UNDEAD,Avalanche,$0.001461,2074605.4,51.03%,620.81%".split("\n").map(s).collect()
   }

   pub fn runoff() -> ErrStr<usize> {
      preamble("quiz08::a_table");
      let lines = calls();
      let _ = parse_and_print_call_datum(lines, 4, "pool")?;
      Ok(1)
   }

   #[cfg(test)]
   mod tests {
      use super::*;
      use book::list_utils::tail;

      #[test] fn fail_parse_datum() {
         let truncated = tail(&calls());
         let ans = parse_datum(truncated, 3, "quote1");
         assert!(ans.is_err());
      }
      #[test] fn test_parse_datum_ok() {
         assert!(parse_datum(calls(), 2, "ids").is_ok());
      }
      #[test] fn test_parse_5_from() -> ErrStr<()> {
         let (_table, datum) = parse_datum(calls(), 5, "from")?;
         assert_eq!("UNDEAD", &datum);
         Ok(())
      }
   }
}

