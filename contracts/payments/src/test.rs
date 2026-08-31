#![cfg(test)]

use lily_common::PaymentStatus;
use lily_test_support::{soroban_string, test_address, test_env};
use soroban_sdk::symbol_short;
use wallet::{WalletContract, WalletContractClient};

use super::{PaymentIntent, PaymentsContract, PaymentsContractClient};

fn setup_wallet(env: &soroban_sdk::Env, admin: &soroban_sdk::Address) -> soroban_sdk::Address {
    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(env, &contract_id);
    client.initialize(admin);
    contract_id
}

fn bind_payer(
    env: &soroban_sdk::Env,
    wallet_id: &soroban_sdk::Address,
    payer: &soroban_sdk::Address,
) {
    let wallet_addr = test_address(env);
    let client = WalletContractClient::new(env, wallet_id);
    client.bind_wallet(payer, &wallet_addr, &symbol_short!("USDC"), &10_000_i128);
}

#[test]
fn creates_and_settles_payment_intents() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let wallet_id = setup_wallet(&env, &admin);
    bind_payer(&env, &wallet_id, &payer);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32, &wallet_id);
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

    let wallet_id = setup_wallet(&env, &admin);
    bind_payer(&env, &wallet_id, &payer);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32, &wallet_id);
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

    let wallet_id = setup_wallet(&env, &admin);
    bind_payer(&env, &wallet_id, &payer);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32, &wallet_id);
    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "cancel me"));
    client.cancel_intent(&id);
    client.settle_intent(&id, &soroban_string(&env, "tx-0002"));
}

#[test]
#[should_panic]
fn rejects_create_intent_without_active_binding() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let wallet_id = setup_wallet(&env, &admin);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32, &wallet_id);
    client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "no binding"));
}

#[test]
#[should_panic]
fn rejects_create_intent_with_disabled_binding() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);
    let wallet_addr = test_address(&env);

    let wallet_id = setup_wallet(&env, &admin);
    let wallet_client = WalletContractClient::new(&env, &wallet_id);
    wallet_client.bind_wallet(&payer, &wallet_addr, &symbol_short!("USDC"), &10_000_i128);
    wallet_client.set_enabled(&payer, &false);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32, &wallet_id);
    client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "disabled"));
}
