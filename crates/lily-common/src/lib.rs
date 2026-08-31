#![no_std]

//! Shared Soroban primitives used across Lily Protocol contracts.

use soroban_sdk::{contracterror, contracttype, panic_with_error, Env};

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
    Overflow = 10,
}

/// Shared payment status used by settlement-oriented contracts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Settled,
    Cancelled,
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

/// Increment a u64 counter, mapping overflow to `ProtocolError::Overflow`.
#[must_use]
#[inline]
pub fn checked_inc(env: &Env, value: u64) -> u64 {
    match value.checked_add(1) {
        Some(next) => next,
        None => panic_with_error!(env, ProtocolError::Overflow),
    }
}

/// Keep instance storage alive for long-lived protocol state.
pub fn bump_instance(env: &Env) {
    env.storage().instance().extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn checked_inc_increments() {
        let env = Env::default();
        assert_eq!(checked_inc(&env, 0), 1);
        assert_eq!(checked_inc(&env, u64::MAX - 1), u64::MAX);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #10)")]
    fn checked_inc_overflow_panics_typed_error() {
        let env = Env::default();
        assert_eq!(checked_inc(&env, u64::MAX - 1), u64::MAX);
        let _ = checked_inc(&env, u64::MAX);
    }

    #[test]
    fn require_true_does_not_panic() {
        let env = Env::default();
        require(&env, true, ProtocolError::Unauthorized);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn require_false_panics_with_typed_error() {
        let env = Env::default();
        require(&env, false, ProtocolError::Unauthorized);
    }

    #[test]
    fn require_non_empty_accepts_nonzero() {
        let env = Env::default();
        require_non_empty(&env, 1);
        require_non_empty(&env, 65_536);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #4)")]
    fn require_non_empty_rejects_zero() {
        let env = Env::default();
        require_non_empty(&env, 0);
    }

    #[test]
    fn require_valid_bps_accepts_boundaries() {
        let env = Env::default();
        require_valid_bps(&env, 0);
        require_valid_bps(&env, MAX_BPS);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn require_valid_bps_rejects_above_max() {
        let env = Env::default();
        require_valid_bps(&env, MAX_BPS + 1);
    }
}
