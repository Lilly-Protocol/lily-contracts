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
fn rejects_binding_when_any_binding_exists() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);
    let wallet = test_address(&env);
    let wallet2 = test_address(&env);

    let contract_id = env.register(WalletContract, ());
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

    let contract_id = env.register(WalletContract, ());
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

    let contract_id = env.register(WalletContract, ());
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

    let contract_id = env.register(WalletContract, ());
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

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &0_i128);
}
