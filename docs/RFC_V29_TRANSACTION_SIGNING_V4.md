# RFC — V29 Transaction Signing V4

**Status:** User-approved emergency migration plan
**Decision:** strict V4; legacy non-genesis signatures rejected; explicit
field-level canonical encoding
**Scope:** every `TransactionType` payload that can affect execution.

---

## 1. Security defect

The current signing preimage includes top-level transaction fields and a
transaction-type byte but omits variant payload fields. A signed transaction can
therefore be modified by changing a variant payload while retaining its hash and
signature. This is a mainnet blocker.

## 2. Approved migration policy

- New normal transactions use the domain separator `BDLM_TX_V4`.
- The signing preimage commits every execution-relevant transaction field.
- Legacy V3 **non-genesis** signatures are rejected at every new transaction
  admission surface (RPC, mempool and block validation). The genesis exception
  remains narrowly defined as zero-from + zero-to + no signature + canonical
  genesis payload.
- Deployment must purge/reject in-flight V3 mempool entries. Existing test or
  fixture constructors must emit V4 transactions.
- No generic JSON or bincode serialization is used as the signing payload.

## 3. Version representation and admission invariant

Add a `signature_version` field to `Transaction`, with an explicit V4 value for
all constructors. Deserialization of older wire data must not silently create an
admissible V3 normal transaction: legacy/default values are rejected before
signature verification for non-genesis transactions.

`verify()` (or an explicitly named replacement) must enforce:

1. canonical transaction hash equals the versioned signing hash;
2. only V4 is accepted for a non-genesis transaction;
3. the genesis exception is exact and cannot be reached by a zero-address
   arbitrary transaction; and
4. the signature verifies against the V4 preimage.

Historical-chain compatibility must be assessed before merge. If persisted
pre-V4 blocks are supported, their verification path needs an explicit,
height-bound historical verifier that is unavailable to mempool/RPC admission.
It must not be a global “accept V3” fallback.

## 4. Canonical V4 preimage

The fixed top-level sequence is:

```text
"BDLM_TX_V4"
signature_version:u8
tx_type_tag:u8
from:32B | to:32B | amount:u64le | fee:u64le | nonce:u64le
data_len:u64le | data:bytes
timestamp:u128le | chain_id:u64le
variant_tag:u8 | variant_payload:canonical bytes
```

Every variable-length byte/string uses an unsigned 64-bit little-endian length
prefix. Optional values use `0x00` for None and `0x01 || value` for Some. Lists
use `len:u64le` followed by each canonical element. Booleans use exactly
`0x00`/`0x01`. Enum values use one explicit tag; no Rust discriminant or serde
layout is assumed canonical.

## 5. Variant coverage manifest

| Variant family | V4 payload commitment |
|---|---|
| Transfer, Stake, Unstake, Vote, ContractCall, BNS and basic NFT variants | Explicit empty variant payload. Their execution data remains committed through top-level length-prefixed `data`. |
| `NftBoost` | `nft_id:u64le`, `amount:u64le` |
| `NftUpdateLight` | `nft_id:u64le`, `delta_mcd:i64le` |
| `NftTag` | `nft_id:u64le`, length-prefixed UTF-8 tag bytes |
| `UniversalRelay` | canonical `ExternalChain` tag, target-address string, payload bytes, external nonce |
| `RelayerResult` | canonical external chain, tx hash string, success flag, optional canonical `CrossDomainMessage`, receipt proof bytes, external root |
| `AiOfferData` | content ID 32B, price:u64le |
| `AiPurchaseData` | offer ID:u64le |
| `HubRegisterApp` | name string, explicit `AppCategory` tag, URL string, optional content ID |
| `AiModelRegister` | every `AiModelSpec` field in declaration order, including active/version/deadlines/limits/owner/model hash/id |
| `AiInferenceRequest` | every request field including input-ref length/content, fee, callback option and both block heights |
| `AiInferenceResult` | every result field including output-ref length/content, nonce, result signature length/content and submitted height |
| `AiFeeReclaim` | request ID 32B |
| `AiModelDeactivate` | model ID 32B |

`CrossDomainMessage` and nested external-chain encodings receive dedicated
private canonical helpers. Helpers must be reused only where their exact field
semantics match; a Merkle leaf or a pre-existing unrelated hash is not a
substitute for the signed payload unless it commits all fields and is itself
versioned for this purpose.

## 6. Required tests

1. For every payload-carrying variant, sign a valid transaction, mutate one
   payload field, then assert both `calculate_hash` and `verify` reject it.
2. Mutate top-level data, type tag, chain ID, timestamp and each optional/list
   length boundary; signatures must fail.
3. A valid V4 transaction for every variant verifies and round-trips through
   JSON/protobuf without changing its signing hash.
4. V3/default/non-genesis transaction is rejected at mempool/RPC/block
   admission. The exact genesis exception remains accepted only for canonical
   genesis.
5. Explicit encoding KAT vectors pin every `ExternalChain`, `AppCategory`,
   Option and length-prefix encoding.
6. Fuzz `Transaction` decode and V4 signing payload construction for panic-free
   malformed inputs.
7. CI must be fully successful before any subsequent V19 implementation.

## 7. Deployment checklist

- [ ] All constructors and typed proto conversions emit `signature_version = 4`.
- [ ] Mempool eviction/purge path removes non-genesis V3 entries at activation.
- [ ] Historical block verification policy is explicitly tested or unsupported
      old databases are fail-loud at startup.
- [ ] Wallet/RPC/client release note specifies V4 re-signing requirement.
- [ ] CI and adversarial mutation tests pass.

**Decision owner:** Ayaz
**Implementation owner:** transaction/core owner, with ARENA3 security review
**V19 dependency:** V19 code remains blocked until this V29 checklist is closed.

*Co-authored-by: ARENA3 <arena3@budlum.xyz>*
