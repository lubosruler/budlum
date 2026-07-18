# RFC — SocialFi Luminance: Uncapped Product Policy and `u128` Migration

**Status:** User decision recorded; implementation remains P2/migration-gated
**Owner:** ARENA3 (security / policy review)
**Related:** `src/socialfi/mod.rs`, `src/socialfi/types.rs`,
`src/execution/executor.rs`, P2 schema-4 plan, Q37 in
`docs/BUDLUM_100_KARAR_ANKETI_ARENA2_2026-07-17.md`.

---

## 1. Decisions

| Decision | User choice | Consequence |
|---|---|---|
| Product luminance cap | **No policy cap** | A valid NFT boost must not stop producing luminance merely because it reaches an arbitrary product threshold. |
| Numeric representation | **`u128`** | Replace the current `u64` luminance storage/serialization/root representation with `u128`. |

This removes the proposed `MAX_LUMINANCE` policy-cap path. `u128` still has a
finite machine maximum, but provides a technical representation range far beyond
the `u64` storage limit without introducing an unbounded integer, new dependency,
or unbounded consensus serialization surface.

## 2. Existing defect and scope

Current `NftRegistry::update_luminance` computes an `i128` intermediate and
casts the result to `u64`. A value above `u64::MAX` can therefore truncate/wrap.
Separately, `NftBoost.amount` is `u64` but was converted through `as i64` before
being applied to luminance; amounts above `i64::MAX` could become a negative
value under that cast.

The migration fixes representation and arithmetic. It does **not** redefine the
Q37 social-ranking algorithm, annual decay, UI thresholds, boost 4/16/80 fee
split, or the existing owner authorization of `NftUpdateLight`.

## 3. Canonical arithmetic after migration

1. Store `Nft.luminance` as `u128` measured in mcd. A newly minted NFT remains
   `1000` mcd.
2. Accept `NftBoost.amount: u64` as a non-negative `u128` delta via lossless
   widening (`u128::from(amount)`), never through `as i64`.
3. Keep `NftUpdateLight.delta_mcd: i64` for the existing signed owner-driven
   interface. Compute the transition with checked signed arithmetic:
   - a negative result becomes zero, preserving the current lower-bound rule;
   - a positive result above `u128::MAX` returns a typed overflow error and
     mutates no state.
4. Do not introduce an arbitrary ranking, UI, economic, or policy ceiling.
5. `NftBoost` must preflight the luminance result before debiting the booster,
   crediting creator/treasury, changing pending rewards, or incrementing nonce.
   An impossible technical overflow therefore leaves all economic state intact.

A suitable API shape is:

```rust
pub fn checked_luminance_after(
    current_mcd: u128,
    delta_mcd: i128,
) -> Result<u128, NftError>;
```

`update_luminance` becomes a thin mutation wrapper around this pure calculation.
A boost-specific caller widens `amount` to `u128` and must reject only if that
value cannot be represented in the signed calculation contract; an alternative
unsigned helper is acceptable if it preserves the same canonical result.

## 4. Consensus, root, and migration requirements

Changing `u64` to `u128` changes serialized state and the byte layout of
`NftRegistry::root`. It must not be merged as an isolated behavioural tweak.
It is part of the user-selected **P2 full root-hardening scope**:

- Define a new SocialFi root version/domain tag; hash luminance with canonical
  16-byte little-endian `u128` encoding.
- Include the updated root semantics in the P2 schema-4 snapshot digest,
  migration/legacy handling, and root pin tests.
- Audit all transaction, protobuf, JSON/RPC, bincode and snapshot conversions
  that expose `Nft` or luminance. A narrowing cast or stringly/non-canonical
  number encoding is not allowed.
- Document any schema compatibility path. Existing persisted `u64` values widen
  exactly to `u128`; no numeric data transformation or cap-clamping occurs.
- Keep the implementation on the single P2 atomic PR/coordination path. ARENA1
  owns C1–C6; ARENA3 reviews root/domain separation and supplies P4 CI/fuzz only
  after the declared integration point.

## 5. Mandatory acceptance tests

1. A value above `u64::MAX` is representable, survives update, serialization,
   root calculation and snapshot roundtrip unchanged.
2. `NftBoost.amount > i64::MAX` remains a positive boost; it never becomes a
   negative signed delta.
3. A `u128` technical overflow is rejected before every balance, treasury,
   pending-reward, NFT or nonce mutation.
4. A negative owner-authorized delta clamps to zero under the selected Q37
   lower-bound semantics.
5. Existing owner checks for `NftUpdateLight` remain enforced.
6. Root/snapshot pin tests distinguish the old u64 layout from the new u128
   layout and prove deterministic schema-4 migration.
7. All required CI checks are `completed/success` for the implementation SHA.

## 6. Non-goals and later decisions

- This RFC does not promise mathematical infinity: fixed-width consensus values
  must remain bounded for deterministic storage, hashing and denial-of-service
  control. `u128` is the selected practical representation, not a product cap.
- Any future desire for a truly arbitrary-precision score would require a
  separate consensus-format, resource-bounding and dependency RFC.
- UI display/formatting for values larger than `u64::MAX` must be specified by
  the client/UI owner before public use; the chain representation remains
  canonical integer mcd.

**Implementation gate:** P2 owner coordination and schema-4 migration design.
**Decision owner:** Ayaz.
**No code is authorized by this RFC alone.**

*Co-authored-by: ARENA3 <arena3@budlum.xyz>*
