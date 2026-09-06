#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use lily_common::{PaymentStatus, ProtocolError, PROTOCOL_VERSION};
use lily_test_support::{soroban_string, test_address, test_env};
use soroban_sdk::testutils::{Events, Ledger};
use soroban_sdk::unwrap::UnwrapOptimized;
use soroban_sdk::{symbol_short, Address, IntoVal, Symbol, TryIntoVal};

use super::{PaymentStatus, PaymentIntent, PaymentsContract, PaymentsContractClient, MAX_PAYMENT_AMOUNT};

fn bootstrap() -> (soroban_sdk::Env, soroban_sdk::Address, PaymentsContractClient<'static>) {
    let env = test_env();
    let treasury = test_address(&env);
    let admin = test_address(&env);

    let contract_id = env.register(PaymentsContract, (admin.clone(),));
    let client = PaymentsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury, &50_u32);
    (env, admin, client)
}

#[test]
fn returns_protocol_version() {
    let env = test_env();
    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    assert_eq!(client.version(), PROTOCOL_VERSION);
}

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
    let (env, admin, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);

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
    let (env, _admin, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);

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
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, (admin.clone(),));
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "cancel me"));
    client
        .mock_auths(&[MockAuth {
            address: &payer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "cancel_intent",
                args: (&id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .cancel_intent(&id);

    let cancelled = client.get_intent(&id);
    assert_eq!(cancelled.status, PaymentStatus::Cancelled);
}

#[test]
fn accepts_the_maximum_payment_amount() {
    let (env, _admin, client) = bootstrap();
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
        &MAX_PAYMENT_AMOUNT,
        &soroban_string(&env, "maximum payment"),
    );

    assert_eq!(client.get_intent(&id).amount, MAX_PAYMENT_AMOUNT);
}

#[test]
fn lists_payer_intents_with_cursor_pagination() {
    let (env, _admin, client) = bootstrap();
    let payer = test_address(&env);
    let other_payer = test_address(&env);
    let payee = test_address(&env);

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
    let (env, _admin, client) = bootstrap();
    let payer = test_address(&env);

    client.list_intents(&payer, &0_u32, &0_u32);
}

#[test]
#[should_panic]
fn rejects_payment_amount_above_the_maximum() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let wallet_id = setup_wallet(&env, &admin);
    bind_payer(&env, &wallet_id, &payer);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
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
    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);
    client.get_config();
}

#[test]
#[should_panic]
fn rejects_settle_after_cancellation() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, (admin.clone(),));
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32, &wallet_id);
    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "cancel me"));
    client.cancel_intent(&id);
    client.settle_intent(&admin, &id, &soroban_string(&env, "tx-0002"));
}

// Typed role error: ProtocolError::Unauthorized = 3 (issue #100).
#[test]
#[should_panic = "Error(Contract, #3)"]
fn settle_rejects_non_admin_caller_with_typed_unauthorized() {
    let (env, _admin, client) = bootstrap();
    let payer = test_address(&env);
    let payee = test_address(&env);

    let id = client.create_intent(&payer, &payee, &5_000_i128, &soroban_string(&env, "not yours"));
    // Payer tries to settle: signature would pass under mock_all_auths, but
    // the typed role check must fire first with ProtocolError::Unauthorized.
    client.settle_intent(&payer, &id, &soroban_string(&env, "tx-not-admin"));
}

