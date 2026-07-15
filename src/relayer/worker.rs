use crate::chain::chain_actor::ChainHandle;
use crate::core::transaction::{Transaction, TransactionType};
use crate::core::address::Address;
use tokio::sync::mpsc;
use std::sync::Arc;
use tracing::{info, warn, error};

/// ADIM 5 §5.1: Universal Relayer Worker.
/// Watches the Budlum chain for UniversalRelay transactions and 
/// "relays" them to external chains (EVM, Solana, etc.).

pub struct RelayerWorker {
    chain: ChainHandle,
    /// Rewards for the relayer are minted in $BUD (Decision 9).
    relayer_address: Address,
}

impl RelayerWorker {
    pub fn new(chain: ChainHandle, relayer_address: Address) -> Self {
        Self {
            chain,
            relayer_address,
        }
    }

    pub async fn run(self) {
        info!("Universal Relayer Worker started for {}", self.relayer_address);
        
        let mut last_height = self.chain.get_height().await;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            
            let current_height = self.chain.get_height().await;
            if current_height <= last_height {
                continue;
            }

            for h in (last_height + 1)..=current_height {
                if let Some(block) = self.chain.get_block(h).await {
                    for tx in block.transactions {
                        if let TransactionType::UniversalRelay(ext_tx) = tx.tx_type {
                            info!(
                                chain = ?ext_tx.chain,
                                target = %ext_tx.target_address,
                                "Relayer: Detected external transaction request"
                            );
                            
                            // Real-world: Connect to Web3 provider (ethers-rs, solana-sdk)
                            // and submit the signed payload.
                            self.process_relay(tx.from, ext_tx).await;
                        }
                    }
                }
            }
            last_height = current_height;
        }
    }

    async fn process_relay(&self, user: Address, ext_tx: crate::core::transaction::ExternalTransaction) {
        // Implementation for different chains (Hat 5.1 extension)
        match ext_tx.chain {
            crate::core::transaction::ExternalChain::Ethereum => {
                info!("Relaying to Ethereum...");
                // Placeholder: Here the relayer would use its own ETH for gas, 
                // and get reimbursed in $BUD or taking fee from asset.
            }
            _ => {
                warn!("Relay for {:?} not yet implemented", ext_tx.chain);
            }
        }
    }
}
