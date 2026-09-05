# Protocol Error Reference

`ProtocolError` is the shared error enum defined in `crates/lily-common/src/lib.rs`. All deployable contracts use these typed codes instead of raw panic messages. This document lists each variant, its numeric code, where it is raised, and example conditions that trigger it.

## Error table

| Code | Variant | Description | Raise sites |
|---|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` was called on a contract that already has `Initialized` set. | `identity::initialize`, `payments::initialize`, `protocol::initialize`, `wallet::initialize` |
| 2 | `NotInitialized` | A state-mutating or state-reading function was called before `initialize`. | All `ensure_initialized` helpers in `identity`, `payments`, `protocol`, `wallet` |
| 3 | `Unauthorized` | Caller is not the expected principal; raised via `require_caller`. | `payments::settle_intent` |
| 4 | `InvalidInput` | A caller-provided value violates a basic invariant (empty, non-positive, disabled, etc.). | `identity::update_profile` (profile inactive), `payments::create_intent` (amount ≤ 0), `wallet::bind_wallet`/`wallet::update_spend_limit` (spend_limit ≤ 0 or binding disabled) |
| 5 | `FeeBpsTooHigh` | A fee value exceeds `MAX_BPS` (10,000 = 100%). | `protocol::initialize`, `protocol::set_fee_bps`, `payments::initialize` (via `require_valid_bps`) |
| 6 | `AlreadyExists` | A unique record already exists and would be overwritten. | `identity::register` (agent profile already registered) |
| 7 | `MissingRecord` | A required storage record was not found. | `identity::get_profile_internal`, `payments::get_intent_internal`, `wallet::get_binding_internal` |
| 8 | `PaymentAlreadyFinalized` | A payment intent is no longer `Pending` when `settle_intent` or `cancel_intent` is called. | `payments::settle_intent`, `payments::cancel_intent` |
| 9 | `WalletAlreadyBound` | An attempt to bind a wallet to an agent that already has a binding record. | `wallet::bind_wallet` |
| 10 | `ReentrantCall` | A reentrant invocation into a guarded state transition was attempted. | `payments::settle_intent`, `payments::cancel_intent` (via `NonReentrantGuard::acquire`) |

## Per-contract details

### `identity`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — any external function called before `initialize`.
- `AlreadyExists` — `register` called for an `agent` that already has a `Profile` entry.
- `InvalidInput` — `update_profile` called on a profile whose `active` flag is `false`.
- `MissingRecord` — `get_profile`/`update_profile`/`deactivate` references an unregistered `agent`.

### `payments`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — any external function called before `initialize`.
- `Unauthorized` — `settle_intent` called by a non-admin caller via `require_caller`.
- `InvalidInput` — `create_intent` with `amount <= 0`; empty `memo`/`settlement_reference` raise via `require_non_empty` (which itself uses `InvalidInput`).
- `FeeBpsTooHigh` — `initialize` with `fee_bps > MAX_BPS`.
- `MissingRecord` — `settle_intent`/`cancel_intent`/`get_intent` references an unknown `intent_id`.
- `PaymentAlreadyFinalized` — `settle_intent` or `cancel_intent` called on an intent whose status is already `Settled` or `Cancelled`.
- `ReentrantCall` — `settle_intent` or `cancel_intent` invoked reentrantly during execution.

### `protocol`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — `get_config`, `set_fee_bps`, `set_treasury`, `transfer_admin` called before `initialize`.
- `FeeBpsTooHigh` — `initialize` or `set_fee_bps` with `fee_bps > MAX_BPS`.

### `wallet`

- `AlreadyInitialized` — `initialize` called twice.
- `NotInitialized` — any external function called before `initialize`.
- `InvalidInput` — `bind_wallet`/`update_spend_limit` with `spend_limit <= 0`, or `update_spend_limit` on a disabled binding.
- `WalletAlreadyBound` — `bind_wallet` called for an agent with an existing `Binding` entry.
- `MissingRecord` — `update_spend_limit`, `set_enabled`, or `get_binding` references an agent with no `Binding` entry.

## Notes

- `MAX_BPS` is defined as `10_000` in `lily-common` and represents 100% in basis points.
- `require_non_empty` maps zero-length strings to `InvalidInput`.
- `Unauthorized` is actively raised by `require_caller` when the caller principal does not match the expected role before `require_auth_or_error`.
- `ReentrantCall` is enforced by `NonReentrantGuard::acquire` across mutating execution windows.
