#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use soroban_sdk::unwrap::UnwrapOptimized;
use soroban_sdk::Address;

use super::{AgentProfile, DataKey, IdentityContract, IdentityContractClient};
use lily_test_support::{soroban_string, test_address, test_env};
use soroban_sdk::{FromVal, IntoVal, Symbol, Val, Vec};

#[test]
fn data_key_encodings_are_stable() {
    let env = test_env();
    let agent = test_address(&env);

    let admin: Vec<Val> = soroban_sdk::vec![&env, Symbol::new(&env, "Admin").into_val(&env)];
    let initialized: Vec<Val> =
        soroban_sdk::vec![&env, Symbol::new(&env, "Initialized").into_val(&env)];
    let profile: Vec<Val> = soroban_sdk::vec![
        &env,
        Symbol::new(&env, "Profile").into_val(&env),
        agent.clone().into_val(&env),
    ];

    let actual_admin: Val = DataKey::Admin.into_val(&env);
    let actual_initialized: Val = DataKey::Initialized.into_val(&env);
    let actual_profile: Val = DataKey::Profile(agent).into_val(&env);
    assert_eq!(Vec::<Val>::from_val(&env, &actual_admin), admin);
    assert_eq!(Vec::<Val>::from_val(&env, &actual_initialized), initialized);
    assert_eq!(Vec::<Val>::from_val(&env, &actual_profile), profile);
}

#[test]
fn returns_protocol_version() {
    let env = test_env();
    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    assert_eq!(client.version(), PROTOCOL_VERSION);
}

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
fn rejects_update_on_deactivated_profile() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
    client.deactivate(&agent);

    client.update_profile(
        &agent,
        &soroban_string(&env, "ipfs://profile-v2"),
        &None,
    );
}

#[test]
#[should_panic]
fn rejects_get_profile_on_unregistered_agent() {
    let env = test_env();
    let admin = test_address(&env);
    let unknown_agent = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.get_profile(&unknown_agent);
}

