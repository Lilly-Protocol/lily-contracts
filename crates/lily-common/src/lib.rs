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

/// Keep instance storage alive for long-lived protocol state.
pub fn bump_instance(env: &Env) {
    env.storage().instance().extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::{require_valid_bps, Env, MAX_BPS};
    use proptest::prelude::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn validation_panics(fee_bps: u32) -> bool {
        let env = Env::default();
        catch_unwind(AssertUnwindSafe(|| require_valid_bps(&env, fee_bps))).is_err()
    }

    proptest! {
        #[test]
        fn accepts_every_basis_point_value_through_the_maximum(fee_bps in 0..=MAX_BPS) {
            prop_assert!(!validation_panics(fee_bps));
        }

        #[test]
        fn rejects_every_basis_point_value_above_the_maximum(
            fee_bps in (MAX_BPS + 1)..=u32::MAX,
        ) {
            prop_assert!(validation_panics(fee_bps));
        }
    }

    #[test]
    fn validates_the_exact_basis_point_boundary() {
        assert!(!validation_panics(10_000));
        assert!(validation_panics(10_001));
    }
}
