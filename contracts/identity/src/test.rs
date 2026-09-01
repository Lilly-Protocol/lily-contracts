#![cfg(test)]

use soroban_sdk::unwrap::UnwrapOptimized;
use soroban_sdk::Address;

use super::{AgentProfile, DataKey, IdentityContract, IdentityContractClient};
use lily_test_support::{soroban_string, test_address, test_env};

#[test]
fn registers_and_updates_profiles() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);
    let new_controller = test_address(&env);

    let contract_id = env.register(IdentityContract, (admin.clone(),));
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://agent-lily/profile-v1"));

    let profile = client.get_profile(&agent);
    assert_eq!(
        profile,
        AgentProfile {
            controller: controller.clone(),
            metadata_uri: soroban_string(&env, "ipfs://agent-lily/profile-v1"),
            active: true,
            revision: 0,
        }
    );

    client.update_profile(
        &agent,
        &soroban_string(&env, "ipfs://agent-lily/profile-v2"),
        &Some(new_controller.clone()),
    );

    let updated = client.get_profile(&agent);
    assert_eq!(updated.controller, new_controller);
    assert_eq!(updated.revision, 1);
}

#[test]
#[should_panic]
fn rejects_duplicate_registration() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, (admin.clone(),));
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
}

#[test]
fn admin_can_deactivate_profiles() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, (admin.clone(),));
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
    client.deactivate(&agent);

    let profile = client.get_profile(&agent);
    assert!(!profile.active);
    assert_eq!(profile.revision, 1);
}

#[test]
#[should_panic]
fn initialize_rejects_admin_other_than_deployer() {
    let env = test_env();
    let deployer_admin = test_address(&env);
    let front_runner = test_address(&env);

    let contract_id = env.register(IdentityContract, (deployer_admin,));
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&front_runner);
}

#[test]
fn initialize_records_admin_pinned_at_deploy_time() {
    let env = test_env();
    let admin = test_address(&env);

    let contract_id = env.register(IdentityContract, (admin.clone(),));
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    let stored = env.as_contract(&contract_id, || {
        let value: Address = env.storage().instance().get(&DataKey::Admin).unwrap_optimized();
        value
    });
    assert_eq!(stored, admin);
}
