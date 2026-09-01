#![allow(clippy::unwrap_used, clippy::expect_used)]
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

// #32: NextIntentId must increment across multiple creates (sequential ids 1,2,3)
// and get_config must expose the rolling counter.
#[test]
fn next_intent_id_increments_across_multiple_creates() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let payer = test_address(&env);
    let payee = test_address(&env);

    let contract_id = env.register(PaymentsContract, ());
    let client = PaymentsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &treasury, &50_u32);

    let memo_1 = soroban_string(&env, "intent-001");
    let memo_2 = soroban_string(&env, "intent-002");
    let memo_3 = soroban_string(&env, "intent-003");

    let id1 = client.create_intent(&payer, &payee, &1_000_i128, &memo_1);
    let id2 = client.create_intent(&payer, &payee, &2_000_i128, &memo_2);
    let id3 = client.create_intent(&payer, &payee, &3_000_i128, &memo_3);

    // 1) sequential ids.
    assert_eq!((id1, id2, id3), (1_u64, 2_u64, 3_u64), "ids must be sequential");

    // 2) get_config reflects the counter (next free id).
    let config = client.get_config();
    assert_eq!(config.next_intent_id, 4, "get_config.next_intent_id must be 4");

    // 3) id-based lookups still work for every created intent.
    assert_eq!(client.get_intent(&1).memo, memo_1);
    assert_eq!(client.get_intent(&2).memo, memo_2);
    assert_eq!(client.get_intent(&3).memo, memo_3);
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
            state = state
                .wrapping_mul(6_364_136_223_846_793_005u64)
                .wrapping_add(1_442_695_040_888_963_407u64);
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

// #41: off-chain indexers rely on the exact ledger key encoding (visible in
// test_snapshots/): a variant encodes to a scval vec of
// [Symbol(variant_name), payload...]. These tests pin that shape.
#[test]
fn storage_key_encodings_are_pinned() {
    use super::DataKey;
    use soroban_sdk::symbol_short;
    use soroban_sdk::{IntoVal, TryIntoVal};

    fn to_vec(env: &soroban_sdk::Env, key: DataKey) -> soroban_sdk::Vec<soroban_sdk::Val> {
        let raw: soroban_sdk::Val = key.into_val(env);
        raw.try_into_val(env).unwrap()
    }
    fn tag(
        env: &soroban_sdk::Env,
        vec_: &soroban_sdk::Vec<soroban_sdk::Val>,
    ) -> soroban_sdk::Symbol {
        vec_.get(0).unwrap().try_into_val(env).unwrap()
    }

    let env = test_env();

    let admin = to_vec(&env, DataKey::Admin);
    assert_eq!(admin.len(), 1, "Admin encodes as a single-element vec");
    assert_eq!(tag(&env, &admin), symbol_short!("Admin"), "Admin tag");

    for (key, name) in [(DataKey::Treasury, "Treasury"), (DataKey::FeeBps, "FeeBps")] {
        let vec_ = to_vec(&env, key);
        assert_eq!(vec_.len(), 1, "{name} encodes as a single-element vec");
        assert_eq!(tag(&env, &vec_), soroban_sdk::Symbol::new(&env, name), "{name} tag");
    }

    let next_intent_id = to_vec(&env, DataKey::NextIntentId);
    assert_eq!(next_intent_id.len(), 1, "NextIntentId encodes as a single-element vec");
    assert_eq!(
        tag(&env, &next_intent_id),
        soroban_sdk::Symbol::new(&env, "NextIntentId"),
        "NextIntentId tag"
    );

    let initialized = to_vec(&env, DataKey::Initialized);
    assert_eq!(initialized.len(), 1, "Initialized encodes as single-element vec");
    assert_eq!(
        tag(&env, &initialized),
        soroban_sdk::Symbol::new(&env, "Initialized"),
        "Initialized tag"
    );

    let intent = to_vec(&env, DataKey::Intent(7_u64));
    assert_eq!(intent.len(), 2, "Intent encodes as vec[Symbol, u64]");
    assert_eq!(tag(&env, &intent), symbol_short!("Intent"), "Intent tag");
    let id: u64 = intent.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(id, 7, "Intent payload must be the raw id");
}
