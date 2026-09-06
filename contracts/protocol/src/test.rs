#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use super::{SCHEMA_VERSION, ProtocolConfig, ProtocolContract, ProtocolContractClient};
use lily_common::PROTOCOL_VERSION;
use lily_test_support::{test_address, test_env};
use soroban_sdk::symbol_short;
use soroban_sdk::testutils::Events;
use soroban_sdk::{
    xdr::{ScErrorCode, ScErrorType},
    Address, Error, TryIntoVal,
};

#[test]
fn returns_protocol_version() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &250_u32);
    assert_eq!(client.schema_version(), SCHEMA_VERSION);
}

#[test]
fn initializes_once_and_reads_config() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &250_u32);
    assert!(client.is_initialized());

    let config = client.get_config();
    assert_eq!(
        config,
        ProtocolConfig { admin: admin.clone(), treasury: treasury.clone(), fee_bps: 250 }
    );
}

#[test]
fn initialize_emits_init_event() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
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
        ProtocolConfig { admin: admin.clone(), treasury: treasury.clone(), fee_bps: 250 }
    );
}

#[test]
#[should_panic]
fn rejects_config_read_before_initialization() {
    let env = test_env();
    let contract_id = env.register(ProtocolContract, (test_address(&env),));
    let client = ProtocolContractClient::new(&env, &contract_id);
    client.get_config();
}

#[test]
#[should_panic]
fn rejects_reinitialization() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.initialize(&admin, &treasury, &100_u32);
}

#[test]
#[should_panic]
fn get_config_before_initialize_panics_not_initialized() {
    // ensure_initialized panics with ProtocolError::NotInitialized via panic_with_error
    // when DataKey::Initialized is absent (lily_common::require -> panic_with_error!).
    let env = test_env();
    let contract_id = env.register(ProtocolContract, (test_address(&env),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    let _ = client.get_config();
}

#[test]
#[should_panic]
fn rejects_fee_bps_above_max() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &10_001_u32);
}

#[test]
fn unauthenticated_invalid_initialization_fails_at_auth() {
    let env = soroban_sdk::Env::default();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    let result = client.try_initialize(&admin, &treasury, &10_001_u32);
    assert_eq!(
        result,
        Err(Ok(Error::from_type_and_code(ScErrorType::Context, ScErrorCode::InvalidAction,)))
    );
}

#[test]
fn unauthenticated_fee_update_fails_before_validation() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    env.set_auths(&[]);

    let result = client.try_set_fee_bps(&10_001_u32);
    assert_eq!(
        result,
        Err(Ok(Error::from_type_and_code(ScErrorType::Context, ScErrorCode::InvalidAction,)))
    );
}

#[test]
fn updates_fee_and_treasury() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.set_fee_bps(&375_u32);
    client.set_treasury(&next_treasury);

    let config = client.get_config();
    assert_eq!(config.fee_bps, 375);
    assert_eq!(config.treasury, next_treasury);
}

#[test]
fn transfers_admin_and_emits_event() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&next_admin);
    client.accept_admin();

    let config = client.get_config();
    assert_eq!(config.admin, next_admin);
}

#[test]
#[should_panic]
fn rejects_set_fee_bps_above_max() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.set_fee_bps(&10_001_u32);
}

#[test]
fn schema_version_matches_constant_after_initialize() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &250);
    assert_eq!(client.schema_version(), SCHEMA_VERSION);
}

#[test]
#[should_panic]
fn rejects_schema_version_before_initialization() {
    let env = test_env();
    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);
    client.schema_version();
}
