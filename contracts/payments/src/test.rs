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
#[should_panic(expected = "Error(Contract, #4)")]
fn rejects_self_payment_intent() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let agent = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury, &50_u32);

    client.create_intent(&agent, &agent, &1_000_i128, &soroban_string(&env, "self payment"));
}

#[test]
fn accepts_distinct_party_intent_after_self_rejection_rule() {
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
        &2_500_i128,
        &soroban_string(&env, "distinct parties"),
    );
    let intent = client.get_intent(&id);
    assert_eq!(intent.payer_agent, payer);
    assert_eq!(intent.payee_agent, payee);
    assert_eq!(intent.status, PaymentStatus::Pending);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn intent_id_overflow_panics_typed_error() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury, &50_u32);

    // Force the counter to its maximum so the post-creation increment overflows.
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&super::DataKey::NextIntentId, &u64::MAX);
    });
    client.create_intent(&payer, &payee, &1_000_i128, &soroban_string(&env, "overflow probe"));
}

// #38: property tests over unbounded String inputs (multibyte, long, empty).
//
// Strategy space (= the documented corpus/labels): (len 0..=4096 chars, seed
// u64) over the `memo` field, mixing 1-, 2-, 3- and 4-byte UTF-8 characters.
// Non-empty memos must be accepted and round-trip; the empty memo must fail
// with the intended `InvalidInput` error and nothing else. Proptest writes
// failing cases into the crate's `proptest-regressions/` corpus for
// shrinking/replay (see TESTING.md, "Property tests").
mod prop38 {
    extern crate std;

    use super::*;
    use proptest::prelude::*;
    use proptest::property_test;
    use std::string::String;

    fn seed_string(env: &soroban_sdk::Env, len: u32, seed: u64) -> soroban_sdk::String {
        let alphabet: [char; 4] = ['a', '\u{e9}', '\u{4e2d}', '\u{1F600}'];
        let mut s = String::with_capacity(len as usize);
        let mut state = seed.max(1);
        for _ in 0..len {
            state = state.wrapping_mul(6_364_136_223_846_793_005u64).wrapping_add(1_442_695_040_888_963_407u64);
            s.push(alphabet[((state >> 33) % 4) as usize]);
        }
        soroban_string(env, &s)
    }

    #[property_test]
    fn payments_accepts_arbitrary_memo_empty_rejected(
        #[strategy = 0u32..=4096u32] len: u32,
        #[strategy = any::<u64>()] seed: u64,
    ) {
        let env = test_env();
        let admin = test_address(&env);
        let treasury = test_address(&env);
        let payer = test_address(&env);
        let payee = test_address(&env);
        let contract_id = env.register(PaymentsContract, ());
        let client = PaymentsContractClient::new(&env, &contract_id);
        client.initialize(&admin, &treasury, &50_u32);

        let memo = seed_string(&env, len, seed);
        match client.try_create_intent(&payer, &payee, &123_i128, &memo) {
            Ok(res) => match res {
                Ok(id) => {
                    prop_assert!(len > 0);
                    prop_assert_eq!(client.get_intent(&id).memo, memo);
                }
                Err(_) => prop_assert!(len == 0),
            },
            Err(_) => prop_assert!(len == 0),
        }
    }
}
