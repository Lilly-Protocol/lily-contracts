import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { calculateFee, GasPrice } from "@cosmjs/stargate";
import { Coin } from "@cosmjs/proto-signing";
import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { instantiateContract, uploadContract, executeContract, queryContract } from "@lilly-protocol/test-helpers";

describe("Cross-contract config reads: payments ↔ protocol", () => {
  let wallet: DirectSecp256k1HdWallet;
  let client: SigningCosmWasmClient;
  let admin: string;
  let paymentsAddr: string;
  let protocolAddr: string;

  const denom = "ustake";
  const feeRate = "0.005";

  beforeAll(async () => {
    const rpcEndpoint = "http://localhost:1317";
    wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      "test test test test test test test test test test test test",
      { prefix: "lily" }
    );
    const accounts = await wallet.getAccounts();
    admin = accounts[0].address;
    client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, {
      gasPrice: GasPrice.fromString("0.025ustake"),
    });
  });

  afterAll(async () => {
    await client.disconnect();
  });

  it("uploads protocol contract and initializes config", async () => {
    const wasm = await uploadContract(client, admin, "./artifacts/lily_protocol.wasm");
    const initMsg = {
      admin,
      fee_rate: feeRate,
    };
    protocolAddr = await instantiateContract(client, admin, wasm, initMsg, "protocol");
    expect(protocolAddr).toMatch(/^lily/);
  });

  it("uploads payments contract with protocol address dependency", async () => {
    const wasm = await uploadContract(client, admin, "./artifacts/lily_payments.wasm");
    const initMsg = {
      admin,
      protocol_addr: protocolAddr,
    };
    paymentsAddr = await instantiateContract(client, admin, wasm, initMsg, "payments");
    expect(paymentsAddr).toMatch(/^lily/);
  });

  it("reads protocol config (fee_rate) from payments contract", async () => {
    const queryMsg = { get_protocol_config: {} };
    const res = await queryContract(client, paymentsAddr, queryMsg);
    expect(res).toEqual({
      protocol_addr: protocolAddr,
      fee_rate: feeRate,
    });
  });

  it("returns error when protocol config is missing", async () => {
    // Deploy a fresh protocol contract without proper config
    const wasm = await uploadContract(client, admin, "./artifacts/lily_protocol.wasm");
    const badProtocolAddr = await instantiateContract(client, admin, wasm, {}, "bad_protocol");
    
    // Update payments to point to bad protocol
    await executeContract(client, admin, paymentsAddr, {
      update_config: {
        protocol_addr: badProtocolAddr,
      },
    });

    // Query should fail or return default/empty config
    const queryMsg = { get_protocol_config: {} };
    const res = await queryContract(client, paymentsAddr, queryMsg);
    expect(res.fee_rate).toBeUndefined();
  });
});