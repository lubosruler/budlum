# Mainnet Hazırlık Raporu — ADIM2+ Planı

**Hazırlayan:** ARENA1  
**Tarih:** 2026-07-15 00:20 UTC+3  
**Temel commit:** `ee95ef0` (main) — 510 test passed, 0 failed  
**Durum:** ADIM1 (B.U.D. iskeleti + L1 stabilizasyon) tamamlandı. Mainnet önkoşulları analiz edildi.

---

## 1. Mevcut Durum Özeti (ADIM1 Sonrası)

| Bileşen | Test | Clippy | Fmt | Durum |
|---------|------|--------|-----|-------|
| `budlum-core` (L1) | 510 passed | `-D warnings` temiz | temiz | ✅ Stabil |
| `budzero/` (ZKVM) | Tüm workspace test geçti | temiz | temiz | ✅ Stabil |
| `fuzz/` | Build kontrolü tamam | — | — | ✅ Setup tamam |
| `docs/operations/` | Runbook, SBOM, audit script mevcut | — | — | ✅ Dokümantasyon tamam |

**ADIM1'de tamamlananlar:**
- B.U.D. Faz 1-2 + Faz 5 iskeleti (`StorageAttestation`, `ContentId`, `StorageDeal`, 7 RPC, E2E)
- `finality_live_path.rs` (4 test) — hatalı revert düzeltildi
- `chain_actor.rs` stub'ları → gerçek entegrasyon (ARENA3, `e5fd27f`)
- 18 derleme hatası + 5 clippy hatası sıfırlandı

---

## 2. Kritik Mainnet Blocker'lar (Kullanıcı Kararı Gerektiren)

Aşağıdaki 4 madde **ciddi stratejik kararlar** içerir. Her biri için seçenekler sunulmuştur. **Lütfen her madde için bir seçenek belirtin.**

### 2.1 VerifyMerkle Z-B Gate (BudZero) — EN KRİTİK

**Durum:** `budzero/bud-proof/src/plonky3_prover.rs:1711` — "Path verification (still TODO, Tur 10.6)."  
`budzero/bud-isa/src/lib.rs:39-43` — `VerifyMerkle` production'da **disabled**.  
`proves_verify_merkle_valid_64_depth` testi `#[ignore]` ile işaretli.

**Etki:** B.U.D. Faz 3 (gerçek Proof-of-Storage) bu gate'e bağlı. Gate açılmadan B.U.D. depolama kanıtı **kriptografik olarak güvenli değil** — sadece "interim retrieval challenge" (operatörün söylediğine güvenme) var.

**Seçenekler:**
- **A)** Gate'i açmadan mainnet'e git. `VerifyMerkle` experimental kalır. B.U.D. Faz 3 sonraki ADIM'a (ADIM4+) ertelenir. *Risk: B.U.D. mainnet'te "gerçek PoS" iddiası taşıyamaz.*
- **B)** ADIM2'de Z-B Commit 3.5'i tamamlayıp gate'i aç. 64-depth Poseidon path + final root check AIR constraint'leri tamamlanır. *Tahmini süre: 2-3 hafta. Risk: Büyük ZK mühendislik işi; süre tahmini güvenilir değil.*
- **C)** B.U.D.'yi mainnet'ten çıkar. Sadece L1 + BudZero core (31 opcode, settlement, bridge) mainnet'e girer. B.U.D. ayrı ADIM'da değerlendirilir. *Risk: Vizyonun merkeziyetsiz depolama bileşeni eksik kalır.*

**ARENA1 önerisi:** A veya C. B seçeneği mainnet lansmanını önemli ölçüde geciktirir ve ZK soundness garantisi verilemez.

---

### 2.2 BLS/PQ Anahtar Koruma Yolu (HSM)

**Durum:** `src/crypto/pkcs11.rs` — Gerçek PKCS#11 HSM entegrasyonu mevcut (Ed25519 için). Ancak BLS finality ve PQ (Dilithium) imzaları için **HSM yolu yok**. `AI_BIRLIGI.md` §4.5'te "Mock backend" seçilmiş ama kodda gerçek HSM var; BLS/PQ için ayrı bir yol gerekiyor.

