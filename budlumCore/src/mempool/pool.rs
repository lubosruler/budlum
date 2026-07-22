use crate::core::address::Address;
use crate::core::transaction::Transaction;
use std::collections::{BTreeMap, BTreeSet, HashMap};

// ADIM-1 (ARENA2, 2026-07-21): consensus determinizmi — aynı fee'deki
// işlemler HashSet iteration sırasıyla (process-random) geliyordu;
// `get_sorted_transactions` → `collect_block_transactions` → blok gövdesi
// sırası node'dan node'a değişebilirdi (aynı-fee tie durumunda farklı blok
// hash'i / potansiyel split). Tie-break artık canonik: `BTreeSet<String>`
// ile tx.hash lexikografik düzeni — ücret DESC, hash ASC. Bu kuralı değiştirmek
// consensus davranışını değiştirir: dokümante ve testli (`test_same_fee_canonical_order_by_hash`).

#[derive(Debug, Clone)]
pub struct MempoolConfig {
    pub max_size: usize,

    pub max_per_sender: usize,

    pub min_fee: u64,

    pub tx_ttl_secs: u64,

    pub rbf_bump_percent: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        MempoolConfig {
            max_size: 20000,
            max_per_sender: 100,
            min_fee: 1,
            tx_ttl_secs: 3600,
            rbf_bump_percent: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MempoolError {
    PoolFull,
    DuplicateTransaction,
    FeeTooLow,
    SenderLimitReached,
    InvalidNonce,
    TransactionExpired,
    RbfFeeTooLow,
    InvalidTransaction(String),
}

#[derive(Debug, Clone)]
struct PendingTx {
    tx: Transaction,
    added_at: u128,
}

#[derive(Clone)]
pub struct Mempool {
    config: MempoolConfig,

    transactions: HashMap<String, PendingTx>,

    by_sender: HashMap<Address, BTreeMap<u64, String>>,

    by_fee: BTreeMap<u64, BTreeSet<String>>,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Mempool {
            config,
            transactions: HashMap::new(),
            by_sender: HashMap::new(),
            by_fee: BTreeMap::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), MempoolError> {
        if self.transactions.contains_key(&tx.hash) {
            return Err(MempoolError::DuplicateTransaction);
        }

        if tx.fee < self.config.min_fee {
            return Err(MempoolError::FeeTooLow);
        }

        if self.transactions.len() >= self.config.max_size && !self.evict_lowest_fee(&tx) {
            return Err(MempoolError::PoolFull);
        }

        let sender_count = self.by_sender.get(&tx.from).map_or(0, |v| v.len());

        if let Some(existing_hash) = self.find_tx_by_sender_nonce(&tx.from, tx.nonce) {
            let existing = self.transactions.get(&existing_hash).unwrap();
            // ADIM-1: RBF bump her zaman POZİTİF olmalı. Tamsayı bölmesiyle
            // küçük fee'lerde bump 0'a yuvarlanıyordu (fee=1, %10 → bump 0)
            // → aynı fee ile limitsiz replace-churn (ucuz DoS vektörü).
            // Artık: bump = max(1, ceil(fee * pct / 100)); replace fee > eski
            // fee olmak ZORUNDA. Overflow'a karşı u128 ara hesaplama.
            let bump =
                ((existing.tx.fee as u128 * self.config.rbf_bump_percent as u128) + 99) / 100;
            let min_new_fee = existing
                .tx
                .fee
                .saturating_add(u64::try_from(bump.max(1)).unwrap_or(u64::MAX));
            if tx.fee < min_new_fee {
                return Err(MempoolError::RbfFeeTooLow);
            }

            self.remove_transaction(&existing_hash);
        } else {
            if sender_count >= self.config.max_per_sender {
                return Err(MempoolError::SenderLimitReached);
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        self.by_sender
            .entry(tx.from)
            .or_default()
            .insert(tx.nonce, tx.hash.clone());
        self.by_fee
            .entry(tx.fee)
            .or_default()
            .insert(tx.hash.clone());

        self.transactions
            .insert(tx.hash.clone(), PendingTx { tx, added_at: now });

        Ok(())
    }

    pub fn remove_transaction(&mut self, hash: &str) -> Option<Transaction> {
        if let Some(pending) = self.transactions.remove(hash) {
            if let Some(sender_txs) = self.by_sender.get_mut(&pending.tx.from) {
                sender_txs.remove(&pending.tx.nonce);
                if sender_txs.is_empty() {
                    self.by_sender.remove(&pending.tx.from);
                }
            }

            if let Some(fee_txs) = self.by_fee.get_mut(&pending.tx.fee) {
                fee_txs.remove(hash);
                if fee_txs.is_empty() {
                    self.by_fee.remove(&pending.tx.fee);
                }
            }
            return Some(pending.tx);
        }
        None
    }

    pub fn get_sorted_transactions(&self, limit: usize) -> Vec<Transaction> {
        let mut result = Vec::with_capacity(limit);

        for (_, hashes) in self.by_fee.iter().rev() {
            for hash in hashes {
                if result.len() >= limit {
                    return result;
                }
                if let Some(pending) = self.transactions.get(hash) {
                    result.push(pending.tx.clone());
                }
            }
        }
        result
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let ttl_ms = self.config.tx_ttl_secs as u128 * 1000;
        let expired: Vec<String> = self
            .transactions
            .iter()
            .filter(|(_, p)| now - p.added_at > ttl_ms)
            .map(|(h, _)| h.clone())
            .collect();

        let count = expired.len();
        for hash in expired {
            self.remove_transaction(&hash);
        }
        count
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn get(&self, hash: &str) -> Option<&Transaction> {
        self.transactions.get(hash).map(|p| &p.tx)
    }

    pub fn sender_transactions(&self, sender: &Address) -> Vec<Transaction> {
        self.by_sender
            .get(sender)
            .map(|nonces| {
                nonces
                    .values()
                    .filter_map(|hash| {
                        self.transactions
                            .get(hash)
                            .map(|pending| pending.tx.clone())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn drain(&mut self) -> Vec<Transaction> {
        let txs: Vec<Transaction> = self.transactions.values().map(|p| p.tx.clone()).collect();
        self.transactions.clear();
        self.by_sender.clear();
        self.by_fee.clear();
        txs
    }

    fn find_tx_by_sender_nonce(&self, sender: &Address, nonce: u64) -> Option<String> {
        self.by_sender
            .get(sender)
            .and_then(|nonces| nonces.get(&nonce).cloned())
    }

    fn evict_lowest_fee(&mut self, new_tx: &Transaction) -> bool {
        if let Some((&lowest_fee, hashes)) = self.by_fee.iter().next() {
            if new_tx.fee > lowest_fee {
                if let Some(hash) = hashes.iter().next().cloned() {
                    self.remove_transaction(&hash);
                    return true;
                }
            }
        }
        false
    }

    pub fn set_min_fee(&mut self, min_fee: u64) {
        self.config.min_fee = min_fee;
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(MempoolConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tx(from_hex: &str, nonce: u64, fee: u64) -> Transaction {
        let from = Address::from_hex(from_hex).unwrap();
        let mut tx = Transaction::new(from, Address::zero(), 100, vec![]);
        tx.nonce = nonce;
        tx.fee = fee;
        tx.hash = format!("tx_{from_hex}_{nonce}");
        tx
    }

    #[test]
    fn test_add_and_get() {
        let mut pool = Mempool::default();
        let tx = create_test_tx(&"01".repeat(32), 0, 10);
        assert!(pool.add_transaction(tx.clone()).is_ok());
        assert_eq!(pool.len(), 1);
        assert!(pool.get(&tx.hash).is_some());
    }

    #[test]
    fn test_duplicate_rejection() {
        let mut pool = Mempool::default();
        let tx = create_test_tx(&"01".repeat(32), 0, 10);
        pool.add_transaction(tx.clone()).unwrap();
        assert_eq!(
            pool.add_transaction(tx),
            Err(MempoolError::DuplicateTransaction)
        );
    }

    #[test]
    fn test_fee_too_low() {
        let mut pool = Mempool::default();
        let tx = create_test_tx(&"01".repeat(32), 0, 0);
        assert_eq!(pool.add_transaction(tx), Err(MempoolError::FeeTooLow));
    }

    #[test]
    fn test_sender_limit() {
        let config = MempoolConfig {
            max_per_sender: 2,
            ..Default::default()
        };
        let mut pool = Mempool::new(config);

        let alice_hex = "01".repeat(32);
        pool.add_transaction(create_test_tx(&alice_hex, 0, 10))
            .unwrap();
        pool.add_transaction(create_test_tx(&alice_hex, 1, 10))
            .unwrap();
        assert_eq!(
            pool.add_transaction(create_test_tx(&alice_hex, 2, 10)),
            Err(MempoolError::SenderLimitReached)
        );
    }

    #[test]
    fn test_sorted_by_fee() {
        let mut pool = Mempool::default();
        pool.add_transaction(create_test_tx(&"01".repeat(32), 0, 5))
            .unwrap();
        pool.add_transaction(create_test_tx(&"02".repeat(32), 0, 20))
            .unwrap();
        pool.add_transaction(create_test_tx(&"03".repeat(32), 0, 10))
            .unwrap();

        let sorted = pool.get_sorted_transactions(10);
        assert_eq!(sorted[0].fee, 20);
        assert_eq!(sorted[1].fee, 10);
        assert_eq!(sorted[2].fee, 5);
    }

    #[test]
    fn test_rbf() {
        let mut pool = Mempool::default();
        let alice_hex = "01".repeat(32);
        let tx1 = create_test_tx(&alice_hex, 0, 10);
        pool.add_transaction(tx1).unwrap();

        let mut tx2 = create_test_tx(&alice_hex, 0, 15);
        tx2.hash = "tx_alice_0_v2".to_string();
        assert!(pool.add_transaction(tx2).is_ok());
        assert_eq!(pool.len(), 1);
    }

    /// ADIM-1: aynı fee tie-break canonik (tx.hash ASC). Farklı ekleme
    /// sırası sonucu DEĞİŞTİRMEMELİ — eski HashSet yolu process-random
    /// iteration ile bu testin iki havuzunda fark verirdi (flaky/üretimde
    /// nondeterministik blok gövdesi sırası).
    #[test]
    fn test_same_fee_canonical_order_by_hash() {
        let mut tx_z = create_test_tx(&"09".repeat(32), 0, 10);
        tx_z.hash = "zz_canonical".to_string();
        let mut tx_a = create_test_tx(&"08".repeat(32), 0, 10);
        tx_a.hash = "aa_canonical".to_string();
        let mut tx_m = create_test_tx(&"07".repeat(32), 0, 10);
        tx_m.hash = "mm_canonical".to_string();

        let mut pool1 = Mempool::default();
        pool1.add_transaction(tx_z.clone()).unwrap();
        pool1.add_transaction(tx_m.clone()).unwrap();
        pool1.add_transaction(tx_a.clone()).unwrap();
        let order1: Vec<String> = pool1
            .get_sorted_transactions(10)
            .iter()
            .map(|t| t.hash.clone())
            .collect();
        assert_eq!(order1, vec!["aa_canonical", "mm_canonical", "zz_canonical"]);

        // Farklı ekleme sırası, aynı canonik çıktı.
        let mut pool2 = Mempool::default();
        pool2.add_transaction(tx_a).unwrap();
        pool2.add_transaction(tx_z).unwrap();
        pool2.add_transaction(tx_m).unwrap();
        let order2: Vec<String> = pool2
            .get_sorted_transactions(10)
            .iter()
            .map(|t| t.hash.clone())
            .collect();
        assert_eq!(order1, order2);
    }

    /// ADIM-1: RBF replace her zaman kat'i pozitif bump ister.
    /// Eski yol: fee=1, %10 → bump=0 → aynı fee ile replace (churn vektörü).
    #[test]
    fn test_rbf_requires_strict_positive_bump() {
        let mut pool = Mempool::default();
        let alice_hex = "01".repeat(32);
        let mut tx1 = create_test_tx(&alice_hex, 0, 1);
        tx1.hash = "tx_v1".to_string();
        pool.add_transaction(tx1).unwrap();

        // Aynı fee ile replace artık RED (önceden kabul ediliyordu).
        let mut tx2 = create_test_tx(&alice_hex, 0, 1);
        tx2.hash = "tx_v2_same_fee".to_string();
        assert_eq!(pool.add_transaction(tx2), Err(MempoolError::RbfFeeTooLow));

        // fee=2 (%10 ⇒ ceil(0.2)=1 ⇒ min 2) KABUL.
        let mut tx3 = create_test_tx(&alice_hex, 0, 2);
        tx3.hash = "tx_v3_bumped".to_string();
        assert!(pool.add_transaction(tx3).is_ok());
        assert_eq!(pool.len(), 1);
        assert!(pool.get("tx_v3_bumped").is_some());

        // fee=100 (%10 ⇒ bump=10 ⇒ min 110): 109 RED, 110 KABUL.
        let mut tx4 = create_test_tx(&alice_hex, 1, 100);
        tx4.hash = "tx_big".to_string();
        pool.add_transaction(tx4).unwrap();
        let mut tx5 = create_test_tx(&alice_hex, 1, 109);
        tx5.hash = "tx_big_v2".to_string();
        assert_eq!(pool.add_transaction(tx5), Err(MempoolError::RbfFeeTooLow));
        let mut tx6 = create_test_tx(&alice_hex, 1, 110);
        tx6.hash = "tx_big_v3".to_string();
        assert!(pool.add_transaction(tx6).is_ok());
    }

    #[test]
    fn test_cleanup_expired() {
        let config = MempoolConfig {
            tx_ttl_secs: 1,
            ..Default::default()
        };
        let mut pool = Mempool::new(config);

        let tx = create_test_tx(&"01".repeat(32), 0, 10);
        pool.add_transaction(tx).unwrap();
        assert_eq!(pool.len(), 1);

        std::thread::sleep(std::time::Duration::from_secs(2));

        let removed = pool.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }
}
