use crate::chain::finality::{FinalityCert, ValidatorSetSnapshot};
use crate::core::block::Block;
use crate::domain::types::{ConsensusDomain, DomainCommitment, DomainId, Hash32};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FinalityStatus {
    Pending {
        required_depth: u64,
        observed_depth: u64,
    },
    Finalized,
    Rejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinalityProof {
    PoW {
        confirmations: u64,
        total_work_hint: u128,
        /// The domain block hash this PoW finality claim refers to. Must equal
        /// `commitment.domain_block_hash` — binds the proof to THIS commitment
        /// (Tur 6 hardening; previously the proof was unbound).
        #[serde(default)]
        declared_head_hash: Hash32,
        /// Declared cumulative proof-of-work up to `declared_head_hash`. Checked
        /// for internal consistency against `confirmations` and against the
        /// domain's minimum-work threshold. Not a full light client, but far
        /// stronger than "write any positive number".
        #[serde(default)]
        declared_cumulative_work: u128,
    },
    PoS {
        cert: FinalityCert,
        validator_snapshot: ValidatorSetSnapshot,
    },
    PoA {
        /// The KYC-approved authority set for this PoA domain (equal-weight, no
        /// stake — PoA deliberately has no stake concept). Order-independent;
        /// duplicates are ignored during verification.
        #[serde(default)]
        authorities: Vec<crate::core::address::Address>,
        /// Real ed25519 signatures over the commitment binding message, each by
        /// an authority (the authority's `Address` IS its ed25519 public key,
        /// per the chain-wide convention). Replaces the former self-reported
        /// `signer_count`/`validator_count` (Tur 7 hardening).
        #[serde(default)]
        signatures: Vec<PoAAuthoritySignature>,
    },
    Bft {
        round: u64,
        commit_hash: Hash32,
        /// Real BFT commit certificate (BLS aggregate over the validator set),
        /// verified cryptographically — replaces the former self-reported
        /// `signer_count`/`total_validators` (Tur 6 hardening).
        cert: FinalityCert,
        validator_snapshot: ValidatorSetSnapshot,
    },
    /// ZK finality (Tur 5, Option B): rather than carrying the raw STARK proof,
    /// this references a proof already submitted to — and cryptographically
    /// verified by — the `ProofClaimRegistry` (via `submit_zk_proof`). This keeps
    /// a single source of truth for ZK verification and removes the two parallel
    /// verification paths that the Tur-4/5 audit flagged.
    ///
    /// - `domain_id` / `target_height`: the `ProofClaimKey` to look up.
    /// - `final_state_root`: must match BOTH the accepted claim's root AND the
    ///   commitment's `state_root`, binding the proof to this specific commitment.
    Zk {
        domain_id: DomainId,
        target_height: u64,
        final_state_root: Hash32,
    },
    Raw(Vec<u8>),
}

/// A single PoA authority's ed25519 signature over a commitment binding message.
/// The `authority` address doubles as the ed25519 public key (chain-wide
/// convention, same as block producers).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PoAAuthoritySignature {
    pub authority: crate::core::address::Address,
    pub signature: Vec<u8>,
}

/// Canonical message a PoA authority signs to attest a commitment. Binds the
/// signature to the specific (domain, height, block hash), so a signature cannot
/// be replayed for a different commitment.
pub fn poa_commit_signing_message(
    domain_id: DomainId,
    domain_height: u64,
    domain_block_hash: &Hash32,
) -> Vec<u8> {
    let mut msg = Vec::with_capacity(8 + 8 + 32 + 16);
    msg.extend_from_slice(b"BUDLUM_POA_COMMIT_V1");
    msg.extend_from_slice(&domain_id.to_le_bytes());
    msg.extend_from_slice(&domain_height.to_le_bytes());
    msg.extend_from_slice(domain_block_hash);
    msg
}

#[derive(Debug, Clone)]
pub struct FinalityError(pub String);

impl std::fmt::Display for FinalityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Finality error: {}", self.0)
    }
}

impl std::error::Error for FinalityError {}

pub trait DomainFinalityAdapter: Send + Sync {
    fn adapter_name(&self) -> &'static str;

    fn verify_finality(
        &self,
        domain: &ConsensusDomain,
        commitment: &DomainCommitment,
        proof: &FinalityProof,
    ) -> Result<FinalityStatus, FinalityError>;
}

