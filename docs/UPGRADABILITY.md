# Upgrade and Migration Strategy

This document describes how the Lily Protocol contracts can be upgraded in the future without losing on-chain state. It is intentionally written as a strategy guide: the current contracts do not yet expose an `upgrade` entrypoint, so the playbook below defines the requirements and guardrails for when that work is undertaken.

## Current state

The four deployed contracts (`identity`, `payments`, `protocol`, `wallet`) store state in Soroban's instance and persistent storage. None of them currently exposes an `upgrade` entrypoint or a storage-version guard. Adding upgrade support is listed as future work in `README.md`.

## Wasm hash pinning

On Soroban, contract code is identified by a wasm hash stored on-chain. Upgrading a contract means replacing the hash that the contract instance points to.

### Requirements

1. Each contract must pin the hash of the wasm it was initialized with.
2. A new wasm must be built, optimized, and its hash computed locally before deployment.
3. The upgrade transaction must explicitly reference both the old and new wasm hashes so the operation is auditable.
4. Avoid unbounded wasm growth; each new version should justify its additional size.

### Recommended workflow

```text
build new wasm
  -> compute wasm hash
  -> store wasm on-chain (if not already stored)
  -> invoke upgrade(new_wasm_hash) from the admin account
  -> run post-upgrade migration (if needed)
  -> emit upgrade event
```

## Storage versioning

Storage layouts change over time. A `storage_version: u32` value should be stored in instance storage for each contract.

### Requirements

1. Initial deployment uses `storage_version = 1`.
2. Every upgrade that changes storage layout must bump the version.
3. Migration logic must read the current version and apply only the deltas required to reach the target version.
4. Downgrades are rejected; versions only increase monotonically.

### Example migration guard

```rust
const CURRENT_STORAGE_VERSION: u32 = 1;

fn ensure_storage_version(env: &Env) {
    let version: u32 = env.storage().instance().get(&DataKey::StorageVersion)
        .unwrap_or(0);
    require(env, version == CURRENT_STORAGE_VERSION, ProtocolError::InvalidInput);
}
```

## Migration windows

Upgrades should not happen instantaneously without notice. A migration window gives integrators time to react.

### Recommended policy

1. **Announcement**: Publish the new wasm hash, storage changes, and migration script at least one week before on-chain execution.
2. **Freeze period**: Pause state-changing admin operations during the upgrade block if the change affects core data layouts.
3. **Execution**: Admin calls `upgrade` in a single transaction that updates the wasm hash and runs the migration.
4. **Verification**: Run a suite of read-only integration tests against the upgraded contract before unfreezing user operations.
5. **Rollback plan**: Keep the previous wasm hash available on-chain so the instance can be reverted if a critical issue is found within the rollback window.

## Future `upgrade` entrypoint requirements

When an `upgrade` entrypoint is added to each contract, it should satisfy the following:

- **Authorization**: Only the contract admin can invoke it.
- **Wasm hash validation**: Accept exactly one `BytesN<32>` argument representing the new wasm hash.
- **Storage migration**: Call an internal `migrate()` function that bumps `storage_version` and rewrites any changed keys.
- **Event emission**: Emit an `("upgrade", admin, old_wasm_hash, new_wasm_hash, new_storage_version)` event.
- **Idempotency**: Re-invoking with the same wasm hash should be a no-op or should revert.
- **Size limits**: Verify the new wasm is within acceptable deployment limits.

### Entrypoint signature

```rust
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
```

### Internal migration hook

```rust
fn migrate(env: &Env, old_version: u32, new_version: u32) {
    // Apply deltas between old_version and new_version.
    env.storage().instance().set(&DataKey::StorageVersion, &new_version);
}
```

## Cross-contract considerations

- `protocol` and `payments` share the concept of admin, treasury, and fee basis points. Upgrading one may require the other to understand new config shapes.
- `identity` profiles are referenced by `wallet` and `payments`. Any change to `AgentProfile` must be coordinated across consumers.
- Event topics and payload shapes are part of the observable interface. Versioning events is recommended if breaking changes are introduced.

## Operational checklist

Before executing an upgrade:

- [ ] New wasm is built with `make build-wasm` and hash is recorded.
- [ ] Storage version delta is documented in this file.
- [ ] Migration script is reviewed and tested against a local network.
- [ ] Integrators have been notified of breaking event or storage changes.
- [ ] Admin key is available and has been tested on a testnet deployment.
- [ ] Rollback wasm hash is known and stored on-chain.

## References

- [Soroban contract upgrade docs](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/upgrade)
- `README.md` future work section for upgrade and migration playbooks
- `docs/EVENTS.md` for event-versioning guidance
- `docs/ARCHITECTURE.md` for per-contract storage layouts (when available)
