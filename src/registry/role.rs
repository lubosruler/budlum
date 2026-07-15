//! Generic, extensible role / permission types for the permissionless registry.
//!
//! Per the Budlum master-context rules the permissionless verifier/validator/
//! relayer registry MUST be a shared primitive for future application layers.
//! To avoid scope-locking it to a fixed set of roles, roles are modelled as an
//! open `RoleId` newtype rather than a hard-coded `enum`. The well-known roles
//! (validator/verifier/relayer) are provided as constants for convenience, but
//! the registry accepts *any* `RoleId` — new roles can be introduced by callers
//! without changing this module or breaking existing tests.

use serde::{Deserialize, Serialize};

/// Open, extensible role identifier.
///
/// This is intentionally NOT an enum: the registry is a generic primitive and
/// future application layers (outside the scope of this instruction set) may
/// define their own roles. Reserve the low range for protocol-level roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u32);

impl RoleId {
    pub const fn new(id: u32) -> Self {
        RoleId(id)
    }

    pub const fn value(&self) -> u32 {
        self.0
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            roles::VALIDATOR => write!(f, "validator"),
            roles::VERIFIER => write!(f, "verifier"),
            roles::RELAYER => write!(f, "relayer"),
            roles::PROVER => write!(f, "prover"),
            roles::STORAGE_OPERATOR => write!(f, "storage_operator"),
            RoleId(id) => write!(f, "role#{id}"),
        }
    }
}

/// Well-known protocol-level roles. These are *conveniences*, not an exhaustive
/// list — the registry never checks membership against this set.
pub mod roles {
    use super::RoleId;

    /// Consensus block-producing validator (PoW/PoS/BFT domains).
    pub const VALIDATOR: RoleId = RoleId(1);
    /// Settlement / proof verifier.
    pub const VERIFIER: RoleId = RoleId(2);
    /// Cross-domain message relayer.
    pub const RELAYER: RoleId = RoleId(3);
    /// ZK proof producer (BudZKVM prover). Registration is OPTIONAL — proof
    /// submission is fully permissionless (STARK proofs are self-verifying);
    /// registering as a PROVER is only required to be eligible for rewards.
    pub const PROVER: RoleId = RoleId(4);
    /// B.U.D. storage operator (Tur 14, Faz 1).
    ///
    /// Registration is OPTIONAL — opening a `StorageDeal` is itself
    /// permissionless (the deal's `operator_bond` is the only gate, see
    /// `domain::storage_deal::StorageRegistry::open_deal`). Registering as
    /// `STORAGE_OPERATOR` is only required to be eligible for the
    /// per-deal reward stream.
    ///
    /// **Note (ADIM3 §0.3):** A `bud_storageActiveOperators` RPC method was
    /// planned for Tur 14.5 but is **not yet implemented**. Querying active
    /// operators currently requires iterating `PermissionlessRegistry` entries
    /// filtered by `RoleId::STORAGE_OPERATOR`.
    ///
    /// Like every other role, it is permissionless: any account can
    /// register by staking the `min_stake` floor from
    /// `PermissionlessRegistry::params`. No whitelist, no admin gate
    /// (master context, CLAUDE.md §2).
    pub const STORAGE_OPERATOR: RoleId = RoleId(5);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary_role_ids_are_allowed() {
        // A caller-defined role that this module never enumerated still works.
        let custom = RoleId::new(9_999);
        assert_eq!(custom.value(), 9_999);
        assert_eq!(format!("{custom}"), "role#9999");
    }

    #[test]
    fn well_known_roles_render_names() {
        assert_eq!(format!("{}", roles::VALIDATOR), "validator");
        assert_eq!(format!("{}", roles::VERIFIER), "verifier");
        assert_eq!(format!("{}", roles::RELAYER), "relayer");
        assert_eq!(format!("{}", roles::PROVER), "prover");
        assert_eq!(format!("{}", roles::STORAGE_OPERATOR), "storage_operator");
    }

    #[test]
    fn storage_operator_role_id_value_is_5() {
        // Pin the protocol-level role id (5) so a future bump is a
        // deliberate, audited change.
        assert_eq!(roles::STORAGE_OPERATOR.value(), 5);
    }
}
