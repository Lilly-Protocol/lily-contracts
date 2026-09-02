//! Shared helpers for Lily Protocol contract tests.

use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Create an environment with auth mocking enabled.
#[must_use]
pub fn test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Generate a synthetic address for tests.
#[must_use]
pub fn test_address(env: &Env) -> Address {
    Address::generate(env)
}

/// Convert a Rust string slice into a Soroban string.
#[must_use]
pub fn soroban_string(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}
