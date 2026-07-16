# BUDLUM & BUDZERO — FORMAL THREAT MODEL & SECURITY SPECIFICATION

**Tarih:** 2026-07-15  
**Sürüm:** v0.3-dev (Controlled Public-Devnet Candidate)  
**Hazırlayan:** Arena AI / ARENA3 (Lubo / Phase 0.378 Paket F)

> **Önemli Uarı:** Bu belge, `DEVIR_RAPORU_YENI.md` §7 Paket F gereğince
> bağımsız dış denetçilerin (`External Security Audit`) incelemesine esas olmak
> üzere hazırlanmış bir **teslim ve tehdit modeli paketidir**. Bu belgenin varlığı,
> harici denetimin tamamlandığı anlamına gelmez; yalnızca sistemin varsayımlarını,
> saldırı yüzeylerini ve mimari sınırlarını dürüstçe tanımlar.

---

## 1. Sistem ve Varlık Tanımı (Assets Identification)

Budlum L1 (`budlum-core`) ve BudZero (`budzero/` BudZKVM + STARK motoru),
çok-konsensüslü (`ConsensusDomain`) bir evrensel mutabakat katmanıdır. Sistem
üzerinde korunan temel varlıklar:

1. **`GlobalBlockHeader` Mutabakat Kaydı:** Çapraz domain transferlerinin ve
   finality sertifikalarının geri dönülemez (durable) kaydı.
2. **Çapraz Domain Köprü Varlıkları (`BridgeState`):** Kilitlenen (`Lock`) varlıkların
   hedef domain'de basılması (`Mint`) ve yakıldıktan (`Burn`) sonra kaynak domain'de
   serbest bırakılması (`Unlock`).
3. **BLS + Dilithium5 (PQ) İmzacı Anahtarları:** Finality koordinatörü ve post-kuantum
   doğrulama anahtarları.
4. **`PermissionlessRegistry` Stake Depoları:** Validator, Verifier, Relayer, Prover
   ve Storage Operator rollerinin teminatları (`bond`).

---

## 2. Kriptografik Varsayımlar (Cryptographic Assumptions)

| Kriptografik Birim | Amaç | Güvenlik Varsayımı & Sınırı |
|--------------------|------|-----------------------------|
| **Ed25519** | Temel hesap kimliği & PoS/PoA blok üretimi | Standart ayrık logaritma zorluğu. Kuantum kırılmasında (`~2030-2035`) L1 blok üretimi BLS/PQ katmanına devredilmelidir. |
| **BLS12-381** | Finality sertifikası agregasyonu | Çift doğrusal eşleme (`bilinear pairing`) zorluğu. Subgroup saldırılarına karşı `verify_bls_sig_rejects_subgroup_attack` denetimleri aktiftir. |
| **Dilithium5 (ML-DSA)** | Post-kuantum finality & QC imza kanıtları | Modül örgüler (`Module-Lattice`) üzerinde kısa vektör bulma zoru (`NIST FIPS 204`). |
| **Poseidon / Poseidon4** | BudZero STARK AIR & B.U.D. `ContentId` | Cebirsel saldırılara ve satır patlamalarına (`trace blowup`) karşı 10 gas sınırlı STARK dostu karma fonsiyonu. |
| **SHA3-256 / Keccak-256** | Blok başlıkları, snapshot ID ve DB adresleme | Klasik çakışma (`collision`) direnci (256-bit). |

---

## 3. Tehdit Aktörleri & Saldırı Vektörleri (Threat Vectors)

### 3.1 Çapraz Domain Köprü Sahtekarlığı (Bridge Forgery & Replay)
- **Tehdit:** Kötü niyetli bir relayer veya domain operatörü, sahte veya henüz kesinleşmemiş (`Pending/Rejected`) bir `DomainCommitment` veya `PoWHeaderChain` kanıtını kullanarak yoktan varlık basmak (`Infinite Mint`) veya çift harcama yapmak (`Replay Attack`).
- **Azaltma / Koruma (Mitigation):**
  - PoW mint işlemleri yalnızca `applied` ve `contiguous` domain zinciri üzerindeki `pow-header-chain-v1` finality sertifikaları üzerinden yapılabilir; legacy `pow-confirmation-depth` kanıtlarına mint yetkisi kapalıdır (`tur13_5_pow_header_finality...`).
  - Çapraz domain mesajlarında deterministik `MessageId` ve her transfer için artan `replay_nonce_root` denetimi mecburidir.

