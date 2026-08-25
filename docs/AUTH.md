# Authorization model

Lily's contracts use Soroban's `Address::require_auth()` at each protected
entry point. The required signer is resolved from contract state where a role
is persistent (for example, the current admin or profile controller) and from
the function arguments where the actor is initiating an operation (for
example, an agent or wallet). A caller cannot select a different address and
act on its behalf: the selected address must authorize the invocation.

Functions marked **Public** do not call `require_auth()`. They may be invoked
without a signer, although normal validation still applies, such as requiring
the contract to be initialized or the requested record to exist.

## Authorization matrix

| Contract | Function | Required authorization | Rationale |
| --- | --- | --- | --- |
| Protocol | `initialize(admin, treasury, fee_bps)` | Proposed `admin` | Initialization installs a privileged configuration, so the address accepting the admin role must authorize it. The treasury does not authorize because it is only the destination recorded by the configuration. |
| Protocol | `is_initialized()` | Public | This exposes only whether initialization has occurred and does not read or change privileged state. |
| Protocol | `get_config()` | Public | Protocol configuration is on-chain public information and the function only reads it. |
| Protocol | `set_fee_bps(fee_bps)` | Current `admin` | Fee policy affects all protocol users and is therefore restricted to the admin stored during initialization or the latest admin transfer. |
| Protocol | `set_treasury(treasury)` | Current `admin` | Changing the fee destination is a protocol-wide privileged action. The new treasury is a destination, not a role being delegated authority, so it need not sign. |
| Protocol | `transfer_admin(new_admin)` | Current `admin` | Only the current authority may delegate the admin role. The new admin does not need to authorize the transfer and becomes authoritative for later calls. |
| Identity | `initialize(admin)` | Proposed `admin` | The address assuming administrative control must consent to initializing the registry. |
| Identity | `register(agent, controller, metadata_uri)` | `agent` | Registration creates a record in the agent's name, so the agent must authorize it. The controller is assigned authority for later profile updates but does not authorize registration. |
| Identity | `update_profile(agent, metadata_uri, new_controller)` | Current profile `controller` | Metadata maintenance and controller rotation belong to the controller already recorded for the agent. When rotating, the new controller does not authorize the same call; it controls subsequent updates. |
| Identity | `deactivate(agent)` | Registry `admin` | Deactivation is an administrative enforcement action rather than a self-service profile update. |
| Identity | `get_profile(agent)` | Public | Profiles are public registry data and reading one does not mutate its contents. |
| Wallet | `initialize(admin)` | Proposed `admin` | The address named as registry admin must consent to initialization, even though current post-initialization wallet operations are agent-controlled. |
| Wallet | `bind_wallet(agent, wallet, settlement_asset, spend_limit)` | Both `agent` and `wallet` | Binding links two independent addresses. Dual authorization proves the agent requests the policy and the wallet consents to being associated with it. |
| Wallet | `update_spend_limit(agent, spend_limit)` | `agent` | The agent owns the policy envelope and must authorize changes to its spend limit. |
| Wallet | `set_enabled(agent, enabled)` | `agent` | Enabling or disabling the agent's binding is controlled by that agent. |
| Wallet | `get_binding(agent)` | Public | A wallet binding is registry state and may be read without authority. |
| Payments | `initialize(admin, treasury, fee_bps)` | Proposed `admin` | The address accepting settlement authority must authorize the initial payment configuration. The treasury is only a configured destination. |
| Payments | `get_config()` | Public | Payment configuration is public on-chain state and this function is read-only. |
| Payments | `create_intent(payer_agent, payee_agent, amount, memo)` | `payer_agent` | Creating an intent commits the payer to a proposed payment, so the payer must authorize it. The payee receives no authority and need not sign. |
| Payments | `settle_intent(intent_id, settlement_reference)` | Payments `admin` | Settlement is the trusted finalization step and is restricted to the configured settlement administrator. Neither party can unilaterally mark an intent settled. |
| Payments | `cancel_intent(intent_id)` | Intent's recorded `payer_agent` | Only the payer that created the still-pending intent may cancel it. The signer comes from the stored intent rather than a caller-supplied address. |
| Payments | `get_intent(intent_id)` | Public | Intent details and status are readable on-chain state and the function does not alter the intent. |

## Role boundaries

- **Admin** authority is contract-specific. Initializing one Lily contract does
  not confer privileges in another, even if deployments choose the same
  address for both.
- **Agent** and **controller** are distinct identity roles. The agent authorizes
  its initial registration, while the stored controller manages later identity
  updates. Wallet policy and payment initiation continue to require the agent
  address directly.
- **Wallet** authority is additionally required only when creating a binding.
  Later policy changes are controlled by the bound agent.
- **Payer** authority can create and cancel an intent, but cannot settle it.
  Settlement remains an admin-only transition.
- Authorization does not replace state validation. Initialization guards,
  record-existence checks, value constraints, and payment-state rules are
  enforced independently of the signer checks.