**Etki:** Mainnet'te validator BLS key'leri ve PQ key'leri diskte saklanırsa, `AI_BIRLIGI.md` §4.4'teki "Mainnet disk ValidatorKeys yasağı" ihlal edilir.

**Seçenekler:**
- **A)** BLS/PQ için mock HSM backend ekle (`src/crypto/hsm_mock.rs`). Sadece test/development. Mainnet'te gerçek HSM zorunlu olur ama kodda mock var. *Risk: Mock production'a karışırsa güvenlik açığı.*
- **B)** BLS/PQ key'leri için de PKCS#11 yolunu genişlet (mevcut `pkcs11.rs`'ye BLS12-381 ve Dilithium mekanizmaları ekle). *Risk: HSM vendor'larının BLS/PQ desteği sınırlı olabilir.*
- **C)** Mainnet v1'de BLS/PQ disk key'ine izin ver (yumuşak geçiş). HSM yolu ADIM3'te zorunlu hale getirilir. *Risk: Güvenlik politikası gevşetilir.*

**ARENA1 önerisi:** B (mümkünse) veya C (zaman baskısı varsa). A seçeneği mock/production karışması riski taşır.

---

### 2.3 B.U.D. Mainnet'e Dahil mi?

**Durum:** ADIM1'de B.U.D. Faz 1-2 (kayıt/muhasebe) + Faz 5 (deal/challenge ekonomisi) tamamlandı. Faz 3 (gerçek PoS) kapalı.

**Etki:** B.U.D. mainnet'e girerse, operatörler `StorageDeal` açabilir, challenge yanıtlayabilir ama **kriptografik depolama kanıtı yok** — sadece ekonomik oyun teorisi (bond/slash) var.

**Seçenekler:**
- **A)** Evet, dahil et. Interim retrieval challenge ile başla. Faz 3 sonraki ADIM'da açılır. *Risk: Kullanıcılar "gerçek PoS" sanabilir.*
- **B)** Hayır, dahil etme. B.U.D. devnet'te kalır. Mainnet sadece L1 settlement + BudZero ZKVM. *Risk: Değer önerisi eksik.*
- **C)** Sadece Faz 1-2'yi dahil et (domain kayıt, manifest oluşturma). Faz 5 (deal/challenge) devnet'te kalır. *Risk: Yarım ürün algısı.*

**ARENA1 önerisi:** B veya C. A seçeneği kullanıcı beklentisi yönetimi açısından riskli.

---

### 2.4 Harici Güvenlik Denetimi (External Audit)

**Durum:** `docs/operations/DEPENDENCY_AUDIT.md` + `SBOM.md` + `scripts/audit-deps.sh` mevcut. Ancak **harici bir güvenlik firmasından kod denetimi yapılmadı**. `STATUS.md` §4.4'te "Harici audit yapılmadan README'de 'audited/mainnet ready' yazma" yasağı var.

**Etki:** Mainnet lansmanı "self-audited" olarak değerlendirilir. Kurumsal kullanıcılar/validator'lar harici audit raporu bekleyebilir.

**Seçenekler:**
- **A)** Harici audit olmadan mainnet'e git. İç denetim (Tur 10-14.9) yeterli kabul edilir. Audit ADIM3+'te yapılır. *Risk: "Kurumsal güven" eksikliği.*
- **B)** ADIM2'de harici audit checklist'ini tamamla (`docs/EXTERNAL_AUDIT_CHECKLIST.md`) ve bir güvenlik firmasına teslim et. *Tahmini süre: 4-8 hafta (firma süreci dahil).*
- **C)** Bug bounty programı ile başla (immunefi.com benzeri). Harici audit yerine crowdsourced denetim. *Risk: Zamanla kazanılan güven, anlık değil.*

**ARENA1 önerisi:** A veya C. B seçeneği mainnet lansmanını önemli ölçüde geciktirir.

---

## 3. ADIM Planı (Kullanıcı Kararlarına Göre Şekillenecek)

Aşağıdaki plan, **2.1-2.4 arasındaki kararların A seçeneği** (en hızlı mainnet yolu) alındığı varsayımıyla hazırlanmıştır. Farklı seçenekler planı değiştirir.

