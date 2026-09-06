use lily_test_support::{test_address, test_env};
use payments::{PaymentsContract, PaymentsContractClient};
use protocol::{ProtocolContract, ProtocolContractClient};

#[test]
fn cross_contract_config_parity_and_independent_control() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let wallet = test_address(&env);
    let new_admin = test_address(&env);

    let protocol_id = env.register(ProtocolContract, (admin.clone(),));
    let payments_id = env.register(PaymentsContract, (admin.clone(),));

    let protocol_client = ProtocolContractClient::new(&env, &protocol_id);
    let payments_client = PaymentsContractClient::new(&env, &payments_id);

    protocol_client.initialize(&admin, &treasury, &100_u32);
    payments_client.initialize(&admin, &treasury, &100_u32, &wallet);

    let protocol_config = protocol_client.get_config();
    let payments_config = payments_client.get_config();

    assert_eq!(protocol_config.admin, admin);
    assert_eq!(payments_config.admin, admin);
    assert_eq!(protocol_config.treasury, treasury);
    assert_eq!(payments_config.treasury, treasury);
    assert_eq!(protocol_config.fee_bps, 100);
    assert_eq!(payments_config.fee_bps, 100);
    assert_eq!(payments_config.wallet, wallet);

    protocol_client.transfer_admin(&new_admin);
    protocol_client.accept_admin();

    let protocol_config_after = protocol_client.get_config();
    let payments_config_after = payments_client.get_config();

    assert_eq!(protocol_config_after.admin, new_admin);
    assert_eq!(payments_config_after.admin, admin);
    assert_eq!(protocol_config_after.treasury, treasury);
    assert_eq!(payments_config_after.treasury, treasury);
    assert_eq!(protocol_config_after.fee_bps, 100);
    assert_eq!(payments_config_after.fee_bps, 100);
}

#[test]
fn independent_fee_updates() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let wallet = test_address(&env);

    let protocol_id = env.register(ProtocolContract, (admin.clone(),));
    let payments_id = env.register(PaymentsContract, (admin.clone(),));

    let protocol_client = ProtocolContractClient::new(&env, &protocol_id);
    let payments_client = PaymentsContractClient::new(&env, &payments_id);

    protocol_client.initialize(&admin, &treasury, &100_u32);
    payments_client.initialize(&admin, &treasury, &100_u32, &wallet);

    protocol_client.set_fee_bps(&200_u32);

    assert_eq!(protocol_client.get_config().fee_bps, 200);
    assert_eq!(payments_client.get_config().fee_bps, 100);
}
