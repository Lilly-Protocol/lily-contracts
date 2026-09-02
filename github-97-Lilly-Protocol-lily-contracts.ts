import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { coin, coins, calculateFee, GasPrice } from '@cosmjs/stargate';
import { wasmExecute } from '@cosmjs/cosmwasm-stargate/build/signingcosmwasmclient';
import { MsgExecuteContract } from 'cosmjs-types/cosmwasm/wasm/v1/tx';
import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';

// Import contract artifacts
import paymentsArtifact from '../artifacts/lily_payments.wasm.json';
import protocolArtifact from '../artifacts/lily_protocol.wasm.json';

describe('Cross-Contract Config Reads: Payments ↔ Protocol', () => {
  let wallet: DirectSecp256k1HdWallet;
  let client: SigningCosmWasmClient;
  let admin: string;
  let paymentsContractAddr: string;
  let protocolContractAddr: string;

  const initFunds = coins(1000000, 'ustake');
  const gasPrice = GasPrice.fromString('0.025ustake');

  beforeAll(async () => {
    const rpcEndpoint = 'http://localhost:26657';
    wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      'test test test test test test test test test test test test',
      { hdPath: [44, 118, 0, 0, 0], prefix: 'lily' }
    );
    const [account] = await wallet.getAccounts();
    client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, {
      gasPrice,
    });
    admin = account.address;

    // Upload & instantiate payments contract
    const paymentsCodeId = await client.upload(
      admin,
      Buffer.from(JSON.parse(JSON.stringify(paymentsArtifact.wasm))),
      'lily-payments-v1'
    );
    const paymentsInitMsg = {
      admin: admin,
      fee_recipient: admin,
      protocol_address: '', // Will be set after protocol deployment
    };
    const paymentsInitFees = calculateFee(150000, '0.025ustake');
    ({ contractAddress: paymentsContractAddr } = await client.instantiate(
      admin,
      paymentsCodeId,
      paymentsInitMsg,
      'lily-payments',
      paymentsInitFees
    ));

    // Upload & instantiate protocol contract
    const protocolCodeId = await client.upload(
      admin,
      Buffer.from(JSON.parse(JSON.stringify(protocolArtifact.wasm))),
      'lily-protocol-v1'
    );
    const protocolInitMsg = {
      admin: admin,
      payments_address: paymentsContractAddr,
    };
    const protocolInitFees = calculateFee(150000, '0.025ustake');
    ({ contractAddress: protocolContractAddr } = await client.instantiate(
      admin,
      protocolCodeId,
      protocolInitMsg,
      'lily-protocol',
      protocolInitFees
    ));

    // Update payments contract with protocol address
    await client.execute(
      admin,
      paymentsContractAddr,
      { set_protocol_address: { protocol_address: protocolContractAddr } },
      paymentsInitFees
    );
  });

  afterAll(async () => {
    await client.disconnect();
  });

  it('should allow protocol contract to read payments config', async () => {
    const queryMsg = { get_fee_recipient: {} };
    const response = await client.queryContractSmart(paymentsContractAddr, queryMsg);
    expect(response).toHaveProperty('fee_recipient');
    expect(response.fee_recipient).toBe(admin);
  });

  it('should allow protocol contract to read payments admin', async () => {
    const queryMsg = { get_admin: {} };
    const response = await client.queryContractSmart(paymentsContractAddr, queryMsg);
    expect(response).toHaveProperty('admin');
    expect(response.admin).toBe(admin);
  });

  it('should handle cross-contract config updates correctly', async () => {
    const newAdmin = 'lily1newadmin';
    const updateMsg = { set_admin: { admin: newAdmin } };
    const updateFees = calculateFee(100000, '0.025ustake');

    await client.execute(admin, paymentsContractAddr, updateMsg, updateFees);

    const queryMsg = { get_admin: {} };
    const response = await client.queryContractSmart(paymentsContractAddr, queryMsg);
    expect(response.admin).toBe(newAdmin);
  });

  it('should return error when querying non-existent config key', async () => {
    const invalidQueryMsg = { get_nonexistent_key: {} };
    await expect(
      client.queryContractSmart(paymentsContractAddr, invalidQueryMsg)
    ).rejects.toThrow();
  });
});