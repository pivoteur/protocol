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

use crate::types::{
   quotes::Quotes,
   util::{Token,Blockchain,Id,Pool},
   measurable::{Measurable,weight,size},
   coins::{Coin,mk_coin},
   gains::Gains,
   headers::AggregateHeader
};

// ----- CLOSE PIVOTS -------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Propose {
   header: AggregateHeader,
   close: Id,
   close_date: NaiveDate,
   principal: Vec<Asset>,
   pivot: Vec<PropAsset>,
   propose: PropAsset
}

impl Measurable for Propose {
   fn sz(&self) -> f32 { self.propose.amount }
   fn aug(&self) -> f32 { self.sz()*self.propose.close_price.amount  }
}

impl Gains for Propose {
   fn roi(&self) -> Percentage {
      let base = size(&self.principal);
      mk_percentage((self.propose.amount - base) / base)
   }
   fn apr(&self) -> Percentage {
      if let Ok((wt, _)) = weighted_days(&self) {
         mk_percentage(self.roi().of(365.0 / wt))
      } else {
         panic!("Can't get an APR for proposal")
      }
   }
}

impl Propose {
   pub fn blockchain(p: &Propose) -> Blockchain {
      if let Some(blk) =
            p.pivot.first().and_then(|q| Some(q.blockchain.to_string())) {
         blk
      } else {
         panic!("No blockchain for proposal!")
      }
   }

   pub fn pool(p: &Propose) -> Pool {
      if let Some(pool) = p.principal
          .first()
          .and_then(|q| 
             p.pivot
              .first()
              .and_then(|r| Some((q.token.to_string(), r.token.to_string())))) {
         pool
      } else {
         panic!("Missing principal or pivot (or both) asset from proposal")
      }
   }
      
   pub fn pivot_amount(p: &Propose) -> Coin {
      pivot_amount0(blockchain(p), pool(p), &p.close_date, &p.pivot)
   }

   pub fn weighted_days(p: &Propose) -> ErrStr<(f32, NaiveDate)> {
      if let Some(start_date) = p.header.opened.first().cloned() {
         let days: Vec<f32> =
            p.header.opened.iter()
                           .map(|&d| ((d-start_date).num_days() + 1) as f32)
                           .collect();
         let weights: Vec<f32> =
            days.iter()
                .zip(p.principal.iter().map(Measurable::sz))
                .map(|(&a, b)| a * b)
                .collect();
         let wt: f32 = weights.iter().sum();
         let wt_days = wt / size(&p.principal);
         let ave_dt = start_date + Days::new((wt_days - 1.0) as u64);
         let duration = (p.close_date - ave_dt).num_days() as f32;
         Ok((duration, ave_dt))
      } else {
         Err("No start date for proposal".to_string())
      }
   }
}

impl CsvHeader for Propose {
   fn header(&self) -> String {
      if let Some(prince) = self.principal.first() {
         if let Some(piv) = self.pivot.first() {
            format!("{},close_id,close_date,{},gain_10_percent,{},{},roi,apr",
                    self.header.header(),
                    prince.header(),
                    piv.header(),
                    self.propose.header())
         } else {
            panic!("No pivots for proposal")
         }
      } else {
         panic!("No principal assets for proposal")
      }
   }
}

impl CsvWriter for Propose {
   fn ncols(&self) -> usize {
      let prince = self.principal.first()
            .unwrap_or_else(|| panic!("No principal for proposal"));
      let piv = self.pivot.first()
            .unwrap_or_else(|| panic!("No pivots for proposal"));
      self.header.ncols() + 2 + prince.ncols() + 1
            + piv.ncols() + self.propose.ncols() + 2
   }
   fn as_csv(&self) -> String {
      let (_, opnd) = weighted_days(&self)
            .unwrap_or_else(|_| panic!("No open date for proposal"));
      let prince = self.principal.first()
            .unwrap_or_else(|| panic!("No principal for proposal"));
      let (amount, ersatz) =
               self.principal
                   .iter()
                   .fold((0.0, 0.0), |(a,e), x| {
                      let Amount { actual, ersatz } = &x.amount;
                      (a + actual, e + ersatz) });
      let pri1 = mk_asset(&prince.token, &prince.blockchain,
                          mk_amt(amount, ersatz),
                          mk_usd(weight(&self.principal)), &prince.kind);
      let piv = self.pivot.first()
            .unwrap_or_else(|| panic!("No pivot for proposal"));
      let piv1 = mk_prop_asset(&piv.token, &piv.blockchain,
                               weight(&self.pivot),
                               size(&self.pivot), &piv.kind);
      format!("{},{},{},{},{},{},{},{},{},{}", 
              opnd,
              self.header.as_csv(),
              self.close,
              self.close_date,
              pri1.as_csv(),
              gain_10_percent(&pri1.amount),
              piv1.as_csv(),
              self.propose.as_csv(),
              self.roi(), self.apr())
   }
}

fn mk_prop(open_pivots: Vec<Pivot>, c: Id, d: &NaiveDate,
           pivot: Vec<PropAsset>, propose: PropAsset) -> (Propose, Id) {
   let header = add_header_info(&open_pivots);
   let principal: Vec<Asset> =
      open_pivots.iter().map(|h| h.from.clone()).collect();
   (Propose {
      header,
      close: c,
      close_date: d.clone(),
      principal,
      pivot,
      propose }, c+1)
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

// ----- FUNCTIONAL TEST ------------------------------------------------

pub mod functional_tests {
   use super::*;
   use crate::types::{
      quotes::functional_tests::test_mk_quotes,
      pivots::functional_tests::mk_hbar_usdc_piv
   };

   fn run_propose() -> ErrStr<usize> {
      println!("\ntypes::proposals::propose functional test\n");
      let piv = mk_hbar_usdc_piv(0.2, mk_amt(0.0,500.0), 0, "virtual pivot")?;
      let quotes = test_mk_quotes(&[("HBAR", 0.15)]);
      let proposer = propose(&quotes);
      if let Some((call, next_id)) = proposer((vec![piv], 1))? {
         println!("call:\n{}\n{}\n\nnext_id: {next_id}",
                  call.header(), call.as_csv());
      } else {
         println!("No call for pivots!");
      }
      println!("\ntypes::proposals::propose...ok\n");
      Ok(1)
   }

   pub fn runoff() -> ErrStr<usize> {
      println!("\ntypes::proposals::proposes functional tests\n");
      let b = run_propose()?;
      Ok(b)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::types::{
      quotes::functional_tests::test_mk_quotes,
      pivots::pivots::functional_tests::mk_hbar_usdc_piv
   };
   use book::{
      num::estimate::mk_estimate,
      string_utils::to_string,
      table_utils::cols
   };
   use crate::{
      tables::{IxTable,index_table},
      types::aliases::aliases
   };

   fn assert_price(a: &Asset, est: f32) {
      let q1 = &a.quote;
      let qe1 = mk_estimate(q1.amount);
      let tok = &a.token;
      assert!(qe1.approximates(est * 1e03), "{tok} price ({q1}) isn't ${est}K");
   }

   fn assert_prices(p: &Propose, a: f32, b: f32) {
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
}

