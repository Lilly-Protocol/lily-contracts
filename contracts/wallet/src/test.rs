#![cfg(test)]

use soroban_sdk::symbol_short;
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

use super::{WalletBinding, WalletContract, WalletContractClient};
use lily_test_support::{test_address, test_env};

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
#[should_panic]
fn initialize_rejects_admin_other_than_deployer() {
    let env = test_env();
    let deployer_admin = test_address(&env);
    let front_runner = test_address(&env);

    let contract_id = env.register(WalletContract, (deployer_admin,));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&front_runner);
}

#[test]
#[should_panic]
fn update_spend_limit_rejects_unbound_agent() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.update_spend_limit(&agent, &500_i128);
}

#[test]
#[should_panic]
fn set_enabled_rejects_unbound_agent() {
    let env = test_env();
    let admin = test_address(&env);
    let agent = test_address(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.set_enabled(&agent, &false);
}

#[test]
fn bind_wallet_succeeds_with_dual_signatures() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let wallet = Address::generate(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "initialize",
                args: (&admin,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .initialize(&admin);

    client
        .mock_auths(&[
            MockAuth {
                address: &agent,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "bind_wallet",
                    args: (&agent, &wallet, symbol_short!("USDC"), 1_000_i128).into_val(&env),
                    sub_invokes: &[],
                },
            },
            MockAuth {
                address: &wallet,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "bind_wallet",
                    args: (&agent, &wallet, symbol_short!("USDC"), 1_000_i128).into_val(&env),
                    sub_invokes: &[],
                },
            },
        ])
        .bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);

    let binding = client.get_binding(&agent);
    assert_eq!(binding.wallet, wallet);
    assert!(binding.enabled);
}

#[test]
#[should_panic]
fn bind_wallet_rejects_missing_wallet_signature() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let wallet = Address::generate(&env);

    let contract_id = env.register(WalletContract, (admin.clone(),));
    let client = WalletContractClient::new(&env, &contract_id);

    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "initialize",
                args: (&admin,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .initialize(&admin);

    // Only the agent signs: the wallet signature is never provided, so the
    // second `require_auth` in `bind_wallet` must fail.
    client
        .mock_auths(&[MockAuth {
            address: &agent,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "bind_wallet",
                args: (&agent, &wallet, symbol_short!("USDC"), 1_000_i128).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &1_000_i128);
}
