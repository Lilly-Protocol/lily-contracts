#![no_std]

//! Shared Soroban primitives used across Lily Protocol contracts.

use soroban_sdk::{contracterror, contracttype, panic_with_error, Address, Env, Symbol};

/// Maximum basis points accepted by percentage-based configuration.
pub const MAX_BPS: u32 = 10_000;

/// Shared on-chain protocol interface version exposed by every contract.
pub const PROTOCOL_VERSION: u32 = 1;

/// TTL threshold used when refreshing instance storage.
pub const INSTANCE_BUMP_THRESHOLD: u32 = 17_280;

/// TTL target used when refreshing instance storage.
pub const INSTANCE_BUMP_AMOUNT: u32 = 172_800;

/// Common protocol errors used across the initial contracts.
///
/// Numeric discriminants are part of the on-wire encoding; new variants are
/// appended at the end so previously encoded values keep their identity.
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
    /// Raised when a reentrancy guard is already held in the current call.
    ReentrantCall = 10,
}

/// Shared payment status used by settlement-oriented contracts.
#[non_exhaustive]
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

/// Single canonical entry point for cryptographic signature checks.
///
/// Soroban 22 has no fallible `require_auth` variant: `Address::require_auth`
/// traps with a host-level `Auth` error when the signature is missing or
/// invalid, and that trap occurs outside user code. Centralising every
/// signature check through this helper gives contracts one auditable call
/// site, and lets integrators map *both* failure classes onto the same
/// application-level concept (see "Auth and error mapping" in
/// `CONTRIBUTING.md`):
///
/// - signature failure  -> host `Auth` error (from this call)
/// - role failure       -> typed `ProtocolError::Unauthorized` (from
///   [`require_caller`])
pub fn require_auth_or_error(addr: &Address, env: &Env) {
    let _ = env;
    addr.require_auth();
}

/// Typed role guard: raise `ProtocolError::Unauthorized` when `caller` is not
/// the expected principal.
///
/// Use this *before* [`require_auth_or_error`] whenever the contract knows
/// which principal must authorize an action (admin settles, bound agent
/// transfers, ...). Typed errors survive the contract boundary as a
/// structured `ContractError`, unlike the host `Auth` trap, so off-chain
/// consumers can branch on it explicitly.
pub fn require_caller(env: &Env, caller: &Address, expected: &Address) {
    require(env, caller == expected, ProtocolError::Unauthorized);
}

/// Reentrancy guard backed by an instance-storage flag.
///
/// `acquire` panics with [`ProtocolError::ReentrantCall`] when the flag is
/// already set in the current call, which is exactly the reentrant-invocation
/// case: the outer frame still owns the guard and has not dropped it. The
/// flag is released automatically when the guard leaves scope, including on
/// panic unwind, so state-transition functions can simply hold one across
/// their body:
///
/// ```ignore
/// let _guard = NonReentrantGuard::acquire(&env, symbol_short!("reent"));
/// // ... read transition state, invoke external contracts, write new state
/// ```
///
/// `key` must be a `Symbol` unique to the contract instance (e.g.
/// `symbol_short!("reent")`) so guards never collide with business storage
/// keys.
#[derive(Clone)]
pub struct NonReentrantGuard {
    env: Env,
    key: Symbol,
}

impl NonReentrantGuard {
    /// Acquire the reentrancy guard for `key` in the current contract.
    ///
    /// # Panics
    /// Raises [`ProtocolError::ReentrantCall`] if the guard is already held.
    pub fn acquire(env: &Env, key: Symbol) -> Self {
        require(env, !env.storage().instance().has(&key), ProtocolError::ReentrantCall);
        env.storage().instance().set(&key, &true);
        Self { env: env.clone(), key }
    }
}

impl Drop for NonReentrantGuard {
    fn drop(&mut self) {
        if self.env.storage().instance().has(&self.key) {
            self.env.storage().instance().remove(&self.key);
        }
    }
}

