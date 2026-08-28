# DID Resolver — Decentralized Identity on Soroban

The `DidResolver` contract is a minimal, W3C-DID-Core-compatible registry. DID documents are stored as opaque bytes (intended to be a canonical-JSON encoding of the W3C document) keyed by DID string, with a single on-chain `controller` address per DID that may update or revoke it.

Implements issue #295.

## W3C Alignment

The contract implements the standard DID CRUD verbs:

| W3C verb | Contract function |
|----------|-------------------|
| Create | `register` |
| Read (resolve) | `resolve`, `status` |
| Update | `update` |
| Deactivate | `revoke` |

DID strings follow the `did:soroban:<method-specific-id>` shape. The registry stores the full string and does not parse the method-specific id — that's the consumer's responsibility, and lets the same contract serve different method definitions if desired.

## Features

- **Single-controller authority**: Each DID has exactly one `controller` address recorded at registration. Only that address can update or revoke the document.
- **Document opacity**: Documents are stored as `Bytes` — the registry treats them as opaque. This means the contract works unchanged for any canonical encoding (JSON, JSON-LD, CBOR, etc.).
- **Monotonic version counter**: Every successful `update` bumps the `version` field, giving consumers a cheap way to detect stale cached copies.
- **Distinction between "revoked" and "never existed"**: `resolve` returns `Some(doc_with_status_Revoked)` for revoked DIDs and `None` for unknown DIDs. Revoked DIDs are not recyclable — re-registering the same string panics.
- **Event-based identity verification updates**: Every state transition emits a `did_attst` event with `(did, controller, version)` so off-chain verifiers can subscribe and react without polling.

## Contract Interface

### `register(controller, did, document)`
Create a new DID record. `controller.require_auth()`. Panics on empty `did` or if the DID is already registered (active or revoked).

### `update(controller, did, new_document)`
Replace the document. Only the recorded controller can call. Bumps `version` and `updated_at_ledger`. Panics if the DID is revoked.

### `revoke(controller, did)`
Mark the DID revoked. Subsequent `update`s panic. `resolve` still returns the document with `status = Revoked`.

### `resolve(did) -> Option<DidDocument>`
Return the current document, or `None` if the DID was never registered.

### `status(did) -> Option<DidStatus>`
Lightweight existence + status check that avoids returning the full document.

## Events

| Event | Topics | Body | When |
|-------|--------|------|------|
| `did_reg` | `(symbol)` | `(did, controller)` | On `register` |
| `did_upd` | `(symbol)` | `(did, new_version)` | On `update` |
| `did_rev` | `(symbol)` | `(did, controller)` | On `revoke` |
| `did_attst` | `(symbol)` | `(did, controller, version)` | On `register` and `update` — the identity-verification event-stream hook |

## Stored Document Shape

```rust
pub struct DidDocument {
    pub did: String,
    pub controller: Address,
    pub document: Bytes,        // canonical-JSON / CBOR / etc.
    pub status: DidStatus,       // Active | Revoked
    pub created_at_ledger: u32,
    pub updated_at_ledger: u32,
    pub version: u32,            // monotonic; bumps on every `update`
}
```

## Non-Recyclable DIDs

Once a DID string has been registered, it cannot be re-registered even after revocation. This matches the W3C "deactivation is permanent" model and prevents controller substitution attacks where a former controller waits for a DID to be revoked and re-claims it.
