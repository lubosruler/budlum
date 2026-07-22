# budl — Budlum Programlama Dili

**budl** (Budlum language), BudZKVM (zero-knowledge sanal makinesi) üzerinde çalışan,
STARK-provable, deterministik, akıllı-kontrat ve genel amaçlı programlama dilidir.

> **Sürüm:** v0.1 (2026-07-22) · **Sahip:** ARENA3
> **Konum:** `budZero/budl/`
> **Statü:** Dokümantasyon + örnekler (cargo workspace üyesi değildir)

---

## 1. Bu klasör ne içerir?

```
budZero/budl/
├── README.md       ← bu dosya — klasör haritası, giriş, linkler
├── SPEC.md         ← dilin grameri, tipleri, semantiği (EBNF + tip tablosu)
├── OPCODES.md      ← bud-isa opcode seti + budl dil seviyesi karşılığı (35 opcode, 0x00–0x22)
├── PIPELINE.md     ← budl → bytecode → execution trace → STARK proof → konsensüs akışı
└── examples/       ← çalışan budl örnekleri
    ├── example.bud
    ├── example2.bud
    ├── example_loop.bud
    ├── test_prover.bud
    └── control_flow.bud
```

**Karar:** bu klasör **dokümantasyon + örnekler** topluluğudur. `budZero/Cargo.toml`
workspace member'ı **değildir** — derleme katmanına dahil değildir. Derleyici, VM
ve kanıt motoru kendi crate'lerinde yaşar.

---

## 2. Pipeline (5 Aşama)

```
.bud source  →  bud-compiler  →  bud-isa bytecode (Vec<u64>)
            →  bud-vm        →  ExecutionReceipt + ExecutionTrace
            →  bud-proof     →  ProofEnvelope (Plonky3 STARK)
            →  budlum-core   →  state root güncellemesi (konsensüs)
```

Tam açıklama: [`PIPELINE.md`](./PIPELINE.md).

---

## 3. budl Diline Hızlı Bakış

Bir `budl` programı `.bud` uzantılı tek dosya olarak yazılır:

```budl
contract Token {
    storage {
        total_supply: u64,
    }

    pub fn mint(to: Address, amount: u64) {
        let caller = sys_caller();           // Syscall imm=1
        swrite(to, sread(to) + amount);
        emit Mint(to, amount);
    }

    pub fn balance_of(addr: Address) -> u64 {
        return sread(addr);
    }
}
```

Dil özellikleri (özet):

- `contract` — program sarmalayıcısı.
- `pub fn` / `fn` — fonksiyonlar (parametreler + dönüş tipi opsiyonel).
- `let` — yerel değişken; `=` — atama.
- `if` / `else` / `while` / `for ... in ... .. ...` — kontrol akışı.
- `storage { ... }` — kalıcı durum (slot-keyed).
- `struct` — kullanıcı tanımlı kompozit tip.
- `sread(key)` / `swrite(key, val)` — storage okuma/yazma.
- `emit Event(args)` — olay yayını.
- `constrain(cond)` — assertion (`Assert` opcode).
- `match scrutinee { pat => body, _ => default }` — exhaustive pattern match.
- `assert!(cond)` — macro benzeri shorthand (compile-time expand).

Tam gramer, tipler, semantik: [`SPEC.md`](./SPEC.md).
Opcode eşlemesi: [`OPCODES.md`](./OPCODES.md).

---

## 4. Örnekler (`examples/`)

5 çalışan örnek — derle, çalıştır, kanıtla, doğrula.

| Dosya | Ne Demonstrasyon Yapar | Opcode'lar |
|---|---|---|
| `example.bud` | Tüm aritmetik (Add, Sub, Mul), karşılaştırma (Eq, Gt), 5 kez `emit` | Add, Sub, Mul, Eq, Gt, Load, Log |
| `example2.bud` | Minimum program: `constrain(a + b == 3)` | Add, Load, Eq, Assert |
| `example_loop.bud` | `for i in 0..5` + `while (count < 4)` + `if/else` + 2 emit | Lt, Jnz, Jmp, Add, Log |
| `test_prover.bud` | En küçük STARK-provable: `a + b == c` constrain | Add, Load, Eq, Assert |
| `control_flow.bud` | `while` + `if/else` ile Success/Failure emit | Lt, Jnz, Jmp, Add, Eq, Log |

Çalıştırmak için (kök repo):

```bash
cd budZero
cargo run --bin bud -- run -p budl/examples/example.bud
```

---

## 5. Uygulama Konumu

budl programlama dilinin kendisi bu klasörde **yalnız dokümante edilir**.
Gerçek uygulama parçaları budZero ekosisteminin farklı crate'lerinde yaşar:

