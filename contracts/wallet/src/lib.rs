#![no_std]

//! Agent wallet binding and policy contract.

use lily_common::{bump_instance, require, require_auth_or_error, ProtocolError, PROTOCOL_VERSION};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
    Symbol,
};

#[contract]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletConfig {
    pub admin: Address,
}

pub struct WalletContract;

/// Wallet contract schema version.
pub const SCHEMA_VERSION: u32 = 1;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletBinding {
    pub wallet: Address,
    pub settlement_asset: Symbol,
    pub spend_limit: i128,
    pub enabled: bool,
    /// Set by `admin_deactivate`; blocks agent-initiated re-enabling via
    /// `set_enabled` until an admin clears it via `admin_reactivate` or the
    /// binding is replaced by `rebind_wallet`.
    pub admin_locked: bool,
    pub revision: u64,
}

/// Storage keys for wallet policy configuration and agent binding records.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// Stores the wallet policy registry admin `Address`. Durability: Instance.
    Admin,
    /// Marker boolean indicating if the contract has been initialized. Durability: Instance.
    Initialized,
    /// Stores the schema version (`u32`). Durability: Instance.
    SchemaVersion,
    /// Maps an agent `Address` to their `WalletBinding` configuration. Durability: Persistent.
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
        require_initial_admin(&env, &admin);
        require_auth_or_error(&admin, &env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::SchemaVersion, &SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Initialized, &true);
        bump_instance(&env);
        env.events().publish((symbol_short!("init"), admin.clone()), WalletConfig { admin });
    }

    /// Return the schema version.
    pub fn schema_version(env: Env) -> u32 {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage()
            .instance()
            .get(&DataKey::SchemaVersion)
            .unwrap_or(SCHEMA_VERSION)
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

    /// Bind an agent to a settlement wallet and policy envelope.
    ///
    /// Fails if the agent already has any binding (enabled or disabled).
    /// Use `rebind_wallet` to explicitly replace an existing binding.
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
        require(&env, !env.storage().persistent().has(&key), ProtocolError::WalletAlreadyBound);

        let binding = WalletBinding {
            wallet,
            settlement_asset,
            spend_limit,
            enabled: true,
            admin_locked: false,
            revision: 0,
        };

        env.storage().persistent().set(&key, &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("bind"), agent), binding);
    }

    /// Explicitly replace an existing wallet binding.
    ///
    /// Requires the agent to already have a binding. The new binding starts at
    /// revision 0 and is enabled. This removes the silent overwrite behavior
    /// that `bind_wallet` previously performed on disabled bindings.
    pub fn rebind_wallet(
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
        require(&env, env.storage().persistent().has(&key), ProtocolError::MissingRecord);

        let binding = WalletBinding {
            wallet,
            settlement_asset,
            spend_limit,
            enabled: true,
            admin_locked: false,
            revision: 0,
        };

        env.storage().persistent().set(&key, &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("rebind"), agent), binding);
    }

    /// Update the spend limit for an enabled binding.
    pub fn update_spend_limit(env: Env, agent: Address, spend_limit: i128) {
        ensure_initialized(&env);
        require(&env, spend_limit > 0, ProtocolError::InvalidInput);

        require_auth_or_error(&agent, &env);

        let mut binding = get_binding_internal(&env, &agent);
        require_enabled(&env, binding.enabled);
        binding.spend_limit = spend_limit;
        binding.revision = checked_inc(&env, binding.revision);

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("limit"), agent), binding);
    }

    /// Enable or disable a wallet binding.
    ///
    /// Re-enabling (`enabled: true`) is rejected with a typed
    /// `ProtocolError::Unauthorized` if the binding is currently
    /// admin-locked (see `admin_deactivate`); only `admin_reactivate` or a
    /// fresh `rebind_wallet` can clear that lock. Disabling (`enabled:
    /// false`) is always allowed regardless of lock state.
    pub fn set_enabled(env: Env, agent: Address, enabled: bool) {
        ensure_initialized(&env);
        require_auth_or_error(&agent, &env);

        let mut binding = get_binding_internal(&env, &agent);
        if enabled {
            require(&env, !binding.admin_locked, ProtocolError::Unauthorized);
        }
        binding.enabled = enabled;
        binding.revision = checked_inc(&env, binding.revision);

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("state"), agent), binding);
    }

    /// Admin emergency deactivation of a wallet binding.
    ///
    /// Sets the admin lock so the agent cannot immediately undo this via
    /// `set_enabled`; only `admin_reactivate` (or a fresh `rebind_wallet`)
    /// can clear it.
    pub fn admin_deactivate(env: Env, agent: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        admin.require_auth();

        let mut binding = get_binding_internal(&env, &agent);
        if !binding.enabled {
            return;
        }

        binding.enabled = false;
        binding.admin_locked = true;
        binding.revision = checked_inc(&env, binding.revision);

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("adm_deact"), agent), binding);
    }

    /// Admin-only restoration of a binding disabled by `admin_deactivate`.
    ///
    /// Clears the admin lock and re-enables the binding in one step, so the
    /// agent regains normal `set_enabled` control afterward.
    pub fn admin_reactivate(env: Env, agent: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        admin.require_auth();

        let mut binding = get_binding_internal(&env, &agent);
        binding.enabled = true;
        binding.admin_locked = false;
        binding.revision = checked_inc(&env, binding.revision);

        env.storage().persistent().set(&DataKey::Binding(agent.clone()), &binding);
        bump_instance(&env);
        env.events().publish((symbol_short!("adm_react"), agent), binding);
    }

    /// Read the current binding for an agent.
    #[must_use]
    pub fn get_binding(env: Env, agent: Address) -> WalletBinding {
        ensure_initialized(&env);
        bump_instance(&env);
        get_binding_internal(&env, &agent)
    }

    /// Read the current binding for an agent if one exists, returning `None` otherwise.
    pub fn get_binding_opt(env: Env, agent: Address) -> Option<WalletBinding> {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage().persistent().get(&DataKey::Binding(agent))
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

fn get_binding_internal(env: &Env, agent: &Address) -> WalletBinding {
    env.storage()
        .persistent()
        .get(&DataKey::Binding(agent.clone()))
        .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, ProtocolError::MissingRecord))
}

// `checked_inc` and `require_enabled` are referenced by other contracts in
// this workspace too (see #311), but this crate wouldn't compile without a
// definition, so they're kept local here rather than blocking this fix on
// that separate, already-in-progress bounty.
fn require_enabled(env: &Env, enabled: bool) {
    require(env, enabled, ProtocolError::InvalidInput);
}

fn checked_inc(env: &Env, value: u64) -> u64 {
    value
        .checked_add(1)
        .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, ProtocolError::InvalidInput))
}

mod test;
