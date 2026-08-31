#![allow(clippy::unwrap_used, clippy::expect_used)]
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
fn admin_transfer_updates_config_revokes_old_and_emits_event() {
    use soroban_sdk::testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke};
    use soroban_sdk::{
        symbol_short, Address as SdkAddress, IntoVal, Symbol as SdkSymbol, TryIntoVal,
        Val as SdkVal,
    };

    let env = soroban_sdk::Env::default();
    let admin = soroban_sdk::Address::generate(&env);
    let new_admin = soroban_sdk::Address::generate(&env);
    let treasury = soroban_sdk::Address::generate(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "initialize",
            args: (&admin, &treasury, &250_u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &treasury, &250_u32);

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "transfer_admin",
            args: (&new_admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.transfer_admin(&new_admin);

    // 1) the emitted event: exactly one, "admin" marker topic, new admin payload.
    let events = env.events().all();
    assert_eq!(events.len(), 1, "transfer_admin must publish exactly one event");
    let (_contract, topics, data): (SdkAddress, soroban_sdk::Vec<SdkVal>, SdkVal) =
        events.get(0).unwrap();
    let admin_marker: SdkSymbol = symbol_short!("admin");
    let found = (0..topics.len()).any(|i| {
        let topic: SdkSymbol = topics.get(i).unwrap().try_into_val(&env).unwrap();
        topic == admin_marker
    });
    assert!(found, "event topics must contain the 'admin' marker");
    let data_addr: SdkAddress = data.try_into_val(&env).unwrap();
    assert_eq!(data_addr, new_admin);

    // 2) get_config reflects the new admin.
    let config = client.get_config();
    assert_eq!(config.admin, new_admin);

    // 3) the old admin is revoked: only their authorization entry exists, so
    // the stored admin's (now new_admin) require_auth check fails.
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_treasury",
            args: (&treasury,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    let outcome = client.try_set_treasury(&treasury);
    assert!(outcome.is_err(), "old admin must no longer be authorized after transfer");

    // and the new admin can still act.
    env.mock_auths(&[MockAuth {
        address: &new_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_fee_bps",
            args: (&300_u32,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_fee_bps(&300_u32);
    assert_eq!(client.get_config().fee_bps, 300);
}

#[test]
#[should_panic]
fn admin_transfer_rejects_non_admin_caller() {
    use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
    use soroban_sdk::IntoVal;

    let env = soroban_sdk::Env::default();
    let admin = soroban_sdk::Address::generate(&env);
    let treasury = soroban_sdk::Address::generate(&env);
    let outsider = soroban_sdk::Address::generate(&env);
    let replacement = soroban_sdk::Address::generate(&env);

    let contract_id = env.register(ProtocolContract, ());
    let client = ProtocolContractClient::new(&env, &contract_id);

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "initialize",
            args: (&admin, &treasury, &250_u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &treasury, &250_u32);

    // Only the outsider's authorization entry exists now, so the stored
    // admin's require_auth() check fails.
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "transfer_admin",
            args: (&replacement,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.transfer_admin(&replacement);
}
