#![no_std]
#![allow(deprecated)]
//! Decentralized Identity (DID) resolver on Soroban.
//!
//! See issue #295. Implements a minimal W3C-DID-Core-compatible
//! registry. DID documents are stored as opaque bytes (intended to be
//! a canonical-JSON encoding) keyed by DID string. Each DID has a
//! single on-chain controller address that may update or revoke it.
//!
//! ## W3C alignment
//! - `did:soroban:<method-specific-id>` strings are the registry keys.
//!   The method-specific id is the part after `did:soroban:` — the
//!   registry does not parse it, just stores the full string.
//! - `register` corresponds to the W3C "create" CRUD verb.
//! - `update` / `revoke` cover "update" and "deactivate".
//! - `resolve` returns the current `DidDocument`; revoked DIDs
//!   resolve as `Revoked` so consumers can distinguish from "never
//!   existed".
//! - Every state transition emits an event (`did_reg`, `did_upd`,
//!   `did_rev`, `did_attest`) so off-chain indexers can mirror the
//!   registry without scanning storage.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, String,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Document(String),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DidStatus {
    Active = 0,
    Revoked = 1,
}

#[contracttype]
#[derive(Clone)]
pub struct DidDocument {
    pub did: String,
    pub controller: Address,
    /// W3C DID Document as canonical JSON or CBOR bytes. The registry
    /// does not parse this — that's the consumer's responsibility.
    pub document: Bytes,
    pub status: DidStatus,
    pub created_at_ledger: u32,
    pub updated_at_ledger: u32,
    /// Monotonic version counter; bumps on every successful `update`.
    pub version: u32,
}

#[contract]
pub struct DidResolver;

#[contractimpl]
impl DidResolver {
    /// Register a new DID under `controller`. Panics if `did` already
    /// exists (active or revoked) — DIDs are non-recyclable.
    pub fn register(env: Env, controller: Address, did: String, document: Bytes) {
        controller.require_auth();
        if did.len() == 0 {
            panic!("did empty");
        }
        if env.storage().persistent().has(&DataKey::Document(did.clone())) {
            panic!("did already registered");
        }
        let now = env.ledger().sequence();
        let doc = DidDocument {
            did: did.clone(),
            controller: controller.clone(),
            document,
            status: DidStatus::Active,
            created_at_ledger: now,
            updated_at_ledger: now,
            version: 1,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Document(did.clone()), &doc);

        env.events()
            .publish((symbol_short!("did_reg"),), (did.clone(), controller.clone()));
        env.events()
            .publish((symbol_short!("did_attst"),), (did, controller, 1u32));
    }

    /// Replace the DID document. Only the recorded controller can call.
    /// Bumps `version` and `updated_at_ledger`. Cannot update a revoked DID.
    pub fn update(env: Env, controller: Address, did: String, new_document: Bytes) {
        controller.require_auth();
        let mut doc: DidDocument = env
            .storage()
            .persistent()
            .get(&DataKey::Document(did.clone()))
            .expect("did not found");
        if doc.controller != controller {
            panic!("not the controller");
        }
        if doc.status != DidStatus::Active {
            panic!("did is revoked");
        }
        doc.document = new_document;
        doc.updated_at_ledger = env.ledger().sequence();
        doc.version += 1;
        let new_version = doc.version;
        env.storage()
            .persistent()
            .set(&DataKey::Document(did.clone()), &doc);

        env.events()
            .publish((symbol_short!("did_upd"),), (did.clone(), new_version));
        env.events()
            .publish((symbol_short!("did_attst"),), (did, controller, new_version));
    }

    /// Mark a DID as revoked. Subsequent `update`s panic; `resolve`
    /// still returns the document but with `status = Revoked`.
    pub fn revoke(env: Env, controller: Address, did: String) {
        controller.require_auth();
        let mut doc: DidDocument = env
            .storage()
            .persistent()
            .get(&DataKey::Document(did.clone()))
            .expect("did not found");
        if doc.controller != controller {
            panic!("not the controller");
        }
        if doc.status == DidStatus::Revoked {
            panic!("already revoked");
        }
        doc.status = DidStatus::Revoked;
        doc.updated_at_ledger = env.ledger().sequence();
        env.storage()
            .persistent()
            .set(&DataKey::Document(did.clone()), &doc);

        env.events()
            .publish((symbol_short!("did_rev"),), (did, controller));
    }

    /// Resolve a DID. Returns `None` if never registered. Revoked DIDs
    /// resolve to `Some(doc)` with `status = Revoked` so consumers can
    /// distinguish from "never existed".
    pub fn resolve(env: Env, did: String) -> Option<DidDocument> {
        env.storage().persistent().get(&DataKey::Document(did))
    }

    /// Lightweight existence + status check that avoids returning the
    /// full document.
    pub fn status(env: Env, did: String) -> Option<DidStatus> {
        Self::resolve(env, did).map(|d| d.status)
    }
}

mod test;
