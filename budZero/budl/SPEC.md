# budl — Dil Spesifikasyonu

> **budl** (Budlum language), BudZKVM üzerinde çalışan, STARK-provable, deterministik
> akıllı-kontrat ve programlama dilidir. Bu doküman dilin gramerini, tiplerini,
> kontrol akışını, dil→bytecode derleme modelini ve çalışma zamanı semantiğini tanımlar.
>
> **Sürüm:** v0.1 (2026-07-22) · **Sahip:** ARENA3 · **Durum:** Draft
> **Uygulama:** `budZero/bud-compiler/` (lexer, parser, ast, sema, codegen)
> **Hedef ISA:** `budZero/bud-isa/` (35 opcode: 0x00–0x22)
> **Çalışma zamanı:** `budZero/bud-vm/` (register + memory VM, Goldilocks alanı)
> **Kanıt:** `budZero/bud-proof/` (Plonky3 STARK AIR)
>
> **İlgili dokümanlar:**
> - `budZero/docs/BudL_SPEC.md` — ARENA1'in eski teknik spec'i (kapsam, gas modeli, storage modeli)
> - `budZero/docs/02_isa_ve_bytecode.md` — ISA ve bytecode detayları
> - `budZero/docs/03_virtual_machine.md` — VM mimarisi
> - `budZero/budl/OPCODES.md` — bud-isa opcode seti ve budl dil seviyesi karşılığı
> - `budZero/budl/PIPELINE.md` — budl → bytecode → execution trace → STARK proof akışı
> - `budZero/budl/README.md` — klasör haritası ve giriş
> - `budZero/budl/examples/` — 5 çalışan örnek

---

## 1. Tasarım İlkeleri

- **Deterministik:** Aynı giriş, aynı program, aynı blok bağlamı → aynı çıktı. Budlum konsensüsünün temel gereksinimi.
- **STARK-provable:** Her budl programı bir execution trace üretir. Trace Plonky3 STARK prover ile prove edilir. Soundness, ekonomik güvenlikten önce gelir.
- **Gas-metered:** Her opcode'un sabit gas maliyeti vardır. `gas_limit` aşılırsa program revert.
- **Alan-merkezli aritmetik:** Tüm aritmetik Goldilocks alanı (p = 2⁶⁴ − 2³² + 1) üzerinde yapılır. Field overflow invariant'ı zorunlu.
- **Statik tip sistemi:** Derleme zamanı tip kontrolü. Dinamik dispatch yok.
- **Yapısal basitlik:** Yinelemeli (recursive) call desteği, struct, kalıcı storage, minimal primitive kümesi.

---

## 2. Sözdizimi (EBNF)

```
program        := contract
contract       := 'contract' IDENT '{' contract_item* '}'
contract_item  := struct_decl | storage_decl | fn_decl

struct_decl    := 'struct' IDENT '{' field_decl+ '}'
field_decl     := type IDENT ';'
type           := 'u32' | 'u64' | 'u128' | 'bool' | 'Address' | 'Hash32' | IDENT

storage_decl   := 'storage' '{' storage_field+ '}'
storage_field  := IDENT ':' type ';'

fn_decl        := 'pub'? 'fn' IDENT '(' params? ')' ( '->' type )? block
params         := param ( ',' param )*
param          := type IDENT

block          := '{' stmt* '}'
stmt           := let_stmt
                | assign_stmt
                | storage_write
                | mapping_write
                | if_stmt
                | while_stmt
                | for_stmt
                | emit_stmt
                | match_stmt
                | return_stmt
                | expr_stmt

let_stmt       := 'let' IDENT ( ':' type )? '=' expr ';'
assign_stmt    := IDENT '=' expr ';'
storage_write  := IDENT '=' expr ';'
                 (storage_decl içindeki değişken için)
mapping_write  := IDENT '[' expr ']' '=' expr ';'
if_stmt        := 'if' expr block ( 'else' ( if_stmt | block ) )?
while_stmt     := 'while' expr block
for_stmt       := 'for' IDENT 'in' expr '..' expr block
emit_stmt      := 'emit' IDENT '(' args? ')' ';'
match_stmt     := 'match' expr '{' match_arm+ '}'
match_arm      := pattern '=>' ( block | ',' )
pattern        := INT_LITERAL | '_'
return_stmt    := 'return' expr? ';'
expr_stmt      := expr ';'

expr           := binary_expr | unary_expr | primary
binary_expr    := expr binop expr
binop          := '+' | '-' | '*' | '/' | '==' | '!=' | '<' | '>' | '<=' | '>='
                | '&&' | '||' | '&' | '|' | '^'
unary_expr     := '!' expr
primary        := INT_LITERAL
                | HEX_LITERAL
                | IDENT
                | IDENT '(' args? ')'            (function call)
                | IDENT '[' expr ']'              (mapping read)
                | primary '.' IDENT               (field access)
                | primary '{' field_init_list '}' (struct literal)
                | '(' expr ')'

args           := expr ( ',' expr )*
field_init_list := IDENT ':' expr ( ',' IDENT ':' expr )*
```

