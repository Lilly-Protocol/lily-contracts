#![no_std]

//! Global protocol configuration contract for Lily Protocol.

use lily_common::{bump_instance, require, require_valid_bps, ProtocolError};
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
    PendingAdmin,
    Treasury,
    FeeBps,
    Initialized,
}

#[contractimpl]
impl ProtocolContract {
    /// Initialize protocol-wide configuration once.
    pub fn initialize(env: Env, admin: Address, treasury: Address, fee_bps: u32) {
        require(
            &env,
            !env.storage().instance().has(&DataKey::Initialized),
            ProtocolError::AlreadyInitialized,
        );
        require_valid_bps(&env, fee_bps);

        admin.require_auth();

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
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Fetch the current protocol configuration.
    pub fn get_config(env: Env) -> ProtocolConfig {
        ensure_initialized(&env);
        bump_instance(&env);

        ProtocolConfig {
            admin: get_admin(&env),
            treasury: env.storage().instance().get(&DataKey::Treasury).unwrap_optimized(),
            fee_bps: env.storage().instance().get(&DataKey::FeeBps).unwrap_optimized(),
        }
    }

    /// Return the pending admin address if a transfer is in progress.
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Update the protocol fee in basis points.
    pub fn set_fee_bps(env: Env, fee_bps: u32) {
        ensure_initialized(&env);
        require_valid_bps(&env, fee_bps);

        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        bump_instance(&env);
        env.events().publish((symbol_short!("fee"), admin), fee_bps);
    }

    /// Update the treasury address used for fee collection.
    pub fn set_treasury(env: Env, treasury: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().instance().set(&DataKey::Treasury, &treasury);
        bump_instance(&env);
        env.events().publish((symbol_short!("treasury"), admin), treasury);
    }

    /// Propose a new protocol admin (step 1 of two-step transfer).
    pub fn transfer_admin(env: Env, new_admin: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().instance().set(&DataKey::PendingAdmin, &new_admin);
        bump_instance(&env);
        env.events().publish((symbol_short!("propose"), admin), new_admin);
    }

    /// Accept protocol admin authority as the proposed pending admin (step 2 of two-step transfer).
    pub fn accept_admin(env: Env) {
        ensure_initialized(&env);

        require(
            &env,
            env.storage().instance().has(&DataKey::PendingAdmin),
            ProtocolError::MissingRecord,
        );

        let pending_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .unwrap_optimized();
        pending_admin.require_auth();

        let old_admin = get_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &pending_admin);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        bump_instance(&env);
        env.events().publish((symbol_short!("admin"), old_admin), pending_admin);
    }
}


fn ensure_initialized(env: &Env) {
    require(
        env,
        env.storage().instance().has(&DataKey::Initialized),
        ProtocolError::NotInitialized,
    );
}

fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap_optimized()
}

mod test;
