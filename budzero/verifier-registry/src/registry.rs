//! Generic, permissionless verifier/relayer/attester registry.
//!
//! **One** registry, **one** staking mechanism, **one** slashing pipeline —
//! shared by every role (Master Verifier, Relayer, Attester, Storage Operator,
//! AI Verifier, or any future caller-defined role).
//!
//! ## Invariants
//! - The ONLY gate is meeting [`MIN_REGISTRATION_STAKE`]. No whitelist.
//! - Slashing one role automatically jails **all** other roles held by the
//!   same address (cross-role slashing).
//! - Evidence-gated slashing: only consensus-verified reports are acted on.
//! - Deterministic `state_root()` for snapshot/consensus commitment.

use crate::address::Address;
use crate::evidence::{EvidenceError, SlashingReport};
use crate::params::{RegistryParams, FIXED_POINT_SCALE};
use crate::role::RoleId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Default minimum stake required to register for a role.
pub const MIN_REGISTRATION_STAKE: u64 = 1_000;

/// Default number of epochs that unbonded stake stays locked.
pub const UNBONDING_EPOCHS: u64 = 7;

/// Reasons a registered member can be slashed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingCondition {
    /// Signed two conflicting blocks/messages at the same height/slot.
    DoubleSign,
    /// Failed liveness / availability obligations.
    LivenessFault,
    /// Provably malicious behaviour.
    MaliciousBehaviour,
}

impl SlashingCondition {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            SlashingCondition::DoubleSign => b"double_sign",
            SlashingCondition::LivenessFault => b"liveness_fault",
            SlashingCondition::MaliciousBehaviour => b"malicious_behaviour",
        }
    }
}

/// Lifecycle status of a registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStatus {
    Active,
    Unbonding { release_epoch: u64 },
    Slashed,
}

/// A single (account, role) registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub account: Address,
    pub role: RoleId,
    pub stake: u64,
    pub status: MemberStatus,
    pub registered_epoch: u64,
}

impl Registration {
    pub fn is_active(&self) -> bool {
        matches!(self.status, MemberStatus::Active) && self.stake > 0
    }
}

/// Errors surfaced by the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    InsufficientStake {
        required: u64,
        provided: u64,
    },
    AlreadyRegistered {
        account: Address,
        role: RoleId,
    },
    NotRegistered {
        account: Address,
        role: RoleId,
    },
    NotActive {
        account: Address,
        role: RoleId,
    },
    StillUnbonding {
        release_epoch: u64,
        current_epoch: u64,
    },
    RelayerNotActive {
        account: Address,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::InsufficientStake { required, provided } => {
                write!(
                    f,
                    "insufficient stake: required {required}, provided {provided}"
                )
            }
            RegistryError::AlreadyRegistered { account, role } => {
                write!(f, "{account} already registered as {role}")
            }
            RegistryError::NotRegistered { account, role } => {
                write!(f, "{account} is not registered as {role}")
            }
            RegistryError::NotActive { account, role } => {
                write!(f, "{account} is not active as {role}")
            }
            RegistryError::StillUnbonding {
                release_epoch,
                current_epoch,
            } => {
                write!(
                    f,
                    "stake still unbonding until epoch {release_epoch} (now {current_epoch})"
                )
            }
            RegistryError::RelayerNotActive { account } => {
                write!(f, "{account} is not an active relayer")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Outcome of a slashing action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashOutcome {
    pub condition: SlashingCondition,
    pub penalty: u64,
    pub remaining_stake: u64,
}

/// A persisted slashing record for audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashingRecord {
    pub report: SlashingReport,
    pub penalty: u64,
    pub remaining_stake: u64,
}

/// The generic, RoleId-based verifier registry.
///
/// Keyed by `(RoleId, Address)` so the same account may hold several roles
/// with independent stakes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifierRegistry {
    #[serde(with = "registrations_as_seq")]
    registrations: BTreeMap<(RoleId, Address), Registration>,
    #[serde(default)]
    params: RegistryParams,
    #[serde(default)]
    slashing_history: Vec<SlashingRecord>,
}