/// Keep instance storage alive for long-lived protocol state.
pub fn bump_instance(env: &Env) {
    env.storage().instance().extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

#[cfg(test)]
mod test {
    use super::{require_caller, NonReentrantGuard, ProtocolError};
    use crate::require;
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::Address as _, Address, Env,
    };

    #[contract]
    pub struct Gatekeeper;

    #[contractimpl]
    impl Gatekeeper {
        /// Report that the acquire/release cycle works and clears the flag.
        pub fn acquire_check(env: Env) -> bool {
            require(
                &env,
                !env.storage().instance().has(&symbol_short!("reent")),
                ProtocolError::AlreadyExists,
            );
            let guard = NonReentrantGuard::acquire(&env, symbol_short!("reent"));
            let held = env.storage().instance().has(&symbol_short!("reent"));
            drop(guard);
            held && !env.storage().instance().has(&symbol_short!("reent"))
        }

        /// Two nested acquisitions in one frame must be rejected.
        pub fn double(env: Env) {
            let _first = NonReentrantGuard::acquire(&env, symbol_short!("reent"));
            let _second = NonReentrantGuard::acquire(&env, symbol_short!("reent"));
        }

        /// Hold the guard while invoking another contract instance.
        pub fn hold_and_hop(env: Env, hop: Address, origin: Address) {
            let _guard = NonReentrantGuard::acquire(&env, symbol_short!("reent"));
            GatekeeperClient::new(&env, &hop).hop_back(&origin);
        }

        /// Trampoline: call back into the origin instance.
        pub fn hop_back(env: Env, origin: Address) {
            GatekeeperClient::new(&env, &origin).reenter();
        }

        /// Invoked while the origin still holds its guard: must be rejected.
        pub fn reenter(env: Env) {
            let _guard = NonReentrantGuard::acquire(&env, symbol_short!("reent"));
        }

        /// Typed role check used by the #100 mapping tests.
        pub fn check_caller(env: Env, caller: Address, expected: Address) {
            require_caller(&env, &caller, &expected);
        }
    }

    fn env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    fn register(env: &Env) -> Address {
        env.register(Gatekeeper, ())
    }

    #[test]
    fn guard_acquire_and_release_cycle() {
        let env = env();
        let id = register(&env);
        let client = GatekeeperClient::new(&env, &id);
        assert!(client.acquire_check());
        // After release the key is gone, so a fresh acquisition succeeds.
        client.reenter();
    }

    // Typed guard error: ProtocolError::ReentrantCall = 10.
    #[test]
    #[should_panic = "Error(Contract, #10)"]
    fn guard_rejects_nested_acquisition_in_same_frame() {
        let env = env();
        let id = register(&env);
        let client = GatekeeperClient::new(&env, &id);
        client.double();
    }

    // Cross-contract reentry is rejected on the Soroban 22 host
    // ("Contract re-entry is not allowed"); the typed guard remains the
    // shared defense-in-depth layer for SDK builds where reentry is
    // permitted.
    #[test]
    #[should_panic = "Error(Context, InvalidAction)"]
    fn guard_rejects_reentrant_invocation_across_contracts() {
        let env = env();
        let origin = register(&env);
        let hop = register(&env);
        let client = GatekeeperClient::new(&env, &origin);
        client.hold_and_hop(&hop, &origin);
    }

    // Typed role error: ProtocolError::Unauthorized = 3.
    #[test]
    #[should_panic = "Error(Contract, #3)"]
    fn require_caller_raises_typed_unauthorized() {
        let env = env();
        let id = register(&env);
        let (caller, expected) = (Address::generate(&env), Address::generate(&env));
        let client = GatekeeperClient::new(&env, &id);
        client.check_caller(&caller, &expected);
    }

    #[test]
    fn require_caller_passes_for_matching_principal() {
        let env = env();
        let id = register(&env);
        let caller = Address::generate(&env);
        let client = GatekeeperClient::new(&env, &id);
        // Matching principal: the typed check must not raise Unauthorized,
        // so a direct call returns cleanly.
        client.check_caller(&caller, &caller);
    }
}
