#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Vec};

fn setup(threshold: u32, n_signers: u32) -> (Env, Vec<Address>, MultisigEventRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MultisigEventRegistry, ());
    let client = MultisigEventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let mut signers: Vec<Address> = vec![&env];
    for _ in 0..n_signers {
        signers.push_back(Address::generate(&env));
    }

    client.init(&admin, &signers, &threshold);
    let mut all = vec![&env, admin];
    for s in signers.iter() {
        all.push_back(s);
    }
    (env, all, client)
}

fn has_event(env: &Env, name: &str) -> bool {
    let expected = soroban_sdk::Symbol::new(env, name);
    for (_, topics, _) in env.events().all().iter() {
        let t: soroban_sdk::Symbol = topics.get(0).unwrap().into_val(env);
        if t == expected {
            return true;
        }
    }
    false
}

#[test]
fn init_then_threshold_1_executes_on_propose() {
    let (env, principals, client) = setup(1, 2);
    let s1 = principals.get(1).unwrap();
    let id = client.propose(
        &s1,
        &symbol_short!("payout"),
        &Bytes::new(&env),
        &Bytes::from_slice(&env, &[1, 2, 3]),
    );
    // Soroban testutils snapshot `events().all()` per invocation, so assert
    // the attestation event before the read-only `get_proposal` call.
    assert!(has_event(&env, "ms_attest"));
    let proposal = client.get_proposal(&id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Executed);
}

#[test]
fn threshold_2_requires_second_approval() {
    let (env, principals, client) = setup(2, 3);
    let s1 = principals.get(1).unwrap();
    let s2 = principals.get(2).unwrap();
    let id = client.propose(
        &s1,
        &symbol_short!("rotate"),
        &Bytes::new(&env),
        &Bytes::from_slice(&env, &[9]),
    );
    // Reading via `get_proposal` clears the per-invocation event buffer;
    // we re-check after the second approval below.
    let p = client.get_proposal(&id).unwrap();
    assert_eq!(p.status, ProposalStatus::Pending);

    client.approve(&s2, &id);
    assert!(has_event(&env, "ms_attest"));
    let p = client.get_proposal(&id).unwrap();
    assert_eq!(p.status, ProposalStatus::Executed);
}

#[test]
#[should_panic(expected = "not a signer")]
fn non_signer_cannot_propose() {
    let (env, _principals, client) = setup(2, 2);
    let stranger = Address::generate(&env);
    client.propose(
        &stranger,
        &symbol_short!("x"),
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
}

#[test]
#[should_panic(expected = "already approved")]
fn double_approve_rejected() {
    let (env, principals, client) = setup(3, 3);
    let s1 = principals.get(1).unwrap();
    let s2 = principals.get(2).unwrap();
    let id = client.propose(
        &s1,
        &symbol_short!("x"),
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
    client.approve(&s2, &id);
    client.approve(&s2, &id);
}

#[test]
fn per_event_threshold_override() {
    let (env, principals, client) = setup(3, 3);
    let admin = principals.get(0).unwrap();
    // Lower the threshold for "low_risk" to 1 so the proposer's vote executes
    client.set_threshold(&admin, &symbol_short!("low_risk"), &1);
    let s1 = principals.get(1).unwrap();
    let id = client.propose(
        &s1,
        &symbol_short!("low_risk"),
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
    let p = client.get_proposal(&id).unwrap();
    assert_eq!(p.status, ProposalStatus::Executed);
}

#[test]
fn proposer_can_cancel_pending() {
    let (env, principals, client) = setup(3, 3);
    let s1 = principals.get(1).unwrap();
    let id = client.propose(
        &s1,
        &symbol_short!("x"),
        &Bytes::new(&env),
        &Bytes::new(&env),
    );
    client.cancel(&s1, &id);
    let p = client.get_proposal(&id).unwrap();
    assert_eq!(p.status, ProposalStatus::Cancelled);
}

#[test]
#[should_panic(expected = "threshold exceeds signer count")]
fn init_rejects_excessive_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MultisigEventRegistry, ());
    let client = MultisigEventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let signers: Vec<Address> = vec![&env, Address::generate(&env)];
    client.init(&admin, &signers, &5);
}
