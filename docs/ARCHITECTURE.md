# Architecture

This document describes the high-level storage, durability, and authorization design shared by the Lily Protocol Soroban contracts. It is intended for contributors, auditors, and integrators who need to understand where state lives, how long it lasts, and who can change it.

## Storage durability

Soroban provides two storage kinds that the contracts use deliberately:

- **Instance storage** (`env.storage().instance()`): small, frequently-accessed state that is tied to the contract deployment. Used for global config, admin addresses, and one-time initialization flags. Bumped on most entrypoint calls to keep the instance alive.
- **Persistent storage** (`env.storage().persistent()`): per-entity state that must survive for the lifetime of the protocol. Used for profiles, intents, and wallet bindings.

Both kinds are keyed by typed `DataKey` enums local to each contract crate. There is no shared `DataKey` across contracts.

## TTL policy

Shared TTL constants live in `crates/lily-common/src/lib.rs`:

```rust
pub const INSTANCE_BUMP_THRESHOLD: u32 = 17_280;   // ~1 day of ledgers
pub const INSTANCE_BUMP_AMOUNT: u32 = 172_800;     // ~10 days of ledgers
```

The helper `bump_instance(env)` extends the **instance storage** TTL by `INSTANCE_BUMP_AMOUNT` whenever it is called. Most contract entrypoints that read or write state call `bump_instance` at the end of the happy path, ensuring the instance does not expire due to inactivity.

Exceptions that do **not** call `bump_instance`:
- Constructors (`__constructor` across contracts)
- Initialization status checks (`is_initialized` across all contracts)
- View of pending admin (`protocol::get_pending_admin`)

Persistent storage entries do not currently call `extend_ttl` explicitly. In a production deployment, long-lived per-entity records (profiles, intents, bindings) should be bumped explicitly or the protocol should rely on periodic keeper transactions to keep critical records alive.

## Initialization state

Every contract follows the same initialization pattern:

1. Pin the initial admin at deployment in `__constructor` under `DataKey::PinnedAdmin`.
2. In `initialize`, verify that `DataKey::Initialized` is not set and caller matches `PinnedAdmin`.
3. Require auth from the actor that will become the admin.
4. Write initial config, schema version, and set `DataKey::Initialized = true`.
5. Emit an `init` event.

Re-initialization is rejected with `ProtocolError::AlreadyInitialized`.

## Authorization model

Three categories of actors appear across the contracts:

- **Admin**: Set at initialization (pinned during deployment). Can change global config, transfer admin rights, and perform privileged actions such as deactivating profiles, emergency deactivating wallet bindings, or settling payment intents.
- **Self-authorized actor**: The agent or payer that owns a specific record. Must sign operations that affect their own profile, wallet binding, or payment intent.
- **Dual authorization**: Some operations require both the agent and a related party to sign. For example, `wallet::bind_wallet` and `wallet::rebind_wallet` require auth from both `agent` and `wallet`.

Auth is always explicit via `Address::require_auth()`; there are no implicit or delegated authorization paths.

---

## `contracts/protocol`

Global protocol configuration.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `PinnedAdmin` | `Address` | Instance | Deployer-pinned initial admin address. |
| `Admin` | `Address` | Instance | Active protocol admin address. |
| `PendingAdmin` | `Address` | Instance | Proposed admin address in 2-step transfer. |
| `Treasury` | `Address` | Instance | Treasury address for fee collection. |
| `FeeBps` | `u32` | Instance | Active protocol fee in basis points. |
| `SchemaVersion` | `u32` | Instance | Protocol contract schema version. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |

### Admin functions

- `initialize` (initial admin signs)
- `set_fee_bps` (admin signs)
- `set_treasury` (admin signs)
- `transfer_admin` (admin signs)

### Self-authorized / Pending admin functions

- `accept_admin` (pending admin signs)

### Public / View functions

- `__constructor`
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
| `PinnedAdmin` | `Address` | Instance | Deployer-pinned initial admin address. |
| `Admin` | `Address` | Instance | Registry admin address. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `Profile(Address)` | `AgentProfile` | Persistent | Per-agent profile record keyed by agent address. |

### Admin functions

- `initialize` (initial admin signs)
- `deactivate` (admin signs)
- `reactivate` (admin signs)

### Self-authorized functions

- `register` (agent signs)
- `update_profile` (current controller signs)

### Public / View functions

- `__constructor`
- `is_initialized`
- `get_profile`
- `get_profile_opt`

---

## `contracts/wallet`

Wallet policy registry.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `PinnedAdmin` | `Address` | Instance | Deployer-pinned initial admin address. |
| `Admin` | `Address` | Instance | Wallet registry admin address. |
| `SchemaVersion` | `u32` | Instance | Wallet contract schema version. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `Binding(Address)` | `WalletBinding` | Persistent | Per-agent wallet binding configuration. |

### Admin functions

- `initialize` (initial admin signs)
- `admin_deactivate` (admin signs)

### Self-authorized functions

- `update_spend_limit` (agent signs)
- `set_enabled` (agent signs)

### Dual-authorized functions

- `bind_wallet` (agent and wallet both sign)
- `rebind_wallet` (agent and wallet both sign)

### Public / View functions

- `__constructor`
- `is_initialized`
- `get_binding`
- `get_binding_opt`

---

## `contracts/payments`

Payment intent and settlement.

### Storage keys

| Key | Type | Durability | Description |
|---|---|---|---|
| `PinnedAdmin` | `Address` | Instance | Deployer-pinned initial admin address. |
| `Admin` | `Address` | Instance | Settlement admin address. |
| `Treasury` | `Address` | Instance | Protocol fee collector treasury address. |
| `FeeBps` | `u32` | Instance | Protocol fee in basis points. |
| `NextIntentId` | `u64` | Instance | Auto-incrementing identifier for next payment intent. |
| `Wallet` | `Address` | Instance | Associated wallet registry contract address. |
| `SchemaVersion` | `u32` | Instance | Payments contract schema version. |
| `Initialized` | `bool` | Instance | One-time initialization flag. |
| `Intent(u64)` | `PaymentIntent` | Persistent | Per-intent payment record keyed by intent ID. |
| `PayerIntents(Address)` | `Vec<u64>` | Persistent | Index of intent IDs created by a specific payer. |

### Admin functions

- `initialize` (initial admin signs)
- `settle_intent` (settlement admin / authorized caller signs)
- `set_fee_bps` (admin signs)
- `set_treasury` (admin signs)
- `transfer_admin` (admin signs)

### Self-authorized functions

- `create_intent` (payer agent signs)
- `cancel_intent` (payer agent signs)

### Public / View functions

- `__constructor`
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

- Instance storage is versioned explicitly via `DataKey::SchemaVersion` where supported, and implicitly by the contract wasm hash.
- A future upgrade path should expand schema migrations across all contracts; see `docs/UPGRADABILITY.md`.
