# State Pruning & Node Tipleri Spec

> **Yazar:** ARENA1, 2026-07-20. **Durum:** Draft.

## 1. Node Tipleri

| Tip | State | Block History | Use Case |
|-----|-------|---------------|----------|
| **Full Node** | Current + recent (N blocks) | Full | Validator, relayer |
| **Archive Node** | All historical state | Full | RPC provider, indexer |
| **Light Node** | None (SPV) | Headers only | Mobile, browser |

## 2. Pruning Stratejisi

### Full Node Pruning (mevcut)
- `PruningManager` (`src/chain/snapshot.rs:108`): `min_blocks_to_keep` params
- Snapshot interval + retention (config-driven)
- NftBurn → CID pruning (F1 fix, `collect_nft_burn_cids`)

### Archive Node
- Pruning **disabled** (`pruning_manager: None`)
- Full historical state query
- Storage cost yüksek → incentive gerek (donation/governance)

### Light Node (SPV)
- Yalnızca block headers + finality proofs
- State query → RPC'ye güven (trust assumption)
- Gelecek: ZK proof ile trustless light client

## 3. Pruning Kuralları

1. **Finalized blocks only:** Pruning yalnızca finalized_height altında
2. **Snapshot retention:** En az 2 snapshot korunur (rollback için)
3. **Bridge state:** Bridge transfer history kalıcı (provenance)
4. **Slashing history:** Kalıcı (audit trail)

## 4. Yapılandırma

```toml
[storage]
pruning_mode = "full"  # full | archive | light
min_blocks_to_keep = 10000
snapshot_interval = 1000
snapshot_retention = 3
```

## 5. Gap Analizi

- **Archive node economics:** Ödüllendirme modeli yok
- **Light node SPV:** Header-only mode implemente değil
- **Historical state query:** Archive node RPC endpoint yok
- **Pruning test:** `PruningManager` test'leri var ama edge-case (rollback after prune) eksik

---

*Co-authored-by: ARENA1 <arena1@budlum.ai>*
