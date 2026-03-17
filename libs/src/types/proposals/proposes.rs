use chrono::{Days,NaiveDate,TimeDelta};

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   err_utils::ErrStr,
   num::percentage::{Percentage,mk_percentage}
};

use super::{
   aggregate_headers::{AggregateHeader,add_header_info},
   prop_assets::{PropAsset,pivot_amount0}
};

use crate::types::{
   assets::{
      assets::{Asset,coalesce,gain_10_percent},
      amounts::mk_amt,
      asset_types::AssetType::*
   },
   coins::Coin,
   gains::Gains,
   measurable::{Measurable,weight,size},
   pivots::{Pivot,headers,froms},
   quotes::Quotes,
   util::{Blockchain,Id,Pool}
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
   fn sz(&self) -> f32 { size(&self.principal) }
   fn aug(&self) -> f32 { weight(&self.principal) }
}

impl Gains for Propose {
   fn roi(&self) -> Percentage {
      let base = size(&self.principal);
      mk_percentage((self.propose.sz() - base) / base)
   }
   fn apr(&self) -> Percentage {
      if let Ok((wt, _)) = self.weighted_days() {
         mk_percentage(self.roi().of(365.0 / wt))
      } else {
         panic!("Can't get an APR for proposal")
      }
   }
}

impl Propose {
   pub fn blockchain(&self) -> Blockchain {
      if let Some(blk) =
            self.pivot.first().and_then(|q| Some(q.blockchain())) {
         blk
      } else {
         panic!("No blockchain for proposal!")
      }
   }

   pub fn pool(&self) -> Pool {
      if let Some(pool) = self.principal
          .first()
          .and_then(|q| 
             self.pivot
              .first()
              .and_then(|r| Some((q.token(), r.token())))) {
         pool
      } else {
         panic!("Missing principal or pivot (or both) asset from proposal")
      }
   }
   pub fn pivot_amount(&self) -> Coin {
      pivot_amount0(self.blockchain(), self.pool(),
                    &self.close_date, &self.pivot)
   }

   pub fn weighted_days(&self) -> ErrStr<(f32, NaiveDate)> {
      let (start_date,days) = self.header.durations()?;
      let weights: Vec<f32> =
         days.iter()
             .zip(self.principal.iter().map(Measurable::sz))
             .map(|(&a, b)| a * b)
             .collect();
      let wt: f32 = weights.iter().sum();
      let wt_days = wt / size(&self.principal);
      let ave_dt = start_date + Days::new((wt_days - 1.0) as u64);
      let dur: TimeDelta = self.close_date - ave_dt;
      let duration = dur.num_days() as f32;
      Ok((duration, ave_dt))
   }
}

impl CsvHeader for Propose {
   fn header(&self) -> String {
      let prince = self.principal.first()
            .unwrap_or_else(|| panic!("No principal assets for proposal"));
      let piv = self.pivot.first()
            .unwrap_or_else(|| panic!("No pivots for proposal"));
      format!("{},close_id,close_date,{},gain_10_percent,{},{},roi,apr",
              self.header.header(), prince.header(),
              piv.header(), self.propose.header())
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
      let (_, opnd) = self.weighted_days()
            .unwrap_or_else(|_| panic!("No open date for proposal"));
      let prince = coalesce(&self.principal)
            .unwrap_or_else(|_| panic!("Cannot coalesce assets!"));
      let pivs = &self.pivot;
      let piv = pivs.first().unwrap_or_else(|| panic!("No pivot for proposal"));
      let piv1 = piv.clone_with(weight(&pivs), size(&pivs), TO);
      format!("{},{},{},{},{},{},{},{},{},{}", 
              opnd,
              self.header.as_csv(),
              self.close,
              self.close_date,
              prince.as_csv(),
              gain_10_percent(prince.sz()),
              piv1.as_csv(),
              self.propose.as_csv(),
              self.roi(), self.apr())
   }
}

fn mk_prop(open_pivots: Vec<Pivot>, c: Id, d: &NaiveDate,
           pivot: Vec<PropAsset>, propose: PropAsset) -> (Propose, Id) {
   let header = add_header_info(&headers(&open_pivots));
   let principal = froms(&open_pivots);
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
      let mut res: Vec<PropAsset> = Vec::new();
      for p in pivots {
         let props = p.trade(q)?;
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
            let proposed = result.clone_with(result.aug(), size(&res), TO);
            Ok(Some(mk_prop(princes, c, &q.date, pivs, proposed)))
         } else {
            Err("No proposal to accumulate on flagged principal".to_string())
         }
      }
   }
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
      pivots::functional_tests::mk_hbar_usdc_piv
   };

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

