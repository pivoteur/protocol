#![crate_name = "libs"]

/// Distillation of pivot pool-management

/// Types used across the library
pub mod types;

/// Collections specific to Pivot Protocol dapps
pub mod collections;

/// Parsing row-data to tables
pub mod tables;

/// Resolves the paths of the pivot-pools
pub mod paths;

/// Fetch data from REST endpoints
pub mod fetchers;

/// reports, ... for when you want to report stuff
pub mod reports;

/// marshalling requests to the git API
pub mod git;

/// the whole kit-and-kaboodle
pub mod processors;
