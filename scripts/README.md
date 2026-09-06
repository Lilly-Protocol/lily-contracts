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

### `generate-manifest.sh`
Builds `dist/manifest.json` with sha256 hashes, workspace version, git commit, and build profile for each Wasm file in `dist/`.

**Usage:**
```bash
make artifacts

# Or run directly after copying Wasm files into dist/
./scripts/generate-manifest.sh

# Override output location or profile when needed
ARTIFACTS_DIR=/tmp/dist BUILD_PROFILE=release ./scripts/generate-manifest.sh
```

**Tests:**
```bash
make test-manifest                  # NIO-60 acceptance (no Rust required)
make verify                         # alias for test-manifest (process verify entry point)
npm run test                        # studio harness verify (detected over cargo test)
./scripts/verify.sh
./scripts/validate-artifacts-ci-wiring.sh
./scripts/check-nio-60-acceptance.sh
```

### `assert-contract-artifacts-bundle.sh`
Confirms `dist/` contains every file uploaded as `contract-artifacts` in CI (`*.wasm` + `manifest.json`).

### `test-artifacts-smoke.sh`
Strict end-to-end: requires `make artifacts` (contract Wasm). Fails if the workspace cannot build. For local manifest-only testing without a compiling workspace, use `test-artifacts-manifest-offline.sh` or set `ARTIFACTS_SMOKE_ALLOW_FALLBACK=1`.

```bash
make artifacts-smoke
```

### `test-artifacts-manifest-offline.sh`
Local dev only — manifest pipeline with rustc minimal Wasm. **Not run in CI.**

### `check-nio-60-acceptance.sh`
NIO-60 bounty acceptance entry point (no Rust required). Runtime-generates `manifest.json`, runs full test suite, validates CI wiring, and proves contract-artifacts bundle on generated `dist/`.

### `validate-artifacts-ci-wiring.sh`
Static validation that CI/release workflows wire `dist/manifest.json` into `contract-artifacts` uploads.

### `prove-contract-artifacts-runtime.sh`
Runtime validation of `dist/` after `make artifacts` — same checks CI runs before uploading `contract-artifacts`. No Rust required if `dist/` already exists.

```bash
make artifacts && make prove-contract-artifacts-runtime
```

### `verify-dist-manifest.sh`
Validates `dist/manifest.json` against `dist/*.wasm` (hashes, version, commit, profile, sorted package order). CI runs this after `make artifacts` and before uploading `contract-artifacts`.
