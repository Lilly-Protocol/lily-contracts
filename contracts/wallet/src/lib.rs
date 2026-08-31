#![no_std]

//! Agent wallet binding and policy contract.

use lily_common::{bump_instance, require, ProtocolError};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

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
}

#[contractimpl]
impl WalletContract {
    /// Initialize the wallet policy registry.
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        require(
            &env,
            !env.storage().instance().has(&DataKey::Initialized),
            ProtocolError::AlreadyInitialized,
        );
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

        agent.require_auth();
        wallet.require_auth();

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

        agent.require_auth();

        let mut binding = get_binding_internal(&env, &agent);
        require(&env, binding.enabled, ProtocolError::InvalidInput);
        binding.spend_limit = spend_limit;
        binding.revision += 1;

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("limit"), agent), binding);
    }

    /// Enable or disable a wallet binding.
    pub fn set_enabled(env: Env, agent: Address, enabled: bool) {
        ensure_initialized(&env);
        agent.require_auth();

        let mut binding = get_binding_internal(&env, &agent);
        binding.enabled = enabled;
        binding.revision += 1;

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("state"), agent), binding);
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

fn get_binding_internal(env: &Env, agent: &Address) -> WalletBinding {
    env.storage()
        .persistent()
        .get(&DataKey::Binding(agent.clone()))
        .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, ProtocolError::MissingRecord))
}

mod test;
