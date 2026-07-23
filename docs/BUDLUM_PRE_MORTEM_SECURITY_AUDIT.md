# 🔴 BUDLUM PRE-MORTEM GÜVENLİK ANALİZİ

**Tarih:** 2026-07-23  
**Repo:** github.com/budlum-xyz/budlum (1487 commit, 22 branch, public)  
**Analiz Yöntemi:** Kaynak kod incelemesi (wallet-core, budzero/bud-proof, budzero/bud-vm, src/crypto, Dockerfile, CI, .gitleaks)  
**Seviye:** KRİTİK — "hacklenirse her şeyi kaybederiz" senaryoları

---

## 📋 İÇİNDEKİLER

1. [Ses Azalı (Soundness) — ZK/AIR Katmanı](#1-soundness--zkair-katmanı)
2. [Privacy Transfer — Fon Kaybı](#2-privacy-transfer--fon-kaybı)
3. [TEE Runtime — Plaintext Sızıntısı](#3-tee-runtime--plaintext-sızıntısı)
4. [PKCS#11 / HSM — Anahtar Çalma](#4-pkcs11--hsm--anahtar-çalma)
5. [Consensus / Validator — Ağa Ele Geçirme](#5-consensus--validator--ağa-ele-geçirme)
6. [CI / Supply Chain — Repo Enjeksiyon](#6-ci--supply-chain--repo-enjeksiyon)
7. [Docker / Infra — Production Erişimi](#7docker--infra--production-erişimi)
8. [BIP39 / Wallet — Kullanıcı Fon Kaybı](#8-bip39--wallet--kullanıcı-fon-kaybı)
9. [Social Recovery — Hesap Ele Geçirme](#9-social-recovery--hesap-ele-geçirme)
10. [AI Execution — VerifyInference Bypass](#10-ai-execution--verifyinference-bypass)
11. [Özet: En Kritik 5 Hack Yolu](#11-özet-en-kritik-5-hack-yolu)
12. [Hedefli Düzeltme Planı](#12-hedefli-düzeltme-planı)

---

## 1. Ses Azalı (Soundness) — ZK/AIR Katmanı ⚠️ KRİTİK

### Bulgu 1.1: Merkle Poseidon Round — FARKLI FIELD KULLANIMI

**Dosya:** `bud-vm/src/lib.rs` — `merkle_poseidon_round()`

VM'deki Merkle Poseidon single-round fonksiyonu **P = 0xFFFFFFFF00000001** (Pallas/Baby Jubjub field) kullanıyor:

```rust
pub fn merkle_poseidon_round(a: u64, b: u64) -> u64 {
    const P: u64 = 0xFFFFFFFF00000001;  // ← Baby Jubjub prime!
    ...
```

Ama STARK AIR (`plonky3_air.rs`) ve VM'nin Add/Sub/Mul opcode'ları **P = 18446744069414584321** (Goldilocks prime) kullanıyor:

```rust
pub const GOLDILOCKS_P: u64 = 18446744069414584321;  // ← Goldilocks prime
pub const TRACE_WIDTH: usize = 414;  // AIR constraint domain
```

**Ses azalı boku (soundness break):** Merkle VerifyMerkle opcode'u, 64 expansion row'da Pallas field'da hesaplama yapıyor, ama AIR constraint'leri Goldilocks field'da çalışıyor. Bu iki field arasında **value mapping** yok — bir malicious prover:

1. Pallas'da geçerli bir Merkle path hesaplar
2. Goldilocks AIR'de bu path'ın "geçerli" görünmesini sağlar  
3. Ama Pallas'daki sonuç ≠ Goldilocks'daki sonuç olabilir
4. Bu, **sahte Merkle root doğrulaması** yapılmasına izin verir

**Sonuç:** State root manipülasyonu → tüm zincir trust'i bozulur → tüm fonlar tehlikeye girer.

**Düzeltme:** `merkle_poseidon_round` GOLDILOCKS_P kullanmalı veya AIR'de Pallas-specific constraint'ler olmalı. **Her iki field aynı olmalı.**

---

### Bulgu 1.2: Poseidon Round Constants — wallet-core vs bud-vm Alignment Riski

**Dosyalar:** `wallet-core/src/privacy_crypto.rs` vs `budzero/bud-vm/src/lib.rs`

wallet-core **deliberatly duplicate** Poseidon primitives (mobile/WASM footprint için bud-vm dependency yok). Kod yorumunda:

```
//! wallet-core intentionally does **not** depend on bud-vm (mobile/WASM
//! footprint); this is a deliberate duplicated primitive with lock tests.
```

Test sadece `poseidon4_hash(1,2) == poseidon4_hash3(1,2,0)` şeklinde **değer bazlı** assertion yapıyor. Round constants (`RC` array), MDS matrix — her iki dosyada ayrı ayrı tanımlı.

**Risk:**  
- RC/MDS değerleri bir PR'da sadece bud-vm'de değiştirilirse → wallet-core ile AIR arasında **desync**  
- PrivacyCommit/NullifierCheck trace'de farklı değerler → AIR constraint fail veya **soundness hole**  
- Lock test var ama sadece `poseidon4_hash`'i test ediyor, **RC array'in her elemanını** test etmiyor

**Sonuç:** wallet-core nullifier hesaplaması AIR'den farklı olur → nullifier collision → **double-spend** yapılabilir.

**Düzeltme:**  
- Poseidon RC/MDS constant'larını **tek bir crate'te** (örneğin `bud-isa` veya yeni `poseidon-params` crate) tanımla  
- wallet-core ve bud-vm bu crate'ı referans al  
- Lock test her RC/MDS değerini tek tek assert et  

---

### Bulgu 1.3: PrivacyCommit — Blinding truncation (imm → u32 → u64)

**Dosya:** `bud-vm/src/lib.rs` PrivacyCommit opcode:

```rust
Opcode::PrivacyCommit => {
    let blinding = inst.imm as u32 as u64;  // ← 64-bit imm TRUNCATED to 32 bits!
    let result = poseidon4_hash3(amount, recipient, blinding);
```

**imm** i32 tipinde (32-bit signed), `as u32 as u64` ile u64'a çevrilir. Ama **wallet-core'da**:

```rust
pub fn privacy_commit(amount: u64, recipient_tag: u64, blinding: u64) -> u64 {
    poseidon4_hash3(amount, recipient_tag, blinding)  // ← full u64 blinding!
```

**Ses azalı boku:** wallet-core 64-bit blinding ile commitment üretir, VM 32-bit truncated blinding ile. Eğer blinding > 2³² ise:

- wallet-core: `commit(A, R, B_full)`  
- VM/AIR: `commit(A, R, B_truncated)`  
- İki commitment farklı → AIR NullifierCheck/PrivacyCommit constraint'leri **uyuşmaz** → proof verify fail veya **malicious prover tarafından manipüle edilebilir**

**Sonuç:** İnvalid proof üretimi veya blinding brute-force (32-bit → 4.3 milyar kombinasyon → brute-force mümkün)

**Düzeltme:** PrivacyCommit opcode blinding'ı **register'dan** almalı (rs2 veya ayrı register), imm truncation kaldırılmalı. En azından AIR constraint'leri truncated blinding ile uyuşmalı.

---

### Bulgu 1.4: SumConservation — Field arithmetic mismatch

**Dosya:** `bud-vm/src/lib.rs`:

```rust
Opcode::SumConservation => {
    let result = if sum_in == sum_out { 1 } else { 0 };  // ← u64 comparison!
```

SumConservation **u64 native comparison** kullanıyor (sum_in == sum_out). Ama AIR'de bu opcode Goldilocks field arithmetic'da constraint'lenmiş. 

**Risk:**  
- u64'da `18446744069414584320 + 1 == 0` (overflow wrapping)  
- Goldilocks'da `18446744069414584320 + 1 == 18446744069414584321 == P` (field modulus)  
- Büyük amount'larda sum conservation **farklı sonuç** → AIR soundness break

**Düzeltme:** SumConservation **Goldilocks field comparison** kullanmalı (`field_add_goldilocks` ile).

---

## 2. Privacy Transfer — Fon Kaybı ⚠️ KRİTİK

### Bulgu 2.1: Nullifier — Deterministic Derivation → Collision

**Dosya:** `wallet-core/src/privacy_crypto.rs`:

```rust
pub fn privacy_nullifier(secret: u64) -> u64 {
    poseidon4_hash(secret, DOMAIN_NULLIFIER)
}
```

Nullifier = Poseidon2(secret, DOMAIN_NULLIFIER). **secret sadece u64** → Goldilocks field element'den küçük → 2⁶⁴ farklı nullifier olası.

**Risk:** Nullifier'ın sadece `secret` limb'e bağlı olması (amount/recipient/blinding dahil değil) →  
- **Same secret, different note** → aynı nullifier üretilir → **first-spend lock** → ikinci note'un nullifier'ı da aynı → double-spend algılanamaz
- Nullifier collision → **tekrar harcanabilir note** → fon kaybı

**Düzeltme:** Nullifier türetimi **commitment + secret** birlikte kullanmalı: `Poseidon2(secret, commitment)` veya `Poseidon3(secret, DOMAIN_NULLIFIER, commitment)`.

---

### Bulgu 2.2: address_to_recipient_tag — Collisions

**Dosya:** `wallet-core/src/privacy_crypto.rs`:

```rust
pub fn address_to_recipient_tag(addr: &[u8; 32]) -> u64 {
    let raw = u64::from_le_bytes(addr[..8].try_into().expect("addr"));
    raw % GOLDILOCKS_P
}
```

32-byte address → sadece ilk 8 byte alınıp mod P ile field element'e map ediliyor.

**Risk:**  
- SHA3-256(public_key) = 32 byte address  
- İlk 8 byte ≈ 2⁶⁴ farklı olası → Goldilocks P ≈ 2⁶⁴  
- Mod P collision probability: ≈ 1/P ≈ çok küçük (pratikte sorun değil)  
- **Ama:** İlk 8 byte truncation → birthday bound'ta ~2³² farklı address ile collision olası  
- Collision → farklı kullanıcıların recipient_tag'leri aynı → **fund gönderimi yanlış recipient'a**

**Not:** Pratik risk düşük ama **sondaya** kadar güvenli değil (2³² ≈ 4 milyar address).

**Düzeltme:** Recipient tag hesaplaması SHA3-256'nın tamamını kullanmalı (örneğin tüm 32 byte'ı Poseidon absorb eden bir hash).

---

### Bulgu 2.3: derive_spend_secret — Predictability

**Dosya:** `wallet-core/src/privacy_transfer.rs`:

```rust
pub fn derive_spend_secret(wallet_seed: &[u8; 32], note_commitment: u64) -> u64 {
    let mut h = Sha3_256::new();
    h.update(b"BUDLUM_NOTE_SPEND_SECRET_V1");
    h.update(wallet_seed);
    h.update(note_commitment.to_le_bytes());
    let out = h.finalize();
    u64::from_le_bytes(out[..8].try_into().unwrap())
}
```

Spend secret = SHA3-256(seed + commitment)'in ilk 8 byte. Deterministic → same seed + same commitment = same secret.

**Risk:**  
- `note_commitment` public (zincire yazılır)  
- `wallet_seed` gizli ama **seed leakage** → tüm spend_secret'ler recoverable  
- spend_secret leak → tüm nullifier'lar hesaplanabilir → **tüm note'lar harcanabilir**  
- Bu Zcash-style tasarım, seed leak = total fund loss

**Bu beklenen bir risk** (seed leak = hesap çalma), ama spend_secret'in **deterministic** olması → aynı seed'den türetilmiş tüm note'lar birbirine bağlı → bir note'un leak'i → seed recoverable mı? Hayır (单向 hash). Ama seed leak → total loss.

**Düzeltme:** Seed saklama kritik. Multi-layer encryption + HSM storage zorunlu.

---

## 3. TEE Runtime — Plaintext Sızıntısı ⚠️ ORTA-YÜKSEK

### Bulgu 3.1: UnavailableTeeRuntime — fail-closed ama ENKLAV YOK

**Dosya:** `wallet-core/src/tee.rs`:

```rust
pub trait TeeRuntime: Send + Sync {
    fn seal_private_intent(&self, _plaintext: &[u8]) -> Result<Vec<u8>, WalletError> {
        Err(WalletError::TeeUnavailable(...))  // ← fail-closed
    }
}
```

UnavailableTeeRuntime `tee_enabled=true` durumunda plaintext path'i reddeder (fail-closed). **Ama:**

- Gerçek SGX/Nitro enklav entegrasyonu **aynı repoda yok** — "separate hardware/SDK track" yorumu
- KALAN_ISLER listesinde "real enclave" açıkça not edilmiş
- `tee_enabled=true` + `UnavailableTeeRuntime` → **kullanıcı privacy seçti ama TEE yok** → tüm işlemler düz-metin olarak relayer'a gider

**Risk:**  
- Kullanıcı "TEE gizlilik" seçeneğini açar  
- Gerçek TEE backend yok → `tee_enabled=true` ama runtime unavailable  
- `WalletError::TeeUnavailable` hatası → **işlem üretilemez** veya uygulama sessizce TEE'yi bypas eder  
- Bypass → **plaintext işlem relayer'a gider** → relayer tüm transfer detaylarını görür → operatör trust break

**Düzeltme:**  
- Mainnet launch'dan önce gerçek SGX/Nitro entegrasyonu zorunlu  
- `tee_enabled=true` + unavailable runtime → **hard fail**, sessiz bypass yok  
- Ama mevcut kodda bu hard fail doğru → **riske açık olan UX fallback logic** olabilir (uygulama sessizce `tee_enabled=false`'a downgrade eder mi?)

---

### Bulgu 3.2: view_key — Selective Disclosure ama Zincir Bağlantısı Yok

**Dosya:** `wallet-core/src/lib.rs`:

```rust
pub fn ensure_view_key(&mut self, wallet_seed: &[u8; 32]) -> [u8; 32] {
    let vk = derive_view_key(wallet_seed);
    self.view_key = Some(vk);
    vk
}
```

View-key wallet seed'den deterministik türetiliyor. **Ama:**

- View-key **zincire yazılmaz** (yorumda açıkça belirtilmiş)  
- View-key **kimlik doğrulama mekanizması** yok → herhangi biri view-key alıp decrypt edebilir  
- View-key rotate mevcut ama **rotation_counter** external state → replay attack olası

**Düzeltme:** View-key exchange authenticated channel + signing zorunlu.

---

## 4. PKCS#11 / HSM — Anahtar Çalma ⚠️ KRİTİK

### Bulgu 4.1: PKCS#11 PIN — Environment Variable Exposure

**Dosya:** `src/crypto/pkcs11.rs`:

```rust
let pin = std::env::var(&token_pin_env).map_err(...)?;
if pin.is_empty() { return Err(...); }
```

PKCS#11 PIN **environment variable'dan** okunuyor. 

**Risk:**  
- Process environment → `/proc/PID/environ` ile erişilebilir (Linux)  
- Container'da environment leak → docker inspect, kubernetes secret mount  
- PIN leak → HSM session open → **validator signing key kullanımı** → ağ ele geçirme

**Düzeltme:**  
- PIN file-based loading (protected file, 0600 permissions)  
- container'da environment variable ENCRYPTED olmalı  
- Process dump protection (prctl PR_SET_DUMPABLE=0)

---

### Bulgu 4.2: BLS/PQ Key Storage in HSM Data Objects

**Dosya:** `src/crypto/pkcs11.rs`:

```rust
Self::create_data_object(&inner.session, BLS_DATA_LABEL, &keypair.to_bytes());
Self::create_data_object(&inner.session, PQ_DATA_LABEL, &bytes);
```

BLS ve PQ secret key'ler HSM'de **data object** olarak saklanıyor (CKO_DATA), **private key object** değil.

**Risk:**  
- CKO_DATA → **extractable** (PKCS#11 spec)  
- `cryptoki::object::Attribute::Extractable` = true (default)  
- HSM data object read → **BLS/PQ secret key extraction**  
- BLS key leak → **finality attack** (BLS signature forge → consensus manipulation)

**Düzeltme:**  
- BLS/PQ secret key'ler **CKO_PRIVATE_KEY** object olarak saklanmalı (non-extractable flag)  
- Veya PKCS#11 vendor-specific mechanism ile hardware-native keygen (in-HSM, never extractable)

---

### Bulgu 4.3: HSM Mock Removed ama Policy Enforcement Gaps

Mock HSM (`hsm_mock.rs`) kaldırılmış ve `MainnetKeyPolicyViolation::HsmMockBackend` reddediyor. **Ama:**

- `signer_backend = "local"` → mainnet validator reddediliyor ✓  
- `signer_backend = None` → reddediliyor ✓  
- **Edge case:** config parsing'de `signer_backend` field eksik → `unwrap_or("")` → `"pkcs11"` ile eşleşmiyor → reddediliyor ✓  
- **Ama:** `signer_backend` config'de `"pkcs11"` yazılı ama actual PKCS#11 module yüklenemez → **runtime crash** → validator offline → liveness risk

**Düzeltme:** Runtime PKCS#11 initialization failure → **graceful shutdown + alerting** zorunlu.

---

## 5. Consensus / Validator — Ağa Ele Geçirme ⚠️ KRİTİK

### Bulgu 5.1: VerifyMerkle Env Var Gate — Runtime Manipulation

**Dosya:** `bud-vm/src/lib.rs`:

```rust
fn is_verify_merkle_enabled() -> bool {
    std::env::var("BUDLUM_VERIFY_MERKLE")
        .map(|v| v.to_lowercase() != "false" && v != "0")
        .unwrap_or(true)  // ← DEFAULT TRUE!
}
```

**Risk:**  
- Environment variable runtime'da değiştirilebilir (process restart gerekmez, bazı container runtime'larda)  
- `BUDLUM_VERIFY_MERKLE=false` → VerifyMerkle opcode **disabled**  
- VerifyMerkle disabled → Merkle path verification bypass → **state root manipulation** → ağ consensus trust break

**Sonuç:** Bir node operator (veya container orchestration) env var değiştirerek Merkle verification'ı bypass eder → sahte state root → tüm ağ trust'ı bozulur.

**Düzeltme:**  
- VerifyMerkle enable/disable **genesis config'de** hard-coded olmalı (env var değil)  
- Veya AIR'de VerifyMerkle selector **her zaman constraint'li** → env var sadece VM execution'ı etkiler, AIR verification'ı etkilemez  
- **Bu çok kritik:** AIR verification ile VM execution uyumlu olmalı

---

### Bulfu 5.2: Syscall Opcode — Information Leak

**Dosya:** `bud-vm/src/lib.rs`:

```rust
Opcode::Syscall => {
    let result = match inst.imm {
        1 => self.context.sender,
        2 => self.context.block_height,
        3 => self.context.nonce,
        6 => {
            self.events.push(0x00A1_00A1);  // ← magic number event
            self.events.push(src1_val);
            self.context.block_height.saturating_add(src1_val)
        }
        _ => 0,
    };
```

Syscall imm=6 → **magic event push** (0x00A1_00A1) + user-controlled value (src1_val) event'e yazılır.

**Risk:**  
- Event leak → src1_val (user data) chain event log'a yazılır → information disclosure  
- Syscall context (sender/nonce/block_height) → prover trace'de observable → privacy break for private transfers  
- Syscall'lar AIR'de **constraint'lenmiyor** → malicious prover farklı syscall result claim edebilir

**Düzeltme:** Syscall result'ları AIR'de constraint'lenmeli. Syscall imm=6 magic event kaldırılmalı veya audit log olmalı (public event değil).

---

## 6. CI / Supply Chain — Repo Enjeksiyon ⚠️ ORTA

### Bulgu 6.1: BADGE_PUSH_TOKEN — Admin PAT in CI

**Dosya:** `.github/workflows/ci.yml`:

```yaml
env:
  BADGE_PUSH_TOKEN: ${{ secrets.BADGE_PUSH_TOKEN }}
```

CI badge push için **admin PAT** (branch protection bypass) kullanılıyor.

**Risk:**  
- PAT leak → main branch'e **doğrudan push** (branch protection bypass!)  
- Fork + PAT → arbitrary code injection → production binary manipulation  
- Gitleaks var ama PAT'ler secret'te → gitleaks tarayamaz  
- `persist-credentials: true` + `http.extraheader` → checkout GITHUB_TOKEN'ı HTTP header'a zorlar → URL-embedded PAT'i ezer (commit comment'da belirtilmiş)

**Sonuç:** BADGE_PUSH_TOKEN leak → main branch'e doğrudan commit → **tüm kod manipüle edilebilir** → supply chain attack → compiled binary trojaned → tüm node'lar compromised.

**Düzeltme:**  
- Badge push → **separate workflow** (different trigger, scoped token)  
- Admin PAT → **deploy key** (read-only + specific file write only)  
- Branch protection: **required review + signed commit** zorunlu  
- `persist-credentials: false` CI'de zorunlu

---

### Bulgu 6.2: CODEOWNERS — 2 Kişi Catch-all

**Dosya:** `.github/CODEOWNERS`:

```
* @lubosruler @eurymedee
/src/consensus/ @lubosruler @eurymedee
/src/crypto/ @lubosruler @eurymedee
/src/rpc/ @lubosruler @eurymedee
/config/ @lubosruler @eurymedee
```

**Tüm dosyalar** sadece 2 gerçek org üyesine ait. Team yapısı yok.

**Risk:**  
- Compromised account (lubosruler veya eurymedee) → **tüm repo erişimi** → self-approve → malicious merge  
- Bus factor = 2 → bir account hack → total repo control  
- No independent review → insider threat

**Düzeltme:**  
- Minimum 3 independent reviewer zorunlu  
- Security-critical path'ler (crypto, consensus, wallet-core) → **dedicated security team** reviewer  
- GitHub team structure kurulmalı

---

### Bulfu 6.3: clippy-extra-baseline Ratchet — Pedantic Warnings Suppressed

**Dosya:** `.github/clippy-extra-baseline.txt` + CI:

```yaml
cargo clippy --all-targets --message-format=json -- -W clippy::pedantic -W clippy::nursery
bash ./scripts/check-clippy-extra.sh /tmp/clippy-extra.json
```

Baseline 191 → 217 bump (26 yeni pedantic/nursery warning allow). Skeleton modüller için gerekçeli bump.

**Risk:**  
- Pedantic/nursery warning'ler **security-relevant** olabilir (integer overflow, unsafe usage, etc.)  
- Baseline bump → new warning'ler **tracking modunda** ama çözülmemiş  
- "Mainnet sonrası" planı → mainnet launch'den önce çözülmeyebilir

**Düzeltme:** Security-relevant pedantic warning'ler (integer arithmetic, unsafe,unwrap) **mainnet'den önce** çözülmeli. Baseline bump sadece cosmetic/style için olmalı.

---

## 7. Docker / Infra — Production Erişimi ⚠️ ORTA

### Bulgu 7.1: Dockerfile — CMD devnet default (düzeltildi ✓)

**Dosya:** `Dockerfile`:

```dockerfile
CMD ["--network", "devnet", "--port", "4001"]
```

CMD devnet default → güvenli. **Ama:**

- Port 8545 (RPC public) ve 8546 (RPC operator) expose edilmiş  
- RPC authentication yok (Dockerfile'da auth middleware görünmüyor)  
- Node deploy → public RPC → **anyone can query/submit transactions** → spam/DoS

**Düzeltme:** RPC endpoint'lerde rate-limiting + authentication zorunlu. Prometheus metrics (9090) internal network'de olmalı.

---

### Bulfu 7.2: Docker Image — Pinned SHA ama Builder Stage Verifiable değil

```dockerfile
FROM rust:1.97.1-bookworm@sha256:77fac8b... AS builder
FROM debian:bookworm-slim@sha256:7b140f3... AS runtime
```

Her iki base image SHA256 pinned ✓. **Ama:**

- Builder stage'de `apt-get install protobuf-compiler clang cmake` → **unpinned package**  
- `cargo build --release --locked` → Cargo.lock pinned ✓  
- Builder stage → **dependency chain trust** → Debian package repo trust

**Risk:** Compromised Debian mirror → trojaned clang/protobuf → **compiled binary manipulation** → production node compromised.

**Düzeltme:**  
- Builder stage package pinning (version lock)  
- Veya: multi-stage build'de builder image reproducibility verification (hash check)  
- Binary reproducibility audit zorunlu

---

## 8. BIP39 / Wallet — Kullanıcı Fon Kaybı ⚠️ KRİTİK

### Bulgu 8.1: Wallet::generate() — CSPRNG Dependency

**Dosya:** `wallet-core/src/lib.rs` (chunk 0'da görünmüyor ama WalletError enum'da):

```rust
WalletError::ProductionEntropyUnavailable(String),
```

CSPRNG unavailable → fail-closed. **Ama:**

- Wallet seed 32 byte → **all security derives from this seed**  
- Seed leak → total fund loss (tüm signing key'ler, spend secret'ler, view key)  
- Seed storage → application responsibility (wallet-core depolama yapmaz)

**Risk:** Seed'in RAM'de tutulması → memory dump attack → **total wallet drain**

**Düzeltme:**  
- Wallet seed memory'de **mlock()** (Linux) → swap prevention  
- Seed zeroization after key derivation  
- `zeroize` crate dependency zorunlu  
- HSM-based seed storage (production)

---

### Bulfu 8.2: Mnemonic Wordlist — Canonical BIP39 ✓ (düzeltildi)

BIP39 wordlist restore commit var (`e103c88`). Ama:

- wordlist 2048 kelime → **compile-time embedded** → mutable değil ✓  
- checksum verification SHA256 → BIP39 spec ✓

**Bu düzgün.** Risk düşük.

---

## 9. Social Recovery — Hesap Ele Geçirme ⚠️ KRİTİK

### Bulfu 9.1: Guardian Approval — No Expiry/Revocation

**Dosya:** `wallet-core/src/lib.rs`:

```rust
pub fn verify_recovery_threshold(
    &self,
    recovery_digest: &[u8],
    approvals: &[GuardianApproval],
) -> bool {
    let mut seen = Vec::<[u8; 32]>::new();
    for approval in approvals {
        if !self.guardians.contains(&approval.public_key) { continue; }
        if seen.contains(&approval.public_key) { continue; }
        if Wallet::verify(&approval.public_key, recovery_digest, &approval.signature) {
            seen.push(approval.public_key);
        }
    }
    seen.len() >= self.threshold
}
```

**Risk:**  
- Guardian approval'lar **expiry yok** → bir guardian 6 ay önce approve etti → hala geçerli  
- Guardian revocation → policy rotate yapılır AMA eski approval'lar **rotate öncesi policy'de** geçerli olabilir  
- `timelock_blocks` minimum → çok kısa timelock → rapid account theft

**Sonuç:** Guardian compromise → threshold guardian'lar çalınır → recovery proposal approve → **hesap ele geçirilir** → tüm fonlar transfer.

**Düzeltme:**  
- Guardian approval expiry (timestamp-based)  
- Recovery proposal revocation mechanism  
- Minimum timelock ≥ 48 hours (blocks-based calculation)  
- Recovery execution → **double confirmation** (current owner + guardians)

---

## 10. AI Execution — VerifyInference Bypass ⚠️ ORTA

### Bulfu 10.1: VerifyInference — V110 Fail-Closed ama AIR Gap

**Dosya:** `bud-vm/src/lib.rs`:

```rust
Opcode::VerifyInference => {
    let _proof_addr = src1_val as usize;
    let _model_addr = src2_val as usize;
    let result = 0u64;  // ← ALWAYS FAILS
```

VerifyInference **her zaman 0 döner** (V110: fail-closed). **Ama:**

- AIR'de COL_IS_VERIFY_INFERENCE (373) selector constraint'lenmiş ✓  
- `rd_val_new = 0` (V110) constraint'lenmiş ✓  
- Expansion rows (8 row) commitment chain constraint'lenmiş ✓

**Risk:**  
- AIR'de VerifyInference expansion rows → 8 commitment limb row → **these rows are constrained** ✓  
- AMA: commitment chain ** içerik doğrulaması** yok → AIR sadece "commitment'lar tutarlı" check eder, **actual inference computation verification** yok  
- "Malicious prover cannot bypass selector or tamper commitment chain" → ✓ (selector bound)  
- AMA: commitment chain **semantic verification** → model_id, input, output'nun gerçekten doğru inference output olduğu → **NOT VERIFIED**  

**Sonuç:** AI execution proof'u sahte olabilir (commitment'lar tutarlı ama computation yanlış). V110'da rd_val_new=0 → **mainnet'de AI verification disabled** → güvenli. Ama gelecekte activation → **semantic verification gap** kapatılmalı.

---

### Bulfu 10.2: AI Model Whitelist — FixedPointMlpV1 Only

**Dosya:** commit message:

```
- FixedPointMlpV1 whitelist, guest ISA builder, structural verify
```

Sadece FixedPointMlpV1 model whitelist'te. **Ama:**

- Whitelist enforcement **AIR'de değil**, VM execution'da  
- Malicious prover → farklı model spec → AIR'de geçerli trace (commitment chain tutarlı) → ama model whitelisted değil → **on-chain verification gap**

**Düzeltme:** Model whitelist AIR constraint'le (model_id → registered program_hash bind).

---

## 11. Özet: En Kritik 5 Hack Yolu 🔴

| # | Hack Yolu | Severity | Sonuç | Düzeltme Aciliyet |
|---|-----------|----------|-------|--------------------|
| **1** | **Merkle Poseidon field mismatch** (Pallas vs Goldilocks) | 🔴 KRİTİK | State root manipülasyonu → tüm zincir trust'ı bozulur → tüm fonlar | **MAİNNET ÖNCESİ ZORUNLU** |
| **2** | **Nullifier collision** (secret-only derivation) | 🔴 KRİTİK | Double-spend → fon kaybı | **MAİNNET ÖNCESİ ZORUNLU** |
| **3** | **PKCS#11 data object BLS/PQ key** (extractable) | 🔴 KRİTİK | Validator key extraction → ağ ele geçirme | **MAİNNET ÖNCESİ ZORUNLU** |
| **4** | **BADGE_PUSH_TOKEN admin PAT leak** | 🔴 KRİTİK | Main branch direct push → supply chain attack → tüm node'lar | **HEMEN** |
| **5** | **PrivacyCommit blinding truncation** (u32 vs u64) | 🟡 YÜKSEK | Proof mismatch / brute-force | **MAİNNET ÖNCESİ ZORUNLU** |

---

## 12. Hedefli Düzeltme Planı

### Phase 1 — HEMEN (0-3 gün)

1. **BADGE_PUSH_TOKEN:** Scope'u daralt → deploy key (read-only, sadece README.md push)  
2. **CI persist-credentials:** `false` zorunlu  
3. **PKCS#11 PIN:** File-based loading, environment variable encryption  
4. **Process dump protection:** `prctl PR_SET_DUMPABLE=0` + container securityContext

### Phase 2 — MAİNNET ÖNCESİ (3-30 gün)

5. **Merkle Poseidon field:** `merkle_poseidon_round` GOLDILOCKS_P kullanmalı  
6. **Poseidon constants dedup:** Tek crate'ten referans  
7. **Nullifier derivation:** `Poseidon(secret, DOMAIN_NULLIFIER, commitment)`  
8. **PrivacyCommit blinding:** Register-based blinding (imm truncation kaldır)  
9. **SumConservation:** Goldilocks field comparison  
10. **BLS/PQ key HSM storage:** CKO_PRIVATE_KEY (non-extractable)  
11. **VerifyMerkle gate:** Genesis config hard-coded, env var kaldır  
12. **CODEOWNERS:** 3+ independent reviewer, security team

### Phase 3 — MAİNNET SONRASI (sürekli)

13. **Fuzz coverage:** Continuous fuzzing CI'da (nightly matrix)  
14. **Wallet seed zeroize:** `zeroize` crate + mlock  
15. **Social recovery expiry:** Timestamp-based approval expiry  
16. **AI execution semantic verification:** Future STARK verification AIR  
17. **RPC authentication:** Rate-limiting + JWT/mTLS  
18. **Reproducible builds:** Builder stage pinning + hash verification

---

**Bu rapor, repodaki mevcut kodun incelenmesiyle hazırlanmıştır. Ses azalı (soundness) bulguları (#1, #5) cryptographic proof sistemi'nin temelini sarsabilir — bu düzeltilmeden mainnet launch **tüm fon kaybı riski** taşır.**
