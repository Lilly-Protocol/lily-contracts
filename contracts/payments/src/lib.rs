#![no_std]

//! Payment intent and settlement primitives for Lily Protocol.

use lily_common::{
    bump_instance, checked_inc, read_instance, require, require_auth_or_error, require_caller,
    require_non_empty, require_valid_bps, NonReentrantGuard, PaymentStatus, ProtocolError, MAX_BPS,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
    String, Vec,
};
use wallet::WalletContractClient;

#[contract]
pub struct PaymentsContract;

/// Largest payment amount that keeps future basis-point multiplication within i128.
pub const MAX_PAYMENT_AMOUNT: i128 = i128::MAX / (MAX_BPS as i128);

/// Maximum number of payment intents returned by one paginated query.
pub const MAX_INTENTS_PAGE_SIZE: u32 = 100;

/// Payments contract schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Snapshot of the payments configuration used for the init event and get_config.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentsConfig {
    pub admin: Address,
    pub treasury: Address,
    pub fee_bps: u32,
    pub next_intent_id: u64,
    pub wallet: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentIntent {
    pub id: u64,
    pub payer_agent: Address,
    pub payee_agent: Address,
    pub amount: i128,
    pub memo: String,
    pub settlement_reference: String,
    pub status: PaymentStatus,
    /// Ledger timestamp (soroban time units, 30-second ledgers) captured at
    /// intent creation. Used for audit ordering and dispute windows.
    pub created_at: u64,
}

/// Storage keys for settlement configuration and payment intent records.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// Stores the settlement admin `Address`. Durability: Instance.
    Admin,
    /// Stores the protocol fee collector treasury `Address`. Durability: Instance.
    Treasury,
    /// Stores the protocol fee in basis points (`u32`). Durability: Instance.
    FeeBps,
    /// Stores the auto-incrementing `u64` identifier for next payment intent. Durability: Instance.
    NextIntentId,
    Wallet,
    Initialized,
    /// Stores the schema version (`u32`). Durability: Instance.
    SchemaVersion,
    /// Maps an intent ID (`u64`) to its `PaymentIntent` record. Durability: Persistent.
    Intent(u64),
    PinnedAdmin,
    PayerIntents(Address),
    PendingAdmin,
}

fn payment_status_symbol(status: PaymentStatus) -> soroban_sdk::Symbol {
    match status {
        PaymentStatus::Pending => symbol_short!("pending"),
        PaymentStatus::Settled => symbol_short!("settled"),
        PaymentStatus::Cancelled => symbol_short!("cancelled"),
    }
}

#[contractimpl]
impl PaymentsContract {
    /// Capture the intended initial admin at deploy time.
    ///
    /// `initialize` only accepts this exact address, so a front-runner cannot
    /// claim a fresh deployment with their own admin.
    pub fn __constructor(env: Env, initial_admin: Address) {
        env.storage().instance().set(&DataKey::PinnedAdmin, &initial_admin);
    }

    /// Initialize settlement configuration once.
    ///
    /// The initial admin must match the address pinned by the constructor at
    /// deploy time, preventing initialization front-running.
    pub fn initialize(env: Env, admin: Address, treasury: Address, fee_bps: u32, wallet: Address) {
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
        env.storage().instance().set(&DataKey::NextIntentId, &1_u64);
        env.storage().instance().set(&DataKey::Wallet, &wallet);
        env.storage().instance().set(&DataKey::Initialized, &true);
        bump_instance(&env);

        let config = PaymentsConfig {
            admin: admin.clone(),
            treasury: treasury.clone(),
            fee_bps,
            next_intent_id: 1,
            wallet,
        };
        env.events().publish((symbol_short!("init"), admin), config);
    }

    /// Update the bound wallet contract address.
    pub fn set_wallet(env: Env, wallet: Address) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().instance().set(&DataKey::Wallet, &wallet);
        bump_instance(&env);

