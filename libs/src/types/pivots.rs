use std::collections::HashMap;

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::mk_usd,
   err_utils::ErrStr,
   list_utils::filter_map_or,
   tuple_utils::Partition
};

use super::{
   assets::{
      assets::{
         Asset,mk_asset,parse_asset,recompute_assets,gain_10_percent,trade
      },
      asset_types::AssetType::{FROM,TO},
      amounts::{Amount,mk_amt}
   },
   headers::{Header, next_close_id as closer, parse_header},
   coins::Coin,
   measurable::Measurable,
   proposals::prop_assets::PropAsset,
   quotes::Quotes,
   util::{Blockchain,Id}
};

// ----- PIVOT types -------------------------------------------------------

/// Defines the structure of an open piv,is_virt},
#[derive(Debug, Clone)]
pub struct Pivot {
   header: Header,
   from: Asset,
   to: Asset
}

impl Measurable for Pivot {
   fn sz(&self) -> f32 { self.from.sz() }
   fn aug(&self) -> f32 { self.from.aug() }
}

impl Pivot {
   pub fn is_virtual(&self) -> bool {
      self.header.no_url() && self.from.is_virt()
   }
   pub fn committed(&self) -> ErrStr<Coin> {
      self.to.committed(&self.header.opened())
   }
   pub fn blockchain(&self) -> Blockchain { self.to.blockchain() }
   pub fn closed(&self) -> bool { self.header.closed() }
   pub fn active(&self) -> bool { !self.closed() }
   pub fn is_updated(&self) -> bool { self.header.is_updated() }
   pub fn index(&self) -> usize { self.header.ix() }
   pub fn trade(&self, qts: &Quotes) -> ErrStr<Option<(PropAsset, PropAsset)>> {
      trade(qts, &self.from, &self.to)
   }
}

impl CsvWriter for Pivot {
   fn ncols(&self) -> usize { 
      self.header.ncols() + self.from.ncols() + self.to.ncols() + 1
   }
   fn as_csv(&self) -> String {
      let gain = gain_10_percent(self.from.sz());
      format!("{},{},{},{}", 
              self.header.as_csv(),
              self.from.as_csv(), gain,
              self.to.as_csv())
   }
}
impl CsvHeader for Pivot {
   fn header(&self) -> String {
      format!("{},{},gain_10_percent,{}",
              self.header.header(), self.from.header(), self.to.header())
   }
}

