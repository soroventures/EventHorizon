#![no_std]
#![allow(deprecated)]
//! Gas-optimized event batching utility for EventHorizon.
//!
//! See issue #293. Lets callers emit N events in a single contract
//! invocation, amortizing the per-tx overhead. Topics and payloads are
//! passed as a single `Vec<EventEntry>` so all events flow through one
//! invocation boundary.
//!
//! ## Design notes
//! - `EventEntry::data` is `Bytes` to keep the payload type-erased; the
//!   consumer reinterprets it per topic. This is the most storage-
//!   efficient way to carry mixed payloads in one batch.
//! - `emit_batch` caps batch size at `MAX_BATCH_SIZE` to prevent a
//!   pathological caller from blowing the per-tx ledger limit.
//! - The contract itself emits exactly one extra `batch_emit` summary
//!   event with `(batch_size, sender)` for downstream indexers.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

/// Hard cap on entries per `emit_batch` call. Soroban budgets are tight;
/// larger batches must be split client-side.
pub const MAX_BATCH_SIZE: u32 = 64;

#[contracttype]
#[derive(Clone)]
pub struct EventEntry {
    /// First topic for the event (e.g. event-type symbol).
    pub topic: Symbol,
    /// Optional second topic (e.g. subject address). Empty Bytes when unused.
    pub topic2: Bytes,
    /// Type-erased payload bytes.
    pub data: Bytes,
}

#[contract]
pub struct EventBatchContract;

#[contractimpl]
impl EventBatchContract {
    /// Emit every event in `entries` followed by a single `batch_emit`
    /// summary event. Panics if `entries.len() > MAX_BATCH_SIZE` or empty.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban env.
    /// * `sender` - The caller; must sign the transaction. Recorded in
    ///   the summary event for traceability.
    /// * `entries` - Ordered list of events to emit in this batch.
    ///
    /// # Returns
    ///
    /// * `u32` - The number of events emitted (excludes the summary event).
    pub fn emit_batch(env: Env, sender: Address, entries: Vec<EventEntry>) -> u32 {
        sender.require_auth();

        let count = entries.len();
        if count == 0 {
            panic!("empty batch");
        }
        if count > MAX_BATCH_SIZE {
            panic!("batch too large");
        }

        for entry in entries.iter() {
            if entry.topic2.is_empty() {
                env.events().publish((entry.topic.clone(),), entry.data.clone());
            } else {
                env.events()
                    .publish((entry.topic.clone(), entry.topic2.clone()), entry.data.clone());
            }
        }

        env.events()
            .publish((symbol_short!("batch_emt"),), (count, sender));

        count
    }
}

mod test;
