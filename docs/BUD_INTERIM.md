# B.U.D. Interim Retrieval Challenge — Mainnet v1 Durumu

**Yazar:** ARENA2 (ADIM3 §3.6 görevi)
**Tarih:** 2026-07-15
**Kaynak:** `BUD_Merkeziyetsiz_Depolama_Vizyonu.md` §8.3, §8.5, §9; `src/domain/storage_deal.rs`; ADIM3 plan §3.6

---

## ⚠️ Özet: B.U.D. Mainnet v1'de "Gerçek Proof-of-Storage" DEĞİLDİR

B.U.D. (Broad Universal Database) Mainnet v1'de **interim retrieval challenge** (geçici erişim sorgulaması) mekanizmasıyla çalışmaktadır. Bu mekanizma:

- ✅ Operatörün **belirli bir byte aralığına** yanıt verip veremediğini test eder
- ✅ Yanıt veremeyen operatörün bond'unu **slasher** (ekonomik caydırıcılık)
- ✅ Permissionless katılım sağlar (whitelist YOK, admin gate YOK)
- ❌ Operatörün **tüm veriyi sürekli sakladığını** kanıtlamaz
- ❌ **Proof-of-Replication** veya **Proof-of-Spacetime** sunmaz
- ❌ Veri bütünlüğünü kriptografik olarak **garanti etmez**

**Kullanıcı beklenti yönetimi:** B.U.D. Mainnet v1'de bir **ekonomik oyun** (game-theoretic deterrent) olarak çalışır. Operatörler veri saklamaya ekonomik olarak teşvik edilir, ancak kriptografik kanıt henüz mevcut değildir. Gerçek Proof-of-Storage, BudZero `VerifyMerkle` opcode'unun production gate'i geçmesiyle (Faz 3) mümkün olacaktır.

---

## 1. Mevcut Mekanizma: Interim Retrieval Challenge

### 1.1 Nasıl Çalışır

```
İstemci                    Zincir (L1)                  Operatör
  │                           │                            │
  ├── storage_open_deal ─────>│                            │
  │   (manifest, shard,       ├── DealStatus::Active ─────>│
  │    operator_bond)         │   (bond kilitlendi)        │
  │                           │                            │
  │   [challenge_interval     │                            │
  │    blok sonra]            │                            │
  │                           │                            │
  ├── storage_open_challenge ─>│                           │
  │   (deal_id, byte_start,   ├── Challenge açık ─────────>│
  │    byte_end, deadline)    │   (deadline = epoch + 10)  │
  │                           │                            │
  │                           │<── storage_answer ─────────┤
  │                           │   (chunk_data_hash)        │
  │                           │                            │
  │                           ├── Outcome::Answered ───────┤
  │                           │   (bond iade)              │
  │                           │                            │
  │   [deadline geçerse]      │                            │
  │                           ├── Outcome::Missed ─────────┤
  │                           │   (bond slashed!)          │
```

### 1.2 Kod Referansları

| Bileşen | Dosya | Açıklama |
|---------|-------|----------|
| `StorageDeal` | `src/domain/storage_deal.rs` | Deal yaşam döngüsü (Active → Slashed/Expired) |
| `RetrievalChallenge` | `src/domain/storage_deal.rs:120` | Challenge yapısı (byte range, deadline) |
| `StorageRegistry` | `src/domain/storage_deal.rs:189` | On-chain deal/challenge registry |
| `ChainCommand::IssueStorageChallenges` | `src/chain/chain_actor.rs` | Otomatik challenge üretimi |
| `ChainCommand::FinalizeMissedStorageChallenges` | `src/chain/chain_actor.rs` | Slash mekanizması |
| `bud_storageOpenDeal` | `src/rpc/server.rs` | RPC endpoint |
| `bud_storageOpenChallenge` | `src/rpc/server.rs` | RPC endpoint |
| `bud_storageAnswerChallenge` | `src/rpc/server.rs` | RPC endpoint |

### 1.3 Ekonomik Parametreler

```rust
pub struct StorageEconomicsParams {
    pub operator_bond: u64,     // Operatörün kilitlediği teminat
    pub fee_per_epoch: u64,     // İstemciden operatöre epoch başına ücret
}
```

- **Bond:** Operatör deal açıldığında bond yatırır. Challenge'ı kaçırırsa bond slash edilir.
- **Fee:** İstemci, operatöre epoch başına ücret öder.
- **Slash oranı:** `StorageDeal::slashed_bond` — kaçırılan challenge başına bond'un tamamı kesilir.

---

## 2. Bilinen Sınırlamalar

### 2.1 "Kısmi Veri Silme" Açığı (Vision §9.2)

