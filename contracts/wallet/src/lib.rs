#![no_std]

//! Agent wallet binding and policy contract.

use lily_common::{bump_instance, require, ProtocolError};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
    Symbol,
};

#[contract]
pub struct WalletContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletBinding {
    pub wallet: Address,
    pub settlement_asset: Symbol,
    pub spend_limit: i128,
    pub enabled: bool,
    pub revision: u64,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Initialized,
    Binding(Address),
    PinnedAdmin,
}

#[contractimpl]
impl WalletContract {
    /// Capture the intended initial admin at deploy time.
    ///
    /// `initialize` only accepts this exact address, so a front-runner cannot
    /// claim a fresh deployment with their own admin.
    pub fn __constructor(env: Env, initial_admin: Address) {
        env.storage().instance().set(&DataKey::PinnedAdmin, &initial_admin);
    }

    /// Initialize the wallet policy registry.
    ///
    /// The initial admin must match the address pinned by the constructor at
    /// deploy time, preventing initialization front-running.
    pub fn initialize(env: Env, admin: Address) {
        require(
            &env,
            !env.storage().instance().has(&DataKey::Initialized),
            ProtocolError::AlreadyInitialized,
        );
        require_auth_or_error(&admin, &env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
        bump_instance(&env);
        env.events().publish((symbol_short!("init"),), admin);
    }

    /// Bind an agent to a settlement wallet and policy envelope.
    pub fn bind_wallet(
        env: Env,
        agent: Address,
        wallet: Address,
        settlement_asset: Symbol,
        spend_limit: i128,
    ) {
        ensure_initialized(&env);
        require(&env, spend_limit > 0, ProtocolError::InvalidInput);

        require_auth_or_error(&agent, &env);
        require_auth_or_error(&wallet, &env);

        let key = DataKey::Binding(agent.clone());
        if let Some(existing) = env.storage().persistent().get::<_, WalletBinding>(&key) {
            require(&env, !existing.enabled, ProtocolError::WalletAlreadyBound);
        }

        let binding =
            WalletBinding { wallet, settlement_asset, spend_limit, enabled: true, revision: 0 };

        env.storage().persistent().set(&key, &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("bind"), agent), binding);
    }

    /// Update the spend limit for an enabled binding.
    pub fn update_spend_limit(env: Env, agent: Address, spend_limit: i128) {
        ensure_initialized(&env);
        require(&env, spend_limit > 0, ProtocolError::InvalidInput);

        require_auth_or_error(&agent, &env);

        let mut binding = get_binding_internal(&env, &agent);
        require_enabled(&env, binding.enabled);
        binding.spend_limit = spend_limit;
        binding.revision += 1;

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("limit"), agent), binding);
    }

    /// Enable or disable a wallet binding.
    pub fn set_enabled(env: Env, agent: Address, enabled: bool) {
        ensure_initialized(&env);
        require_auth_or_error(&agent, &env);

        let mut binding = get_binding_internal(&env, &agent);
        binding.enabled = enabled;
        binding.revision += 1;

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("state"), agent), binding);
    }

    /// Admin emergency deactivation of a wallet binding.
    pub fn admin_deactivate(env: Env, agent: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        admin.require_auth();

        let mut binding = get_binding_internal(&env, &agent);
        binding.enabled = false;
        binding.revision += 1;

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("adm_deact"), agent), binding);
    }

    /// Read the current binding for an agent.
    pub fn get_binding(env: Env, agent: Address) -> WalletBinding {
        ensure_initialized(&env);
        bump_instance(&env);
        get_binding_internal(&env, &agent)
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

fn get_binding_internal(env: &Env, agent: &Address) -> WalletBinding {
    env.storage()
        .persistent()
        .get(&DataKey::Binding(agent.clone()))
        .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, ProtocolError::MissingRecord))
}

mod test;

