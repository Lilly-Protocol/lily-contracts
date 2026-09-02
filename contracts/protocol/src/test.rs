#![cfg(test)]

use super::{ProtocolConfig, ProtocolContract, ProtocolContractClient};
use lily_test_support::{test_address, test_env};

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

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
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

    let contract_id = env.register(ProtocolContract, (admin.clone(),));
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &10_001_u32);
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

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&next_admin);

    let config = client.get_config();
    assert_eq!(config.admin, next_admin);
}

#[test]
#[should_panic]
fn rejects_set_fee_bps_above_max() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.set_fee_bps(&10_001_u32);
}

