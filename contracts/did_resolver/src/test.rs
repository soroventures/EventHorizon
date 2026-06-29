#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, Address, Bytes, Env, IntoVal, String};

fn setup() -> (Env, Address, DidResolverClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(DidResolver, ());
    let client = DidResolverClient::new(&env, &contract_id);
    let controller = Address::generate(&env);
    (env, controller, client)
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
fn register_then_resolve() {
    let (env, controller, client) = setup();
    let did = String::from_str(&env, "did:soroban:alice");
    let doc = Bytes::from_slice(&env, b"{\"id\":\"did:soroban:alice\"}");
    client.register(&controller, &did, &doc);
    // `env.events().all()` is snapshotted per invocation, so assert on
    // emitted events before making the read-only `resolve` call.
    assert!(has_event(&env, "did_reg"));
    assert!(has_event(&env, "did_attst"));

    let resolved = client.resolve(&did).unwrap();
    assert_eq!(resolved.controller, controller);
    assert_eq!(resolved.status, DidStatus::Active);
    assert_eq!(resolved.version, 1);
}

#[test]
fn resolve_unknown_did_returns_none() {
    let (env, _c, client) = setup();
    assert!(client.resolve(&String::from_str(&env, "did:soroban:nope")).is_none());
}

#[test]
#[should_panic(expected = "did already registered")]
fn cannot_register_twice() {
    let (env, controller, client) = setup();
    let did = String::from_str(&env, "did:soroban:bob");
    let doc = Bytes::from_slice(&env, b"a");
    client.register(&controller, &did, &doc);
    client.register(&controller, &did, &doc);
}

#[test]
fn update_bumps_version_and_emits_identity_verification() {
    let (env, controller, client) = setup();
    let did = String::from_str(&env, "did:soroban:carol");
    client.register(&controller, &did, &Bytes::from_slice(&env, b"v1"));

    client.update(&controller, &did, &Bytes::from_slice(&env, b"v2"));
    // Check events immediately after `update` before any read-only call
    // wipes the per-invocation event buffer.
    assert!(has_event(&env, "did_upd"));
    // did_attst is the "event-based identity verification update" hook
    // called out in the issue's technical requirements.
    assert!(has_event(&env, "did_attst"));

    let doc = client.resolve(&did).unwrap();
    assert_eq!(doc.version, 2);
}

#[test]
#[should_panic(expected = "not the controller")]
fn non_controller_cannot_update() {
    let (env, controller, client) = setup();
    let did = String::from_str(&env, "did:soroban:dave");
    client.register(&controller, &did, &Bytes::from_slice(&env, b"v1"));
    let stranger = Address::generate(&env);
    client.update(&stranger, &did, &Bytes::from_slice(&env, b"v2"));
}

#[test]
fn revoke_marks_revoked_but_still_resolves() {
    let (env, controller, client) = setup();
    let did = String::from_str(&env, "did:soroban:eve");
    client.register(&controller, &did, &Bytes::from_slice(&env, b"v1"));
    client.revoke(&controller, &did);
    let doc = client.resolve(&did).unwrap();
    assert_eq!(doc.status, DidStatus::Revoked);
    assert_eq!(client.status(&did), Some(DidStatus::Revoked));
}

#[test]
#[should_panic(expected = "did is revoked")]
fn cannot_update_after_revoke() {
    let (env, controller, client) = setup();
    let did = String::from_str(&env, "did:soroban:frank");
    client.register(&controller, &did, &Bytes::from_slice(&env, b"v1"));
    client.revoke(&controller, &did);
    client.update(&controller, &did, &Bytes::from_slice(&env, b"v2"));
}

#[test]
#[should_panic(expected = "did empty")]
fn empty_did_rejected() {
    let (env, controller, client) = setup();
    client.register(&controller, &String::from_str(&env, ""), &Bytes::from_slice(&env, b"x"));
}

#[test]
fn issue_295_emits_identity_verification_on_update() {
    // Direct test for the "Allow event-based identity verification updates"
    // bullet — every update emits a did_attst event with did + controller +
    // version, which a verifier can subscribe to. Soroban testutils snapshot
    // `events().all()` per invocation, so we assert once per call.
    let (env, controller, client) = setup();
    let did = String::from_str(&env, "did:soroban:verify");

    client.register(&controller, &did, &Bytes::from_slice(&env, b"v1"));
    assert!(has_event(&env, "did_attst"));

    client.update(&controller, &did, &Bytes::from_slice(&env, b"v2"));
    assert!(has_event(&env, "did_attst"));

    client.update(&controller, &did, &Bytes::from_slice(&env, b"v3"));
    assert!(has_event(&env, "did_attst"));
}
