#![no_std]

//! Shared Soroban primitives used across Lily Protocol contracts.

use soroban_sdk::{contracterror, contracttype, panic_with_error, Address, Env, IntoVal, TryFromVal, Val};

/// Maximum basis points accepted by percentage-based configuration.
pub const MAX_BPS: u32 = 10_000;

/// TTL threshold used when refreshing instance storage.
pub const INSTANCE_BUMP_THRESHOLD: u32 = 17_280;

/// TTL target used when refreshing instance storage.
pub const INSTANCE_BUMP_AMOUNT: u32 = 172_800;

/// Common protocol errors used across the initial contracts.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProtocolError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidInput = 4,
    FeeBpsTooHigh = 5,
    AlreadyExists = 6,
    MissingRecord = 7,
    PaymentAlreadyFinalized = 8,
    WalletAlreadyBound = 9,
}

/// Shared payment status used by settlement-oriented contracts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Settled,
    Cancelled,
}

/// Shared protocol configuration fields used by both the protocol and payments
/// contracts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolConfig {
    pub admin: Address,
    pub treasury: Address,
    pub fee_bps: u32,
}

/// Panic with a typed contract error when a condition is not satisfied.
pub fn require(env: &Env, condition: bool, error: ProtocolError) {
    if !condition {
        panic_with_error!(env, error);
    }
}

/// Reject empty Soroban strings for storage-bound metadata fields.
pub fn require_non_empty(env: &Env, len: u32) {
    require(env, len > 0, ProtocolError::InvalidInput);
}

/// Reject fee values greater than 100%.
pub fn require_valid_bps(env: &Env, fee_bps: u32) {
    require(env, fee_bps <= MAX_BPS, ProtocolError::FeeBpsTooHigh);
}

/// Compute the fee portion of `amount` given `fee_bps` basis points.
///
/// Uses integer division truncated toward zero. Panics on overflow or if
/// `fee_bps` exceeds [`MAX_BPS`].
pub fn compute_fee(amount: i128, fee_bps: u32) -> i128 {
    assert!(fee_bps <= MAX_BPS, "fee_bps exceeds MAX_BPS");
    amount
        .checked_mul(fee_bps as i128)
        .and_then(|v| v.checked_div(MAX_BPS as i128))
        .unwrap_or_else(|| panic!("fee computation overflow"))
}

/// Compute the net amount after deducting the fee.
///
/// Panics on overflow or if `fee_bps` exceeds [`MAX_BPS`].
pub fn compute_net(amount: i128, fee_bps: u32) -> i128 {
    let fee = compute_fee(amount, fee_bps);
    amount
        .checked_sub(fee)
        .unwrap_or_else(|| panic!("net computation overflow"))
}

/// Read a value from instance storage, panicking with `MissingRecord` if absent.
///
/// This replaces per-contract `unwrap_optimized` reads with a typed error path.
pub fn read_instance<T: TryFromVal<Env, Val>>(env: &Env, key: impl IntoVal<Env, Val>) -> T {
    env.storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(env, ProtocolError::MissingRecord))
}

/// Keep instance storage alive for long-lived protocol state.
pub fn bump_instance(env: &Env) {
    env.storage().instance().extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

#[cfg(test)]
mod tests {
    use super::{compute_fee, compute_net, MAX_BPS};

    #[test]
    fn computes_zero_fee() {
        assert_eq!(compute_fee(1_000_000, 0), 0);
        assert_eq!(compute_net(1_000_000, 0), 1_000_000);
    }

    #[test]
    fn computes_full_fee() {
        assert_eq!(compute_fee(1_000_000, MAX_BPS), 1_000_000);
        assert_eq!(compute_net(1_000_000, MAX_BPS), 0);
    }

    #[test]
    fn computes_fifty_bps() {
        assert_eq!(compute_fee(10_000, 50), 50);
        assert_eq!(compute_net(10_000, 50), 9_950);
    }

    #[test]
    fn truncates_fractional_fee() {
        assert_eq!(compute_fee(100, 33), 0);
        assert_eq!(compute_fee(10_000, 33), 33);
    }

    #[test]
    fn handles_negative_amounts() {
        assert_eq!(compute_fee(-10_000, 100), -100);
        assert_eq!(compute_net(-10_000, 100), -9_900);
    }

    #[test]
    #[should_panic]
    fn rejects_fee_bps_above_max() {
        compute_fee(100, MAX_BPS + 1);
    }
}
