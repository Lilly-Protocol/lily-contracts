# Architecture

This document describes the high-level storage, durability, and authorization design shared by the Lily Protocol Soroban contracts. It is intended for contributors, auditors, and integrators who need to understand where state lives, how long it lasts, and who can change it.

## Storage durability

Soroban provides two storage kinds that the contracts use deliberately:

- **Instance storage** (`env.storage().instance()`): small, frequently-accessed state that is tied to the contract deployment. Used for global config, admin addresses, and one-time initialization flags. Bumped on every entrypoint call to keep the instance alive.
- **Persistent storage** (`env.storage().persistent()`): per-entity state that must survive for the lifetime of the protocol. Used for profiles, intents, and wallet bindings.

Both kinds are keyed by typed `DataKey` enums local to each contract crate. There is no shared `DataKey` across contracts.

## TTL policy

Shared TTL constants live in `crates/lily-common/src/lib.rs`:

```rust
pub const INSTANCE_BUMP_THRESHOLD: u32 = 17_280;   // ~1 day of ledgers
pub const INSTANCE_BUMP_AMOUNT: u32 = 172_800;     // ~10 days of ledgers
```

The helper `bump_instance(env)` extends the **instance storage** TTL by `INSTANCE_BUMP_AMOUNT` whenever it is called. Every contract entrypoint that reads or writes state calls `bump_instance` at the end of the happy path, ensuring the instance does not expire due to inactivity.

Persistent storage entries do not currently call `extend_ttl` explicitly. In a production deployment, long-lived per-entity records (profiles, intents, bindings) should be bumped explicitly or the protocol should rely on periodic keeper transactions to keep critical records alive.

## Initialization state

Every contract follows the same initialization pattern:

1. Check that `DataKey::Initialized` is not already set.
2. Require auth from the actor that will become the admin.
3. Write initial config and set `DataKey::Initialized = true`.
4. Emit an `init` event.

Re-initialization is rejected with `ProtocolError::AlreadyInitialized`.

## Authorization model

Three categories of actors appear across the contracts:

- **Admin**: Set at initialization. Can change global config, transfer admin rights, and perform privileged actions such as deactivating profiles or settling payment intents.
- **Self-authorized actor**: The agent or payer that owns a specific record. Must sign operations that affect their own profile, wallet binding, or payment intent.
- **Dual authorization**: Some operations require both the agent and a related party to sign. For example, `wallet::bind_wallet` requires auth from both `agent` and `wallet`.

Auth is always explicit via `Address::require_auth()`; there are no implicit or delegated authorization paths.

---

## `contracts/protocol`

Global protocol configuration.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `Admin` | `Address` | Instance | Protocol admin address. |
| `Treasury` | `Address` | Instance | Treasury address for fee collection. |
| `FeeBps` | `u32` | Instance | Fee in basis points. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |

### Admin functions

- `initialize`
- `set_fee_bps`
- `set_treasury`
- `transfer_admin`

---

## `contracts/identity`

Agent identity registry.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `Admin` | `Address` | Instance | Registry admin address. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `Profile(Address)` | `AgentProfile` | Persistent | Per-agent profile record. |

### Admin functions

- `initialize`
- `deactivate`

### Self-authorized functions

- `register` (agent signs)
- `update_profile` (current controller signs)

---

## `contracts/wallet`

Wallet policy registry.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `Admin` | `Address` | Instance | Wallet registry admin address. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `Binding(Address)` | `WalletBinding` | Persistent | Per-agent wallet binding. |

### Admin functions

- `initialize`

### Self-authorized functions

- `update_spend_limit` (agent signs)
- `set_enabled` (agent signs)

### Dual-authorized functions

- `bind_wallet` (agent and wallet both sign)

---

## `contracts/payments`

Payment intent and settlement.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `Admin` | `Address` | Instance | Settlement admin address. |
| `Treasury` | `Address` | Instance | Treasury address for fee collection. |
| `FeeBps` | `u32` | Instance | Fee in basis points. |
| `NextIntentId` | `u64` | Instance | Monotonically increasing intent ID counter. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `Intent(u64)` | `PaymentIntent` | Persistent | Per-intent payment record. |

### Admin functions

- `initialize`
- `settle_intent`

### Self-authorized functions

- `create_intent` (payer signs)
- `cancel_intent` (payer signs)

---

## Shared primitives

### `crates/lily-common`

- `ProtocolError`: typed errors used across all contracts.
- `PaymentStatus`: enum used by `payments` (and potentially future settlement contracts).
- `MAX_BPS`: basis-point ceiling.
- `bump_instance`: TTL refresh helper.

### `crates/lily-test-support`

Test-only helpers; no runtime storage.

## Versioning

- Instance storage is versioned implicitly by the contract wasm hash.
- A future upgrade path should introduce an explicit `StorageVersion` key; see `docs/UPGRADABILITY.md`.
