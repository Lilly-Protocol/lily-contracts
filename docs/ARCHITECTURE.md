# Architecture

This document describes the high-level storage, durability, and authorization design shared by the Lily Protocol Soroban contracts. It is intended for contributors, auditors, and integrators who need to understand where state lives, how long it lasts, and who can change it.

## Storage durability

Soroban provides two storage kinds that the contracts use deliberately:

- **Instance storage** (`env.storage().instance()`): small, frequently-accessed state that is tied to the contract deployment. Used for global config, admin addresses, schema versions, and one-time initialization flags. Bumped on happy-path entrypoint calls to keep the instance alive.
- **Persistent storage** (`env.storage().persistent()`): per-entity state that must survive for the lifetime of the protocol. Used for profiles, intents, wallet bindings, and payer intent indexes.

Both kinds are keyed by typed `DataKey` enums local to each contract crate. There is no shared `DataKey` across contracts.

## TTL policy

Shared TTL constants live in `crates/lily-common/src/lib.rs`:

```rust
pub const INSTANCE_BUMP_THRESHOLD: u32 = 17_280;   // ~1 day of ledgers
pub const INSTANCE_BUMP_AMOUNT: u32 = 172_800;     // ~10 days of ledgers