### ADIM2 — Mainnet Önkoşulları (Tahmini: 1-2 hafta)

**Hedef:** L1 + BudZero core'u mainnet'e hazır hale getirmek (B.U.D. hariç).

| # | Görev | Dosya/Hedef | Test Kriteri | Sahip |
|---|-------|-------------|--------------|-------|
| 2.1 | `VerifyMerkle` production gate kararını uygula | `budzero/bud-isa/src/lib.rs` | `tur119_verify_merkle_disabled_in_production` testi güncel | ARENA1/ARENA3 |
| 2.2 | BLS/PQ HSM mock backend ekle (eğer C seçilmediyse) | `src/crypto/hsm_mock.rs` | Mock BLS/PQ imza üretimi testi | ARENA1 |
| 2.3 | `ConsensusStateV2` migration hook ekle | `src/chain/snapshot.rs` | V2 → V3 migration testi | ARENA2 |
| 2.4 | README roadmap kapanış tablosu güncelle | `README.md` | Tüm org maddeleri "done/open" olarak işaretli | ARENA2 |
| 2.5 | Prometheus latency histogram wiring | `src/observability/` veya mevcut | Histogram metrikleri `/metrics`'te görünür | ARENA3 |
| 2.6 | Per-IP quota / operator admin methods netleştir | `src/rpc/server.rs` | Quota testleri mevcut | ARENA3 |
| 2.7 | Fuzzing CI build kontrolü | `fuzz/Cargo.toml` | `cargo check --manifest-path fuzz/Cargo.toml` temiz | ARENA1 |
| 2.8 | SBOM + dependency audit script CI'ya bağla (kullanıcı manuel) | `scripts/audit-deps.sh` | Script çalışır, rapor üretir | ARENA1 |

**CI Kabul Kriteri:** `cargo test --lib` + `cargo fmt --check` + `cargo clippy --lib --tests -- -D warnings` + `cargo test --manifest-path budzero/Cargo.toml --workspace` → hepsi yeşil.

---

### ADIM3 — Mainnet v1 Lansman Hazırlığı (Tahmini: 1 hafta)

**Hedef:** Genesis config, node dağıtım, operatör onboarding.

| # | Görev | Dosya/Hedef | Test Kriteri |
|---|-------|-------------|--------------|
| 3.1 | Mainnet genesis config oluştur | `src/chain/genesis.rs` | `test_genesis_deterministic` + yeni mainnet config testi |
| 3.2 | Docker image + systemd unit güncelle | `Dockerfile`, `docs/operations/` | Container başlar, RPC yanıt verir |
| 3.3 | Operatör runbook güncelle (mainnet spesifik) | `docs/operations/PRODUCTION_RUNBOOK.md` | Runbook'da mainnet genesis hash, seed node listesi |
| 3.4 | Network hardening (p2p, RPC rate limit) | `src/network/`, `src/rpc/` | Stress test: 10k bağlantı, rate limit çalışır |
| 3.5 | Validator onboarding flow (stake + register) | `src/registry/permissionless.rs` | E2E: yeni validator stake edip aktif olur |

---

### ADIM4 — B.U.D. Faz 3 (VerifyMerkle Açılışı) (Tahmini: 2-4 hafta)

**Hedef:** Gerçek kriptografik Proof-of-Storage. **Bu ADIM sadece 2.1'de B seçeneği seçilmediyse ayrı bir ADIM olarak kalır.**

| # | Görev | Dosya/Hedef | Test Kriteri |
|---|-------|-------------|--------------|
| 4.1 | `proves_verify_merkle_valid_64_depth` testi `#[ignore]`'den çıkar | `budzero/bud-proof/src/plonky3_prover.rs` | Test geçer, proof üretir ve verify eder |
| 4.2 | `VerifyMerkle` production gate aç | `budzero/bud-isa/src/lib.rs` | `tur119_verify_merkle_disabled_in_production` testi kaldır veya güncelle |
| 4.3 | B.U.D. Faz 3: `StorageDeal` + `VerifyMerkle` entegrasyonu | `src/domain/storage_deal.rs` | Deal açan operatör 64-depth Merkle proof sunar |
| 4.4 | B.U.D. Faz 4: `GlobalBlockHeader.storage_root` | `src/core/block.rs` | Block header'da storage_root alanı hash'e dahil |

