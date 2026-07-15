use crate::core::address::Address;
use crate::storage::content_id::ContentId;
use serde::{Deserialize, Serialize};

/// Budlum NFT — Native support for SocialFi posts and D-Web content.
/// Every SocialFi post is a permanent, owner-controlled NFT.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Nft {
    pub id: u64,
    pub owner: Address,
    /// Link to the actual content in B.U.D. (image, video, post text).
    pub content_id: ContentId,
    pub minted_at_epoch: u64,
    /// Optional BNS name linked to this NFT at mint time (e.g. "ayaz.bud").
    pub author_name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum NftError {
    #[error("NFT not found")]
    NotFound,
    #[error("Not the owner")]
    NotOwner,
    #[error("Duplicate ID")]
    DuplicateId,
}
