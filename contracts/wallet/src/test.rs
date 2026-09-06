#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(test)]

use soroban_sdk::symbol_short;
use soroban_sdk::testutils::Events;

use super::{SCHEMA_VERSION, WalletBinding, WalletContract, WalletContractClient};
use lily_common::PROTOCOL_VERSION;
use lily_test_support::{test_address, test_env};

#[test]
fn returns_protocol_version() {
    let env = test_env();
    let admin = test_address(&env);
    let contract_id = env.register(WalletContract, (admin,));
    let client = WalletContractClient::new(&env, &contract_id);

    assert_eq!(client.version(), PROTOCOL_VERSION);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn rejects_unpinned_admin_initialization() {
    let env = test_env();
    let deployer = test_address(&env);
    let other = test_address(&env);

    let contract_id = env.register(WalletContract, (deployer,));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&other);
}

#[test]
fn binds_wallet_and_updates_policy() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);

    let binding = client.get_binding(&agent);
    assert_eq!(
        binding,
        WalletBinding {
            wallet: wallet.clone(),
            settlement_asset: symbol_short!("USDC"),
            spend_limit: 1_000,
            enabled: true,
            admin_locked: false,
            revision: 0,
        }
    );

    client.update_spend_limit(&agent, &2_500_i128);
    client.set_enabled(&agent, &false);

    let updated = client.get_binding(&agent);
    assert_eq!(updated.spend_limit, 2_500);
    assert!(!updated.enabled);
    assert_eq!(updated.revision, 2);
}

#[test]
#[should_panic]
fn rejects_double_binding_while_active() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
}

#[test]
#[should_panic]
fn rejects_binding_when_any_binding_exists() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);
    let wallet2 = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
    client.set_enabled(&agent, &false);

    // bind_wallet must fail even when the existing binding is disabled.
    client.bind_wallet(&agent, &wallet2, &symbol_short!("USDC"), &200_i128);
}

#[test]
fn rebinds_disabled_binding_explicitly() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);
    let new_wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
    client.set_enabled(&agent, &false);

    client.rebind_wallet(&agent, &new_wallet, &symbol_short!("XLM"), &500_i128);

    let binding = client.get_binding(&agent);
    assert_eq!(binding.wallet, new_wallet);
    assert_eq!(binding.settlement_asset, symbol_short!("XLM"));
    assert_eq!(binding.spend_limit, 500);
    assert!(binding.enabled);
    assert_eq!(binding.revision, 0);
}

#[test]
fn rebinds_enabled_binding_explicitly() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);
    let new_wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
    client.update_spend_limit(&agent, &200_i128);
    assert_eq!(client.get_binding(&agent).revision, 1);

    client.rebind_wallet(&agent, &new_wallet, &symbol_short!("USDC"), &1_000_i128);

    let binding = client.get_binding(&agent);
    assert_eq!(binding.wallet, new_wallet);
    assert_eq!(binding.spend_limit, 1_000);
    assert!(binding.enabled);
    assert_eq!(binding.revision, 0);
}

#[test]
#[should_panic]
fn rejects_rebind_without_existing_binding() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.rebind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
}

#[test]
#[should_panic]
fn rejects_zero_spend_limit() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &0_i128);
}

#[test]
fn admin_can_deactivate_wallet_binding() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);
    client.admin_deactivate(&agent);

    let binding = client.get_binding(&agent);
    assert!(!binding.enabled);
    assert!(binding.admin_locked);
    assert_eq!(binding.revision, 1);
}

#[test]
#[should_panic]
fn agent_cannot_undo_admin_deactivate_via_set_enabled() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);
    client.admin_deactivate(&agent);

    // Must panic with a typed ProtocolError: the admin lock blocks this.
    client.set_enabled(&agent, &true);
}

#[test]
fn agent_can_still_self_disable_after_admin_deactivate() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);

    // Before admin_deactivate: agent can freely disable.
    client.set_enabled(&agent, &false);
    assert!(!client.get_binding(&agent).enabled);
    client.set_enabled(&agent, &true);
    assert!(client.get_binding(&agent).enabled);

    client.admin_deactivate(&agent);

    // After admin_deactivate: agent can still disable (no-op on `enabled`,
    // but must not panic), just not re-enable.
    client.set_enabled(&agent, &false);
    let binding = client.get_binding(&agent);
    assert!(!binding.enabled);
    assert!(binding.admin_locked);
}

#[test]
fn admin_reactivate_clears_lock_and_bumps_once() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);
    client.admin_deactivate(&agent);
    let locked = client.get_binding(&agent);
    assert_eq!(locked.revision, 1);

    let events_before = env.events().all().len();
    client.admin_reactivate(&agent);
    let events_after = env.events().all().len();
    assert_eq!(events_after - events_before, 1, "admin_reactivate must emit exactly one event");

    let binding = client.get_binding(&agent);
    assert!(binding.enabled);
    assert!(!binding.admin_locked);
    assert_eq!(binding.revision, 2, "admin_reactivate must bump revision exactly once");

    // The agent regains normal set_enabled control after the lock clears.
    client.set_enabled(&agent, &false);
    client.set_enabled(&agent, &true);
    assert!(client.get_binding(&agent).enabled);
}

#[test]
fn rebind_wallet_clears_admin_lock() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);
    let new_wallet = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);
    client.admin_deactivate(&agent);

    client.rebind_wallet(&agent, &new_wallet, &symbol_short!("USDC"), &500_i128);

    let binding = client.get_binding(&agent);
    assert!(binding.enabled);
    assert!(!binding.admin_locked);
    assert_eq!(binding.revision, 0);

    // Fully enabled means the agent can immediately manage it again.
    client.set_enabled(&agent, &false);
    client.set_enabled(&agent, &true);
    assert!(client.get_binding(&agent).enabled);
}
