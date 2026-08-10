use std::{ fmt, str::FromStr };
use book::err_utils::ErrStr;

#[derive(Debug,PartialEq)]
pub enum BoolCell { YES, NO }

use BoolCell::*;

impl FromStr for BoolCell {
   type Err = String;
   fn from_str(elt: &str) -> ErrStr<Self> {
      match elt.to_uppercase().as_str() {
         "YES" => Ok(YES),
         "NO"  => Ok(NO),
         _     => Err(format!("Unable to parse BoolCell from {elt}"))
      }
   }
}

impl fmt::Display for BoolCell {
   fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "{:?}", self)
   }
}

impl From<BoolCell> for bool {
   fn from(cell: BoolCell) -> bool { cell == YES }
}

