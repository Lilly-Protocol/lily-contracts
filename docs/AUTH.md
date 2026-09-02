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
| initializer | The address passed to `initialize`; authorized exactly once to set the initial value. |
| `agent` | A registered Lily agent (an `Address` with a profile). |
| `controller` | The address delegated by an agent to manage its profile. |
| `payer_agent` | The agent that opens a payment intent and funds it. |
| `wallet` | The external wallet bound to an agent for settlement. |

Views (`*get_*`, `is_initialized`) require no authorization: they read state only and bump instance TTL.

## `contracts/protocol`

| Function | Required authorization | Why |
| --- | --- | --- |
| `initialize` | initializer admin | One-shot bootstrap; the address chosen at deploy time becomes admin. |
| `is_initialized` | none | Read-only bootstrap probe. |
| `get_config` | none | Read-only view; consumers poll it constantly. |
| `set_fee_bps` | stored admin | Changing the fee changes revenue split for every agent — a governance decision. |
| `set_treasury` | stored admin | Treasury is where fees land; only governance may redirect it. |
| `transfer_admin` | stored admin | Handing over governance must be ratified by the current holder; the old admin's authority is revoked because the stored `Admin` key is what every later check reads. |

## `contracts/identity`

| Function | Required authorization | Why |
| --- | --- | --- |
| `initialize` | initializer admin | Establishes the governance address for the registry. |
| `register` | agent | An agent chooses its own controller and metadata on first registration; the controller is a *delegation* made by the agent, not an imposition. |
| `update_profile` | profile controller | Day-to-day profile management is delegated to the controller; the agent does not need custody of every call, and a deactivated profile fails before auth matters (`require(profile.active)`), so an old controller cannot resurrect a profile. |
| `deactivate` | stored admin | Deactivation is a governance action (offboarding an agent), which is why it is admin-gated rather than agent-gated. |
| `get_profile` | none | Read-only view used by wallets, payments, and operators. |

## `contracts/wallet`

| Function | Required authorization | Why |
| --- | --- | --- |
| `initialize` | initializer admin | Establishes governance for the binding registry. |
| `bind_wallet` | agent **and** wallet | Binding is a two-party decision: the agent must opt in to use the wallet, and the wallet must consent to being bound. Dual auth prevents either side being pinned to the other. |
| `update_spend_limit` | agent | Spend limits protect the *agent's* budget; only the agent (through its own auth) decides how much policy headroom exists. |
| `set_enabled` | agent | Enabling/disabling the binding is likewise the agent's policy choice. |
| `get_binding` | none | Read-only view used by settlement checks. |

## `contracts/payments`

| Function | Required authorization | Why |
| --- | --- | --- |
| `initialize` | initializer admin | Establishes governance plus treasury/fee configuration. |
| `get_config` | none | Read-only view. |
| `create_intent` | payer agent | Opening a payment obligation must be an act of the payer; the payer agent's auth is the commitment that binds it to pay. |
| `settle_intent` | stored admin | Settlement moves protocol-managed state to final; restricting it to admin keeps the lifecycle transition a governance act rather than something any participant can force. |
| `cancel_intent` | intent payer | Only the payer that opened the intent can rescind it; the payer reference is captured on the intent at creation so a replacement payer cannot cancel someone else's intent. |
| `get_intent` | none | Read-only view used by payees and operators. |

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
