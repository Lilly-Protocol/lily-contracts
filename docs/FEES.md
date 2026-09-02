# Fee Semantics and Treasury Handling

This document describes how protocol fees are configured, how they are calculated, and how the treasury address participates in fee collection.

## Basis-point fee configuration

Both the global `protocol` contract and the `payments` contract store a `fee_bps` value:

- `fee_bps` is an unsigned 32-bit integer interpreted as **basis points**.
- `MAX_BPS = 10_000` represents 100%.
- A `fee_bps` value of `100` therefore represents a 1% fee (`100 / 10_000`).

The shared helper `require_valid_bps` in `crates/lily-common/src/lib.rs` rejects any value greater than `MAX_BPS`, ensuring the fee can never exceed 100%.

## Treasury role

The treasury `Address` is stored in:

- `contracts/protocol/src/lib.rs` as protocol-wide configuration.
- `contracts/payments/src/lib.rs` as the settlement-specific treasury.

The treasury is the destination to which collected fees will be transferred when a payment intent is settled. Only the admin can update the treasury address via `set_treasury`.

## Fee application at settlement

Although the contracts currently configure fees, the actual transfer logic is intentionally left for a future settlement integration. When implemented, the expected behavior is:

1. A payer creates a `PaymentIntent` with a gross `amount`.
2. Upon settlement, the gross amount is split into:
   - `fee_amount = (amount * fee_bps) / MAX_BPS`
   - `net_amount = amount - fee_amount`
3. `fee_amount` is credited to the treasury.
4. `net_amount` is credited to the payee.

This keeps the fee calculation on-chain transparent and deterministic.

## Rounding rules

Fee calculations use integer arithmetic. The protocol uses **floor rounding**:

```rust
let fee_amount = (amount * fee_bps) / MAX_BPS;
```

This means the fee is rounded down to the smallest representable unit of the settlement asset. The protocol absorbs any rounding residue rather than overcharging the payer.

### Examples

| Gross amount | fee_bps | Fee calculation | Fee charged | Net to payee |
|---|---|---|---|---|
| 1_000_000 | 100 (1%) | `(1_000_000 * 100) / 10_000` | 10_000 | 990_000 |
| 1_000_000 | 50 (0.5%) | `(1_000_000 * 50) / 10_000` | 5_000 | 995_000 |
| 100 | 30 (0.3%) | `(100 * 30) / 10_000` | 0 | 100 |
| 10_000 | 1 (0.01%) | `(10_000 * 1) / 10_000` | 1 | 9_999 |

## Boundary behavior

- `fee_bps = 0` results in no fee: `fee_amount = 0`.
- `fee_bps = MAX_BPS` (10_000) results in the entire amount being taken as a fee: `net_amount = 0`.
- Values outside the `0..=MAX_BPS` range are rejected at configuration time by `require_valid_bps`.

## Updating fees

- The protocol admin calls `set_fee_bps` on the `protocol` contract to change the global fee.
- The protocol admin calls `set_fee_bps` on the `payments` contract to change the settlement-specific fee.
- Each successful update emits a `("fee", admin)` event carrying the new `fee_bps` value.

## Open design questions for future settlement work

- Whether fees are calculated once at settlement or cached in the intent record.
- Whether the treasury receives the fee as the same settlement asset or through a separate conversion path.
- Whether partial settlement or refund paths also apply fees.

These decisions will be documented in the settlement integration design when that work is undertaken.
