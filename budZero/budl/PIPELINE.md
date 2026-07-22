# budl Derleme ve Kanıt Pipeline'ı

> budl kaynak programının baştan sona nasıl derlenip, çalıştırılıp, STARK kanıtına
> dönüştüğünü ve Budlum konsensüsüne nasıl girdiğini tanımlar.
>
> **Sürüm:** v0.1 (2026-07-22) · **Sahip:** ARENA3

---

## 1. Genel Bakış (5 Aşama)

```
                ┌─────────────┐
                │ .bud source │
                └──────┬──────┘
                       │  bud-compiler
                       ▼
        ┌──────────────────────────┐
        │ AST + Typed + Bytecode   │  bud-isa Instruction stream
        │ Vec<u64>                 │
        └──────┬───────────────────┘
               │  bud-vm
               ▼
    ┌────────────────────────┐
    │ ExecutionReceipt        │
    │ + ExecutionTrace        │
    │ (deterministic, ordered)│
    └──────┬──────────────────┘
           │  bud-proof (Plonky3 STARK)
           ▼
   ┌─────────────────────┐
   │ ProofEnvelope        │  STARK proof (air-bundled)
   │ public_inputs        │
   │ trace_commitment     │
   └──────┬───────────────┘
          │  bud-cli Run
          ▼
   ┌─────────────────────────────┐
   │ State root güncellemesi      │  Budlum konsensüsüne
   │ (AccountState.apply_block)   │  budlum-core (src/) tarafından
   └─────────────────────────────┘
```

---

## 2. Aşama 1 — bud-compiler (`budZero/bud-compiler/`)

### 2.1 Lexer (`lexer.rs`, 99 satır)

`logos` crate'i ile tokenization. Token'lar:

