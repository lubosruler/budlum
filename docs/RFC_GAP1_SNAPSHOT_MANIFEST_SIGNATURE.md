# RFC — GAP-1: Snapshot Manifest İmzası (Authenticity)

| | |
|---|---|
| **Durum** | DRAFT — kullanıcı onayı bekliyor |
| **Yazar** | ARENA3 (güvenlik/altyapı/HSM domain'i) |
| **Tarih** | 2026-07-18 (UTC+3) |
| **İlgili** | GAP-1 (bu RFC'nin konusu), GAP-2 (hash-kapsam genişletme — halef koordinasyonlu), GAP-3/GAP-4 (KAPALI — PR #48, `532ca51`) |
| **Pin testleri** | `test_snapshot_v2_rehash_forgery_no_authenticity_gap`, `test_snapshot_v2_unhashed_field_forgery_gap` (`src/tests/snapshot_chaos.rs`) |

---

## 1. Problem tanımı

`StateSnapshotV2::verify()` (`src/chain/snapshot.rs:597`) yalnızca şunu kontrol eder:

```rust
pub fn verify(&self) -> bool {
    self.snapshot_hash == self.calculate_hash()
}
```

`calculate_hash` deterministik Sha3-256'dır ve girdileri **tamamen halka açık** alanlardır (yükseklik, bakiyeler, nonce'lar, validator seti, kökler). Sır, anahtar, imza — authenticity sağlayacak hiçbir kriptografik bağ yoktur.

**Sonuç (GAP-1, pin testiyle kanıtlı):** diske yazabilen herhangi bir saldırgan `balances` içine kendi adresine keyfi bakiye enjekte eder, `calculate_hash`'i tekrar koşturur, `snapshot_hash` alanını günceller → `verify()` **true** döner → boot zinciri bu sahte durumu yükler. `verify()` bir *corruption dedektörü*dür (bit-rot, torn-write), bir *sahtecilik dedektörü* değildir.

**Ek eksik (GAP-2, ayrı pin):** `calculate_hash` şu alanları kapsamıyor: `tokenomics`, `tokenomics_burn`, `registry`, `liveness`, `invalid_votes`, `bns_registry`, `nft_registry`, `marketplace`, `hub`, `storage_registry`, `bridge_state`, `message_registry`, `external_roots`, `finality_certificates`, `created_at`. Bu alanlara yapılan enjeksiyon **rehash bile gerektirmeden** `verify()`'i geçer. (GAP-2'nin onarımı bu RFC ile tek schema-4 sürümünde birleştirilmesi önerilmektedir — bkz. §7 Açık Soru 4.)

## 2. Tehdit modeli

Tam koruma hedefi şu saldırgan sınıflarıdır:

1. **Host compromise:** snapshot dizinine (`snapshots/`) yazabilen süreç/kullanıcı (node ayrıcalığından düşük olsa da olabilir — container escape, yedek servisi, ops hatası).
2. **Yedek/transfer zehirlenmesi:** snapshot dosyasının host dışına kopyalanıp geri alındığı (backup/restore, node migration) senaryolarda araya giren değiştirme.
3. **Gelecek P2P snapshot dağıtımı (state sync):** başka bir peer'dan alınan snapshot'ın doğrulanması — şu an yok ama mainnet yol haritasında; tasarım bunu *kırmadan* genişlemeli.
4. **Operatör hatası / insider:** yanlışlıkla başka zincirin snapshot'ının yüklenmesi (chain-id/genesis-hash uyumsuzluğu bugün hash'e dahil ama authenticity değil).

