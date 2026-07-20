# Domain Fork-Choice & Lifecycle Spec

> **Yazar:** ARENA1, 2026-07-20. **Durum:** Draft.

## 1. Fork-Choice Kuralları (Domain-Özel)

### PoW Domain (longest-chain)
- **Kural:** En yüksek cumulative work = canonical chain
- **Reorg:** `MAX_REORG_DEPTH = 100` sınırı
- **Finality:** Probabilistic (k-deep confirmations)
- **Kod:** `src/consensus/pow.rs`, `Blockchain::try_reorg()`

### PoS Domain (instant finality)
- **Kural:** FinalityCert ile finalize → immutable
- **Reorg:** Finalized blok sonrası reorg YASAK
- **Finality:** BLS12-381 quorum (2/3+ stake)
- **Kod:** `src/chain/finality.rs`, `FinalityAggregator`

### BFT Domain (instant finality)
- **Kural:** Aggregated signature → instant finality
- **Leader:** Hash-mix deterministic leader selection (Phase 0.338)
- **Finality:**QC (quorum certificate) → finalize
- **Kod:** `src/consensus/qc.rs`, `QcBlob`

### PoA Domain (authority quorum)
- **Kural:** Authority set'in 2/3+ imzası = valid block
- **Leader:** Rotating (hash-mix, pure round-robin DEĞİL)
- **Finality:** Authority signatures → instant
- **İzolasyon:** PoA → permissionless registry'ye sızma YASAK (CLAUDE.md §2)

## 2. Cross-Domain Settlement

- Tüm domain'ler `GlobalBlockHeader` üzerinden settle edilir
- Her domain'in finality proof'u `DomainFinalityAdapter` ile doğrulanır
- Cross-domain mesajlar `CrossDomainMessage` + nonce/replay protection

## 3. Domain Lifecycle

### Start (genesis)
- `bootstrap_domains` config → domain_registry.register()
- Domain `DomainStatus::Active` olarak başlar

### Stop (governance)
- **Yok** — domain stop mekanizması henüz tasarlanmadı
- **Öneri:** Governance proposal → domain status → `Paused`
- **Risk:** Aktif deal'ları olan domain'i durdurmak → fon kilidi

### Upgrade (governance)
- Domain parametreleri (`StorageDomainParams`) governance ile değiştirilebilir
- **Risk:** Parametre değişimi aktif deal'ları etkiler

### Fork (domain-internal)
- Her domain kendi internal fork-choice'unu kullanır
- Budlum L1 bunları settle eder, internal fork'a karışmaz

## 4. Gap Analizi

- **Domain pause/stop:** Kod yok — governance proposal tipi gerekiyor
- **Domain upgrade:** Parametre değişimi var ama sürecin tanımı belirsiz
- **Cross-domain finality conflict:** İki domain farklı fork'larda → resolution?
- **PoA domain rotation:** Authority set değişimi governance gerekiyor

---

*Co-authored-by: ARENA1 <arena1@budlum.ai>*
