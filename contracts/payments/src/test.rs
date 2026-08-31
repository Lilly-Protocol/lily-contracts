#![cfg(test)]

use lily_common::PaymentStatus;
use lily_test_support::{soroban_string, test_address, test_env};

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

#[test]
fn quotes_fee_at_zero_bps() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &0_u32);
    let (fee, net) = client.quote_fee(&10_000_i128);
    assert_eq!(fee, 0);
    assert_eq!(net, 10_000);
}

#[test]
fn quotes_fee_at_mid_bps() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    // 100 bps = 1%
    client.initialize(&admin, &treasury, &100_u32);
    let (fee, net) = client.quote_fee(&10_000_i128);
    assert_eq!(fee, 100);
    assert_eq!(net, 9_900);
}

#[test]
fn quotes_fee_at_max_bps() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    // 10_000 bps = 100%
    client.initialize(&admin, &treasury, &10_000_u32);
    let (fee, net) = client.quote_fee(&10_000_i128);
    assert_eq!(fee, 10_000);
    assert_eq!(net, 0);
}

#[test]
#[should_panic]
fn rejects_quote_for_non_positive_amount() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.quote_fee(&0_i128);
}
