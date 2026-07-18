# BUDLUM
## Whitepaper — Universal Settlement Layer for a Post-Quantum, Multi-Consensus World

**Version:** 1.0 (Phase 10 state) · **Date:** 18 July 2026 · **License:** MIT
**Repository:** [github.com/budlum-xyz/budlum](https://github.com/budlum-xyz/budlum) · **Status:** Controlled public-devnet candidate (v0.3-dev) — *research-grade; not audited mainnet software*

---

## Türkçe Özet (Executive Summary — TR)

Budlum, mevcut blokzincirlerinin **yerini almayan**, onları **uzlaştıran** (settle eden) araştırma sınıfı bir Layer-1 protokolüdür. Her alan (domain) kendi konsensüsünü korur — PoW, PoS, PoA, BFT, ZK veya özel; Budlum bu alanların finalite kanıtlarını doğrular ve alanlar arası değer transferini **kriptografik bir gerçek** olarak kaydeder. TCP/IP ağların iç çalışmasını sormadan paket taşır; Budlum değerin hangi konsensüsle üretildiğini sormadan finaliteyi doğrular.

Temel direkler:

- **Kuantum-sonrası finalite:** BLS + Dilithium5 hibrit imza, protokolün çekirdek yolunda — "şimdi güvenli, kuantum çağında da güvenli."
- **Çok-konsensüslü settlement:** `GlobalBlockHeader` ile tek bir küresel kriptografik gerçeklik noktası; PoW light-client finalitesi, nonce-invariant ile çifte harcama koruması.
- **Güven-minimize köprü yaşam döngüsü:** Lock → Mint → Burn → Unlock, her adımda kanıt kapıları (2,5 milyar dolarlık köprü-hack sorununa doğrudan yanıt).
- **BudZero / BudZKVM:** Ağa gömülü STARK tabanlı sıfır-bilgi sanal makinesi; AI çıkarımı (inference) doğrulama primitifleriyle **Agentic Economy** altyapısı.
- **B.U.D. (Broad Universal Database):** İzinsiz, deal + challenge + slashing ekonomili merkeziyetsiz depolama ağı; veri egemenliği ilkesiyle whitelist/admin/pause kancası **yok**.
- **BNS (`.bud`) + SocialFi:** İçerik = NFT ("Dijital Tomurcuk"). Sahiplik, kontrol ve "kill-switch" tek varlıkta; "Işık Şiddeti" (Luminance) ile organik sıralama; Boost ekonomisi %4 depolama operatörü / %16 içerik üreticisi / %80 protokol hazine-yakım.
- **$BUD token:** Gas, depolama operatörü emisyonu, relayer teşvikleri, boost ve veri pazarı ödemeleri.

Proje, kanıta dayalı mühendislik kültürüyle geliştiriliyor: **755 Core lib testi, 124 BudZero, 12 B.U.D. invariant ve 8 BNS testi — hepsi CI-kanıtlı**; "sahte-yeşil" (kanıtsız iddia) bilinçli olarak işaretlenip dışlanıyor.

> ⚠️ Dürüst durum bildirimi: Budlum **henüz bağımsız dış denetimden geçmedi**; gerçek değer taşıyan üretim trafiği için kullanılmamalıdır.

---

## Abstract (EN)

Budlum is a research-grade Layer-1 protocol designed as a **Universal Settlement Layer**: rather than competing with existing chains, it *settles* them. Every Consensus Domain — Proof-of-Work, Proof-of-Stake, Proof-of-Authority, BFT, ZK-based, or a custom mechanism not yet invented — keeps its own consensus. Budlum verifies domain-specific finality proofs through pluggable adapters and records cross-domain value transfer as a single cryptographic fact: the `GlobalBlockHeader`.

The protocol stitches **hybrid post-quantum finality** (BLS aggregate signatures + ML-DSA/Dilithium5 QC blobs) into the core consensus path, so no "quantum hard-fork day" is required: the security model is quantum-ready from genesis. An in-tree STARK-proving virtual machine (**BudZero/BudZKVM**) provides deterministic, verifiable execution — including **AI inference attestation** primitives for the coming agent economy. A permissionless storage network (**B.U.D.**), a name registry (**BNS**, `.bud`), and an NFT-bound social layer (**Digital Buds**) complete a human-centric, data-sovereign ecosystem coordinated by the **$BUD** utility token.

Budlum is engineered under an explicit *evidence-first* discipline: every capability claim in this paper is traceable to code, a CI gate, or is explicitly marked as pending work.

---

## 1. Problem Statement

Today's digital value infrastructure suffers from seven structural failures. Each maps directly to a Budlum design decision.

| # | Problem today | Consequence | Budlum shift |
|---|---------------|-------------|--------------|
| 1 | **Quantum break (~Y2Q, 2030–35):** ECDSA/Ed25519 underlie every major chain; "Harvest Now, Decrypt Later" is already happening | Eventual cryptographic collapse of all financial ledgers | BLS + Dilithium5 **hybrid finality in the core path** — no emergency fork needed |
| 2 | **Walled gardens:** 20,000+ isolated chains; interop solutions (IBC, parachains) each demand *their* paradigm | Liquidity fragmentation, duplicated security budgets | **Universal Settlement Layer** — consensus-agnostic finality verification |
| 3 | **CBDC sovereign silos:** 130+ countries building isolated digital currencies | No trust-minimized cross-border settlement primitive | Domains + trust-minimized bridge lifecycle; math replaces correspondent trust |
| 4 | **TradFi ↔ DeFi wall:** permissioned PoA vs. public PoS never share a settlement record | Custodial bridges and oracles as single points of failure | One `GlobalBlockHeader` settles PoA and PoS finality side-by-side |
| 5 | **Bridge hacks ($2.5B+ stolen 2022–24):** Ronin, Wormhole, Nomad… | Systemic loss; bridges are the weakest link | Lock → Mint → Burn → Unlock with proof gates at every step |
| 6 | **AI agents without settlement:** the agentic economy has no merchant rail, no verifiable inference | Centralized payment chokepoints for machine economies | In-tree STARK zkVM + native AI-inference attestation events |
| 7 | **Data sovereignty collapse:** users rent their digital existence from platforms | No ownership, no portability, no delete right | **B.U.D.** storage + NFT-bound content: ownership, control, and kill-switch |

---

## 2. Design Philosophy

Four non-negotiable principles govern the codebase:

1. **Fail-closed by default.** Public RPC auth, Merkle-verification gates, mainnet key storage — every gate defaults to *deny*. A missing configuration is an error, not a silent fallback. (Example: disk-backed consensus keys are *rejected on mainnet* until HSM paths exist.)
2. **Evidence over claims ("no fake-green").** Test counts come only from CI summary lines; intentionally incomplete features (e.g., interim storage challenges) are labeled *not* proof-of-storage in plain text. Roadmap tables distinguish *implemented* from *policy ready* from *research*.
3. **Consensus pluralism.** No domain is forced onto Budlum's consensus; sovereignty is preserved at the domain level while settlement integrity is enforced at the layer level.
4. **Sovereignty from protocol, not operator.** No Budlum-critical function may depend on "a service run by the Budlum team." No whitelist, admin, pause, or freeze hooks exist in the storage layer; every storage RPC is servable by any node.

---

## 3. System Architecture

```
   PoW domain     PoS domain     PoA domain     ZK / Custom domain
        \              |              |              /
         \             |              |             /
          v            v              v            v
              DomainFinalityAdapter (per-consensus proof)
                                |
                                v
              ┌─────────────────────────────────────┐
              │         BUDLUM SETTLEMENT L1        │
              │  GlobalBlockHeader (Merkle aggregate)│
              │  BridgeState + ReplayNonceStore      │
              │  Global account state + Nonce Rule   │
              │  BudZKVM proofs (BudZero, in-tree)   │
              └─────────────────────────────────────┘
                                |
        ┌───────────────┬───────┴────────┬────────────────┐
        v               v                v                v
   B.U.D. storage   BNS (.bud)     SocialFi /       AI Inference
   network          name registry  Digital Buds     Verifier
```

**Crate layout (implementation reality, not aspiration):**

| Path | Role |
|------|------|
| `src/consensus/` | PoW · PoS · PoA engines |
| `src/domain/` | Domain registry, finality adapters |
| `src/cross_domain/` | Bridge, messages, replay protection |
| `src/chain/` | Blockchain, finality (BLS/QC), snapshots |
| `src/execution/` | Transaction executor + BudZKVM host |
| `src/core/governance.rs` | Validator-only, stake-weighted governance |
| `src/rpc/` | JSON-RPC (auth, IP policy, CORS, rate limits) |
| `src/crypto/` | Ed25519, BLS, Dilithium, PKCS#11 |
| `src/storage/` | **B.U.D.** — manifests, deals, challenges |
| `src/bns/` | **BNS** — `.bud` name registry |
| `budzero/` | BudZKVM: ISA, VM, compiler, state, STARK prover |

---

## 4. The Multi-Consensus Settlement Model

### 4.1 Consensus Domains

A **Consensus Domain** is any independent chain or execution environment with its own rules. Domains are first-class registry entries:

- **Identity:** unique `DomainId` + non-zero operator address bonded with a minimum stake. Registrations without operator bond are rejected.
- **Kind:** `ConsensusKind::PoW | PoS | PoA | BFT | ZK | Custom(String)` — the `Custom` variant is a deliberate forward-compatibility escape hatch (a future `ConsensusKind::Ai` is architecturally plausible).
- **Lifecycle:** `Active → Frozen → Retired`. A sovereign domain can exit without permission (see §7, CBDC/digital-sovereignty use case).

### 4.2 DomainCommitment — the unit of settlement

A domain does not push transactions to Budlum; it proves **state transitions**:

```
DomainCommitment {
    domain_id,                 // source of the update
    domain_height,             // committed block height
    state_root,                // resulting domain state
    state_updates,             // account nonce/balance deltas
    finality_proof_hash,       // consensus-specific proof reference
    parent_domain_block_hash,  // chain-link to previous commitment
    validator_set_hash         // binds commitment to registered set
}
```

Raw commitment submission is **disabled** on public RPC and production paths. Operators must present `VerifiedDomainCommitment`s whose proof hashes match the embedded commitment and whose adapter finalizes under the registered configuration.

**Adapter hardening (per consensus):**

- **PoW:** requires confirmation depth *and* non-zero total work; mint requires a bounded, recomputed **`pow-header-chain-v1`** proof (contiguous headers, recomputed hash/link/root/difficulty/work). Legacy declared-depth proofs are mint-gated.
- **PoS:** finality certificate verified against the validator snapshot; snapshot/certificate/commitment hashes bound to the registered `validator_set_hash`.
- **PoA / BFT / ZK:** quorum and proof-hash models with mismatch enforcement; PoA leader selection uses hash-mix (not pure round-robin).

### 4.3 GlobalBlockHeader — one cryptographic fact

The Settlement Layer maintains a `GlobalBlockHeader`: a Merkle aggregate of all verified domain commitments, with **deterministic timestamps** (repeated builds over identical state yield identical hashes) and **atomic persistence** — commitment acceptance and domain-height updates are written in a single batch, so restart recovery can never observe a half-committed settlement transition.

### 4.4 Global shared-state safety — the Nonce Invariant

Cross-domain double-spend prevention is enforced by a strict ordering rule:

$$Account\_{nonce}^{Global} < Commitment\_{nonce}^{Domain}$$

A commitment is valid only if its account nonce strictly exceeds the current global nonce. Out-of-order commitments arriving over gossip are buffered in a `pending_buffer` and applied when the gap fills; restart replay only advances nonces, never rewinds them.

### 4.5 Deterministic conflict resolution & Byzantine handling

- **Same nonce, two domains:** first commitment to reach the settlement registry wins; the second is rejected before touching state.
- **Same domain, same height, different hash (equivocation):** the domain is **globally frozen** — no further commitments are ever accepted — and conflicting commitments are stored as evidence. The operator bond provides the economic slashing hook.
- **Validator-level equivocation** (e.g., PoS double-signing) is handled separately: `SlashingEvidence` is detected, gossiped as a top-level network message, included by later block producers, and applied as stake slashing in execution.

### 4.6 Networking & convergence

Commitments propagate over a **libp2p gossipsub** mesh with idempotent re-submission semantics. The v2 protobuf transport is **lossless**: all existing transaction variants — including the three AI variants — round-trip through an exhaustive `TryFrom` fail-closed mapping; the old silent `Transfer` fallback has been removed.

---

## 5. Post-Quantum Finality

Budlum's finality gadget is **hybrid by construction**:

- **BLS aggregate signatures** — current-era security and compact quorum certificates.
- **Dilithium5 (ML-DSA) QC blobs** — post-quantum security per NIST's standardized lattice family.
- **Both are mandatory on the finality path.** Security degrades to neither "classical-only" nor a bolt-on PQ sidecar: an attacker must break both primitives.

**Key management.** Consensus signing supports **PKCS#11 HSMs**; disk-backed validator keys (BLS + PQ material) are rejected on mainnet until vendor-native HSM paths for those secrets exist (Phase 2.1 policy/tooling complete; vendor-native BLS/PQ HSM verification remains a declared audit item). BLS keypair loading validates G2 encoding and checks `pk = g·sk`. There is deliberately **no social recovery**: if the HSM key is lost, the account is locked forever — maximum security is a stated constitutional choice.

**Why this matters (Y2Q).** Incumbent chains must eventually execute a coordinated, all-node hard fork to swap signature schemes — historically the highest-risk operation a chain can perform. A system built natively on Budlum's settlement record crosses into the quantum era with zero protocol interruption.

---

## 6. Cross-Domain Bridge — Lock → Mint → Burn → Unlock

The 2022–24 bridge era proved that ad-hoc bridges are the ecosystem's softest target ($2.5B+ lost). Budlum treats bridging as a **core protocol lifecycle with proof gates**, not a contract:

1. **Registration** — assets require a registered, bridge-enabled source domain.
2. **Lock** — distinct registered source/target domains, non-zero amount, expiry height strictly after the source event height.
3. **Mint** — requires `expected_block_hash` matching; PoW-originated mints require the recomputed `pow-header-chain-v1` proof (§4.2). Legacy proof formats remain valid for accounting but are **mint-gated**.
4. **Burn / Unlock (return leg)** — raw burn and raw unlock paths are disabled. Return requires a target-domain `BridgeBurned` event committed into settlement and **verified through its event Merkle proof** (`bud_burnBridgeTransferWithEvent`, `bud_unlockBridgeTransferVerified`).

A `ReplayNonceStore` makes every cross-domain message single-use. Per the Constitution, **inbound bridging is zero-upfront-fee**: if the source chain or relayer charges a fee, it is deducted in-flight from the arriving asset, so a new user needs no pre-existing $BUD to enter the network.

---

## 7. BudZKVM & BudZero — Verifiable Execution

**BudZero** is Budlum's zkVM workspace, integrated in-tree (since Phase 0.37) at a one-commit compatibility boundary: `bud-isa`, `bud-vm`, `bud-compiler`, `bud-state`, `bud-node`, `bud-proof` crates.

- **ISA with explicit activation gates.** `VerifyMerkle` is gated off in the Production ISA until the 64-depth Merkle-soundness gate (Z-B Commit 3.5) turns green — a deliberate fail-closed choice, with the gate now flippable by configuration (`BUDLUM_VERIFY_MERKLE`) at genesis ceremony time rather than by code change.
- **STARK trace layout is documented and boundary-tested** (`TRACE_WIDTH = 414` columns; AIR/public-input alignment retained).
- **Overflow-safe arithmetic** (u128 paths), Poseidon-based Merkle rounds.
- **Deterministic benchmarking** harness records reproducible proof-time/size baselines for optimization tracking.

**The honest status, by design:** valid 64-depth soundness is *partial / production-gated*. Whitepaper claims must match code claims — so this paper claims exactly what the repo claims.

### 7.1 AI Inference Verification (native primitives)

Budlum extends the execution model to the **agent economy** with first-class types:

- `RoleId::AI_VERIFIER (6)` joins the role set (alongside `STORAGE_OPERATOR (5)`).
- `AiModelSpec`, `AiInferenceRequest`, `AiInferenceResult`, `AiInferenceOutcome` are protocol-level structures.
- An `AiRegistry` keeps model state with a deterministic state root folded safely into the global account state; the executor integrates model registration, inference requests, and verifier attestation.

Combined with BudZKVM STARK proofs, an agent's claim — *"model M on input X produced output Y"* — becomes economically settleable and cryptographically attestable on-chain.

---

## 8. B.U.D. — Broad Universal Database

B.U.D. is Budlum's permissionless **decentralized storage network** (currently devnet-only by explicit policy).

- **Actors:** storage operators are permissionless (`RoleId::STORAGE_OPERATOR`); deals, replicas, challenges and outcomes are ledger entries (`ContentManifest`, `ContentId`, `StorageRegistry`).
- **Economics:** deal fees, operator bonds, missed-challenge finalization, operator reward accrual, and a slashed-bond ledger are accounted on-chain; storage maintenance (challenge issuance) is automated in the chain actor.
- **Open access:** 9 storage RPCs (register/get manifests, deals by manifest/shard, open/answer challenges, outcomes) are servable by **any** node. Anti-spam is economic (opener bond > 0), not administrative.
- **Data sovereignty rule:** no critical function depends on "a service run by the Budlum team." No whitelist / admin / pause / freeze hooks.

**Integrity roadmap with fake-green ban.** The interim `RetrievalChallenge` is **explicitly not Proof-of-Storage** — an operator could pass while keeping only the requested byte-range. The full proof (Phase roadmap §8.3 vision) is bound to BudZKVM `VerifyMerkle` + the 64-depth SMT production gate; until that gate passes, B.U.D. makes **no** proof-of-storage claim.

---

## 9. BNS — the `.bud` Name Registry

BNS provides human-meaningful names for the Budlum ecosystem, resolving wallets, content manifests (CIDs), dApps, and D-Web sites.

- **First Come, First Served** — absolute name rights on registration; no trademark arbitration layer.
- **Sub-BNS market is parent-controlled** (`x.ayaz.bud` governed by `ayaz.bud`'s owner).
- **Premium tiers** can grant a *Verified* badge (annual high-tier payment), pricing squatting resistance into the fee schedule.
- The interface layer (budlum.xyz search, `bud.scan`) resolves `.bud` natively, rendering B.U.D.-stored sites without traditional DNS — a full **D-Web** path.

Registry integrity is enforced by name-locked CI gates (duplicate live registrations are rejected as `NameTaken`), keeping the skeleton honest while extension proceeds under separate instruction.

---

## 10. SocialFi & Digital Buds — Content as Property

Budlum's social layer binds content to NFTs, called **Digital Buds** (Dijital Tomurcuk). The constitutional model:

- **NFT = ownership, control, and kill-switch.** Burning via `NftBurn` triggers *hard pruning*: the linked B.U.D. shards are physically deleted from operator nodes. The right to be forgotten is an owner right, enforced cryptographically rather than by platform policy.
- **Content portability by ownership.** Transferring the NFT moves the content, its visibility authority, and **all future earnings** to the new owner's profile/feed.
- **Spam resistance is economic:** every mint/post pays a fee. Data is **permanent by default** until the owner burns it; a **Self-Host** option lets rent-averse users serve their shards from their own node (incl. mobile) with full protocol-level resolvability.
- **Selective encryption:** each post is individually *Public* or *Encrypted*.
- **Luminance ("Işık Şiddeti") ranking.** Every NFT starts at 1 cd (candela) and gains/loses light through organic interaction (dwell time, "sparkling"/"darkening") — no engagement-maximizing black-box algorithm.
- **Moderation by community vote**, with a constitutional **DAO Halt** emergency brake for critical-failure scenarios.

---

## 11. $BUD Token Economics

**$BUD is the single utility token** of the ecosystem. Per the Digital Constitution, its flows:

| Flow | Sink / source |
|------|----------------|
| **Gas & fees** — transact, BNS registration, SocialFi mints, data-marketplace access | Paid in $BUD (MIN_TX_FEE enforced in execution) |
| **Storage emission** — the *majority* of new issuance goes to B.U.D. operators (storage-provider-heavy rewards) | Operators earning for availability deals/challenges |
| **Relayer rewards** — protocol mints $BUD to universal relayers; inbound relayers take a small in-flight fee portion | Bridge/relayer service incentive |
| **Boost (NftBoost)** — split: **4%** B.U.D. storage operators · **16%** content creator/context origin · **80%** protocol | Protocol share credited to the treasury/pool if configured, otherwise **burned**; B.U.D. share distributed weighted by deal fee |
| **AI data marketplace** — permissioned access; AI agents pay users in $BUD for data rights | Creator/user income |

Also: physical plug-&-play nodes purchasable in $BUD; $BUD "boosters" buy storage-access speed/priority for specific CIDs.

> **Honest parameter note (no-invented-numbers rule).** Final monetary parameters — total supply, emission curve, genesis allocation, fee floor values — are set at the **mainnet genesis ceremony** per `MAINNET_GENESIS_CEREMONY.md`. This whitepaper deliberately does not publish supply figures that are not yet sealed constants; publishing unsealed numbers would violate the project's own evidence-first discipline.

---

## 12. Governance

- **Validator-only, stake-weighted proposals** covering fees, rewards, registry parameters, and slashing; quorum-based finalization; bounded parameter validation to prevent hostile value injection.
- **Constitution as law:** §1–7 decisions (content/moderation, identity/recovery, data economics, ecosystem/relayer, AI access, hardware, BNS rules) are canonical and implemented across the stack.
- **Emergency:** DAO Halt — a community vote can temporarily halt the chain during critical failure; the no-rollback principle and archive fail-closed policy bound the response space.
- Mainnet activation flips (e.g., `verify_merkle`) are **config-driven ceremony decisions**, not code edits.

---

## 13. Security Architecture

Iterative hardening (Phase 0.16 → 12.5), selected highlights:

- **DoS ordering:** cheap transaction checks run *before* signature verification.
- **Bridge forgery gates:** `expected_block_hash` on mint; bounded recomputed PoW header chains; legacy proofs mint-gated; verified return-leg only.
- **Crypto hygiene:** BLS keypair load validates G2 encoding + public-key derivation; constant-time API-key comparison; HSM-required mainnet consensus signing.
- **RPC hardening:** public auth fail-closed; operator methods mode-gated/localhost-only; `X-Real-IP` honored only with configured `trusted_proxies`; rate limits, CORS, dual-listener model; Prometheus metrics (request-latency histograms, rate-limit counters).
- **Supply-chain hygiene:** `fuzz/` targets, dependency audit script, SBOM generation, Dependabot security updates merged only on full-green gates; archive policy fail-closed; atomic verified backup + restore drills; documented runbooks (production, archive/restore).
- **Formal methods started, honestly scoped:** a TLA+ skeleton (`MultiConsensus.tla`) models safety (NoRollback, MonotonicHeight, SlashValidator) and liveness under honest quorum — labeled a *skeleton*, full formal verification reserved for external audit.

**Evidence dashboard (all CI-evidenced, module gates name-locked):**

| Module | Tests | Gate |
|--------|-------|------|
| Budlum Core | **755 lib** | fmt + clippy `-D warnings` + test |
| BudZero / BudZKVM | **124** | BudZero workspace gate |
| B.U.D. | **12 mandatory** (9 invariant + 3 e2e) | `check-bud-e2e.sh` + E2E Invariants job |
| BNS (`.bud`) | **8** | `check-bns-gate.sh` name-locked |

*(Core's 755 includes B.U.D./BNS suites via the shared lib; module gates are additionally enforced per the "total line never replaces a module line" reporting rule.)*

> This section is **not** a substitute for a professional external audit — see §15.

---

## 14. Ecosystem Interface — the Visible Network

- **budlum.com** — gateway: vision, docs, onboarding.
- **budlum.xyz** — the network as a *navigable digital territory*: an infinite grid where each square is a wallet, clusters are dApps. Exploration is spatial — "Minecraft-style."
- **bud.scan search & +Context Maps:** universal lookup for addresses, NFT CIDs, `.bud` names, D-Web sites; wallet Context Maps render transaction-relationship graphs; token *Bubble Maps* render distributions/whale structure.
- **Budlum Hub:** open, democratic dApp registration.
- **Universal Relayer ("master key"):** Budlum wallets/HSMs sign and execute on external chains (EVM, Solana, …) paying gas in $BUD, protocol-minted relayer incentives, and mandatory multi-device/multi-sig approval for critical cross-chain operations.
- **Mobile-first sovereignty:** phones are first-class storage and validation nodes; users can self-host their own B.U.D. shards.

---

## 15. Roadmap & Mainnet Blockers

**Completed (evidence-closed):** multi-consensus domains; BLS+Dilithium hybrid finality; BLS finality protocol prevote/precommit; bridge lifecycle incl. verified return path and PoW mint gates; settlement atomicity; sync hardening; PKCS#11 Ed25519 signer; RPC dual listener; P2P hardening; Snapshot V2 + archive policy; Prometheus observability; docker/systemd deployment; ConsensusStateV2 migration skeleton (fail-closed); fuzz/dependency-audit/SBOM tooling; audit checklist; personas (user/dev/enterprise-PoA); BudZero in-tree + performance baseline harness; B.U.D. Phase 1–2 + skeleton 5 (devnet); BNS skeleton; TLA+ skeleton; AI inference verifier primitives.

**Open mainnet blockers (declared, un-claimed):**

1. **Independent external security audit** — checklist ready; *no audit has been performed*.
2. **Vendor-native BLS/PQ HSM mechanism verification.**
3. **Z-B 64-depth Merkle soundness** — production gate for `VerifyMerkle` and thus for B.U.D. true Proof-of-Storage (Phase 3+).
4. **Privacy layer** — research stage.
5. **AI execution layer beyond primitives** — research/integration stage.
6. Full formal verification beyond the TLA+ skeleton.

**Activation path:** B.U.D. mainnet inclusion is evaluated after its readiness step closes; genesis-side feature flips (e.g., `verify_merkle=true`) are config-driven ceremony decisions with checklist verification (`GENESIS_FLIP_CHECKLIST`).

---

## 16. Risk Factors & Disclosures

- **Pre-audit software.** Budlum is a controlled public-devnet candidate. Do **not** use for real-value production traffic.
- **Research-stage components.** AI layer, privacy layer, and B.U.D. Phase 3+ are forward-looking statements with material execution risk.
- **No-recovery key model.** The maximum-security zero-recovery stance transfers key-loss risk fully to the holder. Multi-device/HSM practices are mandatory for material balances.
- **Parameter pending.** Monetary parameters are unsealed until the genesis ceremony; any third-party quoting a "supply" should be treated as unauthorized.
- **Regulatory surface.** Settlement of sovereign domains (CBDC use case) and paid AI data markets touch evolving regulation; deployments must assess jurisdiction-specific compliance.
- **Honest-label risk acceptance.** Known interim limitations (e.g., retrieval challenges ≠ proof-of-storage) are accepted and documented rather than hidden; users must read module READMEs before relying on claims.

---

## 17. Conclusion

The industry has spent a decade choosing between security models: PoW *or* PoS, permissioned *or* public, sovereign *or* interoperable, transparent *or* private. Budlum's wager is that these are false dichotomies. A settlement layer that verifies finality instead of dictating consensus can host all of them at once — quantum-ready from genesis, bridged by proofs rather than promises, extended by verifiable computation, and humane in its economics: content you can own, move, and delete; storage you can sell from your phone; data that pays you when AI consumes it.

What distinguishes this whitepaper from the genre's habit is what it refuses to do: claim tests it cannot show, claim audits that have not happened, claim proofs whose gates are still gated. The numbers here are CI-evidenced; the gaps here are named. That is not modesty — it is the protocol's own fail-closed philosophy applied to prose.

**Budlum does not replace the world's chains. It settles them.**

---

## References (in-repository)

- `docs/01_multi_consensus_settlement.md` — settlement architecture detail
- `docs/03_paradigma_analizi.md` — seven paradigm shifts, strategic rationale
- `docs/BUDLUM_CONSTITUTION.md` — social & economic framework (canonical)
- `docs/BUDLUM_ECOSYSTEM_INTERFACE.md` — grid/search/context-map UX
- `docs/BUDZVM_TRACE_LAYOUT.md`, `docs/BUDZERO_DERIN_DENETIM_ARENA3.md` — zkVM trace & audit notes
- `docs/MAINNET_READINESS.md`, `docs/MAINNET_GENESIS_CEREMONY.md` — launch path
- `docs/operations/*` — runbooks (production, archive/backup, finality live-path)
- `src/bns/README.md`, `src/storage/README.md`, `budzero/README.md` — module-level honest status

*Whitepaper prepared July 2026 from live repository state (Phase 10). MIT-licensed project; this document mirrors the repository's evidence rules: total lines never replace module lines, and no claim without a gate.*
