# Lilly Contracts Tooling & Scripts

Utility scripts for developing, inspecting, and deploying Lilly Protocol contracts.

## Scripts Overview

### `check-tooling.sh`
Verifies that local compiler toolchains, Rust targets (`wasm32v1-none`), and Stellar CLI are properly installed.

```sh
./scripts/check-tooling.sh
```

### `rpc-health.sh`
Probes Soroban RPC health and ledger status via JSON-RPC `getHealth` and `getLatestLedger`. Exits non-zero if the endpoint is unreachable or degraded.

```sh
# Probe local testnet/quickstart RPC (default: http://localhost:8000/soroban/rpc)
./scripts/rpc-health.sh

# Probe custom testnet RPC
SOROBAN_RPC_URL="https://soroban-testnet.stellar.org" ./scripts/rpc-health.sh
```

### `deploy.sh`
Automates deployment of all four contracts (`protocol`, `identity`, `wallet`, `payments`) in topological dependency order. Emits structured JSON deployment manifests and supports simulated dry-runs.

```sh
# Test deployment in dry-run mode
./scripts/deploy.sh --dry-run

# Deploy to testnet with custom source identity
./scripts/deploy.sh --network=testnet --source=alice --output=deployed-contracts.json
```