Bir operatör manifest'in büyük kısmını silip yalnızca challenge'da sorgulanacak byte aralığını tutabilir. Challenge'lar deterministik olduğu için (`deal_id` ve `epoch`'tan türetilir), operatör hangi aralığın sorgulanacağını önceden hesaplayabilir.

**Mitigasyon:** Challenge byte aralığı `deal_id * 17 ^ epoch` formülüyle pseudo-random türetilir. Ancak bu gerçek bir Proof-of-Storage yerine geçmez.

### 2.2 Veri Dışsallaştırma (Outsourcing) Açığı (Vision §9.1)

Operatör veriyi gerçekten saklamak yerine, challenge anında komşu bir node'dan ilgili byte aralığını çekip yanıt verebilir. `RESPONSE_WINDOW_BLOCKS` (10 epoch) yeterince genişse bu ekonomik olarak kârlı olabilir.

**Mitigasyon:** Deadline 10 epoch ile sınırlıdır. Ancak gerçek PoRep/PoSt olmadan bu tamamen engellenemez.

### 2.3 Seed Grinding Riski (Vision §9.3)

Challenge seed'i `global_block_hash`'e bağlıdır. Aynı taraf hem blok üretici hem depolama operatörüyse, kendi challenge'ını dolaylı etkileyebilir.

**Mitigasyon:** Mevcut tasarımda bu risk kabul edilmiştir. Faz 3'te VRF/commit-reveal eklenecektir.

---

## 3. Gerçek Proof-of-Storage'a Giden Yol (Faz 3)

### 3.1 Gate: BudZero VerifyMerkle Z-B Commit 3.5

Gerçek Proof-of-Storage, BudZKVM'in `VerifyMerkle` opcode'unun (0x1E) production gate'i geçmesine bağlıdır. Bu opcode:

- 64-derinlikte Poseidon4 tabanlı Merkle proof doğrulaması yapar
- STARK prover tarafından doğrulanabilir
- `GlobalBlockHeader.storage_root`'a bağlanabilir

**Mevcut durum:** `proves_verify_merkle_valid_64_depth` testi `#[ignore]` durumundadır. ARENA2 prover'da `wrapping_add → u128` modüler aritmetik hatasını buldu ve düzeltti, ancak AIR constraint tarafında ek uyumsuzluklar bulunmaktadır.

### 3.2 Faz 3 Tamamlandığında Ne Değişir

| Özellik | Interim (şimdi) | Faz 3 sonrası |
|---------|-----------------|---------------|
| Kanıt tipi | Byte range yanıtı | STARK-aggregated Merkle proof |
| Doğrulama | Zincir üstü basit hash | BudZKVM STARK verifier |
| Kapsam | Tek byte aralığı | Tüm chunk (256 KiB) |
| Süreklilik | Anlık erişim | Sürekli saklama kanıtı |
| Güvenlik | Ekonomik caydırıcılık | Kriptografik garanti |

---

## 4. Karar Kayıtları

| Karar | Kaynak | Durum |
|-------|--------|-------|
| B.U.D. mainnet'e dahil (interim retrieval ile) | Kullanıcı kararı 2.3=A | ✅ Aktif |
| Faz 3 VerifyMerkle gate'e bağımlı | `MAINNET_READINESS.md` §2.1 | ⏳ Beklemede |
| Mock HSM kaldırıldı, sadece gerçek PKCS#11 | `AI_BIRLIGI.md` §5 | ✅ Kesin |
| StorageAttestationFinalityAdapter gerçek cert.verify() | ARENA2 `49b6b46` | ✅ Tamamlandı |

---

## 5. Operatörler İçin Pratik Rehber

### 5.1 Deal Açma

```bash
# 1. Storage operator olarak kayıt ol (opsiyonel, ödül için gerekli)
budlum-cli register --role STORAGE_OPERATOR --stake 1000000

# 2. Deal aç (manifest, shard, bond)
budlum-cli storage open-deal \
  --manifest-id 0xabc... \
  --shard-id 0xdef... \
  --operator-bond 500000 \
  --fee-per-epoch 100
```

### 5.2 Challenge Yanıtlama

```bash
# Challenge'ları izle
budlum-cli storage get-challenges --operator <adres>

# Challenge'a yanıt ver (byte aralığı verisi gerekli)
budlum-cli storage answer-challenge \
  --challenge-id 42 \
  --chunk-data-hash 0x123...
```

### 5.3 Risk Uyarısı

- Challenge'ı kaçırmak = **bond'un tamamı slash edilir**
- Veriyi saklamadan challenge'ları yanıtlamak teorik olarak mümkündür ama ekonomik olarak sürdürülebilir değildir
- Faz 3 tamamlandığında gerçek Proof-of-Storage gerekecektir — hazırlıklı olun

---

**Son güncelleme:** 2026-07-15 (ARENA2, commit `49b6b46`)