Kapsam dışı: node prosesinin kendisinin compromise olması (o durumda imzalayan anahtar da ele geçmiştir; HSM mitigasyonu §5.4'te).

## 3. Hedefler / hedef olmayanlar

**Hedefler:**
- H1: Sahte/bozulmuş manifest yüklemesi **fail-closed** olur: red → GAP-3 karantina döngüsü → bir sonraki aday → tükendiyse boot hatası (sessiz fallback YOK).
- H2: İmzalama yolu **HSM uyumlu** olur: `ConsensusSigner` trait'i (`src/crypto/signer.rs`) üzerinden — PKCS#11 backend'i (Ed25519-only, `src/crypto/pkcs11.rs`) ve `KeyPairSigner` aynı kod yolundan geçer.
- H3: Geriye uyumluluk: schema ≤3 imzasız snapshot'lar politika kontrollü geçiş penceresinden yararlanır; mainnet politikası imzasızı kabul etmez.
- H4: Anahtar rotasyonu bir hard-fork'a gerek kalmadan tanımlanabilir (yükseklik-sınırlı trust kayıtları).

**Hedef değil (bu RFC):**
- Snapshot içeriğinin zincir-üstü consensus'a bağlanması (state-root anchoring) — ayrı tasarım, finality katmanı işi.
- GAP-2'nin alan-bazlı ayrıntı tasarımı (halef-koordineli; bu RFC yalnızca "tek schema-4" birleşimini önerir).

## 4. Tasarım

### 4.1 İmzalanan baytlar

İmza, `calculate_hash()` çıktısının **ham 32 baytı** üzerindedir (hex string değil):

```
digest = Sha3-256(canonical fields...)   // mevcut calculate_hash gövdesi, byte-array'e çevrilir
sig    = Ed25519_sign(sk_snapshot, digest)
```

Mevcut `calculate_hash` hex-String dönüyor; RFC implementasyonunda iç gövde `fn calculate_digest(&self) -> [u8; 32]`'e ayrıştırılır, `calculate_hash` = `hex::encode(digest)` olarak kalır (mevcut kayıtlı format bozulmaz).

GAP-2 kapsam genişletmesi (tek schema-4 birleşimi kabul edilirse) digest hesabına yeni alanları **sıralı, versiyonlu** ekler: `SCHEMA4_HASH_DOMAIN = "budlum.snapshot.v4"` gibi domain-separation prefix'i ile başlar — schema-3 hesabıyla bayt düzeyinde ayrışır, cross-schema confusion imkânsızlaşır.

### 4.2 Wire formatı (schema_version = 4)

```rust
/// schema_version 4 (GAP-1): manifest authenticity.
#[serde(default)]
pub manifest_signer: Option<[u8; 32]>,      // Ed25519 açık anahtarı
#[serde(default)]
pub manifest_signature: Option<Vec<u8>>,    // 64B Ed25519 imzası (digest üzerinde)
```

- `Option` + `serde(default)`: schema ≤3 dosyaları deserialize olur; imza alanları `None` gelir → politika kararına düşer (§4.4).
- İmza hesabı yapılırken bu iki alan digest'e **dahil edilmez** (self-reference engeli); `calculate_digest` imza alanlarını hiç görmez.

### 4.3 Trust modeli — üç seçenek ve öneri

| Model | Tanım | Güç | Zayıflık |
|---|---|---|---|
| **A. Self-signed** | Node kendi ürettiği snapshot'ı kendi anahtarıyla imzalar; yalnızca kendi açık anahtarına güvenir | Corruption + "bana ait olmayan dosya" ayrımı | Host compromise'da saldırgan dosyayı değiştirip **yeniden imzalayabilir** (anahtar diskteyse); restore/migration senaryosunu kapsamaz |
| **B. Quorum imzası** | Snapshot digest'i validator quorum'u (≥2/3) tarafından imzalanır — BLS aggregate adayı (`Validator.bls_public_key` zaten manifest'te) | Host-key compromise tek başına yetersiz; P2P state-sync için doğru temel | Finality katmanıyla anahtar-rotasyonu eşlemesi gerekir; implementasyon hacmi büyük; halefin chain-domain'iyle kesişir |
| **C. Hibrit fazlı (ÖNERİLEN)** | **Faz 1 (şimdi):** Ed25519 tek-imza + politika bazlı trust-list (yükseklik-sınırlı). **Faz 2 (mainnet state-sync öncesi):** B seçeneği quorum'a genişleme — trust-list şeması `QuorumRule`'a evrilir | H1-H4'ü bugün karşılar; B'ye kırmadan geçiş | Faz-1'de trust-list dağıtım kanalı operatöre yükler (§4.5 mitigasyonu) |

**Öneri: C — Faz 1.** Gerekçe: (i) mevcut saldırı yüzeyi **tek host'a yazan saldırgan**; HSM-arkalı Ed25519 tek-imza bu sınıfı kapatır (imzalama anahtarı diske hiç çıkmaz — pkcs11 seam'i mevcut); (ii) B'nin finality entegrasyonu halef devreye girmeden sorumluluk sınırını aşar (ARENA3 kripto domain'i, chain/consensus değil); (iii) wire formatı QuorumRule'a `Option` alanıyla kırılmadan genişler.

### 4.4 Doğrulama politikası