#[test]
fn two_step_admin_transfer_lifecycle() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(PaymentsContract, (admin.clone(),));
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);

    // Prior to transfer, get_pending_admin returns None
    assert_eq!(client.get_pending_admin(), None);

    // Step 1: propose next_admin
    client.transfer_admin(&next_admin);

    // After propose: previous admin can still call set_fee_bps, and get_pending_admin returns Some(next_admin)
    client.set_fee_bps(&100_u32);
    assert_eq!(client.get_config().fee_bps, 100);
    assert_eq!(client.get_config().admin, admin);
    assert_eq!(client.get_pending_admin(), Some(next_admin.clone()));

    // Verify "propose" event was emitted
    let propose_events: Vec<_> = env
        .events()
        .all()
        .iter()
        .filter(|(contract, topics, _)| {
            *contract == contract_id
                && topics.get(0).map_or(false, |t| {
                    let sym: Result<Symbol, _> = t.try_into_val(&env);
                    sym == Ok(symbol_short!("propose"))
                })
        })
        .collect();
    assert_eq!(propose_events.len(), 1);

    // Step 2: next_admin accepts
    client.accept_admin();

    // After accept: admin is updated, pending_admin is cleared (None)
    assert_eq!(client.get_config().admin, next_admin);
    assert_eq!(client.get_pending_admin(), None);

    // Exactly one "admin" event was fired
    let admin_events: Vec<_> = env
        .events()
        .all()
        .iter()
        .filter(|(contract, topics, _)| {
            *contract == contract_id
                && topics.get(0).map_or(false, |t| {
                    let sym: Result<Symbol, _> = t.try_into_val(&env);
                    sym == Ok(symbol_short!("admin"))
                })
        })
        .collect();
    assert_eq!(admin_events.len(), 1);
    let (_, topics, payload) = &admin_events[0];
    let topic0: Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
    let topic1: Address = topics.get(1).unwrap().try_into_val(&env).unwrap();
    let data: Address = payload.clone().try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("admin"));
    assert_eq!(topic1, admin);
    assert_eq!(data, next_admin);
}

// Typed error: ProtocolError::MissingRecord = 7.
#[test]
#[should_panic = "Error(Contract, #7)"]
fn accept_admin_panics_missing_record_when_no_pending_admin() {
    let (env, _admin, client) = bootstrap();
    client.accept_admin();
}

#[test]
#[should_panic = "Error(Contract, #7)"]
fn later_accepts_panic_missing_record_after_transfer_completed() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(PaymentsContract, (admin.clone(),));
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    client.transfer_admin(&next_admin);
    client.accept_admin();

    // Calling accept_admin again after it was already accepted must panic MissingRecord
    client.accept_admin();
}

#[test]
#[should_panic]
fn only_pending_admin_can_accept_and_old_admin_accepting_panics() {
    let env = soroban_sdk::Env::default();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(PaymentsContract, (admin.clone(),));
    let client = PaymentsContractClient::new(&env, &contract_id);

    // Initialize with admin auth
    client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "initialize",
                args: (&admin, &treasury, &50_u32).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .initialize(&admin, &treasury, &50_u32);

    // Propose next_admin with admin auth
    client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "transfer_admin",
                args: (&next_admin,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .transfer_admin(&next_admin);

    // Old admin attempts to accept (only next_admin can accept): must panic auth failure
    client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "accept_admin",
                args: ().into_val(&env),
                sub_invokes: &[],
            },
        }])
        .accept_admin();
}

#[test]
#[should_panic]
fn rejects_zero_amount_intent() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    client.create_intent(&payer, &payee, &0_i128, &soroban_string(&env, "invalid zero amount"));
}

#[test]
#[should_panic]
fn rejects_get_intent_on_missing_record() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);
    client.get_intent(&999_u64);
}

#[test]
fn get_intent_opt_returns_none_for_unknown_and_some_for_cancelled_or_settled() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);
    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &50_u32);

    assert_eq!(client.get_intent_opt(&999_u64), None);

    let intent_id = client.create_intent(&payer, &payee, &100_i128, &soroban_string(&env, "memo"));
    let intent = client.get_intent_opt(&intent_id);
    assert!(intent.is_some());
    assert_eq!(intent.unwrap().status, PaymentStatus::Pending);

    client.cancel_intent(&intent_id);
    let cancelled = client.get_intent_opt(&intent_id);
    assert!(cancelled.is_some());
    assert_eq!(cancelled.unwrap().status, PaymentStatus::Cancelled);
}
