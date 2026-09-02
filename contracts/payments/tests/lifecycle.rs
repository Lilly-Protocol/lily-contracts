//! End-to-end agent lifecycle: `register` -> `bind_wallet` -> `create_intent`
//! -> `settle_intent`, exercising the identity, wallet, and payments
//! contracts together.

use identity::{IdentityContract, IdentityContractClient};
use lily_common::PaymentStatus;
use lily_test_support::{soroban_string, test_address, test_env};
use payments::{PaymentIntent, PaymentsContract, PaymentsContractClient};
use soroban_sdk::symbol_short;
use wallet::{WalletContract, WalletContractClient};

#[test]
fn agent_lifecycle_register_bind_create_settle() {
    let env = test_env();
    let admin = test_address(&env);
    let treasury = test_address(&env);
    let agent = test_address(&env);
    let payee = test_address(&env);
    let controller = test_address(&env);
    let wallet = test_address(&env);

    // 1. Identity: register both agents in the registry.
    let identity_id = env.register(IdentityContract, (admin.clone(),));
    let identity = IdentityContractClient::new(&env, &identity_id);
    identity.initialize(&admin);
    identity.register(&agent, &controller, &soroban_string(&env, "ipfs://lifecycle/agent"));
    identity.register(&payee, &controller, &soroban_string(&env, "ipfs://lifecycle/payee"));
    assert!(identity.get_profile(&agent).active);
    assert!(identity.get_profile(&payee).active);

    // 2. Wallet: bind the payer agent to a settlement wallet.
    let wallet_id = env.register(WalletContract, (admin.clone(),));
    let wallet_client = WalletContractClient::new(&env, &wallet_id);
    wallet_client.initialize(&admin);
    wallet_client.bind_wallet(&agent, &wallet, &symbol_short!("USDC"), &10_000_i128);
    let binding = wallet_client.get_binding(&agent);
    assert_eq!(binding.wallet, wallet);
    assert!(binding.enabled);

    // 3. Payments: create an intent from the registered agent to the payee.
    let payments_id = env.register(PaymentsContract, (admin.clone(),));
    let payments = PaymentsContractClient::new(&env, &payments_id);
    payments.initialize(&admin, &treasury, &250_u32);
    let intent_id = payments.create_intent(
        &agent,
        &payee,
        &250_i128,
        &soroban_string(&env, "agent-to-payee service fee"),
    );

    // Cross-contract state: the intent references the registry identities
    // and the payment is pending while both upstream contracts are intact.
    let intent = payments.get_intent(&intent_id);
    assert_eq!(
        intent,
        PaymentIntent {
            id: intent_id,
            payer_agent: agent.clone(),
            payee_agent: payee.clone(),
            amount: 250,
            memo: soroban_string(&env, "agent-to-payee service fee"),
            settlement_reference: soroban_string(&env, ""),
            status: PaymentStatus::Pending,
        }
    );
    assert!(identity.get_profile(&agent).active);
    assert!(identity.get_profile(&payee).active);
    assert!(wallet_client.get_binding(&agent).enabled);

    // 4. Settlement flips only the intent; upstream contracts are untouched.
    payments.settle_intent(&intent_id, &soroban_string(&env, "tx-lifecycle-0001"));
    let settled = payments.get_intent(&intent_id);
    assert_eq!(settled.status, PaymentStatus::Settled);
    assert_eq!(settled.settlement_reference, soroban_string(&env, "tx-lifecycle-0001"));

    let profile = identity.get_profile(&agent);
    assert!(profile.active);
    assert_eq!(profile.revision, 0);
    let binding_after = wallet_client.get_binding(&agent);
    assert!(binding_after.enabled);
    assert_eq!(binding_after.revision, 0);
}
