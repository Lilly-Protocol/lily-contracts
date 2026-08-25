# Authorization Model

This document is the function-by-function authorization matrix for the Lily
Protocol Soroban contracts. Every public function is listed with the parties
that must authorize the call (`require_auth()` call sites) and the reasoning
behind each choice.

Legend:

- **Admin** — protocol administrator set at `initialize` and rotatable via
  `transfer_admin`.
- **Agent** — a registered agent identity (subject of an `AgentProfile`).
- **Controller** — the address recorded in `AgentProfile.controller` that can
  manage an agent's profile metadata.
- **Payer Agent** — the agent that created a payment intent.
- **Wallet** — the settlement wallet bound to an agent.

## Identity contract

| Function | Required authorizations | Reasoning |
| --- | --- | --- |
| `initialize(admin)` | Admin (first caller becomes admin) | One-time bootstrap; only the deployer should be able to seed storage. |
| `register(agent, controller, metadata_uri)` | Agent | Only the agent itself may claim its identity; self-registration keeps onboarding permissionless. |
| `update_profile(agent, metadata_uri, new_controller)` | Controller | Profile data is managed by the controller so recovery/rotation does not require the original agent key. |
| `deactivate(agent)` | Admin | Deactivation is a protocol-level kill switch, not a user action; prevents agents from dodging disputes. |
| `get_profile(agent)` | None (read-only) | Public view; no state change. |

## Payments contract

| Function | Required authorizations | Reasoning |
| --- | --- | --- |
| `initialize(admin, treasury, fee_bps)` | Admin (first caller becomes admin) | One-time bootstrap of treasury and fee configuration. |
| `create_intent(payer_agent, payee_agent, amount, memo)` | Payer Agent | Only the party committing funds may open an intent. |
| `settle_intent(intent_id, settlement_reference)` | Admin | Settlement moves real value out of escrow and is restricted to the operator until automated settlement lands. |
| `cancel_intent(intent_id)` | Payer Agent | The payer owns the pending intent and may withdraw it before finalization; already-finalized intents are rejected by state checks. |
| `get_config()` / `get_intent(intent_id)` | None (read-only) | Public views; no state change. |

## Protocol contract

| Function | Required authorizations | Reasoning |
| --- | --- | --- |
| `initialize(admin, treasury, fee_bps)` | Admin (first caller becomes admin) | One-time bootstrap. |
| `set_fee_bps(fee_bps)` | Admin | Fee changes are economically sensitive and admin-only. |
| `set_treasury(treasury)` | Admin | Redirecting fee flows must not be callable by agents. |
| `transfer_admin(new_admin)` | Admin (current) | Two-step style rotation guarded by the current admin; prevents hostile takeover. |
| `is_initialized()` / `get_config()` | None (read-only) | Public views; no state change. |

## Wallet contract

| Function | Required authorizations | Reasoning |
| --- | --- | --- |
| `initialize(admin)` | Admin (first caller becomes admin) | One-time bootstrap. |
| `bind_wallet(agent, wallet, settlement_asset, spend_limit)` | Agent + Wallet (dual) | Binding custody to an agent requires consent from both sides; dual auth prevents binding someone else's wallet without their key. |
| `update_spend_limit(agent, spend_limit)` | Agent | Spending policy belongs to the agent that owns the funds flow. |
| `set_enabled(agent, enabled)` | Agent | Enable/disable mirrors spend-limit ownership. |
| `get_binding(agent)` | None (read-only) | Public view; no state change. |

## Policy notes

- **Additive variants:** error and status enums are (or will be)
  `#[non_exhaustive]` so downstream matchers survive additive changes — see
  the policy documented in #47.
- **Admin rotation:** `transfer_admin` requires the *current* admin's
  authorization; there is no renounce path, so the contract can always recover.
- **Read-only functions** never call `require_auth()`; they cannot mutate
  storage or emit events.
