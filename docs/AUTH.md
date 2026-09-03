# Authorization Model

This document is the function-by-function authorization matrix for Lily Protocol. It mirrors the current contract entrypoints and their `require_auth()` / `require_auth_or_error()` / typed role checks. The model separates **protocol governance** (admin), **agent lifecycle** (the agent itself), **delegated control** (controller), and **funding/policy ownership** (payer, wallet) so each address holds only the authority required for its role.

## Vocabulary

| Role | Meaning |
| --- | --- |
| `admin` | The governance address stored by a contract after initialization. |
| pinned admin | The address stored by `__constructor` under `PinnedAdmin` for contracts that enforce deploy-time bootstrap identity. |
| pending admin | The address proposed by protocol `transfer_admin` and allowed to complete the two-step handover with `accept_admin`. |
| `agent` | A registered Lily agent (an `Address` with a profile). |
| `controller` | The address delegated by an agent to manage its profile. |
| `payer_agent` | The agent that opens a payment intent and funds it. |
| `wallet` | The external wallet bound to an agent for settlement. |

Read-only views require no caller authorization. Their initialization guards and TTL behavior are implementation details separate from the authorization boundary documented here.

## `contracts/protocol`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none inside the contract | Records the deploy-time `initial_admin` as `PinnedAdmin`; constructor invocation is part of deployment rather than an authenticated runtime governance call. |
| `initialize` | submitted admin, which must also equal the pinned admin | The submitted admin signs the bootstrap call, and `require_initial_admin` rejects an address different from the one pinned at deployment. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `schema_version` | none | Read-only schema-version view. |
| `get_config` | none | Read-only configuration view. |
| `get_pending_admin` | none | Read-only pending-handover view. |
| `set_fee_bps` | stored admin | Changing the fee is a governance action. |
| `set_treasury` | stored admin | Only the current governance address may redirect the treasury. |
| `transfer_admin` | stored admin | Step 1 of the handover: the current admin proposes `new_admin` and remains the active admin until acceptance. |
| `accept_admin` | pending admin | Step 2: the address stored in `PendingAdmin` must authenticate before it replaces the current admin. |

### Protocol admin handover

Protocol admin transfer is deliberately two-step. `transfer_admin(new_admin)` authenticates the current stored admin and writes `PendingAdmin`; it does **not** replace `Admin`. Until the pending address calls `accept_admin()`, the old admin remains authoritative. `accept_admin()` authenticates the pending address, writes it to `Admin`, clears `PendingAdmin`, and completes the handover.

## `contracts/identity`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none inside the contract | Records the deploy-time `initial_admin` under `PinnedAdmin`. |
| `initialize` | submitted admin | The address passed as `admin` must authenticate. The current implementation does not yet compare it with `PinnedAdmin`; that enforcement is separate from this authorization-matrix update. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `register` | agent | An agent authorizes creation of its own profile and delegation to a controller. |
| `update_profile` | current profile controller | Metadata edits and controller rotation are delegated to the controller stored on the profile. |
| `deactivate` | stored admin | Offboarding is a governance action. |
| `reactivate` | stored admin | Restoring a deactivated profile is likewise governance-controlled. |
| `get_profile` | none | Read-only profile view. |
| `get_profile_opt` | none | Read-only optional profile probe. |

## `contracts/wallet`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none inside the contract | Records the deploy-time `initial_admin` under `PinnedAdmin`. |
| `initialize` | submitted admin | The address passed as `admin` must authenticate. The current implementation does not yet compare it with `PinnedAdmin`; that enforcement is separate from this authorization-matrix update. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `bind_wallet` | agent **and** wallet | Binding is a two-party decision: both the agent and the external wallet must consent. |
| `rebind_wallet` | agent **and** replacement wallet | Replacing an existing binding again requires consent from the agent and the wallet being bound. |
| `update_spend_limit` | agent | The bound agent controls its policy limit. |
| `set_enabled` | agent | The bound agent controls the normal enabled/disabled state. |
| `admin_deactivate` | stored admin | Emergency deactivation is reserved for governance. |
| `get_binding` | none | Read-only binding view. |
| `get_binding_opt` | none | Read-only optional binding probe. |

## `contracts/payments`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none inside the contract | Records the deploy-time `initial_admin` as `PinnedAdmin`. |
| `initialize` | submitted admin, which must also equal the pinned admin | The submitted admin authenticates and `require_initial_admin` enforces the deploy-time pin before configuration is stored. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `schema_version` | none | Read-only schema-version view. |
| `get_config` | none | Read-only configuration view. |
| `get_next_intent_id` | none | Read-only counter view. |
| `create_intent` | payer agent | Opening a payment obligation requires the payer agent's signature. |
| `settle_intent` | caller must be the stored admin **and** authenticate | `require_caller` first enforces the typed stored-admin role, then the same caller must provide authorization before finalization. |
| `cancel_intent` | intent payer | The payer captured on the stored intent must authenticate before cancellation. |
| `set_fee_bps` | stored admin | Only the current payments admin may change the fee. |
| `set_treasury` | stored admin | Only the current payments admin may redirect the treasury. |
| `transfer_admin` | stored admin | Payments currently performs a single-step handover: the authenticated current admin directly replaces the stored admin with `new_admin`. |
| `get_intent` | none | Read-only intent view. |
| `get_intent_opt` | none | Read-only optional intent probe. |

## Cross-cutting authorization invariants

1. **Stored principals gate privileged actions.** After initialization, admin-gated functions read the current `Admin` value from contract storage rather than trusting an arbitrary admin argument.
2. **Protocol handover is two-step.** The old protocol admin remains active after `transfer_admin`; only the stored pending admin may call `accept_admin`, after which the pending key is removed.
3. **Payments handover is currently single-step.** Its `transfer_admin` directly replaces the stored admin, so it should not be assumed to share protocol's pending-accept semantics.
4. **Identity delegation is per profile.** `update_profile` authenticates the controller stored on that `AgentProfile`; controller rotation therefore changes who can authorize later edits.
5. **Wallet binding and rebinding are dual-consent operations.** Both the agent and the wallet being bound authenticate, while later policy updates are agent-authorized and emergency deactivation is admin-authorized.
6. **Payer authority is captured at intent creation.** `create_intent` authenticates the payer agent, and `cancel_intent` later authenticates the payer address stored on that intent.
7. **Typed role and signature failures are distinct where both are used.** `payments::settle_intent` checks that `caller` equals the stored admin with a typed `Unauthorized` error before requiring that caller's cryptographic authorization.
