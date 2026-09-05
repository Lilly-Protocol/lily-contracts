# Authorization Model

This document is the function-by-function authorization matrix for Lily Protocol.
Every `require_auth()` call site in the workspace is listed, along with the
reasoning behind each choice. The model separates **protocol governance**
(admin), **agent lifecycle** (the agent itself), **delegated control**
(controller), and **funding/policy ownership** (payer, wallet) so that each
address holds only the minimum authority its role requires.

## Vocabulary

| Role | Meaning |
| --- | --- |
| `admin` | The protocol's governance address, stored at initialization per contract. |
| `initial_admin` | The deployer-pinned address authorized to initialize the contract. |
| `pending_admin` | The proposed new admin in a two-step admin transfer, authorized to accept governance. |
| `agent` | A registered Lily agent (an `Address` with a profile). |
| `controller` | The address delegated by an agent to manage its profile. |
| `payer_agent` | The agent that opens a payment intent and funds it. |
| `wallet` | The external wallet bound to an agent for settlement. |

Views (`*get_*`, `is_initialized`, `schema_version`) require no authorization: they read state only and bump instance TTL (with documented exceptions for pre-initialization checks).

## `contracts/protocol`

### Two-Step Admin Handover
Protocol admin handover follows a secure two-step transfer pattern to prevent accidental transfers to unrecoverable addresses. First, the active admin calls `transfer_admin(new_admin)`, which stores `new_admin` in `DataKey::PendingAdmin` while keeping the current admin fully in power. Second, the proposed pending admin must sign and call `accept_admin()`, which replaces `DataKey::Admin` with `new_admin` and clears `DataKey::PendingAdmin`.

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none (deployer) | Captures the intended initial admin in `DataKey::PinnedAdmin` at deploy time. |
| `initialize` | initial pinned admin | One-shot bootstrap; caller must match `PinnedAdmin` to prevent front-running. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `schema_version` | none | Read-only view of protocol schema version. |
| `get_config` | none | Read-only view; consumers poll it constantly. |
| `get_pending_admin` | none | Read-only view returning the current pending admin address, if any. |
| `set_fee_bps` | stored admin | Changing the fee changes revenue split for every agent — a governance decision. |
| `set_treasury` | stored admin | Treasury is where fees land; only governance may redirect it. |
| `transfer_admin` | stored admin | Proposes a new admin address (step 1); the existing admin remains active until acceptance. |
| `accept_admin` | pending admin | Handover acceptance (step 2); only the proposed pending admin can claim admin authority. |

## `contracts/identity`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none (deployer) | Captures the intended initial admin in `DataKey::PinnedAdmin` at deploy time. |
| `initialize` | initial pinned admin | Establishes the governance address for the registry, requiring pinned deployer auth. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `register` | agent | An agent chooses its own controller and metadata on first registration; the controller is a *delegation* made by the agent, not an imposition. |
| `update_profile` | profile controller | Day-to-day profile management is delegated to the controller; the agent does not need custody of every call, and a deactivated profile fails before auth matters (`require(profile.active)`), so an old controller cannot resurrect a profile. |
| `deactivate` | stored admin | Deactivation is a governance action (offboarding an agent), which is why it is admin-gated rather than agent-gated. |
| `reactivate` | stored admin | Re-enabling a previously deactivated profile is a governance action requiring stored admin auth. |
| `get_profile` | none | Read-only view used by wallets, payments, and operators. |
| `get_profile_opt` | none | Read-only view returning `None` for unregistered agent addresses. |

## `contracts/wallet`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none (deployer) | Captures the intended initial admin in `DataKey::PinnedAdmin` at deploy time. |
| `initialize` | initial pinned admin | Establishes governance for the binding registry, requiring pinned deployer auth. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `bind_wallet` | agent **and** wallet | Binding is a two-party decision: the agent must opt in to use the wallet, and the wallet must consent to being bound. Dual auth prevents either side being pinned to the other. |
| `rebind_wallet` | agent **and** wallet | Replacing an existing binding requires consent from both the agent and the new wallet. |
| `update_spend_limit` | agent | Spend limits protect the *agent's* budget; only the agent (through its own auth) decides how much policy headroom exists. |
| `set_enabled` | agent | Enabling/disabling the binding is likewise the agent's policy choice. |
| `admin_deactivate` | stored admin | Emergency administrative deactivation of an agent's binding; gated by stored admin auth. |
| `get_binding` | none | Read-only view used by settlement checks. |
| `get_binding_opt` | none | Read-only view returning `None` if an agent has no binding. |

## `contracts/payments`

| Function | Required authorization | Why |
| --- | --- | --- |
| `__constructor` | none (deployer) | Captures the intended initial admin in `DataKey::PinnedAdmin` at deploy time. |
| `initialize` | initial pinned admin | Establishes governance plus treasury, fee, and wallet contract configuration. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `schema_version` | none | Read-only view of payments schema version. |
| `get_config` | none | Read-only view of active payments configuration. |
| `get_next_intent_id` | none | Read-only counter view. |
| `create_intent` | payer agent | Opening a payment obligation must be an act of the payer; the payer agent's auth is the commitment that binds it to pay. |
| `settle_intent` | stored admin (`caller`) | Settlement moves protocol-managed state to final; restricting it to admin keeps the lifecycle transition a governance act rather than something any participant can force. |
| `cancel_intent` | intent payer | Only the payer that opened the intent can rescind it; the payer reference is captured on the intent at creation so a replacement payer cannot cancel someone else's intent. |
| `set_fee_bps` | stored admin | Updating the settlement fee rate is a governance decision requiring stored admin auth. |
| `set_treasury` | stored admin | Updating the fee recipient treasury address requires stored admin auth. |
| `transfer_admin` | stored admin | Transferring payments settlement authority requires current stored admin auth. |
| `get_intent` | none | Read-only view used by payees and operators. |
| `get_intent_opt` | none | Read-only view returning `None` for non-existent intent IDs. |

## Cross-cutting invariants

1. **Auth before state checks.** `initialize` enforces one-time semantics via a
   `has(Initialized)` check *and* `require_auth()` on the initializer; views
   check `ensure_initialized` first so they fail with `NotInitialized` rather
   than reading empty state.
2. **Stored principals, not call arguments.** Every admin-gated function re-reads
   the `Admin` storage key instead of trusting an `admin` argument, so the
   current holder of record is the only valid signer.
3. **Delegation is per-record.** The controller is stored inside the agent's
   `AgentProfile`; changing controllers goes through the controller-authorized
   `update_profile` path, keeping delegation revocable by the current controller.
4. **Payer capture at creation.** `PaymentIntent.payer_agent` is written once by
   `create_intent` and then used for every later `cancel` auth check, so the
   payer's authority is fixed at commitment time.
