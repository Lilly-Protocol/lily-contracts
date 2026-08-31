#![cfg(test)]

use soroban_sdk::symbol_short;

use super::{WalletBinding, WalletContract, WalletContractClient};
use lily_test_support::{test_address, test_env};

#[test]
fn binds_wallet_and_updates_policy() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, ());
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

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &100_i128);
}

#[test]
#[should_panic]
fn rejects_zero_spend_limit() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &0_i128);
}
#[test]
fn preserves_revision_on_rebind_after_disable() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);
    let new_wallet = test_address(&env);

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);
    client.update_spend_limit(&agent, &2_000_i128);
    client.set_enabled(&agent, &false);

    // Re-binding after disabling must preserve revision continuity.
    client.bind_wallet(&agent, &new_wallet, &symbol_short!("XLM"), &5_000_i128);

    let rebound = client.get_binding(&agent);
    assert!(rebound.enabled);
    assert_eq!(rebound.wallet, new_wallet);
    assert_eq!(rebound.settlement_asset, symbol_short!("XLM"));
    assert_eq!(rebound.spend_limit, 5_000);
    // Original bind = 0, update = 1, disable = 2, rebind = 3.
    assert_eq!(rebound.revision, 3);
}

#[test]
