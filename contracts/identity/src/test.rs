#![cfg(test)]

use super::{AgentProfile, IdentityContract, IdentityContractClient};
use lily_test_support::{soroban_string, test_address, test_env};
use soroban_sdk::{symbol_short, Address, TryIntoVal};
use soroban_sdk::testutils::Events;

#[test]
fn registers_and_updates_profiles() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);
    let new_controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
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

    let contract_id = env.register(IdentityContract, ());
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

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
    client.deactivate(&agent);

    let profile = client.get_profile(&agent);
    assert!(!profile.active);
    assert_eq!(profile.revision, 1);
}

#[test]
fn initialize_emits_init_event() {
    let env = test_env();
    let admin = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("init"));

    let data: Address = event.2.try_into_val(&env).unwrap();
    assert_eq!(data, admin);
}

#[test]
fn register_emits_register_event() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));

    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("register"));

    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic1, agent);

    let data: AgentProfile = event.2.try_into_val(&env).unwrap();
    assert_eq!(
        data,
        AgentProfile {
            controller,
            metadata_uri: soroban_string(&env, "ipfs://profile"),
            active: true,
            revision: 0,
        }
    );
}

#[test]
fn update_emits_update_event() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);
    let new_controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
    client.update_profile(
        &agent,
        &soroban_string(&env, "ipfs://profile-v2"),
        &Some(new_controller.clone()),
    );

    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("update"));

    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic1, agent);

    let data: AgentProfile = event.2.try_into_val(&env).unwrap();
    assert_eq!(
        data,
        AgentProfile {
            controller: new_controller,
            metadata_uri: soroban_string(&env, "ipfs://profile-v2"),
            active: true,
            revision: 1,
        }
    );
}

#[test]
fn deactivate_emits_deact_event() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
    client.deactivate(&agent);

    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(event.0, contract_id);

    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("deact"));

    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic1, agent);

    let data: AgentProfile = event.2.try_into_val(&env).unwrap();
    assert_eq!(
        data,
        AgentProfile {
            controller,
            metadata_uri: soroban_string(&env, "ipfs://profile"),
            active: false,
            revision: 1,
        }
    );
}
