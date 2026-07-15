pub mod types;

use crate::core::address::Address;
use crate::hub::types::{AppCategory, AppRecord, HubError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HubRegistry {
    /// app_id -> record
    pub apps: BTreeMap<u64, AppRecord>,
    pub next_app_id: u64,
}

impl HubRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_app(
        &mut self,
        name: String,
        developer: Address,
        category: AppCategory,
        website_url: String,
        manifest_id: Option<crate::storage::content_id::ContentId>,
        epoch: u64,
    ) -> u64 {
        let id = self.next_app_id;
        let record = AppRecord {
            id,
            name,
            developer,
            category,
            website_url,
            manifest_id,
            registered_at_epoch: epoch,
            verified: false,
        };
        self.apps.insert(id, record);
        self.next_app_id += 1;
        id
    }

    pub fn update_app(
        &mut self,
        id: u64,
        caller: &Address,
        new_url: Option<String>,
        new_manifest: Option<crate::storage::content_id::ContentId>,
    ) -> Result<(), HubError> {
        let app = self.apps.get_mut(&id).ok_or(HubError::NotFound)?;
        if &app.developer != caller {
            return Err(HubError::NotDeveloper);
        }
        if let Some(url) = new_url {
            app.website_url = url;
        }
        if let Some(manifest) = new_manifest {
            app.manifest_id = Some(manifest);
        }
        Ok(())
    }

    pub fn verify_app(&mut self, id: u64) -> Result<(), HubError> {
        let app = self.apps.get_mut(&id).ok_or(HubError::NotFound)?;
        app.verified = true;
        Ok(())
    }

    pub fn list_apps(&self) -> Vec<AppRecord> {
        self.apps.values().cloned().collect()
    }
}
