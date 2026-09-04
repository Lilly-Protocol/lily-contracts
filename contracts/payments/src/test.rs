#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use lily_common::PaymentStatus;
use lily_test_support::{soroban_string, test_address, test_env};
use soroban_sdk::symbol_short;
use soroban_sdk::testutils::Ledger;
use soroban_sdk::unwrap::UnwrapOptimized;
use wallet::{WalletContract, WalletContractClient};

use super::{
    PaymentIntent, PaymentsContract, PaymentsContractClient, MAX_PAYMENT_AMOUNT, SCHEMA_VERSION,
};

fn bootstrap() -> (
    soroban_sdk::Env,
    soroban_sdk::Address,
    soroban_sdk::Address,
    soroban_sdk::Address,
    PaymentsContractClient<'static>,
) {
    let env = test_env();
    let treasury = test_address(&env);
    let admin = test_address(&env);

    let wallet_id = env.register(WalletContract, (admin.clone(),));
    WalletContractClient::new(&env, &wallet_id).initialize(&admin);

    let contract_id = env.register(PaymentsContract, (admin.clone(),));
    let client = PaymentsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury, &50_u32, &wallet_id);
    (env, admin, treasury, wallet_id, client)
}

fn bind_payer(
    env: &soroban_sdk::Env,
    wallet_id: &soroban_sdk::Address,
    payer: &soroban_sdk::Address,
    spend_limit: &i128,
) {
    let wallet_addr = test_address(env);
    let client = WalletContractClient::new(env, wallet_id);
    client.bind_wallet(payer, &wallet_addr, &symbol_short!("USDC"), spend_limit);
}

#[test]
fn returns_protocol_version() {
    let (_env, _admin, _treasury, _wallet_id, client) = bootstrap();

    assert_eq!(client.schema_version(), SCHEMA_VERSION);
}

#[test]
fn creates_and_settles_payment_intents() {
    let (env, admin, treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &10_000_i128);

    let id = client.create_intent(
        &payer,
        &payee,
        &5_000_i128,
        &soroban_string(&env, "settle agent service fee"),
    );

    assert_eq!(id, 1);
    assert_eq!(client.get_next_intent_id(), 2);

    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.treasury, treasury);
    assert_eq!(config.fee_bps, 50);

    let intent = client.get_intent(&id);
    assert_eq!(
        intent,
        PaymentIntent {
            id: 1,
            payer_agent: payer.clone(),
            payee_agent: payee.clone(),
            amount: 5_000,
            memo: soroban_string(&env, "settle agent service fee"),
            settlement_reference: soroban_string(&env, ""),
            status: PaymentStatus::Pending,
            created_at: env.ledger().get().timestamp,
        }
    );

    client.settle_intent(&admin, &id, &soroban_string(&env, "tx-0001"));
    let settled = client.get_intent(&id);
    assert_eq!(settled.status, PaymentStatus::Settled);
    assert_eq!(settled.settlement_reference, soroban_string(&env, "tx-0001"));
}

#[test]
fn created_at_uses_mocked_ledger_timestamp() {
    let (env, _admin, _treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &10_000_i128);

    let created_at: u64 = 1_750_000_000;
    env.ledger().set_timestamp(created_at);

    let id = client.create_intent(
        &payer,
        &payee,
        &5_000_i128,
        &soroban_string(&env, "timestamps come from the ledger"),
    );

    assert_eq!(client.get_intent(&id).created_at, created_at);
}

#[test]
fn payer_can_cancel_pending_intents() {
    let (env, _admin, _treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &10_000_i128);

    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "cancel me"));
    client.cancel_intent(&id);

    let cancelled = client.get_intent(&id);
    assert_eq!(cancelled.status, PaymentStatus::Cancelled);
}

#[test]
fn accepts_the_maximum_payment_amount() {
    let (env, _admin, _treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &MAX_PAYMENT_AMOUNT);

    let id = client.create_intent(
        &payer,
        &payee,
        &MAX_PAYMENT_AMOUNT,
        &soroban_string(&env, "maximum payment"),
    );

    assert_eq!(client.get_intent(&id).amount, MAX_PAYMENT_AMOUNT);
}

#[test]
fn lists_payer_intents_with_cursor_pagination() {
    let (env, _admin, _treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let other_payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &10_000_i128);
    bind_payer(&env, &wallet_id, &other_payer, &10_000_i128);

    let first_id = client.create_intent(&payer, &payee, &10_i128, &soroban_string(&env, "first"));
    let second_id = client.create_intent(&payer, &payee, &20_i128, &soroban_string(&env, "second"));
    let third_id = client.create_intent(&payer, &payee, &30_i128, &soroban_string(&env, "third"));
    client.create_intent(&other_payer, &payee, &40_i128, &soroban_string(&env, "other payer"));

    let first_page = client.list_intents(&payer, &0_u32, &2_u32);
    assert_eq!(first_page.len(), 2);
    assert_eq!(first_page.get(0).unwrap_optimized().id, first_id);
    assert_eq!(first_page.get(1).unwrap_optimized().id, second_id);

    let second_page = client.list_intents(&payer, &2_u32, &2_u32);
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page.get(0).unwrap_optimized().id, third_id);

    let exhausted_page = client.list_intents(&payer, &3_u32, &2_u32);
    assert!(exhausted_page.is_empty());
}

#[test]
#[should_panic]
fn rejects_zero_page_limit() {
    let (env, _admin, _treasury, _wallet_id, client) = bootstrap();
    let payer = test_address(&env);

    client.list_intents(&payer, &0_u32, &0_u32);
}

#[test]
#[should_panic]
fn rejects_payment_amount_above_the_maximum() {
    let (env, _admin, _treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &MAX_PAYMENT_AMOUNT);

    client.create_intent(
        &payer,
        &payee,
        &(MAX_PAYMENT_AMOUNT + 1),
        &soroban_string(&env, "too large"),
    );
}

#[test]
#[should_panic]
fn rejects_config_read_before_initialization() {
    let env = test_env();
    let contract_id = env.register(PaymentsContract, (test_address(&env),));
    let client = PaymentsContractClient::new(&env, &contract_id);
    client.get_config();
}

#[test]
#[should_panic]
fn rejects_settle_after_cancellation() {
    let (env, admin, _treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &10_000_i128);

    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "cancel me"));
    client.cancel_intent(&id);
    client.settle_intent(&admin, &id, &soroban_string(&env, "tx-0002"));
}

// Typed role error: ProtocolError::Unauthorized = 3.
#[test]
#[should_panic = "Error(Contract, #3)"]
fn settle_rejects_non_admin_caller_with_typed_unauthorized() {
    let (env, _admin, _treasury, wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);
    bind_payer(&env, &wallet_id, &payer, &10_000_i128);

    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "not yours"));
    // Payer tries to settle: signature would pass under mock_all_auths, but
    // the typed role check must fire first with ProtocolError::Unauthorized.
    client.settle_intent(&payer, &id, &soroban_string(&env, "tx-not-admin"));
}

#[test]
#[should_panic]
fn rejects_zero_amount_intent() {
    let (env, _admin, _treasury, _wallet_id, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);

    client.create_intent(&payer, &payee, &0_i128, &soroban_string(&env, "invalid zero amount"));
}

#[test]
#[should_panic]
fn rejects_get_intent_on_missing_record() {
    let (_env, _admin, _treasury, _wallet_id, client) = bootstrap();
    client.get_intent(&999_u64);
}
