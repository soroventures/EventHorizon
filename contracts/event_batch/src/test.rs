#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env};

fn setup() -> (Env, Address, EventBatchContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventBatchContract, ());
    let client = EventBatchContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    (env, sender, client)
}

#[test]
fn emits_two_events_plus_summary() {
    let (env, sender, client) = setup();
    let entries = vec![
        &env,
        EventEntry {
            topic: symbol_short!("transfer"),
            topic2: Bytes::new(&env),
            data: Bytes::from_slice(&env, &[1, 2, 3]),
        },
        EventEntry {
            topic: symbol_short!("approve"),
            topic2: Bytes::new(&env),
            data: Bytes::from_slice(&env, &[4, 5]),
        },
    ];

    let n = client.emit_batch(&sender, &entries);
    assert_eq!(n, 2);

    // 2 user events + 1 batch_emt summary
    assert_eq!(env.events().all().len(), 3);
}

#[test]
fn second_topic_included_when_non_empty() {
    let (env, sender, client) = setup();
    let entries = vec![
        &env,
        EventEntry {
            topic: symbol_short!("trans"),
            topic2: Bytes::from_slice(&env, b"addr"),
            data: Bytes::from_slice(&env, &[42]),
        },
    ];
    client.emit_batch(&sender, &entries);

    let all = env.events().all();
    // First event = our entry with 2 topics; second = batch_emt summary
    let (_, topics, _) = all.get(0).unwrap();
    assert_eq!(topics.len(), 2);
}

#[test]
#[should_panic(expected = "empty batch")]
fn empty_batch_rejected() {
    let (env, sender, client) = setup();
    let entries: Vec<EventEntry> = vec![&env];
    client.emit_batch(&sender, &entries);
}

#[test]
#[should_panic(expected = "batch too large")]
fn oversized_batch_rejected() {
    let (env, sender, client) = setup();
    let mut entries: Vec<EventEntry> = vec![&env];
    for _ in 0..(MAX_BATCH_SIZE + 1) {
        entries.push_back(EventEntry {
            topic: symbol_short!("x"),
            topic2: Bytes::new(&env),
            data: Bytes::new(&env),
        });
    }
    client.emit_batch(&sender, &entries);
}

#[test]
fn max_batch_size_accepted() {
    let (env, sender, client) = setup();
    let mut entries: Vec<EventEntry> = vec![&env];
    for _ in 0..MAX_BATCH_SIZE {
        entries.push_back(EventEntry {
            topic: symbol_short!("x"),
            topic2: Bytes::new(&env),
            data: Bytes::new(&env),
        });
    }
    let n = client.emit_batch(&sender, &entries);
    assert_eq!(n, MAX_BATCH_SIZE);
}
