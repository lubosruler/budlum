//! B.U.D. (Broad Universal Database) — P2P Storage Node
//!
//! This crate implements the P2P storage backend for the B.U.D. network,
//! providing content-addressed storage helpers, discovery key mapping, and
//! a Bitswap-like request/response codec.
//!
//! **Honesty note (DENETLEYİCİ):** this crate currently ships **unit-level**
//! store/codec/discovery helpers. Full libp2p `Swarm` + `kad::Behaviour`
//! wiring is not yet in this package — do not claim live DHT/Bitswap network
//! integration until a NetworkBehaviour is connected.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │              BudNode                     │
//! │                                          │
//! │  ┌─────────────┐  ┌──────────────────┐  │
//! │  │ ContentStore │  │ ContentDiscovery │  │
//! │  │ (store.rs)   │  │ (discovery.rs)   │  │
//! │  └──────┬───────┘  └────────┬─────────┘  │
//! │         │                    │            │
//! │  ┌──────┴────────────────────┴─────────┐  │
//! │  │         BudBitswap (bitswap.rs)     │  │
//! │  │    libp2p request-response codec    │  │
//! │  └─────────────────┬───────────────────┘  │
//! │                    │                      │
//! └────────────────────┼──────────────────────┘
//!                      │
//!              libp2p swarm (kad + noise + yamux)
//! ```
//!
//! # B.U.D. Vision Reference
//!
//! - `budlum-xyz/B.U.D./BUD_Merkeziyetsiz_Depolama_Vizyonu.md` §2 (mantık örtüşmesi)
//! - §7 (bugün kodda OLMAYANLAR — Bitswap, içerik routing)
//! - Faz 2 (içerik adresleme)

pub mod bitswap;
pub mod discovery;
pub mod store;

pub use bitswap::{BitswapCodec, BitswapRequest, BitswapResponse, BudBitswap};
pub use discovery::ContentDiscovery;
pub use store::{ContentStore, MemoryContentStore};
