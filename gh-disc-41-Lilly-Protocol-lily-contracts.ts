import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { calculateFee, GasPrice } from "@cosmjs/stargate";
import { sha256 } from "@cosmjs/crypto";
import { encodeBase64 } from "@cosmjs/encoding";

// Helper to assert ledger storage key encoding matches expected format
function assertLedgerStorageKey(key: Uint8Array, expectedPrefix: string, expectedHash: string): void {
  const keyStr = encodeBase64(key);
  // Ledger storage keys in CosmWasm typically use keccak256 for user data keys
  // but for contract state, keys are prefixed with contract address hash + user key hash
  // We assert the key starts with the expected prefix and contains the expected hash
  expect(keyStr.startsWith(expectedPrefix)).toBe(true);
  expect(keyStr.includes(expectedHash)).toBe(true);
}

// Example test case (to be integrated into existing test suite)
describe("Ledger Storage Key Encoding", () => {
  let client: SigningCosmWasmClient;
  let wallet: DirectSecp256k1HdWallet;
  let signer: string;

  beforeAll(async () => {
    wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      "test test test test test test test test test test test test test test test test test test test test test test test test",
      { hdPaths: [0] }
    );
    const rpcEndpoint = "http://localhost:1317"; // Replace with testnet or local node
    client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, {
      gasPrice: GasPrice.fromString("0.025uatom"),
    });
    const accounts = await wallet.getAccounts();
    signer = accounts[0].address;
  });

  it("should encode and assert ledger storage key for contract state", async () => {
    // Upload and instantiate contract (simplified for illustration)
    const wasm = require("./../artifacts/contract.wasm");
    const codeId = await client.upload(signer, wasm, calculateFee(1500000));
    const contractAddr = await client.instantiate(
      signer,
      codeId,
      { init_msg: {} },
      "test-contract",
      calculateFee(2000000)
    );

    // Simulate storing data and reading back the storage key
    const key = new TextEncoder().encode("user_data");
    const keyHash = sha256(key);
    const storageKey = new Uint8Array([
      ...new Uint8Array(32), // contract address prefix (simplified)
      ...keyHash,
    ]);

    // Assert the storage key is correctly encoded
    assertLedgerStorageKey(storageKey, "contract_state_prefix", encodeBase64(keyHash));
  });
});