        env.events().publish((symbol_short!("wallet"), admin), wallet);
    }

    /// Return the bound wallet contract address.
    pub fn get_wallet(env: Env) -> Address {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage()
            .instance()
            .get(&DataKey::Wallet)
            .unwrap_or_else(|| {
                soroban_sdk::panic_with_error!(&env, ProtocolError::MissingRecord)
            })
    }

    /// Return whether the contract has been initialized.
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Return the contract schema version.
    pub fn schema_version(env: Env) -> u32 {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage().instance().get(&DataKey::SchemaVersion).unwrap_or(SCHEMA_VERSION)
    }

    /// Return the active payments configuration.
    #[must_use]
    pub fn get_config(env: Env) -> PaymentsConfig {
        ensure_initialized(&env);
        bump_instance(&env);
        PaymentsConfig {
            admin: read_instance(&env, DataKey::Admin),
            treasury: read_instance(&env, DataKey::Treasury),
            fee_bps: read_instance(&env, DataKey::FeeBps),
            next_intent_id: read_instance(&env, DataKey::NextIntentId),
            wallet: read_instance(&env, DataKey::Wallet),
        }
    }

    /// Return the next intent id counter.
    pub fn get_next_intent_id(env: Env) -> u64 {
        ensure_initialized(&env);
        bump_instance(&env);
        read_instance(&env, DataKey::NextIntentId)
    }

    /// Create a payment intent that can be settled asynchronously.
    #[must_use]
    pub fn create_intent(
        env: Env,
        payer_agent: Address,
        payee_agent: Address,
        amount: i128,
        memo: String,
    ) -> u64 {
        ensure_initialized(&env);
        require(&env, amount > 0 && amount <= MAX_PAYMENT_AMOUNT, ProtocolError::InvalidInput);
        require_non_empty(&env, memo.len());
        require(&env, payer_agent != payee_agent, ProtocolError::InvalidInput);

        payer_agent.require_auth();

        let wallet: Address =
            env.storage().instance().get(&DataKey::Wallet).unwrap_or_else(|| {
                soroban_sdk::panic_with_error!(&env, ProtocolError::MissingRecord)
            });
        let wallet_client = WalletContractClient::new(&env, &wallet);
        let binding = wallet_client
            .get_binding_opt(&payer_agent)
            .unwrap_or_else(|| soroban_sdk::panic_with_error!(&env, ProtocolError::WalletNotBound));
        require(&env, binding.enabled, ProtocolError::WalletDisabled);
        require(&env, amount <= binding.spend_limit, ProtocolError::SpendLimitExceeded);

        let id: u64 = env.storage().instance().get(&DataKey::NextIntentId).unwrap_optimized();

        let intent = PaymentIntent {
            id,
            payer_agent,
            payee_agent,
            amount,
            memo,
            settlement_reference: String::from_str(&env, ""),
            status: PaymentStatus::Pending,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Intent(id), &intent);
        let payer_index_key = DataKey::PayerIntents(intent.payer_agent.clone());
        let mut payer_intent_ids: Vec<u64> =
            env.storage().persistent().get(&payer_index_key).unwrap_or_else(|| Vec::new(&env));
        payer_intent_ids.push_back(id);
        env.storage().persistent().set(&payer_index_key, &payer_intent_ids);
        env.storage().instance().set(&DataKey::NextIntentId, &checked_inc(&env, id));
        bump_instance(&env);
        env.events().publish((symbol_short!("create"), id), intent);
        id
    }

    /// Mark a payment intent as settled.
    ///
    /// `caller` is the principal authorized to settle. The typed role check
    /// raises `ProtocolError::Unauthorized` when `caller` is not the stored
    /// admin; a missing/invalid signature then surfaces as the host `Auth`
    /// error from `require_auth` (see `CONTRIBUTING.md` for the mapping).
    pub fn settle_intent(env: Env, caller: Address, intent_id: u64, settlement_reference: String) {
        ensure_initialized(&env);
        let admin = get_admin(&env);
        require_caller(&env, &caller, &admin);
        require_auth_or_error(&caller, &env);

        // Guard the status transition against reentrant settlement.
        let _guard = NonReentrantGuard::acquire(&env, symbol_short!("settle"));

        require_non_empty(&env, settlement_reference.len());

        let mut intent = get_intent_internal(&env, intent_id);
        require(
            &env,
            intent.status == PaymentStatus::Pending,
            ProtocolError::PaymentAlreadyFinalized,
        );
        let prior_status = intent.status;
        intent.status = PaymentStatus::Settled;
        intent.settlement_reference = settlement_reference;

        env.storage().persistent().set(&DataKey::Intent(intent_id), &intent);
        bump_instance(&env);
        env.events().publish(
            (symbol_short!("settle"), intent_id, payment_status_symbol(prior_status)),
            intent,
        );
    }

    /// Cancel a payment intent before settlement.
    pub fn cancel_intent(env: Env, intent_id: u64) {
        ensure_initialized(&env);

        let mut intent = get_intent_internal(&env, intent_id);
        intent.payer_agent.require_auth();
        // Guard the status transition against reentrant cancellation.
        let _guard = NonReentrantGuard::acquire(&env, symbol_short!("cancel"));
        require(
            &env,
            intent.status == PaymentStatus::Pending,
            ProtocolError::PaymentAlreadyFinalized,
        );

        let prior_status = intent.status;
        intent.status = PaymentStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Intent(intent_id), &intent);
        bump_instance(&env);
        env.events().publish(
            (symbol_short!("cancel"), intent_id, payment_status_symbol(prior_status)),
            intent,
        );
    }

    /// Update the fee charged on payment intents, in basis points.
    pub fn set_fee_bps(env: Env, fee_bps: u32) {
        ensure_initialized(&env);
        require_valid_bps(&env, fee_bps);

        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        bump_instance(&env);
        env.events().publish((symbol_short!("fee"), admin), fee_bps);
    }

    /// Update the treasury address used to collect fees.
    pub fn set_treasury(env: Env, treasury: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().instance().set(&DataKey::Treasury, &treasury);
        bump_instance(&env);
        env.events().publish((symbol_short!("treasury"), admin), treasury);
    }

    /// Propose a new payments admin (step 1 of two-step transfer).
    pub fn transfer_admin(env: Env, new_admin: Address) {
        ensure_initialized(&env);

        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().instance().set(&DataKey::PendingAdmin, &new_admin);
        bump_instance(&env);
        env.events().publish((symbol_short!("propose"), admin), new_admin);
    }

    /// Accept payments admin authority as the proposed pending admin (step 2 of two-step transfer).
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

    /// Read the currently proposed pending admin, if any.
    #[must_use]
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Read an individual payment intent.
    #[must_use]
    pub fn get_intent(env: Env, intent_id: u64) -> PaymentIntent {
        ensure_initialized(&env);
        bump_instance(&env);
        get_intent_internal(&env, intent_id)
    }

    /// Read an individual payment intent if it exists, returning `None` otherwise.
    pub fn get_intent_opt(env: Env, intent_id: u64) -> Option<PaymentIntent> {
        ensure_initialized(&env);
        bump_instance(&env);
        env.storage().persistent().get(&DataKey::Intent(intent_id))
    }

    /// List a payer's payment intents with cursor pagination.
    pub fn list_intents(env: Env, payer: Address, cursor: u32, limit: u32) -> Vec<PaymentIntent> {
        ensure_initialized(&env);
        require(&env, limit > 0, ProtocolError::InvalidInput);
        bump_instance(&env);

        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PayerIntents(payer))
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        let start = cursor as u64;
        let end = start + limit as u64;
        let mut i = 0_u64;
        for id in ids.iter() {
            if i >= start && i < end {
                result.push_back(get_intent_internal(&env, id));
            }
            i += 1;
        }
        result
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
    read_instance(env, DataKey::Admin)
}

fn get_intent_internal(env: &Env, intent_id: u64) -> PaymentIntent {
    env.storage()
        .persistent()
        .get(&DataKey::Intent(intent_id))
        .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, ProtocolError::MissingRecord))
}

mod test;
