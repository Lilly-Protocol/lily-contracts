#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use super::{ProtocolConfig, ProtocolContract, ProtocolContractClient, SCHEMA_VERSION};
use lily_common::{ProtocolError, INSTANCE_BUMP_AMOUNT, INSTANCE_BUMP_THRESHOLD};
use lily_test_support::{test_address, test_env};
use soroban_sdk::{
    symbol_short,
    testutils::{storage::Instance as _, Events, Ledger as _, MockAuth, MockAuthInvoke},
    vec,
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
    let admin = test_address(&env);
    let contract_id = env.register(ProtocolContract, (admin,));
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
    let admin = test_address(&env);
    let contract_id = env.register(ProtocolContract, (admin,));
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
    assert_eq!(client.get_pending_admin(), None);

    // Step 1: Current admin proposes next admin
    client.transfer_admin(&next_admin);

    // Assert propose event
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let propose_event = events.get_unchecked(0);
    assert_eq!(propose_event.0, contract_id);
    let propose_topic0: soroban_sdk::Symbol =
        propose_event.1.get_unchecked(0).try_into_val(&env).unwrap();
    let propose_topic1: Address = propose_event.1.get_unchecked(1).try_into_val(&env).unwrap();
    let propose_data: Address = propose_event.2.try_into_val(&env).unwrap();
    assert_eq!(propose_topic0, symbol_short!("propose"));
    assert_eq!(propose_topic1, admin);
    assert_eq!(propose_data, next_admin);

    // Assert pending admin is set, but active admin remains unchanged
    assert_eq!(client.get_pending_admin(), Some(next_admin.clone()));
    let config_mid = client.get_config();
    assert_eq!(config_mid.admin, admin);

    // Step 2: Next admin accepts admin authority
    client.accept_admin();

    // Assert admin event
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let admin_event = events.get_unchecked(0);
    assert_eq!(admin_event.0, contract_id);
    let admin_topic0: soroban_sdk::Symbol =
        admin_event.1.get_unchecked(0).try_into_val(&env).unwrap();
    let admin_topic1: Address = admin_event.1.get_unchecked(1).try_into_val(&env).unwrap();
    let admin_data: Address = admin_event.2.try_into_val(&env).unwrap();
    assert_eq!(admin_topic0, symbol_short!("admin"));
    assert_eq!(admin_topic1, admin);
    assert_eq!(admin_data, next_admin);

    // Assert active admin is transferred and pending admin is cleared
    assert_eq!(client.get_pending_admin(), None);
    let config_final = client.get_config();
    assert_eq!(config_final.admin, next_admin);
}

#[test]
#[should_panic]
fn rejects_accept_admin_by_unauthorized_party() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);
    let unauthorized = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&next_admin);

    // Mock auth as unauthorized non-pending caller
    env.mock_auths(&[MockAuth {
        address: &unauthorized,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "accept_admin",
            args: vec![&env],
            sub_invokes: &[],
        },
    }]);

    client.accept_admin();
}

#[test]
#[should_panic]
fn rejects_accept_admin_without_pending() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    // Should panic because no pending admin exists (ProtocolError::MissingRecord)
    client.accept_admin();
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
#[should_panic]
fn get_pending_admin_before_initialize_panics_not_initialized() {
    let env = test_env();
    let admin = test_address(&env);
    let contract_id = env.register(ProtocolContract, (admin,));
    let client = ProtocolContractClient::new(&env, &contract_id);

    let _ = client.get_pending_admin();
}

#[test]
fn get_pending_admin_lifecycle_after_transfer_and_accept() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    assert_eq!(client.get_pending_admin(), None);

    client.transfer_admin(&next_admin);
    assert_eq!(client.get_pending_admin(), Some(next_admin.clone()));

    client.accept_admin();
    assert_eq!(client.get_pending_admin(), None);
}

#[test]
fn get_pending_admin_bumps_instance_ttl() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    env.ledger().set_sequence_number(INSTANCE_BUMP_AMOUNT - INSTANCE_BUMP_THRESHOLD + 1);
    let ttl_before = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
    assert!(ttl_before < INSTANCE_BUMP_THRESHOLD);

    let _ = client.get_pending_admin();

    let ttl_after = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
    assert_eq!(ttl_after, INSTANCE_BUMP_AMOUNT);
}

#[test]
fn transfer_admin_sets_pending_and_preserves_current_admin() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    assert_eq!(client.get_pending_admin(), None);

    client.transfer_admin(&next_admin);

    // After transfer_admin, get_pending_admin == Some(next) and get_config().admin is still the old admin
    assert_eq!(client.get_pending_admin(), Some(next_admin));
    assert_eq!(client.get_config().admin, admin);
}

#[test]
#[should_panic]
fn rejects_accept_admin_by_old_admin() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&next_admin);

    // Old admin attempts to invoke accept_admin, but accept_admin requires pending_admin's auth
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "accept_admin",
            args: vec![&env],
            sub_invokes: &[],
        },
    }]);

    client.accept_admin();
}

#[test]
fn old_admin_authority_holds_until_pending_admin_accepts() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&next_admin);

    // Old admin authority holds until acceptance: old admin can still configure protocol
    client.set_fee_bps(&200_u32);
    assert_eq!(client.get_config().fee_bps, 200);

    // Pending admin's accept flips get_config().admin
    client.accept_admin();
    assert_eq!(client.get_config().admin, next_admin);

    // New admin now holds authority
    client.set_fee_bps(&300_u32);
    assert_eq!(client.get_config().fee_bps, 300);
}

#[test]
fn second_accept_admin_panics_missing_record() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&next_admin);

    // First accept succeeds
    client.accept_admin();
    assert_eq!(client.get_config().admin, next_admin);

    // No pending key remains in storage
    assert_eq!(client.get_pending_admin(), None);

    // A second accept_admin panics with ProtocolError::MissingRecord
    let result = client.try_accept_admin();
    assert_eq!(result, Err(Ok(ProtocolError::MissingRecord.into())));
}

#[test]
#[should_panic]
fn second_accept_admin_panics() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let next_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&next_admin);
    client.accept_admin();

    assert_eq!(client.get_pending_admin(), None);
    // Direct invocation panics
    client.accept_admin();
}
