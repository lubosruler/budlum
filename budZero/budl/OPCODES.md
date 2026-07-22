# budl Opcode Mapping

> budl dilindeki ifadelerin BudZKVM ISA opcode'larına nasıl derlendiğini tanımlar.
> Kaynak: `budZero/bud-isa/src/lib.rs` (Opcode enum + Instruction encode/decode) +
> `budZero/bud-compiler/src/codegen.rs` (AST → bytecode) +
> `budZero/bud-vm/src/lib.rs` (execute, gas).
>
> **Sürüm:** v0.1 (2026-07-22) · **Sahip:** ARENA3

---

## 1. Komut Formatı (Instruction Encoding)

Her instruction 64-bit (`u64`) little-endian kodlanmış `Instruction` yapısıdır:

```rust
pub struct Instruction {
    pub opcode: Opcode,  //  8 bit (0x00–0xFF, bizde 0x00–0x22)
    pub rd:     u8,      //  5 bit (bits 8–12)   — destination register (0..=31)
    pub rs1:    u8,      //  5 bit (bits 13–17)  — source 1 register
    pub rs2:    u8,      //  5 bit (bits 18–22)  — source 2 register
    pub imm:    i32,     // 32 bit (bits 23–54)  — immediate (sign-extended)
}
```

**Bit düzeni** (encode):
```
bits  0..=7   : opcode
bits  8..=12  : rd
bits 13..=17  : rs1
bits 18..=22  : rs2
bits 23..=54  : imm (i32, sign-extended)
bits 55..=63  : reserved (0)
```

Kullanılmayan üst bitler şu an 0. `Instruction::encode()` ve `decode_any()` her zaman aynı şemayı uygular.

**Register modeli:**
- `r0` — zero register (her zaman 0)
- `r1..=r30` — genel amaçlı
- `r31` — heap pointer (başlangıçta 4096, sözleşme girişinde `Load 31, 0, 0, 4096` ile set edilir)

---

## 2. Opcode Tablosu (Tam Liste)

### 2.1 Kontrol Akışı

| Opcode | Hex | Semantik | budl karşılığı | Gas | Encoding örneği |
|---|---|---|---|---|---|
| `Halt` | `0x00` | Programı sonlandır. | `}` (fn/contract sonu) | 0 | `Halt 0,0,0,0` |
| `Jmp` | `0x10` | `pc += imm` (göreli sıçrama). | `while`, `for`, `if/else` (forward/backward) | 2 | `Jmp 0,0,0,±N` |
| `Jnz` | `0x11` | `if rs1 != 0 { pc += imm }` | `if (cond) { ... }`, `while (cond)` test | 2 | `Jnz 0,cond,0,±N` |
| `Call` | `0x12` | Push return_pc, `pc += imm`. | `fn(arg)` çağrısı | 5 | `Call 0,0,0,fn_offset` |
| `Ret` | `0x13` | Pop return_pc, ona sıçra. | `return;` | 5 | `Ret 0,0,0,0` |

### 2.2 Aritmetik (Goldilocks field p = 2⁶⁴ − 2³² + 1)

| Opcode | Hex | Semantik | budl karşılığı | Gas |
|---|---|---|---|---|
| `Add` | `0x01` | `rd = (rs1 + rs2) mod p` | `a + b` | 1 |
| `Sub` | `0x02` | `rd = (rs1 − rs2) mod p` | `a - b` | 1 |
| `Mul` | `0x03` | `rd = (rs1 × rs2) mod p` | `a * b` | 3 |
| `Div` | `0x04` | `rd = rs1 × rs2⁻¹ mod p` (rs2 ≠ 0) | `a / b` | 10 |
| `Inv` | `0x05` | `rd = rs1⁻¹ mod p` (rs1 ≠ 0) | `1 / a` (alan tersi) | 50 |

### 2.3 Bitsel / Lojik

| Opcode | Hex | Semantik | budl karşılığı | Gas |
|---|---|---|---|---|
| `And` | `0x06` | `rd = rs1 & rs2` (bitwise) | `a && b` (lexer birebir eşler) | 1 |
| `Or` | `0x07` | `rd = rs1 \| rs2` | `a \|\| b` | 1 |
| `Xor` | `0x08` | `rd = rs1 ^ rs2` | `a ^ b` | 1 |
| `Not` | `0x09` | `rd = !rs1` (bitwise tümleyen) | `!a` | 1 |