mod registrations_as_seq {
    use super::{Address, Registration, RoleId};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeMap;

    pub fn serialize<S>(
        map: &BTreeMap<(RoleId, Address), Registration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let entries: Vec<&Registration> = map.values().collect();
        entries.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<BTreeMap<(RoleId, Address), Registration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = Vec::<Registration>::deserialize(deserializer)?;
        Ok(entries
            .into_iter()
            .map(|r| ((r.role, r.account), r))
            .collect())
    }
}

impl VerifierRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_params(params: RegistryParams) -> Self {
        Self {
            registrations: BTreeMap::new(),
            params,
            slashing_history: Vec::new(),
        }
    }

    pub fn params(&self) -> &RegistryParams {
        &self.params
    }

    pub fn set_params(&mut self, params: RegistryParams) {
        self.params = params;
    }

    // ─── Registration ──────────────────────────────────────────────────

    /// Register `account` for `role` by bonding `stake`.
    ///
    /// **Permissionless**: any account may call it. The ONLY precondition is
    /// meeting the `min_stake` floor.
    pub fn register(
        &mut self,
        account: Address,
        role: RoleId,
        stake: u64,
        current_epoch: u64,
    ) -> Result<(), RegistryError> {
        if stake < self.params.min_stake {
            return Err(RegistryError::InsufficientStake {
                required: self.params.min_stake,
                provided: stake,
            });
        }
        let key = (role, account);
        if self.registrations.contains_key(&key) {
            return Err(RegistryError::AlreadyRegistered { account, role });
        }
        self.registrations.insert(
            key,
            Registration {
                account,
                role,
                stake,
                status: MemberStatus::Active,
                registered_epoch: current_epoch,
            },
        );
        Ok(())
    }

    /// Register as Master Verifier.
    pub fn register_master_verifier(
        &mut self,
        account: Address,
        stake: u64,
        current_epoch: u64,
    ) -> Result<(), RegistryError> {
        self.register(
            account,
            crate::role::roles::MASTER_VERIFIER,
            stake,
            current_epoch,
        )
    }

    /// Register as Relayer.
    pub fn register_relayer(
        &mut self,
        account: Address,
        stake: u64,
        current_epoch: u64,
    ) -> Result<(), RegistryError> {
        self.register(account, crate::role::roles::RELAYER, stake, current_epoch)
    }

    /// Register as Attester.
    pub fn register_attester(
        &mut self,
        account: Address,
        stake: u64,
        current_epoch: u64,
    ) -> Result<(), RegistryError> {
        self.register(account, crate::role::roles::ATTESTER, stake, current_epoch)
    }

    /// Register as Validator.
    pub fn register_validator(
        &mut self,
        account: Address,
        stake: u64,
        current_epoch: u64,
    ) -> Result<(), RegistryError> {
        self.register(account, crate::role::roles::VALIDATOR, stake, current_epoch)
    }

    // ─── Stake management ──────────────────────────────────────────────

    /// Idempotently sync a role's bonded stake to `total_stake`.
    pub fn upsert_stake(
        &mut self,
        account: Address,
        role: RoleId,
        total_stake: u64,
        current_epoch: u64,
    ) {
        let key = (role, account);
        match self.registrations.get_mut(&key) {
            Some(reg) => {
                if matches!(reg.status, MemberStatus::Slashed) {
                    reg.stake = total_stake;
                    return;
                }
                if total_stake == 0 {
                    self.registrations.remove(&key);
                } else {
                    reg.stake = total_stake;
                }
            }
            None => {
                if total_stake >= self.params.min_stake {
                    self.registrations.insert(
                        key,
                        Registration {
                            account,
                            role,
                            stake: total_stake,
                            status: MemberStatus::Active,
                            registered_epoch: current_epoch,
                        },
                    );
                }
            }
        }
    }

    /// Increase an existing bond.
    pub fn add_stake(
        &mut self,
        account: Address,
        role: RoleId,
        amount: u64,
    ) -> Result<u64, RegistryError> {
        let reg = self
            .registrations
            .get_mut(&(role, account))
            .ok_or(RegistryError::NotRegistered { account, role })?;
        reg.stake = reg.stake.saturating_add(amount);
        Ok(reg.stake)
    }

    // ─── Unbonding / withdrawal ────────────────────────────────────────

    /// Begin unbonding: stake locked until `current_epoch + unbonding_epochs`.
    pub fn begin_unbonding(
        &mut self,
        account: Address,
        role: RoleId,
        current_epoch: u64,
    ) -> Result<u64, RegistryError> {
        let reg = self
            .registrations
            .get_mut(&(role, account))
            .ok_or(RegistryError::NotRegistered { account, role })?;
        if !matches!(reg.status, MemberStatus::Active) {
            return Err(RegistryError::NotActive { account, role });
        }
        let release_epoch = current_epoch.saturating_add(self.params.unbonding_epochs);
        reg.status = MemberStatus::Unbonding { release_epoch };
        Ok(release_epoch)
    }

    /// Complete withdrawal after the unbonding period elapses.
    pub fn withdraw(
        &mut self,
        account: Address,
        role: RoleId,
        current_epoch: u64,
    ) -> Result<u64, RegistryError> {
        let reg = self
            .registrations
            .get(&(role, account))
            .ok_or(RegistryError::NotRegistered { account, role })?;
        match reg.status {
            MemberStatus::Unbonding { release_epoch } => {
                if current_epoch < release_epoch {
                    return Err(RegistryError::StillUnbonding {
                        release_epoch,
                        current_epoch,
                    });
                }
            }
            _ => return Err(RegistryError::NotActive { account, role }),
        }
        let reg = self
            .registrations
            .remove(&(role, account))
            .expect("checked");
        Ok(reg.stake)
    }

    // ─── Slashing ──────────────────────────────────────────────────────

    /// Slash a member for a proven offence.
    ///
    /// Applies a fixed-point ratio penalty and jails the member.
    /// Cross-role slashing: all other roles held by the same address are
    /// also jailed.
    pub fn slash(
        &mut self,
        account: Address,
        role: RoleId,
        condition: SlashingCondition,
        slash_ratio_fixed: u64,
    ) -> Result<SlashOutcome, RegistryError> {
        let reg = self
            .registrations
            .get_mut(&(role, account))
            .ok_or(RegistryError::NotRegistered { account, role })?;

        let penalty =
            ((reg.stake as u128 * slash_ratio_fixed as u128) / FIXED_POINT_SCALE as u128) as u64;
        reg.stake = reg.stake.saturating_sub(penalty);
        reg.status = MemberStatus::Slashed;
        let remaining_stake = reg.stake;

        // Cross-role slash: jail every other registration of the same address.
        self.slash_cross_role(account, role, slash_ratio_fixed);

        Ok(SlashOutcome {
            condition,
            penalty,
            remaining_stake,
        })
    }

    fn slash_cross_role(&mut self, account: Address, primary_role: RoleId, slash_ratio_fixed: u64) {
        let other_keys: Vec<RoleId> = self
            .registrations
            .keys()
            .filter_map(|(role, addr)| {
                if *addr == account && *role != primary_role {
                    Some(*role)
                } else {
                    None
                }
            })
            .collect();

        for role in other_keys {
            if let Some(reg) = self.registrations.get_mut(&(role, account)) {
                if matches!(reg.status, MemberStatus::Slashed) {
                    continue;
                }
                let penalty = ((reg.stake as u128 * slash_ratio_fixed as u128)
                    / FIXED_POINT_SCALE as u128) as u64;
                reg.stake = reg.stake.saturating_sub(penalty);
                reg.status = MemberStatus::Slashed;
            }
        }
    }

    /// Slash from a canonical [`SlashingReport`].
    ///
    /// Only acts on structurally valid AND consensus-verified reports.
    pub fn slash_from_report(
        &mut self,
        report: &SlashingReport,
    ) -> Result<Option<SlashOutcome>, EvidenceError> {
        report.is_actionable()?;
        let condition = report.condition();
        let ratio = self.params.slash_ratio(condition);
        match self.slash(report.offender, report.role, condition, ratio) {
            Ok(outcome) => {
                self.slashing_history.push(SlashingRecord {
                    report: report.clone(),
                    penalty: outcome.penalty,
                    remaining_stake: outcome.remaining_stake,
                });
                Ok(Some(outcome))
            }
            Err(RegistryError::NotRegistered { .. }) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    pub fn slashing_history(&self) -> &[SlashingRecord] {
        &self.slashing_history
    }

    pub fn slashing_history_for(&self, offender: &Address) -> Vec<&SlashingRecord> {
        self.slashing_history
            .iter()
            .filter(|r| &r.report.offender == offender)
            .collect()
    }

    // ─── Queries ───────────────────────────────────────────────────────

    pub fn get(&self, account: &Address, role: RoleId) -> Option<&Registration> {
        self.registrations.get(&(role, *account))
    }

    /// True iff the account is *actively* registered for the role.
    pub fn is_active(&self, account: &Address, role: RoleId) -> bool {
        self.get(account, role)
            .map(Registration::is_active)
            .unwrap_or(false)
    }

    /// All active members of a role.
    pub fn active_members(&self, role: RoleId) -> Vec<&Registration> {
        self.registrations
            .values()
            .filter(|r| r.role == role && r.is_active())
            .collect()
    }

    /// Relayer eligibility: Active OR Unbonding with positive stake.
    pub fn is_active_relayer(&self, account: &Address) -> bool {
        match self.get(account, crate::role::roles::RELAYER) {
            Some(reg) => {
                matches!(
                    reg.status,
                    MemberStatus::Active | MemberStatus::Unbonding { .. }
                ) && reg.stake > 0
            }
            None => false,
        }
    }

    /// Attester eligibility: Active OR Unbonding with positive stake.
    pub fn is_active_attester(&self, account: &Address) -> bool {
        match self.get(account, crate::role::roles::ATTESTER) {
            Some(reg) => {
                matches!(
                    reg.status,
                    MemberStatus::Active | MemberStatus::Unbonding { .. }
                ) && reg.stake > 0
            }
            None => false,
        }
    }

    /// Master Verifier eligibility: Active with positive stake.
    pub fn is_active_master_verifier(&self, account: &Address) -> bool {
        self.is_active(account, crate::role::roles::MASTER_VERIFIER)
    }

    /// Total stake bonded to a role (active registrations only).
    pub fn total_stake(&self, role: RoleId) -> u64 {
        self.registrations
            .values()
            .filter(|r| r.role == role && r.is_active())
            .map(|r| r.stake)
            .fold(0u64, |acc, s| acc.saturating_add(s))
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    pub fn registrations_as_seq(&self) -> Vec<&Registration> {
        self.registrations.values().collect()
    }

    // ─── State Root ────────────────────────────────────────────────────

    /// Deterministic domain-separated SHA-256 root of all registrations.
    pub fn state_root(&self) -> [u8; 32] {
        if self.is_empty() {
            return [0u8; 32];
        }
        let mut hasher = Sha256::new();
        hasher.update(b"BDLM_VERIFIER_REGISTRY_V1");
        for (key, reg) in &self.registrations {
            hasher.update(key.0.as_bytes());
            hasher.update(key.1.as_bytes());
            hasher.update(reg.stake.to_le_bytes());
            hasher.update((reg.registered_epoch).to_le_bytes());
            match reg.status {
                MemberStatus::Active => hasher.update(b"active"),
                MemberStatus::Unbonding { release_epoch } => {
                    hasher.update(b"unbonding");
                    hasher.update(release_epoch.to_le_bytes());
                }
                MemberStatus::Slashed => hasher.update(b"slashed"),
            }
        }
        hasher.finalize().into()
    }
}

// ─── Unit Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role::roles;

    fn addr(b: u8) -> Address {
        Address::from([b; 32])
    }

    // ─── Basic registration ────────────────────────────────────────

    #[test]
    fn anyone_can_register_by_staking() {
        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(1), MIN_REGISTRATION_STAKE, 0)
            .unwrap();
        assert!(reg.is_active_master_verifier(&addr(1)));
    }

    #[test]
    fn below_min_stake_rejected() {
        let mut reg = VerifierRegistry::new();
        let err = reg
            .register_relayer(addr(2), MIN_REGISTRATION_STAKE - 1, 0)
            .unwrap_err();
        assert!(matches!(err, RegistryError::InsufficientStake { .. }));
    }

    #[test]
    fn duplicate_registration_rejected() {
        let mut reg = VerifierRegistry::new();
        reg.register_attester(addr(3), MIN_REGISTRATION_STAKE, 0)
            .unwrap();
        assert!(reg
            .register_attester(addr(3), MIN_REGISTRATION_STAKE, 0)
            .is_err());
    }

    // ─── Multi-role: Master Verifier + Relayer + Attester ──────────

    #[test]
    fn same_account_can_hold_all_three_roles() {
        let mut reg = VerifierRegistry::new();
        let account = addr(7);
        reg.register_master_verifier(account, 5_000, 0).unwrap();
        reg.register_relayer(account, 3_000, 0).unwrap();
        reg.register_attester(account, 2_000, 0).unwrap();

        assert!(reg.is_active_master_verifier(&account));
        assert!(reg.is_active_relayer(&account));
        assert!(reg.is_active_attester(&account));

        assert_eq!(
            reg.get(&account, roles::MASTER_VERIFIER).unwrap().stake,
            5_000
        );
        assert_eq!(reg.get(&account, roles::RELAYER).unwrap().stake, 3_000);
        assert_eq!(reg.get(&account, roles::ATTESTER).unwrap().stake, 2_000);
    }

    // ─── Unbonding lifecycle ───────────────────────────────────────

    #[test]
    fn unbonding_locks_stake_until_release() {
        let mut reg = VerifierRegistry::new();
        reg.register_validator(addr(4), 5_000, 10).unwrap();
        let release = reg.begin_unbonding(addr(4), roles::VALIDATOR, 10).unwrap();
        assert_eq!(release, 10 + UNBONDING_EPOCHS);
        assert!(reg
            .withdraw(addr(4), roles::VALIDATOR, release - 1)
            .is_err());
        let released = reg.withdraw(addr(4), roles::VALIDATOR, release).unwrap();
        assert_eq!(released, 5_000);
        assert!(reg.get(&addr(4), roles::VALIDATOR).is_none());
    }

    #[test]
    fn cannot_withdraw_while_active() {
        let mut reg = VerifierRegistry::new();
        reg.register_attester(addr(8), MIN_REGISTRATION_STAKE, 0)
            .unwrap();
        assert!(reg.withdraw(addr(8), roles::ATTESTER, 100).is_err());
    }

    // ─── Slashing ──────────────────────────────────────────────────

    #[test]
    fn slashing_reduces_stake_and_jails() {
        let mut reg = VerifierRegistry::new();
        reg.register_validator(addr(5), 10_000, 0).unwrap();
        let outcome = reg
            .slash(
                addr(5),
                roles::VALIDATOR,
                SlashingCondition::DoubleSign,
                FIXED_POINT_SCALE / 2,
            )
            .unwrap();
        assert_eq!(outcome.penalty, 5_000);
        assert_eq!(outcome.remaining_stake, 5_000);
        assert!(!reg.is_active(&addr(5), roles::VALIDATOR));
    }

    #[test]
    fn cross_role_slash_jails_all_roles() {
        let mut reg = VerifierRegistry::new();
        let account = addr(77);
        reg.register_master_verifier(account, 10_000, 0).unwrap();
        reg.register_relayer(account, MIN_REGISTRATION_STAKE, 0)
            .unwrap();
        reg.register_attester(account, 2_000, 0).unwrap();

        assert!(reg.is_active_master_verifier(&account));
        assert!(reg.is_active_relayer(&account));
        assert!(reg.is_active_attester(&account));

        // Slash on MASTER_VERIFIER → all roles jailed
        reg.slash(
            account,
            roles::MASTER_VERIFIER,
            SlashingCondition::DoubleSign,
            FIXED_POINT_SCALE / 2,
        )
        .unwrap();

        assert!(!reg.is_active_master_verifier(&account));
        assert!(!reg.is_active_relayer(&account));
        assert!(!reg.is_active_attester(&account));

        // Relayer and attester stake also halved
        assert_eq!(
            reg.get(&account, roles::RELAYER).unwrap().stake,
            MIN_REGISTRATION_STAKE / 2
        );
        assert_eq!(reg.get(&account, roles::ATTESTER).unwrap().stake, 1_000);
    }

    #[test]
    fn malicious_slash_burns_entire_bond() {
        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(9), 10_000, 0).unwrap();
        let outcome = reg
            .slash(
                addr(9),
                roles::MASTER_VERIFIER,
                SlashingCondition::MaliciousBehaviour,
                FIXED_POINT_SCALE,
            )
            .unwrap();
        assert_eq!(outcome.penalty, 10_000);
        assert_eq!(outcome.remaining_stake, 0);
    }

    // ─── Evidence-gated slashing ───────────────────────────────────

    #[test]
    fn unverified_report_not_actionable() {
        use crate::evidence::{ProofProvenance, SlashingProof};

        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(10), 10_000, 0).unwrap();

        let report = SlashingReport::new(
            addr(10),
            roles::MASTER_VERIFIER,
            SlashingProof::Liveness {
                window_start_epoch: 0,
                window_end_epoch: 10,
                missed: 5,
                expected: 10,
            },
            ProofProvenance::Unverified,
            None,
        );

        let result = reg.slash_from_report(&report);
        assert!(result.is_err()); // EvidenceError::Unverified
                                  // Registration unchanged
        assert!(reg.is_active_master_verifier(&addr(10)));
    }

    #[test]
    fn consensus_verified_report_slashes() {
        use crate::evidence::{ProofProvenance, SlashingProof};

        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(11), 10_000, 0).unwrap();

        let report = SlashingReport::new(
            addr(11),
            roles::MASTER_VERIFIER,
            SlashingProof::Liveness {
                window_start_epoch: 0,
                window_end_epoch: 10,
                missed: 5,
                expected: 10,
            },
            ProofProvenance::ConsensusVerified,
            None,
        );

        let result = reg.slash_from_report(&report).unwrap();
        assert!(result.is_some());
        assert!(!reg.is_active_master_verifier(&addr(11)));
        assert_eq!(reg.slashing_history().len(), 1);
    }

    // ─── upsert_stake ──────────────────────────────────────────────

    #[test]
    fn upsert_creates_registration_above_floor() {
        let mut reg = VerifierRegistry::new();
        reg.upsert_stake(addr(20), roles::RELAYER, 5_000, 0);
        assert!(reg.is_active_relayer(&addr(20)));
        assert_eq!(reg.get(&addr(20), roles::RELAYER).unwrap().stake, 5_000);
    }

    #[test]
    fn upsert_noop_below_floor() {
        let mut reg = VerifierRegistry::new();
        reg.upsert_stake(addr(21), roles::RELAYER, 100, 0);
        assert!(!reg.is_active_relayer(&addr(21)));
        assert!(reg.get(&addr(21), roles::RELAYER).is_none());
    }

    #[test]
    fn upsert_removes_on_zero_stake() {
        let mut reg = VerifierRegistry::new();
        reg.register_relayer(addr(22), MIN_REGISTRATION_STAKE, 0)
            .unwrap();
        assert!(reg.is_active_relayer(&addr(22)));
        reg.upsert_stake(addr(22), roles::RELAYER, 0, 0);
        assert!(reg.get(&addr(22), roles::RELAYER).is_none());
    }

    // ─── Generic over arbitrary roles ──────────────────────────────

    #[test]
    fn arbitrary_role_works() {
        let mut reg = VerifierRegistry::new();
        let custom = RoleId::new(4242);
        reg.register(addr(30), custom, MIN_REGISTRATION_STAKE, 0)
            .unwrap();
        assert!(reg.is_active(&addr(30), custom));
        assert_eq!(reg.active_members(custom).len(), 1);
    }

    // ─── total_stake ───────────────────────────────────────────────

    #[test]
    fn total_stake_sums_active_only() {
        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(40), 5_000, 0).unwrap();
        reg.register_master_verifier(addr(41), 3_000, 0).unwrap();
        reg.register_master_verifier(addr(42), 2_000, 0).unwrap();
        // Slash one
        reg.slash(
            addr(42),
            roles::MASTER_VERIFIER,
            SlashingCondition::DoubleSign,
            FIXED_POINT_SCALE / 2,
        )
        .unwrap();

        // Total = 5000 + 3000 = 8000 (addr(42) is slashed → inactive)
        assert_eq!(reg.total_stake(roles::MASTER_VERIFIER), 8_000);
    }

    // ─── state_root ────────────────────────────────────────────────

    #[test]
    fn empty_registry_root_is_zero() {
        let reg = VerifierRegistry::new();
        assert_eq!(reg.state_root(), [0u8; 32]);
    }

    #[test]
    fn state_root_changes_on_registration() {
        let mut reg = VerifierRegistry::new();
        let root_before = reg.state_root();
        reg.register_master_verifier(addr(50), MIN_REGISTRATION_STAKE, 0)
            .unwrap();
        assert_ne!(root_before, reg.state_root());
    }

    #[test]
    fn state_root_changes_on_slash() {
        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(51), 10_000, 0).unwrap();
        let root_before = reg.state_root();
        reg.slash(
            addr(51),
            roles::MASTER_VERIFIER,
            SlashingCondition::DoubleSign,
            FIXED_POINT_SCALE / 2,
        )
        .unwrap();
        assert_ne!(root_before, reg.state_root());
    }

    #[test]
    fn state_root_is_deterministic() {
        let mut reg1 = VerifierRegistry::new();
        let mut reg2 = VerifierRegistry::new();
        reg1.register_master_verifier(addr(60), 5_000, 0).unwrap();
        reg1.register_relayer(addr(61), 3_000, 0).unwrap();

        reg2.register_master_verifier(addr(60), 5_000, 0).unwrap();
        reg2.register_relayer(addr(61), 3_000, 0).unwrap();

        assert_eq!(reg1.state_root(), reg2.state_root());
    }

    // ─── serde roundtrip ───────────────────────────────────────────

    #[test]
    fn serialization_roundtrip() {
        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(70), 5_000, 0).unwrap();
        reg.register_relayer(addr(70), 3_000, 0).unwrap();
        reg.register_attester(addr(71), 2_000, 0).unwrap();

        let json = serde_json::to_string(&reg).unwrap();
        let restored: VerifierRegistry = serde_json::from_str(&json).unwrap();

        assert_eq!(reg.len(), restored.len());
        assert!(restored.is_active_master_verifier(&addr(70)));
        assert!(restored.is_active_relayer(&addr(70)));
        assert!(restored.is_active_attester(&addr(71)));
        assert_eq!(reg.state_root(), restored.state_root());
    }

    // ─── add_stake ─────────────────────────────────────────────────

    #[test]
    fn add_stake_increases_bond() {
        let mut reg = VerifierRegistry::new();
        reg.register_master_verifier(addr(80), 5_000, 0).unwrap();
        let new_stake = reg
            .add_stake(addr(80), roles::MASTER_VERIFIER, 3_000)
            .unwrap();
        assert_eq!(new_stake, 8_000);
    }

    #[test]
    fn add_stake_unregistered_fails() {
        let mut reg = VerifierRegistry::new();
        assert!(reg.add_stake(addr(81), roles::RELAYER, 1_000).is_err());
    }
}
