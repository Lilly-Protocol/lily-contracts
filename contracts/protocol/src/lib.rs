#![no_std]

//! Global protocol configuration contract for Lily Protocol.

use lily_common::{
    bump_instance, require, require_auth_or_error, require_valid_bps, ProtocolError,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
};

#[contract]
pub struct ProtocolContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolConfig {
    pub admin: Address,
    pub treasury: Address,
    pub fee_bps: u32,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Treasury,
    FeeBps,
    Initialized,
    PinnedAdmin,
}

#[contractimpl]
impl ProtocolContract {
    /// Capture the intended initial admin at deploy time.
    ///
    /// `initialize` only accepts this exact address, so a front-runner cannot
    /// claim a fresh deployment with their own admin.
    pub fn __constructor(env: Env, initial_admin: Address) {
        env.storage().instance().set(&DataKey::PinnedAdmin, &initial_admin);
    }

    /// Initialize protocol-wide configuration once.
    ///
    /// The initial admin must match the address pinned by the constructor at
    /// deploy time, preventing initialization front-running.
    pub fn initialize(env: Env, admin: Address, treasury: Address, fee_bps: u32) {
        require(
            &env,
            !env.storage().instance().has(&DataKey::Initialized),
            ProtocolError::AlreadyInitialized,
        );
        require_initial_admin(&env, &admin);
        require_valid_bps(&env, fee_bps);

        require_auth_or_error(&admin, &env);

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        env.storage().instance().set(&DataKey::Initialized, &true);
        bump_instance(&env);

        env.events().publish(
            (symbol_short!("init"), admin.clone()),
            ProtocolConfig { admin, treasury, fee_bps },
        );
    }

    /// Return whether the contract has been initialized.
    #[must_use]
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Fetch the current protocol configuration.
    #[must_use]
    pub fn get_config(env: Env) -> ProtocolConfig {
        ensure_initialized(&env);
        bump_instance(&env);

        ProtocolConfig {
            admin: get_admin(&env),
            treasury: env.storage().instance().get(&DataKey::Treasury).unwrap_optimized(),
            fee_bps: env.storage().instance().get(&DataKey::FeeBps).unwrap_optimized(),
        }
    }

    /// Update the protocol fee in basis points.
    pub fn set_fee_bps(env: Env, fee_bps: u32) {
        ensure_initialized(&env);
        require_valid_bps(&env, fee_bps);

        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        bump_instance(&env);
        env.events().publish((symbol_short!("fee"), admin), fee_bps);
    }

    /// Update the treasury address used for fee collection.
    pub fn set_treasury(env: Env, treasury: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

        env.storage().instance().set(&DataKey::Treasury, &treasury);
        bump_instance(&env);
        env.events().publish((symbol_short!("treasury"), admin), treasury);
    }

    /// Transfer protocol admin authority.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        bump_instance(&env);
        env.events().publish((symbol_short!("admin"), admin), new_admin);
    }
}

fn ensure_initialized(env: &Env) {
    require(
        env,
        env.storage().instance().has(&DataKey::Initialized),
        ProtocolError::NotInitialized,
    );
}

fn require_initial_admin(env: &Env, admin: &Address) {
    let pinned: Address = env.storage().instance().get(&DataKey::PinnedAdmin).unwrap_optimized();
    require(env, *admin == pinned, ProtocolError::Unauthorized);
}

fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap_optimized()
}

mod test;
