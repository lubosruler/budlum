use crate::core::address::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalType {
    ChangeBaseFee(u64),
    ChangeBlockReward(u64),
    SlashValidator {
        address: Address,
        /// V40 (ARENAX): Hash of the slashing evidence that proves misbehavior.
        /// Governance slash now requires cryptographic proof, not just a vote.
        evidence_hash: [u8; 32],
    },
    ParameterUpdate(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub p_type: ProposalType,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub votes_for: u64,     // Total stake voting FOR
    pub votes_against: u64, // Total stake voting AGAINST
    pub status: ProposalStatus,
    pub voters: HashMap<Address, bool>, // Address -> Vote (true = for)
}

impl Proposal {
    pub fn new(
        id: u64,
        proposer: Address,
        p_type: ProposalType,
        start_epoch: u64,
        duration: u64,
    ) -> Self {
        Proposal {
            id,
            proposer,
            p_type,
            start_epoch,
            end_epoch: start_epoch + duration,
            votes_for: 0,
            votes_against: 0,
            status: ProposalStatus::Active,
            voters: HashMap::new(),
        }
    }

    pub fn add_vote(&mut self, voter: Address, stake: u64, vote_for: bool) -> Result<(), String> {
        if self.status != ProposalStatus::Active {
            return Err("Proposal is not active".into());
        }
        if self.voters.contains_key(&voter) {
            return Err("Already voted".into());
        }

        if vote_for {
            self.votes_for = self.votes_for.saturating_add(stake);
        } else {
            self.votes_against = self.votes_against.saturating_add(stake);
        }
        self.voters.insert(voter, vote_for);
        Ok(())
    }

    pub fn finalize(&mut self, total_stake: u64, quorum_pct: u64) {
        // V70: Use u128 to prevent overflow in the quorum calculation
        let total_votes = (self.votes_for as u128) + (self.votes_against as u128);
        let quorum_threshold = (total_stake as u128) * (quorum_pct as u128);
        let reached_quorum = total_votes * 100 >= quorum_threshold;
        if reached_quorum && self.votes_for > self.votes_against {
            self.status = ProposalStatus::Passed;
        } else {
            self.status = ProposalStatus::Failed;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GovernanceState {
    pub proposals: Vec<Proposal>,
    pub next_proposal_id: u64,
}

impl GovernanceState {
    pub fn create_proposal(
        &mut self,
        proposer: Address,
        p_type: ProposalType,
        current_epoch: u64,
        duration: u64,
    ) -> Result<u64, String> {
        // V68: Validate proposal duration
        const MIN_PROPOSAL_DURATION: u64 = 10; // Minimum 10 epochs
        const MAX_PROPOSAL_DURATION: u64 = 100_000; // Maximum 100,000 epochs
        if duration < MIN_PROPOSAL_DURATION || duration > MAX_PROPOSAL_DURATION {
            return Err(format!(
                "Proposal duration must be between {} and {} epochs",
                MIN_PROPOSAL_DURATION, MAX_PROPOSAL_DURATION
            ));
        }

        // V69: Limit active proposals to prevent state bloat
        const MAX_ACTIVE_PROPOSALS: usize = 100;
        let active_count = self.proposals.iter()
            .filter(|p| p.status == ProposalStatus::Active)
            .count();
        if active_count >= MAX_ACTIVE_PROPOSALS {
            return Err("Too many active proposals".into());
        }

        let id = self.next_proposal_id;
        let proposal = Proposal::new(id, proposer, p_type, current_epoch, duration);
        self.proposals.push(proposal);
        self.next_proposal_id += 1;
        Ok(id)
    }

    pub fn find_proposal_mut(&mut self, id: u64) -> Option<&mut Proposal> {
        self.proposals.iter_mut().find(|p| p.id == id)
    }

    pub fn active_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    /// V71: Cancel a proposal. Only the original proposer can cancel.
    /// The proposal must still be Active.
    pub fn cancel_proposal(
        &mut self,
        proposal_id: u64,
        caller: &Address,
    ) -> Result<(), String> {
        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| format!("Proposal {} not found", proposal_id))?;

        if proposal.proposer != *caller {
            return Err("Only the proposer can cancel the proposal".into());
        }
        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active".into());
        }

        proposal.status = ProposalStatus::Failed;
        Ok(())
    }
}
