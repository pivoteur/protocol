
/// A composition of two assets: a pivot pool
pub mod comps;

/// The historical-data of quotes of all the assets in the portfolio
pub mod quotes;

/// Represents a pivot (from -> to swap) and also recommends a proposal
pub mod pivots;

/// Alias one wrapped or synthetic token-name to the base asset
pub mod aliases;

/// trait that measures types (across divers types)
pub mod measurable;

/// types used across the systems, like Id, Token, Blochcain
pub mod util;

/// trait to define gains (ROI and APR)
pub mod gains;

