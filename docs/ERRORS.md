# Protocol Error Reference

`ProtocolError` is the shared error enum defined in `crates/lily-common/src/lib.rs`. All deployable contracts use these typed codes instead of raw panic messages. This document lists each variant, its numeric code, where it is raised, and example conditions that trigger it.

## Error table

| Code | Variant | Description | Raise sites |
|---|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` was called on a contract that already has `Initialized` set. | `identity::initialize`, `payments::initialize`, `protocol::initialize`, `wallet::initialize` |
| 2 | `NotInitialized` | A state-mutating or state-reading function was called before `initialize`. | All `ensure_initialized` helpers in `identity`, `payments`, `protocol`, `wallet` |
| 3 | `Unauthorized` | Caller principal does not match the required authorization role (enforced via `require_caller`). | `payments::settle_intent` |
| 4 | `InvalidInput` | A caller-provided value violates a basic invariant (empty, non-positive, disabled, etc.). | `identity::update_profile` (profile inactive), `payments::create_intent` (amount ≤ 0), `wallet::bind_wallet`/`wallet::rebind_wallet`/`wallet::update_spend_limit` (spend_limit ≤ 0 or binding disabled) |
| 5 | `FeeBpsTooHigh` | A fee value exceeds `MAX_BPS` (10,000 = 100%). | `protocol::initialize`, `protocol::set_fee_bps`, `payments::initialize`, `payments::set_fee_bps` (via `require_valid_bps`) |
| 6 | `AlreadyExists` | A unique record already exists and would be overwritten. | `identity::register` (agent profile already registered) |
| 7 | `MissingRecord` | A required storage record was not found. | `identity::get_profile_internal`, `identity::update_profile`, `identity::deactivate`, `identity::reactivate`, `payments::get_intent_internal`, `payments::settle_intent`, `payments::cancel_intent`, `protocol::accept_admin`, `wallet::get_binding_internal`, `wallet::rebind_wallet`, `wallet::update_spend_limit`, `wallet::set_enabled`, `wallet::admin_deactivate` |
| 8 | `PaymentAlreadyFinalized` | A payment intent is no longer `Pending` when `settle_intent` or `cancel_intent` is called. | `payments::settle_intent`, `payments::cancel_intent` |
| 9 | `WalletAlreadyBound` | An attempt to bind a wallet to an agent that already has an existing binding in persistent storage (regardless of enabled/disabled status). | `wallet::bind_wallet` |
| 10 | `ReentrantCall` | A reentrancy guard is already held in the current execution call (enforced via `NonReentrantGuard::acquire`). | `payments::settle_intent`, `payments::cancel_intent` |

## Per-contract details

### `identity`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — any external function called before `initialize`.
- `AlreadyExists` — `register` called for an `agent` that already has a `Profile` entry.
- `InvalidInput` — `update_profile` called on a profile whose `active` flag is `false`, or empty/whitespace-only metadata strings passed to `require_non_whitespace`.
- `MissingRecord` — `get_profile`, `update_profile`, `deactivate`, or `reactivate` references an unregistered `agent`.

### `payments`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — any external function called before `initialize`.
- `Unauthorized` — `settle_intent` called by a `caller` address that does not match the configured protocol `admin` (enforced via `require_caller`).
- `InvalidInput` — `create_intent` with `amount <= 0`; empty or whitespace-only `memo` or `settlement_reference` via `require_non_whitespace`.
- `FeeBpsTooHigh` — `initialize` or `set_fee_bps` with `fee_bps > MAX_BPS`.
- `MissingRecord` — `settle_intent`, `cancel_intent`, or `get_intent` references an unknown `intent_id`.
- `PaymentAlreadyFinalized` — `settle_intent` or `cancel_intent` called on an intent whose status is already `Settled` or `Cancelled`.
- `ReentrantCall` — `settle_intent` or `cancel_intent` invoked reentrantly while the `NonReentrantGuard` instance flag is already held.

### `protocol`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — `get_config`, `set_fee_bps`, `set_treasury`, `transfer_admin`, `accept_admin`, or `schema_version` called before `initialize`.
- `FeeBpsTooHigh` — `initialize` or `set_fee_bps` with `fee_bps > MAX_BPS`.
- `MissingRecord` — `accept_admin` called when no pending admin transfer is active (`PendingAdmin` storage key not found).

### `wallet`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — any external function called before `initialize`.
- `InvalidInput` — `bind_wallet`, `rebind_wallet`, or `update_spend_limit` with `spend_limit <= 0`, or `update_spend_limit` on a disabled binding.
- `WalletAlreadyBound` — `bind_wallet` called for an agent that already has a `Binding` record in persistent storage (any existing binding blocks `bind_wallet`, even if disabled; callers must use `rebind_wallet` to update or replace an existing binding).
- `MissingRecord` — `rebind_wallet`, `update_spend_limit`, `set_enabled`, `admin_deactivate`, or `get_binding` references an agent with no `Binding` entry in persistent storage.

## Notes

- `MAX_BPS` is defined as `10_000` in `lily-common` and represents 100% in basis points.
- `require_non_empty` and `require_non_whitespace` map empty or whitespace-only strings to `InvalidInput`.
- Signature authentication is enforced by `Address::require_auth()` (which traps with a host-level `Auth` error), whereas typed role/principal checks are enforced by `require_caller` which raises `ProtocolError::Unauthorized`.
- Reentrancy protection: Guarded state transitions use `NonReentrantGuard::acquire` backed by ephemeral instance storage to raise `ProtocolError::ReentrantCall` if invoked reentrantly.
