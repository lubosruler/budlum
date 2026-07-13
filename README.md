# Budlum Core

> The Universal Settlement Layer for the post-quantum, multi-consensus
> world. A controlled public-devnet candidate for Layer-1 blockchain
> research: modular, deterministic, and proof-driven.

[![CI](https://img.shields.io/badge/CI-success-brightgreen)](https://github.com/lubosruler/budlum/actions)
[![Tests](https://img.shields.io/badge/tests-423-blue)](https://github.com/lubosruler/budlum)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.94.0-orange.svg)](https://www.rust-lang.org/)

---

> **Controlled Public Devnet Candidate (v0.3-dev)**
>
> Budlum Core is suitable for controlled public devnet experiments with
> clear risk disclaimers. It is **NOT** audited mainnet software, has not
> completed professional security review, and should **NOT** be used for
> financial transactions or production applications carrying real
> value.

---

## The 2035 Vision

Every blockchain today is an island. Bitcoin, Ethereum, Solana, every
CBDC pilot, every corporate chain — they all disagree on one
fundamental question: **how should value be settled across chains
without one side trusting the other?**

Budlum answers that question by separating *production* from
*settlement*:

* Each chain keeps its own consensus — PoW, PoS, PoA, BFT, ZK, or
  anything custom. **How** value is produced stays local.
* Budlum holds the cryptographic proof that the work was done. The
  finality proof is the **same** regardless of which consensus
  produced it. **Whether** value is real becomes universal.

In one sentence: *Hangi consensus kullandığın önemli değil. Sen
state'ini doğrulayabilirsin — budlum seni doğrular.* (Which consensus
you use does not matter. You can verify your own state — Budlum
verifies you.)

This is the analogue of TCP/IP for value. TCP/IP does not care how a
network is built; it just carries packets. Budlum does not care how a
chain produces blocks; it just settles the proofs.

The strategic analysis behind this vision is documented in
[`BUDLUM_PARADIGMA_ANALIZI`](/lubosruler/budlum/blob/main/docs/03_paradigma_analizi.md)
in this repository.

---

## The Seven Paradigm Shifts

Budlum's architecture is designed to collapse seven structural problems
that every other blockchain inherits from the pre-quantum,
single-consensus era.

| # | Problem (today) | Budlum's shift |
| --- | --- | --- |
| 1 | ECDSA + Ed25519 will be broken by quantum computers around 2030-2035 ("Y2Q" problem). Nation-states are harvesting encrypted traffic today to decrypt later. | Dilithium5 + BLS **hybrid** finality is in the consensus core, not bolted on. Quantum-safe from day one. |
| 2 | 20,000+ blockchains cannot talk to each other. Polkadot/Cosmos/LayerZero all require a single consensus model. | **Universal Settlement Layer** — each chain runs on its own consensus; Budlum verifies the proofs. |
| 3 | 130+ countries are piloting CBDCs in isolation. A Turkish citizen cannot use digital lira in Germany. | Each CBDC is a `ConsensusDomain`. `BridgeState` + `ReplayNonceStore` + `CrossDomainMessage` provide trustless cross-sovereign settlement. |
| 4 | TradFi (closed, fast, permissioned) and DeFi (open, slow, risky) cannot interoperate. | `ConsensusKind::PoA` (banks) and `ConsensusKind::PoS` (public) coexist in the same settlement layer, anchored to the same `GlobalBlockHeader`. |
| 5 | AI agents cannot transact value safely or prove the provenance of their decisions. | `ConsensusKind::Custom` (designed for future) + BudZKVM STARK proofs make any decision — including AI decisions — verifiably on-chain. |
| 6 | National digital identity, land registry, social security systems are isolated. Cross-border movement is impossible. | Each nation keeps its own domain (`DomainStatus::Active`/`Frozen`/`Retired`); data is shared via cross-domain messages without sovereignty loss. |
| 7 | Cross-chain bridge hacks stole $2.5B+ in 2022-2024 (Ronin $625M, Wormhole $320M, Nomad $190M). All were trust-minimized at best, never trustless. | `BridgeState`'s lock → mint → burn → unlock lifecycle is end-to-end cryptographically proven. No human approval, no multisig, no oracle. |

---

## Architecture

```
                       CONSENSUS DOMAINS (the producers)
   +-----------+   +-----------+   +-----------+   +-----------+
   |    PoW     |   |    PoS     |   |    PoA     |   |   Custom   |
   |  domain    |   |  domain    |   |  domain    |   |  domain    |
   +------+-----+   +------+-----+   +------+-----+   +------+-----+
          |                |                |                |
          |   DomainFinalityAdapter produces a proof for each  |
          |   block: BLS aggregate signature + Dilithium5 QC     |
          v                v                v                v
   +--------------------------------------------------------------+
   |              BUDLUM SETTLEMENT LAYER (this repo)            |
   |                                                              |
   |   GlobalBlockHeader (one per height) commits every          |
   |   domain's proof into a single settlement record.           |
   |                                                              |
   |   +------------------------------------------------------+   |
   |   |  PoW / PoS / PoA / BFT / ZK consensus engines          |   |
   |   |  (--lib budlum-core --bin budlum-core)                |   |
   |   +------------------------------------------------------+   |
   |   +------------------------------------------------------+   |
   |   |  Permissionless Registry (validators / relayers /     |  |
   |   |  provers) — Tur 5 — stake == registration, no whitelist|  |
   |   +------------------------------------------------------+   |
   |   +------------------------------------------------------+   |
   |   |  Cross-Domain Bridge (BridgeState, CrossDomainMessage,| |
   |   |  ReplayNonceStore, DomainCommitmentRegistry)         |  |
   |   +------------------------------------------------------+   |
   |   +------------------------------------------------------+   |
   |   |  ZK execution via sibling BudZKVM (bud-isa, bud-vm,    |  |
   |   |  bud-proof crates) -- end-to-end STARK proofs          |  |
   |   +------------------------------------------------------+   |
   |                                                              |
   |   libp2p / GossipSub / JSON-RPC (public + operator) /       |
   |   Prometheus metrics / Snapshot V2 / Prometheus             |
   +--------------------------------------------------------------+
                                |
                                v
                  Verified, settled, audit-able value transfer
```

---

## What This Repo Is

Budlum Core (`budlum-core`) is the Rust L1 implementation of the
Universal Settlement Layer. It is one of two sibling repositories:

* `lubosruler/budlum` (this repo) — the L1 settlement engine.
* `lubosruler/BudZero` — the ZK execution layer (BudZKVM).

The two are consumed via Cargo path dependencies; `budlum-core`'s
`Cargo.toml` references `../BudZKVM/bud-isa`, `../BudZKVM/bud-vm`, and
`../BudZKVM/bud-proof`.

---

## Devnet Candidate Features (v0.3)

### Multi-Consensus Settlement (Model B)

* **Verified-Only Commitments** — RPC rejects raw domain commitments;
  settlement updates arrive as `VerifiedDomainCommitment` with a
  matching finality proof hash.
* **Adapter Hardening (all real verification)** — PoW binds the proof
  to the commitment block hash + confirmation depth + internally
  consistent declared cumulative work; PoS and BFT verify a real BLS
  commit certificate against the validator snapshot; ZK resolves
  through the STARK-verified `ProofClaimRegistry`; PoA verifies real
  ed25519 signatures with a count-based quorum, fully isolated from
  the permissionless stake registry.
* **Parent-Linked Domain History** — commitments whose
  `parent_domain_block_hash` does not link to the last committed
  domain block are rejected.
* **Strict Nonce Invariant** — stale or equal nonce updates are
  rejected before durable insertion.
* **Byzantine Resilience** — global state convergence verified via an
  18-test "Chaos Matrix" under simulated partitions and delays.
* **Equivocation Immunity** — protocol-level detection and global
  freezing of conflicting domains; duplicates remain idempotent.
* **Atomic Settlement Persistence** — commitment insertions and
  domain height/hash updates persisted in one storage batch.

### `$BUD` Tokenomics

* **Fixed Supply** — 100,000,000 `$BUD`, 6 decimals
  (`src/tokenomics/mod.rs`). No `total_supply` field; supply is the sum
  of all balances; there is no inflation mint on the burn paths.
* **Genesis Distribution** — Community 10M / Liquidity 10M / Ecosystem
  20M / Team 20M / Burn Reserve 40M (config-driven `TokenomicsParams`,
  validated to sum to exactly 100M). Wired into genesis via the
  opt-in `GenesisConfig::with_bud_tokenomics()`.
* **Team Vesting (enforced)** — standard cliff + linear schedule;
  transfers from the team account are rejected (`vesting_locked`)
  if they would spend below the still-locked portion.
* **Timed Reserve Burn** — time-triggered (epoch-based, not usage-based)
  annual burn of the 40M reserve, fired automatically at each epoch
  transition (`advance_epoch`); strictly reduces supply, never offset
  by a mint.
* **Metabolic Burn** — a configurable fraction of every transaction
  fee is burned instead of paid to the block producer
  (in `Executor::apply_block`).
* *Out of scope (future work): PoSV consensus, `$LUM` token,
  launchpad/presale.*

### Permissionless Participation Registry

* **No Whitelist / No Approval** — validator, verifier and relayer
  participation is open; the only requirement is bonding stake
  (`src/registry/permissionless.rs`). Security is stake + slashing,
  never permission.
* **Staking == Registration** — applying a `Stake` transaction
  automatically registers the account in the registry.
* **Generic, Extensible Roles** — roles are an open `RoleId` (not a
  hard-coded enum), so future application layers can define new roles
  without changing the registry.
* **Config/Governance Parameters** — minimum stake, unbonding window
  and per-offence slash ratios are `RegistryParams` — tunable, not
  hard-coded (`src/registry/params.rs`).
* **Canonical Slashing Evidence** — a single `SlashingReport` format
  (`src/registry/evidence.rs`) is shared by consensus, RPC and other
  domains. Only consensus-verified reports are actioned;
  externally-submitted unverified reports never cause a slash.
* **Relayer-Gated Cross-Domain Messaging** — relayed
  `CrossDomainMessage` submission (RPC + p2p) requires the sender to
  be an active relayer; system-generated bridge events bypass the
  gate. No whitelist — registration is by bonding stake.
* **Anti-Spam Fee for Slashing Reports** — `bud_submitSlashingReport`
  charges `RegistryParams::slashing_report_fee` (default 10) to the
  reporter, refunded when actionable, burned otherwise.
* **Automatic Liveness Slashing** — `LivenessTracker`
  (`src/registry/liveness.rs`) counts consecutive
  missed-participation epochs; crossing the threshold emits a
  consensus-verified liveness report.
* **Permissionless ZK Prover Bridge** — anyone may submit a BudZKVM
  proof via `CrossDomainMessage`; STARK proofs are self-verifying,
  so no registration is required. A small `proof_submission_fee`
  (default 10) is refunded on valid proofs and burned on invalid
  ones. Registering as a `PROVER` is optional and only grants reward
  eligibility. "First valid wins" per `(domain, height)`.
* **Isolated PoA Membership** — the permissioned PoA domain uses a
  completely separate KYC/approval-gated registry
  (`src/registry/poa_membership.rs`); stake never grants PoA
  authority.
* **RPC** — `bud_registryRegister`, `bud_registryBondRelayer`,
  `bud_registryBondProver`, `bud_submitZkProof`, `bud_registryQuery`,
  `bud_registryActiveMembers`, `bud_submitSlashingReport` — all
  permissionless.

### Verified Cross-Domain Bridge

* **Bridge-Enabled Domains Only** — asset registration requires
  active, registered, bridge-enabled domains.
* **Safe Lock Constraints** — source/target domains must differ,
  transfer amount must be non-zero.
* **Raw Burn/Unlock Disabled** — direct bridge burn and unlock calls
  rejected.
* **Proof-Based Return Path** — funds return only after target-domain
  `BridgeBurned` event is committed and verified.

### BLS + Dilithium5 Hybrid Finality Protocol (v0.3)

The core of the post-quantum story. Both are required simultaneously:

* **`BlsKeypair`** — BLS12-381 keypair integrated into
  `ValidatorKeys` with `sign_bls()` / `verify_bls_sig()`.
* **Signed Votes** — validators produce BLS-signed prevote/precommit
  messages.
* **Auto-Precommit** — periodic loop detects prevote quorum and
  automatically signs + broadcasts precommit.
* **Aggregate Verification** — `FinalityCert` verified with BLS
  pairing: `e(sig, G2_gen) == e(H(msg), agg_pk)`.
* **Dilithium5 QC Blob** — every checkpoint carries a post-quantum
  signature from a quorum of validators. The L1 rejects any
  checkpoint whose QC blob does not verify under Dilithium5.
* **Adversarial Tests** — byzantine equivocation rejection, tampered
  aggregate signature detection, full 4-validator flow.

### RPC Hardening (v0.3)

* **Dual Listeners** — separate public and operator HTTP servers.
* **Trusted Proxy** — only configured proxy IPs may set
  `X-Forwarded-For` for client identification.
* **Per-IP Rate Limiting** — independent token buckets per client IP.
* **Health Endpoints** — `bud_health`, `bud_nodeInfo`.
* **Body/Connection Limits** — public 10MB/500 conn, operator
  50MB/10 conn.

### P2P Hardening (v0.3)

* **Persistent Identity** — P2P keypair survives restarts.
* **Durable Peer Bans** — JSON-persisted, reloaded on startup.
* **mDNS Policy** — per-network (`mainnet`/`testnet` off, `devnet` on).
* **DNS Seed Resolution** — resolves hostnames to multiaddrs at startup.

### Snapshot V2 (v0.3)

* **Canonical V2 Format** — full consensus metadata (epoch, base_fee,
  block_reward, unbonding_queue, cross-domain roots, finality certs).
* **Replay Equivalence** — state root matches original.
* **Chunk-Session Binding** — `session_id` prevents cross-peer chunk
  mixing.
* **V2-First Restore** — startup tries V2 snapshot, falls back to V1.

### Observability (v0.3)

* **Prometheus** — chain height, finalized height, blocks produced,
  transactions, reorgs, mempool, P2P messages/peers.
* **Docker** — multi-stage image, 4-node `docker-compose`.
* **systemd** — production unit file.
* **Fuzzing** — 4 `cargo-fuzz` targets (block, transaction, snapshot,
  consensus header).

### Post-Quantum Architecture (Tur 8)

* **Dilithium5 + BLS hybrid finality** in the consensus core.
* **HNDL threat model** documented in
  [`docs/03_post_quantum_security.md`](docs/03_post_quantum_security.md).
* **Tur 9-14 migration roadmap** — key ceremony, validator key
  rotation, governance parameter addition, hybrid signature envelope.

---

## Verification & Test Coverage

* **Total Tests** — `423` (all passing).
* **Byzantine Chaos Matrix** — 18 scenarios.
* **BLS Finality Tests** — sign/verify, aggregator, equivocation,
  certificate tampering, replay.
* **RPC Security Tests** — auth, CORS, IP filtering, rate limiting,
  trusted proxy.
* **P2P Hardening Tests** — persistent identity, durable ban
  roundtrip, DNS seed resolution.
* **Snapshot V2 Tests** — V2 metadata, replay, serialization,
  PruningManager V2.
* **Metrics Tests** — chain metrics emit, counter increments, encoding.
* **Distributed Devnet Simulation** — 5-node libp2p mesh convergence.
* **BudL Compiler Tests** (sibling BudZKVM) — 9/9 match expressions.

To run the full suite:

```bash
cargo test --lib
```

To run fuzz targets:

```bash
cd fuzz && cargo fuzz run block_deserialize
```

---

## Production Hardening Status

**v0.3-dev** closes 5 of 7 Mainnet blockers. Remaining work: external
security audit, archive-node policy, migration framework, and production
runbooks.

The latest security audit was the **Tur 8 post-quantum + bridge
denetimleri** (see `docs/03_post_quantum_security.md`). The Tur 8 CI
pipeline runs three gates on every push:

1. `cargo fmt --all -- --check`
2. `cargo clippy -D warnings`
3. `cargo test --lib`

Read the book's
[**Production Hardening Status**](docs/en/book/ch12_production_hardening.md)
for the full implementation matrix.

---

## Quick Start

### Requirements

* Rust `1.94.0`
* `protoc` (for `build.rs`)
* Sibling checkout of [BudZKVM](https://github.com/lubosruler/BudZero)

### Build

```bash
git clone https://github.com/lubosruler/BudZero.git
git clone https://github.com/lubosruler/budlum.git infra
cd infra
cargo build --release
```

### Run a Devnet Node

```bash
# Proof of Work
./target/release/budlum-core --consensus pow --difficulty 3 --port 4001

# Proof of Stake
./target/release/budlum-core --consensus pos --min-stake 5000 --db-path ./data/pos_node
```

### Docker 4-node Devnet

```bash
docker compose up -d
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"bud_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

### RPC Usage

```bash
# Health check
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"bud_health","params":[],"id":1}' \
  http://localhost:8545

# Block height
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"bud_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

See the [**Protocol Specification**](SPECIFICATION.md) for the full
API reference.

---

## Research Roadmap

### Completed (Tur 1–7)

* Devnet economic hardening — validator rewards, slashing execution.
* Settlement atomicity — atomic commitment + domain height/hash
  persistence.
* Verified settlement hardening — proof-gated commitments,
  parent-link, strict nonce.
* Verified bridge return path — proof-based unlock.
* Sync hardening — handshake-triggered headers-first.
* PKCS#11 HSM signer — `ConsensusSigner` trait, `Pkcs11Signer`,
  `KeyPairSigner`.
* BLS finality protocol — `BlsKeypair`, signed prevote/precommit,
  auto-precommit.
* RPC dual listener — public/operator, trusted proxies, health
  endpoints, rate limiting.
* P2P hardening — persistent identity, durable bans, mDNS, DNS seeds.
* Snapshot V2 — canonical V2 restore, replay equivalence, chunk
  binding.
* Observability — Prometheus, metrics wiring.
* Deployment — Docker, docker-compose, systemd, fuzz targets.

### Completed (Tur 8)

* **Registry Integration** (Tur 5) — `state.registry/liveness/invalid_votes`
  reactivated, 6 `cfg(false)` suites restored, 408-test baseline.
* **Security Wiring** (Tur 6–7) — snapshot timeout, bridge lock RPC
  removal, auth defaults, keyfile mode, QcBlob quorum.
* **Post-Quantum Architecture** (Tur 8) — Dilithium5 + BLS hybrid
  finality design, HNDL threat model, Tur 9-14 migration roadmap. See
  [`docs/03_post_quantum_security.md`](docs/03_post_quantum_security.md).
* **CI Mainnet-Readiness** (Tur 8.x) — three-gate pipeline
  (fmt + clippy `-D warnings` + test) on every push to `main`.

### In Progress / Planned

* **Tur 9+ — Audit Follow-ups** — see the linked
  `BUDLUM_BUG_REPORT.md` in the L1 issue tracker; latent and live
  security findings.
* **ZKVM BudL Maturity** (tracked in sibling
  [BudZKVM repo](https://github.com/lubosruler/BudZero)):
  match expressions (Tur 8 — done), ADTs/exhaustiveness, witness
  variables, error spans.
* **ZKVM Optimizations** — STARK proof generation performance.
* **Formal Verification** — TLA+ models for settlement convergence.
* **External Audit** — professional security review.
* **Privacy Layer** — Monero/Zcash-style privacy primitives.
* **AI Execution Layer** — AI-assisted protocol automation and
  risk scoring.

Budlum is built for protocol researchers and developers who like
looking under the hood. We welcome technical reviews, protocol design
discussions, and security feedback.

---

## What This Repo Does *Not* Do (Scope Boundaries)

To prevent scope creep, `budlum-core` deliberately excludes the
following:

* **No whitelist / admin-approval / central gate** for validator,
  verifier or relayer roles on the PoW/PoS/BFT domains. Participation
  is **permissionless**: the only requirement is bonding stake (see
  `src/registry/permissionless.rs`). Security comes from stake +
  slashing, never from permission.
* **No fixed or committee-based relayer set.** Anyone can become a
  relayer by staking; there is no hard-coded relayer list.
* **The permissioned PoA domain is an isolated exception, not the
  norm.** PoA membership (`src/registry/poa_membership.rs`) is
  KYC/approval-gated and kept in a completely separate data structure
  from the permissionless registry. PoA's admission rules must not leak
  into other domains, and stake must not grant PoA authority. This
  isolation is enforced by `src/tests/permissionless.rs`.
* **No application-layer logic** (DeEd, SocialFi, B.U.D., Budlum Go,
  DeArt, participation-bank product). This repo is the network / L1
  core only.
* **No bespoke L1↔ZKVM bridge protocol.** Cross-domain interaction
  with BudZKVM goes through the existing `CrossDomainMessage`
  primitive.

---

## Join the Research

### How to Contribute

1. **Star the Repository** — it helps other researchers find our work.
2. **Fork and Experiment** — try building a custom `ConsensusKind`.
3. **Open a Discussion** — have an idea for the Privacy Layer or AI
   Execution?
4. **Report Bugs** — use GitHub Issues for any technical anomalies.

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before participating. For
security-sensitive reports, please use [`SECURITY.md`](SECURITY.md).

---

## License

MIT License. Copyright (c) 2026 The Budlum Developers.
