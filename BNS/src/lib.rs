//! BNS — Budlum Name Service workspace crate.
pub mod registry;
pub mod types;
pub use registry::BnsRegistry;
pub use types::{BnsError, BnsResolved, NameRecord};
