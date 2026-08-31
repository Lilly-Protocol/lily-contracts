#![cfg(test)]

use super::{ProtocolConfig, ProtocolContract, ProtocolContractClient};
use lily_test_support::{test_address, test_env};

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
fn two_step_admin_transfer_flow() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let new_admin = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    assert_eq!(client.get_pending_admin(), None);

    // Step 1: Current admin proposes new admin
    client.transfer_admin(&new_admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin.clone()));
    // Old admin retains authority until acceptance
    assert_eq!(client.get_config().admin, admin);

    // Step 2: Proposed admin accepts authority
    client.accept_admin();
    assert_eq!(client.get_pending_admin(), None);
    assert_eq!(client.get_config().admin, new_admin);
}

#[test]
#[should_panic]
fn rejects_accept_admin_without_pending() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    // Should panic because no pending admin exists
    client.accept_admin();
}

#[test]
#[should_panic]
fn rejects_accept_admin_by_unauthorized_party() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let new_admin = test_address(&env);
    let unauthorized = test_address(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &100_u32);
    client.transfer_admin(&new_admin);

    // Mock auth as unauthorized non-pending caller
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &unauthorized,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "accept_admin",
            args: soroban_sdk::vec![&env],
            sub_invokes: &[],
        },
    }]);

    client.accept_admin();
}

