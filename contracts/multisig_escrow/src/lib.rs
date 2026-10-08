#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token, Address, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidThreshold = 4,
    InvalidAmount = 5,
    ProposalNotFound = 6,
    ProposalAlreadyExecuted = 7,
    AlreadyApproved = 8,
    NotApproved = 9,
    ThresholdNotMet = 10,
    RecoveryTimelockNotExpired = 11,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalInfo {
    pub id: u64,
    pub proposer: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub created_at: u64,
    pub approval_count: u32,
    pub status: ProposalStatus,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Signers,
    Threshold,
    RecoveryTimelock,
    NextProposalId,
    Proposal(u64),
    Approved(u64, Address),
}

#[contract]
pub struct MultisigEscrowContract;

#[contractimpl]
impl MultisigEscrowContract {
    /// Initialize the multisig escrow contract with signers, threshold, and recovery timelock.
    pub fn initialize(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
        recovery_timelock_secs: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        if threshold == 0 || threshold > signers.len() {
            return Err(Error::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        env.storage()
            .instance()
            .set(&DataKey::RecoveryTimelock, &recovery_timelock_secs);
        env.storage().instance().set(&DataKey::NextProposalId, &1u64);

        Ok(())
    }

    /// Deposit funds into the escrow contract.
    pub fn deposit(env: Env, depositor: Address, token: Address, amount: i128) -> Result<(), Error> {
        depositor.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        Ok(())
    }

    /// Propose releasing funds from escrow to a designated recipient.
    pub fn propose_release(
        env: Env,
        proposer: Address,
        recipient: Address,
        token: Address,
        amount: i128,
    ) -> Result<u64, Error> {
        proposer.require_auth();
        Self::assert_is_signer(&env, &proposer)?;

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextProposalId)
            .unwrap_or(1);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &(proposal_id + 1));

        let now = env.ledger().timestamp();
        let proposal = ProposalInfo {
            id: proposal_id,
            proposer: proposer.clone(),
            recipient,
            token,
            amount,
            created_at: now,
            approval_count: 1, // Proposer automatically approves
            status: ProposalStatus::Pending,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::Approved(proposal_id, proposer), &true);

        Ok(proposal_id)
    }

    /// Approve an existing release proposal.
    pub fn approve_release(env: Env, signer: Address, proposal_id: u64) -> Result<(), Error> {
        signer.require_auth();
        Self::assert_is_signer(&env, &signer)?;

        let mut proposal: ProposalInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalAlreadyExecuted);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Approved(proposal_id, signer.clone()))
        {
            return Err(Error::AlreadyApproved);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Approved(proposal_id, signer), &true);
        proposal.approval_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        Ok(())
    }

    /// Revoke approval for a pending proposal.
    pub fn revoke_approval(env: Env, signer: Address, proposal_id: u64) -> Result<(), Error> {
        signer.require_auth();
        Self::assert_is_signer(&env, &signer)?;

        let mut proposal: ProposalInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalAlreadyExecuted);
        }

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Approved(proposal_id, signer.clone()))
        {
            return Err(Error::NotApproved);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::Approved(proposal_id, signer));
        proposal.approval_count -= 1;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        Ok(())
    }

    /// Execute the asset release if threshold approvals are met.
    pub fn execute_release(env: Env, caller: Address, proposal_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut proposal: ProposalInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalAlreadyExecuted);
        }

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(Error::NotInitialized)?;

        if proposal.approval_count < threshold {
            return Err(Error::ThresholdNotMet);
        }

        proposal.status = ProposalStatus::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        let token_client = token::Client::new(&env, &proposal.token);
        token_client.transfer(
            &env.current_contract_address(),
            &proposal.recipient,
            &proposal.amount,
        );

        Ok(())
    }

    /// Recovery release override by admin after timelock expiration.
    pub fn recovery_release(env: Env, admin: Address, proposal_id: u64) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        let mut proposal: ProposalInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalAlreadyExecuted);
        }

        let timelock: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RecoveryTimelock)
            .unwrap_or(604_800); // 7 days default

        let now = env.ledger().timestamp();
        if now < proposal.created_at + timelock {
            return Err(Error::RecoveryTimelockNotExpired);
        }

        proposal.status = ProposalStatus::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        let token_client = token::Client::new(&env, &proposal.token);
        token_client.transfer(
            &env.current_contract_address(),
            &proposal.recipient,
            &proposal.amount,
        );

        Ok(())
    }

    // View methods
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<ProposalInfo, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)
    }

    pub fn get_signers(env: Env) -> Result<Vec<Address>, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(Error::NotInitialized)
    }

    pub fn get_threshold(env: Env) -> Result<u32, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(Error::NotInitialized)
    }

    pub fn is_approved(env: Env, proposal_id: u64, signer: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Approved(proposal_id, signer))
    }

    fn assert_is_signer(env: &Env, address: &Address) -> Result<(), Error> {
        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(Error::NotInitialized)?;

        for i in 0..signers.len() {
            if signers.get(i).unwrap() == *address {
                return Ok(());
            }
        }
        Err(Error::NotAuthorized)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        vec, Env,
    };

    #[test]
    fn test_multisig_escrow_workflow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MultisigEscrowContract);
        let client = MultisigEscrowContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let s1 = Address::generate(&env);
        let s2 = Address::generate(&env);
        let s3 = Address::generate(&env);

        let signers = vec![&env, s1.clone(), s2.clone(), s3.clone()];
        client.mock_all_auths().initialize(&admin, &signers, &2, &86400);

        assert_eq!(client.get_threshold(), 2);
        assert_eq!(client.get_signers().len(), 3);

        let recipient = Address::generate(&env);
        let token = Address::generate(&env);

        let prop_id = client
            .mock_all_auths()
            .propose_release(&s1, &recipient, &token, &1000);

        assert_eq!(prop_id, 1);
        let prop = client.get_proposal(&1);
        assert_eq!(prop.approval_count, 1);
        assert_eq!(client.is_approved(&1, &s1), true);
        assert_eq!(client.is_approved(&1, &s2), false);

        // Approve with second signer
        client.mock_all_auths().approve_release(&s2, &1);
        let prop_updated = client.get_proposal(&1);
        assert_eq!(prop_updated.approval_count, 2);
        assert_eq!(client.is_approved(&1, &s2), true);
    }
}
