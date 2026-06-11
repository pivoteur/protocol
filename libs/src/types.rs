/// Alias one wrapped or synthetic token-name to the base asset
pub mod aliases;

/// Assets used across pivots and proposals
pub mod assets;

/// Enumerated blockchains that we support-...ish.
pub mod blockchains;

/// A proposed call
pub mod calls;

/// A composition of two assets: a pivot pool
pub mod comps;

/// trait to define gains (ROI and APR)
pub mod gains;

/// header information for pivots and aggregate headers for proposals
pub mod headers;

/// trait that measures types (across divers types)
pub mod measurable;

/// The historical-data of quotes of all the assets in the portfolio
pub mod quotes;

/// Represents a pivot (from -> to swap)
pub mod pivots;

/// recommends proposals
pub mod proposals;

/// Representation of tokens, blockchains, and their amounts
pub mod tokens;

/// types used across the systems, like Id, Token, Blochcain
pub mod util;

