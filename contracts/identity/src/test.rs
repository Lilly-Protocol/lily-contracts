#![cfg(test)]

use super::{AgentProfile, IdentityContract, IdentityContractClient};
use lily_test_support::{soroban_string, test_address, test_env};

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
#[should_panic(expected = "Error(Contract, #4)")]
fn deactivated_agent_cannot_update_profile_active_flag_verified() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);
    let uri = soroban_string(&env, "ipfs://profile-1");

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.register(&agent, &controller, &uri);

    // active flag verified BEFORE deactivation...
    assert!(client.get_profile(&agent).active);

    client.deactivate(&agent);
    // ...and AFTER.
    assert!(!client.get_profile(&agent).active);

    // update_profile then panics InvalidInput (Error(Contract, #4)).
    let uri2 = soroban_string(&env, "ipfs://profile-2");
    client.update_profile(&agent, &uri2, &None::<soroban_sdk::Address>);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn profile_revision_overflow_panics_typed_error() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let controller = test_address(&env);
    let uri = soroban_string(&env, "ipfs://profile-1");
    let uri2 = soroban_string(&env, "ipfs://profile-2");

    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.register(&agent, &controller, &uri);

    // Force the revision counter to its maximum so the next bump overflows.
    let mut profile = client.get_profile(&agent);
    profile.revision = u64::MAX;
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&super::DataKey::Profile(agent.clone()), &profile);
    });

    client.update_profile(&agent, &uri2, &None::<soroban_sdk::Address>);
}

// #38: property tests over unbounded String inputs (multibyte, long, empty).
//
// Strategy space (= the documented corpus/labels): identity URIs use
// (len 1..=4096 chars, seed u64); payment memos use (len 0..=4096, seed u64).
// Generated values mix 1-, 2-, 3- and 4-byte UTF-8 characters. Proptest
// writes failing cases into the crate's `proptest-regressions/` corpus for
// shrinking/replay (see TESTING.md, "Property tests").
mod prop38 {
    extern crate std;

    use super::*;
    use proptest::prelude::*;
    use proptest::property_test;
    use std::string::String;

    fn seed_string(env: &soroban_sdk::Env, len: u32, seed: u64) -> soroban_sdk::String {
        let alphabet: [char; 4] = ['a', '\u{e9}', '\u{4e2d}', '\u{1F600}'];
        let mut s = String::with_capacity(len as usize);
        let mut state = seed.max(1);
        for _ in 0..len {
            state = state.wrapping_mul(6_364_136_223_846_793_005u64).wrapping_add(1_442_695_040_888_963_407u64);
            s.push(alphabet[((state >> 33) % 4) as usize]);
        }
        soroban_string(env, &s)
    }

    #[property_test]
    fn identity_accepts_arbitrary_uris_roundtrip(
        #[strategy = 1u32..=4096u32] len: u32,
        #[strategy = any::<u64>()] seed: u64,
    ) {
        let env = test_env();
        let admin = test_address(&env);
        let agent = test_address(&env);
        let controller = test_address(&env);
        let contract_id = env.register(IdentityContract, ());
        let client = IdentityContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        let uri = seed_string(&env, len, seed);
        prop_assert!(client.try_register(&agent, &controller, &uri).is_ok());
        prop_assert_eq!(client.get_profile(&agent).metadata_uri, uri);

        let uri2 = seed_string(&env, len.saturating_add(1), seed.wrapping_add(97));
        prop_assert!(client
            .try_update_profile(&agent, &uri2, &None::<soroban_sdk::Address>)
            .is_ok());
        prop_assert_eq!(client.get_profile(&agent).metadata_uri, uri2);
    }
}