```rust
pub enum SnapshotTrustPolicy {
    /// Devnet/dev: imzasız kabul (bugünkü davranış; boot log WARN).
    AllowUnsigned,
    /// Mainnet/varsayılan-güvenli: imza zorunlu + imzalayan trust-list'te
    /// ve snapshot.height anahtarın geçerlilik aralığında olmalı.
    RequireTrusted { keys: Vec<TrustedSnapshotKey> },
    /// Faz 2 hazırlığı (bu RFC'de yalnızca şema): quorum doğrulama.
    RequireQuorum { /* halef+ARENA3 ortak tasarım */ },
}

pub struct TrustedSnapshotKey {
    pub pubkey: [u8; 32],
    pub valid_from_height: u64,
    pub valid_until_height: Option<u64>,   // None = açık uç
    pub label: String,                     // audit kolaylığı ("genesis-ceremony-1")
}
```

Yeni doğrulama API'si (mevcut `verify()` **değişmez** — integrity katmanı olarak kalır):

```rust
pub enum SnapshotAuthError {
    Unsigned,            // politika RequireTrusted iken manifest_signature None
    UntrustedSigner,     // imzalayan trust-list dışı
    OutOfValidityRange,  // snapshot.height anahtar aralığı dışında (rotasyon sınırı)
    BadSignature,        // ed25519 verify fail
    IntegrityMismatch,   // rehash uyuşmazlığı (mevcut verify() == false)
}

pub fn verify_authentic(&self, policy: &SnapshotTrustPolicy) -> Result<(), SnapshotAuthError>
```

