# Contract Event Schema

This document lists every event emitted by the Lily Protocol contracts, grouped by contract. Events follow Soroban conventions: each entry has a topic tuple (starting with a short symbol) and a typed payload. The topic is designed for efficient filtering; the payload carries the full post-transition state.

## `contracts/identity`

| Topic | Payload type | Trigger function | Payload fields |
|---|---|---|---|
| `("init", admin)` | `IdentityConfig` | `initialize(env, admin)` | `admin: Address` |
| `("register", agent)` | `AgentProfile` | `register(env, agent, controller, metadata_uri)` | `controller: Address`, `metadata_uri: String`, `active: bool`, `revision: u64` |
| `("metadata_updated", agent)` | `String` | `update_profile(env, agent, metadata_uri, new_controller)` | New metadata URI (emitted only when `metadata_uri` changed). |
| `("controller_rotated", agent)` | `Address` | `update_profile(env, agent, metadata_uri, new_controller)` | New controller address (emitted only when `controller` changed). |
| `("deact", agent)` | `AgentProfile` | `deactivate(env, agent)` | Profile after `active` has been set to `false`. |
| `("react", agent)` | `AgentProfile` | `reactivate(env, agent)` | Profile after `active` has been set to `true` and `revision` incremented. |

## `contracts/protocol`

| Topic | Payload type | Trigger function | Payload fields |
|---|---|---|---|
| `("init", admin)` | `ProtocolConfig` | `initialize(env, admin, treasury, fee_bps)` | `admin: Address`, `treasury: Address`, `fee_bps: u32` |
| `("fee", admin)` | `u32` | `set_fee_bps(env, fee_bps)` | The new fee value in basis points. |
| `("treasury", admin)` | `Address` | `set_treasury(env, treasury)` | The new treasury address. |
| `("propose", admin)` | `Address` | `transfer_admin(env, new_admin)` | The proposed new admin address set in `DataKey::PendingAdmin`. |
| `("admin", old_admin)` | `Address` | `accept_admin(env)` | The accepted new admin address after `DataKey::Admin` is updated. |

## `contracts/payments`

| Topic | Payload type | Trigger function | Payload fields |
|---|---|---|---|
| `("init", admin)` | `PaymentsConfig` | `initialize(env, admin, treasury, fee_bps)` | `admin: Address`, `treasury: Address`, `fee_bps: u32` |
| `("create", id)` | `PaymentIntent` | `create_intent(...)` | `id: u64`, `payer_agent: Address`, `payee_agent: Address`, `amount: i128`, `memo: String`, `settlement_reference: String`, `status: PaymentStatus` |
| `("settle", id, prior_status)` | `PaymentIntent` | `settle_intent(env, caller, intent_id, settlement_reference)` | Intent after `status` is set to `Settled` and the settlement reference is recorded. Third topic element carries prior status symbol (e.g. `"pending"`). |
| `("cancel", id, prior_status)` | `PaymentIntent` | `cancel_intent(env, intent_id)` | Intent after `status` is set to `Cancelled`. Third topic element carries prior status symbol (e.g. `"pending"`). |
| `("fee", admin)` | `u32` | `set_fee_bps(env, fee_bps)` | The new fee value in basis points. |
| `("treasury", admin)` | `Address` | `set_treasury(env, treasury)` | The new treasury address. |
| `("admin", admin)` | `Address` | `transfer_admin(env, new_admin)` | The new payments contract admin address. |

## `contracts/wallet`

| Topic | Payload type | Trigger function | Payload fields |
|---|---|---|---|
| `("init", admin)` | `WalletConfig` | `initialize(env, admin)` | `admin: Address` |
| `("bind", agent)` | `WalletBinding` | `bind_wallet(env, agent, wallet, settlement_asset, spend_limit)` | `wallet: Address`, `settlement_asset: Symbol`, `spend_limit: i128`, `enabled: bool`, `revision: u64` |
| `("rebind", agent)` | `WalletBinding` | `rebind_wallet(env, agent, new_wallet)` | Binding after `wallet` is updated to `new_wallet` and `revision` is incremented. |
| `("limit", agent)` | `WalletBinding` | `update_spend_limit(env, agent, spend_limit)` | Binding after the spend limit is updated and `revision` is incremented. |
| `("state", agent)` | `WalletBinding` | `set_enabled(env, agent, enabled)` | Binding after `enabled` is toggled and `revision` is incremented. |
| `("adm_deact", agent)` | `WalletBinding` | `admin_deactivate(env, agent)` | Binding after admin forces `enabled` to `false` and increments `revision`. |

## Common payload types

```rust
// crates/lily-common/src/lib.rs
pub enum PaymentStatus {
    Pending,
    Settled,
    Cancelled,
}

// contracts/identity/src/lib.rs
pub struct IdentityConfig {
    pub admin: Address,
}

pub struct AgentProfile {
    pub controller: Address,
    pub metadata_uri: String,
    pub active: bool,
    pub revision: u64,
}

// contracts/protocol/src/lib.rs
pub struct ProtocolConfig {
    pub admin: Address,
    pub treasury: Address,
    pub fee_bps: u32,
}

// contracts/payments/src/lib.rs
pub struct PaymentsConfig {
    pub admin: Address,
    pub treasury: Address,
    pub fee_bps: u32,
}

pub struct PaymentIntent {
    pub id: u64,
    pub payer_agent: Address,
    pub payee_agent: Address,
    pub amount: i128,
    pub memo: String,
    pub settlement_reference: String,
    pub status: PaymentStatus,
}

// contracts/wallet/src/lib.rs
pub struct WalletBinding {
    pub wallet: Address,
    pub settlement_asset: Symbol,
    pub spend_limit: i128,
    pub enabled: bool,
    pub revision: u64,
}
```

## Versioning

Event topics and payload shapes are considered part of the contract's observable interface. Any future change that alters a topic element or payload field should be documented in both this file and in the release notes so that indexers and off-chain integrations can migrate.
