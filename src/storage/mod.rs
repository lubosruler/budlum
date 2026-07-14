//! Storage layer.
//!
//! Two intentionally separate namespaces live in `src/storage/`:
//!
//! * [`db`] / [`traits`] — the *node-local* key-value store (sled) that
//!   holds chain state, accounts, blocks, etc. Pre-existing, not touched
//!   by Tur 14.
//!
//! * [`content_id`] / [`manifest`] — the *B.U.D. on-chain content-addressing
//!   primitives* introduced by Tur 14 (Faz 2 + Tur 14.5 §2.1). These are
//!   pure data shapes — no I/O, no admin hooks, no team-server dependency
//!   (Tur 14.5 plan §0.5).
//!
//! The domain-level deal / challenge accounting lives in
//! `crate::domain::storage_deal::StorageRegistry` (kept under
//! `domain/` because the data shapes it owns are consensus types, not
//! transport types).

pub mod content_id;
pub mod db;
pub mod manifest;
pub mod traits;

pub use content_id::{ContentId, DEFAULT_CHUNK_SIZE_BYTES};
pub use manifest::{manifest_id_from_shards, ContentManifest, ShardRef};
