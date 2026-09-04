#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use soroban_sdk::unwrap::UnwrapOptimized;
use soroban_sdk::Address;

use super::{AgentProfile, DataKey, IdentityContract, IdentityContractClient};
use lily_test_support::{soroban_string, test_address, test_env};
use soroban_sdk::testutils::Events as _;
use soroban_sdk::{symbol_short, FromVal, IntoVal, Symbol, TryIntoVal, Val, Vec};

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
fn initializes_and_exposes_config() {
    let env = test_env();
    let admin = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let config = client.get_config();
    assert_eq!(config, IdentityConfig { admin: admin.clone() });
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
fn emits_both_events_when_metadata_and_controller_change() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);
    let new_controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile-v1"));
    client.update_profile(
        &agent,
        &soroban_string(&env, "ipfs://profile-v2"),
        &Some(new_controller),
    );

    let events = env.events().all();
    let metadata_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "metadata_updated"))
        .collect();
    assert_eq!(metadata_events.len(), 1);

    let rotate_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "controller_rotated"))
        .collect();
    assert_eq!(rotate_events.len(), 1);
}

#[test]
fn emits_only_metadata_updated_when_controller_unchanged() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile-v1"));

    client.update_profile(&agent, &soroban_string(&env, "ipfs://profile-v2"), &None);

    let events = env.events().all();
    let metadata_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "metadata_updated"))
        .collect();
    assert_eq!(metadata_events.len(), 1);

    let rotate_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "controller_rotated"))
        .collect();
    assert_eq!(rotate_events.len(), 0);
}

#[test]
fn emits_only_controller_rotated_when_metadata_unchanged() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);
    let new_controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile-v1"));

    client.update_profile(
        &agent,
        &soroban_string(&env, "ipfs://profile-v1"),
        &Some(new_controller.clone()),
    );

    let events = env.events().all();
    let metadata_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "metadata_updated"))
        .collect();
    assert_eq!(metadata_events.len(), 0);

    let rotate_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "controller_rotated"))
        .collect();
    assert_eq!(rotate_events.len(), 1);
}

#[test]
fn emits_no_update_events_when_nothing_changes() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile-v1"));
    client.update_profile(&agent, &soroban_string(&env, "ipfs://profile-v1"), &None);

    let events = env.events().all();
    let metadata_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "metadata_updated"))
        .collect();
    assert_eq!(metadata_events.len(), 0);

    let rotate_events: Vec<_> = events
        .iter()
        .filter(|(_, topics, _)| event_has_topic(&env, topics, "controller_rotated"))
        .collect();
    assert_eq!(rotate_events.len(), 0);
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

// Panics with ProtocolError::MissingRecord when get_profile is called for an unknown agent.
#[test]
#[should_panic]
fn get_profile_rejects_unregistered_agent() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.get_profile(&agent);
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

    client.update_profile(&agent, &soroban_string(&env, "ipfs://profile-v2"), &None);
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

#[test]
fn reactivate_deactivated_agent_restores_active_bumps_revision_and_emits_event() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, (admin.clone(),));
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));
    client.deactivate(&agent);

    let deactivated = client.get_profile(&agent);
    assert!(!deactivated.active);
    assert_eq!(deactivated.revision, 1);

    // Clear events before reactivate
    let _ = env.events().all();

    // Reactivate
    client.reactivate(&agent);

    // Emits exactly one event
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let event = events.get_unchecked(0);
    assert_eq!(event.0, contract_id);
    let topic0: soroban_sdk::Symbol = event.1.get_unchecked(0).try_into_val(&env).unwrap();
    let topic1: Address = event.1.get_unchecked(1).try_into_val(&env).unwrap();
    assert_eq!(topic0, symbol_short!("react"));
    assert_eq!(topic1, agent);

    let reactivated = client.get_profile(&agent);
    assert!(reactivated.active);
    assert_eq!(reactivated.revision, 2);

    let data: AgentProfile = event.2.try_into_val(&env).unwrap();
    assert_eq!(data, reactivated);
}

#[test]
fn reactivating_already_active_agent_is_noop() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);

    let contract_id = env.register(IdentityContract, (admin.clone(),));
    let client = IdentityContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.register(&agent, &controller, &soroban_string(&env, "ipfs://profile"));

    let initial = client.get_profile(&agent);
    assert!(initial.active);
    assert_eq!(initial.revision, 0);

    // Clear events
    let _ = env.events().all();

    // Reactivate on already active agent
    client.reactivate(&agent);

    // No react event emitted
    let events = env.events().all();
    assert_eq!(events.len(), 0);

    let profile_after = client.get_profile(&agent);
    assert_eq!(profile_after, initial);
    assert_eq!(profile_after.revision, 0);
    assert!(profile_after.active);

    // Now deactivate, reactivate once, then test second reactivate is noop
    client.deactivate(&agent);
    let _ = env.events().all();
    client.reactivate(&agent);
    let events_first = env.events().all();
    assert_eq!(events_first.len(), 1);

    // Second reactivate call while active: leaves revision and stored profile unchanged
    client.reactivate(&agent);
    let events_second = env.events().all();
    assert_eq!(events_second.len(), 0);

    let profile_second = client.get_profile(&agent);
    assert_eq!(profile_second.revision, 2);
}
