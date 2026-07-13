# Budlum Documentation

Choose a language:

- [Turkce dokumantasyon](tr/book/README.md)
- [English documentation](en/book/README.md)

Current production-readiness status:

- [Turkce production hardening durumu](tr/book/ch12_production_hardening.md)
- [English production hardening status](en/book/ch12_production_hardening.md)

Specialised deep-dives:

- [Post-quantum security architecture (Tur 8)](03_post_quantum_security.md) — Dilithium5 integration, hybrid roadmap, threat model

## Sibling Project: BudZKVM

`budlum-core` consumes the ZK execution environment from the sibling
[`BudZKVM` (lubosruler/BudZero)](https://github.com/lubosruler/BudZero)
repository via path dependencies on `bud-isa`, `bud-vm`, and
`bud-proof`. See the [BudZKVM README](https://github.com/lubosruler/BudZero#readme)
for the ZK-side language surface, prover architecture, and Tur 1–8
milestone log.