**Notlar:**
- `&&`, `||` short-circuit değildir — `And(0x06)` / `Or(0x07)` opcode'ları bitwise işlem yapar (alan üzerinde). Mantıksal kısa-devre için codegen `Jnz + atla` desenine genişlemez (kapsam dışı).
- `&`, `|`, `^` lexer'da **operatör olarak ayrılmamış**; `&` ve `|` mevcut lexer'da `&&` ve `||` token üretir (logos default). Yani `a & b` iki token üretir: `And And`. Pratikte budl'da bitwise ifade yazımı **önerilmez** — örneklerde yok.
- `for i in start..end` aralık yarı-açık: `start, start+1, ..., end-1`. Adım = 1. Negatif adım, `..=` (kapsayıcı) desteklenmez.
- `match` arms `,` ile bitmeli; son arm sonrası `,` opsiyonel. Wildcard (`_`) her zaman en sonda olmalı (exhaustiveness sema kontrol eder).
- `storage { ... }` içindeki değişkenlere atama `IDENT = expr;` ile yapılır (farklı keyword yok).
- `emit EventName(args...)` — event adı serbest string; `Log (0x1A)` opcode'u değerleri sırayla `events` vektörüne push eder.

---

## 3. Tip Sistemi

| Tip | Budl | Boyut | Açıklama |
|---|---|---|---|
| Tamsayı | `u32` | 32-bit | Index, sayaç |
| Tamsayı | `u64` | 64-bit | **Varsayılan** tamsayı. Goldilocks alan elemanı (canonical < p) |
| Tamsayı | `u128` | 128-bit | Geniş tutar, mul sonucu (henüz derleyici seviyesinde zorunlu değil) |
| Boole | `bool` | 1-bit | `true` / `false`; `u64` üzerinde 0 / 1 |
| Adres | `Address` | 256-bit | 32 byte. Address primitive (u256) |
| Hash | `Hash32` | 256-bit | 32 byte. SHA-256 / Poseidon çıktısı |
| Struct | `IDENT` | değişken | Kullanıcı tanımlı. Alanlar sırayla register'lara yerleşir |

**Struct örneği** (kavramsal — `budl` örneklerinde henüz kullanılmıyor):

```budl
struct Balance {
    owner: Address,
    amount: u64,
    nonce: u64,
}

contract Token {
    storage {
        total_supply: u64,
    }

    pub fn mint(to: Address, amount: u64) {
        swrite(to, sread(to) + amount);
        emit Mint(to, amount);
    }
}
```

---

## 4. Kontrol Akışı İfadeleri

### 4.1 `if` / `else`

```budl
if (cond) {
    // true branch
} else {
    // false branch
}
```

`else` zorunlu değildir. İç içe `else if` deseni serbest.

