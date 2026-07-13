use crate::core::account::AccountState;
use crate::core::address::Address;
use crate::core::transaction::{Transaction, TransactionType};
use crate::error::{BudlumError, BudlumResult};
use crate::execution::zkvm::{ZkVmExecutor, DEFAULT_CONTRACT_GAS_LIMIT};

pub struct Executor;

impl Executor {
    pub fn apply_transaction(state: &mut AccountState, tx: &Transaction) -> Result<(), String> {
        Self::apply_transaction_checked(state, tx).map_err(|e| e.message().to_string())
    }

    pub fn apply_transaction_checked(
        state: &mut AccountState,
        tx: &Transaction,
    ) -> BudlumResult<()> {
        if tx.from == Address::zero() {
            return Ok(());
        }

        // Tur 9.5 (security audit §10): enforce the cost-floor /
        // shape checks for Unstake and Vote at the consensus
        // boundary. The `tx_precheck` layer catches these at the
        // RPC boundary, but consensus (this function) is the
        // canonical gatekeeper — a zero-fee, zero-amount, empty-data
        // Unstake/Vote must be rejected here too, otherwise an
        // internal path (replay, sync, etc.) could inject spam
        // that bypasses the RPC check.
        //
        // We do NOT call `is_valid()` here because that helper
        // also runs the full signature check, which is performed
        // by the caller (`validate_and_add_block` /
        // `validate_pool_transaction`); running it twice is
        // redundant and also breaks the in-test pattern of
        // constructing unsigned txs for unit-test convenience.
        // The cost-floor / shape rules below are the consensus
        // invariant — duplicated here, not derived from `is_valid`.
        match tx.tx_type {
            TransactionType::Unstake => {
                if tx.amount == 0 {
                    return Err(BudlumError::validation(
                        "unstake_amount_zero",
                        "Unstake amount cannot be 0",
                    ));
                }
                if tx.fee == 0 {
                    return Err(BudlumError::validation(
                        "unstake_fee_zero",
                        "Unstake fee cannot be 0 (consensus cost-floor)",
                    ));
                }
            }
            TransactionType::Vote if tx.fee == 0 => {
                return Err(BudlumError::validation(
                    "vote_fee_zero",
                    "Vote fee cannot be 0 (consensus cost-floor)",
                ));
            }
            _ => {}
        }

        let total_cost = tx.total_cost();

        {
            let sender_account = state.get_or_create(&tx.from);
            if sender_account.balance < total_cost {
                return Err(BudlumError::validation(
                    "insufficient_balance",
                    "Insufficient balance",
                ));
            }
        }

        match tx.tx_type {
            TransactionType::Transfer => {
                // Tur 2 tokenomics integration: enforce team-vesting on
                // outgoing transfers. The `state.spendable_balance` helper
                // already accounts for the locked portion of the team
                // account at the current epoch; we check it BEFORE any
                // mutable move so the rejection is atomic.
                let spendable = state.spendable_balance(&tx.from);
                if total_cost > spendable {
                    return Err(BudlumError::validation(
                        "vesting_locked",
                        format!(
                            "Transfer exceeds spendable balance: have {spendable}, need {total_cost}"
                        ),
                    ));
                }
                let sender = state.get_or_create(&tx.from);
                sender.balance = sender.balance.saturating_sub(total_cost);
                sender.nonce = sender.nonce.saturating_add(1);

                let receiver = state.get_or_create(&tx.to);
                receiver.balance = receiver.balance.saturating_add(tx.amount);
            }
            TransactionType::Stake => {
                let sender = state.get_or_create(&tx.from);
                sender.balance = sender.balance.saturating_sub(total_cost);
                sender.nonce = sender.nonce.saturating_add(1);

                let stake_amount = tx.amount;
                let validator = state.get_validator_mut(&tx.from);

                if let Some(v) = validator {
                    v.stake = v.stake.saturating_add(stake_amount);
                    v.active = true;
                } else {
                    state.add_validator(tx.from, stake_amount);
                }
                // Tur 5: keep the permissionless registry in lock-step with the
                // on-chain validator set so `is_active(staker, VALIDATOR)`
                // returns `true` the moment the first stake lands.
                state.sync_validator_registration(&tx.from);
            }
            TransactionType::Unstake => {
                let sender_start_balance = state.get_balance(&tx.from);
                if sender_start_balance < tx.fee {
                    return Err(BudlumError::validation(
                        "insufficient_fee_balance",
                        "Insufficient balance for fee",
                    ));
                }

                if let Some(validator) = state.get_validator_mut(&tx.from) {
                    if validator.stake < tx.amount {
                        return Err(BudlumError::validation(
                            "insufficient_stake",
                            "Insufficient stake",
                        ));
                    }
                    validator.stake = validator.stake.saturating_sub(tx.amount);
                    if validator.stake == 0 {
                        validator.active = false;
                    }
                } else {
                    return Err(BudlumError::validation("not_validator", "Not a validator"));
                }

                state
                    .unbonding_queue
                    .push(crate::core::account::UnbondingEntry {
                        address: tx.from,
                        amount: tx.amount,
                        release_epoch: state.epoch_index + crate::core::account::UNBONDING_EPOCHS,
                    });

                let sender = state.get_or_create(&tx.from);
                sender.balance = sender.balance.saturating_sub(tx.fee);
                sender.nonce = sender.nonce.saturating_add(1);
            }
            TransactionType::Vote => {
                let sender_acc = state.get_or_create(&tx.from);
                sender_acc.balance = sender_acc.balance.saturating_sub(tx.fee);
                sender_acc.nonce = sender_acc.nonce.saturating_add(1);

                if tx.to != Address::zero() {
                    if let Some(target) = state.get_validator_mut(&tx.to) {
                        if tx.amount > 0 {
                            target.votes_for += 1;
                        } else {
                            target.votes_against += 1;
                        }
                        tracing::info!("Validator Vote recorded: {} -> {}", tx.from, tx.to);
                    }
                } else if !tx.data.is_empty() && tx.data.len() >= 9 {
                    if tx.data.len() == 9 {
                        let vote_for = tx.data[0] != 0;
                        let mut id_bytes = [0u8; 8];
                        id_bytes.copy_from_slice(&tx.data[1..9]);
                        let proposal_id = u64::from_le_bytes(id_bytes);

                        let voter_stake =
                            state.get_validator(&tx.from).map(|v| v.stake).unwrap_or(0);
                        if voter_stake == 0 {
                            return Err(BudlumError::validation(
                                "governance_voter_not_validator",
                                "Only validators can vote in governance",
                            ));
                        }

                        if let Some(proposal) = state.governance.find_proposal_mut(proposal_id) {
                            proposal
                                .add_vote(tx.from, voter_stake, vote_for)
                                .map_err(|e| {
                                    BudlumError::validation("governance_vote_failed", e)
                                })?;
                            tracing::info!(
                                "Governance Vote: Proposal {} from {}",
                                proposal_id,
                                tx.from
                            );
                        } else {
                            return Err(BudlumError::validation(
                                "proposal_not_found",
                                "Proposal not found",
                            ));
                        }
                    } else {
                        // Likely a Proposal: [duration (8), ProposalType (...)]
                        let mut dur_bytes = [0u8; 8];
                        dur_bytes.copy_from_slice(&tx.data[0..8]);
                        let duration = u64::from_le_bytes(dur_bytes);

                        let p_type: crate::core::governance::ProposalType =
                            serde_json::from_slice(&tx.data[8..]).map_err(|e| {
                                BudlumError::validation(
                                    "invalid_proposal_data",
                                    format!("Invalid proposal data: {}", e),
                                )
                            })?;

                        let id = state.governance.create_proposal(
                            tx.from,
                            p_type,
                            state.epoch_index,
                            duration,
                        );
                        tracing::info!("Governance Proposal Created: ID {} from {}", id, tx.from);
                    }
                }
            }
            TransactionType::ContractCall => {
                ZkVmExecutor::execute_bytecode(&tx.data, DEFAULT_CONTRACT_GAS_LIMIT)
                    .map_err(|e| BudlumError::validation("contract_execution_failed", e))?;

                let sender = state.get_or_create(&tx.from);
                sender.balance = sender.balance.saturating_sub(tx.fee);
                sender.nonce = sender.nonce.saturating_add(1);
            }
        }

        Ok(())
    }

