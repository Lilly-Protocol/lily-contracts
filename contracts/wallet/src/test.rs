#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use soroban_sdk::symbol_short;

use super::{WalletBinding, WalletContract, WalletContractClient};
use lily_test_support::{test_address, test_env};

#[test]
fn binds_wallet_and_updates_policy() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);

    let binding = client.get_binding(&agent);
    assert_eq!(
        binding,
        WalletBinding {
            wallet: wallet.clone(),
            settlement_asset: symbol_short!("USDC"),
            spend_limit: 1_000,
            enabled: true,
            revision: 0,
        }
    );

    client.update_spend_limit(&agent, &2_500_i128);
    client.set_enabled(&agent, &false);

    let updated = client.get_binding(&agent);
    assert_eq!(updated.spend_limit, 2_500);
    assert!(!updated.enabled);
    assert_eq!(updated.revision, 2);
}

#[test]
#[should_panic]
fn rejects_double_binding_while_active() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
}

#[test]
#[should_panic]
fn rejects_zero_spend_limit() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &0_i128);
}

// #36: every wallet mutating function must emit its event with the correct
// short-symbol topic and the full binding as payload.
#[test]
fn wallet_publishes_event_for_every_mutating_operation() {
    use soroban_sdk::testutils::Events;
    use soroban_sdk::TryIntoVal;

    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    assert_eq!(env.events().all().len(), 1, "initialize emits one event");
    let (_c, topics, data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 1, "init event carries one topic");
    let init_topic: soroban_sdk::Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(init_topic, symbol_short!("init"), "init topic");
    let init_payload: soroban_sdk::Address = data.try_into_val(&env).unwrap();
    assert_eq!(init_payload, admin, "init payload must be admin");

    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &10_000_i128);
    assert_eq!(env.events().all().len(), 1, "bind_wallet emits one event");
    let (_c, topics, data) = env.events().all().get(0).unwrap();
    assert_eq!(topics.len(), 2, "bind event carries topic + agent");
    let bind_topic: soroban_sdk::Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(bind_topic, symbol_short!("bind"), "bind topic");
    let bind_agent: soroban_sdk::Address = topics.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(bind_agent, agent, "bind topic carries agent");
    let bound: WalletBinding = data.try_into_val(&env).unwrap();
    assert_eq!(bound.wallet, wallet);
    assert!(bound.enabled);
    assert_eq!(bound.revision, 0);
    assert_eq!(bound.settlement_asset, symbol_short!("USDC"));
    assert_eq!(bound.spend_limit, 10_000);

    client.update_spend_limit(&agent, &25_000_i128);
    assert_eq!(env.events().all().len(), 1, "update_spend_limit emits one event");
    let (_c, topics, data) = env.events().all().get(0).unwrap();
    let limit_topic: soroban_sdk::Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(limit_topic, symbol_short!("limit"), "limit topic");
    let limit_agent: soroban_sdk::Address = topics.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(limit_agent, agent, "limit topic carries agent");
    let lim: WalletBinding = data.try_into_val(&env).unwrap();
    assert_eq!(lim.spend_limit, 25_000);
    assert_eq!(lim.revision, 1, "revision must bump on limit update");

    client.set_enabled(&agent, &false);
    assert_eq!(env.events().all().len(), 1, "set_enabled emits one event");
    let (_c, topics, data) = env.events().all().get(0).unwrap();
    let state_topic: soroban_sdk::Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(state_topic, symbol_short!("state"), "state topic");
    let state_agent: soroban_sdk::Address = topics.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(state_agent, agent, "state topic carries agent");
    let st: WalletBinding = data.try_into_val(&env).unwrap();
    assert!(!st.enabled, "payload must be disabled");
    assert_eq!(st.revision, 2, "revision must bump on state update");
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

    let initialized = to_vec(&env, DataKey::Initialized);
    assert_eq!(initialized.len(), 1, "Initialized encodes as single-element vec");
    assert_eq!(
        tag(&env, &initialized),
        soroban_sdk::Symbol::new(&env, "Initialized"),
        "Initialized tag"
    );

    let agent = test_address(&env);
    let binding = to_vec(&env, DataKey::Binding(agent.clone()));
    assert_eq!(binding.len(), 2, "Binding encodes as vec[Symbol, Address]");
    assert_eq!(tag(&env, &binding), symbol_short!("Binding"), "Binding tag");
    let addr: soroban_sdk::Address = binding.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(addr, agent, "Binding payload must be the agent address");
}