### 2.4 Karşılaştırma

| Opcode | Hex | Semantik | budl karşılığı | Gas |
|---|---|---|---|---|
| `Eq` | `0x0A` | `rd = (rs1 == rs2) ? 1 : 0` | `a == b` | 1 |
| `Neq` | `0x0B` | `rd = (rs1 != rs2) ? 1 : 0` | `a != b` | 1 |
| `Lt` | `0x0C` | `rd = (rs1 < rs2) ? 1 : 0` | `a < b` | 1 |
| `Gt` | `0x0D` | `rd = (rs1 > rs2) ? 1 : 0` | `a > b` | 1 |
| `Lte` | `0x0E` | `rd = (rs1 <= rs2) ? 1 : 0` | `a <= b` | 1 |
| `Gte` | `0x0F` | `rd = (rs1 >= rs2) ? 1 : 0` | `a >= b` | 1 |

### 2.5 Bellek ve Yığın

| Opcode | Hex | Semantik | budl karşılığı | Gas |
|---|---|---|---|---|
| `Load` | `0x14` | `rd = memory[imm]` (veya `rd = imm` if rs1==0, immediate-load) | `let x = literal` | 1 |
| `Store` | `0x15` | `memory[imm] = rs1` | struct field write, mapping slot | 1 |
| `Push` | `0x16` | push `rs1` to call-stack | fn parametre pass, return value | 1 |
| `Pop` | `0x17` | pop call-stack → `rd` | discard, parameter receive | 1 |

### 2.6 Doğrulama ve Kanıt

| Opcode | Hex | Semantik | budl karşılığı | Gas | Mainnet-gate |
|---|---|---|---|---|---|
| `Assert` | `0x18` | `if rs1 == 0 { revert }` | `constrain(cond)` | 1 | hayır |
| `Poseidon` | `0x19` | `rd = poseidon(rs1, rs2)` (4-round Goldilocks Poseidon) | `hash(a, b)` | 10 | hayır |

### 2.7 Depolama ve Olay

| Opcode | Hex | Semantik | budl karşılığı | Gas |
|---|---|---|---|---|
| `Log` | `0x1A` | `events.push(rs1)` | `emit Event(arg)` | 10 |
| `SRead` | `0x1B` | `rd = storage[slot]` | `let x = sread(key)` | 8 |
| `SWrite` | `0x1C` | `storage[slot] = rs1` (slot: imm veya rs2) | `swrite(key, val)` | 12 |

### 2.8 Sistem Çağrıları

| Opcode | Hex | imm | Semantik | budl karşılığı | Gas |
|---|---|---|---|---|---|
| `Syscall` | `0x1D` | 1 | `rd = context.sender` | `caller()` | 5 |
| `Syscall` | `0x1D` | 2 | `rd = context.block_height` | `block_height()` | 5 |
| `Syscall` | `0x1D` | 3 | `rd = context.nonce` | `nonce(addr)` | 5 |
| `Syscall` | `0x1D` | 4 | (planlanan) `rd = chain_id` | `chain_id()` | 5 |
| `Syscall` | `0x1D` | 5 | (planlanan) `rd = timestamp` | `timestamp()` | 5 |
| `Syscall` | `0x1D` | 6 | AI request: events.push(0x00A1_00A1); events.push(rs1); rd = block_height + rs1 | `ai_request(input)` | 5 |
| `Syscall` | `0x1D` | diğer | rd = 0 (no-op) | — | 5 |

### 2.9 Kriptografik Doğrulama (Mainnet-Gated)

| Opcode | Hex | Semantik | budl karşılığı | Gas | Mainnet-gate |
|---|---|---|---|---|---|
| `VerifyMerkle` | `0x1E` | 64-depth SMT path verify. `rd` = 0/1 sonuç. `rs1` = root, `rs2` = leaf, `imm` = path address in memory (520 byte: 8 byte key + 64×u64 sibling). AIR trace 64 Poseidon round expansion ekler. | `verify_merkle(root, leaf, path)` | 10 | ✅ `verify_merkle_enabled` |
| `VerifyInference` | `0x1F` | ZKVM execution proof for AI inference verify. `rd` = 0/1. `rs1` = `AiExecutionProof` struct ptr, `rs2` = model_id + input_commitment ptr, `imm` = proof_type (0=STARK, 1=SNARK wrap). | `verify_inference(proof, model, input)` | 10 | ✅ `verify_inference_enabled` |

