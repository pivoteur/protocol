use chrono::NaiveDate;

use std::collections::{ HashMap, HashSet };
use crate::types::{ measurable::Measurable, pivots::opens::Pivot, util::Id };

pub fn extract_dates_amounts(open_pivots: &[Pivot])
      -> HashMap<usize, (NaiveDate, f32)> {
   let ans: HashMap<usize, (NaiveDate, f32)> =
      open_pivots.iter()
                 .map(|p| (p.index(), (p.opened(), p.sz() * 1.1)))
                 .collect();
   ans
}

pub fn filter_pivots(opens: &[Pivot], ids: &[Id]) -> Vec<Pivot> {
   let id_set: HashSet<Id> = ids.into_iter().cloned().collect();
   opens.iter().filter(|piv| id_set.contains(&piv.index())).cloned().collect()
}
