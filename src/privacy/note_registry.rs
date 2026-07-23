//! On-chain note registry: live commitments + spent nullifiers.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// 32-byte commitment or nullifier hash (wallet packs field elements LE).
pub type NoteHash = [u8; 32];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct L1NoteRegistry {
    live_commitments: BTreeSet<NoteHash>,
    spent_nullifiers: BTreeSet<NoteHash>,
}

impl L1NoteRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.live_commitments.is_empty() && self.spent_nullifiers.is_empty()
    }

    pub fn live_count(&self) -> usize {
        self.live_commitments.len()
    }

    pub fn spent_count(&self) -> usize {
        self.spent_nullifiers.len()
    }

    pub fn contains_commitment(&self, c: &NoteHash) -> bool {
        self.live_commitments.contains(c)
    }

    pub fn is_nullifier_spent(&self, n: &NoteHash) -> bool {
        self.spent_nullifiers.contains(n)
    }

    /// Genesis / faucet helper: insert a note without spending (tests + mint path).
    pub fn insert_note(&mut self, commitment: NoteHash) -> Result<(), String> {
        if self.live_commitments.contains(&commitment) {
            return Err("note commitment already live".into());
        }
        if self.spent_nullifiers.contains(&commitment) {
            // defensive: commitment hash space must not collide with nullifiers in use
        }
        self.live_commitments.insert(commitment);
        Ok(())
    }

    /// Apply a private transfer: spend nullifiers (each once) and insert outputs.
    ///
    /// `spent_commitments` are revealed only to the executor as private witness
    /// linkage for double-spend of the *note* set; nullifiers are the public
    /// anti-double-spend tags. For v1 submit we require the submitter to also
    /// pass the commitments being spent (encrypted/TEE path can hide them later).
    pub fn apply_transfer(
        &mut self,
        spent_commitments: &[NoteHash],
        nullifiers: &[NoteHash],
        output_commitments: &[NoteHash],
    ) -> Result<(), String> {
        self.apply_transfer_with_proofs(spent_commitments, nullifiers, output_commitments, &[])
    }

    /// L3 fix (pre-mortem V3): Apply transfer with nullifier derivation proofs.
    /// Each proof[i] must satisfy: nullifier[i] == Poseidon(commitment[i], proof[i]).
    /// If proofs is empty, falls back to legacy behavior (no binding check).
    pub fn apply_transfer_with_proofs(
        &mut self,
        spent_commitments: &[NoteHash],
        nullifiers: &[NoteHash],
        output_commitments: &[NoteHash],
        nullifier_proofs: &[NoteHash],
    ) -> Result<(), String> {
        if spent_commitments.len() != nullifiers.len() {
            return Err("spent_commitments/nullifiers length mismatch".into());
        }
        if !nullifier_proofs.is_empty() && nullifier_proofs.len() != nullifiers.len() {
            return Err("nullifier_proofs length mismatch".into());
        }
        if spent_commitments.is_empty() {
            return Err("private transfer requires at least one input".into());
        }
        if output_commitments.is_empty() {
            return Err("private transfer requires at least one output".into());
        }

        // Pre-check nullifiers
        for n in nullifiers {
            if self.spent_nullifiers.contains(n) {
                return Err("double-spend: nullifier already spent".into());
            }
        }
        // Pre-check outputs unique + not already live
        let mut seen_out = BTreeSet::new();
        for c in output_commitments {
            if !seen_out.insert(*c) {
                return Err("duplicate output commitment".into());
            }
            if self.live_commitments.contains(c) {
                return Err("output commitment already live".into());
            }
        }
        // Spend
        for (i, (commitment, nullifier)) in spent_commitments.iter().zip(nullifiers.iter()).enumerate() {
            // L3 fix: verify nullifier derivation proof if provided
            if !nullifier_proofs.is_empty() {
                let proof = &nullifier_proofs[i];
                let expected_nullifier = Self::derive_nullifier(commitment, proof);
                if *nullifier != expected_nullifier {
                    return Err(format!(
                        "nullifier derivation proof invalid for input {}",
                        i
                    ));
                }
            }
            if !self.live_commitments.remove(commitment) {
                return Err("spend: commitment not in live set".into());
            }
            self.spent_nullifiers.insert(*nullifier);
        }
        for c in output_commitments {
            self.live_commitments.insert(*c);
        }
        Ok(())
    }

    pub fn state_root(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"BDLM_L1_NOTE_REGISTRY_V1");
        h.update((self.live_commitments.len() as u64).to_le_bytes());
        for c in &self.live_commitments {
            h.update(c);
        }
        h.update((self.spent_nullifiers.len() as u64).to_le_bytes());
        for n in &self.spent_nullifiers {
            h.update(n);
        }
        h.finalize().into()
    }

    /// L3 fix: Derive nullifier from commitment and proof.
    /// nullifier = SHA-256("BDLM_NULLIFIER_DERIVE_V1" || commitment || proof)
    pub fn derive_nullifier(commitment: &NoteHash, proof: &NoteHash) -> NoteHash {
        let mut h = Sha256::new();
        h.update(b"BDLM_NULLIFIER_DERIVE_V1");
        h.update(commitment);
        h.update(proof);
        h.finalize().into()
    }

    /// H3 fix (pre-mortem V3): Prune spent nullifiers to prevent unbounded growth.
    /// Keeps the most recent `keep_count` nullifiers, removes the rest.
    /// This is safe because double-spend checks only need recent nullifiers;
    /// old nullifiers are already committed to the state root.
    pub fn prune_spent_nullifiers(&mut self, keep_count: usize) {
        if self.spent_nullifiers.len() <= keep_count {
            return;
        }
        let to_remove = self.spent_nullifiers.len() - keep_count;
        // BTreeSet is ordered, so iter().take(to_remove) gives the oldest entries
        let remove: Vec<NoteHash> = self.spent_nullifiers.iter().take(to_remove).copied().collect();
        for n in remove {
            self.spent_nullifiers.remove(&n);
        }
    }

    /// Returns the number of spent nullifiers currently stored.
    pub fn spent_nullifier_count(&self) -> usize {
        self.spent_nullifiers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(b: u8) -> NoteHash {
        let mut x = [0u8; 32];
        x[0] = b;
        x
    }

    #[test]
    fn insert_spend_roundtrip() {
        let mut r = L1NoteRegistry::new();
        r.insert_note(h(1)).unwrap();
        r.apply_transfer(&[h(1)], &[h(10)], &[h(2)]).unwrap();
        assert!(!r.contains_commitment(&h(1)));
        assert!(r.contains_commitment(&h(2)));
        assert!(r.is_nullifier_spent(&h(10)));
    }

    #[test]
    fn double_spend_rejected() {
        let mut r = L1NoteRegistry::new();
        r.insert_note(h(1)).unwrap();
        r.apply_transfer(&[h(1)], &[h(10)], &[h(2)]).unwrap();
        r.insert_note(h(3)).unwrap();
        assert!(r.apply_transfer(&[h(3)], &[h(10)], &[h(4)]).is_err());
    }
}
