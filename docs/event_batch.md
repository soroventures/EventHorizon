# Event Batch — Gas-Optimized Event Batching Utility

The `EventBatchContract` lets a caller emit up to `MAX_BATCH_SIZE` (64) events in a single contract invocation. This amortizes the per-transaction overhead — contract dispatch, auth check, ledger entry growth — across many emissions, dropping the effective per-event cost for indexers and integrations that need to fan out many notifications at once.

Implements issue #293.

## Features

- **Up to 64 events per call**: Constrained by `MAX_BATCH_SIZE` to keep individual transactions within Soroban's per-tx ledger budget.
- **Mixed topic shapes**: Each `EventEntry` carries a primary `topic` and an optional second `topic2` (set to empty `Bytes` to skip). This lets a single batch include both single-topic and dual-topic events.
- **Type-erased payloads**: `data: Bytes` keeps the payload opaque so a single batch can carry heterogeneous event shapes; the consumer reinterprets `data` per topic.
- **Per-batch summary event**: After all entries are emitted, the contract emits one `batch_emt` event with `(count, sender)` so indexers that want to group emissions can do so without inferring batch boundaries from timestamps.

## Contract Interface

### `emit_batch(sender, entries) -> u32`
Emit every entry then a single `batch_emt` summary. `sender` must sign the transaction. Returns the number of user events emitted (excludes the summary).

Panics on:
- Empty batch (`empty batch`)
- Batches larger than `MAX_BATCH_SIZE` (`batch too large`)

## Storage Efficiency

The `EventEntry` struct is designed for minimal storage footprint:

```rust
pub struct EventEntry {
    pub topic: Symbol,   // small inline value
    pub topic2: Bytes,   // empty when single-topic
    pub data: Bytes,     // type-erased payload
}
```

Soroban's `Bytes` and `Symbol` are both small handles; the payload bytes are carried by reference. The total per-entry overhead is therefore close to the size of the payload itself, with no per-event ledger entry write since events don't persist in storage.

## Gas Considerations

A single `emit_batch` invocation pays:
- One contract dispatch + auth check (versus N for N separate calls)
- One Wasm instantiation
- N event emissions (each is O(payload_size))
- One summary event emission

For batches of 10+ small events the savings on dispatch + auth typically dominate the total cost.
