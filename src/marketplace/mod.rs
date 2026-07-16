use crate::core::address::Address;
use crate::storage::content_id::ContentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Phase 5 §5.5: AI Data Marketplace — Economic layer for user-to-AI data sales.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataOffer {
    pub id: u64,
    pub seller: Address,
    pub cid: ContentId,
    pub price: u64, // Price in $BUD
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketplaceRegistry {
    pub offers: BTreeMap<u64, DataOffer>,
    pub next_offer_id: u64,
}

impl MarketplaceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_offer(&mut self, seller: Address, cid: ContentId, price: u64) -> u64 {
        let id = self.next_offer_id;
        let offer = DataOffer {
            id,
            seller,
            cid,
            price,
            active: true,
        };
        self.offers.insert(id, offer);
        self.next_offer_id += 1;
        id
    }

    pub fn close_offer(&mut self, id: u64, caller: &Address) -> Result<(), String> {
        let offer = self.offers.get_mut(&id).ok_or("Offer not found")?;
        if &offer.seller != caller {
            return Err("Not the seller".into());
        }
        offer.active = false;
        Ok(())
    }

    pub fn get_offer(&self, id: u64) -> Option<&DataOffer> {
        self.offers.get(&id)
    }
}
