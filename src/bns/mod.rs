//! Compatibility facade; canonical BNS ownership is the `BNS/` workspace crate.
pub use budlum_bns::{BnsError, BnsRegistry, BnsResolved, NameRecord};
/// Compatibility path for callers migrating from `crate::bns::types`.
pub mod types {
    pub use budlum_bns::{BnsError, BnsResolved, NameRecord};
}
