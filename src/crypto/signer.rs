use crate::core::address::Address;
use crate::crypto::primitives::{BlsKeypair, CryptoError, KeyPair, PqKeyPair};

pub trait ConsensusSigner: Send + Sync {
    fn public_key_bytes(&self) -> [u8; 32];
    fn address(&self) -> Address {
        Address::from(self.public_key_bytes())
    }
    fn sign_block(&self, block_hash: &[u8; 32]) -> Result<Vec<u8>, CryptoError>;
    fn sign_prevote(&self, _msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.sign_block(&crate::core::hash::calculate_hash_bytes(_msg))
    }
    fn sign_precommit(&self, _msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.sign_block(&crate::core::hash::calculate_hash_bytes(_msg))
    }
    fn bls_sign(&self, _msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Err(CryptoError::Signing(
            "BLS signing not supported by this backend".to_string(),
        ))
    }
    fn pq_sign(&self, _msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Err(CryptoError::Signing(
            "PQ signing not supported by this backend".to_string(),
        ))
    }
    fn backend_name(&self) -> &'static str;
}

pub struct KeyPairSigner {
    keypair: KeyPair,
    bls_key: Option<BlsKeypair>,
    pq_key: Option<PqKeyPair>,
}

impl KeyPairSigner {
    pub fn new(keypair: KeyPair) -> Self {
        Self {
            keypair,
            bls_key: None,
            pq_key: None,
        }
    }

    pub fn with_bls(mut self, bls: BlsKeypair) -> Self {
        self.bls_key = Some(bls);
        self
    }

    pub fn with_pq(mut self, pq: PqKeyPair) -> Self {
        self.pq_key = Some(pq);
        self
    }
}

impl ConsensusSigner for KeyPairSigner {
    fn public_key_bytes(&self) -> [u8; 32] {
        self.keypair.public_key_bytes()
    }

    fn sign_block(&self, block_hash: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        Ok(self.keypair.sign(block_hash).to_vec())
    }

    fn bls_sign(&self, msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let bls = self
            .bls_key
            .as_ref()
            .ok_or_else(|| CryptoError::Signing("No BLS key available".to_string()))?;
        Ok(crate::chain::finality::sign_bls(&bls.secret_key, msg))
    }

    fn pq_sign(&self, msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let pq = self
            .pq_key
            .as_ref()
            .ok_or_else(|| CryptoError::Signing("No PQ key available".to_string()))?;
        pq.sign(msg)
    }

    fn backend_name(&self) -> &'static str {
        "local"
    }
}
