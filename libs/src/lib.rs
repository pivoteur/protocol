#![crate_name = "libs"]

/// Distillation of pivot pool-management

/// Types used across the library
pub mod types;

/// Parsing row-data to tables
pub mod tables;

/// parsers used to convert raw data to Rust-types
pub mod parsers;

/// Resolves the paths of the pivot-pools
pub mod paths;

/// Fetch data from REST endpoints
pub mod fetchers;

/// reports, ... for when you want to report stuff
pub mod reports;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