### 4.2 `while`

```budl
while (cond) {
    // body
}
```

Koşul `u64` olarak değerlendirilir: 0 → false, nonzero → true. `Jnz (0x11)` ile sıçrama.

### 4.3 `for`

```budl
for i in 0..10 {
    sum = sum + i;
}
```

Yarı-açık aralık `[start, end)`. Sayaç değişkeni her iterasyonda +1 artar. `for_sum = 0; for i in 0..5 { for_sum = for_sum + i; }` örneği `examples/example_loop.bud` içinde.

### 4.4 `return`

```budl
return expr;   // değer döndür
return;        // void (opsiyonel)
```

`pub fn` ve `fn` her ikisi de değer döndürebilir. Dönüş tipi imzada `-> type` ile belirtilir.

### 4.5 `match`

```budl
match scrutinee {
    0 => { emit Zero(); },
    1 => { emit One(); },
    _ => { emit Other(); },
}
```

Exhaustiveness **zorunlu**: en az bir `_` arm bulunmalıdır (sema kontrol eder). Pattern yalnız tam sayı literal veya `_` olabilir. Struct destructuring ve range pattern Phase 0.16+.

### 4.6 `emit` — Olay yayını

```budl
emit Sum(sum);
emit Transfer(from, to, amount);
```

`Log (0x1A)` opcode'una derlenir. Argümanlar sırayla `events` vektörüne `u64` olarak push edilir. Olay adı yalnız belgeleme amaçlıdır (execution'da taşınmaz; keccak digest'i bud-cli tarafından hesaplanır).

---

## 5. Bellek ve Storage

### 5.1 Register ve bellek modeli

- **Register:** 32 adet `u64` (r0..r31). `r0` zero register, `r31` heap pointer (init: 4096).
- **Bellek:** Doğrusal `Vec<u64>`, default 64 KB. `Load (0x14)` / `Store (0x15)` ile okuma/yazma.
- **Stack:** Call/ret mekaniği için sanal stack. `Push (0x16)` / `Pop (0x17)`.

### 5.2 `storage { ... }` — Kalıcı durum

```budl
storage {
    total_supply: u64,
    balances_root: Hash32,
}
```

`SRead (0x1B)` ve `SWrite (0x1C)` opcode'ları ile erişilir. Slot, `IDENT` adının derive edilmiş pozisyonu (lexicographic index) veya doğrudan slot numarası olabilir (codegen karar verir).

### 5.3 Mapping (dictionary) — `IDENT[k] = v`

```budl
storage {
    balances: Hash32,  // Merkle root of balance tree
}

// Sözleşme içinde:
let bal = balances[addr];        // MappingRead
balances[addr] = bal + amount;   // MappingWrite
```

`Poseidon (0x19)` ile `slot || key` hash'i hesaplanır, sonra `SRead`/`SWrite` ile okunur/yazılır.

---

## 6. Standart Kütüphane (Planlanan)

budl için dahili yardımcılar henüz derleyicide tam implement edilmedi. Aşağıdaki liste `bud-compiler` `caller()` gibi inline `Syscall` çağrılarına genişleyebilir veya harici stdlib crate'i (`bud-std`) olarak ayrı eklenebilir.

