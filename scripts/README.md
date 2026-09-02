# Utility Scripts

This directory contains utility scripts for development, continuous integration, and operations with Lily Protocol contracts.

## Available Scripts

### `rpc-health.sh`
Probes a target Soroban RPC endpoint to verify node health and query the latest ledger sequence.

**Usage:**
```bash
# Using default testnet endpoint
./scripts/rpc-health.sh

# Specifying a custom RPC endpoint via argument
./scripts/rpc-health.sh https://soroban-testnet.stellar.org

# Specifying via environment variable
SOROBAN_RPC_URL="http://localhost:8000/soroban/rpc" ./scripts/rpc-health.sh
```

**Exit codes:**
- `0`: RPC endpoint is online, returns `healthy`, and successfully yields the latest ledger sequence.
- `1`: Endpoint is unreachable, unhealthy, timed out, or returned an RPC error.

### `check-tooling.sh`
Verifies that all required build and test tooling (`cargo`, `stellar-cli`, `rustfmt`, `clippy`) is installed and matches project expectations.
