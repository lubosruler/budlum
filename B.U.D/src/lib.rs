//! B.U.D. — Broad Universal Database workspace boundary.
//!
//! Planned source ownership: content addressing, manifests, storage-domain
//! parameters and deal/challenge economics. Core durable block storage remains
//! in `budlum-core`; it is not a B.U.D. module.

pub mod content_id;
pub mod manifest;
pub mod storage_params;
pub mod deals;
pub use content_id::ContentId;
pub use manifest::{ContentManifest, ShardRef};
pub use storage_params::StorageDomainParams;
pub use deals::{ChallengeOutcome, ChallengeResult, DealStatus, RetrievalChallenge, RetrievalChallengeRequest, RetrievalResponse, StorageDeal, StorageEconomicsParams, StorageError, StorageRegistry};
