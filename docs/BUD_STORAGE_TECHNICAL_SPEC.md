# B.U.D. Storage — Teknik Spec (Vision → Implementation)

> **Yazar:** ARENA1 (görev yöneticisi), 2026-07-20.
> **Durum:** Draft (vision → teknik spec dönüşümü).
> **Kaynak Vision:** `budlum-xyz/B.U.D./BUD_Merkeziyetsiz_Depolama_Vizyonu.md` (495 satır, 12 bölüm).

---

## 1. Mevcut Kod Haritası

| Bileşen | Dosya | Durum | Faz |
|---------|-------|-------|-----|
| ContentId (32-byte hash) | `src/storage/content_id.rs` | ✅ | Faz 2 |
| ContentManifest (shard list + owner) | `src/storage/manifest.rs` | ✅ (F01 owner) | Faz 2 |
| StorageDomainParams | `src/domain/storage_params.rs` | ✅ | Faz 1 |
| StorageDeal + DealStatus | `src/domain/storage_deal.rs` | ✅ | Faz 5 |
| RetrievalChallenge/Response/Outcome | `src/domain/storage_deal.rs` | ✅ (interim) | Faz 5 |
| StorageRegistry (permissionless) | `src/domain/storage_deal.rs` | ✅ | Faz 5 |
| StorageEconomicsParams | `src/domain/storage_deal.rs` | ✅ | Faz 5 |
| 9 JSON-RPC uç noktası | `src/rpc/api.rs` + `server.rs` | ✅ | Faz 5 |
| 3-aktör E2E test (9 invariant) | `src/tests/bud_e2e.rs` | ✅ | Faz 5 |
| MerkleTrie (state tree) | `src/storage/merkle_trie.rs` | ✅ | Faz 4 |
| Storage pruning (NftBurn→CID) | `src/chain/blockchain.rs` | ✅ (F1) | Faz 5 |
| **VerifyMerkle 64-depth** | `budzero/bud-proof/` | 🔒 Production-gated | **Faz 3** |
| **Real Proof-of-Storage** | — | ❌ | **Faz 3** |
| **DataAsset/AccessGrant** | `src/pollen/` (P0 tipler) | 🟡 iskelet | Marketplace |
| **BNS .bud** | `src/bns/registry.rs` | ✅ iskelet | Faz 6 |

## 2. Faz Haritası (Vision → Kod)

### Faz 1 — Domain Kaydı (✅ Tamam)
- `ConsensusKind::StorageAttestation(StorageDomainParams)` enum varyantı
- `STORAGE_OPERATOR = RoleId(5)` permissionless rol
- Domain parametreleri: `chunk_size`, `max_committed_chunks`, `challenge_interval`, `min_operator_bond`

### Faz 2 — Content Addressing (✅ Tamam)
- `ContentId = Hash32` (32-byte SHA-256, domain-tagged)
- `ContentManifest` (manifest_id, owner, total_size, shard_count, shards)
- `ShardRef` (shard_id, size, content_id)

### Faz 3 — Proof-of-Storage (🔒 Production-gated)
- **Gerçek PoS** = BudZKVM `VerifyMerkle` 64-depth STARK proof
- Mevcut: `RetrievalChallenge` interim (byte-range hash — operator sadece istenen
  range'i saklayarak geçebilir, **gerçek kanıt DEĞİL**)
- **Engel:** VerifyMerkle Production ISA gate'i kapalı (MR-3)
- **Plan:** Gate açılınca → operator 64-depth Merkle proof sunar → gerçek PoS

### Faz 4 — Block Header Integration (🟡 Kısmi)
- `GlobalBlockHeader.storage_root` — Mevcut değil (vision §8.5)
- `MerkleTrie` state tree var ama `storage_root` header'a bağlı değil
- **Plan:** Block header'a `storage_root` alanı ekle → hash'e dahil

### Faz 5 — Deal/Challenge Ekonomisi (✅ Tamam)
- `StorageDeal` (operator, shard, bond, fee, epoch)
- `RetrievalChallenge` / `Response` / `Outcome`
- `StorageEconomicsParams` (operator_bond, fee_per_epoch)
- Permissionless: `open_deal`, `open_challenge` — whitelist YOK

### Faz 6 — BNS .bud Integration (✅ İskelet)
- `BnsRegistry` — name → address/content resolution
- `.bud` domain kaydı + transfer + renewal + grace-period (F14)

## 3. Gap Analizi (Vision ↔ Kod)

| Vision Özelliği | Kod Durumu | Gap |
|-----------------|------------|-----|
| Content addressing | ✅ ContentId + Manifest | — |
| Sharding (off-chain) | ✅ ShardRef | — |
| Deal marketplace | ✅ StorageDeal + 9 RPC | Pollen marketplace (P0 tipler) ayrı |
| Proof-of-Storage | 🔒 Interim (byte-range) | **VerifyMerkle gate** |
| Slashing (missed challenge) | ✅ finalize_missed_challenge | — |
| Pruning (NftBurn→CID delete) | ✅ F1 fix | — |
| Storage root in header | ❌ | **Faz 4 gap** |
| Retrieval (data delivery) | ❌ | Off-chain, bu spec dışı |
| Encryption (HPKE) | ❌ | Pollen Faz-2 (F02) |
| Multi-replica deals | ✅ deals_for_manifest | — |

## 4. Veri Egemenliği Kuralı (§0.5)

Hiçbir kritik fonksiyon "Budlum ekibinin servisine" bağımlı değildir:
- `open_deal` — permissionless (herkes)
- `open_challenge` — permissionless (anti-spam bond)
- `answer_challenge` — yalnızca deal.operator
- Whitelist/admin/pause/freeze hook'u **YOK** (kod incelemesi + grep kanıtı)

## 5. Sıradaki Adımlar

1. **MR-3 VerifyMerkle** gate açılması (ARENA3 budzero domain)
2. **Faz 4:** `GlobalBlockHeader.storage_root` entegrasyonu
3. **F02:** AccessGrant HPKE hard-enforcement (pollen Faz-2)
4. **B05:** B.U.D. operator churn/grace-period politikası

---

*Co-authored-by: ARENA1 <arena1@budlum.ai>*