pub fn parse_pivot(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Pivot> {
   let header = parse_header(hdrs, row)?;
   let from = parse_asset(FROM, hdrs, row)?;
   let to = parse_asset(TO, hdrs, row)?;
   Ok( Pivot { header, from, to } )
}

// ----- COLLECTIONS OPERATIONS ------------------------------------------

pub fn next_close_id(pivs: &Vec<Pivot>) -> Id {
   closer(&headers(pivs))
}

pub fn headers(pivs: &Vec<Pivot>) -> Vec<Header> {
   pivs.into_iter().map(|p| p.header.clone()).collect()
}

pub fn froms(pivs: &Vec<Pivot>) -> Vec<Asset> {
   pivs.into_iter().map(|p| p.from.clone()).collect()
}

// ----- RECOMPUTING VIRTUAL PIVOTS --------------------------------------

pub fn recompute_pivot(quotes: &Quotes, debug: bool)
      -> impl Fn(Pivot) -> ErrStr<Pivot> {
   move |p| {
      if !p.is_virtual() { Err("Can only recompute virtual pivots".to_string())
      } else if p.closed() { Err("Pivot closed; cannot recompute".to_string())
      } else { recompute1(quotes, p, debug)
      }
   }
}

fn recompute1(quotes: &Quotes, p: Pivot, debug: bool) -> ErrStr<Pivot> {
   if debug { println!("For pivot:\n{}\n{}", p.header(), p.as_csv()); }
   let mb_new_assets = recompute_assets(quotes, &p.from, &p.to)?;
   Ok(match mb_new_assets {
      Some((from, to)) => {
         let today = quotes.date.clone();
         let header = p.header.update_to(today);
         let new_piv1 = Pivot { header, from, to };
         if debug { println!("\tRecomputed to:\n{}", new_piv1.as_csv()); }
         new_piv1
      },
      None => {
         if debug { println!("\tNo change"); }
         p
      }
   })
}

// ----- GROUPING  -------------------------------------------------------

/// Partitions open-pivots by principal asset
pub fn partition_on(tok: &str, opens: Vec<Pivot>) -> Partition<Pivot> {
   opens.into_iter().partition(|p: &Pivot| &p.from.token() == tok)
}

// ----- FUNCTIONAL TEST ------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use book::{ string_utils::to_string, table_utils::cols };
   use crate::{
      tables::{IxTable,index_table},
      types::{
         aliases::aliases,
         quotes::functional_tests::test_mk_quotes,
         headers::mk_hdr
      }
   };

   pub fn mk_hbar_usdc_piv(q: f32, a: Amount, c: usize, tx: &str)
         -> ErrStr<Pivot> {
      let qt = mk_usd(q);
      let to = mk_asset("USDC", "Hedera", mk_amt(100.0, 0.0), mk_usd(1.0), &TO);
      let header = mk_hdr("2026-03-10",1,c, tx.to_string(), None)?;
      Ok(Pivot { header, from: mk_asset("HBAR", "Hedera", a, qt, &FROM), to })
   }

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

   pub fn btc_eth() -> ErrStr<(IxTable, HashMap<String, usize>)> {
      let lines: Vec<String> =
         btc_eth_raw().split("\n").map(to_string).collect();
      let table = index_table(lines)?;
      let ix = aliases().enum_headers(cols(&table));
      Ok((table, ix))
   }

   pub fn btc_eth_pivots() -> ErrStr<Vec<Pivot>> {
      let (tabl, ix) = btc_eth()?;
      filter_map_or(|row| parse_pivot(&ix, &row), tabl.data)
   }

   fn run_recompute_pivot() -> ErrStr<usize> {
      println!("\ntypes::pivot::recompute_pivot functional test\n");
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0,500.0), 0, "virtual pivot")?;
      let quotes = test_mk_quotes(&[("HBAR", 0.25)]);
      let _new_piv = recompute_pivot(&quotes, true)(piv)?;
      println!("\ntypes::pivot::recompute_pivot...ok\n");
      Ok(1)
   }

   pub fn runoff() -> ErrStr<usize> {
      println!("\ntypes::pivots functional tests\n");
      let a = run_recompute_pivot()?;
      Ok(a)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use super::functional_tests::{btc_eth,btc_eth_pivots,mk_hbar_usdc_piv};
   use crate::{
      types::{
         assets::assets::functional_tests::assert_price_k,
         quotes::functional_tests::test_mk_quotes
      }
   };

   #[test]
   fn test_partition_on_btc() -> ErrStr<()> {
      let pivs = btc_eth_pivots()?;
      let (btcs, eths) = partition_on("BTC", pivs);
      assert_eq!(4, btcs.len());
      assert_eq!(1, eths.len());
      Ok(())
   }

   fn assert_prices_k(p: &Pivot, a: f32, b: f32) {
      assert_price_k(&p.from, a);
      assert_price_k(&p.to, b);
   }

   #[test]
   fn test_asset_quotes() -> ErrStr<()> {
      let pivs = btc_eth_pivots()?;
      assert_prices_k(&pivs[0], 113.9, 3.6);
      assert_prices_k(&pivs[1], 113.1, 4.1);
      assert_prices_k(&pivs[2], 114.7, 4.8);
      assert_prices_k(&pivs[3], 2.0, 68.4);
      assert_prices_k(&pivs[4], 107.9, 3.7);
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
      assert!(!piv.header.no_url());
      assert!(piv.closed());
      assert!(!piv.is_virtual());
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
   fn test_next_close_id() -> ErrStr<()> {
      let pivs = btc_eth_pivots()?;
      assert_eq!(2, next_close_id(&pivs));
      Ok(())
   }
}

