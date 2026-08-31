#![no_std]

//! Global protocol configuration contract for Lily Protocol.

use lily_common::{bump_instance, read_instance, require, require_valid_bps, ProtocolConfig, ProtocolError};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[contract]
pub struct ProtocolContract;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
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
            admin: read_instance(&env, DataKey::Admin),
            treasury: read_instance(&env, DataKey::Treasury),
            fee_bps: read_instance(&env, DataKey::FeeBps),
        }
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

    /// Transfer protocol admin authority.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        admin.require_auth();

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

fn get_admin(env: &Env) -> Address {
    read_instance(env, DataKey::Admin)
}

mod test;