- **Anahtar kelimeler:** `contract`, `fn`, `let`, `if`, `else`, `match`, `storage`, `witness`, `constrain`, `pub`, `while`, `for`, `in`, `return`, `struct`
- **Noktalama:** `{ } ( ) [ ] : , ; -> => .. . + - * / == != < > <= >= =`
- **Değişken tanımlayıcı:** `[a-zA-Z_][a-zA-Z0-9_]*` → `Ident(String)`
- **Tamsayı:** `[0-9]+` → `Int(u64)` (doğrudan `u64`'e parse)
- **Hex:** `0x[0-9a-fA-F]+` → `Hex(String)`
- **Yorum:** `//...\n` ve `/* ... */` skip
- **Boşluk:** skip

### 2.2 Parser (`parser.rs`, 644 satır)

`Parser::new(source)` token vektörü oluşturur. Sonra:

- `parse_contract()` — `Contract` AST node'u: storage, struct'lar, fonksiyonlar.
- `parse_function()` — `Function { name, params, return_type, body, is_pub }`.
- `parse_statement()` — `let / assign / if / while / for / return / emit / match / expr`.
- `parse_expression()` — `Expr::Binary(Box, BinOp, Box)` vb.

Hata durumunda `CompileError::ParserError` veya `LexerError` döner.

### 2.3 AST (`ast.rs`, 109 satır)

Veri yapıları (yapılar):

```rust
pub struct Contract { name, storage, structs, functions }
pub struct Function { name, params, return_type, body, is_pub }
pub enum Stmt { Let, Constrain, Assign, StorageWrite, MappingWrite, If, While, For, Return, Emit, Match, Expr }
pub enum Expr { Int, Ident, StorageRead, MappingRead, FieldAccess, StructLiteral, Binary, Call }
pub enum BinOp { Add, Sub, Mul, Div, Eq, Neq, Lt, Gt, Lte, Gte }
pub enum MatchPattern { IntLit, Wildcard }
```

### 2.4 Sema (`sema.rs`, 488 satır)

- Tip kontrolü.
- Değişken tanımlılığı.
- `match` exhaustiveness.
- Struct alan erişimi.

### 2.5 Codegen (`codegen.rs`, 948 satır)

`Codegen` struct'ı:
- `next_reg: u8` — register tahsisi.
- `instructions: Vec<u64>` — çıktı bytecode.
- `unpatched_calls: Vec<(idx, name)>` — call patch listesi (önce-derleme sonra-yamala).
- `struct_layouts: HashMap<String, Vec<String>>` — struct alan sırası (deterministic).

**Çıktı:** `Vec<u64>` — her `u64` bir `Instruction` encode edilmiş.

**Call patch mekaniği:** `generate()` sonunda tüm forward call'lar hedef fonksiyonun PC'sine yamalanır (gerçek relative offset).

---

## 3. Aşama 2 — bud-vm (`budZero/bud-vm/`)

### 3.1 Mimari

- 32 register (r0 zero, r1..r30 genel, r31 heap pointer).
- Doğrusal `Vec<u64>` memory.
- `Vm::run_receipt(&program) -> ExecutionReceipt` ana entry point.

### 3.2 Yapılar

```rust
pub struct Vm { registers: [u64; 32], memory: Vec<u64>, pc: u64, gas_used: u64, ... }
pub struct VmContext { sender, block_height, nonce, ... }
pub struct ExecutionReceipt { success, error, gas_used, exit_code, events, final_pc, trace_len, state_writes_digest }
pub enum VmError { OutOfGas, AssertionFailed, StackUnderflow, StackOverflow, InvalidOpcode, InvalidPc, InvalidMemoryAccess }
```

### 3.3 Execute döngüsü

```
loop:
    inst = Instruction::decode(memory[pc])
    (op, args) = inst
    (next_result, next_pc) = match op { ... }
    registers[rd] = next_result
    pc = next_pc
    gas_used += gas_cost(op)
    trace.push(row)
```

`gas_cost()` her opcode için sabit sayı döner (ör. `Div` = 10, `Poseidon` = 10, `VerifyMerkle` = 10).

### 3.4 Çıktılar

- `ExecutionReceipt.events: Vec<u64>` — `Log` ile yayınlanan değerler.
- `ExecutionReceipt.state_writes_digest: [u8; 32]` — storage değişikliklerinin Poseidon hash'i.
- Internal `trace: Vec<TraceRow>` — her instruction için (AIR için gereken tamalanmış trace, `bud-proof` tarafından tüketilir).

### 3.5 Syscall'lar

| imm | İşlem |
|---|---|
| 1 | `result = context.sender` |
| 2 | `result = context.block_height` |
| 3 | `result = context.nonce` |
| 4 | planlanan (`chain_id()`) |
| 5 | planlanan (`timestamp()`) |
| 6 | AI request: events.push(0x00A1_00A1); events.push(rs1); result = block_height + rs1 |
| diğer | no-op (result = 0) |

---

## 4. Aşama 3 — bud-proof (`budZero/bud-proof/`)

### 4.1 Plonky3 STARK AIR

`bud-proof/src/plonky3_prover.rs` ve `plonky3_air.rs`:

- AIR (Algebraic Intermediate Representation) — execution trace'in her step'i için kısıtlamalar.
- `MemoryAir` — `Load`/`Store` operasyonları.
- `Poseidon` round constraints — `Poseidon` opcode + `VerifyMerkle` 64-round expansion.

### 4.2 Proof üretimi

```
ExecutionTrace (from bud-vm)
    │
    ▼
Plonky3 prover (p3 FRI + constraints)
    │
    ▼
ProofEnvelope { proof_bytes, public_inputs: ExecutionPublicInputs, trace_commitment }
```

### 4.3 Public inputs (ExecutionPublicInputs)

- `initial_state_root` — trace başlangıcındaki state root.
- `final_state_root` — trace sonundaki state root.
- `initial_pc`, `final_pc`.
- `events_commitment` — `events` vektörünün Poseidon hash'i.
- `trace_commitment` — trace'in Merkle commitment'i.

### 4.4 Verification

`bud_proof::DefaultAdapter::verify(proof, public_inputs) -> bool`. Mainnet'te bu çağrı, her konsensüs katılımcısı tarafından bağımsız yapılır.

---

## 5. Aşama 4 — bud-cli (`budZero/bud-cli/`)

`bud` komut satırı aracı, pipeline'ı baştan sona çalıştırır:

```
bud run -p examples/example.bud
    │
    ├─ compile (bud-compiler)
    ├─ execute (bud-vm)
    ├─ prove  (bud-proof)
    └─ verify (bud-proof)
```

### 5.1 Subcommands (bud-cli/src/main.rs)

| Subcommand | İşlev |
|---|---|
| `run -p <file>` | Tam pipeline (compile + execute + prove + verify) |
| `compile -p <file>` | Sadece derleme → bytecode stdout |
| `execute -p <file>` | Sadece çalıştırma → receipt stdout |
| `prove -p <file>` | Sadece STARK proof üretimi |
| `verify -p <file>` | Sadece proof verification |

### 5.2 Çıktı

- `keccak256(contract_source)` — kod commitment.
- `keccak256(events || final_pc || state_writes_digest)` — execution commitment.
- `proof.size_bytes` — kanıt boyutu.
- `verify: true/false`.

---

## 6. Aşama 5 — Budlum Konsensüsü (`src/`, ana repo kök)

budl programı **konsensüs state'ine doğrudan yazmaz**. Bunun yerine:

1. `bud` CLI bir `ProofEnvelope` üretir.
2. Bu envelope, `src/budzero/` (eski `budzero/`) üzerinden veya harici bir **relayer** aracılığıyla `budlum-core`'a gönderilir.
3. `src/chain/blockchain.rs` veya `src/execution/executor.rs` (duruma göre) `verify_zk_proof` çağrısı yaparak envelope'u doğrular.
4. `public_inputs.final_state_root` state'e uygulanır.
5. Blok finalize olunca state root yeni değer alır.

**Önemli:** budZKVM execution'ı, normal bir işlem gibi (tx) gönderilir; diğer node'lar proof'u bağımsız doğrular. Execution **kanıtla doğrulanır**, tekrar çalıştırılmaz.

---

## 7. End-to-End Örnek

Bir geliştirici `examples/example.bud` dosyasını yazıyor:

```bash
# 1. Lokal: derle → çalıştır → prove → verify
bud run -p examples/example.bud --sender 0x42 --nonce 1 --block-height 100

# Çıktı:
#   compiled 17 instructions (134 bytes)
#   executed: 8 events (Sum=137, Diff=63, Prod=3700, IsEq=0, IsGt=1)
#   gas_used: 47
#   proof_size: 38 KB
#   verify: true
#
# 2. Envelope'u mainnet'e gönder (relayer veya doğrudan):
#   Tx tipi: ZkProofSubmit
#   payload: ProofEnvelope
#   public_inputs: { initial_state_root, final_state_root, events_commitment, trace_commitment }
```

State root güncellenir → blok finalize olur → mainnet'e committed.

---

## 8. Ses Yüzeyleri (Soundness)

Pipeline'ın her aşamasında **fail-closed** davranış:

- **Compile:** Hata → bytecode üretilmez → tx gönderilemez.
- **Execute:** Assertion fail / OOG → `VmError` + `success=false` → proof üretilmez.
- **Prove:** Trace, AIR kısıtlamalarını karşılamazsa → `ProofGenerationFailed` → proof üretilmez.
- **Verify:** Proof yanlış veya public inputs mismatch → `false` → state güncellenmez.

Soundness garantisi: budZKVM üzerinde yalnızca geçerli programlar, geçerli inputlarla, geçerli bir state geçişi üretebilir.

---

## 9. Konfigürasyon (Config-Driven Activation)

- `mainnet.toml [features] verify_merkle = false` → `MainnetActivation::default()` (gated).
- `BUDLUM_VERIFY_MERKLE=false` env değişkeni → aynı.
- `[features] verify_inference = false` → benzer.
- `[features] privacy_commit = false` → D2 gate kapalı.
- `default = true` (geriye uyumluluk).

`bud-vm` her başlangıçta bu config/env'leri okur, decode fonksiyonlarına uygun activation'ı geçer.

---

## 10. Test Coverage Pipeline'ı

- `bud-compiler` birim testleri: her AST düğümü, her opcode kalıbı.
- `bud-isa` birim testleri: encode/decode roundtrip, MainnetActivation gate testleri (D2 dahil).
- `bud-vm` testleri: `tests/trace_fixtures.rs` execution trace determinism; her opcode bir test.
- `bud-proof` testleri: `tests/soundness_negatives.rs` geçersiz proof reddi; `trace_layout_tests.rs` doğru layout; `test_goldilocks.rs` Goldilocks aritmetiği.
- `bud-cli` smoke: end-to-end `examples/*.bud` çalıştırma.

---

## 11. Performans Karakteristikleri

- **Compile:** O(n) — AST boyutunda. ~1ms / 100 instruction.
- **Execute:** O(n) — instruction sayısında. ~10µs / instruction (modern CPU).
- **Prove:** O(n log²n) Plonky3 karmaşıklığı. **Ana darboğaz.** 100K instruction = ~30s, proof boyutu ~100 KB.
- **Verify:** O(log²n) — proof boyutunda. ~50ms.

---

*Co-authored-by: ARENA3 <arena3@budlum.xyz>*