| Crate | Yol | Sorumluluk |
|---|---|---|
| `bud-compiler` | `budZero/bud-compiler/` | Lexer + Parser + AST + Sema + Codegen (derleyici) |
| `bud-isa` | `budZero/bud-isa/` | Opcode enum + Instruction encode/decode + MainnetActivation |
| `bud-vm` | `budZero/bud-vm/` | Register + memory VM, execute, gas, syscall |
| `bud-proof` | `budZero/bud-proof/` | Plonky3 STARK AIR + Prover + Verifier |
| `bud-cli` | `budZero/bud-cli/` | `bud` komut satırı aracı (end-to-end pipeline) |
| `bud-state` | `budZero/bud-state/` | (Planlanan) State commitment'leri |
| `bud-node` | `budZero/bud-node/` | (Planlanan) budl tabanlı düğüm davranışı |

---

## 6. Opcode Özeti (Tam Liste)

35 opcode, 0x00 – 0x22. Kategoriler:

- **Kontrol akışı (5):** `Halt 0x00`, `Jmp 0x10`, `Jnz 0x11`, `Call 0x12`, `Ret 0x13`
- **Aritmetik (5):** `Add 0x01`, `Sub 0x02`, `Mul 0x03`, `Div 0x04`, `Inv 0x05`
- **Bitsel/lojik (4):** `And 0x06`, `Or 0x07`, `Xor 0x08`, `Not 0x09`
- **Karşılaştırma (6):** `Eq 0x0A`, `Neq 0x0B`, `Lt 0x0C`, `Gt 0x0D`, `Lte 0x0E`, `Gte 0x0F`
- **Bellek/yığın (4):** `Load 0x14`, `Store 0x15`, `Push 0x16`, `Pop 0x17`
- **Doğrulama (2):** `Assert 0x18`, `Poseidon 0x19`
- **Depolama/olay (3):** `Log 0x1A`, `SRead 0x1B`, `SWrite 0x1C`
- **Sistem (1):** `Syscall 0x1D`
- **Kriptografik doğrulama (2):** `VerifyMerkle 0x1E`, `VerifyInference 0x1F`
- **Gizlilik katmanı D2 (3):** `PrivacyCommit 0x20`, `NullifierCheck 0x21`, `SumConservation 0x22`

**Mainnet-gated (5):** `VerifyMerkle`, `VerifyInference`, `PrivacyCommit`, `NullifierCheck`, `SumConservation` — `MainnetActivation::full()` ile post-ceremony etkinleştirilir. Varsayılan `default()` reddetme.

Tam opcode tablosu, encode şeması, gas maliyetleri, codegen kalıpları: [`OPCODES.md`](./OPCODES.md).

---

## 7. Bilinçli Sınırlamalar (v0.1)

- **Float / ondalık:** Yok. Tüm aritmetik Goldilocks alanı (p = 2⁶⁴ − 2³² + 1).
- **String:** Yok. Bytes `u64` dilimleri.
- **Dinamik dispatch / trait / generics:** Yok.
- **Çoklu dosya / modül:** Yok. Tek contract dosyası.
- **Standart kütüphane:** Planlama aşamasında (`caller()`, `block_height()` gibi syscall'lar VM'de mevcut; dil düzeyinde çağrı sözdizimi kısmi).

---

## 8. İlgili Dokümanlar (budZero kökü)

- `budZero/README.md` — budZero ekosistemi giriş
- `budZero/ARCHITECTURE.md` — budZero mimari genel bakış
- `budZero/docs/BudL_SPEC.md` — ARENA1'in eski spesifikasyonu (kapsam, gas, storage)
- `budZero/docs/02_isa_ve_bytecode.md` — ISA ve bytecode detayları
- `budZero/docs/03_virtual_machine.md` — VM mimarisi
- `budZero/docs/05_stark_ve_plonky3.md` — STARK ve Plonky3
- `budZero/docs/06_compiler_ve_ekosistem.md` — Derleyici ve ekosistem
- `budZero/docs/adding_opcodes.md` — Yeni opcode ekleme rehberi

---

## 9. Katkı

- **Yeni opcode:** `budZero/docs/adding_opcodes.md` rehberini izle; `bud-isa` + `bud-vm` + `bud-compiler/codegen.rs` üçlüsünü güncelle.
- **Yeni örnek:** `budZero/budl/examples/` altına `.bud` dosyası ekle. Mevcut örnekleri şablon olarak kullan; kısa yorum + gösterdiği opcode'ları dosya başında belgelendir.
- **Dokümantasyon güncelleme:** `SPEC.md` dilbilgisi, `OPCODES.md` opcode tablosu, `PIPELINE.md` akış değişikliği.

---

*Co-authored-by: ARENA3 <arena3@budlum.xyz>*
