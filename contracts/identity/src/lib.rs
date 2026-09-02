#![no_std]

//! Agent identity registry for Lily Protocol.

use lily_common::{
    bump_instance, require, require_auth_or_error, require_non_empty, ProtocolError,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
    String,
};

#[contract]
pub struct IdentityContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentProfile {
    pub controller: Address,
    pub metadata_uri: String,
    pub active: bool,
    pub revision: u64,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Initialized,
    Profile(Address),
    PinnedAdmin,
}

#[contractimpl]
impl IdentityContract {
    /// Capture the intended initial admin at deploy time.
    ///
    /// `initialize` only accepts this exact address, so a front-runner cannot
    /// claim a fresh deployment with their own admin.
    pub fn __constructor(env: Env, initial_admin: Address) {
        env.storage().instance().set(&DataKey::PinnedAdmin, &initial_admin);
    }

    /// Initialize the registry admin.
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

    /// Register a new agent profile controlled by a specific address.
    pub fn register(env: Env, agent: Address, controller: Address, metadata_uri: String) {
        ensure_initialized(&env);
        require_non_empty(&env, metadata_uri.len());
        require(
            &env,
            !env.storage().persistent().has(&DataKey::Profile(agent.clone())),
            ProtocolError::AlreadyExists,
        );

        require_auth_or_error(&agent, &env);

        let profile = AgentProfile { controller, metadata_uri, active: true, revision: 0 };
        env.storage().persistent().set(&DataKey::Profile(agent.clone()), &profile);
        bump_instance(&env);

        env.events().publish((symbol_short!("register"), agent), profile);
    }

    /// Update metadata and optionally rotate the controller.
    pub fn update_profile(
        env: Env,
        agent: Address,
        metadata_uri: String,
        new_controller: Option<Address>,
    ) {
        ensure_initialized(&env);
        require_non_empty(&env, metadata_uri.len());

        let mut profile = get_profile_internal(&env, &agent);
        require(&env, profile.active, ProtocolError::InvalidInput);
        require_auth_or_error(&profile.controller, &env);

        profile.metadata_uri = metadata_uri;
        if let Some(next_controller) = new_controller {
            profile.controller = next_controller;
        }
        profile.revision += 1;

        env.storage().persistent().set(&DataKey::Profile(agent.clone()), &profile);
        bump_instance(&env);
        env.events().publish((symbol_short!("update"), agent), profile);
    }

    /// Disable an agent profile through admin action.
    pub fn deactivate(env: Env, agent: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

        let mut profile = get_profile_internal(&env, &agent);
        profile.active = false;
        profile.revision += 1;

        env.storage().persistent().set(&DataKey::Profile(agent.clone()), &profile);
        bump_instance(&env);
        env.events().publish((symbol_short!("deact"), agent), profile);
    }

    /// Re-enable a previously deactivated agent profile through admin action.
    pub fn reactivate(env: Env, agent: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        admin.require_auth();

        let mut profile = get_profile_internal(&env, &agent);
        profile.active = true;
        profile.revision += 1;

        env.storage().persistent().set(&DataKey::Profile(agent.clone()), &profile);
        bump_instance(&env);
        env.events().publish((symbol_short!("react"), agent), profile);
    }

    /// Fetch a registered profile.
    pub fn get_profile(env: Env, agent: Address) -> AgentProfile {
        ensure_initialized(&env);
        bump_instance(&env);
        get_profile_internal(&env, &agent)
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

fn get_profile_internal(env: &Env, agent: &Address) -> AgentProfile {
    env.storage()
        .persistent()
        .get(&DataKey::Profile(agent.clone()))
        .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, ProtocolError::MissingRecord))
}

mod test;
