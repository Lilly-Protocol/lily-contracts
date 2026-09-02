#![cfg(test)]

use lily_common::PaymentStatus;
use lily_test_support::{soroban_string, test_address, test_env};
use soroban_sdk::unwrap::UnwrapOptimized;

use super::{PaymentIntent, PaymentsContract, PaymentsContractClient};

#[test]
fn creates_and_settles_payment_intents() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    let id = client.create_intent(
        &payer,
        &payee,
        &5_000_i128,
        &soroban_string(&env, "settle agent service fee"),
    );

    assert_eq!(id, 1);
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
        }
    );

    client.settle_intent(&id, &soroban_string(&env, "tx-0001"));
    let settled = client.get_intent(&id);
    assert_eq!(settled.status, PaymentStatus::Settled);
    assert_eq!(settled.settlement_reference, soroban_string(&env, "tx-0001"));
}

#[test]
fn payer_can_cancel_pending_intents() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "cancel me"));
    client.cancel_intent(&id);

    let cancelled = client.get_intent(&id);
    assert_eq!(cancelled.status, PaymentStatus::Cancelled);
}

#[test]
fn lists_payer_intents_with_cursor_pagination() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let other_payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    let first_id =
        client.create_intent(&payer, &payee, &10_i128, &soroban_string(&env, "first"));
    let second_id =
        client.create_intent(&payer, &payee, &20_i128, &soroban_string(&env, "second"));
    let third_id =
        client.create_intent(&payer, &payee, &30_i128, &soroban_string(&env, "third"));
    client.create_intent(
        &other_payer,
        &payee,
        &40_i128,
        &soroban_string(&env, "other payer"),
    );

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
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    client.list_intents(&payer, &0_u32, &0_u32);
}

#[test]
#[should_panic]
fn rejects_settle_after_cancellation() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "cancel me"));
    client.cancel_intent(&id);
    client.settle_intent(&id, &soroban_string(&env, "tx-0002"));
}
