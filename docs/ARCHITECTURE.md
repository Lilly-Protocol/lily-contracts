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

The helper `bump_instance(env)` extends the **instance storage** TTL by `INSTANCE_BUMP_AMOUNT` whenever it is called. State-mutating and active state-reading contract entrypoints call `bump_instance` at the end of the happy path, ensuring the instance does not expire due to inactivity. Lightweight view functions—specifically `is_initialized` across all contracts and `protocol::get_pending_admin`—do not invoke `bump_instance`.

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
| `PendingAdmin` | `Address` | Instance | Pending admin address during two-step admin transfer. |
| `Treasury` | `Address` | Instance | Treasury address for fee collection. |
| `FeeBps` | `u32` | Instance | Fee in basis points. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `PinnedAdmin` | `Address` | Instance | Initial admin address pinned at deployment in `__constructor`. |
| `SchemaVersion` | `u32` | Instance | Protocol contract schema version. |

### Admin functions

- `initialize`
- `set_fee_bps`
- `set_treasury`
- `transfer_admin`

### Pending-admin functions

- `accept_admin` (pending admin signs)

### View functions

- `is_initialized`
- `schema_version`
- `get_config`
- `get_pending_admin`

---

## `contracts/identity`

Agent identity registry.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `Admin` | `Address` | Instance | Registry admin address. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `Profile(Address)` | `AgentProfile` | Persistent | Per-agent profile record. |
| `PinnedAdmin` | `Address` | Instance | Initial admin address pinned at deployment in `__constructor`. |

### Admin functions

- `initialize`
- `deactivate`

### Self-authorized functions

- `register` (agent signs)
- `update_profile` (current controller signs)
- `reactivate` (current controller signs)

### View functions

- `is_initialized`
- `get_profile`
- `get_profile_opt`

---

## `contracts/wallet`

Wallet policy registry.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `Admin` | `Address` | Instance | Wallet registry admin address. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `SchemaVersion` | `u32` | Instance | Wallet contract schema version. |
| `Binding(Address)` | `WalletBinding` | Persistent | Per-agent wallet binding configuration. |
| `PinnedAdmin` | `Address` | Instance | Initial admin address pinned at deployment in `__constructor`. |

### Admin functions

- `initialize`
- `admin_deactivate`

### Self-authorized functions

- `update_spend_limit` (agent signs)
- `set_enabled` (agent signs)

### Dual-authorized functions

- `bind_wallet` (agent and wallet both sign)
- `rebind_wallet` (agent and new wallet both sign)

### View functions

- `is_initialized`
- `get_binding`
- `get_binding_opt`

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
| `Wallet` | `Address` | Instance | Bound wallet policy contract address. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `SchemaVersion` | `u32` | Instance | Payments contract schema version. |
| `Intent(u64)` | `PaymentIntent` | Persistent | Per-intent payment record. |
| `PinnedAdmin` | `Address` | Instance | Initial admin address pinned at deployment in `__constructor`. |
| `PayerIntents(Address)` | `Vec<u64>` | Persistent | Intent ID indices created by a specific payer address. |

### Admin functions

- `initialize`
- `settle_intent`
- `set_fee_bps`
- `set_treasury`
- `transfer_admin`

### Self-authorized functions

- `create_intent` (payer signs)
- `cancel_intent` (payer signs)

### View functions

- `is_initialized`
- `schema_version`
- `get_config`
- `get_next_intent_id`
- `get_intent`
- `get_intent_opt`

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