Sıra önemli: önce `verify()` (integrity), sonra imza. İmza bozuksa **karantina sınıfı: AuthFailure** (GAP-3 döngüsüyle aynı muamele — `loader`, `Err` durumunda sonraki adaya geçer; `quarantined_any` bayrağı boot'ta fail-loud loglanır).

### 4.5 Anahtar yönetimi

**İmzalama anahtarı nerede:**
- Node başına ayrı snapshot-imzalama anahtarı (`--snapshot-signing-key` / `BUD_SNAPSHOT_SIGNING_KEY`), consensus anahtarından **türev değil, bağımsız** (consensus-key compromise'ı snapshot zincirine sıçramasın).
- Üretim: `budlum keygen --snapshot` (Ed25519) veya HSM etiketi (`--hsm --key-label=snapshot-manifest`); PKCS#11 backend `ConsensusSigner` arkasında zaten Ed25519-only doğrulanmış (signer.rs §Phase 2 notu).
- Disk'e anahtar yazılmaz; HSM tercih edilen mainnet yolu (ARENA3 HSM domain'i).

**Trust-list nereden gelir (öncelik sırası):**
1. CLI/config: `--snapshot-trust-key=<hex pub>[:from=H][:until=H]` (tekrarlanabilir) — operatör kontrolü, tören dostu.
2. Genesis bundle: `genesis.json` içine `snapshot_trust: [...]` (MAINNET_GENESIS_CEREMONY.md akışına ek — **kullanıcı kararı gerekir**, Açık Soru 2).
3. Hiçbiri yoksa: `RequireTrusted` boot'u reddeder (fail-closed; yanlış yapılandırmayla sessiz imzasız kabul **yok**).

**Rotasyon:** yeni anahtar `valid_from_height = H` ile eklenir; eski anahtar `valid_until_height = H-1` yapılır. Loader, snapshot'ın kendi `height`'ına göre uygun kaydı seçer → eski arşiv snapshot'ları eski anahtarla doğrulanmaya devam eder, yeni üretimler yeni anahtarla. Hard-fork gerekmez.

**Devnet politikası:** `AllowUnsigned` (mevcut davranışı bozmaz; chaos/upgrade matrisleri imza altyapısı olmadan koşmaya devam eder). CI'da RequireTrusted yolu ayrı bir boot-testiyle kilitlenir.

### 4.6 P2P state-sync'e uzatma noktası (Faz-2 zemini)

`manifest_signer` + trust-list şeması, Faz-2'de şuna evrilir: digest + BLS aggregate signature + signer-bitset (finality certificate benzeri). Wire formatına `manifest_quorum: Option<...>` eklemek schema-4'ü kırmaz (`Option` + `serde(default)`). Bu RFC Faz-2 API'sini **dondurmaz**, yalnızca genişleme noktası bırakır.

## 5. Boot akışı (Faz-1 sonrası)

```
load_latest_snapshot_v2(policy):
  for candidate in sorted_candidates:
      dosya → parse → schema gate (mevcut)
      verify() == false            → quarantine(IntegrityMismatch)  [mevcut, GAP-3]
      verify_authentic(policy) Err → quarantine(AuthFailure)       [YENİ]
      quarantined_any = true
  aday tükendiyse:
      policy AllowUnsigned ve bulunan imzasız schema≤3 → WARN + kabul (devnet)
      aksi halde Err → boot fail-loud (mevcut, GAP-3 davranışı)
```

## 6. Test planı

**Pin dönüşümü (GAP-1 kapanış kanıtı):**
- `test_snapshot_v2_rehash_forgery_no_authenticity_gap`: RequireTrusted politikası altında rehash forgery artık `Err(BadSignature)`/`Err(Unsigned)` → reddedildiğini doğrular. (Test adı `_gap` suffix'inden `_enforced`'a evrilir veya davranış assert'i terslenir — STATUS_ONLINE'da ilan edilir.)

**Yeni negatifler:**
- unsigned + RequireTrusted → `Unsigned` (karantina)
- yanlış trust key (self-signed saldırgan anahtarı) → `UntrustedSigner`
- doğru anahtar ama `height` aralık dışı (rotasyon senaryosu) → `OutOfValidityRange`
- geçerli imza + alan değiştirme (rehash'siz, GAP-2 sınıfı alanlarda — schema-4 kapsamıyla birlikte `IntegrityMismatch`)
- rotasyon: eski snapshot eski anahtarla OK, yeni snapshot eski anahtarla reject

**Pozitifler:** KeyPairSigner uçtan-uca round-trip; HSM-mock backend yolu (pkcs11 test seam'i — mock signer `ConsensusSigner` arkasında aynı digest'i imzalar); AllowUnsigned devnet round-trip.

**Çapraz:** multisig/chaos matrislerinde mevcut snapshot testleri kırılmamalı (AllowUnsigned default'uyla) — CI yeşil kalması uyumluluk kanıtı.

## 7. Açık sorular (kullanıcı/halef kararı gerektirir)

1. **Trust model onayı:** Faz-1 = C-hibrit önerisi kabul mü? (B-quorum'u ileri çekmek isterseniz Faz-2 tasarımı halef gelince ortak başlar.)
2. **Trust-list kanalı:** genesis bundle'a `snapshot_trust` eklensin mi (ceremony dokümanına işlenir) yoksa yalnız CLI/config mi kalsın? Öneri: ikisi birden (CLI override genesis'i ezer — operatör kurtarma senaryosu).
3. **İmzasız geçiş penceresi:** `AllowUnsigned` devnet-dışında hiç var olmasın mı, yoksa upgrade süresi boyunca uyarılı "legacy-import" modu mu tanınsın? Öneri: devnet-only; production build'de `AllowUnsigned` derleme-uyarısı üretsin.
4. **GAP-2 birleşimi:** hash-kapsam genişletmesi schema-4'e TEK sürümde mi, yoksa kendi schema-5'iyle mi? **Öneri: tek schema-4** (iki kırıcı wire-değişikliğini tek migrasyon penceresinde toplar; `--migrate-v2` hook'u tek geçişte schema 2/3→4 üretir). Bu kalemin alan listesi halefle kesinleşir (chain domain'i: `calculate_hash` sahipliği ARENA2-halefte, doğrulama/imza ARENA3'te).

## 8. Uygulama planı (onay sonrası)

| PR | Kapsam | Sorumlu |
|---|---|---|
| P1 | `calculate_digest` ayrıştırma + schema-4 wire alanları + `SnapshotTrustPolicy`/`verify_authentic` + unit testler | ARENA3 |
| P2 | Loader/boot entegrasyonu (karantina sınıfı AuthFailure, CLI flag'leri, policy wiring) | ARENA3 |
| P3 | GAP-2 kapsam genişletme (schema-4 digest alan listesi, domain prefix) | ARENA2-halef + ARENA3 (ortak) |
| P4 | Pin dönüşümü + yeni negatif/rotasyon/HSM-mock chaos testleri | ARENA3 |

Her PR ayrı CI-doğrulamalı; P3 halef devreye girene dek bekler (P1+P2, schema-4'ü imza alanlarıyla sınırlı olarak tek başına anlamlıdır — GAP-2 alanları `serde(default)` ile sonradan eklenir).

---

*Co-authored-by: ARENA3 <arena3@budlum.xyz>*
