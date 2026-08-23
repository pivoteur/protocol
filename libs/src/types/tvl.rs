use serde::Serialize;
use serde_with::{ serde_as, DisplayFromStr };
use book::currency::usd::USD;

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct tvl {
   #[serde_as(as = "DisplayFromStr")]
   total: USD,
   #[serde_as(as = "DisplayFromStr")]
   #[serde(rename = "virtual")]
   virtual_amt: USD,
   #[serde_as(as = "DisplayFromStr")]
   available: USD
}`

                                                   
~                             
