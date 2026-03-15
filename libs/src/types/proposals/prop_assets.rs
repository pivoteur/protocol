use std::{
   collections::HashMap,
   fmt::Display
};

use chrono::{Days,NaiveDate};

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::{USD,mk_usd},
   date_utils::parse_date,
   err_utils::ErrStr,
   num::percentage::{Percentage,mk_percentage},
   num_utils::parse_commaless,
   parse_utils::parse_id,
   tuple_utils::Partition,
   utils::pred
};

use super::{
   quotes::Quotes,
   util::{Token,Blockchain,Id,Pool},
   measurable::{Measurable,weight,size},
   assets::{Asset as Coin,mk_asset as mk_coin}
};

use crate::types::gains::Gains;

#[derive(Debug, Clone)]
struct PropAsset {
   token: Token,
   blockchain: Blockchain,
   close_price: USD,
   amount: f32,
   kind: AssetType
}

impl CsvHeader for PropAsset {
   fn header(&self) -> String {
      let preface = match self.kind {
         AssetType::FROM => "pivot",
         AssetType::TO   => "proposed"
      };
      ["token","blockchain","close_price","amount"]
         .iter()
         .map(|elt| format!("{}_{}", preface, elt))
         .collect::<Vec<_>>()
         .join(",")
   }
}
impl CsvWriter for PropAsset {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String {
      format!("{},{},{},{}",
              self.token, self.blockchain, self.close_price, self.amount)
   }
}

impl Measurable for PropAsset {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.sz() * self.close_price.amount }
}

fn mk_prop_asset(tkn: &str, blk: &str, c: f32, amount: f32, knd: &AssetType)
      -> PropAsset {
   PropAsset { token: tkn.to_string(), blockchain: blk.to_string(),
               close_price: mk_usd(c), amount, kind: knd.clone() }
}

fn pivot_amount0(blockchain: Blockchain, pool: Pool,
                 date: &NaiveDate, assets: &Vec<PropAsset>) -> Coin {
   let (_, piv) = pool.clone();
   mk_coin(&(blockchain, piv), size(&assets), &mk_usd(weight(&assets)), &date)
}

pub fn propose(q: &Quotes)
      -> impl Fn((Vec<Pivot>, Id)) -> ErrStr<Option<(Propose, Id)>> {
   move |(pivots, c): (Vec<Pivot>, Id)| {
      let mut princes = Vec::new();
      let mut pivs = Vec::new();
      let mut res = Vec::new();
      for p in pivots {
         let props = trade(q, &p)?;
         let _ = props.and_then(|(frm, to)| {
            princes.push(p);
            pivs.push(frm);
            res.push(to);
            Some(1)
         });
      }
      if princes.is_empty() {
         Ok(None)
      } else {
         if let Some(result) = res.first() {
            let proposed = mk_prop_asset(&result.token, &result.blockchain,
                                         result.close_price.amount,
                                         size(&res), &AssetType::TO);
            Ok(Some(mk_prop(princes, c, &q.date, pivs, proposed)))
         } else {
            Err("No proposal to accumulate on flagged principal".to_string())
         }
      }
   }
}

fn trade(q: &Quotes, p: &Pivot) -> ErrStr<Option<(PropAsset, PropAsset)>> {
   // with the quotes for the assets, ...
   let prim = &p.from.token;
   let prim_blk = &p.from.blockchain;
   let piv = &p.to.token; 
   let piv_blk = &p.to.blockchain;
   let prim_qt = q.lookup(prim)?;
   let piv_qt = q.lookup(piv)?;
   let to_trade = amount(&p.to.amount);
   let target = gain_10_percent(&p.from.amount);
   let computed_amount = to_trade * piv_qt / prim_qt;
   Ok(pred(computed_amount > target,
           (mk_prop_asset(piv, piv_blk, piv_qt, to_trade, &AssetType::FROM),
            mk_prop_asset(prim, prim_blk, prim_qt,
                          computed_amount, &AssetType::TO))))
}

// ----- GROUPING  -------------------------------------------------------

/// Partitions open-pivots by principal asset
pub fn partition_on(tok: &str, opens: Vec<Pivot>) -> Partition<Pivot> {
   opens.into_iter().partition(|p: &Pivot| p.from.token == tok)
}

// ----- FUNCTIONAL TEST ------------------------------------------------

pub mod functional_tests {
   use super::*;
   use crate::types::quotes::functional_tests::test_mk_quotes;

   pub fn mk_hbar_usdc_piv(q: f32, a: Amount, c: usize, tx: &str)
         -> ErrStr<Pivot> {
      let qt = mk_usd(q);
      let to = mk_asset("USDC", "Hedera", mk_amt(100.0, 0.0), mk_usd(1.0), &TO);
      let header = mk_hdr("2026-03-10",1,c, tx.to_string(), None)?;
      Ok(Pivot { header, from: mk_asset("HBAR", "Hedera", a, qt, &FROM), to })
   }

