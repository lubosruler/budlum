//! Phase 11.18 — MASAK/AML compliance primitives isolated to PoA domains.
//!
//! The permissionless network must never inherit PoA-only compliance hooks. This
//! module therefore requires every state-changing operation to declare its
//! domain kind and fails closed for [`ComplianceDomainKind::Permissionless`].

use crate::core::address::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceDomainKind {
    Permissionless,
    PoA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreeningStatus {
    Clear,
    Watchlist,
    Sanctioned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceAction {
    ScreeningUpdated,
    AccountFrozen,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreeningRecord {
    pub address: Address,
    pub status: ScreeningStatus,
    pub oracle_reference_hash: [u8; 32],
    pub updated_block: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreezeRecord {
    pub address: Address,
    pub reason_hash: [u8; 32],
    pub frozen_at_block: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceAuditEvent {
    pub action: ComplianceAction,
    pub address: Address,
    pub block: u64,
    pub evidence_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoaComplianceError {
    PermissionlessDomainForbidden,
    AdminRequired,
    ZeroEvidenceHash,
}

impl std::fmt::Display for PoaComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoaComplianceError::PermissionlessDomainForbidden => {
                write!(f, "PoA compliance hooks are forbidden in permissionless domains")
            }
            PoaComplianceError::AdminRequired => {
                write!(f, "PoA compliance admin approval required")
            }
            PoaComplianceError::ZeroEvidenceHash => {
                write!(f, "compliance evidence hash must be non-zero")
            }
        }
    }
}

impl std::error::Error for PoaComplianceError {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PoaComplianceRegistry {
    screenings: HashMap<Address, ScreeningRecord>,
    freezes: HashMap<Address, FreezeRecord>,
    audit_log: Vec<ComplianceAuditEvent>,
}

impl PoaComplianceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_poa(domain: ComplianceDomainKind) -> Result<(), PoaComplianceError> {
        match domain {
            ComplianceDomainKind::PoA => Ok(()),
            ComplianceDomainKind::Permissionless => {
                Err(PoaComplianceError::PermissionlessDomainForbidden)
            }
        }
    }

    fn ensure_non_zero(hash: [u8; 32]) -> Result<[u8; 32], PoaComplianceError> {
        if hash == [0u8; 32] {
            Err(PoaComplianceError::ZeroEvidenceHash)
        } else {
            Ok(hash)
        }
    }

    pub fn screen_address(
        &mut self,
        domain: ComplianceDomainKind,
        address: Address,
        status: ScreeningStatus,
        oracle_reference_hash: [u8; 32],
        block: u64,
    ) -> Result<(), PoaComplianceError> {
        Self::ensure_poa(domain)?;
        let evidence_hash = Self::ensure_non_zero(oracle_reference_hash)?;
        let record = ScreeningRecord {
            address,
            status,
            oracle_reference_hash: evidence_hash,
            updated_block: block,
        };
        self.screenings.insert(address, record);
        self.audit_log.push(ComplianceAuditEvent {
            action: ComplianceAction::ScreeningUpdated,
            address,
            block,
            evidence_hash,
        });
        Ok(())
    }

    pub fn freeze_suspicious(
        &mut self,
        domain: ComplianceDomainKind,
        admin_authorized: bool,
        address: Address,
        reason_hash: [u8; 32],
        block: u64,
    ) -> Result<(), PoaComplianceError> {
        Self::ensure_poa(domain)?;
        if !admin_authorized {
            return Err(PoaComplianceError::AdminRequired);
        }
        let evidence_hash = Self::ensure_non_zero(reason_hash)?;
        self.freezes.insert(
            address,
            FreezeRecord {
                address,
                reason_hash: evidence_hash,
                frozen_at_block: block,
            },
        );
        self.audit_log.push(ComplianceAuditEvent {
            action: ComplianceAction::AccountFrozen,
            address,
            block,
            evidence_hash,
        });
        Ok(())
    }

    pub fn screening(&self, address: &Address) -> Option<&ScreeningRecord> {
        self.screenings.get(address)
    }

    /// Permissionless domains deliberately ignore PoA freeze state.
    pub fn is_frozen(&self, domain: ComplianceDomainKind, address: &Address) -> bool {
        domain == ComplianceDomainKind::PoA && self.freezes.contains_key(address)
    }

    pub fn audit_events(&self) -> &[ComplianceAuditEvent] {
        &self.audit_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(byte: u8) -> Address {
        Address::from([byte; 32])
    }

    fn hash(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn phase11_18_poa_compliance_rejects_permissionless_screening() {
        let mut registry = PoaComplianceRegistry::new();
        let err = registry
            .screen_address(
                ComplianceDomainKind::Permissionless,
                addr(1),
                ScreeningStatus::Watchlist,
                hash(9),
                10,
            )
            .unwrap_err();
        assert_eq!(err, PoaComplianceError::PermissionlessDomainForbidden);
        assert!(registry.audit_events().is_empty());
    }

    #[test]
    fn phase11_18_poa_compliance_screening_updates_status() {
        let mut registry = PoaComplianceRegistry::new();
        registry
            .screen_address(
                ComplianceDomainKind::PoA,
                addr(2),
                ScreeningStatus::Clear,
                hash(1),
                10,
            )
            .unwrap();
        registry
            .screen_address(
                ComplianceDomainKind::PoA,
                addr(2),
                ScreeningStatus::Sanctioned,
                hash(2),
                11,
            )
            .unwrap();
        let record = registry.screening(&addr(2)).unwrap();
        assert_eq!(record.status, ScreeningStatus::Sanctioned);
        assert_eq!(record.updated_block, 11);
        assert_eq!(registry.audit_events().len(), 2);
    }

    #[test]
    fn phase11_18_poa_compliance_requires_admin_for_freeze() {
        let mut registry = PoaComplianceRegistry::new();
        let err = registry
            .freeze_suspicious(ComplianceDomainKind::PoA, false, addr(3), hash(3), 12)
            .unwrap_err();
        assert_eq!(err, PoaComplianceError::AdminRequired);
        assert!(!registry.is_frozen(ComplianceDomainKind::PoA, &addr(3)));
    }

    #[test]
    fn phase11_18_poa_compliance_freeze_is_poa_only() {
        let mut registry = PoaComplianceRegistry::new();
        registry
            .freeze_suspicious(ComplianceDomainKind::PoA, true, addr(4), hash(4), 13)
            .unwrap();
        assert!(registry.is_frozen(ComplianceDomainKind::PoA, &addr(4)));
        assert!(!registry.is_frozen(ComplianceDomainKind::Permissionless, &addr(4)));
    }

    #[test]
    fn phase11_18_poa_compliance_audit_log_is_append_only() {
        let mut registry = PoaComplianceRegistry::new();
        registry
            .screen_address(
                ComplianceDomainKind::PoA,
                addr(5),
                ScreeningStatus::Watchlist,
                hash(5),
                20,
            )
            .unwrap();
        registry
            .freeze_suspicious(ComplianceDomainKind::PoA, true, addr(5), hash(6), 21)
            .unwrap();
        assert_eq!(registry.audit_events().len(), 2);
        assert_eq!(registry.audit_events()[0].action, ComplianceAction::ScreeningUpdated);
        assert_eq!(registry.audit_events()[1].action, ComplianceAction::AccountFrozen);
    }

    #[test]
    fn phase11_18_poa_compliance_rejects_zero_evidence_hashes() {
        let mut registry = PoaComplianceRegistry::new();
        assert_eq!(
            registry
                .screen_address(
                    ComplianceDomainKind::PoA,
                    addr(6),
                    ScreeningStatus::Watchlist,
                    [0u8; 32],
                    30,
                )
                .unwrap_err(),
            PoaComplianceError::ZeroEvidenceHash
        );
        assert_eq!(
            registry
                .freeze_suspicious(ComplianceDomainKind::PoA, true, addr(6), [0u8; 32], 31)
                .unwrap_err(),
            PoaComplianceError::ZeroEvidenceHash
        );
    }
}
