# BudZKVM Trace Layout — `TRACE_WIDTH = 414`

Bu doküman `budzero/bud-proof/src/plonky3_air.rs` içindeki STARK trace
sütunlarının atanmasını, her aralığın amacını ve bilinçli olarak
boş bırakılan yerleri açıklar.

> **Kapsam:** BudZero/BudZKVM AIR (Plonky3 0.5.2, Goldilocks field).  
> **Hedef:** Gelecekte sütun ekleneceğinde çakışma riskini sıfıra
> indirmek; `TRACE_WIDTH` artırılmadan önce bu tabloya dokunulması
> zorunludur.

---

## 1. Özet

| Aralık | Sütunlar | Amaç |
|--------|----------|------|
| 0 – 10 | 11 | CPU çekirdek (clk, pc, opcode, register index/values, next_pc, imm) |
| 11 – 22 | 12 | İlk opcode selektörleri (Add..Log, JnzCond) |
| 23 – 28 | 6 | Register bus (register_events CTL) |
| 29 – 48 | 20 | Genişletilmiş opcode selektörleri (Div..VerifyMerkle) |
| 49 – 56 | 8 | Memory bus + stack pointer + register sub-clk |
| 57 – 64 | 8 | Soundness/public-input yardımcı sütunlar |
| 65 – 128 | 64 | Karşılaştırma/bitwise: rs1 bit ayrışımı |
| 129 – 192 | 64 | Karşılaştırma/bitwise: rs2 bit ayrışımı |
| 193 – 256 | 64 | Karşılaştırma: eşitlik önek bayrakları |
| 257 | 1 | Karşılaştırma: ham less-than sonucu |
| 258 – 289 | 32 | Poseidon durum sütunları (4 round × 8 eleman) |
| 290 – 321 | 32 | Poseidon x^2 ara değerleri |
| 322 – 353 | 32 | Poseidon x^4 ara değerleri |
| 354 – 361 | 8 | Public-input: final state root |
| 362 – 369 | 8 | Public-input: initial state root |
| 370 – 377 | 8 | **Bilinçli boşluk** — gelecek genişleme için ayrıldı |
| 378 | 1 | Public-input: trace_len sayacı |
| 379 | 1 | Public-input: gas_limit |
| 380 – 387 | 8 | Public-input: event digest accumulator |
| 388 | 1 | Public-input: exit_code |
| 389 | 1 | Public-input: chain_id |
| 390 – 395 | 6 | VerifyMerkle path expansion (key, bit, current, sibling, round, is_expand) |
| 396 – 403 | 8 | VerifyMerkle Poseidon x^2 (8 eleman) |
| 404 – 411 | 8 | VerifyMerkle Poseidon x^4 (8 eleman) |
| 412 | 1 | VerifyMerkle final root diff inv |
| 413 | 1 | VerifyMerkle final flag |
| **Toplam** | **414** | `TRACE_WIDTH` |

---

## 2. Kritik Kısıtlar

1. **Selector booleanity:** Tüm opcode selektörleri (`COL_IS_*`) `0/1`
   değerli olmalı; toplamları `is_cpu = 1` olmalı.
2. **Register/Memory CTL:** `COL_REG_*` ve `COL_MEM_*` LogUp bus terimleri
   `COL_CLK`, `COL_PC` gibi çekirdek sütunlara bağlıdır.
3. **Poseidon + VerifyMerkle RC/MDS:** `COL_POSEIDON_*` ve
   `COL_MERKLE_POSEIDON_*` aynı round-constant/MDS matrisini kullanır.
   Tekilleştirme planı için `docs/BUDZKVM_POSEIDON_REFACTOR.md` (Paket C)
   takip edilecektir.
4. **Bilinçli boşluk (370–377):** Şu an hiçbir kısıt tarafından
   kullanılmıyor. Yeni public-input veya genişleme ihtiyacı olduğunda
   önce bu aralık doldurulmalı; yetersiz kalırsa `TRACE_WIDTH`
   artırılmalı.
5. **TRACE_WIDTH sınırı:** Son atanmış sütun `COL_MERKLE_FINAL_FLAG = 413`.
   `TRACE_WIDTH = 414` olmak zorundadır.

---

## 3. Değişiklik Rehberi

Yeni bir sütun eklerken:

1. Bu tabloyu güncelleyin.
2. `TRACE_WIDTH` değerini değiştirmeyin, yeni sütun mevcut boşluklara
   veya sonuna eklenmeli.
3. `budzero/bud-proof/src/trace_layout_tests.rs` içindeki aralık
   listesini güncelleyin; test otomatik olarak çakışma ve sınır
   kontrolü yapar.
4. `cargo test -p bud-proof trace_layout` CI'da yeşil olmadan push
   yapmayın.

---

## 4. İlişkili Dosyalar

- `budzero/bud-proof/src/plonky3_air.rs` — AIR tanımları ve sütun const'ları
- `budzero/bud-proof/src/trace_layout_tests.rs` — sütun bütünlüğü testleri
- `budzero/bud-vm/src/lib.rs` — VM trace popülasyonu
