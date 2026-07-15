# Mainnet Genesis Ceremony — Prosedür Belgesi

**Hazırlayan:** ARENA5  
**Tarih:** 2026-07-15  
**Sürüm:** v1.0 (Template — gerçek tören öncesi doldurulacak)  
**Durum:** Taslak — Kullanıcı onayı ile finalize edilecek

> **⚠️ Bu belge bir şablondur.** Gerçek anahtarlar, adresler ve
> multiaddr'lar tören sırasında doldurulacaktır. Bu belgenin varlığı,
> törenin yapıldığı anlamına gelmez.

---

## 1. Tören Tanımı

Budlum Mainnet Genesis Ceremony, placeholder konfigürasyondan gerçek
production konfigürasyona geçişin **şeffaf, doğrulanabilir ve geri
dönülemez** şekilde gerçekleştirildiği törendir.

**Katılımcılar:**
- Kullanıcı (key holder / ceremony master)
- ARENA5 (dokümantasyon, doğrulama, hash freeze)
- Validator operatörleri (N-of-M)

**Güvenlik kuralları:**
- Air-gap: Anahtar üretimi internet bağlantısı olmayan makinede
- HSM: BLS/PQ anahtarları HSM içinde üretilir, dışarı çıkmaz
- Witness: En az 2 bağımsız gözlemci (opsiyonel)

## 2. Önkoşullar

| # | Koşul | Durum |
|---|-------|-------|
| 1 | `cargo test --lib` tüm testler yeşil | ⏳ Tören öncesi kontrol |
| 2 | `cargo clippy -- -D warnings` temiz | ⏳ Tören öncesi kontrol |
| 3 | `cargo fmt --check` temiz | ⏳ Tören öncesi kontrol |
| 4 | VerifyMerkle M5 raporu okundu | ✅ `M5_VERIFYMERKLE_RAPOR_ARENA5.md` |
| 5 | ADIM7 planı okundu | ✅ `ADIM7_CEREMONY_PLAN.md` |
| 6 | Snapshot round-trip test geçti | ⏳ `02dae79` doğrulama |
| 7 | HSM donanım hazır (veya Ed25519-only kararı) | ⏳ Kullanıcı kararı |

## 3. Adım 1: Validator Key Üretimi

### 3.1 Ed25519 Anahtarları (Her Validator İçin)

```bash
# Air-gap makinede
budlum-core keygen --type ed25519 --output validator_N_ed25519.key
# Pubkey: validator_N_ed25519.pub
```

| Validator | Ed25519 Pubkey | Coğrafi Bölge |
|-----------|----------------|---------------|
| V1 | `___DOLDUR___` | EU |
| V2 | `___DOLDUR___` | US-East |
| V3 | `___DOLDUR___` | US-West |
| V4 | `___DOLDUR___` | AS-Southeast |

### 3.2 BLS12-381 Anahtarları (HSM İçinde)

```bash
# HSM üzerinde (PKCS#11)
# Key label: BUD_BLS_KEY
# Mechanism: vendor-specific (örn: 0x80000001)
```

| Validator | BLS Pubkey (G2) | HSM Slot |
|-----------|-----------------|----------|
| V1 | `___DOLDUR___` | `___DOLDUR___` |
| V2 | `___DOLDUR___` | `___DOLDUR___` |
| V3 | `___DOLDUR___` | `___DOLDUR___` |
| V4 | `___DOLDUR___` | `___DOLDUR___` |

### 3.3 Dilithium5 (PQ) Anahtarları (HSM İçinde)

```bash
# HSM üzerinde (PKCS#11)
# Key label: BUD_PQ_KEY
# Mechanism: vendor-specific
```

| Validator | PQ Pubkey | HSM Slot |
|-----------|-----------|----------|
| V1 | `___DOLDUR___` | `___DOLDUR___` |
| V2 | `___DOLDUR___` | `___DOLDUR___` |

**Not:** HSM donanımı yoksa, BLS/PQ anahtarları **üretilmez**.
Mainnet PoA/Ed25519-only ile başlar, BLS/PQ post-launch eklenir.

## 4. Adım 2: Treasury Allocation

| Havuz | Adres | Miktar (BUD) | Vesting |
|-------|-------|-------------|---------|
| Community | `___DOLDUR___` | 10,000,000,000,000 | Yok |
| Liquidity | `___DOLDUR___` | 10,000,000,000,000 | Yok |
| Ecosystem | `___DOLDUR___` | 20,000,000,000,000 | Yok |
| Team | `___DOLDUR___` | 20,000,000,000,000 | Cliff: 52560 epoch (~1 yıl), Vesting: 210240 epoch (~4 yıl) |
| Burn Reserve | `___DOLDUR___` | 40,000,000,000,000 | Yok (sabit yakım) |

**Toplam arz:** 100,000,000,000,000 BUD (100 trilyon)

## 5. Adım 3: Genesis JSON Finalize