| Yardımcı | Opcode | imm | Açıklama | Durum |
|---|---|---|---|---|
| `caller()` | `Syscall` | 1 | Çağıran adres | ✅ VM'de |
| `block_height()` | `Syscall` | 2 | Mevcut blok yüksekliği | ✅ VM'de |
| `nonce(addr)` | `Syscall` | 3 | Adres nonce | ✅ VM'de |
| `chain_id()` | `Syscall` | 4 | Zincir ID | Planlanan |
| `timestamp()` | `Syscall` | 5 | Blok timestamp | Planlanan |
| `ai_request(input)` | `Syscall` | 6 | AI inference isteği (event + accumulator) | ✅ VM'de (event tag 0x00A1_00A1) |
| `poseidon(data)` | `Poseidon` | — | Poseidon hash | ✅ VM'de |
| `verify_merkle(root, leaf, path)` | `VerifyMerkle` | — | 64-depth SMT | ✅ Mainnet-gated |
| `verify_inference(proof, model, input)` | `VerifyInference` | — | AI proof verify | ✅ Mainnet-gated |
| `commit(amount, recipient, blinding)` | `PrivacyCommit` | — | Özel commitment | ✅ Mainnet-gated (D2) |
| `nullifier_check(nullifier)` | `NullifierCheck` | — | Double-spend önleme | ✅ Mainnet-gated (D2) |
| `sum_conservation(...)` | `SumConservation` | — | Homomorfik tutar eşitliği | ✅ Mainnet-gated (D2) |

**Mainnet-gated opcodes:** `VerifyMerkle`, `VerifyInference`, `PrivacyCommit`, `NullifierCheck`, `SumConservation` — `MainnetActivation::full()` ile post-ceremony etkinleştirilir. Varsayılan (default) reddetme.

---

## 7. Anlam Kontrolü (Sema)

`bud-compiler/src/sema.rs` 488 satır; temel kontroller:

- **Tip kontrolü:** Aritmetik/lojik operatörler için operand tipleri uyumlu mu.
- **Değişken tanımlılığı:** Kullanılan her `IDENT` ya `let` ile tanımlanmış ya parametre ya storage alanı.
- **Struct alan erişimi:** Var olan alana erişim.
- **`match` exhaustiveness:** En az bir `_` arm.
- **Gas aritmetiği:** Recursive call'lar ve büyük storage erişimi için conservatif limit.

Sema hataları `CompileError::SemanticError` olarak yüzeye çıkar.

---

## 8. Derleme Akışı (özet)

```
.bud source
    │
    ▼  bud-compiler/src/lexer.rs
TokenStream (logos tabanlı)
    │
    ▼  bud-compiler/src/parser.rs
AST (bud-compiler/src/ast.rs)
    │
    ▼  bud-compiler/src/sema.rs
Typed AST (semantic kontroller geçti)
    │
    ▼  bud-compiler/src/codegen.rs
Vec<u64> (bud-isa bytecode)
    │
    ▼  bud-vm/src/lib.rs
ExecutionReceipt + ExecutionTrace
    │
    ▼  bud-proof::DefaultAdapter
ProofEnvelope (STARK Plonky3)
    │
    ▼  budlum konsensüsü
State root güncellemesi
```

Ayrıntılar: `budZero/budl/PIPELINE.md`.

---

## 9. Bilinçli Sınırlamalar (v0.1)

- **Float / ondalık:** Yok. Tüm aritmetik alan üzerinde.
- **String:** Yok. Bytes'lar `u64` dilimleri olarak taşınır.
- **Dinamik dispatch:** Yok. Trait / interface yok.
- **Generics:** Yok.
- **Modül sistemi:** Yok. Tek contract dosyası.
- **`&`/`|`/`^` bitwise:** Lexer kısıtı nedeniyle pratikte kullanılamaz.
- **`while true` sonsuz döngü:** Gas limit ile sınırlı (revert). Sema uyarısı yok.

---

## 10. Örnekler

Tüm örnekler `budZero/budl/examples/` altında:

- `example.bud` — Tüm aritmetik/lojik operatörler (Add, Sub, Mul, Eq, Gt) + emit zinciri.
- `example2.bud` — Minimum: `constrain` ile assertion.
- `example_loop.bud` — `for` (0..5) + `while` (count < 4) + `if` ile doğrulama.
- `test_prover.bud` — En küçük STARK-provable program: `a + b == c` constrain.
- `control_flow.bud` — `while` + `if/else` ile emit (Success/Failure).

---

*Co-authored-by: ARENA3 <arena3@budlum.xyz>*
