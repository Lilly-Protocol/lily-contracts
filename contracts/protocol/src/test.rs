#![cfg(test)]

use super::{ProtocolConfig, ProtocolContract, ProtocolContractClient};
use lily_test_support::{test_address, test_env};
use soroban_sdk::{symbol_short, Address, TryIntoVal};
use soroban_sdk::testutils::Events;

#[test]
fn initializes_once_and_reads_config() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &250_u32);
    assert!(client.is_initialized());

    let config = client.get_config();
    assert_eq!(config, ProtocolConfig { admin, treasury, fee_bps: 250 });
}

#[test]
fn initialize_emits_init_event() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &250_u32);

    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let event = events.get_unchecked(0);
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("init"));

    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic1, admin);

    let data: ProtocolConfig = event.2.try_into_val(&env).unwrap();
    assert_eq!(
        data,
        ProtocolConfig {
            admin: admin.clone(),
            treasury: treasury.clone(),
            fee_bps: 250,
        }
    );
}

#[test]
#[should_panic]
fn rejects_reinitialization() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.initialize(&admin, &treasury, &100_u32);
}

#[test]
#[should_panic]
fn rejects_fee_bps_above_max() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &10_001_u32);
}

#[test]
fn updates_fee_and_treasury() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let new_treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.set_fee_bps(&375_u32);
    client.set_treasury(&new_treasury);

    let config = client.get_config();
    assert_eq!(config.fee_bps, 375);
    assert_eq!(config.treasury, new_treasury);
}

#[test]
fn update_fee_emits_fee_event() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.set_fee_bps(&375_u32);

    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("fee"));

    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic1, admin);

    let data: u32 = event.2.try_into_val(&env).unwrap();
    assert_eq!(data, 375);
}

#[test]
fn update_treasury_emits_treasury_event() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let new_treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.set_treasury(&new_treasury);

    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("treasury"));

    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic1, admin);

    let data: Address = event.2.try_into_val(&env).unwrap();
    assert_eq!(data, new_treasury);
}

#[test]
fn transfer_admin_emits_admin_event() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let new_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&new_admin);

    // Read events before any further contract call, because env.events().all()
    // only reflects the most recent invocation in the Soroban test host.
    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("admin"));

    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic1, admin);

    let data: Address = event.2.try_into_val(&env).unwrap();
    assert_eq!(data, new_admin);

    let config = client.get_config();
    assert_eq!(config.admin, new_admin);
}