### 2.10 Gizlilik Katmanı (D2, Mainnet-Gated)

| Opcode | Hex | Semantik | budl karşılığı | Gas | Mainnet-gate |
|---|---|---|---|---|---|
| `PrivacyCommit` | `0x20` | Poseidon commitment: `rd = poseidon(amount, recipient, blinding)`. Tutar + alıcı + blinding faktörünü tek commitment hash'ine bağlar. | `commit(amount, recipient, blinding)` | 10 | ✅ `privacy_commit_enabled` |
| `NullifierCheck` | `0x21` | Spent-commitment marker. `rd` = 0/1 (0 = zaten harcanmış, double-spend reddedilir). Bellekte bir nullifier set'i tutar. | `nullifier_check(nullifier)` | 10 | ✅ `nullifier_check_enabled` |
| `SumConservation` | `0x22` | Σinputs == Σoutputs homomorfik kanıtı (tutarlar açığa çıkmadan). `rd` = 0/1 (1 = balance sağlanmış). | `sum_conservation(inputs, outputs)` | 10 | ✅ `sum_conservation_enabled` |

---

## 3. Mainnet Activation Gate

`bud-isa` üç seviyeli gate sağlar:

```rust
pub struct MainnetActivation {
    pub verify_merkle_enabled:    bool,  // default false (staged rollout)
    pub verify_inference_enabled: bool,  // default false
    pub privacy_commit_enabled:   bool,  // default false (D2)
    pub nullifier_check_enabled:  bool,  // default false (D2)
    pub sum_conservation_enabled: bool,  // default false (D2)
}
```

**Decode yolları:**

| Fonksiyon | Profil | Davranış |
|---|---|---|
| `decode_any(u64)` | — | Sadece opcode aralığı (0x00–0x22) kontrol edilir. Gate yok. |
| `decode_for_profile(u64, IsaProfile)` | Production / Experimental / Testing | `is_experimental()` true + Production ise reddedilir. Şu an hiçbir opcode experimental değil. |
| `decode_for_mainnet(u64, MainnetActivation)` | Mainnet | `requires_mainnet_activation()` true + ilgili flag false ise `MainnetActivationRequired` döner. |

**Mainnet'te varsayılan:** 5 gated opcode (`VerifyMerkle`, `VerifyInference`, `PrivacyCommit`, `NullifierCheck`, `SumConservation`) `MainnetActivation::default()` ile decode edilemez. Ceremony sonrası `MainnetActivation::full()` kullanılır.

**Test/development:** `IsaProfile::Testing` veya `Experimental` (cfg `experimental` feature) gate'leri devre dışı bırakır.

**bud-vm davranışı:** `Vm::run_receipt()` `is_verify_merkle_enabled()` env değişkenine (`BUDLUM_VERIFY_MERKLE`) bakar; varsayılan `true` (geriye uyumluluk). Mainnet config TOML `[features] verify_merkle=false` ise `MainnetActivation::default()` seçilir.

---

## 4. Derleme Sırası (Codegen Akışı)

`bud-compiler/src/codegen.rs` AST → bytecode çevirisi:

### 4.1 Contract girişi

```rust
self.emit(Opcode::Load, 31, 0, 0, 4096);    // heap pointer init
self.emit(Opcode::Call, 0, 0, 0, 0);        // main()'a call (offset patch'lenecek)
self.emit(Opcode::Halt, 0, 0, 0, 0);        // dönüşte dur
```

### 4.2 Statement çevirisi (özet)

