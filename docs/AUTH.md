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

Views (`*get_*`, `is_initialized`, `schema_version`) require no authorization: they read state only and bump instance TTL.

## `contracts/protocol`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none inside the contract | Records the deploy-time `initial_admin` as `PinnedAdmin`; constructor invocation is part of deployment rather than an authenticated runtime governance call. |
| `initialize` | submitted admin, which must also equal the pinned admin | The submitted admin signs the bootstrap call, and `require_initial_admin` rejects an address different from the one pinned at deployment. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `schema_version` | none | Read-only schema contract version view. |
| `get_config` | none | Read-only view; consumers poll it constantly. |
| `get_pending_admin` | none | Read-only view returning the current pending admin address, if any. |
| `set_fee_bps` | stored admin | Changing the fee changes revenue split for every agent — a governance decision. |
| `set_treasury` | stored admin | Treasury is where fees land; only governance may redirect it. |
| `transfer_admin` | stored admin | Proposes a new governance address by writing `PendingAdmin`. The current admin remains active and retains authority until the pending admin accepts. |
| `accept_admin` | pending admin | Finalizes the two-step governance handover. Only the proposed pending admin may authorize acceptance; on execution, `Admin` is updated, `PendingAdmin` is cleared, and the old admin's authority is revoked. |

### Protocol Two-Step Admin Handover
Governance handover uses a two-step pattern (`transfer_admin` followed by `accept_admin`) to prevent irrecoverable transfer to an erroneous address. Calling `transfer_admin(new_admin)` sets `DataKey::PendingAdmin` to the proposed recipient while leaving the existing admin fully empowered. Only the nominated pending admin can call `accept_admin()`, which verifies `pending_admin.require_auth()`, overwrites `DataKey::Admin`, clears `DataKey::PendingAdmin`, and emits the handover event. Until `accept_admin()` completes, the current admin retains full administrative authority.

## `contracts/identity`

| Function | Required authorization | Why |
| --- | --- | --- |
| `initialize` | initializer admin | Establishes the governance address for the registry. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `register` | agent | An agent chooses its own controller and metadata on first registration; the controller is a *delegation* made by the agent, not an imposition. |
| `update_profile` | profile controller | Day-to-day profile management is delegated to the controller; the agent does not need custody of every call, and a deactivated profile fails before auth matters (`require(profile.active)`), so an old controller cannot resurrect a profile. |
| `deactivate` | stored admin | Deactivation is a governance action (offboarding an agent), which is why it is admin-gated rather than agent-gated. |
| `reactivate` | stored admin | Reactivation restores an offboarded agent to active status; gated strictly by protocol governance (admin) to ensure re-entry satisfies compliance/audit requirements. |
| `get_profile` | none | Read-only view used by wallets, payments, and operators. |
| `get_profile_opt` | none | Read-only optional view returning `Option<AgentProfile>`. |

## `contracts/wallet`

| Function | Required authorization | Why |
| --- | --- | --- |
| `initialize` | initializer admin | Establishes governance for the binding registry. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `bind_wallet` | agent **and** wallet | Binding is a two-party decision: the agent must opt in to use the wallet, and the wallet must consent to being bound. Dual auth prevents either side being pinned to the other. |
| `rebind_wallet` | agent **and** new wallet | Rebinding changes the bound external settlement wallet; requires dual authorization from both the agent and the new wallet to prevent unconsented reassignment. |
| `update_spend_limit` | agent | Spend limits protect the *agent's* budget; only the agent (through its own auth) decides how much policy headroom exists. |
| `set_enabled` | agent | Enabling/disabling the binding is likewise the agent's policy choice. |
| `admin_deactivate` | stored admin | Emergency administrative deactivation of an agent's wallet binding; admin-gated governance action to freeze malicious or compromised agent settlement paths. |
| `get_binding` | none | Read-only view used by settlement checks. |
| `get_binding_opt` | none | Read-only optional view returning `Option<WalletBinding>`. |

## `contracts/payments`

| Function | Required authorization | Why |
| --- | --- | --- |
| `initialize` | initializer admin | Establishes governance plus treasury/fee configuration. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `schema_version` | none | Read-only schema version view. |
| `get_config` | none | Read-only view. |
| `get_next_intent_id` | none | Read-only counter view returning the next sequence identifier. |
| `create_intent` | payer agent | Opening a payment obligation must be an act of the payer; the payer agent's auth is the commitment that binds it to pay. |
| `settle_intent` | stored admin | Settlement moves protocol-managed state to final; restricting it to admin keeps the lifecycle transition a governance act rather than something any participant can force. |
| `cancel_intent` | intent payer | Only the payer that opened the intent can rescind it; the payer reference is captured on the intent at creation so a replacement payer cannot cancel someone else's intent. |
| `set_fee_bps` | stored admin | Fee adjustments are governance decisions modifying protocol take rates. |
| `set_treasury` | stored admin | Redirecting protocol fee payouts requires governance authority. |
| `transfer_admin` | stored admin | Administrative handover of the payments contract governance address. |
| `get_intent` | none | Read-only view used by payees and operators. |
| `get_intent_opt` | none | Read-only optional view returning `Option<PaymentIntent>`. |

## Cross-cutting authorization invariants

1. **Stored principals gate privileged actions.** After initialization, admin-gated functions read the current `Admin` value from contract storage rather than trusting an arbitrary admin argument.
2. **Protocol handover is two-step.** The old protocol admin remains active after `transfer_admin`; only the stored pending admin may call `accept_admin`, after which the pending key is removed.
3. **Payments handover is currently single-step.** Its `transfer_admin` directly replaces the stored admin, so it should not be assumed to share protocol's pending-accept semantics.
4. **Identity delegation is per profile.** `update_profile` authenticates the controller stored on that `AgentProfile`; controller rotation therefore changes who can authorize later edits.
5. **Wallet binding and rebinding are dual-consent operations.** Both the agent and the wallet being bound authenticate, while later policy updates are agent-authorized and emergency deactivation is admin-authorized.
6. **Payer authority is captured at intent creation.** `create_intent` authenticates the payer agent, and `cancel_intent` later authenticates the payer address stored on that intent.
7. **Typed role and signature failures are distinct where both are used.** `payments::settle_intent` checks that `caller` equals the stored admin with a typed `Unauthorized` error before requiring that caller's cryptographic authorization.