### 3.2 ZKVM Soundness & Sahte Kanıt Kabulü (STARK Forgery)
- **Tehdit:** Kötü niyetli bir prover, geçersiz bir execution trace veya Merkle yolunu sahte bir STARK kanıtı ile sarmalayarak (`ProofClaimRegistry`) L1 mutabakatına kabul ettirmek.
- **Azaltma / Koruma (Mitigation):**
  - ZK finality adaptörü (`ZkFinalityAdapter::verify_finality`), trait üzerinden asla doğrudan kabul etmez (`fail-closed reject`). Yalnızca `verify_finality_with_claim` üzerinden `final_state_root` ve `ProofClaimRegistry` çift taraflı eşleşmesiyle onaylar.
  - `VerifyMerkle` (0x1E) opcode'u, pozitif 64-depth proof tamamen yeşil olana kadar production decode aşamasında fail-closed reddedilir (`experimental gate`).

### 3.3 Düz Metin Anahtar Sızdırması (Key Exfiltration on Disk)
- **Tehdit:** Sunucuya sızan bir saldırıcının disktki konfigürasyon veya anahtar dosyalarından BLS veya PQ Dilithium5 özel anahtarlarını ele geçirmesi.
- **Azaltma / Koruma (Mitigation):**
  - Mainnet konfigürasyonlarında düz metin olarak diske yazılmış BLS (`bls_key`) ve PQ Dilithium5 (`pq_key`) anahtarlarının yüklenmesi `validate_mainnet_disk_policy` tarafından anında reddedilir (`CryptoError::PlaintextDiskKeysForbiddenOnMainnet`). Mainnet üzerinde donanımsal HSM (`PKCS#11 / BLS-PQ HSM mock`) kullanımı zorunludur.

### 3.4 Çapraz Sürüm (Schema Version) / Snapshot Zehirlenmesi
- **Tehdit:** Saldırganın ağa eski şemalı (`v1`) eksik durum zarfları veya bozuk V2 anlık görüntüleri (`StateSnapshotV2`) sürerek düğümleri çökertecek reorg veya mutabakat yarılması (`Split-Brain`) yaratması.
- **Azaltma / Koruma (Mitigation):**
  - `StateSnapshotV2::from_bytes` içerisindeki `ConsensusStateV2` şema kancası, `MIN_SCHEMA_VERSION = 2` altındaki ve `MAX_SCHEMA_VERSION = 3` üstündeki tüm snapshot'ları fail-closed reddeder. Bozuk snapshot dosyaları `quarantine` edilerek `.corrupted` uzantısıyla izole edilir.

---

## 4. Bilinen Sınırlar ve Dış Denetim Borçları (Known Limitations)

Aşağıdaki mimari yetenekler **Phase 0.40** kapsamında tamamlanacak olup, mevcut `v0.3-dev` sürümünde bilinçli olarak kapalı veya araştırma aşamasındadır:

1. **BLS / PQ Dilithium5 Donanım HSM Sürücüsü (`Phase 0.402`):** Ed25519 için PKCS#11 sürücüsü aktiftir; BLS/PQ için donanım entegrasyonu tamamlanana kadar disk yasağı fail-closed devrededir.
2. **`ConsensusStateV2` Canlı Şema Göçü (`Phase 0.408`):** Şema kancaları (`snapshot.rs`) hazırdır; canlı zincir üzerinde dinamik state transform hook'ları Phase 0.408 borcudur.
3. **Sürekli Fuzzing Altyapısı (`Phase 0.414`):** `cargo-audit` ve `SBOM` üretimi mevcuttur; sürekli (`continuous`) OZZ/AFL fuzzing hedeflenmektedir.
