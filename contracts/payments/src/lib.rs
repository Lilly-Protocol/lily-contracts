#![no_std]

//! Payment intent and settlement primitives for Lily Protocol.

use lily_common::{
    bump_instance, require, require_non_empty, require_valid_bps, PaymentStatus, ProtocolError,
    MAX_BPS,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env,
    String,
};

#[contract]
pub struct PaymentsContract;

/// Largest payment amount that keeps future basis-point multiplication within i128.
pub const MAX_PAYMENT_AMOUNT: i128 = i128::MAX / (MAX_BPS as i128);

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentsConfig {
    pub admin: Address,
    pub treasury: Address,
    pub fee_bps: u32,
    pub next_intent_id: u64,
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
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Treasury,
    FeeBps,
    NextIntentId,
    Initialized,
    Intent(u64),
}

#[contractimpl]
impl PaymentsContract {
    /// Initialize settlement configuration once.
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
        env.storage().instance().set(&DataKey::NextIntentId, &1_u64);
        env.storage().instance().set(&DataKey::Initialized, &true);
        bump_instance(&env);

        env.events().publish((symbol_short!("init"),), treasury);
    }

    /// Return the active payments configuration.
    pub fn get_config(env: Env) -> PaymentsConfig {
        ensure_initialized(&env);
        bump_instance(&env);
        PaymentsConfig {
            admin: get_admin(&env),
            treasury: env.storage().instance().get(&DataKey::Treasury).unwrap_optimized(),
            fee_bps: env.storage().instance().get(&DataKey::FeeBps).unwrap_optimized(),
            next_intent_id: env.storage().instance().get(&DataKey::NextIntentId).unwrap_optimized(),
        }
    }

    /// Create a payment intent that can be settled asynchronously.
    pub fn create_intent(
        env: Env,
        payer_agent: Address,
        payee_agent: Address,
        amount: i128,
        memo: String,
    ) -> u64 {
        ensure_initialized(&env);
        require(
            &env,
            amount > 0 && amount <= MAX_PAYMENT_AMOUNT,
            ProtocolError::InvalidInput,
        );
        require_non_empty(&env, memo.len());

        payer_agent.require_auth();

        let id: u64 = env.storage().instance().get(&DataKey::NextIntentId).unwrap_optimized();

        let intent = PaymentIntent {
            id,
            payer_agent,
            payee_agent,
            amount,
            memo,
            settlement_reference: String::from_str(&env, ""),
            status: PaymentStatus::Pending,
        };

        env.storage().persistent().set(&DataKey::Intent(id), &intent);
        env.storage().instance().set(&DataKey::NextIntentId, &(id + 1));
        bump_instance(&env);
        env.events().publish((symbol_short!("create"), id), intent);
        id
    }

    /// Mark a payment intent as settled.
    pub fn settle_intent(env: Env, intent_id: u64, settlement_reference: String) {
        ensure_initialized(&env);
        require_non_empty(&env, settlement_reference.len());

        let admin = get_admin(&env);
        admin.require_auth();

        let mut intent = get_intent_internal(&env, intent_id);
        require(
            &env,
            intent.status == PaymentStatus::Pending,
            ProtocolError::PaymentAlreadyFinalized,
        );
        intent.status = PaymentStatus::Settled;
        intent.settlement_reference = settlement_reference;

        env.storage().persistent().set(&DataKey::Intent(intent_id), &intent);
        bump_instance(&env);
        env.events().publish((symbol_short!("settle"), intent_id), intent);
    }

    /// Cancel a payment intent before settlement.
    pub fn cancel_intent(env: Env, intent_id: u64) {
        ensure_initialized(&env);

        let mut intent = get_intent_internal(&env, intent_id);
        intent.payer_agent.require_auth();
        require(
            &env,
            intent.status == PaymentStatus::Pending,
            ProtocolError::PaymentAlreadyFinalized,
        );

        intent.status = PaymentStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Intent(intent_id), &intent);
        bump_instance(&env);
        env.events().publish((symbol_short!("cancel"), intent_id), intent);
    }

    /// Read an individual payment intent.
    pub fn get_intent(env: Env, intent_id: u64) -> PaymentIntent {
        ensure_initialized(&env);
        bump_instance(&env);
        get_intent_internal(&env, intent_id)
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

fn get_intent_internal(env: &Env, intent_id: u64) -> PaymentIntent {
    env.storage()
        .persistent()
        .get(&DataKey::Intent(intent_id))
        .unwrap_or_else(|| soroban_sdk::panic_with_error!(env, ProtocolError::MissingRecord))
}

mod test;