---

### ADIM5 — Harici Denetim + Hardening (Tahmini: 2-8 hafta, firma bağımlı)

**Hedef:** Kurumsal güven ve uzun vadeli güvenlik.

| # | Görev | Dosya/Hedef | Test Kriteri |
|---|-------|-------------|--------------|
| 5.1 | Harici audit checklist tamamla | `docs/EXTERNAL_AUDIT_CHECKLIST.md` (yeni) | Teslim paketi hazır |
| 5.2 | Fuzzing run (24+ saat) | `fuzz/fuzz_targets/` | 0 crash |
| 5.3 | Chaos engineering testleri | `src/tests/chaos.rs` | Rastgele partition, latency injection |
| 5.4 | BNS/.bud isimlendirme (Faz 6) | Ayrı repo/ADIM | — |

---

## 4. Açık Teknik Borçlar (Kullanıcı Kararı Gerektirmeyen)

Bu maddeler **otomatik olarak** ADIM2 kapsamına alınabilir; stratejik karar gerektirmez.

| # | Borç | Neden Açık | Çözüm | Öncelik |
|---|------|------------|-------|---------|
| 4.1 | `budzero/bud-proof/src/bud_stark/prover.rs` 4 TODO | Optimizasyon/iyileştirme | ZK soundness'ı etkilemiyor; ADIM2'de temizlenebilir | 🟡 Düşük |
| 4.2 | `budzero/bud-proof/src/bud_stark/verifier.rs` 2 TODO | Preprocessed commitment taşıma | Performans etkisi; ADIM2'de temizlenebilir | 🟡 Düşük |
| 4.3 | `src/rpc/server.rs:1415,1451` zero-address placeholder | Tur 15'te tamamlanacak | Etkisi sınırlı; placeholder kullanımı güvenli | 🟡 Düşük |
| 4.4 | `src/chain/snapshot.rs:299` "ConsensusStateV2 fields" yorumu | Zaten `StateSnapshotV2` var | Yorum güncellemesi yeterli | 🟢 Çok düşük |

---

## 5. Diğer AI'lara Notlar

### ARENA2'ye
- Lütfen `ORG_ROADMAP_AUDIT.md` §4a'daki 18 madde tablosunu gözden geçir. ADIM1 sonrası hangi maddeler hâlâ "açık" olarak işaretlenmeli?
- `docs/MAINNET_READINESS.md` §2'deki 4 stratejik kararı kullanıcıyla birlikte değerlendir.
- ADIM2'deki görevlerden (§3) 2.3 (ConsensusStateV2 migration) ve 2.4 (README roadmap) sana atanabilir.

### ARENA3'e
- ADIM2'deki 2.5 (Prometheus histogram) + 2.6 (per-IP quota) + 2.1 (VerifyMerkle gate kararı uygulama) sana atanabilir.
- `chain_actor.rs` entegrasyonu (`e5fd27f`) için teşekkürler. Eksik bir `ChainCommand` var mı diye son kontrol yapabilir misin?

### Genel
- **Force-push yasak** (`AI_BIRLIGI.md` §6.1). Bu raporun commit'i normal push ile gönderilecek.
- **Workflow dosyası push yasak** (`AI_BIRLIGI.md` §6.2). CI entegrasyonu kullanıcıya bırakıldı.
- Her ADIM başlangıcında `STATUS_ONLINE.md`'ye entry yazılacak.

---

## 6. Sonraki Adım

1. **Kullanıcı kararı bekle:** §2.1-2.4 arasındaki 4 stratejik seçenek için yanıt.
2. Kararlar geldikten sonra ADIM2 branch'i aç (`arena/adim2-mainnet-prep`).
3. ADIM2 görev tablosunu parçala ve AI'lar arasında dağıt.
4. Her görev için ayrı commit; her commit öncesi `cargo test --lib` + `fmt` + `clippy` zorunlu.

**Kanıt:** Bu rapor `git log`, `cargo test --lib` (510 passed), `grep -rn TODO src/` (production kodunda 0) ve `grep -rn VerifyMerkle budzero/` (experimental gate aktif) verilerine dayanır.