    pub fn apply_block(
        state: &mut AccountState,
        transactions: &[Transaction],
        block_producer: Option<&Address>,
    ) -> Result<(), String> {
        Self::apply_block_checked(state, transactions, block_producer)
            .map_err(|e| e.message().to_string())
    }

    pub fn apply_block_checked(
        state: &mut AccountState,
        transactions: &[Transaction],
        block_producer: Option<&Address>,
    ) -> BudlumResult<()> {
        let mut total_fees: u64 = 0;
        for tx in transactions {
            if tx.from == Address::zero() {
                continue;
            }
            if let Err(e) = Self::apply_transaction_checked(state, tx) {
                return Err(BudlumError::validation(
                    "transaction_apply_failed",
                    format!("TX apply failed: {}", e),
                ));
            }
            total_fees = total_fees.saturating_add(tx.fee);
        }
        // Tur 2 tokenomics integration: the metabolic (tx-fee) burn must be
        // subtracted from the producer's reward and permanently destroyed.
        // We compute everything from immutable borrows first, then take the
        // mutable borrows for the actual balance moves, so the borrow checker
        // stays happy.
        let (block_reward, metabolic_burn_total) = {
            // Use the canonical `metabolic_burn(fee)` helper from
            // `TokenomicsParams` so the rounding behaviour is identical to
            // the module's own unit tests.
            let mut total_burn: u64 = 0;
            for tx in transactions {
                if tx.from == Address::zero() {
                    continue;
                }
                total_burn = total_burn.saturating_add(state.tokenomics.metabolic_burn(tx.fee));
            }
            (state.tokenomics.block_reward, total_burn)
        };

        if let Some(producer) = block_producer {
            // Producer reward = block_reward + (fees - metabolic_burn).
            // The burn is permanently destroyed (no account receives it), so
            // total supply strictly decreases by `metabolic_burn_total` minus
            // the freshly minted `block_reward` (a net deflationary effect
            // when burn > 0).
            //
            // TUR 4 SUPPLY CAP (Tur 24 kararı, Seçenek B — sert tavan):
            // $BUD arz tavanı (`BUD_TOTAL_SUPPLY = 100M`) aşılamaz. Kalan pay
            // tam ödemeye yetmiyorsa, block_reward kısmi ödenir (clamp); fee
            // kısmı her zaman tam ödenir (fee'ler zaten supply'den çıkmış —
            // sender'lardan kesildi). Net: cap asla aşılmaz, fee'ler her zaman
            // teslim edilir, block_reward payı clamp edilir.
            let max_supply = crate::tokenomics::BUD_TOTAL_SUPPLY as u128;
            let current_supply = state.circulating_supply();
            let cap_room = max_supply.saturating_sub(current_supply);

            // block_reward kısmı: cap'te yer varsa hepsi, yoksa sadece kalan oda kadar
            let block_reward_paid: u64 = if cap_room >= block_reward as u128 {
                block_reward
            } else {
                cap_room as u64
            };
            // fee kısmı (zaten sender'lardan kesildi): cap'te yer varsa hepsi, yoksa kalan
            let fee_remainder = total_fees.saturating_sub(metabolic_burn_total);
            let fee_paid: u64 = if cap_room >= block_reward as u128 {
                fee_remainder
            } else {
                let room_after_block = cap_room.saturating_sub(block_reward_paid as u128);
                if (fee_remainder as u128) <= room_after_block {
                    fee_remainder
                } else {
                    room_after_block as u64
                }
            };

            let reward = block_reward_paid.saturating_add(fee_paid);
            if reward > 0 {
                let producer_account = state.get_or_create(producer);
                producer_account.balance = producer_account.balance.saturating_add(reward);
                tracing::info!(
                    "Producer {} received reward: {} (fees_paid: {}, burn: {}, block_paid: {} / block_full: {})",
                    producer,
                    reward,
                    fee_paid,
                    metabolic_burn_total,
                    block_reward_paid,
                    block_reward
                );
            } else {
                tracing::info!(
                    "Producer {} received no reward (fees: {}, burn: {}, block: {}; cap reached)",
                    producer,
                    total_fees,
                    metabolic_burn_total,
                    block_reward
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::AccountState;
    use crate::core::transaction::{Transaction, TransactionType};
    use bud_isa::{Instruction, Opcode};

    #[test]
    fn test_apply_block_reward() {
        let mut state = AccountState::new();
        let producer = Address::from_hex(&"0".repeat(64)).unwrap();
        let txs = vec![];

        Executor::apply_block(&mut state, &txs, Some(&producer)).unwrap();

        let reward = state.tokenomics.block_reward;
        let account = state.get_or_create(&producer);
        assert_eq!(account.balance, reward);
    }

    #[test]
    fn test_apply_block_reward_with_fees() {
        let mut state = AccountState::new();
        let producer = Address::from_hex(&"01".repeat(32)).unwrap();
        let alice = Address::from_hex(&"02".repeat(32)).unwrap();
        state.add_balance(&alice, 100);

        let mut tx = Transaction::new(alice, Address::zero(), 10, vec![]);
        tx.fee = 5;
        tx.nonce = 0;

        Executor::apply_block(&mut state, &[tx], Some(&producer)).unwrap();

        let reward = state.tokenomics.block_reward;
        let producer_acc = state.get_or_create(&producer);
        assert_eq!(producer_acc.balance, reward + 5);

        let alice_acc = state.get_or_create(&alice);
        assert_eq!(alice_acc.balance, 100 - 15);
    }

    #[test]
    fn test_vote_for_transaction() {
        let mut state = AccountState::new();
        let alice = Address::from_hex(&"01".repeat(32)).unwrap();
        let val_pubkey = Address::from_hex(&"02".repeat(32)).unwrap();

        state.add_balance(&alice, 100);
        state.add_validator(val_pubkey, 1000);

        let mut tx = Transaction::new(alice, val_pubkey, 1, vec![]);
        tx.tx_type = TransactionType::Vote;
        tx.fee = 2;

        Executor::apply_transaction(&mut state, &tx).unwrap();

        let validator = state.get_validator(&val_pubkey).unwrap();
        assert_eq!(validator.votes_for, 1);
        assert_eq!(validator.votes_against, 0);

        let alice_acc = state.get_or_create(&alice);
        assert_eq!(alice_acc.balance, 98);
    }

    #[test]
    fn test_vote_against_transaction() {
        let mut state = AccountState::new();
        let alice = Address::from_hex(&"01".repeat(32)).unwrap();
        let val_pubkey = Address::from_hex(&"02".repeat(32)).unwrap();

        state.add_balance(&alice, 100);
        state.add_validator(val_pubkey, 1000);

        let mut tx = Transaction::new(alice, val_pubkey, 0, vec![]);
        tx.tx_type = TransactionType::Vote;
        tx.fee = 2;

        Executor::apply_transaction(&mut state, &tx).unwrap();

        let validator = state.get_validator(&val_pubkey).unwrap();
        assert_eq!(validator.votes_for, 0);
        assert_eq!(validator.votes_against, 1);
    }

    #[test]
    fn test_contract_call_executes_budzkvm_bytecode() {
        let mut state = AccountState::new();
        let alice = Address::from_hex(&"03".repeat(32)).unwrap();
        state.add_balance(&alice, 100);

        let program = vec![
            Instruction {
                opcode: Opcode::Load,
                rd: 1,
                rs1: 0,
                rs2: 0,
                imm: 11,
            }
            .encode(),
            Instruction {
                opcode: Opcode::Log,
                rd: 0,
                rs1: 1,
                rs2: 0,
                imm: 0,
            }
            .encode(),
            Instruction {
                opcode: Opcode::Halt,
                rd: 0,
                rs1: 0,
                rs2: 0,
                imm: 0,
            }
            .encode(),
        ];
        let bytecode: Vec<u8> = program
            .into_iter()
            .flat_map(|instruction| instruction.to_le_bytes())
            .collect();
        let tx = Transaction::new_contract_call(alice, 7, 0, bytecode);

        Executor::apply_transaction(&mut state, &tx).unwrap();

        let alice_acc = state.get_or_create(&alice);
        assert_eq!(alice_acc.balance, 93);
        assert_eq!(alice_acc.nonce, 1);
    }
}

/// Tur 9.5 (security audit §10): a zero-fee Unstake must be
/// rejected at the consensus boundary, not only at the RPC
/// `tx_precheck` boundary. Without this, an internal path
/// (replay, sync, etc.) could inject zero-fee Unstake spam
/// that bloats the mempool and chain without paying the
/// cost-floor.
#[test]
fn consensus_rejects_zero_fee_unstake() {
    let mut state = AccountState::new();
    let alice = Address::from_hex(&"01".repeat(32)).unwrap();
    state.add_balance(&alice, 1_000_000);
    state.add_validator(alice, 1_000);

    let mut tx = Transaction::new(alice, Address::zero(), 100, vec![]);
    tx.tx_type = TransactionType::Unstake;
    tx.fee = 0; // zero-fee spam
    tx.nonce = 0;
    let kp = crate::crypto::primitives::KeyPair::generate().unwrap();
    tx.sign(&kp);

    let err = Executor::apply_transaction(&mut state, &tx)
        .expect_err("zero-fee Unstake must be rejected at consensus");
    assert!(
        err.contains("unstake_fee_zero") || err.contains("Unstake fee cannot be 0"),
        "expected cost-floor error, got: {err}"
    );
}

/// Tur 9.5 (security audit §10): a zero-amount Unstake must
/// be rejected. Without this, an Unstake with amount=0 would
/// be a silent no-op (executor skips the stake subtraction
/// because `validator.stake < 0` is false, but still bumps the
/// nonce and pays the fee). It also bypasses the unbonding
/// queue invariant.
#[test]
fn consensus_rejects_zero_amount_unstake() {
    let mut state = AccountState::new();
    let alice = Address::from_hex(&"01".repeat(32)).unwrap();
    state.add_balance(&alice, 1_000_000);
    state.add_validator(alice, 1_000);

    let mut tx = Transaction::new(alice, Address::zero(), 0, vec![]);
    tx.tx_type = TransactionType::Unstake;
    tx.fee = 1;
    tx.nonce = 0;
    let kp = crate::crypto::primitives::KeyPair::generate().unwrap();
    tx.sign(&kp);

    let err = Executor::apply_transaction(&mut state, &tx)
        .expect_err("zero-amount Unstake must be rejected at consensus");
    assert!(
        err.contains("unstake_amount_zero") || err.contains("Unstake amount cannot be 0"),
        "expected amount-zero error, got: {err}"
    );
}

/// Tur 9.5 (security audit §10): a zero-fee Vote must be
/// rejected at consensus. Same rationale as the Unstake
/// cost-floor: governance spam must not be free.
#[test]
fn consensus_rejects_zero_fee_vote() {
    let mut state = AccountState::new();
    let alice = Address::from_hex(&"01".repeat(32)).unwrap();
    state.add_balance(&alice, 1_000_000);
    state.add_validator(alice, 1_000);

    // 9-byte vote data: bool + u64 proposal_id
    let mut data = vec![1u8];
    data.extend_from_slice(&42u64.to_le_bytes());
    let mut tx = Transaction::new(alice, Address::zero(), 0, data);
    tx.tx_type = TransactionType::Vote;
    tx.fee = 0; // zero-fee spam
    tx.nonce = 0;
    let kp = crate::crypto::primitives::KeyPair::generate().unwrap();
    tx.sign(&kp);

    let err = Executor::apply_transaction(&mut state, &tx)
        .expect_err("zero-fee Vote must be rejected at consensus");
    assert!(
        err.contains("vote_fee_zero") || err.contains("Vote fee cannot be 0"),
        "expected cost-floor error, got: {err}"
    );
}
