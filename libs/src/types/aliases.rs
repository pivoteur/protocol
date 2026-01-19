use std::collections::HashMap;

use super::util::Token;

type Alias = HashMap<Token, Token>;

#[derive(Debug, Clone)]
pub struct Aliases { aliaz: Alias }

pub fn aliases() -> Aliases {
   let mut aliaz = HashMap::new();
   fn inserter(h: &mut Alias, t1: &str, t2: &str) {
      h.insert(t1.to_string(), t2.to_string());
   }
   inserter(&mut aliaz, "WBTC", "BTC");
   inserter(&mut aliaz, "IBTC", "BTC");
   inserter(&mut aliaz, "WETH", "ETH");
   inserter(&mut aliaz, "IETH", "ETH");
   inserter(&mut aliaz, "IUSD", "USDC");
   inserter(&mut aliaz, "ISOL", "SOL");
   Aliases { aliaz }
}

impl Aliases {
   pub fn alias(&self, t: &str) -> Token {
      let cap = t.to_uppercase();
      if let Some(ans) = self.aliaz.get(&cap).or(Some(&cap)) {
         ans.to_string()
      } else {
         panic!("No alias for {cap}")
      }
   }
}
