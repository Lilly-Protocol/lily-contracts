#![no_std]

//! Agent identity registry for Lily Protocol.

use lily_common::{
    bump_instance, checked_inc, require, require_auth_or_error, require_non_empty, ProtocolError,
    PROTOCOL_VERSION,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
    String, Symbol,
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

/// Storage keys for identity registry configuration and agent profile records.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityConfig {
    pub admin: Address,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// Stores the identity registry admin `Address`. Durability: Instance.
    Admin,
    /// Marker boolean indicating if the contract has been initialized. Durability: Instance.
    Initialized,
    /// Maps an agent `Address` to their `AgentProfile`. Durability: Persistent.
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
        require_initial_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
        bump_instance(&env);
        env.events().publish((symbol_short!("init"), admin.clone()), IdentityConfig { admin });
    }

    /// Return whether the contract has been initialized.
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Return the shared protocol interface version.
    #[must_use]
    pub fn version(_env: Env) -> u32 {
        PROTOCOL_VERSION
    }

    /// Return the current registry configuration.
    #[must_use]
    pub fn get_config(env: Env) -> IdentityConfig {
        IdentityConfig { admin: get_admin(&env) }
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
    ///
    /// Emits `metadata_updated` when the metadata URI changes and
    /// `controller_rotated` when the controller changes.
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

        let metadata_changed = profile.metadata_uri != metadata_uri;
        let controller_changed =
            new_controller.as_ref().is_some_and(|next| next != &profile.controller);

        if !metadata_changed && !controller_changed {
            bump_instance(&env);
            return;
        }

        profile.metadata_uri = metadata_uri;
        if let Some(next_controller) = new_controller {
            profile.controller = next_controller;
        }
        profile.revision = checked_inc(&env, profile.revision);

        env.storage().persistent().set(&DataKey::Profile(agent.clone()), &profile);
        bump_instance(&env);

        if metadata_changed {
            env.events().publish(
                (Symbol::new(&env, "metadata_updated"), agent.clone()),
                profile.metadata_uri.clone(),
            );
        }
        if controller_changed {
            env.events().publish(
                (Symbol::new(&env, "controller_rotated"), agent.clone()),
                profile.controller.clone(),
            );
        }
    }

    /// Disable an agent profile through admin action.
    ///
    /// Repeated calls on an already inactive profile are a no-op and do not
    /// increment the revision or emit an event.
    pub fn deactivate(env: Env, agent: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

        let mut profile = get_profile_internal(&env, &agent);
        if !profile.active {
            bump_instance(&env);
            return;
        }
        profile.active = false;
        profile.revision = checked_inc(&env, profile.revision);

        env.storage().persistent().set(&DataKey::Profile(agent.clone()), &profile);
        bump_instance(&env);
        env.events().publish((symbol_short!("deact"), agent), profile);
    }

    /// Re-enable a previously deactivated agent profile through admin action.
    ///
    /// Repeated calls on an already active profile are a no-op and do not
    /// increment the revision or emit an event.
    pub fn reactivate(env: Env, agent: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        require_auth_or_error(&admin, &env);

        let mut profile = get_profile_internal(&env, &agent);
        if profile.active {
            bump_instance(&env);
            return;
        }
        profile.active = true;
        profile.revision = checked_inc(&env, profile.revision);

        env.storage().persistent().set(&DataKey::Profile(agent.clone()), &profile);
        bump_instance(&env);
        env.events().publish((symbol_short!("react"), agent), profile);
    }

    /// Fetch a registered profile.
    #[must_use]
    pub fn get_profile(env: Env, agent: Address) -> AgentProfile {
        ensure_initialized(&env);
        bump_instance(&env);
        get_profile_internal(&env, &agent)
    }

    /// Fetch a registered profile if it exists, returning `None` for missing records.
    pub fn get_profile_opt(env: Env, agent: Address) -> Option<AgentProfile> {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage().persistent().get(&DataKey::Profile(agent))
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
