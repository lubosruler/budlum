use crate::core::address::Address;
use serde::{Deserialize, Serialize};
use std::fmt;

/// B.U.D. Name Service (BNS) — decentralized naming for the Budlum network.
/// Phase 6 started early by user decision (Q10 defer formal ADIM5 packaging).

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NameRecord {
    /// e.g. "ayaz.bud"
    pub name: String,
    /// Account that owns the name
    pub owner: Address,
    /// Epoch when the name expires
    pub expires_at: u64,
    /// Optional contract for complex resolution
    pub resolver: Option<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BnsError {
    InvalidName,
    NameTaken,
    NotOwner,
    Expired,
}

impl fmt::Display for BnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BnsError::InvalidName => write!(f, "Name too short or long"),
            BnsError::NameTaken => write!(f, "Name already taken"),
            BnsError::NotOwner => write!(f, "Not the owner"),
            BnsError::Expired => write!(f, "Name expired"),
        }
    }
}

impl std::error::Error for BnsError {}
