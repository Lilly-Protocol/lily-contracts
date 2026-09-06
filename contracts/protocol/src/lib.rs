#![no_std]

//! Global protocol configuration contract for Lily Protocol.

pub use lily_common::ProtocolConfig;
use lily_common::{
    bump_instance, read_instance, require, require_auth_or_error, require_valid_bps, ProtocolError,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
    TryFromVal, Val,
};

#[contract]
pub struct ProtocolContract;

/// Protocol contract schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Instance storage keys for protocol configuration and lifecycle state.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// Stores the active admin `Address`. Durability: Instance.
    Admin,
    PendingAdmin,
    Treasury,
    /// Stores the active protocol fee in basis points (`u32`). Durability: Instance.
    FeeBps,
    /// Marker boolean indicating if the contract has been initialized. Durability: Instance.
    Initialized,
    /// Stores the schema version (`u32`). Durability: Instance.
    SchemaVersion,
    PinnedAdmin,
    SchemaVersion,
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

    /// Return the protocol version.
    #[must_use]
    pub fn version(_env: Env) -> u32 {
        lily_common::PROTOCOL_VERSION
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
        require_auth_or_error(&admin, &env);
        require_initial_admin(&env, &admin);
        require_valid_bps(&env, fee_bps);

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        env.storage().instance().set(&DataKey::SchemaVersion, &SCHEMA_VERSION);
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

    /// Return the contract schema version.
    #[must_use]
    pub fn schema_version(env: Env) -> u32 {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage().instance().get(&DataKey::SchemaVersion).unwrap_or(SCHEMA_VERSION)
    }

    /// Fetch the current protocol configuration.
    #[must_use]
    pub fn get_config(env: Env) -> ProtocolConfig {
        ensure_initialized(&env);
        bump_instance(&env);
        ProtocolConfig {
            admin: get_admin_internal(&env),
            treasury: read_instance(&env, DataKey::Treasury),
            fee_bps: read_instance(&env, DataKey::FeeBps),
        }
    }

    /// Return the pending admin address if a transfer is in progress.
    #[must_use]
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Update the protocol fee in basis points.
    pub fn set_fee_bps(env: Env, fee_bps: u32) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

        require_valid_bps(&env, fee_bps);

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

    /// Propose a new protocol admin (step 1 of two-step transfer).
    pub fn transfer_admin(env: Env, new_admin: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

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

        let pending_admin: Address =
            env.storage().instance().get(&DataKey::PendingAdmin).unwrap_optimized();
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

fn require_initial_admin(env: &Env, admin: &Address) {
    let pinned: Address = env.storage().instance().get(&DataKey::PinnedAdmin).unwrap_optimized();
    require(env, *admin == pinned, ProtocolError::Unauthorized);
}

fn get_admin_internal(env: &Env) -> Address {
    read_instance(env, DataKey::Admin)
}

fn get_admin(env: &Env) -> Address {
    get_admin_internal(env)
}

mod test;
