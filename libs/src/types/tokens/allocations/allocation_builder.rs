use crate::types::{ measurable::Measurable, util::Token };
use super::{ allocations::{ Allocation,mk_allocation }, committed::Committed };
use book::err_utils::ErrStr;

#[derive(Debug, Default)]
pub struct AllocationBuilder {
    token: Option<Token>,
    total: Option<f32>,
    committed: Option<Committed>
}

impl AllocationBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn total(mut self, total: f32) -> Self {
        self.total = Some(total);
        self
    }

    pub fn committed(mut self, committed: Committed) -> Self {
        self.committed = Some(committed);
        self
    }

    pub fn build(self) -> ErrStr<Allocation> {
        // Enforce that a token must be supplied
        let token = self.token.ok_or("Missing field: token")?;
        
        // Provide defaults if total or committed are omitted
        let total = self.total.ok_or("Missing field: total")?;
        let committed = self.committed.ok_or("Missing field: committed")?;

        // Optional business rule check
        let committed_sz = committed.sz();
        if committed_sz > total {
            let a = format!("Committed allocation ({committed_sz})");
            let b = format!("cannot exceed total allocation ({total})");
            Err(format!("{a} {b}"))
        } else {
           Ok(mk_allocation(&token, total, committed))
        }
    }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use crate::types::tokens::allocations::committed::mk_committed;

    #[test] fn test_successful_build_with_all_fields() {
        let alloc = Allocation::builder()
            .token("BTC")
            .total(100.0)
            .committed(mk_committed(42.0, 12.0))
            .build();

        assert!(alloc.is_ok());
        let alloc = alloc.unwrap();
        assert_eq!("BTC", alloc.key());
        assert_eq!(100.0, alloc.sz());
        assert_eq!(54.0, alloc.committed().sz());
    }

    #[test] fn test_string_and_str_acceptability() {
        // Test that passing an owned String works just like a &str literal
        let token_string = String::from("ETH");
        let alloc = Allocation::builder()
            .token(token_string)
            .total(50.0)
            .committed(mk_committed(10.0, 20.0))
            .build()
            .unwrap();

        assert_eq!("ETH", alloc.key());
    }

    #[test] fn test_missing_token_fails() {
        let alloc = Allocation::builder()
            .total(100.0)
            .committed(mk_committed(24.0, 26.1))
            .build();

        assert!(alloc.is_err());
        assert_eq!(alloc.unwrap_err(), "Missing field: token");
    }

    #[test] fn test_committed_cannot_exceed_total() {
        let alloc = Allocation::builder()
            .token("USDT")
            .total(100.0)
            .committed(mk_committed(72.5, 63.9))
                         // Fails business logic: committed > total
            .build();

        assert!(alloc.is_err());
    }

    #[test] fn test_committed_exactly_equals_total_succeeds() {
        let alloc = Allocation::builder()
            .token("USDC")
            .total(100.0)
            .committed(mk_committed(25.0, 25.0))
            .build();

        assert!(alloc.is_ok());
    }
}

