# Multisig Event Registry — High-Value Event Emission Gate

The `MultisigEventRegistry` contract wraps event emission behind an M-of-N signer approval gate. Sensitive events (treasury withdrawals, oracle rotation, large-value transfers) cannot be emitted without sign-off from the configured signer set, and every executed event is accompanied by a `ms_attest` attestation event that records the full approving signer list and the proposal id — giving downstream consumers non-repudiation evidence that the event was authorized.

Implements issue #292.

## Features

- **M-of-N signer set**: An immutable list of signer addresses configured at `init` time, with an admin who can override per-event-type thresholds later.
- **Per-event-type threshold override**: `set_threshold(event_type, threshold)` lets the admin require more approvals for high-risk event types than for routine ones. Falls back to `default_threshold` when no override exists.
- **Auto-execution**: As soon as approvals reach the threshold the underlying event is emitted automatically — no separate `execute` call needed.
- **Non-repudiation attestation**: Every executed proposal also emits a `ms_attest` event with `(proposal_id, event_type, approvals: Vec<Address>)` so off-chain consumers can independently verify which signers authorized the emission.
- **Cancellation**: The proposer or the admin can cancel a pending proposal before it executes.

## Contract Interface

### `init(admin, signers, default_threshold)`
One-time initialization. Panics if called twice. `default_threshold` must be between 1 and `signers.len()` inclusive.

### `set_threshold(admin, event_type, threshold)`
Admin-only. Sets the per-event-type approval threshold. The new threshold must satisfy the same `0 < t ≤ signer_count` invariant.

### `propose(proposer, event_type, topic2, data) -> u64`
A signer proposes the emission of a high-value event. The proposer's vote is recorded automatically. If the threshold for `event_type` is 1, the event is emitted immediately and the proposal moves to `Executed`. Returns the new `proposal_id`.

### `approve(signer, proposal_id)`
A signer approves a pending proposal. Auto-executes when the threshold is reached. Panics if the signer has already approved this proposal or if the proposal is no longer pending.

### `cancel(caller, proposal_id)`
Proposer or admin only. Marks the proposal as `Cancelled`.

### `get_proposal(proposal_id) -> Option<Proposal>`
### `get_threshold(event_type) -> u32`
### `get_signers() -> Vec<Address>`
Read-only queries.

## Events

| Event | Topics | Body |
|-------|--------|------|
| `proposed` | `(symbol)` | `(proposal_id, proposer)` |
| `approved` | `(symbol)` | `(proposal_id, signer)` |
| `cancel` | `(symbol)` | `proposal_id` |
| **caller's event_type** | `(event_type, topic2?)` | the original `data` (emitted on execution) |
| `ms_attest` | `(symbol)` | `(proposal_id, event_type, approvals: Vec<Address>)` |

The combination of the original event and the `ms_attest` envelope is what gives the registry its non-repudiation guarantee.

## Security Notes

- The contract validates `0 < threshold ≤ signer_count` at every entry point that sets a threshold, so a misconfigured admin cannot lock the registry.
- `require_auth` is enforced on every state-changing call.
- The signer set is captured at `init`; rotating signers requires a contract upgrade or a wrapper contract. This is deliberate — a multisig whose signer set can be changed unilaterally has weaker security guarantees than one whose set is immutable.