| AST | Bytecode kalıbı |
|---|---|
| `let x = literal` | `Load r_x, 0, 0, literal` |
| `let x = expr` (binary) | `expr_compile → r_temp; Push r_temp` |
| `let x = call(args)` | `arg_compile → push'lar; Call r_x, 0, 0, fn_offset` |
| `let x = ident` (read) | `Load r_x, 0, 0, slot_of(ident)` |
| `IDENT = expr` (storage write) | `expr_compile → r_v; SWrite 0, r_v, 0, slot` |
| `IDENT[expr_key] = expr_v` (mapping write) | `key_compile → r_k; v_compile → r_v; Load r_h, 0, 0, base_slot; Poseidon r_t, r_h, r_k, 0; SWrite 0, r_v, r_t, -1` |
| `if (cond) { A } else { B }` | `cond → r_c; Jnz 0, r_c, 0, +len_A; (B); Jmp 0, 0, 0, +end; (A);` |
| `while (cond) { body }` | `(start) cond → r_c; Jnz 0, r_c, 0, +end; body; Jmp 0, 0, 0, -(len+2);` |
| `for i in s..e { body }` | `Load r_i, 0, 0, s; (loop) Lt r_c, r_i, r_e, 0; Jnz 0, r_c, 0, +end; body; Load r_1, 0, 0, 1; Add r_i, r_i, r_1, 0; Jmp 0, 0, 0, -(loop_len);` |
| `return expr` | `expr → r_v; Push r_v; Ret 0, 0, 0, 0` |
| `return` | `Ret 0, 0, 0, 0` |
| `constrain(expr)` | `expr → r_c; Assert 0, r_c, 0, 0` |
| `emit Event(args)` | her arg → push; `Log 0, r_arg, 0, 0` (args sırayla) |
| `match scrutinee { ... }` | her arm: `Load r_p, 0, 0, pat; Sub r_d, r_s, r_p, 0; Jnz 0, r_d, 0, +body_len; Jmp 0, 0, 0, +next_arm; body; Jmp 0, 0, 0, +end;` |

### 4.3 Örnek: `let a = 100; let b = 37; let sum = a + b;`

```asm
Load  r1, 0, 0, 100      ; a = 100
Load  r2, 0, 0, 37       ; b = 37
Add   r3, r1, r2, 0      ; sum = a + b
```

`example.bud` tam derlenince yaklaşık 25-30 instruction, ardından `Log` zinciri.

---

## 5. Örneklerden Opcode Akışı

### 5.1 `examples/test_prover.bud`

```budl
contract Test {
    fn main() {
        let x = 10;
        let y = 20;
        let z = x + y;
        constrain(z == 30);
    }
}
```

Beklenen bytecode iskeleti:
```asm
Load  r31, 0, 0, 4096     ; heap init
Call  r0,  0, 0, +main     ; jump to main (patched)
Halt  r0,  0, 0, 0
main:
  Load r1,  0, 0, 10
  Load r2,  0, 0, 20
  Add  r3,  r1, r2, 0
  Load r4,  0, 0, 30
  Eq   r5,  r3, r4, 0
  Assert r0, r5, 0, 0
  Ret  r0,  0, 0, 0
```

### 5.2 `examples/example_loop.bud` (iç-içe for/while)

```budl
contract LoopExamples {
    pub fn main() {
        let for_sum = 0;
        for i in 0..5 {
            for_sum = for_sum + i;
        }
        ...
        if (for_sum == 10) { emit ForLoopOk(for_sum); }
    }
}
```

`for` 0..5 = `[0,1,2,3,4]` toplam = 10. `for` JIT: `Lt + Jnz + Add + Jmp` döngüsü. Toplam instruction sayısı ~40-50 (iç-içe for + while + if-else + 2 emit).

### 5.3 `examples/example.bud` (tüm operatörler)

Tüm aritmetik (Add, Sub, Mul) + karşılaştırma (Eq, Gt) + 5 kez `emit`. ~25-30 instruction.

---

## 6. Bilinçli Kapsam Dışı

- **Float ops:** YOK. Tüm aritmetik Goldilocks field.
- **String ops:** YOK. Bytes `u64` dilimleri.
- **Tablo / array:** YOK. Sadece mapping (Poseidon-keyed).
- **Modül / import:** YOK. Tek contract dosyası.
- **Trait / dyn dispatch:** YOK.
- **Generics:** YOK.
- **Inline assembly:** YOK. budl → bytecode dönüşümü tek yönlü.

---

## 7. Referanslar

- `budZero/bud-isa/src/lib.rs` — Opcode enum + encode/decode + MainnetActivation
- `budZero/bud-compiler/src/codegen.rs` — AST → bytecode patternleri
- `budZero/bud-vm/src/lib.rs` — execute + gas_cost() + syscall handler
- `budZero/bud-proof/src/plonky3_prover.rs` — STARK trace → proof
- `budZero/docs/BudL_SPEC.md` — Eski kapsam/spec (ARENA1)
- `budZero/docs/02_isa_ve_bytecode.md` — ISA ve bytecode detayları

---

*Co-authored-by: ARENA3 <arena3@budlum.xyz>*