#[derive(Debug, Clone)]
pub struct PoWFinalityAdapter {
    pub default_min_confirmations: u64,
    /// Minimum plausible proof-of-work attributed to a single confirmation.
    /// Used to (a) reject cumulative-work claims that are inconsistent with the
    /// declared confirmation depth and (b) enforce a minimum-work floor. This is
    /// a config parameter, not a hard-coded constant (Tur 6).
    pub min_work_per_confirmation: u128,
}

impl Default for PoWFinalityAdapter {
    fn default() -> Self {
        Self {
            default_min_confirmations: 64,
            // Conservative non-zero floor: each confirmation must carry at least
            // this much declared work. Tunable per deployment.
            min_work_per_confirmation: 1,
        }
    }
}

impl DomainFinalityAdapter for PoWFinalityAdapter {
    fn adapter_name(&self) -> &'static str {
        "pow-confirmation-depth"
    }

    fn verify_finality(
        &self,
        domain: &ConsensusDomain,
        commitment: &DomainCommitment,
        proof: &FinalityProof,
    ) -> Result<FinalityStatus, FinalityError> {
        let min_depth = domain.min_confirmations.max(self.default_min_confirmations);
        match proof {
            FinalityProof::PoW {
                confirmations,
                total_work_hint,
                declared_head_hash,
                declared_cumulative_work,
            } => {
                // (1) Bind the proof to THIS commitment (Tur 6: previously the
                // commitment was ignored, so any commitment could borrow any
                // depth claim).
                if *declared_head_hash != commitment.domain_block_hash {
                    return Ok(FinalityStatus::Rejected(
                        "PoW declared head hash does not match commitment block hash".into(),
                    ));
                }

                // (2) Cumulative work must be positive (supersedes the old
                // symbolic `total_work_hint > 0`).
                if *declared_cumulative_work == 0 {
                    return Ok(FinalityStatus::Rejected(
                        "PoW declared cumulative work is zero".into(),
                    ));
                }

                // (3) Internal consistency: the declared cumulative work must be
                // at least `confirmations * min_work_per_confirmation`. This stops
                // a submitter from claiming a huge confirmation depth backed by a
                // tiny amount of work (the core weakness the audit flagged).
                let required_work = (*confirmations as u128)
                    .saturating_mul(self.min_work_per_confirmation);
                if *declared_cumulative_work < required_work {
                    return Ok(FinalityStatus::Rejected(format!(
                        "PoW declared work {} inconsistent with {} confirmations (need >= {})",
                        declared_cumulative_work, confirmations, required_work
                    )));
                }

                // (4) `total_work_hint` must not contradict the declared work.
                if *total_work_hint > *declared_cumulative_work {
                    return Ok(FinalityStatus::Rejected(
                        "PoW total_work_hint exceeds declared cumulative work".into(),
                    ));
                }

                // (5) Depth threshold.
                if *confirmations >= min_depth {
                    Ok(FinalityStatus::Finalized)
                } else {
                    Ok(FinalityStatus::Pending {
                        required_depth: min_depth,
                        observed_depth: *confirmations,
                    })
                }
            }
            _ => Err(FinalityError("Expected PoW finality proof".into())),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PoSFinalityAdapter;

impl DomainFinalityAdapter for PoSFinalityAdapter {
    fn adapter_name(&self) -> &'static str {
        "pos-qc-finality"
    }

    fn verify_finality(
        &self,
        domain: &ConsensusDomain,
        commitment: &DomainCommitment,
        proof: &FinalityProof,
    ) -> Result<FinalityStatus, FinalityError> {
        let FinalityProof::PoS {
            cert,
            validator_snapshot,
        } = proof
        else {
            return Err(FinalityError("Expected PoS finality proof".into()));
        };

        if cert.checkpoint_height != commitment.domain_height {
            return Ok(FinalityStatus::Rejected(
                "PoS cert height does not match commitment".into(),
            ));
        }

        let commitment_hash = hex::encode(commitment.domain_block_hash);
        if cert.checkpoint_hash != commitment_hash {
            return Ok(FinalityStatus::Rejected(
                "PoS cert hash does not match commitment".into(),
            ));
        }

        if validator_snapshot.set_hash != cert.set_hash {
            return Ok(FinalityStatus::Rejected(
                "PoS cert set hash does not match validator snapshot".into(),
            ));
        }

        if let Ok(decoded_set_hash) = hex::decode(&validator_snapshot.set_hash) {
            if decoded_set_hash.len() == 32 {
                let mut snapshot_set_hash = [0u8; 32];
                snapshot_set_hash.copy_from_slice(&decoded_set_hash);
                if domain.validator_set_hash != [0u8; 32]
                    && snapshot_set_hash != domain.validator_set_hash
                {
                    return Ok(FinalityStatus::Rejected(
                        "PoS validator snapshot does not match registered domain set".into(),
                    ));
                }
                if commitment.validator_set_hash != [0u8; 32]
                    && commitment.validator_set_hash != snapshot_set_hash
                {
                    return Ok(FinalityStatus::Rejected(
                        "PoS commitment validator set does not match finality proof".into(),
                    ));
                }
            }
        }

        cert.verify(validator_snapshot)
            .map_err(|e| FinalityError(format!("Invalid PoS finality cert: {}", e)))?;

        Ok(FinalityStatus::Finalized)
    }
}

#[derive(Debug, Clone)]
pub struct PoAFinalityAdapter {
    /// Count-based quorum numerator (PoA is equal-weight, NOT stake-weighted —
    /// PoA deliberately has no stake concept, preserving Tur 1-2 isolation).
    pub quorum_numerator: u64,
    /// Count-based quorum denominator.
    pub quorum_denominator: u64,
}

impl Default for PoAFinalityAdapter {
    fn default() -> Self {
        Self {
            quorum_numerator: 2,
            quorum_denominator: 3,
        }
    }
}

impl PoAFinalityAdapter {
    /// Number of authority signatures required for finality: ceil(N * num / den).
    pub fn required_signatures(&self, authority_count: usize) -> usize {
        ((authority_count as u64 * self.quorum_numerator).div_ceil(self.quorum_denominator)) as usize
    }
}

impl DomainFinalityAdapter for PoAFinalityAdapter {
    fn adapter_name(&self) -> &'static str {
        "poa-authority-quorum"
    }

    fn verify_finality(
        &self,
        domain: &ConsensusDomain,
        commitment: &DomainCommitment,
        proof: &FinalityProof,
    ) -> Result<FinalityStatus, FinalityError> {
        // Tur 7: PoA finality now verifies REAL ed25519 signatures from the
        // approved authority set (count-based quorum), instead of trusting a
        // self-reported signer_count. `domain` and `commitment` are genuinely
        // used. This does NOT touch the permissionless stake registry — PoA
        // keeps its own separate, stake-free authority/signature model
        // (isolation from Tur 1-2 preserved).
        let FinalityProof::PoA {
            authorities,
            signatures,
        } = proof
        else {
            return Err(FinalityError("Expected PoA finality proof".into()));
        };

        if authorities.is_empty() {
            return Ok(FinalityStatus::Rejected(
                "PoA authority set is empty".into(),
            ));
        }

        // De-duplicate the declared authority set (order-independent).
        let authority_set: std::collections::BTreeSet<crate::core::address::Address> =
            authorities.iter().copied().collect();

        // The message every authority must have signed, bound to THIS commitment.
        let msg = poa_commit_signing_message(
            domain.id,
            commitment.domain_height,
            &commitment.domain_block_hash,
        );

        // Count DISTINCT authorities with a valid signature over `msg`.
        let mut valid_signers: std::collections::BTreeSet<crate::core::address::Address> =
            std::collections::BTreeSet::new();
        for sig in signatures {
            // The signer must be a member of the declared authority set.
            if !authority_set.contains(&sig.authority) {
                return Ok(FinalityStatus::Rejected(
                    "PoA signature from a non-authority".into(),
                ));
            }
            // Verify the real ed25519 signature (authority address == pubkey).
            if crate::crypto::primitives::verify_signature(
                &msg,
                &sig.signature,
                sig.authority.as_bytes(),
            )
            .is_err()
            {
                return Ok(FinalityStatus::Rejected(
                    "PoA signature verification failed".into(),
                ));
            }
            valid_signers.insert(sig.authority);
        }

        let required = self.required_signatures(authority_set.len());
        if valid_signers.len() >= required {
            Ok(FinalityStatus::Finalized)
        } else {
            Ok(FinalityStatus::Pending {
                required_depth: required as u64,
                observed_depth: valid_signers.len() as u64,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct BftFinalityAdapter {
    /// Retained for API/config compatibility. NOTE (Tur 6): the effective quorum
    /// is now enforced cryptographically inside `FinalityCert::verify` via
    /// `ValidatorSetSnapshot::quorum_stake()` (stake-weighted, using the global
    /// `FINALITY_QUORUM_*` constants), not by these fields.
    pub quorum_numerator: u64,
    pub quorum_denominator: u64,
}

impl Default for BftFinalityAdapter {
    fn default() -> Self {
        Self {
            quorum_numerator: 2,
            quorum_denominator: 3,
        }
    }
}

impl DomainFinalityAdapter for BftFinalityAdapter {
    fn adapter_name(&self) -> &'static str {
        "bft-quorum-commit"
    }

    fn verify_finality(
        &self,
        domain: &ConsensusDomain,
        commitment: &DomainCommitment,
        proof: &FinalityProof,
    ) -> Result<FinalityStatus, FinalityError> {
        // Tur 6: BFT now verifies a REAL commit certificate (BLS aggregate over
        // the validator set) using the same primitive as PoS
        // (`FinalityCert::verify`), instead of trusting a self-reported
        // `signer_count`. `domain` and `commitment` are genuinely used.
        let FinalityProof::Bft {
            round: _,
            commit_hash,
            cert,
            validator_snapshot,
        } = proof
        else {
            return Err(FinalityError("Expected BFT finality proof".into()));
        };

        if validator_snapshot.validators.is_empty() {
            return Ok(FinalityStatus::Rejected(
                "BFT validator set is empty".into(),
            ));
        }

        // Bind the commit hash to THIS commitment's block hash.
        if *commit_hash != commitment.domain_block_hash {
            return Ok(FinalityStatus::Rejected(
                "BFT commit hash does not match commitment block hash".into(),
            ));
        }

        // Bind the certificate to THIS commitment (height + hash).
        if cert.checkpoint_height != commitment.domain_height {
            return Ok(FinalityStatus::Rejected(
                "BFT cert height does not match commitment".into(),
            ));
        }
        let commitment_hash = hex::encode(commitment.domain_block_hash);
        if cert.checkpoint_hash != commitment_hash {
            return Ok(FinalityStatus::Rejected(
                "BFT cert hash does not match commitment".into(),
            ));
        }

        // Bind cert/snapshot together and to the registered domain set.
        if validator_snapshot.set_hash != cert.set_hash {
            return Ok(FinalityStatus::Rejected(
                "BFT cert set hash does not match validator snapshot".into(),
            ));
        }
        if let Ok(decoded_set_hash) = hex::decode(&validator_snapshot.set_hash) {
            if decoded_set_hash.len() == 32 {
                let mut snapshot_set_hash = [0u8; 32];
                snapshot_set_hash.copy_from_slice(&decoded_set_hash);
                if domain.validator_set_hash != [0u8; 32]
                    && snapshot_set_hash != domain.validator_set_hash
                {
                    return Ok(FinalityStatus::Rejected(
                        "BFT validator snapshot does not match registered domain set".into(),
                    ));
                }
                if commitment.validator_set_hash != [0u8; 32]
                    && commitment.validator_set_hash != snapshot_set_hash
                {
                    return Ok(FinalityStatus::Rejected(
                        "BFT commitment validator set does not match finality proof".into(),
                    ));
                }
            }
        }

        // Cryptographic quorum + aggregate-signature verification.
        cert.verify(validator_snapshot)
            .map_err(|e| FinalityError(format!("Invalid BFT finality cert: {}", e)))?;

        Ok(FinalityStatus::Finalized)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZkFinalityAdapter;

impl ZkFinalityAdapter {
    /// Verify ZK finality against an already-accepted proof claim (Tur 5,
    /// Option B).
    ///
    /// The raw STARK proof is NOT re-verified here — it was already
    /// cryptographically verified when it was submitted via `submit_zk_proof`
    /// and recorded in the `ProofClaimRegistry`. This method enforces the
    /// binding the audit found missing:
    ///
    /// - `accepted_claim_root` is the `final_state_root` of the claim the
    ///   registry accepted for `(domain_id, target_height)` — `None` if no such
    ///   claim exists.
    /// - It must match BOTH the proof's declared `final_state_root` AND the
    ///   `commitment.state_root`, so a finality request cannot borrow a proof
    ///   accepted for a different state.
    ///
    /// `domain` and `commitment` are now genuinely USED (the audit flagged their
    /// former underscore-ignored state).
    pub fn verify_finality_with_claim(
        &self,
        domain: &ConsensusDomain,
        commitment: &DomainCommitment,
        proof: &FinalityProof,
        accepted_claim_root: Option<Hash32>,
    ) -> Result<FinalityStatus, FinalityError> {
        let FinalityProof::Zk {
            domain_id,
            target_height,
            final_state_root,
        } = proof
        else {
            return Err(FinalityError("Expected ZK finality proof".into()));
        };

        // The proof reference must be for THIS domain and THIS commitment height.
        if *domain_id != domain.id {
            return Ok(FinalityStatus::Rejected(format!(
                "ZK proof domain {} does not match commitment domain {}",
                domain_id, domain.id
            )));
        }
        if *target_height != commitment.domain_height {
            return Ok(FinalityStatus::Rejected(format!(
                "ZK proof height {} does not match commitment height {}",
                target_height, commitment.domain_height
            )));
        }

        // There must be an accepted, cryptographically-verified claim.
        let claim_root = match accepted_claim_root {
            Some(root) => root,
            None => {
                return Ok(FinalityStatus::Rejected(
                    "no accepted ZK proof for this (domain, height) claim".into(),
                ));
            }
        };

        // Bind the proof to the accepted claim...
        if claim_root != *final_state_root {
            return Ok(FinalityStatus::Rejected(
                "ZK proof final_state_root does not match accepted claim".into(),
            ));
        }
        // ...and bind the claim to THIS commitment (the missing link the audit
        // called out).
        if commitment.state_root != *final_state_root {
            return Ok(FinalityStatus::Rejected(
                "ZK proof/commitment state root mismatch".into(),
            ));
        }

        Ok(FinalityStatus::Finalized)
    }
}

impl DomainFinalityAdapter for ZkFinalityAdapter {
    fn adapter_name(&self) -> &'static str {
        "zk-proof-verification"
    }

    /// The generic trait entry point cannot reach the `ProofClaimRegistry`, so
    /// it must NEVER finalise on its own. ZK finality is resolved exclusively
    /// through `Blockchain::verify_domain_commitment_finality`, which calls
    /// [`ZkFinalityAdapter::verify_finality_with_claim`] with the registry
    /// lookup. This fail-closed default prevents a second, registry-less
    /// verification path from re-emerging.
    fn verify_finality(
        &self,
        _domain: &ConsensusDomain,
        _commitment: &DomainCommitment,
        _proof: &FinalityProof,
    ) -> Result<FinalityStatus, FinalityError> {
        Ok(FinalityStatus::Rejected(
            "ZK finality must be resolved via the ProofClaimRegistry (verify_finality_with_claim)"
                .into(),
        ))
    }
}

pub fn hash_finality_proof(proof: &FinalityProof) -> [u8; 32] {
    // SECURITY (Tur 11): must not silently hash empty bytes on serialize failure
    // — two distinct proofs could collide. Fail-fast on the (deterministic,
    // non-attacker-triggerable) programming error instead.
    let encoded = bincode::serialize(proof)
        .expect("BUG: FinalityProof must serialize for finality proof hash");
    crate::core::hash::hash_fields_bytes(&[b"BDLM_FINALITY_PROOF_V1", &encoded])
}

pub fn empty_event_root() -> [u8; 32] {
    crate::core::hash::hash_fields_bytes(&[b"BDLM_EMPTY_DOMAIN_EVENT_ROOT_V1"])
}

pub fn block_finality_proof_hash(_block: &Block) -> [u8; 32] {
    crate::core::hash::hash_fields_bytes(&[b"BDLM_NO_FINALITY_PROOF_YET_V1"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::finality::FinalityCert;
    use crate::domain::plugin::default_domain;
    use crate::domain::types::{ConsensusKind, DomainCommitment};

    fn commitment(kind: ConsensusKind) -> DomainCommitment {
        DomainCommitment {
            domain_id: 1,
            domain_height: 10,
            domain_block_hash: [1u8; 32],
            parent_domain_block_hash: [0u8; 32],
            state_root: [2u8; 32],
            tx_root: [3u8; 32],
            event_root: [4u8; 32],
            finality_proof_hash: [5u8; 32],
            consensus_kind: kind,
            validator_set_hash: [6u8; 32],
            timestamp_ms: 123,
            sequence: 0,
            producer: None,
            state_updates: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn pow_finality_requires_confirmation_depth_and_rejects_wrong_proof() {
        let domain = default_domain(1, ConsensusKind::PoW, 1337, "pow-confirmation-depth", 80);
        let commitment = commitment(ConsensusKind::PoW);
        let adapter = PoWFinalityAdapter::default();

        assert_eq!(
            adapter
                .verify_finality(
                    &domain,
                    &commitment,
                    &FinalityProof::PoW {
                        confirmations: 79,
                        total_work_hint: 79,
                        declared_head_hash: [1u8; 32],
                        declared_cumulative_work: 79,
                    },
                )
                .unwrap(),
            FinalityStatus::Pending {
                required_depth: 80,
                observed_depth: 79,
            }
        );

        assert_eq!(
            adapter
                .verify_finality(
                    &domain,
                    &commitment,
                    &FinalityProof::PoW {
                        confirmations: 80,
                        total_work_hint: 80,
                        declared_head_hash: [1u8; 32],
                        declared_cumulative_work: 80,
                    },
                )
                .unwrap(),
            FinalityStatus::Finalized
        );

        assert!(adapter
            .verify_finality(
                &domain,
                &commitment,
                &FinalityProof::PoA {
                    authorities: vec![],
                    signatures: vec![],
                },
            )
            .is_err());
    }

    #[test]
    fn poa_finality_enforces_quorum_and_empty_validator_set_rejection() {
        use crate::crypto::primitives::KeyPair;
        let domain = default_domain(2, ConsensusKind::PoA, 1337, "poa-authority-quorum", 0);
        let commitment = commitment(ConsensusKind::PoA);
        let adapter = PoAFinalityAdapter::default();

        // Build 4 real ed25519 authorities and sign the commit message.
        let mut kps = Vec::new();
        let mut authorities = Vec::new();
        for i in 0..4u8 {
            let mut seed = [0u8; 32];
            seed[0] = 0xB0 + i;
            let kp = KeyPair::from_seed(&seed).unwrap();
            authorities.push(crate::core::address::Address::from(kp.public_key_bytes()));
            kps.push(kp);
        }
        let msg = poa_commit_signing_message(
            domain.id,
            commitment.domain_height,
            &commitment.domain_block_hash,
        );
        let sig = |i: usize| PoAAuthoritySignature {
            authority: authorities[i],
            signature: kps[i].sign(&msg).to_vec(),
        };

        // 2 of 4 signatures -> pending (need ceil(4*2/3)=3).
        assert_eq!(
            adapter
                .verify_finality(
                    &domain,
                    &commitment,
                    &FinalityProof::PoA {
                        authorities: authorities.clone(),
                        signatures: vec![sig(0), sig(1)],
                    },
                )
                .unwrap(),
            FinalityStatus::Pending {
                required_depth: 3,
                observed_depth: 2,
            }
        );
        // 3 of 4 -> finalized.
        assert_eq!(
            adapter
                .verify_finality(
                    &domain,
                    &commitment,
                    &FinalityProof::PoA {
                        authorities: authorities.clone(),
                        signatures: vec![sig(0), sig(1), sig(2)],
                    },
                )
                .unwrap(),
            FinalityStatus::Finalized
        );
        // Empty authority set -> rejected.
        assert!(matches!(
            adapter
                .verify_finality(
                    &domain,
                    &commitment,
                    &FinalityProof::PoA {
                        authorities: vec![],
                        signatures: vec![],
                    },
                )
                .unwrap(),
            FinalityStatus::Rejected(_)
        ));
    }

    #[test]
    fn pos_finality_rejects_mismatched_height_or_hash_before_signature_work() {
        let domain = default_domain(3, ConsensusKind::PoS, 1337, "pos-qc-finality", 0);
        let commitment = commitment(ConsensusKind::PoS);
        let adapter = PoSFinalityAdapter;
        let snapshot = ValidatorSetSnapshot::new(0, vec![]);

        let wrong_height = FinalityCert {
            epoch: 0,
            checkpoint_height: 9,
            checkpoint_hash: hex::encode(commitment.domain_block_hash),
            agg_sig_bls: vec![],
            bitmap: vec![],
            set_hash: snapshot.set_hash.clone(),
        };
        assert!(matches!(
            adapter
                .verify_finality(
                    &domain,
                    &commitment,
                    &FinalityProof::PoS {
                        cert: wrong_height,
                        validator_snapshot: snapshot.clone(),
                    },
                )
                .unwrap(),
            FinalityStatus::Rejected(_)
        ));

        let wrong_hash = FinalityCert {
            epoch: 0,
            checkpoint_height: commitment.domain_height,
            checkpoint_hash: "ff".repeat(32),
            agg_sig_bls: vec![],
            bitmap: vec![],
            set_hash: snapshot.set_hash.clone(),
        };
        assert!(matches!(
            adapter
                .verify_finality(
                    &domain,
                    &commitment,
                    &FinalityProof::PoS {
                        cert: wrong_hash,
                        validator_snapshot: snapshot,
                    },
                )
                .unwrap(),
            FinalityStatus::Rejected(_)
        ));
    }
}