```json
{
  "chain_id": 1,
  "allocations": [
    {"address": "___COMMUNITY_ADDR___", "amount": 10000000000000},
    {"address": "___LIQUIDITY_ADDR___", "amount": 10000000000000},
    {"address": "___ECOSYSTEM_ADDR___", "amount": 20000000000000},
    {"address": "___TEAM_ADDR___", "amount": 20000000000000, "vesting": {"cliff": 52560, "period": 210240}},
    {"address": "___BURN_RESERVE_ADDR___", "amount": 40000000000000}
  ],
  "validators": [
    {"pubkey": "___V1_ED25519___", "bls_pubkey": "___V1_BLS___", "pq_pubkey": "___V1_PQ___", "stake": 1000000, "commission": 500},
    {"pubkey": "___V2_ED25519___", "bls_pubkey": "___V2_BLS___", "pq_pubkey": "___V2_PQ___", "stake": 1000000, "commission": 500},
    {"pubkey": "___V3_ED25519___", "bls_pubkey": "___V3_BLS___", "pq_pubkey": "___V3_PQ___", "stake": 1000000, "commission": 500},
    {"pubkey": "___V4_ED25519___", "bls_pubkey": "___V4_BLS___", "pq_pubkey": "___V4_PQ___", "stake": 1000000, "commission": 500}
  ],
  "block_reward": 50,
  "base_fee": 10,
  "gas_schedule": {
    "base_fee": 10,
    "gas_per_byte": 2,
    "gas_per_signature": 1000,
    "transfer_gas": 21000,
    "stake_gas": 45000,
    "vote_gas": 35000,
    "contract_call_gas": 50000
  },
  "timestamp": ___GENESIS_TIMESTAMP___,
  "bud_tokenomics": {
    "community": 10000000000000,
    "liquidity": 10000000000000,
    "ecosystem": 20000000000000,
    "team": 20000000000000,
    "burn_reserve": 40000000000000,
    "epochs_per_year": 52560,
    "annual_burn_ratio_fixed": 100000,
    "team_cliff_epochs": 52560,
    "team_vesting_epochs": 210240,
    "tx_fee_burn_ratio_fixed": 10000,
    "block_reward": 50,
    "validator_annual_yield_ratio_fixed": 50000,
    "slot_duration_secs": 10,
    "epoch_length_slots": 32
  }
}
```

## 6. Adım 4: Genesis Hash Freeze

```bash
# Hash hesapla
sha256sum config/mainnet-genesis.json
# Çıktı: ___GENESIS_HASH___

# PRODUCTION_RUNBOOK.md §8.2'ye yaz
echo "Genesis hash: ___GENESIS_HASH___" >> docs/operations/PRODUCTION_RUNBOOK.md

# mainnet.toml comment güncelle
# genesis_hash = "___GENESIS_HASH___"
```

**⚠️ Bu hash bir kez yazıldıktan sonra DEĞİŞTİRİLEMEZ.**
Değişiklik = farklı bir zincir = tüm allocation'lar geçersiz.

## 7. Adım 5: Bootnode Kurulumu

| # | Sunucu | IP | Multiaddr | PeerID |
|---|--------|----|-----------|---------|
| BN1 | `___DOLDUR___` | `___DOLDUR___` | `/ip4/___/tcp/4001/p2p/___` | `12D3KooW___` |
| BN2 | `___DOLDUR___` | `___DOLDUR___` | `/ip4/___/tcp/4001/p2p/___` | `12D3KooW___` |
| BN3 | `___DOLDUR___` | `___DOLDUR___` | `/ip4/___/tcp/4001/p2p/___` | `12D3KooW___` |

**DNS Seeds:**
| # | Domain | TXT Record |
|---|--------|------------|
| DS1 | `_dnsaddr.mainnet-seed-1.budlum.___` | `dnsaddr=/ip4/___/tcp/4001/p2p/___` |
| DS2 | `_dnsaddr.mainnet-seed-2.budlum.___` | `dnsaddr=/ip4/___/tcp/4001/p2p/___` |

## 8. Adım 6: İlk Blok (T-0)

1. Tüm validator node'ları başlatılır:
   ```bash
   budlum-core --config config/mainnet.toml --network mainnet
   ```

2. İlk BLS prevote/precommit:
   - 4/4 validator prevote → quorum
   - 4/4 validator precommit → quorum
   - FinalityCert üretilir

3. İlk blok (block 1):
   - `bud_blockNumber` → 1
   - `bud_finalizedHeight` → 1
   - Prometheus: `budlum_blocks_produced_total` → 1

4. Sağlık kontrolü:
   ```bash
   curl -X POST http://localhost:8545 \
     -d '{"jsonrpc":"2.0","method":"bud_health","params":[],"id":1}'
   ```

## 9. Bilinçli Borçlar (Post-Launch)

| # | Borç | Aktivasyon Koşulu |
|---|------|-------------------|
| M5 | VerifyMerkle Z-B gate | 64-depth STARK yeşil → soft-fork PR |
| M6 | HSM vendor-native BLS/PQ | Donanım temin → config aktivasyonu |
| M7 | External audit | Bug bounty launch → firma seçimi |
| M10 | SocialFi/Hub/Marketplace | Küçük PR'larla post-launch |

## 10. İmza ve Onay

| Rol | İsim | İmza | Tarih |
|-----|------|------|-------|
| Ceremony Master | `___DOLDUR___` | `___DOLDUR___` | `___DOLDUR___` |
| Validator V1 Operatörü | `___DOLDUR___` | `___DOLDUR___` | `___DOLDUR___` |
| Validator V2 Operatörü | `___DOLDUR___` | `___DOLDUR___` | `___DOLDUR___` |
| Validator V3 Operatörü | `___DOLDUR___` | `___DOLDUR___` | `___DOLDUR___` |
| Validator V4 Operatörü | `___DOLDUR___` | `___DOLDUR___` | `___DOLDUR___` |
| ARENA5 (Doğrulama) | ARENA5 | `commit SHA` | `___DOLDUR___` |

---

**Force-push YASAK. Workflow push YASAK.**  
Co-authored-by: ARENA5