   fn run_recompute_pivot() -> ErrStr<usize> {
      println!("\ntypes::pivot::recompute_pivot functional test\n");
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0,500.0), 0, "virtual pivot")?;
      let quotes = test_mk_quotes(&[("HBAR", 0.25)]);
      let _new_piv = recompute_pivot(&quotes, true)(piv)?;
      println!("\ntypes::pivot::recompute_pivot...ok\n");
      Ok(1)
   }

   fn run_propose() -> ErrStr<usize> {
      println!("\ntypes::pivot::propose functional test\n");
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0,500.0), 0, "virtual pivot")?;
      let quotes = test_mk_quotes(&[("HBAR", 0.15)]);
      let proposer = propose(&quotes);
      if let Some((call, next_id)) = proposer((vec![piv], 1))? {
         println!("call:\n{}\n{}\n\nnext_id: {next_id}",
                  call.header(), call.as_csv());
      } else {
         println!("No call for pivots!");
      }
      println!("\ntypes::pivot::propose...ok\n");
      Ok(1)
   }

   pub fn runoff() -> ErrStr<usize> {
      println!("\ntypes::pivots functional tests\n");
      let a = run_recompute_pivot()?;
      let b = run_propose()?;
      Ok(a+b)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use super::functional_tests::mk_hbar_usdc_piv;
   use book::{
      num::estimate::mk_estimate,
      string_utils::to_string,
      table_utils::cols
   };
   use crate::{
      tables::{IxTable,index_table},
      types::{
         aliases::aliases,
         quotes::functional_tests::test_mk_quotes
      }
   };

   // this test data contains 
   // a closed pivot
   // an opened pivot
   // a virtual pivot
   // and a non-virtual virtual pivot (protocol_issue_010_non_virtual_pivots)
   fn btc_eth_raw() -> String {
"opened	open	close	tx_id	updated	from	from_blockchain	amount1	virtual	quote1	val1	gain_10_percent	to	to_blockchain	net	ersatz	quote2	val2
2025-08-06	1	1	https://snowtrace.io/tx/0x60a2129cf19213def46b4355739cf69e998ed6245fe0ade6563e83c1ba2d83c8	n/a	BTC	Avalanche	0.004498	0	$113883.00	$512.25	0.0049477997	ETH	Avalanche	0.14203	0	$3588.72	$509.71
2025-09-30	28	0	https://snowtrace.io/tx/0xdef66cbfea4687eff8390728557a07b9697dc15927de51d0819e07aa5bc71963	n/a	BTC	Avalanche	0.0087	0	$113056.00	$983.59	0.009570001	ETH	Avalanche	0.2305	0	$4162.11	$959.37
2026-02-21	17	0	virtual swap	n/a	BTC	Avalanche	0	0.009205	$114701.00	$1055.82	0.0101255	ETH	Avalanche	0.3177	0	$4810.58	$1528.32
2026-02-21	52	0	https://snowtrace.io/tx/0x267ed024578621a51aabc47b9b0d7f4791c6624863130ad0dcab4c1328fd8a90	n/a	ETH	Avalanche	5.046	0	$1987.48	$10028.82	5.5506	BTC	Avalanche	0.14587	0	$68429.00	$9981.74
2026-02-21	41	0	https://snowtrace.io/tx/0x77fe7489ccb408e103e86f12bdfa1fbf0dc4476912a7a0bff6ad4b12b32e55c1	n/a	BTC	Avalanche	0	0.0074	$107858.00	$798.15	0.0081400005	ETH	Avalanche	0.2559	0	$3715.49	$950.79".to_string()
   }

   fn btc_eth() -> ErrStr<(IxTable, HashMap<String, usize>)> {
      let lines: Vec<String> =
         btc_eth_raw().split("\n").map(to_string).collect();
      let table = index_table(lines)?;
      let ix = aliases().enum_headers(cols(&table));
      Ok((table, ix))
   }

   fn btc_eth_pivots() -> ErrStr<Vec<Pivot>> {
      let (tabl, ix) = btc_eth()?;
      Ok(tabl.data.into_iter()
               .filter_map(|row| parse_pivot(&ix, &row).ok())
               .collect())
   }

   #[test]
   fn test_partition_on_btc() -> ErrStr<()> {
      let pivs = btc_eth_pivots()?;
      let (btcs, eths) = partition_on("BTC", pivs);
      assert_eq!(4, btcs.len());
      assert_eq!(1, eths.len());
      Ok(())
   }

   fn assert_price(a: &Asset, est: f32) {
      let q1 = &a.quote;
      let qe1 = mk_estimate(q1.amount);
      let tok = &a.token;
      assert!(qe1.approximates(est * 1e03), "{tok} price ({q1}) isn't ${est}K");
   }

   fn assert_prices(p: &Pivot, a: f32, b: f32) {
      assert_price(&p.from, a);
      assert_price(&p.to, b);
   }

   #[test]
   fn test_asset_quotes() -> ErrStr<()> {
      let pivs = btc_eth_pivots()?;
      assert_prices(&pivs[0], 113.9, 3.6);
      assert_prices(&pivs[1], 113.1, 4.1);
      assert_prices(&pivs[2], 114.7, 4.8);
      assert_prices(&pivs[3], 2.0, 68.4);
      assert_prices(&pivs[4], 107.9, 3.7);
      Ok(())
   }

   #[test]
   fn test_parse_pivot_ok() -> ErrStr<()> {
      let (table, ix) = btc_eth()?;
      let row = table.data.first().unwrap();
      let ans = parse_pivot(&ix, &row);
      assert!(ans.is_ok());
      Ok(())
   }

   #[test]
   fn test_parse_virtual_pivot() -> ErrStr<()> {
      let (table, ix) = btc_eth()?;
      let mut virt = false;
      for row in table.data {
         let piv = parse_pivot(&ix, &row)?;
         virt = virt || piv.is_virtual();
      }
      assert!(virt);
      Ok(())
   }

   #[test]
   fn test_no_url() -> ErrStr<()> {
      let (table, ix) = btc_eth()?;
      let row = table.data.first().unwrap();
      let piv = parse_pivot(&ix, &row)?;
      assert!(!no_url(&piv.header));
      Ok(())
   }

   #[test]
   fn test_parse_pivots() -> ErrStr<()> {
      let (table, ix) = btc_eth()?;
      let mut virts = 0;
      for row in &table.data {
         let piv = parse_pivot(&ix, &row)?;
         virts += piv.is_virtual() as i32;
      }
      assert_eq!(1, virts);
      assert_eq!(5, table.data.len());
      Ok(())
   }

   #[test]
   fn fail_recompute_non_virtual_amt_pivot() -> ErrStr<()> {
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(500.0, 0.0), 0, "https://yo")?;
      let reckt =
         recompute_pivot(&test_mk_quotes(&[("HBAR", 0.22)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("virtual"));
         Ok(())
      } else { 
         Err(format!("reckt ({reckt:?}) succeeds (???) unfortunately."))
      }
   }

   #[test]
   fn fail_recompute_non_virtual_tx_pivot() -> ErrStr<()> {
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0, 500.0), 0, "https://yo")?;
      let reckt =
         recompute_pivot(&test_mk_quotes(&[("HBAR",0.22)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("virtual"));
         Ok(())
      } else { 
         Err(format!("reckt ({reckt:?}) succeeds (???) unfortunately."))
      }
   }

   #[test]
   fn fail_recompute_closed_pivot() -> ErrStr<()> {
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0, 500.0), 1, "virtual pivot")?;
      let reckt =
         recompute_pivot(&test_mk_quotes(&[("HBAR",0.22)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("close"));
         Ok(())
      } else { 
         let cls = "closed pivot recompute";
         Err(format!("{cls} {reckt:?} succeeds (???) unfortunately."))
      }
   }

   #[test]
   fn test_no_recompute_virtual_pivot_ok() -> ErrStr<()> {
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0, 500.0), 0, "virtual_pivot")?;
      assert!(!piv.is_updated());
      let neiner =
         recompute_pivot(&test_mk_quotes(&[("HBAR",0.15)]), false)(piv);
      assert!(neiner.is_ok());
      assert!(!neiner.unwrap().is_updated());
      Ok(())
   }

   #[test]
   fn test_recompute_virtual_pivot_ok() -> ErrStr<()> {
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0, 500.0), 0, "virtual_pivot")?;
      assert!(!piv.is_updated());
      let neiner =
         recompute_pivot(&test_mk_quotes(&[("HBAR",0.25)]), false)(piv);
      assert!(neiner.is_ok());
      assert!(neiner.unwrap().is_updated());
      Ok(())
   }

   #[test]
   fn test_propose_ok_no_call() -> ErrStr<()> {
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0, 500.0), 0, "virtual_pivot")?;
      let quotes = test_mk_quotes(&[("HBAR",0.25)]);
      let proposer = propose(&quotes);
      let max_id = 1;
      let ans = proposer((vec![piv], max_id));
      assert!(ans.is_ok());
      let call = ans?;
      assert!(call.is_none());
      Ok(())
   }

   #[test]
   fn test_propose_ok_with_call() -> ErrStr<()> {
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0, 500.0), 0, "virtual_pivot")?;
      let quotes = test_mk_quotes(&[("HBAR",0.15)]);
      let max_id = 1;
      let proposer = propose(&quotes);
      let ans = proposer((vec![piv], max_id));
      assert!(ans.is_ok());
      if let Some((_call, next_id)) = ans? {
         assert_eq!(2, next_id);
         Ok(())
      } else {
         Err(format!("Should have been a call"))
      }
   }

   #[test]
   fn test_next_close_id() -> ErrStr<()> {
      let pivs = btc_eth_pivots()?;
      assert_eq!(2, next_close_id(&pivs));
      Ok(())
   }
}

