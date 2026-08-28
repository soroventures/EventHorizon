#![no_std]
#![allow(deprecated)]
//! Multi-signature registry for high-value event emission.
//!
//! See issue #292. Wraps event emission behind an M-of-N signer
//! approval gate so sensitive events (e.g. treasury withdrawals,
//! oracle rotation, large-value transfers) cannot be emitted without
//! sign-off from the configured signer set.
//!
//! ## Lifecycle
//! 1. Admin calls `init(admin, signers, default_threshold)`.
//! 2. Admin (or anyone, depending on policy) calls
//!    `set_threshold(event_type, threshold)` to override the per-type
//!    threshold. If unset, `default_threshold` applies.
//! 3. Any signer calls `propose(event_type, topic2, data)` → returns
//!    a `proposal_id`.
//! 4. Other signers call `approve(proposal_id)`; once approvals reach
//!    the threshold the proposal auto-executes and the event is
//!    emitted with a `multisig_attest` confirming proposal_id and the
//!    full signer list, providing non-repudiation.
//! 5. `cancel(proposal_id)` is callable by the proposer or admin
//!    before execution.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Signers,
    DefaultThreshold,
    /// Per-event-type threshold override.
    Threshold(Symbol),
    NextProposalId,
    Proposal(u64),
    /// True once `signer` has approved `proposal_id`.
    Approved(u64, Address),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProposalStatus {
    Pending = 0,
    Executed = 1,
    Cancelled = 2,
}

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub proposer: Address,
    pub event_type: Symbol,
    pub topic2: Bytes,
    pub data: Bytes,
    pub approvals: Vec<Address>,
    pub status: ProposalStatus,
    pub created_at_ledger: u32,
}

#[contract]
pub struct MultisigEventRegistry;

#[contractimpl]
impl MultisigEventRegistry {
    /// One-shot initializer. Panics if called twice.
    pub fn init(env: Env, admin: Address, signers: Vec<Address>, default_threshold: u32) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        Self::validate_threshold(default_threshold, signers.len());
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage()
            .instance()
            .set(&DataKey::DefaultThreshold, &default_threshold);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &0u64);
    }

    /// Override the approval threshold for a specific event type.
    pub fn set_threshold(env: Env, admin: Address, event_type: Symbol, threshold: u32) {
        Self::assert_admin(&env, &admin);
        let signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        Self::validate_threshold(threshold, signers.len());
        env.storage()
            .persistent()
            .set(&DataKey::Threshold(event_type), &threshold);
    }

    /// Propose a high-value event emission. Caller must be a signer.
    /// Records the proposer's approval, so a 1-of-1 threshold executes
    /// immediately on proposal.
    pub fn propose(
        env: Env,
        proposer: Address,
        event_type: Symbol,
        topic2: Bytes,
        data: Bytes,
    ) -> u64 {
        proposer.require_auth();
        Self::require_signer(&env, &proposer);

        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextProposalId)
            .unwrap_or(0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &(proposal_id + 1));

        let proposal = Proposal {
            proposer: proposer.clone(),
            event_type: event_type.clone(),
            topic2: topic2.clone(),
            data: data.clone(),
            approvals: Vec::from_array(&env, [proposer.clone()]),
            status: ProposalStatus::Pending,
            created_at_ledger: env.ledger().sequence(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::Approved(proposal_id, proposer.clone()), &true);

        env.events()
            .publish((symbol_short!("proposed"),), (proposal_id, proposer));

        Self::try_execute(&env, proposal_id);
        proposal_id
    }

    /// Approve a pending proposal. Auto-executes when threshold is met.
    pub fn approve(env: Env, signer: Address, proposal_id: u64) {
        signer.require_auth();
        Self::require_signer(&env, &signer);

        let already: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Approved(proposal_id, signer.clone()))
            .unwrap_or(false);
        if already {
            panic!("already approved");
        }
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("unknown proposal");
        if proposal.status != ProposalStatus::Pending {
            panic!("proposal not pending");
        }
        proposal.approvals.push_back(signer.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::Approved(proposal_id, signer.clone()), &true);

        env.events()
            .publish((symbol_short!("approved"),), (proposal_id, signer));

        Self::try_execute(&env, proposal_id);
    }

    /// Cancel a pending proposal. Callable by the proposer or admin.
    pub fn cancel(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("unknown proposal");
        if proposal.status != ProposalStatus::Pending {
            panic!("proposal not pending");
        }
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != proposal.proposer && caller != admin {
            panic!("not authorized to cancel");
        }
        proposal.status = ProposalStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.events()
            .publish((symbol_short!("cancel"),), proposal_id);
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
    }

    pub fn get_threshold(env: Env, event_type: Symbol) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::Threshold(event_type))
            .unwrap_or_else(|| {
                env.storage()
                    .instance()
                    .get(&DataKey::DefaultThreshold)
                    .unwrap()
            })
    }

    pub fn get_signers(env: Env) -> Vec<Address> {
        env.storage().instance().get(&DataKey::Signers).unwrap()
    }

    // ── internal helpers ────────────────────────────────────────────────

    fn assert_admin(env: &Env, who: &Address) {
        who.require_auth();
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if who != &admin {
            panic!("admin only");
        }
    }

    fn require_signer(env: &Env, who: &Address) {
        let signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        for s in signers.iter() {
            if &s == who {
                return;
            }
        }
        panic!("not a signer");
    }

    fn validate_threshold(threshold: u32, signer_count: u32) {
        if threshold == 0 {
            panic!("threshold must be > 0");
        }
        if threshold > signer_count {
            panic!("threshold exceeds signer count");
        }
    }

    /// Emit the underlying event + a `multisig_attest` proof. Idempotent
    /// once status flips to `Executed`.
    fn try_execute(env: &Env, proposal_id: u64) {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap();
        if proposal.status != ProposalStatus::Pending {
            return;
        }
        let threshold = Self::get_threshold(env.clone(), proposal.event_type.clone());
        if proposal.approvals.len() < threshold {
            return;
        }
        proposal.status = ProposalStatus::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        // 1. The underlying high-value event the caller asked for.
        if proposal.topic2.is_empty() {
            env.events()
                .publish((proposal.event_type.clone(),), proposal.data.clone());
        } else {
            env.events().publish(
                (proposal.event_type.clone(), proposal.topic2.clone()),
                proposal.data.clone(),
            );
        }
        // 2. Non-repudiation attestation linking the event back to the
        //    full signer set + proposal id.
        env.events().publish(
            (symbol_short!("ms_attest"),),
            (proposal_id, proposal.event_type, proposal.approvals),
        );
    }
}

mod test;
